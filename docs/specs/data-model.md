# AI Studio — Data Model Specification

> **Version**: 2.0
> **Status**: Draft
> **Depends on**: product-vision.md, architecture.md, event-system.md

---

## Storage Strategy

**Engine**: SQLite 3 (via `rusqlite` in Tauri/Rust)

**Location**: `~/.ai-studio/data.db`

**Why SQLite**:
- Zero setup (no database server)
- Single file (easy backup: just copy the file)
- Fast for our workload (thousands of events, not millions)
- Full SQL for Inspector queries
- `json_extract()` for querying event payloads
- Cross-platform (macOS, Windows, Linux)

**Migrations**: Versioned SQL scripts. Tauri checks schema version on startup and runs pending migrations. Schema version stored in a `_meta` table.

---

## Schema

### Meta Table

Tracks schema version for migrations.

```sql
CREATE TABLE _meta (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- Initial row
INSERT INTO _meta (key, value) VALUES ('schema_version', '1');
```

### Agents

Agent configurations. The "source code" of AI Studio.

```sql
CREATE TABLE agents (
    id          TEXT PRIMARY KEY,           -- UUID v4
    name        TEXT NOT NULL,
    description TEXT DEFAULT '',

    -- LLM Configuration
    provider    TEXT NOT NULL,              -- "ollama", "anthropic", "openai", "google"
    model       TEXT NOT NULL,              -- "llama3.2", "claude-sonnet-4-5-20250929", etc.
    system_prompt TEXT DEFAULT '',
    temperature REAL DEFAULT 0.7,
    max_tokens  INTEGER DEFAULT 4096,

    -- Tool Configuration
    tools_mode  TEXT DEFAULT 'restricted',  -- "sandboxed", "restricted", "full"
    mcp_servers TEXT DEFAULT '[]',          -- JSON array of MCP server IDs

    -- Approval Rules (JSON array)
    -- e.g., [{"pattern": "shell:git *", "action": "auto_approve"}, ...]
    approval_rules TEXT DEFAULT '[]',

    -- Metadata
    created_at  TEXT NOT NULL,              -- ISO 8601
    updated_at  TEXT NOT NULL,              -- ISO 8601
    is_archived INTEGER DEFAULT 0           -- Soft delete
);

CREATE INDEX idx_agents_archived ON agents(is_archived);
```

### Sessions

A session is a conversation between a user and an agent.

```sql
CREATE TABLE sessions (
    id          TEXT PRIMARY KEY,           -- UUID v4
    agent_id    TEXT NOT NULL REFERENCES agents(id),
    title       TEXT DEFAULT '',            -- Auto-generated or user-set

    -- Branching support
    parent_session_id TEXT REFERENCES sessions(id),   -- NULL for root sessions
    branch_from_seq   INTEGER,                         -- seq number where this session branched

    -- Status
    status      TEXT DEFAULT 'active',     -- "active", "completed", "archived"

    -- Counters (denormalized for fast listing)
    message_count   INTEGER DEFAULT 0,
    event_count     INTEGER DEFAULT 0,
    total_input_tokens  INTEGER DEFAULT 0,
    total_output_tokens INTEGER DEFAULT 0,
    total_cost_usd  REAL DEFAULT 0.0,

    -- Metadata
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    ended_at    TEXT                        -- When session was completed/closed
);

CREATE INDEX idx_sessions_agent ON sessions(agent_id);
CREATE INDEX idx_sessions_status ON sessions(status);
CREATE INDEX idx_sessions_updated ON sessions(updated_at DESC);
CREATE INDEX idx_sessions_parent ON sessions(parent_session_id);
```

### Messages

Chat messages (user + assistant). Stored separately from events for fast retrieval of conversation history (the event table has much more data).

```sql
CREATE TABLE messages (
    id          TEXT PRIMARY KEY,           -- UUID v4
    session_id  TEXT NOT NULL REFERENCES sessions(id),
    seq         INTEGER NOT NULL,           -- Matches the event seq
    role        TEXT NOT NULL,              -- "user", "assistant", "system"
    content     TEXT NOT NULL,

    -- LLM metadata (for assistant messages)
    model       TEXT,
    provider    TEXT,
    input_tokens  INTEGER,
    output_tokens INTEGER,
    cost_usd    REAL,
    duration_ms INTEGER,

    -- Tool use (if this message triggered or resulted from tool calls)
    tool_calls  TEXT,                       -- JSON array of tool_call_ids

    created_at  TEXT NOT NULL
);

CREATE INDEX idx_messages_session ON messages(session_id, seq ASC);
```

### Events

The full event log. The Inspector reads from here. This is the largest table.

