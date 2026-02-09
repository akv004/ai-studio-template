"""
Built-in Tools
==============
Wraps the existing shell + filesystem tools as MCP-compatible ToolDefinitions.
These run in-process (no subprocess overhead).
"""

import json
from .registry import ToolRegistry, ToolDefinition
from agent.tools import ShellTool, FilesystemTool


def register_builtin_tools(
    registry: ToolRegistry,
    shell_tool: ShellTool,
    fs_tool: FilesystemTool,
):
    """Register built-in shell and filesystem tools."""

    async def handle_shell(command: str, timeout: float = 30.0, cwd: str | None = None) -> str:
        result = await shell_tool.run(command=command, timeout=timeout, cwd=cwd)
        output = ""
        if result.stdout:
            output += result.stdout
        if result.stderr:
            output += f"\n[stderr] {result.stderr}"
        if result.timed_out:
            output += "\n[timed out]"
        if result.return_code != 0:
            output += f"\n[exit code: {result.return_code}]"
        return output.strip() or "(no output)"

    async def handle_read_file(path: str) -> str:
        result = fs_tool.read(path)
        if result.success:
            return result.data or "(empty file)"
        return f"Error: {result.error}"

    async def handle_write_file(path: str, content: str) -> str:
        result = fs_tool.write(path, content)
        if result.success:
            return f"Written to {path}"
        return f"Error: {result.error}"

    async def handle_list_dir(path: str) -> str:
        result = fs_tool.list_dir(path)
        if result.success:
            return json.dumps(result.data) if result.data else "(empty directory)"
        return f"Error: {result.error}"

    registry.register_many([
        ToolDefinition(
            server="builtin",
            name="shell",
            description="Execute a shell command and return stdout/stderr. Use for running programs, checking system state, git operations, etc.",
            input_schema={
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "The shell command to execute",
                    },
                    "timeout": {
                        "type": "number",
                        "description": "Timeout in seconds (default: 30)",
                        "default": 30.0,
                    },
                    "cwd": {
                        "type": "string",
                        "description": "Working directory for the command (optional)",
                    },
                },
                "required": ["command"],
            },
            handler=handle_shell,
        ),
        ToolDefinition(
            server="builtin",
            name="read_file",
            description="Read the contents of a file at the given path.",
            input_schema={
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Absolute or relative file path to read",
                    },
                },
                "required": ["path"],
            },
            handler=handle_read_file,
        ),
        ToolDefinition(
            server="builtin",
            name="write_file",
            description="Write content to a file, creating it if it doesn't exist.",
            input_schema={
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "File path to write to",
                    },
                    "content": {
                        "type": "string",
                        "description": "Content to write",
                    },
                },
                "required": ["path", "content"],
            },
            handler=handle_write_file,
        ),
        ToolDefinition(
            server="builtin",
            name="list_directory",
            description="List files and directories at the given path.",
            input_schema={
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Directory path to list",
                    },
                },
                "required": ["path"],
            },
            handler=handle_list_dir,
        ),
    ])
