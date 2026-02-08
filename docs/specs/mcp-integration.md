# AI Studio — MCP Integration Specification

> **Version**: 2.0
> **Status**: Draft
> **Depends on**: product-vision.md, architecture.md, event-system.md

---

## What Is MCP?

**Model Context Protocol (MCP)** is an open standard by Anthropic that defines how AI applications connect to external tools and data sources. It's becoming the universal standard for AI tool integration.

**Why MCP-native matters for AI Studio:**
- Any MCP-compatible tool works in AI Studio instantly (growing ecosystem)
- Users can bring tools from Claude Desktop, Cursor, Zed, and other MCP-supporting apps
- AI Studio becomes interoperable rather than a walled garden
- The community can build tools once and they work everywhere

**AI Studio's role**: MCP **client**. It connects to MCP **servers** that provide tools.

---

## Architecture

```
┌──────────────────────────────────────────────────┐
│                AI Studio (MCP Client)             │
│                                                  │
│  ┌────────────┐                                  │
│  │  Sidecar   │──── MCP Protocol ──── Server A   │
│  │  (Python)  │──── MCP Protocol ──── Server B   │
│  │            │──── MCP Protocol ──── Server C   │
│  │  MCP       │                                  │
│  │  Client    │     Built-in tools also exposed  │
│  │  Manager   │     as internal MCP-compatible   │
│  └────────────┘     handlers (shell, fs, browser)│
└──────────────────────────────────────────────────┘
```

### Where MCP Lives

The **Python sidecar** manages all MCP connections. Why:
- MCP SDK is Python-native (`mcp` package)
- The sidecar already handles LLM calls — it needs tool access in the same process
- Tool discovery feeds directly into LLM tool definitions
- Tauri doesn't need to know MCP protocol details — it just sees tool events

### Config Lives in Tauri

MCP server configurations are stored in SQLite (see `mcp_servers` table in data-model.md). Tauri passes the config to the sidecar on startup or when the user adds/removes servers.

---

## MCP Transport Support

MCP defines three transport mechanisms. AI Studio supports all three:

| Transport | How It Works | Use Case |
|---|---|---|
| **stdio** | Sidecar spawns a subprocess, communicates via stdin/stdout | Local tools (most common) |
| **SSE** | HTTP Server-Sent Events connection | Remote servers, long-lived connections |
| **Streamable HTTP** | HTTP POST/GET with streaming | Modern remote servers |

### stdio (Primary — Most MCP Servers Use This)

```
AI Studio Sidecar
    │
    ├── spawns → npx @modelcontextprotocol/server-filesystem /path
    ├── spawns → python -m mcp_server_git --repo /path
    └── spawns → npx @anthropic/mcp-server-github
```

The sidecar manages subprocess lifecycles:
- Spawn on connection
- Kill on disconnect
- Restart on crash (with backoff)
- Environment variables passed from config

### SSE / Streamable HTTP

For remote MCP servers (e.g., a team-hosted tool server):
- Sidecar opens HTTP connection to the server URL
- Maintains connection for the lifetime of the session
- Reconnects on failure

---

## MCP Server Configuration

### User Configuration (Settings UI)

Users add MCP servers in Settings → MCP Servers:

```
┌─────────────────────────────────────────────────┐
│  MCP Servers                          [+ Add]   │
│                                                 │
│  ┌───────────────────────────────────────────┐  │
│  │ 🟢 Filesystem                             │  │
│  │    stdio: npx @modelcontextprotocol/...   │  │
│  │    Tools: read_file, write_file, ...  (8) │  │
│  │    [Edit] [Disable] [Remove]              │  │
│  ├───────────────────────────────────────────┤  │
│  │ 🟢 GitHub                                 │  │
│  │    stdio: npx @anthropic/mcp-server-...   │  │
│  │    Tools: create_issue, list_prs, ... (12)│  │
│  │    [Edit] [Disable] [Remove]              │  │
│  ├───────────────────────────────────────────┤  │
│  │ 🔴 Custom RAG Server                      │  │
│  │    sse: http://localhost:3001/mcp          │  │
│  │    Error: Connection refused               │  │
│  │    [Edit] [Retry] [Remove]                │  │
│  └───────────────────────────────────────────┘  │
└─────────────────────────────────────────────────┘
```

### Add Server Dialog

