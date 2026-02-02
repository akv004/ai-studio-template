"""
Chat Service
============
High-level chat interface with conversation memory.
"""

from typing import Optional
from dataclasses import dataclass, field
from agent.providers import AgentProvider, Message, ChatResponse, OllamaProvider


@dataclass 
class Conversation:
    """Represents a conversation with history"""
    id: str
    messages: list[Message] = field(default_factory=list)
    provider_name: str = "ollama"
    model: Optional[str] = None


class ChatService:
    """
    Chat service with conversation memory and provider management.
    
    Example:
        service = ChatService()
        response = await service.chat("conv_123", "Hello!")
    """
    
    def __init__(self):
        self.providers: dict[str, AgentProvider] = {}
        self.conversations: dict[str, Conversation] = {}
        self.default_provider = "ollama"
        
        # Register default providers
        self.register_provider(OllamaProvider())
    
    def register_provider(self, provider: AgentProvider):
        """Register a provider instance"""
        self.providers[provider.name] = provider
    
    def get_provider(self, name: str) -> AgentProvider:
        """Get a provider by name"""
        if name not in self.providers:
            raise ValueError(f"Provider '{name}' not registered")
        return self.providers[name]
    
    def create_conversation(
        self,
        conversation_id: str,
        provider_name: Optional[str] = None,
        system_prompt: Optional[str] = None,
    ) -> Conversation:
        """Create a new conversation"""
        conv = Conversation(
            id=conversation_id,
            provider_name=provider_name or self.default_provider,
        )
        
        if system_prompt:
            conv.messages.append(Message(role="system", content=system_prompt))
        
        self.conversations[conversation_id] = conv
        return conv
    
    def get_or_create_conversation(
        self,
        conversation_id: str,
        **kwargs,
    ) -> Conversation:
        """Get existing conversation or create new one"""
        if conversation_id not in self.conversations:
            return self.create_conversation(conversation_id, **kwargs)
        return self.conversations[conversation_id]
    
    async def chat(
        self,
        conversation_id: str,
        user_message: str,
        provider_name: Optional[str] = None,
        model: Optional[str] = None,
        temperature: float = 0.7,
    ) -> ChatResponse:
        """
        Send a message and get a response.
        
        Args:
            conversation_id: Unique conversation identifier
            user_message: User's input
            provider_name: Override conversation's default provider
            model: Override provider's default model
            temperature: Sampling temperature
            
        Returns:
            ChatResponse with the assistant's reply
        """
        conv = self.get_or_create_conversation(conversation_id)
        
        # Add user message to history
        conv.messages.append(Message(role="user", content=user_message))
        
        # Get provider
        provider = self.get_provider(provider_name or conv.provider_name)
        
        # Get response
        response = await provider.chat(
            messages=conv.messages,
            model=model or conv.model,
            temperature=temperature,
        )
        
        # Add assistant response to history
        conv.messages.append(Message(role="assistant", content=response.content))
        
        return response
    
    async def health_check(self) -> dict[str, bool]:
        """Check health of all providers"""
        results = {}
        for name, provider in self.providers.items():
            results[name] = await provider.health()
        return results
    
    def list_providers(self) -> list[dict]:
        """List all registered providers with their models"""
        return [
            {
                "name": name,
                "models": provider.list_models(),
                "default": name == self.default_provider,
            }
            for name, provider in self.providers.items()
        ]
    
    def clear_conversation(self, conversation_id: str):
        """Clear a conversation's history"""
        if conversation_id in self.conversations:
            self.conversations[conversation_id].messages = []
    
    def delete_conversation(self, conversation_id: str):
        """Delete a conversation"""
        self.conversations.pop(conversation_id, None)
