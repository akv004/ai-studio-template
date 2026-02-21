"""
Ollama Provider
===============
Local LLM provider using Ollama.
Runs on your local machine with GPU acceleration.
"""

import json
import httpx
from typing import AsyncGenerator, Optional
from .base import AgentProvider, Message, ChatResponse


class OllamaProvider(AgentProvider):
    """
    Ollama provider for local LLM inference.

    Requires Ollama running locally or via Docker:
    - Local: `ollama serve`
    - Docker: `docker run -d --gpus all -p 11434:11434 ollama/ollama`
    """

    name = "ollama"

    def __init__(
        self,
        base_url: str = "http://localhost:11434",
        default_model: str = "llama3.2",
        timeout: float = 120.0,
    ):
        self.base_url = base_url.rstrip("/")
        self.default_model = default_model
        self.timeout = timeout

    def _convert_messages(self, messages: list[Message]) -> list[dict]:
        """Convert messages to Ollama format, extracting images for vision."""
        ollama_messages = []
        for m in messages:
            msg = {"role": m.role}
            if isinstance(m.content, list):
                # OpenAI-style content blocks → Ollama images field
                text_parts = []
                images = []
                for block in m.content:
                    if isinstance(block, dict) and block.get("type") == "image_url":
                        url = block.get("image_url", {}).get("url", "")
                        if url.startswith("data:"):
                            b64_data = url.split(",", 1)[1]
                            images.append(b64_data)
                    elif isinstance(block, dict) and block.get("type") == "text":
                        text_parts.append(block.get("text", ""))
                msg["content"] = "\n".join(text_parts) if text_parts else ""
                if images:
                    msg["images"] = images
            else:
                msg["content"] = m.content
            ollama_messages.append(msg)
        return ollama_messages

    async def chat(
        self,
        messages: list[Message],
        model: Optional[str] = None,
        temperature: float = 0.7,
        max_tokens: int = 2048,
        tools: Optional[list[dict]] = None,
    ) -> ChatResponse:
        """Send chat request to Ollama"""
        model = model or self.default_model
        ollama_messages = self._convert_messages(messages)

        async with httpx.AsyncClient(timeout=self.timeout) as client:
            response = await client.post(
                f"{self.base_url}/api/chat",
                json={
                    "model": model,
                    "messages": ollama_messages,
                    "stream": False,
                    "options": {
                        "temperature": temperature,
                        "num_predict": max_tokens,
                    },
                },
            )
            response.raise_for_status()
            data = response.json()

            return ChatResponse(
                content=data["message"]["content"],
                model=model,
                provider=self.name,
                usage={
                    "prompt_tokens": data.get("prompt_eval_count", 0),
                    "completion_tokens": data.get("eval_count", 0),
                },
            )

    async def chat_stream(
        self,
        messages: list[Message],
        model: Optional[str] = None,
        temperature: float = 0.7,
        max_tokens: int = 2048,
    ) -> AsyncGenerator[dict, None]:
        """Stream chat tokens from Ollama (NDJSON → SSE chunks)."""
        model = model or self.default_model
        ollama_messages = self._convert_messages(messages)

        async with httpx.AsyncClient(timeout=self.timeout) as client:
            async with client.stream(
                "POST",
                f"{self.base_url}/api/chat",
                json={
                    "model": model,
                    "messages": ollama_messages,
                    "stream": True,
                    "options": {
                        "temperature": temperature,
                        "num_predict": max_tokens,
                    },
                },
            ) as response:
                response.raise_for_status()
                accumulated = ""
                index = 0
                async for line in response.aiter_lines():
                    if not line.strip():
                        continue
                    chunk = json.loads(line)
                    if chunk.get("done"):
                        yield {
                            'type': 'done',
                            'content': accumulated,
                            'usage': {
                                'prompt_tokens': chunk.get('prompt_eval_count', 0),
                                'completion_tokens': chunk.get('eval_count', 0),
                            },
                        }
                        return
                    token = chunk.get("message", {}).get("content", "")
                    if token:
                        accumulated += token
                        yield {'type': 'token', 'content': token, 'index': index}
                        index += 1

    async def health(self) -> bool:
        """Check if Ollama is running"""
        try:
            async with httpx.AsyncClient(timeout=5.0) as client:
                response = await client.get(f"{self.base_url}/api/tags")
                return response.status_code == 200
        except Exception:
            return False

    def list_models(self) -> list[str]:
        """List common Ollama models (async version would query API)"""
        return [
            "llama3.2",
            "llama3.2:70b",
            "codellama",
            "mistral",
            "mixtral",
            "qwen2.5",
            "deepseek-r1",
            "phi3",
        ]

    async def pull_model(self, model: str) -> bool:
        """Pull a model from Ollama registry"""
        try:
            async with httpx.AsyncClient(timeout=600.0) as client:
                response = await client.post(
                    f"{self.base_url}/api/pull",
                    json={"name": model, "stream": False},
                )
                return response.status_code == 200
        except Exception:
            return False
