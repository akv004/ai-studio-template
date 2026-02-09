"""
Azure OpenAI Provider
=====================
Cloud LLM provider using Azure's OpenAI Service.
"""

import httpx
from typing import Optional
from .base import AgentProvider, Message, ChatResponse


class AzureOpenAIProvider(AgentProvider):
    """
    Azure OpenAI provider for enterprise LLM inference.

    Requires endpoint, api_key, deployment name, and api_version.
    """

    name = "azure_openai"

    def __init__(
        self,
        endpoint: str = "",
        api_key: str = "",
        deployment: str = "",
        api_version: str = "2024-08-01-preview",
        timeout: float = 60.0,
    ):
        self.endpoint = endpoint.rstrip("/")
        self.api_key = api_key
        self.deployment = deployment
        self.api_version = api_version
        self.timeout = timeout

    async def chat(
        self,
        messages: list[Message],
        model: Optional[str] = None,
        temperature: float = 0.7,
        max_tokens: int = 2048,
    ) -> ChatResponse:
        """Send chat request to Azure OpenAI"""
        if not self.api_key or not self.endpoint:
            raise ValueError("Azure OpenAI endpoint and api_key are required")

        deployment = model or self.deployment
        url = (
            f"{self.endpoint}/openai/deployments/{deployment}"
            f"/chat/completions?api-version={self.api_version}"
        )

        async with httpx.AsyncClient(timeout=self.timeout) as client:
            response = await client.post(
                url,
                headers={
                    "api-key": self.api_key,
                    "Content-Type": "application/json",
                },
                json={
                    "messages": [{"role": m.role, "content": m.content} for m in messages],
                    "max_tokens": max_tokens,
                    "temperature": temperature,
                },
            )
            response.raise_for_status()
            data = response.json()

            return ChatResponse(
                content=data["choices"][0]["message"]["content"],
                model=deployment,
                provider=self.name,
                usage={
                    "prompt_tokens": data["usage"]["prompt_tokens"],
                    "completion_tokens": data["usage"]["completion_tokens"],
                },
            )

    async def health(self) -> bool:
        """Check if endpoint is reachable with the given key"""
        if not self.api_key or not self.endpoint:
            return False
        try:
            async with httpx.AsyncClient(timeout=10.0) as client:
                r = await client.get(
                    f"{self.endpoint}/openai/models?api-version={self.api_version}",
                    headers={"api-key": self.api_key},
                )
                return r.status_code < 400
        except Exception:
            return False

    def list_models(self) -> list[str]:
        """Azure deployments are user-defined"""
        return ["gpt-4o", "gpt-4o-mini", "gpt-4-turbo", "gpt-35-turbo"]