```
┌─────────────────────────────────────────────┐
│  Add MCP Server                             │
│                                             │
│  Name:      [GitHub Tools              ]    │
│                                             │
│  Transport: [stdio ▾]                       │
│                                             │
│  Command:   [npx                       ]    │
│  Args:      [@anthropic/mcp-server-github]  │
│                                             │
│  Environment Variables:                     │
│  GITHUB_TOKEN = [ghp_xxxx              ]    │
│  [+ Add Variable]                           │
│                                             │
│            [Cancel] [Test Connection] [Add] │
└─────────────────────────────────────────────┘
```

### Configuration Schema (Stored in SQLite)

```typescript
interface McpServerConfig {
  id: string;           // UUID
  name: string;         // Display name
  transport: 'stdio' | 'sse' | 'streamable-http';

  // For stdio
  command?: string;     // e.g., "npx", "python", "node"
  args?: string[];      // e.g., ["@anthropic/mcp-server-github"]

  // For sse / streamable-http
  url?: string;         // e.g., "http://localhost:3001/mcp"

  // Common
  env?: Record<string, string>;  // Environment variables
  enabled: boolean;
}
```

---

## Tool Discovery

When a MCP server connects, the sidecar queries it for available tools:

```
Sidecar → MCP Server: tools/list
MCP Server → Sidecar: [
  {
    name: "create_issue",
    description: "Create a GitHub issue",
    inputSchema: { ... JSON Schema ... }
  },
  ...
]
```

The sidecar maintains an in-memory registry of all discovered tools across all connected servers:

```python
class McpToolRegistry:
    """Registry of all available MCP tools."""

    tools: dict[str, McpTool]  # key: "server_name:tool_name"

    def get_tools_for_agent(self, agent_config: Agent) -> list[McpTool]:
        """Return tools available to a specific agent based on its mcp_servers config."""

    def get_tool_definitions(self, tools: list[McpTool]) -> list[dict]:
        """Convert to LLM tool format (OpenAI/Anthropic tool_use schema)."""
```

### Tool Naming Convention

Tools are namespaced by server to avoid collisions:

```
{server_name}:{tool_name}
```

Examples:
- `github:create_issue`
- `filesystem:read_file`
- `custom-rag:search_documents`

In the LLM prompt, tools appear with their server prefix:
```json
{
  "name": "github__create_issue",
  "description": "Create a GitHub issue",
  "input_schema": { ... }
}
```

(Double underscore `__` replaces colon for LLM compatibility)

---

## Tool Execution Flow

When the LLM wants to use a tool:

```
1. LLM returns tool_use: { name: "github__create_issue", input: { title: "Bug", body: "..." } }

2. Sidecar parses tool name → resolves to server "github", tool "create_issue"

3. Sidecar emits event: tool.requested {
     tool_call_id: "tc_123",
     tool_name: "github:create_issue",
     tool_input: { title: "Bug", body: "..." },
     mcp_server: "github"
   }

4. Event flows to Tauri → approval check:
   - Check agent approval rules
   - Check global approval rules
   - If auto_approve → proceed
   - If ask → show UI modal
   - If auto_deny → deny

5. On approval, Tauri signals sidecar to proceed

6. Sidecar calls MCP server:
   Sidecar → MCP Server: tools/call { name: "create_issue", arguments: { ... } }
   MCP Server → Sidecar: { content: [{ type: "text", text: "Issue #42 created" }] }

7. Sidecar emits event: tool.completed {
     tool_call_id: "tc_123",
     tool_name: "github:create_issue",
     output: "Issue #42 created",
     duration_ms: 340
   }

8. Sidecar feeds tool result back to LLM for next response
```

---

## Built-in Tools as MCP-Compatible

The existing built-in tools (shell, filesystem, browser) are wrapped as **internal MCP-compatible handlers**. This means:

- They use the same tool execution flow as external MCP servers
- They appear in the agent's tool list alongside MCP tools
- They go through the same approval workflow
- The Inspector shows them identically to MCP tools

**They do NOT run as separate processes** (no subprocess overhead). They're registered as local handlers in the sidecar:

```python
class BuiltinToolProvider:
    """Wraps built-in tools with MCP-compatible interface."""

    async def call_tool(self, name: str, arguments: dict) -> str:
        if name == "shell":
            return await self.shell_tool.run(**arguments)
        elif name == "filesystem":
            return await self.filesystem_tool.run(**arguments)
        elif name == "browser":
            return await self.browser_tool.run(**arguments)
```

### Default Tool List

Every agent has these built-in tools available (unless disabled):

