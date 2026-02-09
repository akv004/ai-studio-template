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

from fastapi import FastAPI, HTTPException, Request
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse
from pydantic import BaseModel

from agent.chat import ChatService
from agent.providers import (
    OllamaProvider,
    AnthropicProvider,
    OpenAIProvider,
    GoogleProvider,
    AzureOpenAIProvider,
    LocalOpenAIProvider,
    AgentProvider,
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
    # Per-request provider config (passed from Rust, loaded from SQLite settings)
    api_key: Optional[str] = None
    base_url: Optional[str] = None
    extra_config: Optional[dict] = None


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
    
    # Configure Google AI / Gemini (if API key is set)
    if os.getenv("GOOGLE_API_KEY"):
        chat_service.register_provider(GoogleProvider())
        print("✓ Google AI (Gemini) provider enabled")
    
    print(f"🤖 AI Studio Sidecar initialized")
    print(f"   Ollama: {ollama_host} (model: {ollama_model})")
    
    yield


app = FastAPI(
    title="AI Studio Agent",
    description="Multi-provider LLM agent server",
    version="0.1.0",
    lifespan=lifespan,
)

# Optional auth token injected by the desktop app.
AI_STUDIO_TOKEN = os.getenv("AI_STUDIO_TOKEN")

@app.middleware("http")
async def auth_middleware(request: Request, call_next):
    if not AI_STUDIO_TOKEN:
        return await call_next(request)

    # Allow unauthenticated health checks (used for startup probing).
    if request.url.path == "/health":
        return await call_next(request)

    token = request.headers.get("x-ai-studio-token")
    if token != AI_STUDIO_TOKEN:
        return JSONResponse(status_code=401, content={"detail": "Unauthorized"})

    return await call_next(request)

# CORS for Tauri/React frontend (used mainly when running UI in a browser).
app.add_middleware(
    CORSMiddleware,
    allow_origins=["tauri://localhost", "http://localhost:1420"] if AI_STUDIO_TOKEN else ["*"],
    allow_credentials=False,
    allow_methods=["*"],
    allow_headers=["*"],
)


# ============================================================================
# Dynamic Provider Creation
# ============================================================================

def create_provider_for_request(
    name: str,
    api_key: Optional[str] = None,
    base_url: Optional[str] = None,
    extra_config: Optional[dict] = None,
) -> AgentProvider:
    """Create a provider instance from per-request config."""
    cfg = extra_config or {}

    if name == "anthropic":
        return AnthropicProvider(api_key=api_key or "")
    elif name == "google":
        return GoogleProvider(api_key=api_key or "")
    elif name == "azure_openai":
        return AzureOpenAIProvider(
            endpoint=base_url or cfg.get("endpoint", ""),
            api_key=api_key or "",
            deployment=cfg.get("deployment", ""),
            api_version=cfg.get("api_version", "2024-02-01"),
        )
    elif name == "local":
        return LocalOpenAIProvider(
            base_url=base_url or cfg.get("base_url", "http://localhost:11434/v1"),
            api_key=api_key or "",
            model_name=cfg.get("model_name", ""),
        )
    elif name == "openai":
        return OpenAIProvider(api_key=api_key or "")
    elif name == "ollama":
        return OllamaProvider(
            base_url=base_url or "http://localhost:11434",
            default_model=cfg.get("model_name", "llama3.2"),
        )
    else:
        raise ValueError(f"Unknown provider: {name}")


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
# Provider Test Endpoint
# ============================================================================

class ProviderTestRequest(BaseModel):
    """Test a provider connection with given credentials"""
    provider: str
    api_key: Optional[str] = None
    base_url: Optional[str] = None
    extra_config: Optional[dict] = None


@app.post("/providers/test")
async def test_provider(request: ProviderTestRequest):
    """Test if a provider connection works with the given config."""
    try:
        provider = create_provider_for_request(
            name=request.provider,
            api_key=request.api_key,
            base_url=request.base_url,
            extra_config=request.extra_config,
        )
        healthy = await provider.health()
        return {"success": healthy, "message": "Connected" if healthy else "Health check failed"}
    except Exception as e:
        return {"success": False, "message": str(e)}


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

        # If per-request API key provided, create and register provider on-the-fly
        if request.api_key and request.provider:
            provider = create_provider_for_request(
                name=request.provider,
                api_key=request.api_key,
                base_url=request.base_url,
                extra_config=request.extra_config,
            )
            chat_service.register_provider(provider)

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
# Tools Endpoints
# ============================================================================

from agent.tools import ShellTool, BrowserTool, FilesystemTool

# Initialize tools with restricted mode by default
shell_tool = ShellTool(mode=os.getenv("TOOLS_MODE", "restricted"))
filesystem_tool = FilesystemTool(mode=os.getenv("TOOLS_MODE", "restricted"))
browser_tool = BrowserTool(headless=True)


class ShellRequest(BaseModel):
    """Shell command request"""
    command: str
    timeout: Optional[float] = 30.0
    cwd: Optional[str] = None


class FilesystemRequest(BaseModel):
    """Filesystem operation request"""
    action: str  # read, write, delete, list_dir, mkdir, exists, copy, move
    path: str
    content: Optional[str] = None  # For write/append
    dest: Optional[str] = None  # For copy/move


class BrowserRequest(BaseModel):
    """Browser action request"""
    action: str  # navigate, screenshot, click, fill, extract_text, get_html, evaluate
    url: Optional[str] = None
    selector: Optional[str] = None
    value: Optional[str] = None
    script: Optional[str] = None


@app.post("/tools/shell")
async def run_shell(request: ShellRequest):
    """
    Execute a shell command.
    
    Mode is controlled by TOOLS_MODE env var: sandboxed, restricted (default), full
    """
    result = await shell_tool.run(
        command=request.command,
        timeout=request.timeout,
        cwd=request.cwd,
    )
    return {
        "command": result.command,
        "stdout": result.stdout,
        "stderr": result.stderr,
        "return_code": result.return_code,
        "timed_out": result.timed_out,
    }


@app.post("/tools/filesystem")
async def run_filesystem(request: FilesystemRequest):
    """
    Perform filesystem operations.
    
    Actions: read, write, append, delete, list_dir, mkdir, exists, copy, move
    """
    action = request.action.lower()
    
    if action == "read":
        result = filesystem_tool.read(request.path)
    elif action == "write":
        result = filesystem_tool.write(request.path, request.content or "")
    elif action == "append":
        result = filesystem_tool.append(request.path, request.content or "")
    elif action == "delete":
        result = filesystem_tool.delete(request.path)
    elif action == "list_dir":
        result = filesystem_tool.list_dir(request.path)
    elif action == "mkdir":
        result = filesystem_tool.mkdir(request.path)
    elif action == "exists":
        result = filesystem_tool.exists(request.path)
    elif action == "copy":
        result = filesystem_tool.copy(request.path, request.dest or "")
    elif action == "move":
        result = filesystem_tool.move(request.path, request.dest or "")
    else:
        raise HTTPException(status_code=400, detail=f"Unknown action: {action}")
    
    return {
        "success": result.success,
        "action": result.action,
        "path": result.path,
        "data": result.data,
        "error": result.error,
    }


@app.post("/tools/browser/start")
async def browser_start():
    """Start the browser instance"""
    result = await browser_tool.start()
    return {"success": result.success, "error": result.error}


@app.post("/tools/browser/stop")
async def browser_stop():
    """Stop the browser instance"""
    result = await browser_tool.stop()
    return {"success": result.success, "error": result.error}


@app.post("/tools/browser")
async def run_browser(request: BrowserRequest):
    """
    Perform browser actions.
    
    Actions: navigate, screenshot, click, fill, extract_text, get_html, evaluate, wait_for
    """
    action = request.action.lower()
    
    if action == "navigate":
        result = await browser_tool.navigate(request.url or "")
    elif action == "screenshot":
        result = await browser_tool.screenshot()
    elif action == "click":
        result = await browser_tool.click(request.selector or "")
    elif action == "fill":
        result = await browser_tool.fill(request.selector or "", request.value or "")
    elif action == "extract_text":
        result = await browser_tool.extract_text(request.selector or "body")
    elif action == "get_html":
        result = await browser_tool.get_html(request.selector or "body")
    elif action == "evaluate":
        result = await browser_tool.evaluate(request.script or "")
    elif action == "wait_for":
        result = await browser_tool.wait_for(request.selector or "")
    else:
        raise HTTPException(status_code=400, detail=f"Unknown action: {action}")
    
    return {
        "success": result.success,
        "action": result.action,
        "data": result.data,
        "screenshot": result.screenshot,
        "error": result.error,
    }


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
