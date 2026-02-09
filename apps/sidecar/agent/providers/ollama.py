"""
Ollama Provider
===============
Local LLM provider using Ollama.
Runs on your local machine with GPU acceleration.
"""

import httpx
from typing import Optional
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
        
        async with httpx.AsyncClient(timeout=self.timeout) as client:
            response = await client.post(
                f"{self.base_url}/api/chat",
                json={
                    "model": model,
                    "messages": [{"role": m.role, "content": m.content} for m in messages],
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
