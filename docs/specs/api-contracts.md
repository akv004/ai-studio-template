# AI Studio — API Contracts

> **Version**: 3.0
> **Status**: Draft
> **Depends on**: architecture.md, event-system.md, data-model.md, mcp-integration.md, node-editor.md

---

## Overview

AI Studio has two communication boundaries:

1. **UI ↔ Tauri** — Tauri IPC (`invoke` + `listen`)
2. **Tauri ↔ Sidecar** — HTTP REST + WebSocket (localhost)

The UI **never** talks directly to the sidecar. All requests go through Tauri.

---

## Part 1: Tauri IPC Commands (UI ↔ Tauri)

These are the `invoke()` commands the React UI calls via `@tauri-apps/api/core`.

### Agents

#### `list_agents`
```typescript
invoke<Agent[]>('list_agents')
```
Returns all non-archived agents, ordered by `updated_at DESC`.

**Response:**
```typescript
interface Agent {
  id: string;
  name: string;
  description: string;
  provider: string;
  model: string;
  system_prompt: string;
  temperature: number;
  max_tokens: number;
  tools_mode: string;
  mcp_servers: string[];
  approval_rules: ApprovalRule[];
  created_at: string;
  updated_at: string;
}
```

#### `get_agent`
```typescript
invoke<Agent>('get_agent', { id: string })
```

#### `create_agent`
```typescript
invoke<Agent>('create_agent', { agent: CreateAgentRequest })
```
```typescript
interface CreateAgentRequest {
  name: string;
  description?: string;
  provider: string;
  model: string;
  system_prompt?: string;
  temperature?: number;       // default: 0.7
  max_tokens?: number;        // default: 4096
  tools_mode?: string;        // default: "restricted"
  mcp_servers?: string[];
  approval_rules?: ApprovalRule[];
}
```

#### `update_agent`
```typescript
invoke<Agent>('update_agent', { id: string, updates: Partial<CreateAgentRequest> })
```

#### `delete_agent`
```typescript
invoke('delete_agent', { id: string })
```
Soft delete (sets `is_archived = 1`).

---

### Sessions

#### `list_sessions`
```typescript
invoke<Session[]>('list_sessions', { filters?: SessionFilters })
```
```typescript
interface SessionFilters {
  agent_id?: string;
  status?: 'active' | 'completed' | 'archived';
  limit?: number;            // default: 50
  offset?: number;           // default: 0
}

interface Session {
  id: string;
  agent_id: string;
  agent_name: string;
  title: string;
  status: string;
  parent_session_id?: string;
  branch_from_seq?: number;
  message_count: number;
  total_input_tokens: number;
  total_output_tokens: number;
  total_cost_usd: number;
  created_at: string;
  updated_at: string;
  ended_at?: string;
}
```

#### `get_session`
```typescript
invoke<SessionDetail>('get_session', { id: string })
```
```typescript
interface SessionDetail extends Session {
  messages: Message[];
  agent: Agent;
}

interface Message {
  id: string;
  session_id: string;
  seq: number;
  role: 'user' | 'assistant' | 'system';
  content: string;
  model?: string;
  provider?: string;
  input_tokens?: number;
  output_tokens?: number;
  cost_usd?: number;
  duration_ms?: number;
  tool_calls?: string[];
  created_at: string;
}
```

#### `create_session`
```typescript
invoke<Session>('create_session', { agent_id: string })
```

#### `send_message`
```typescript
invoke<void>('send_message', { session_id: string, content: string })
```
Fire-and-forget. Response comes back as events via `listen('agent_event')`.

Flow:
1. Tauri saves user message to SQLite
2. Tauri emits `message.user` event
3. Tauri forwards to sidecar `POST /chat`
4. Sidecar streams response via `WS /events`
5. UI receives events via `listen('agent_event')`

#### `branch_session`
```typescript
invoke<Session>('branch_session', { session_id: string, from_seq: number })
```

