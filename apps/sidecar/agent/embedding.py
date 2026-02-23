"""
Embedding client for RAG Knowledge Base.
Separate from AgentProvider (which only has chat/stream).
Reads the same Settings config (api_key, base_url, extra_config).
"""

import asyncio
import httpx
from typing import Optional


class EmbedResult:
    def __init__(self, vectors: list[list[float]], model: str, dimensions: int, usage: dict):
        self.vectors = vectors
        self.model = model
        self.dimensions = dimensions
        self.usage = usage


class EmbeddingClient:
    """Base embedding client. Subclasses implement _embed_batch()."""

    def __init__(self, base_url: str, api_key: str = '', max_batch: int = 100):
        self.base_url = base_url.rstrip('/')
        self.api_key = api_key
        self.max_batch = max_batch
        self.max_tokens_per_text = 8191

    async def embed(self, texts: list[str], model: str) -> EmbedResult:
        """Embed texts with automatic batching, truncation, and retry."""
        # Truncate overlong texts
        truncated = []
        warnings = []
        for i, text in enumerate(texts):
            estimated_tokens = len(text) // 4
            if estimated_tokens > self.max_tokens_per_text:
                max_chars = self.max_tokens_per_text * 4
                text = text[:max_chars]
                warnings.append(i)
            truncated.append(text)

        if warnings:
            print(f'[embed] Truncated {len(warnings)} text(s) exceeding {self.max_tokens_per_text} token limit')

        # Batch and embed
        all_vectors: list[list[float]] = []
        total_usage = {'prompt_tokens': 0, 'total_tokens': 0}

        for batch_start in range(0, len(truncated), self.max_batch):
            batch = truncated[batch_start:batch_start + self.max_batch]
            vectors, usage = await self._embed_with_retry(batch, model)
            all_vectors.extend(vectors)
            total_usage['prompt_tokens'] += usage.get('prompt_tokens', 0)
            total_usage['total_tokens'] += usage.get('total_tokens', 0)

        dims = len(all_vectors[0]) if all_vectors else 0
        return EmbedResult(
            vectors=all_vectors,
            model=model,
            dimensions=dims,
            usage=total_usage,
        )

    async def _embed_with_retry(self, texts: list[str], model: str, max_retries: int = 3):
        """Retry with exponential backoff."""
        last_error = None
        for attempt in range(max_retries):
            try:
                return await self._embed_batch(texts, model)
            except Exception as e:
                last_error = e
                if attempt < max_retries - 1:
                    wait = 2 ** attempt
                    print(f'[embed] Attempt {attempt + 1} failed: {e}. Retrying in {wait}s...')
                    await asyncio.sleep(wait)
        raise last_error

    async def _embed_batch(self, texts: list[str], model: str) -> tuple[list[list[float]], dict]:
        raise NotImplementedError


class AzureOpenAIEmbeddingClient(EmbeddingClient):
    """Azure OpenAI embeddings via deployment API."""

    def __init__(self, endpoint: str, api_key: str, api_version: str = '2024-02-01'):
        super().__init__(base_url=endpoint, api_key=api_key, max_batch=2048)
        self.api_version = api_version

    async def _embed_batch(self, texts: list[str], model: str) -> tuple[list[list[float]], dict]:
        url = f'{self.base_url}/openai/deployments/{model}/embeddings?api-version={self.api_version}'
        async with httpx.AsyncClient(timeout=60.0) as client:
            resp = await client.post(
                url,
                json={'input': texts},
                headers={'api-key': self.api_key, 'Content-Type': 'application/json'},
            )
            resp.raise_for_status()
            data = resp.json()
            vectors = [item['embedding'] for item in data['data']]
            usage = data.get('usage', {})
            return vectors, usage


class OpenAICompatibleEmbeddingClient(EmbeddingClient):
    """OpenAI-compatible embeddings (OpenAI, Local vLLM, etc.)."""

    def __init__(self, base_url: str, api_key: str = '', max_batch: int = 32):
        super().__init__(base_url=base_url, api_key=api_key, max_batch=max_batch)

    async def _embed_batch(self, texts: list[str], model: str) -> tuple[list[list[float]], dict]:
        url = f'{self.base_url}/embeddings'
        headers = {'Content-Type': 'application/json'}
        if self.api_key:
            headers['Authorization'] = f'Bearer {self.api_key}'

        async with httpx.AsyncClient(timeout=60.0) as client:
            resp = await client.post(
                url,
                json={'input': texts, 'model': model},
                headers=headers,
            )
            resp.raise_for_status()
            data = resp.json()
            vectors = [item['embedding'] for item in data['data']]
            usage = data.get('usage', {})
            return vectors, usage


def create_embedding_client(
    provider: str,
    api_key: str = '',
    base_url: str = '',
    extra_config: Optional[dict] = None,
) -> EmbeddingClient:
    """Factory: create embedding client from provider config."""
    cfg = extra_config or {}

    if provider == 'azure_openai':
        return AzureOpenAIEmbeddingClient(
            endpoint=base_url or cfg.get('endpoint', ''),
            api_key=api_key,
            api_version=cfg.get('api_version', '2024-02-01'),
        )
    elif provider in ('local', 'openai'):
        default_url = 'http://localhost:11434/v1' if provider == 'local' else 'https://api.openai.com/v1'
        return OpenAICompatibleEmbeddingClient(
            base_url=base_url or cfg.get('base_url', default_url),
            api_key=api_key,
            max_batch=32 if provider == 'local' else 100,
        )
    elif provider == 'ollama':
        return OpenAICompatibleEmbeddingClient(
            base_url=base_url or 'http://localhost:11434/v1',
            api_key='',
            max_batch=32,
        )
    else:
        raise ValueError(f'Unsupported embedding provider: {provider}')
