"""
AI Studio - Python Sidecar Server
=================================

FastAPI-based server providing multi-provider LLM chat capabilities.
Connects to local LLMs (Ollama) and cloud providers (Anthropic, OpenAI).

Run modes:
- Development: `python server.py`
- Docker: `docker compose up`
"""

import os
import uuid
from contextlib import asynccontextmanager
from typing import Optional

from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel

from agent.chat import ChatService
from agent.providers import (
    OllamaProvider,
    AnthropicProvider,
    OpenAIProvider,
    Message,
)


# ============================================================================
# Request/Response Models
# ============================================================================

class ChatRequest(BaseModel):
    """Chat completion request"""
    conversation_id: Optional[str] = None
    message: str
    provider: Optional[str] = None
    model: Optional[str] = None
    temperature: float = 0.7
    system_prompt: Optional[str] = None


class ChatMessageRequest(BaseModel):
    """Direct message request (no conversation)"""
    messages: list[dict]
    provider: Optional[str] = "ollama"
    model: Optional[str] = None
    temperature: float = 0.7


class ChatResponse(BaseModel):
    """Chat completion response"""
    conversation_id: str
    content: str
    model: str
    provider: str
    usage: Optional[dict] = None


# ============================================================================
# Application Setup
# ============================================================================

chat_service: ChatService = None


@asynccontextmanager
async def lifespan(app: FastAPI):
    """Initialize services on startup"""
    global chat_service
    chat_service = ChatService()
    
    # Configure Ollama (local LLM)
    ollama_host = os.getenv("OLLAMA_HOST", "http://localhost:11434")
    ollama_model = os.getenv("OLLAMA_MODEL", "llama3.2")
    chat_service.register_provider(OllamaProvider(
        base_url=ollama_host,
        default_model=ollama_model,
    ))
    
    # Configure Anthropic (if API key is set)
    if os.getenv("ANTHROPIC_API_KEY"):
        chat_service.register_provider(AnthropicProvider())
        print("✓ Anthropic provider enabled")
    
    # Configure OpenAI (if API key is set)
    if os.getenv("OPENAI_API_KEY"):
        chat_service.register_provider(OpenAIProvider())
        print("✓ OpenAI provider enabled")
    
    print(f"🤖 AI Studio Sidecar initialized")
    print(f"   Ollama: {ollama_host} (model: {ollama_model})")
    
    yield


app = FastAPI(
    title="AI Studio Agent",
    description="Multi-provider LLM agent server",
    version="0.1.0",
    lifespan=lifespan,
)

# CORS for Tauri/React frontend
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


# ============================================================================
# Health & Status Endpoints
# ============================================================================

@app.get("/health")
async def health():
    """Health check endpoint"""
    return {"status": "healthy", "version": "0.1.0"}


@app.get("/status")
async def status():
    """Detailed status including provider health"""
    provider_health = await chat_service.health_check()
    return {
        "status": "healthy",
        "providers": provider_health,
        "active_conversations": len(chat_service.conversations),
    }


@app.get("/providers")
async def list_providers():
    """List available LLM providers and their models"""
    return {"providers": chat_service.list_providers()}


# ============================================================================
# Chat Endpoints
# ============================================================================

@app.post("/chat", response_model=ChatResponse)
async def chat(request: ChatRequest):
    """
    Send a chat message and get a response.
    
    Creates a new conversation if conversation_id is not provided.
    Maintains conversation history for follow-up messages.
    """
    try:
        # Generate conversation ID if not provided
        conversation_id = request.conversation_id or f"conv_{uuid.uuid4().hex[:8]}"
        
        # Create conversation with system prompt if provided
        if request.system_prompt and conversation_id not in chat_service.conversations:
            chat_service.create_conversation(
                conversation_id,
                provider_name=request.provider,
                system_prompt=request.system_prompt,
            )
        
        # Get response
        response = await chat_service.chat(
            conversation_id=conversation_id,
            user_message=request.message,
            provider_name=request.provider,
            model=request.model,
            temperature=request.temperature,
        )
        
        return ChatResponse(
            conversation_id=conversation_id,
            content=response.content,
            model=response.model,
            provider=response.provider,
            usage=response.usage,
        )
    
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Chat error: {str(e)}")


@app.post("/chat/direct")
async def chat_direct(request: ChatMessageRequest):
    """
    Direct chat without conversation history.
    
    Useful for one-off queries or when managing history client-side.
    """
    try:
        messages = [Message(role=m["role"], content=m["content"]) for m in request.messages]
        provider = chat_service.get_provider(request.provider or "ollama")
        
        response = await provider.chat(
            messages=messages,
            model=request.model,
            temperature=request.temperature,
        )
        
        return {
            "content": response.content,
            "model": response.model,
            "provider": response.provider,
            "usage": response.usage,
        }
    
    except ValueError as e:
        raise HTTPException(status_code=400, detail=str(e))
    except Exception as e:
        raise HTTPException(status_code=500, detail=f"Chat error: {str(e)}")


@app.delete("/chat/{conversation_id}")
async def delete_conversation(conversation_id: str):
    """Delete a conversation"""
    chat_service.delete_conversation(conversation_id)
    return {"status": "deleted", "conversation_id": conversation_id}


@app.post("/chat/{conversation_id}/clear")
async def clear_conversation(conversation_id: str):
    """Clear a conversation's history"""
    chat_service.clear_conversation(conversation_id)
    return {"status": "cleared", "conversation_id": conversation_id}


# ============================================================================
# Main Entry Point
# ============================================================================

if __name__ == "__main__":
    import uvicorn
    
    host = os.getenv("HOST", "127.0.0.1")
    port = int(os.getenv("PORT", "8765"))
    
    print(f"\n🚀 Starting AI Studio Agent Server")
    print(f"   URL: http://{host}:{port}")
    print(f"   Docs: http://{host}:{port}/docs")
    print(f"\n   Press Ctrl+C to stop\n")
    
    uvicorn.run(app, host=host, port=port)
