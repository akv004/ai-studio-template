"""
Chat Service
============
High-level chat interface with conversation memory and tool execution.

When tools are available, runs a multi-turn loop:
  LLM → tool_use → execute tool → feed result → LLM → ... → final text
"""

import time
import json
from typing import Optional
from dataclasses import dataclass, field
from agent.providers import AgentProvider, Message, ChatResponse, OllamaProvider, LocalOpenAIProvider


@dataclass
class ToolCallRecord:
    """Record of a single tool call during a chat turn."""
    tool_call_id: str
    tool_name: str        # qualified name: "builtin__shell"
    display_name: str     # human-readable: "builtin:shell"
    tool_input: dict
    tool_output: str
    duration_ms: int
    error: Optional[str] = None


@dataclass
class ChatResult:
    """Result of a chat turn, including any tool calls that happened."""
    response: ChatResponse
    tool_calls: list[ToolCallRecord] = field(default_factory=list)
    total_input_tokens: int = 0
    total_output_tokens: int = 0


@dataclass
class Conversation:
    """Represents a conversation with history"""
    id: str
    messages: list[Message] = field(default_factory=list)
    provider_name: str = "ollama"
    model: Optional[str] = None


MAX_TOOL_TURNS = 10  # Safety limit to prevent infinite loops