#### `delete_session`
```typescript
invoke('delete_session', { id: string })
```

---

### Inspector (Events)

#### `get_session_events`
```typescript
invoke<Event[]>('get_session_events', {
  session_id: string,
  filters?: EventFilters
})
```
```typescript
interface EventFilters {
  types?: string[];          // e.g., ["tool.*", "llm.*"]
  from_seq?: number;
  to_seq?: number;
  limit?: number;            // default: 500
  offset?: number;
}

interface Event {
  event_id: string;
  type: string;
  ts: string;
  session_id: string;
  source: string;
  seq: number;
  payload: Record<string, unknown>;
  cost_usd?: number;
}
```

#### `get_session_stats`
```typescript
invoke<SessionStats>('get_session_stats', { session_id: string })
```
```typescript
interface SessionStats {
  total_input_tokens: number;
  total_output_tokens: number;
  total_cost_usd: number;
  total_duration_ms: number;
  llm_call_count: number;
  tool_call_count: number;
  tool_approved_count: number;
  tool_denied_count: number;
  model: string;
  provider: string;
}
```

#### `export_session`
```typescript
invoke<string>('export_session', { session_id: string, format: 'json' | 'markdown' })
```
Returns exported content as string. UI saves via Tauri file dialog.

---

### Runs

#### `list_runs`
```typescript
invoke<Run[]>('list_runs', { filters?: RunFilters })
```
```typescript
interface RunFilters {
  agent_id?: string;
  status?: string;
  limit?: number;
  offset?: number;
}

interface Run {
  id: string;
  agent_id: string;
  agent_name: string;
  session_id: string;
  input: string;
  status: 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';
  output?: string;
  error?: string;
  total_events: number;
  total_tokens: number;
  total_cost_usd: number;
  duration_ms?: number;
  created_at: string;
  started_at?: string;
  completed_at?: string;
}
```

#### `create_run`
```typescript
invoke<Run>('create_run', {
  agent_id: string,
  input: string,
  auto_approve_rules?: ApprovalRule[]
})
```

#### `cancel_run`
```typescript
invoke('cancel_run', { id: string })
```

---

### Tool Approval

#### `approve_tool_request`
```typescript
invoke('approve_tool_request', { id: string, approve: boolean })
```
Already exists. No changes needed.

---

### Settings

#### `get_all_settings`
```typescript
invoke<Record<string, unknown>>('get_all_settings')
```

#### `get_setting`
```typescript
invoke<unknown>('get_setting', { key: string })
```

#### `set_setting`
```typescript
invoke('set_setting', { key: string, value: unknown })
```

---

### MCP Servers

#### `list_mcp_servers`
```typescript
invoke<McpServer[]>('list_mcp_servers')
```
```typescript
interface McpServer {
  id: string;
  name: string;
  transport: 'stdio' | 'sse' | 'streamable-http';
  command?: string;
  args?: string[];
  url?: string;
  env?: Record<string, string>;
  enabled: boolean;
  last_status: 'connected' | 'disconnected' | 'error';
  last_error?: string;
  tools_count: number;
  created_at: string;
  updated_at: string;
}
```

#### `add_mcp_server`
```typescript
invoke<McpServer>('add_mcp_server', { config: CreateMcpServerRequest })
```
```typescript
interface CreateMcpServerRequest {
  name: string;
  transport: 'stdio' | 'sse' | 'streamable-http';
  command?: string;
  args?: string[];
  url?: string;
  env?: Record<string, string>;
}
```

#### `update_mcp_server`
```typescript
invoke<McpServer>('update_mcp_server', { id: string, updates: Partial<CreateMcpServerRequest> })
```

#### `remove_mcp_server`
```typescript
invoke('remove_mcp_server', { id: string })
```

#### `test_mcp_server`
```typescript
invoke<McpTestResult>('test_mcp_server', { id: string })
```
```typescript
interface McpTestResult {
  success: boolean;
  tools: string[];
  error?: string;
}
```