```sql
CREATE TABLE events (
    event_id    TEXT PRIMARY KEY,
    type        TEXT NOT NULL,
    ts          TEXT NOT NULL,              -- ISO 8601 with milliseconds
    session_id  TEXT NOT NULL REFERENCES sessions(id),
    source      TEXT NOT NULL,
    seq         INTEGER NOT NULL,
    payload     TEXT NOT NULL DEFAULT '{}', -- JSON

    -- Denormalized for common queries
    cost_usd    REAL                        -- Calculated by Tauri for llm.response.completed events
);

CREATE INDEX idx_events_session_seq ON events(session_id, seq ASC);
CREATE INDEX idx_events_session_type ON events(session_id, type);
CREATE INDEX idx_events_type ON events(type);
```

### Runs

Headless/batch agent executions. A run is like a session but triggered programmatically.

```sql
CREATE TABLE runs (
    id          TEXT PRIMARY KEY,           -- UUID v4
    agent_id    TEXT NOT NULL REFERENCES agents(id),
    session_id  TEXT NOT NULL REFERENCES sessions(id),  -- Every run creates a session for its events

    -- Run configuration
    input       TEXT NOT NULL,              -- The prompt/task given to the agent
    auto_approve_rules TEXT DEFAULT '[]',   -- JSON: approval rules for this run

    -- Status
    status      TEXT DEFAULT 'pending',    -- "pending", "running", "completed", "failed", "cancelled"

    -- Results
    output      TEXT,                       -- Final agent output
    error       TEXT,                       -- Error message if failed

    -- Counters
    total_events    INTEGER DEFAULT 0,
    total_tokens    INTEGER DEFAULT 0,
    total_cost_usd  REAL DEFAULT 0.0,
    duration_ms     INTEGER,

    -- Metadata
    created_at  TEXT NOT NULL,
    started_at  TEXT,
    completed_at TEXT
);

CREATE INDEX idx_runs_agent ON runs(agent_id);
CREATE INDEX idx_runs_status ON runs(status);
CREATE INDEX idx_runs_created ON runs(created_at DESC);
```

### MCP Servers

Configured MCP server connections.

```sql
CREATE TABLE mcp_servers (
    id          TEXT PRIMARY KEY,           -- UUID v4
    name        TEXT NOT NULL UNIQUE,       -- Display name

    -- Connection
    transport   TEXT NOT NULL,              -- "stdio", "sse", "streamable-http"
    command     TEXT,                       -- For stdio: command to run
    args        TEXT DEFAULT '[]',          -- For stdio: JSON array of arguments
    url         TEXT,                       -- For sse/http: server URL
    env         TEXT DEFAULT '{}',          -- JSON: environment variables

    -- Status (not persisted — runtime only; but we track last known state)
    last_status TEXT DEFAULT 'disconnected', -- "connected", "disconnected", "error"
    last_error  TEXT,
    tools_count INTEGER DEFAULT 0,

    -- Metadata
    enabled     INTEGER DEFAULT 1,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);
```

### Approval Rules (Global)

Global tool approval rules (agent-level rules are stored in the agents table).

```sql
CREATE TABLE approval_rules (
    id          TEXT PRIMARY KEY,           -- UUID v4
    name        TEXT NOT NULL,              -- Display name

    -- Matching
    tool_pattern TEXT NOT NULL,             -- Glob: "shell:git *", "filesystem:read *", "mcp:*"

    -- Action
    action      TEXT NOT NULL,              -- "auto_approve", "auto_deny", "ask"

    -- Metadata
    priority    INTEGER DEFAULT 0,          -- Higher priority rules evaluated first
    enabled     INTEGER DEFAULT 1,
    created_at  TEXT NOT NULL
);

CREATE INDEX idx_approval_rules_enabled ON approval_rules(enabled, priority DESC);
```

### Settings

Key-value settings store.

```sql
CREATE TABLE settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL                     -- JSON-encoded value
);
```

**Default settings:**

| Key | Default Value | Description |
|---|---|---|
| `theme` | `"dark"` | UI theme |
| `default_provider` | `"ollama"` | Default LLM provider for new agents |
| `default_model` | `"llama3.2"` | Default model for new agents |
| `tools_mode` | `"restricted"` | Global default tool security mode |
| `model_pricing` | `[...]` | JSON array of ModelPricing entries |
| `keyboard_shortcuts` | `{...}` | Custom keyboard shortcut overrides |
| `data_dir` | `"~/.ai-studio"` | Data directory path |

### Provider Keys

API keys stored separately (encrypted in future versions).

```sql
CREATE TABLE provider_keys (
    provider TEXT PRIMARY KEY,              -- "anthropic", "openai", "google"
    api_key  TEXT NOT NULL,
    base_url TEXT,                          -- Override URL (e.g., custom Ollama host)
    updated_at TEXT NOT NULL
);
```

> **Security note**: In v1, API keys are stored as plaintext in SQLite. In a future version, these should be stored in the OS keychain (macOS Keychain, Windows Credential Manager, Linux Secret Service) via Tauri's keychain plugin. This is a known trade-off for simplicity in Phase 1.

