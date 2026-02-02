"""
OpenAI Provider
===============
Cloud LLM provider using OpenAI's API.
"""

import os
import httpx
from typing import Optional
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