---

### Provider Keys

#### `list_provider_keys`
```typescript
invoke<ProviderKeyInfo[]>('list_provider_keys')
```
```typescript
interface ProviderKeyInfo {
  provider: string;
  has_key: boolean;          // Never expose the actual key to UI
  base_url?: string;
}
```

#### `set_provider_key`
```typescript
invoke('set_provider_key', { provider: string, api_key: string, base_url?: string })
```

#### `delete_provider_key`
```typescript
invoke('delete_provider_key', { provider: string })
```

---

### Sidecar Management (Existing)

```typescript
invoke<SidecarStatus>('sidecar_start')
invoke('sidecar_stop')
invoke<SidecarStatus>('sidecar_status')
```

---

## Part 2: Tauri Events (Tauri → UI)

Events pushed from Tauri to the UI via `listen()`.

### `agent_event`
Main event channel. Every event from the sidecar is forwarded here.

```typescript
listen<Event>('agent_event', (e) => {
  const event = e.payload;
  // event.type: "message.user", "llm.response.chunk", "tool.completed", etc.
})
```

### `tool_approval_requested`
Already exists. When a tool call needs user approval.

```typescript
listen<ToolApprovalRequest>('tool_approval_requested', (e) => {
  // Show approval modal
})
```
```typescript
interface ToolApprovalRequest {
  id: string;
  tool_name: string;
  tool_input: Record<string, unknown>;
  mcp_server?: string;
}
```

### `sidecar_status_changed`
When sidecar health changes.

```typescript
listen<{ running: boolean }>('sidecar_status_changed', (e) => {
  // Update status indicator
})
```

---

## Part 3: Sidecar REST API (Tauri ↔ Sidecar)

HTTP endpoints on the Python sidecar (localhost:8765). Only Tauri calls these.

### Health

#### `GET /health`
```json
{ "status": "healthy", "version": "0.2.0" }
```
No auth required (startup probe).

#### `GET /status`
```json
{
  "status": "healthy",
  "providers": { "ollama": true, "anthropic": true },
  "mcp_servers": { "github": "connected", "filesystem": "connected" },
  "active_sessions": 2
}
```

### Chat

#### `POST /chat`
```json
{
  "session_id": "sess_abc123",
  "message": "What's the git status?",
  "agent_config": {
    "provider": "anthropic",
    "model": "claude-sonnet-4-5-20250929",
    "system_prompt": "You are a helpful coding assistant.",
    "temperature": 0.7,
    "max_tokens": 4096,
    "tools": [
      { "name": "builtin__shell", "description": "...", "input_schema": {} },
      { "name": "github__create_issue", "description": "...", "input_schema": {} }
    ]
  }
}
```

Response is the final result. Intermediate events stream via WebSocket.

```json
{
  "session_id": "sess_abc123",
  "content": "Here's the git status...",
  "model": "claude-sonnet-4-5-20250929",
  "provider": "anthropic",
  "usage": { "input_tokens": 1247, "output_tokens": 89 }
}
```

#### `POST /chat/direct`
Stateless single-shot message. Kept for backward compatibility.

### Providers

#### `GET /providers`
```json
{
  "providers": [
    { "name": "ollama", "available": true, "models": ["llama3.2", "codellama"] },
    { "name": "anthropic", "available": true, "models": ["claude-sonnet-4-5-20250929"] },
    { "name": "openai", "available": false, "error": "No API key" }
  ]
}
```

### MCP

#### `POST /mcp/connect`
```json
{
  "name": "github",
  "transport": "stdio",
  "command": "npx",
  "args": ["@anthropic/mcp-server-github"],
  "env": { "GITHUB_TOKEN": "ghp_xxxx" }
}
```
Response: `{ "status": "connected", "tools": ["create_issue", "list_prs"] }`

#### `POST /mcp/disconnect`
```json
{ "name": "github" }
```

