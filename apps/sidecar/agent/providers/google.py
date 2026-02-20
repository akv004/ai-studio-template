"""
Google AI Provider (Gemini)
===========================
Cloud LLM provider using Google AI Studio / Gemini API.
Supports tool calling (function calling).
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
        tools: Optional[list[dict]] = None,
    ) -> ChatResponse:
        """Send chat request to Google AI, optionally with tools."""
        if not self.api_key:
            raise ValueError("GOOGLE_API_KEY not set")

        model = model or self.default_model

        # Convert messages to Gemini format
        contents = []
        system_instruction = None

        for m in messages:
            if m.role == "system":
                system_instruction = m.content if isinstance(m.content, str) else str(m.content)
            elif m.role == "tool":
                # Tool results in Gemini format
                if isinstance(m.content, list):
                    # Already structured as function response parts
                    contents.append({
                        "role": "function",
                        "parts": m.content,
                    })
                else:
                    contents.append({
                        "role": "function",
                        "parts": [{"functionResponse": {"name": "tool", "response": {"result": m.content}}}],
                    })
            else:
                role = "model" if m.role == "assistant" else "user"
                if isinstance(m.content, str):
                    contents.append({
                        "role": role,
                        "parts": [{"text": m.content}]
                    })
                elif isinstance(m.content, list):
                    # Convert OpenAI-style content blocks to Gemini parts
                    parts = []
                    for block in m.content:
                        if isinstance(block, dict) and block.get("type") == "image_url":
                            url = block.get("image_url", {}).get("url", "")
                            if url.startswith("data:"):
                                # Parse data URI: data:image/png;base64,iVBOR...
                                header, b64_data = url.split(",", 1)
                                mime = header.split(":")[1].split(";")[0]
                                parts.append({"inlineData": {"mimeType": mime, "data": b64_data}})
                            else:
                                parts.append({"text": f"[image: {url}]"})
                        elif isinstance(block, dict) and block.get("type") == "text":
                            parts.append({"text": block.get("text", "")})
                        elif isinstance(block, dict) and "functionCall" in block:
                            parts.append(block)
                        else:
                            parts.append(block)
                    contents.append({"role": role, "parts": parts})
                else:
                    contents.append({
                        "role": role,
                        "parts": [{"text": str(m.content)}]
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

            if tools:
                payload["tools"] = [{"functionDeclarations": tools}]

            response = await client.post(
                f"{self.base_url}/models/{model}:generateContent",
                params={"key": self.api_key},
                headers={"Content-Type": "application/json"},
                json=payload,
            )
            response.raise_for_status()
            data = response.json()

            # Parse response
            candidate = data["candidates"][0]
            parts = candidate["content"]["parts"]
            finish_reason = candidate.get("finishReason", "STOP")

            text_parts = []
            tool_calls = []
            raw_parts = []

            for part in parts:
                if "text" in part:
                    text_parts.append(part["text"])
                elif "functionCall" in part:
                    fc = part["functionCall"]
                    tool_calls.append({
                        "id": fc["name"],  # Gemini doesn't have separate IDs
                        "name": fc["name"],
                        "input": fc.get("args", {}),
                    })
                raw_parts.append(part)

            # Usage metadata
            usage = {}
            if "usageMetadata" in data:
                usage = {
                    "prompt_tokens": data["usageMetadata"].get("promptTokenCount", 0),
                    "completion_tokens": data["usageMetadata"].get("candidatesTokenCount", 0),
                }

            return ChatResponse(
                content="\n".join(text_parts) if text_parts else "",
                model=model,
                provider=self.name,
                usage=usage,
                tool_calls=tool_calls if tool_calls else None,
                stop_reason="tool_use" if tool_calls else finish_reason,
                raw_content=raw_parts if tool_calls else None,
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
