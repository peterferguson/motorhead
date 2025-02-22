use actix_web::{delete, error, get, post, web, HttpResponse, Responder};
use std::sync::Arc;
use tokio;

use crate::models::{
    AckResponse, AppState, MemoryMessage, MemoryMessagesAndContext, MemoryResponse,
};
use crate::reducer::handle_compaction;

#[get("/sessions/{session_id}/memory")]
pub async fn get_memory(
    session_id: web::Path<String>,
    data: web::Data<Arc<AppState>>,
    redis: web::Data<redis::Client>,
) -> actix_web::Result<impl Responder> {
    let mut conn = redis
        .get_tokio_connection_manager()
        .await
        .map_err(error::ErrorInternalServerError)?;

    let lrange_key = &*session_id;
    let context_key = format!("{}_context", &*session_id);
    let token_count_key = format!("{}_tokens", &*session_id);
    let keys = vec![context_key, token_count_key];

    let (messages, values): (Vec<String>, Vec<Option<String>>) = redis::pipe()
        .cmd("LRANGE")
        .arg(lrange_key)
        .arg(0)
        .arg(data.window_size as isize)
        .cmd("MGET")
        .arg(keys)
        .query_async(&mut conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let context = values.get(0).cloned().flatten();
    let tokens = values
        .get(1)
        .cloned()
        .flatten()
        .and_then(|tokens_string| tokens_string.parse::<i64>().ok())
        .unwrap_or(0);

    let messages: Vec<MemoryMessage> = messages
        .into_iter()
        .filter_map(|message| {
            let mut parts = message.splitn(2, ": ");
            match (parts.next(), parts.next()) {
                (Some(role), Some(content)) => Some(MemoryMessage {
                    role: role.to_string(),
                    content: content.to_string(),
                }),
                _ => None,
            }
        })
        .collect();

    let response = MemoryResponse {
        messages,
        context,
        tokens: Some(tokens),
    };

    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(response))
}

#[post("/sessions/{session_id}/memory")]
pub async fn post_memory(
    session_id: web::Path<String>,
    web::Json(memory_messages): web::Json<MemoryMessagesAndContext>,
    data: web::Data<Arc<AppState>>,
    redis: web::Data<redis::Client>,
) -> actix_web::Result<impl Responder> {
    let mut conn = redis
        .get_tokio_connection_manager()
        .await
        .map_err(error::ErrorInternalServerError)?;

    let messages: Vec<String> = memory_messages
        .messages
        .into_iter()
        .map(|memory_message| format!("{}: {}", memory_message.role, memory_message.content))
        .collect();

    if let Some(context) = memory_messages.context {
        redis::Cmd::set(format!("{}_context", &*session_id), context)
            .query_async::<_, ()>(&mut conn)
            .await
            .map_err(error::ErrorInternalServerError)?;
    }

    let res: i64 = redis::Cmd::lpush(&*session_id, messages)
        .query_async::<_, i64>(&mut conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    if res > data.window_size {
        let state = data.into_inner();
        let mut session_cleanup = state.session_cleanup.lock().await;

        if !session_cleanup.get(&*session_id).unwrap_or_else(|| &false) {
            session_cleanup.insert((&*session_id.to_string()).into(), true);
            let session_cleanup = Arc::clone(&state.session_cleanup);
            let session_id = session_id.clone();
            let state_clone = Arc::clone(&state);

            tokio::spawn(async move {
                log::info!("running compact");
                let _compaction_result =
                    handle_compaction(session_id.to_string(), state_clone, conn).await;

                let mut lock = session_cleanup.lock().await;
                lock.remove(&session_id);
            });
        }
    }

    let response = AckResponse { status: "Ok" };
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(response))
}

#[delete("/sessions/{session_id}/memory")]
pub async fn delete_memory(
    session_id: web::Path<String>,
    redis: web::Data<redis::Client>,
) -> actix_web::Result<impl Responder> {
    let mut conn = redis
        .get_tokio_connection_manager()
        .await
        .map_err(error::ErrorInternalServerError)?;

    let context_key = format!("{}_context", &*session_id);
    let token_count_key = format!("{}_tokens", &*session_id);
    let session_key = format!("{}", &*session_id);
    let keys = vec![context_key, session_key, token_count_key];

    redis::Cmd::del(keys)
        .query_async(&mut conn)
        .await
        .map_err(error::ErrorInternalServerError)?;

    let response = AckResponse { status: "Ok" };
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .json(response))
}