#### `GET /mcp/tools`
```json
{
  "tools": [
    { "server": "github", "name": "create_issue", "description": "..." },
    { "server": "builtin", "name": "shell", "description": "..." }
  ]
}
```

### Tools (Legacy)

```
POST /tools/shell
POST /tools/filesystem
POST /tools/browser
POST /tools/browser/start
POST /tools/browser/stop
```
Continue to work as-is. Tool calls will migrate to MCP over time.

---

## Part 4: Sidecar WebSocket (Sidecar → Tauri)

### `WS /events`

**Connection:** `ws://127.0.0.1:8765/events`

**Auth (first message):**
```json
{ "type": "auth", "token": "AI_STUDIO_TOKEN_VALUE" }
```

**Server confirms:**
```json
{ "type": "auth_ok" }
```

**Event stream (one JSON per WebSocket message):**
```json
{"event_id":"...","type":"llm.response.chunk","ts":"...","session_id":"...","source":"sidecar.chat","seq":12,"payload":{"delta":"Hello"}}
```

**Keepalive:** Standard WebSocket ping every 30s.

**Reconnection:** Tauri reconnects with backoff: 1s, 2s, 4s, 8s, max 30s.

---

## Shared Types

### ApprovalRule
```typescript
interface ApprovalRule {
  id?: string;
  name?: string;
  tool_pattern: string;      // Glob: "builtin:shell:git *", "github:*"
  action: 'auto_approve' | 'auto_deny' | 'ask';
  priority?: number;
  enabled?: boolean;
}
```

### Workflows (Phase 3A)

#### `list_workflows`
```typescript
invoke<WorkflowSummary[]>('list_workflows')
```
Returns all non-archived workflows, ordered by `updated_at DESC`.

**Response** (`WorkflowSummary`):
```typescript
{
  id: string;
  name: string;
  description: string;
  agentId: string | null;
  nodeCount: number;         // Computed from graph_json
  isArchived: boolean;
  createdAt: string;
  updatedAt: string;
}
```

#### `get_workflow`
```typescript
invoke<Workflow>('get_workflow', { id: string })
```
Returns a single workflow with full graph data.

**Response** (`Workflow`):
```typescript
{
  id: string;
  name: string;
  description: string;
  graphJson: string;          // React Flow serialized graph
  variablesJson: string;      // Workflow-level variables
  agentId: string | null;
  isArchived: boolean;
  createdAt: string;
  updatedAt: string;
}
```

#### `create_workflow`
```typescript
invoke<Workflow>('create_workflow', { request: CreateWorkflowRequest })
```

**Params** (`CreateWorkflowRequest`):
```typescript
{
  name: string;
  description?: string;
  graphJson?: string;
  variablesJson?: string;
  agentId?: string;
}
```

#### `update_workflow`
```typescript
invoke<Workflow>('update_workflow', { id: string, request: UpdateWorkflowRequest })
```
Partial update — only provided fields are modified. `updated_at` is always refreshed.

**Params** (`UpdateWorkflowRequest`):
```typescript
{
  name?: string;
  description?: string;
  graphJson?: string;
  variablesJson?: string;
  agentId?: string;
}
```

#### `delete_workflow`
```typescript
invoke('delete_workflow', { id: string })
```
Soft delete — sets `is_archived = 1`. Archived workflows are excluded from `list_workflows`.

#### `duplicate_workflow`
```typescript
invoke<Workflow>('duplicate_workflow', { id: string })
```
Creates a copy with name `"Copy of {original}"`. New UUID, new timestamps, copies `graph_json` and `variables_json`.

---

### Error Format

Tauri IPC errors are strings:
```typescript
try {
  await invoke('get_agent', { id: 'xxx' });
} catch (error) {
  // error: "Agent not found: xxx"
}
```

Sidecar HTTP errors:
```json
{ "detail": "Error description" }
```
With HTTP status codes (400, 401, 404, 500).