| Tool | Description | Default Approval |
|---|---|---|
| `builtin:shell` | Execute shell commands | Ask (unless matching auto-approve rule) |
| `builtin:filesystem_read` | Read files and directories | Auto-approve |
| `builtin:filesystem_write` | Write/create/delete files | Ask |
| `builtin:browser` | Browser automation (navigate, screenshot, etc.) | Ask |

---

## Agent ↔ MCP Server Binding

Each agent configuration specifies which MCP servers it can access:

```json
{
  "id": "agent_123",
  "name": "Code Assistant",
  "mcp_servers": ["github", "filesystem"],
  "approval_rules": [
    { "pattern": "builtin:filesystem_read:*", "action": "auto_approve" },
    { "pattern": "github:list_*", "action": "auto_approve" },
    { "pattern": "github:create_*", "action": "ask" }
  ]
}
```

When a session starts with this agent:
1. Sidecar connects to "github" and "filesystem" MCP servers (if not already connected)
2. Discovers tools from both
3. Includes built-in tools
4. Sends combined tool list to the LLM with the system prompt

---

## Connection Lifecycle

```
App Startup
    │
    ├── Tauri loads MCP server configs from SQLite
    ├── Passes configs to sidecar (via HTTP or startup env)
    │
    ▼
Sidecar MCP Manager
    │
    ├── For each enabled server:
    │   ├── stdio: spawn subprocess, initialize MCP protocol
    │   ├── sse/http: open connection
    │   └── Query tools/list
    │
    ├── On success: emit mcp.server.connected event
    ├── On failure: emit mcp.server.disconnected event, retry with backoff
    │
    ▼
Runtime
    │
    ├── Tools available for agent sessions
    ├── Health check loop (ping each server periodically)
    └── Reconnect on failure
    │
    ▼
App Shutdown
    │
    ├── Close all MCP connections
    └── Kill stdio subprocesses
```

---

## Approval Rules for MCP Tools

Approval rules use glob patterns against the namespaced tool name:

```
{server}:{tool_name}
```

**Pattern examples:**

| Pattern | Matches | Use Case |
|---|---|---|
| `*:*` | All tools from all servers | "Ask for everything" |
| `builtin:filesystem_read:*` | All filesystem reads | "Auto-approve reads" |
| `github:list_*` | All GitHub list operations | "Auto-approve read-only GitHub" |
| `github:create_*` | GitHub create operations | "Ask before creating" |
| `*:delete_*` | Any delete operation | "Always ask before deleting" |

**Evaluation order:**
1. Agent-level rules (highest priority)
2. Global rules
3. Default: `ask` (if no rule matches)

First matching rule wins.

---

## Error Handling

| Error | Handling |
|---|---|
| MCP server fails to start | Emit `mcp.server.disconnected` with error. Mark as disconnected in UI. Don't block app startup. |
| MCP server crashes mid-session | Emit error event. Retry connection with backoff. Inform user via event. |
| Tool call timeout | 60-second default. Emit `tool.error` with timeout reason. |
| Tool returns error | Emit `tool.error`. Feed error back to LLM so it can adjust. |
| Server returns malformed response | Log warning. Return generic error to LLM. |

---

## Popular MCP Servers (Bundled Suggestions)

The Settings UI will show a curated list of popular MCP servers users can add with one click:

| Server | Package | What It Does |
|---|---|---|
| Filesystem | `@modelcontextprotocol/server-filesystem` | Read/write local files |
| GitHub | `@modelcontextprotocol/server-github` | Issues, PRs, repos |
| Git | `mcp-server-git` (Python) | Git operations |
| Postgres | `@modelcontextprotocol/server-postgres` | Database queries |
| Brave Search | `@modelcontextprotocol/server-brave-search` | Web search |
| Fetch | `@modelcontextprotocol/server-fetch` | HTTP requests |
| Memory | `@modelcontextprotocol/server-memory` | Persistent key-value store |

These are just pre-filled configs — the user still clicks "Add" to install them.

---

## Implementation Priority

| Component | Phase | Effort |
|---|---|---|
| MCP client library integration (`mcp` SDK) | Phase 1 | Medium |
| stdio transport support | Phase 1 | Medium |
| Tool discovery + registry | Phase 1 | Low |
| Built-in tools wrapped as MCP-compatible | Phase 1 | Low |
| Tool execution flow with approval | Phase 1 | Already exists — wire to MCP |
| Settings UI for adding/managing servers | Phase 1 | Medium |
| SSE transport support | Phase 2 | Low |
| Streamable HTTP transport | Phase 2 | Low |
| Curated server suggestions | Phase 2 | Low |
| Connection health monitoring | Phase 2 | Low |
