"""
Base Provider Interface
=======================
Abstract base class for all LLM providers.
"""

from abc import ABC, abstractmethod
from typing import Optional, Any
from pydantic import BaseModel


class Message(BaseModel):
    """A single message in a conversation"""
    role: str  # "user" | "assistant" | "system" | "tool"
    content: Any  # str or list of content blocks (for tool results)


class ChatResponse(BaseModel):
    """Response from a chat completion"""
    content: str
    model: str
    provider: str
    usage: Optional[dict] = None
    tool_calls: Optional[list[dict]] = None  # [{id, name, input}]
    stop_reason: Optional[str] = None  # "end_turn", "tool_use", etc.
    raw_content: Optional[list[dict]] = None  # Raw content blocks (for multi-turn tool use)


class AgentProvider(ABC):
    """
    Abstract base class for LLM providers.

    Implement this interface to add support for new LLM backends:
    - Local: Ollama, vLLM, LM Studio
    - Cloud: Anthropic, OpenAI, Google
    """

    name: str = "base"

    @abstractmethod
    async def chat(
        self,
        messages: list[Message],
        model: Optional[str] = None,
        temperature: float = 0.7,
        max_tokens: int = 2048,
        tools: Optional[list[dict]] = None,
    ) -> ChatResponse:
        """
        Send messages and get a completion.

        Args:
            messages: Conversation history
            model: Override default model
            temperature: Sampling temperature (0-2)
            max_tokens: Maximum response length
            tools: Optional list of tool definitions (provider-specific format)

        Returns:
            ChatResponse with the model's reply
        """
        pass

    @abstractmethod
    async def health(self) -> bool:
        """Check if the provider is available and responding."""
        pass

    @abstractmethod
    def list_models(self) -> list[str]:
        """List available models for this provider."""
        pass
