"""
OpenAI Provider
===============
Cloud LLM provider using OpenAI's API.
"""

import json
import os
import httpx
from typing import AsyncGenerator, Optional
from .base import AgentProvider, Message, ChatResponse


class OpenAIProvider(AgentProvider):
    """
    OpenAI provider for cloud LLM inference.
    
    Requires OPENAI_API_KEY environment variable.
    """
    
    name = "openai"
    
    def __init__(
        self,
        api_key: Optional[str] = None,
        default_model: str = "gpt-4o",
        timeout: float = 60.0,
    ):
        self.api_key = api_key or os.getenv("OPENAI_API_KEY")
        self.default_model = default_model
        self.timeout = timeout
        self.base_url = "https://api.openai.com"
    
    async def chat(
        self,
        messages: list[Message],
        model: Optional[str] = None,
        temperature: float = 0.7,
        max_tokens: int = 2048,
        tools: Optional[list[dict]] = None,
    ) -> ChatResponse:
        """Send chat request to OpenAI"""
        if not self.api_key:
            raise ValueError("OPENAI_API_KEY not set")
        
        model = model or self.default_model
        
        async with httpx.AsyncClient(timeout=self.timeout) as client:
            response = await client.post(
                f"{self.base_url}/v1/chat/completions",
                headers={
                    "Authorization": f"Bearer {self.api_key}",
                    "Content-Type": "application/json",
                },
                json={
                    "model": model,
                    "messages": [{"role": m.role, "content": m.content} for m in messages],
                    "max_tokens": max_tokens,
                    "temperature": temperature,
                },
            )
            response.raise_for_status()
            data = response.json()
            
            return ChatResponse(
                content=data["choices"][0]["message"]["content"],
                model=model,
                provider=self.name,
                usage={
                    "prompt_tokens": data["usage"]["prompt_tokens"],
                    "completion_tokens": data["usage"]["completion_tokens"],
                },
            )
    
    async def chat_stream(
        self,
        messages: list[Message],
        model: Optional[str] = None,
        temperature: float = 0.7,
        max_tokens: int = 2048,
    ) -> AsyncGenerator[dict, None]:
        """Stream chat tokens from OpenAI (SSE with delta content)."""
        if not self.api_key:
            raise ValueError("OPENAI_API_KEY not set")

        model = model or self.default_model

        async with httpx.AsyncClient(timeout=self.timeout) as client:
            async with client.stream(
                "POST",
                f"{self.base_url}/v1/chat/completions",
                headers={
                    "Authorization": f"Bearer {self.api_key}",
                    "Content-Type": "application/json",
                },
                json={
                    "model": model,
                    "messages": [{"role": m.role, "content": m.content} for m in messages],
                    "max_tokens": max_tokens,
                    "temperature": temperature,
                    "stream": True,
                    "stream_options": {"include_usage": True},
                },
            ) as response:
                response.raise_for_status()
                accumulated = ""
                index = 0
                usage = {}
                async for line in response.aiter_lines():
                    line = line.strip()
                    if not line or line.startswith(":"):
                        continue
                    if not line.startswith("data: "):
                        continue
                    data = line[6:]
                    if data == "[DONE]":
                        break
                    chunk = json.loads(data)
                    if chunk.get("usage"):
                        usage = {
                            "prompt_tokens": chunk["usage"].get("prompt_tokens", 0),
                            "completion_tokens": chunk["usage"].get("completion_tokens", 0),
                        }
                    choices = chunk.get("choices", [])
                    if choices:
                        token = choices[0].get("delta", {}).get("content", "")
                        if token:
                            accumulated += token
                            yield {'type': 'token', 'content': token, 'index': index}
                            index += 1
                yield {'type': 'done', 'content': accumulated, 'usage': usage}

    async def health(self) -> bool:
        """Check if API key is valid (lightweight check)"""
        return bool(self.api_key)
    
    def list_models(self) -> list[str]:
        """List available OpenAI models"""
        return [
            "gpt-4o",
            "gpt-4o-mini",
            "gpt-4-turbo",
            "o1",
            "o1-mini",
        ]