---

## Entity Relationships

```
agents (1) ──────── (*) sessions
                          │
sessions (1) ────── (*) messages
sessions (1) ────── (*) events
sessions (1) ────── (0..1) runs

sessions (1) ────── (*) sessions  (branching: parent_session_id)

agents (*) ─── references ─── (*) mcp_servers  (via agents.mcp_servers JSON)

approval_rules: standalone (global rules)
agents.approval_rules: per-agent rules (JSON column)
```

---

## Session Branching

When a user wants to "replay from here" or "try a different approach", we create a branch:

1. User is in session `S1` at event `seq=15`
2. User clicks "Branch from here"
3. System creates new session `S2` with:
   - `parent_session_id = S1`
   - `branch_from_seq = 15`
4. System copies messages from `S1` where `seq <= 15` into `S2`
5. `S2` continues independently from that point

**Querying a session's full history (including parent):**
```sql
-- Get messages for a branched session (include parent messages up to branch point)
SELECT m.* FROM messages m
WHERE m.session_id = :session_id
UNION ALL
SELECT m.* FROM messages m
JOIN sessions s ON s.id = :session_id
WHERE m.session_id = s.parent_session_id
  AND m.seq <= s.branch_from_seq
  AND s.parent_session_id IS NOT NULL
ORDER BY seq ASC;
```

---

## Data Lifecycle

### Retention
- **Sessions**: Kept indefinitely by default. User can archive or delete.
- **Events**: Kept with their session. Deleted when session is deleted.
- **Agents**: Soft-deleted (is_archived = 1). Hard delete available in Settings.

### Backup
The entire database is one file: `~/.ai-studio/data.db`. Users can back it up by copying this file. A future "Export" feature in Settings will create a `.zip` of the DB + artifacts.

### Size Estimates
| Usage Pattern | Sessions/month | Events/session | DB Size/month |
|---|---|---|---|
| Light | 50 | 100 | ~5 MB |
| Moderate | 200 | 300 | ~50 MB |
| Heavy | 1000 | 500 | ~500 MB |

SQLite handles databases up to 281 TB. We won't hit limits.

### Cleanup
A future "Storage" section in Settings will show DB size and let users:
- Delete sessions older than N days
- Delete archived agents
- Vacuum the database (reclaim space)

---

## Migration Strategy

### How Migrations Work

```
apps/desktop/src-tauri/src/db/
├── migrations/
│   ├── 001_initial.sql
│   ├── 002_add_mcp_servers.sql
│   └── ...
```

On startup:
1. Tauri opens/creates `~/.ai-studio/data.db`
2. Reads `schema_version` from `_meta`
3. Runs any migration scripts with version > current
4. Updates `schema_version`

### From Current Mocks to SQLite

The current app uses hardcoded mock data in `fixtures/mocks.ts`. Migration path:

1. **Phase 1**: Add SQLite layer in Tauri. New IPC commands read/write from DB.
2. **Phase 1**: Keep mock data as seed data — on first launch, if DB is empty, seed with example agent configs.
3. **Phase 1**: Remove mock imports from Zustand store. Store reads from Tauri IPC instead.
4. **Phase 1**: Web-only dev mode (`npm run dev`) uses an in-memory mock service that mimics the Tauri IPC interface.

---

## Tauri IPC Commands (Data Access)

These are the Tauri commands the UI will use to access data. Full signatures in `api-contracts.md`.

### Agents
```
list_agents()              → Agent[]
get_agent(id)              → Agent
create_agent(agent)        → Agent
update_agent(id, updates)  → Agent
delete_agent(id)           → void
```

### Sessions
```
list_sessions(filters?)    → Session[]  (paginated)
get_session(id)            → Session + Messages
create_session(agent_id)   → Session
update_session(id, updates)→ Session
delete_session(id)         → void
branch_session(id, seq)    → Session    (new branched session)
```

### Events (Inspector)
```
get_session_events(session_id, filters?)  → Event[]
get_session_stats(session_id)             → { tokens, cost, duration, tool_calls }
```

### Runs
```
list_runs(filters?)        → Run[]
create_run(agent_id, input, rules?)  → Run
cancel_run(id)             → void
get_run(id)                → Run + Session
```

### Settings
```
get_setting(key)           → value
set_setting(key, value)    → void
get_all_settings()         → Record<string, value>
```

### MCP Servers
```
list_mcp_servers()         → McpServer[]
add_mcp_server(config)     → McpServer
update_mcp_server(id, updates) → McpServer
remove_mcp_server(id)      → void
test_mcp_server(id)        → { status, tools }
```

### Provider Keys
```
get_provider_keys()        → { provider: string, has_key: boolean }[]
set_provider_key(provider, key, base_url?) → void
delete_provider_key(provider)              → void
```
