"""
Local / OpenAI-Compatible Provider
===================================
Generic provider for any server that speaks OpenAI's /v1/chat/completions API.
Works with: Ollama, vLLM, LM Studio, text-generation-inference, local Qwen, etc.
"""

import httpx
from typing import Optional
from .base import AgentProvider, Message, ChatResponse


class LocalOpenAIProvider(AgentProvider):
    """
    Generic OpenAI-compatible provider.

    Connects to any server exposing /v1/chat/completions.
    """

    name = "local"

    def __init__(
        self,
        base_url: str = "http://localhost:11434/v1",
        api_key: str = "",
        model_name: str = "llama3.2",
        timeout: float = 120.0,
    ):
        self.base_url = base_url.rstrip("/")
        self.api_key = api_key
        self.model_name = model_name
        self.timeout = timeout
        print(f"[local] LocalOpenAIProvider init: base_url={self.base_url}, model={self.model_name}")

    async def chat(
        self,
        messages: list[Message],
        model: Optional[str] = None,
        temperature: float = 0.7,
        max_tokens: int = 2048,
        tools: Optional[list[dict]] = None,
    ) -> ChatResponse:
        """Send chat request to OpenAI-compatible server"""
        if not self.base_url:
            raise ValueError("base_url is required for local provider")

        model = model or self.model_name
        url = f"{self.base_url}/chat/completions"
        last_content = messages[-1].content if messages else "(empty)"
        if isinstance(last_content, list):
            msg_preview = f"[multimodal: {len(last_content)} blocks]"
        else:
            msg_preview = str(last_content)[:80]
        print(f"[local] chat → {url} model={model} msgs={len(messages)} preview='{msg_preview}'")

        headers: dict[str, str] = {"Content-Type": "application/json"}
        if self.api_key:
            headers["Authorization"] = f"Bearer {self.api_key}"

        async with httpx.AsyncClient(timeout=self.timeout) as client:
            response = await client.post(
                f"{self.base_url}/chat/completions",
                headers=headers,
                json={
                    "model": model,
                    "messages": [{"role": m.role, "content": m.content} for m in messages],
                    "max_tokens": max_tokens,
                    "temperature": temperature,
                },
            )
            response.raise_for_status()
            data = response.json()

            usage = data.get("usage", {})
            return ChatResponse(
                content=data["choices"][0]["message"]["content"],
                model=model,
                provider=self.name,
                usage={
                    "prompt_tokens": usage.get("prompt_tokens", 0),
                    "completion_tokens": usage.get("completion_tokens", 0),
                },
            )

    async def health(self) -> bool:
        """Check if server is reachable"""
        if not self.base_url:
            return False
        try:
            async with httpx.AsyncClient(timeout=5.0) as client:
                # Try /health first, fall back to /v1/models
                for path in ["/health", "/v1/models", "/models"]:
                    try:
                        url = self.base_url.replace("/v1", "") + path
                        r = await client.get(url)
                        if r.status_code < 500:
                            return True
                    except httpx.ConnectError:
                        continue
        except Exception:
            pass
        return False

    def list_models(self) -> list[str]:
        """User-defined — return configured model"""
        return [self.model_name] if self.model_name else []
