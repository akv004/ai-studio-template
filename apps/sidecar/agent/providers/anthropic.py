"""
Anthropic Provider
==================
Cloud LLM provider using Anthropic's Claude API.
Supports tool calling (function calling).
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
        tools: Optional[list[dict]] = None,
    ) -> ChatResponse:
        """Send chat request to Anthropic, optionally with tools."""
        if not self.api_key:
            raise ValueError("ANTHROPIC_API_KEY not set")

        model = model or self.default_model

        # Extract system message if present
        system_content = None
        chat_messages = []
        for m in messages:
            if m.role == "system":
                system_content = m.content if isinstance(m.content, str) else str(m.content)
            elif isinstance(m.content, list):
                # Convert OpenAI-style content blocks to Anthropic format
                blocks = []
                for block in m.content:
                    if isinstance(block, dict) and block.get("type") == "image_url":
                        url = block.get("image_url", {}).get("url", "")
                        if url.startswith("data:"):
                            header, b64_data = url.split(",", 1)
                            media_type = header.split(":")[1].split(";")[0]
                            blocks.append({
                                "type": "image",
                                "source": {"type": "base64", "media_type": media_type, "data": b64_data},
                            })
                        else:
                            blocks.append({"type": "text", "text": f"[image: {url}]"})
                    elif isinstance(block, dict) and block.get("type") == "text":
                        blocks.append({"type": "text", "text": block.get("text", "")})
                    else:
                        blocks.append(block)
                chat_messages.append({"role": m.role, "content": blocks})
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
            if tools:
                payload["tools"] = tools

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

            # Parse response content blocks
            content_blocks = data.get("content", [])
            stop_reason = data.get("stop_reason", "end_turn")

            text_parts = []
            tool_calls = []
            for block in content_blocks:
                if block["type"] == "text":
                    text_parts.append(block["text"])
                elif block["type"] == "tool_use":
                    tool_calls.append({
                        "id": block["id"],
                        "name": block["name"],
                        "input": block["input"],
                    })

            return ChatResponse(
                content="\n".join(text_parts) if text_parts else "",
                model=model,
                provider=self.name,
                usage={
                    "prompt_tokens": data["usage"]["input_tokens"],
                    "completion_tokens": data["usage"]["output_tokens"],
                },
                tool_calls=tool_calls if tool_calls else None,
                stop_reason=stop_reason,
                raw_content=content_blocks if tool_calls else None,
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
