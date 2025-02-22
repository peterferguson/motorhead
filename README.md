<h1 align="center" >
Motörhead
</h1>
<p align="center">
    <a href="https://github.com/getmetal/motorhead/blob/main/LICENSE">
    <img src="https://img.shields.io/github/license/getmetal/motorhead?style=flat&label=license&logo=github&color=4f46e5&logoColor=fff" alt="License" />
    </a>
  <a href="https://twitter.com/Metal_io">
    <img src="https://img.shields.io/badge/metal-message?style=flat&logo=twitter&color=4f46e5&logoColor=#4f46e5" alt="Metal" style="margin-right:3px" />
  </a>
  <a href="https://discord.gg/GHY3Y8tU3J">
    <img src="https://dcbadge.vercel.app/api/server/GHY3Y8tU3J?compact=true&style=flat" alt="License" />
  </a>
</p>

Motörhead is a memory and information retrieval server for LLMs.

[![Deploy on Railway](https://railway.app/button.svg)](https://railway.app/template/bmO_uf?referralCode=5NirXf)

## Why use Motörhead?

When building chat applications using LLMs, memory handling is something that  has to be built every time. Motörhead is a server to assist with that process. It provides 3 simple APIs:

- GET `/sessions/:id/memory` returns messages up to `MAX_WINDOW_SIZE`.
```json
{
    "messages": [
        {
            "role": "AI",
            "content": "Electronic music and salsa are two very different genres of music, and the way people dance to them is also quite different."
        },
        {
            "role": "Human",
            "content": "how does it compare to salsa?"
        },
        {
            "role": "AI",
            "content": "Electronic music is a broad genre that encompasses many different styles, so there is no one \"right\" way to dance to it."
        },
        {
            "role": "Human",
            "content": "how do you dance electronic music?"
        },
        {
            "role": "AI",
            "content": "Colombia has a vibrant electronic music scene, and there are many talented DJs and producers who have gained international recognition."
        },
        {
            "role": "Human",
            "content": "What are some famous djs from Colombia?"
        },
        {
            "role": "AI",
            "content": "Baum opened its doors in 2014 and has quickly become one of the most popular clubs for electronic music in Bogotá."
        }
    ],
    "context": "The conversation covers topics such as clubs for electronic music in Bogotá, popular tourist attractions in the city, and general information about Colombia. The AI provides information about popular electronic music clubs such as Baum and Video Club, as well as electronic music festivals that take place in Bogotá. The AI also recommends tourist attractions such as La Candelaria, Monserrate and the Salt Cathedral of Zipaquirá, and provides general information about Colombia's diverse culture, landscape and wildlife.",
    "tokens": 744 // tokens used for incremental summarization
}
```

- POST `/sessions/:id/memory` - you can send multiple messages to Motorhead to store.

```bash
curl --location 'localhost:8080/sessions/${SESSION_ID}/memory' \
--header 'Content-Type: application/json' \
--data '{
    "messages": [{ "role": "Human", "content": "ping" }, { "role": "AI", "content": "pong" }]
}'
```

Optionally, `context` can be send in if it needs to get loaded from another datastore.

- DELETE `/sessions/:id/memory` - deletes the session's message list.

A max `window_size` is set for the LLM to keep track of the conversation. Once that max is hit, Motörhead will process (`window_size  / 2` messages) and summarize them. Subsequent summaries, as the messages grow, are incremental.

## Config

- `MOTORHEAD_MAX_WINDOW_SIZE` (default:12) - Number of max messages returned by the server. When this number is reached, a job is triggered to halve it.
- `MOTORHEAD_PORT` (default:8000) - Motörhead Server Port
- `OPENAI_API_KEY` (required)- Number of max messages returned by the server. When this number is reached, a job is triggered to halve it.
- `REDIS_URL` (required)- URL used to connect to `redis`.

## How to run

With docker-compose:
```bash
docker-compose build && docker-compose up
```

Or you can use the image `docker pull ghcr.io/getmetal/motorhead:latest` directly:
```bash
docker run --name motorhead -p 8080:8080 -e MOTORHEAD_PORT=8080 -e REDIS_URL='redis://redis:6379' -d ghcr.io/getmetal/motorhead:latest
```

## Examples

- Check out our [Chat JS Example](examples/chat-js/)
