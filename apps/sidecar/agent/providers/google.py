"""
Google AI Provider (Gemini)
===========================
Cloud LLM provider using Google AI Studio / Gemini API.
"""

import os
import httpx
from typing import Optional
from .base import AgentProvider, Message, ChatResponse


class GoogleProvider(AgentProvider):
    """
    Google AI (Gemini) provider for cloud LLM inference.
    
    Requires GOOGLE_API_KEY environment variable.
    Get your key from: https://aistudio.google.com/apikey
    """
    
    name = "google"
    
    def __init__(
        self,
        api_key: Optional[str] = None,
        default_model: str = "gemini-2.0-flash",
        timeout: float = 60.0,
    ):
        self.api_key = api_key or os.getenv("GOOGLE_API_KEY")
        self.default_model = default_model
        self.timeout = timeout
        self.base_url = "https://generativelanguage.googleapis.com/v1beta"
    
    async def chat(
        self,
        messages: list[Message],
        model: Optional[str] = None,
        temperature: float = 0.7,
        max_tokens: int = 2048,
    ) -> ChatResponse:
        """Send chat request to Google AI"""
        if not self.api_key:
            raise ValueError("GOOGLE_API_KEY not set")
        
        model = model or self.default_model
        
        # Convert messages to Gemini format
        contents = []
        system_instruction = None
        
        for m in messages:
            if m.role == "system":
                system_instruction = m.content
            else:
                # Gemini uses "user" and "model" roles
                role = "model" if m.role == "assistant" else "user"
                contents.append({
                    "role": role,
                    "parts": [{"text": m.content}]
                })
        
        async with httpx.AsyncClient(timeout=self.timeout) as client:
            payload = {
                "contents": contents,
                "generationConfig": {
                    "temperature": temperature,
                    "maxOutputTokens": max_tokens,
                }
            }
            
            if system_instruction:
                payload["systemInstruction"] = {
                    "parts": [{"text": system_instruction}]
                }
            
            response = await client.post(
                f"{self.base_url}/models/{model}:generateContent",
                params={"key": self.api_key},
                headers={"Content-Type": "application/json"},
                json=payload,
            )
            response.raise_for_status()
            data = response.json()
            
            # Extract response text
            content = data["candidates"][0]["content"]["parts"][0]["text"]
            
            # Usage metadata
            usage = {}
            if "usageMetadata" in data:
                usage = {
                    "prompt_tokens": data["usageMetadata"].get("promptTokenCount", 0),
                    "completion_tokens": data["usageMetadata"].get("candidatesTokenCount", 0),
                }
            
            return ChatResponse(
                content=content,
                model=model,
                provider=self.name,
                usage=usage,
            )
    
    async def health(self) -> bool:
        """Check if API key is valid by listing models"""
        if not self.api_key:
            return False
        try:
            async with httpx.AsyncClient(timeout=10.0) as client:
                r = await client.get(
                    f"{self.base_url}/models",
                    params={"key": self.api_key},
                )
                return r.status_code == 200
        except Exception:
            return False
    
    def list_models(self) -> list[str]:
        """List available Gemini models"""
        return [
            "gemini-2.0-flash",
            "gemini-2.0-flash-lite", 
            "gemini-1.5-pro",
            "gemini-1.5-flash",
            "gemini-1.5-flash-8b",
        ]