class ChatService:
    """
    Chat service with conversation memory, provider management, and tool execution.

    Example:
        service = ChatService()
        result = await service.chat_with_tools("conv_123", "List files in /tmp", registry=tool_registry)
    """

    def __init__(self):
        self.providers: dict[str, AgentProvider] = {}
        self.conversations: dict[str, Conversation] = {}
        self.default_provider = "ollama"

        # Register default providers
        self.register_provider(OllamaProvider())
        self.register_provider(LocalOpenAIProvider())

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
        tools: Optional[list[dict]] = None,
    ) -> ChatResponse:
        """
        Simple chat — no tool execution loop. Backward compatible.
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
            tools=tools,
        )

        # Add assistant response to history
        conv.messages.append(Message(role="assistant", content=response.content))

        return response

    async def chat_with_tools(
        self,
        conversation_id: str,
        user_message: str,
        provider_name: Optional[str] = None,
        model: Optional[str] = None,
        temperature: float = 0.7,
        tool_definitions: Optional[list[dict]] = None,
        tool_registry=None,
        mcp_client=None,
        event_bus=None,
    ) -> ChatResult:
        """
        Chat with tool execution loop.

        When the LLM returns tool_use, this method:
        1. Executes the tool (via registry handler or MCP client)
        2. Feeds the result back to the LLM
        3. Repeats until the LLM gives a final text response

        Returns ChatResult with the final response + all tool calls that happened.
        """
        conv = self.get_or_create_conversation(conversation_id)

        # Add user message
        conv.messages.append(Message(role="user", content=user_message))

        provider = self.get_provider(provider_name or conv.provider_name)
        all_tool_calls: list[ToolCallRecord] = []
        total_input = 0
        total_output = 0

        for turn in range(MAX_TOOL_TURNS):
            # Emit llm.request.started
            if event_bus:
                await event_bus.emit(
                    "llm.request.started", conversation_id, "sidecar.chat",
                    {"model": model or conv.model or "", "provider": provider.name, "turn": turn},
                )

            llm_start = time.monotonic()
            try:
                response = await provider.chat(
                    messages=conv.messages,
                    model=model or conv.model,
                    temperature=temperature,
                    tools=tool_definitions,
                )
            except Exception as e:
                llm_duration_ms = int((time.monotonic() - llm_start) * 1000)
                if event_bus:
                    await event_bus.emit(
                        "llm.response.error", conversation_id, "sidecar.chat",
                        {
                            "error": str(e),
                            "error_code": type(e).__name__,
                            "model": model or conv.model or "",
                            "provider": provider.name,
                            "duration_ms": llm_duration_ms,
                            "turn": turn,
                        },
                    )
                raise
            llm_duration_ms = int((time.monotonic() - llm_start) * 1000)

            input_toks = (response.usage or {}).get("prompt_tokens", 0)
            output_toks = (response.usage or {}).get("completion_tokens", 0)
            total_input += input_toks
            total_output += output_toks

            # Emit llm.response.completed
            if event_bus:
                await event_bus.emit(
                    "llm.response.completed", conversation_id, "sidecar.chat",
                    {
                        "model": response.model,
                        "provider": response.provider,
                        "input_tokens": input_toks,
                        "output_tokens": output_toks,
                        "duration_ms": llm_duration_ms,
                        "stop_reason": response.stop_reason or ("tool_use" if response.tool_calls else "end_turn"),
                        "content": response.content[:500] if response.content else "",
                    },
                )

            # If no tool calls, we're done
            if not response.tool_calls:
                conv.messages.append(Message(role="assistant", content=response.content))
                return ChatResult(
                    response=response,
                    tool_calls=all_tool_calls,
                    total_input_tokens=total_input,
                    total_output_tokens=total_output,
                )

            # LLM wants to call tools — execute them
            # First, add the assistant message with tool calls to history
            if provider.name == "anthropic":
                # Anthropic needs the raw content blocks in the assistant message
                conv.messages.append(Message(
                    role="assistant",
                    content=response.raw_content or response.content,
                ))
            elif provider.name == "google":
                # Google needs the raw parts
                conv.messages.append(Message(
                    role="assistant",
                    content=response.raw_content or response.content,
                ))
            else:
                conv.messages.append(Message(role="assistant", content=response.content))

            # Execute each tool call
            tool_results = []
            for tc in response.tool_calls:
                tool_name = tc["name"]       # e.g., "builtin__shell"
                tool_input = tc["input"]
                tool_id = tc["id"]

                # Parse the qualified name
                parts = tool_name.split("__", 1)
                server = parts[0] if len(parts) == 2 else "unknown"
                local_name = parts[1] if len(parts) == 2 else tool_name
                display_name = f"{server}:{local_name}"

                # Emit tool.requested
                if event_bus:
                    await event_bus.emit(
                        "tool.requested", conversation_id, "sidecar.tools",
                        {"tool_call_id": tool_id, "tool_name": display_name, "tool_input": tool_input},
                    )

                start = time.monotonic()
                output = ""
                error = None

                try:
                    if tool_registry:
                        tool_def = tool_registry.resolve(tool_name)
                        if tool_def and tool_def.handler:
                            output = await tool_def.handler(**tool_input)
                        elif mcp_client and mcp_client.is_connected(server):
                            output = await mcp_client.call_tool(server, local_name, tool_input)
                        else:
                            output = f"Error: Tool '{tool_name}' not found or not connected"
                            error = output
                    else:
                        output = f"Error: No tool registry available"
                        error = output
                except Exception as e:
                    output = f"Error executing tool: {e}"
                    error = str(e)

                duration_ms = int((time.monotonic() - start) * 1000)

                # Emit tool.completed or tool.error
                if event_bus:
                    if error:
                        await event_bus.emit(
                            "tool.error", conversation_id, "sidecar.tools",
                            {"tool_call_id": tool_id, "tool_name": display_name, "error": error, "duration_ms": duration_ms},
                        )
                    else:
                        await event_bus.emit(
                            "tool.completed", conversation_id, "sidecar.tools",
                            {"tool_call_id": tool_id, "tool_name": display_name, "output": output[:1000], "duration_ms": duration_ms},
                        )

                record = ToolCallRecord(
                    tool_call_id=tool_id,
                    tool_name=tool_name,
                    display_name=display_name,
                    tool_input=tool_input,
                    tool_output=output,
                    duration_ms=duration_ms,
                    error=error,
                )
                all_tool_calls.append(record)
                tool_results.append((tool_id, tool_name, local_name, output))

                print(f"[tool] {display_name}: {json.dumps(tool_input)[:100]} → {output[:100]}")

            # Feed tool results back to the LLM
            if provider.name == "anthropic":
                # Anthropic expects tool results as a user message with tool_result blocks
                result_blocks = []
                for tool_id, _, _, output in tool_results:
                    result_blocks.append({
                        "type": "tool_result",
                        "tool_use_id": tool_id,
                        "content": output,
                    })
                conv.messages.append(Message(role="user", content=result_blocks))

            elif provider.name == "google":
                # Gemini expects function response parts
                result_parts = []
                for _, _, local_name, output in tool_results:
                    result_parts.append({
                        "functionResponse": {
                            "name": local_name,
                            "response": {"result": output},
                        }
                    })
                conv.messages.append(Message(role="tool", content=result_parts))

            else:
                # Generic fallback — append tool output as user message
                for _, tool_name, _, output in tool_results:
                    conv.messages.append(Message(
                        role="user",
                        content=f"Tool result for {tool_name}:\n{output}",
                    ))

        # If we exhausted MAX_TOOL_TURNS, return last response
        return ChatResult(
            response=response,
            tool_calls=all_tool_calls,
            total_input_tokens=total_input,
            total_output_tokens=total_output,
        )

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
