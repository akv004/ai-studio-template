"""
Anthropic Provider
==================
Cloud LLM provider using Anthropic's Claude API.
"""

import os
import httpx
from typing import Optional
from .base import AgentProvider, Message, ChatResponse


class AnthropicProvider(AgentProvider):
    """
    Anthropic Claude provider for cloud LLM inference.
    
    Requires ANTHROPIC_API_KEY environment variable.
    """
    
    name = "anthropic"
    
    def __init__(
        self,
        api_key: Optional[str] = None,
        default_model: str = "claude-sonnet-4-20250514",
        timeout: float = 60.0,
    ):
        self.api_key = api_key or os.getenv("ANTHROPIC_API_KEY")
        self.default_model = default_model
        self.timeout = timeout
        self.base_url = "https://api.anthropic.com"
    
    async def chat(
        self,
        messages: list[Message],
        model: Optional[str] = None,
        temperature: float = 0.7,
        max_tokens: int = 2048,
    ) -> ChatResponse:
        """Send chat request to Anthropic"""
        if not self.api_key:
            raise ValueError("ANTHROPIC_API_KEY not set")
        
        model = model or self.default_model
        
        # Extract system message if present
        system_content = None
        chat_messages = []
        for m in messages:
            if m.role == "system":
                system_content = m.content
            else:
                chat_messages.append({"role": m.role, "content": m.content})
        
        async with httpx.AsyncClient(timeout=self.timeout) as client:
            payload = {
                "model": model,
                "messages": chat_messages,
                "max_tokens": max_tokens,
                "temperature": temperature,
            }
            if system_content:
                payload["system"] = system_content
            
            response = await client.post(
                f"{self.base_url}/v1/messages",
                headers={
                    "x-api-key": self.api_key,
                    "anthropic-version": "2023-06-01",
                    "content-type": "application/json",
                },
                json=payload,
            )
            response.raise_for_status()
            data = response.json()
            
            return ChatResponse(
                content=data["content"][0]["text"],
                model=model,
                provider=self.name,
                usage={
                    "prompt_tokens": data["usage"]["input_tokens"],
                    "completion_tokens": data["usage"]["output_tokens"],
                },
            )
    
    async def health(self) -> bool:
        """Check if API key is valid by calling the models endpoint"""
        if not self.api_key:
            return False
        try:
            async with httpx.AsyncClient(timeout=10.0) as client:
                r = await client.get(
                    f"{self.base_url}/v1/models",
                    headers={
                        "x-api-key": self.api_key,
                        "anthropic-version": "2023-06-01",
                    },
                )
                return r.status_code == 200
        except Exception:
            return False
    
    def list_models(self) -> list[str]:
        """List available Claude models"""
        return [
            "claude-sonnet-4-20250514",
            "claude-opus-4-20250514",
            "claude-3-5-haiku-20241022",
        ]
