"""
Tool Registry
=============
Central registry for all available tools (built-in + external MCP servers).
Tools are namespaced: "server_name:tool_name" → LLM sees "server_name__tool_name".
"""

from dataclasses import dataclass, field
from typing import Any, Callable, Awaitable, Optional


@dataclass
class ToolDefinition:
    """A tool that can be used by the LLM."""
    server: str           # e.g., "builtin", "github", "filesystem"
    name: str             # e.g., "shell", "create_issue"
    description: str
    input_schema: dict    # JSON Schema for the tool's input
    handler: Optional[Callable[..., Awaitable[str]]] = None  # For built-in tools

    @property
    def qualified_name(self) -> str:
        """Namespaced name: server__tool_name (double underscore for LLM compat)."""
        return f"{self.server}__{self.name}"

    @property
    def display_name(self) -> str:
        """Human-readable: server:tool_name."""
        return f"{self.server}:{self.name}"

    def to_anthropic_tool(self) -> dict:
        """Convert to Anthropic API tool format."""
        return {
            "name": self.qualified_name,
            "description": self.description,
            "input_schema": self.input_schema,
        }

    def to_google_tool(self) -> dict:
        """Convert to Google Gemini function declaration format."""
        # Gemini uses a different schema format — clean out unsupported fields
        schema = dict(self.input_schema)
        schema.pop("additionalProperties", None)
        return {
            "name": self.qualified_name,
            "description": self.description,
            "parameters": schema,
        }


class ToolRegistry:
    """
    Manages all available tools across built-in and MCP servers.

    Usage:
        registry = ToolRegistry()
        register_builtin_tools(registry, shell_tool, fs_tool)
        # Later: registry.register_mcp_tools("github", [...])

        tools = registry.get_tool_definitions()  # For sending to LLM
        tool = registry.resolve("builtin__shell")  # For executing
    """

    def __init__(self):
        self._tools: dict[str, ToolDefinition] = {}  # key: qualified_name

    def register(self, tool: ToolDefinition):
        """Register a single tool."""
        self._tools[tool.qualified_name] = tool

    def register_many(self, tools: list[ToolDefinition]):
        """Register multiple tools."""
        for tool in tools:
            self.register(tool)

    def unregister_server(self, server: str):
        """Remove all tools from a specific server."""
        to_remove = [k for k, v in self._tools.items() if v.server == server]
        for k in to_remove:
            del self._tools[k]

    def resolve(self, qualified_name: str) -> Optional[ToolDefinition]:
        """Look up a tool by its qualified name (server__tool_name)."""
        return self._tools.get(qualified_name)

    def get_all(self) -> list[ToolDefinition]:
        """Get all registered tools."""
        return list(self._tools.values())

    def get_for_server(self, server: str) -> list[ToolDefinition]:
        """Get tools from a specific server."""
        return [t for t in self._tools.values() if t.server == server]

    def get_anthropic_tools(self) -> list[dict]:
        """Get all tools in Anthropic API format."""
        return [t.to_anthropic_tool() for t in self._tools.values()]

    def get_google_tools(self) -> list[dict]:
        """Get all tools in Google Gemini function declaration format."""
        return [t.to_google_tool() for t in self._tools.values()]

    def to_summary(self) -> list[dict]:
        """Summary for the /mcp/tools endpoint."""
        return [
            {
                "server": t.server,
                "name": t.name,
                "qualified_name": t.qualified_name,
                "description": t.description,
            }
            for t in self._tools.values()
        ]
