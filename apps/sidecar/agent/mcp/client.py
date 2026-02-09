"""
MCP Client Manager
==================
Manages connections to external MCP servers (stdio transport).
Discovers tools and registers them in the ToolRegistry.

Phase 1: stdio transport only.
Phase 2: SSE + Streamable HTTP.
"""

import asyncio
import json
import sys
from dataclasses import dataclass, field
from typing import Any, Optional

from .registry import ToolRegistry, ToolDefinition


@dataclass
class McpServerConfig:
    """Configuration for connecting to an MCP server."""
    name: str
    transport: str = "stdio"
    command: str | None = None
    args: list[str] = field(default_factory=list)
    url: str | None = None
    env: dict[str, str] = field(default_factory=dict)


@dataclass
class McpConnection:
    """A live connection to an MCP server."""
    config: McpServerConfig
    process: Optional[asyncio.subprocess.Process] = None
    tools: list[dict] = field(default_factory=list)
    request_id: int = 0

    def next_id(self) -> int:
        self.request_id += 1
        return self.request_id


class McpClientManager:
    """
    Manages connections to external MCP servers.

    Usage:
        manager = McpClientManager(registry)
        await manager.connect(McpServerConfig(name="fs", command="npx", args=["@modelcontextprotocol/server-filesystem", "/tmp"]))
        # Tools from "fs" server are now in the registry
        result = await manager.call_tool("fs", "read_file", {"path": "/tmp/test.txt"})
        await manager.disconnect("fs")
    """

    def __init__(self, registry: ToolRegistry):
        self.registry = registry
        self._connections: dict[str, McpConnection] = {}

    async def connect(self, config: McpServerConfig) -> dict:
        """
        Connect to an MCP server, discover its tools, and register them.
        Returns {"status": "connected", "tools": [...tool_names...]}.
        """
        if config.transport != "stdio":
            return {"status": "error", "error": f"Transport '{config.transport}' not yet supported (Phase 2)"}

        if not config.command:
            return {"status": "error", "error": "No command specified for stdio transport"}

        # Disconnect existing connection if any
        if config.name in self._connections:
            await self.disconnect(config.name)

        try:
            # Spawn the MCP server subprocess
            cmd = [config.command] + config.args
            import os
            env = {**os.environ, **config.env}

            process = await asyncio.create_subprocess_exec(
                *cmd,
                stdin=asyncio.subprocess.PIPE,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
                env=env,
            )

            conn = McpConnection(config=config, process=process)
            self._connections[config.name] = conn

            # Initialize MCP protocol
            await self._send_jsonrpc(conn, "initialize", {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": "ai-studio", "version": "0.1.0"},
            })

            # Send initialized notification
            await self._send_notification(conn, "notifications/initialized", {})

            # Discover tools
            tools_response = await self._send_jsonrpc(conn, "tools/list", {})
            tools = tools_response.get("tools", [])
            conn.tools = tools

            # Register tools in registry
            for tool in tools:
                self.registry.register(ToolDefinition(
                    server=config.name,
                    name=tool["name"],
                    description=tool.get("description", ""),
                    input_schema=tool.get("inputSchema", {"type": "object", "properties": {}}),
                    handler=None,  # External tools use call_tool() instead
                ))

            tool_names = [t["name"] for t in tools]
            print(f"[mcp] Connected to '{config.name}': {len(tools)} tools discovered: {tool_names}")
            return {"status": "connected", "tools": tool_names}

        except Exception as e:
            # Clean up on failure
            if config.name in self._connections:
                conn = self._connections.pop(config.name)
                if conn.process:
                    conn.process.kill()
            error_msg = str(e)
            print(f"[mcp] Failed to connect to '{config.name}': {error_msg}", file=sys.stderr)
            return {"status": "error", "error": error_msg}

    async def disconnect(self, name: str):
        """Disconnect from an MCP server and unregister its tools."""
        conn = self._connections.pop(name, None)
        if conn and conn.process:
            try:
                conn.process.stdin.close()
                conn.process.kill()
                await conn.process.wait()
            except Exception:
                pass
        self.registry.unregister_server(name)
        print(f"[mcp] Disconnected from '{name}'")

    async def call_tool(self, server_name: str, tool_name: str, arguments: dict) -> str:
        """Execute a tool on an MCP server. Returns the result as a string."""
        conn = self._connections.get(server_name)
        if not conn or not conn.process:
            return f"Error: MCP server '{server_name}' not connected"

        try:
            result = await self._send_jsonrpc(conn, "tools/call", {
                "name": tool_name,
                "arguments": arguments,
            })

            # MCP tools return content array
            content = result.get("content", [])
            texts = []
            for item in content:
                if item.get("type") == "text":
                    texts.append(item.get("text", ""))
                elif item.get("type") == "image":
                    texts.append("[image data]")
                else:
                    texts.append(json.dumps(item))
            return "\n".join(texts) or "(no output)"

        except Exception as e:
            return f"Error executing tool: {e}"

    def get_connected_servers(self) -> list[str]:
        """List names of connected MCP servers."""
        return list(self._connections.keys())

    def is_connected(self, name: str) -> bool:
        """Check if an MCP server is connected."""
        return name in self._connections

    async def _send_jsonrpc(self, conn: McpConnection, method: str, params: dict) -> dict:
        """Send a JSON-RPC request and wait for the response."""
        request_id = conn.next_id()
        request = {
            "jsonrpc": "2.0",
            "id": request_id,
            "method": method,
            "params": params,
        }

        line = json.dumps(request) + "\n"
        conn.process.stdin.write(line.encode())
        await conn.process.stdin.drain()

        # Read response lines until we get one with matching id
        while True:
            raw = await asyncio.wait_for(conn.process.stdout.readline(), timeout=30.0)
            if not raw:
                raise ConnectionError(f"MCP server '{conn.config.name}' closed connection")
            try:
                response = json.loads(raw.decode().strip())
            except json.JSONDecodeError:
                continue  # Skip non-JSON lines (e.g., stderr leaking)

            if response.get("id") == request_id:
                if "error" in response:
                    err = response["error"]
                    raise RuntimeError(f"MCP error: {err.get('message', err)}")
                return response.get("result", {})

    async def _send_notification(self, conn: McpConnection, method: str, params: dict):
        """Send a JSON-RPC notification (no response expected)."""
        notification = {
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        }
        line = json.dumps(notification) + "\n"
        conn.process.stdin.write(line.encode())
        await conn.process.stdin.drain()

    async def shutdown(self):
        """Disconnect all servers. Call on app shutdown."""
        for name in list(self._connections.keys()):
            await self.disconnect(name)
