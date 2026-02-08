# AI Studio — System Architecture

> **Version**: 2.0
> **Status**: Draft
> **Depends on**: product-vision.md

---

## Overview

AI Studio is a 3-layer desktop application. Each layer has a single responsibility, communicates through well-defined contracts, and can be developed/tested independently.

```
┌─────────────────────────────────────────────────────────┐
│                    UI Layer (React)                      │
│  Agents │ Sessions │ Runs │ Inspector │ Settings        │
│                                                         │
│  State: Zustand    Styling: Tailwind    Build: Vite     │
└────────────────────────┬────────────────────────────────┘
                         │ Tauri IPC (invoke / listen)
┌────────────────────────┴────────────────────────────────┐
│                  Desktop Layer (Rust/Tauri)              │
│                                                         │
│  Sidecar Manager │ SQLite DB │ Approval Engine │ IPC    │
│                                                         │
│  Source of truth for persistence and security            │
└────────────────────────┬────────────────────────────────┘
                         │ HTTP + WebSocket (localhost)
┌────────────────────────┴────────────────────────────────┐
│                  Agent Layer (Python Sidecar)            │
│                                                         │
│  Provider Router │ MCP Client │ Tool Executor │ Events  │
│                                                         │
│  Runs LLMs, executes tools, emits events                │
└─────────────────────────────────────────────────────────┘
```

---

## Layer 1: UI (React + TypeScript)

### Responsibility
Presentation, navigation, user interaction. No business logic. No direct network calls to the sidecar — everything goes through Tauri IPC.

### Tech Stack
| Component | Technology | Why |
|---|---|---|
| Framework | React 19 | Industry standard, huge ecosystem |
| Language | TypeScript 5.8 | Type safety across the codebase |
| Build | Vite 7 | Fast HMR, simple config |
| State | Zustand | Minimal boilerplate, works great with React |
| Styling | Tailwind CSS 4 | Utility-first, consistent design |
| Icons | Lucide React | Clean, consistent icon set |
| Desktop Shell | Tauri 2.0 WebView | Native window, IPC, system access |

### Module Structure (New — 5 Pillars)

```
apps/ui/src/
├── app/
│   ├── layout/          # AppShell, Header, Sidebar, CommandPalette, ToolApprovalModal
│   ├── pages/
│   │   ├── AgentsPage/     # Agent CRUD, config editor
│   │   ├── SessionsPage/   # Chat interface, tool approval inline
│   │   ├── RunsPage/       # Run list, status, trigger
│   │   ├── InspectorPage/  # Event timeline, tool traces, cost, replay
│   │   └── SettingsPage/   # Providers, MCP servers, preferences
│   └── components/      # Shared UI components
├── state/
│   └── store.ts         # Zustand store (agents, sessions, runs, settings)
├── services/
│   └── tauri.ts         # Tauri IPC wrapper (all backend communication)
├── hooks/               # Custom React hooks
├── commands/            # Keyboard shortcut definitions
└── types/               # UI-specific types (shared types live in packages/shared)
```

### Key Architectural Rules for UI
1. **No direct HTTP calls**. All communication with the sidecar goes through Tauri IPC (`invoke`). This ensures the Tauri layer can intercept, authorize, and log everything.
2. **Events via Tauri `listen`**. WebSocket events from the sidecar are bridged through Tauri to the UI as Tauri events.
3. **State is read from Zustand**, but persistence happens in the Tauri layer (SQLite). The UI sends "save" commands to Tauri; Tauri writes to disk and confirms.
4. **No mock data in production**. Mock fixtures are for development only. Feature flags or environment detection control this.

---

## Layer 2: Desktop (Rust / Tauri)

### Responsibility
Security boundary, persistence, process management. The Tauri layer is the **source of truth** for all stored data and the **gatekeeper** for all tool execution.

### Tech Stack
| Component | Technology | Why |
|---|---|---|
| Framework | Tauri 2.0 | Lightweight, Rust-powered, cross-platform |
| Language | Rust | Performance, safety, no GC |
| Database | SQLite (via rusqlite or sqlx) | Local-first, single-file, fast |
| Async | Tokio | Async runtime for HTTP/WS bridging |
| HTTP Client | Reqwest | Communicates with sidecar |

### Module Structure

```
apps/desktop/src-tauri/src/
├── main.rs              # Entry point
├── lib.rs               # Tauri builder, command registration
├── commands/
│   ├── agents.rs        # Agent CRUD commands
│   ├── sessions.rs      # Session management commands
│   ├── runs.rs          # Run management commands
│   ├── inspector.rs     # Event query commands
│   └── settings.rs      # Settings/preferences commands
├── sidecar.rs           # Sidecar lifecycle (spawn, health, stop)
├── approval.rs          # Tool approval engine (rules + manual)
├── db/
│   ├── mod.rs           # Database initialization, migrations
│   ├── schema.sql       # SQLite schema
│   └── queries.rs       # Typed query functions
├── events.rs            # WebSocket bridge (sidecar events → Tauri events)
└── system.rs            # OS info, paths
```

### Key Responsibilities

#### 2a. Sidecar Lifecycle (Exists — Enhance)
- Spawn Python sidecar on app startup
- Generate auth token per session
- Health check loop
- Kill on app close
- **New**: Auto-restart on crash with backoff

#### 2b. SQLite Persistence (New)
- All data stored in a single SQLite file (`~/.ai-studio/data.db`)
- Schema managed with versioned migrations
- Tables: agents, sessions, messages, events, runs, settings
- Full spec in `data-model.md`

#### 2c. Tool Approval Engine (Exists — Enhance)
- Currently: every `/tools/*` call requires manual approval
- **New**: Rules engine with patterns
  - Auto-approve: `shell:git *`, `filesystem:read *`
  - Always deny: `shell:rm -rf *`, `shell:sudo *`
  - Ask user: everything else
- Rules stored in SQLite, configurable via Settings UI

#### 2d. Event Bridge (New)
- Connect to sidecar's `WS /events` endpoint
- Persist every event to SQLite (the Inspector reads from here)
- Forward events to UI via Tauri `emit`
- Handle reconnection if sidecar restarts

### Key Architectural Rules for Desktop Layer
1. **The Tauri layer owns persistence**. The UI reads/writes through IPC commands. The sidecar doesn't access the database.
2. **The Tauri layer owns security**. Tool approval, token generation, CORS enforcement — all here.
3. **The Tauri layer bridges events**. The UI never connects directly to the sidecar's WebSocket. Tauri subscribes and re-emits.

---

## Layer 3: Agent (Python Sidecar)

### Responsibility
LLM inference, tool execution, event emission. The sidecar is a stateless (session-state only) worker that does what it's told and reports what happened.

### Tech Stack
| Component | Technology | Why |
|---|---|---|
| Framework | FastAPI | Async, auto-docs, type validation |
| Language | Python 3.10+ | ML/AI ecosystem, provider SDKs |
| Validation | Pydantic | Request/response typing |
| Server | Uvicorn | ASGI, production-grade |
| MCP | mcp SDK | Model Context Protocol client |
| WebSocket | FastAPI WebSocket | Event streaming |

### Module Structure (Revised)

```
apps/sidecar/
├── server.py            # FastAPI app, middleware, startup
├── agent/
│   ├── providers/       # LLM providers (Ollama, Anthropic, OpenAI, Google)
│   │   ├── base.py      # Abstract provider interface
│   │   ├── ollama.py
│   │   ├── anthropic.py
│   │   ├── openai.py
│   │   └── google.py
│   ├── chat.py          # ChatService (conversation memory, provider routing)
│   ├── mcp/             # NEW: MCP client integration
│   │   ├── client.py    # MCP client manager (connect to MCP servers)
│   │   ├── discovery.py # Tool discovery from connected MCP servers
│   │   └── bridge.py    # Bridge MCP tools into agent tool calls
│   └── tools/           # Built-in tools (legacy, migrate to MCP over time)
│       ├── shell.py
│       ├── filesystem.py
│       └── browser.py
├── events/              # NEW: Event system
│   ├── bus.py           # Event emitter + WebSocket broadcast
│   ├── types.py         # Typed event definitions
│   └── recorder.py      # Event recording for session replay
├── requirements.txt
└── Dockerfile
```

### Key Responsibilities

#### 3a. Provider Router (Exists — Keep)
- Multi-provider LLM abstraction (Ollama, Anthropic, OpenAI, Google)
- Conversation memory per session
- Model selection and configuration

#### 3b. MCP Client (New)
- Connect to configured MCP servers
- Discover available tools from each server
- Route agent tool calls to the correct MCP server
- Report tool results back to the LLM
- Full spec in `mcp-integration.md`

#### 3c. Tool Executor (Exists — Evolve)
- Built-in tools (shell, filesystem, browser) remain as fallbacks
- Over time, these become MCP servers themselves
- Every tool call emits events (requested, approved, completed, failed)

#### 3d. Event Emitter (New)
- Every significant action emits a typed event
- Events broadcast over `WS /events`
- Event types defined in `event-system.md`
- The sidecar doesn't persist events — it only emits. Tauri persists.

### Key Architectural Rules for Agent Layer
1. **The sidecar is stateless (for persistence)**. It holds conversation memory in-memory for active sessions, but does not write to disk. Tauri handles persistence.
2. **Every action emits an event**. LLM call started, tokens streaming, tool requested, tool completed, error occurred — all events.
3. **Tool calls go through the Tauri proxy**. The sidecar doesn't execute tools directly when running under Tauri. It requests execution, Tauri approves, then the sidecar executes. (When running standalone/Docker, tools execute directly.)
4. **MCP is the primary tool interface**. Built-in tools are a convenience for zero-config usage. MCP servers are the extensible path.

---

## Communication Contracts

### UI ↔ Tauri (IPC)

All communication is via `invoke` (request/response) and `listen` (events).

```typescript
// Request/response pattern
const agents = await invoke<Agent[]>('list_agents');
await invoke('create_agent', { agent: newAgent });

// Event listening pattern
const unlisten = await listen<AgentEvent>('agent_event', (event) => {
  // Update UI state
});
```

**Full command list** defined in `api-contracts.md`.

### Tauri ↔ Sidecar (HTTP + WebSocket)

```
HTTP (REST):
  POST /chat              # Send message, get response
  POST /chat/direct       # Stateless message
  GET  /health            # Health check
  GET  /providers         # List available providers
  GET  /mcp/tools         # List available MCP tools
  POST /mcp/connect       # Connect to MCP server
  POST /tools/*           # Execute tools (proxied through Tauri)

WebSocket:
  WS /events              # Real-time event stream
```

**Full API spec** defined in `api-contracts.md`.

### Event Flow (The Backbone)

```
Sidecar emits event
    → WS /events
        → Tauri receives
            → Tauri persists to SQLite
            → Tauri emits to UI via Tauri event
                → UI updates (Inspector, Session, Run status)
```

This is the central nervous system. Every feature reads/writes events. The Inspector visualizes them. Runs produce them. Sessions produce them. This is what makes the product debuggable and replayable.

---

## Data Flow Examples

### Example 1: User sends a chat message

```
1. User types message in SessionsPage
2. UI calls: invoke('send_message', { sessionId, content })
3. Tauri:
   a. Saves user message to SQLite
   b. Forwards to sidecar: POST /chat { message, provider, model }
4. Sidecar:
   a. Emits event: message.received
   b. Calls LLM provider
   c. Emits event: llm.response.started (streaming)
   d. Emits event: llm.response.chunk (per token)
   e. Emits event: llm.response.completed { content, usage }
5. Tauri:
   a. Receives events via WS
   b. Persists events + assistant message to SQLite
   c. Emits events to UI
6. UI:
   a. Renders streaming response
   b. Updates token/cost counters
```

### Example 2: Agent requests a tool call

```
1. LLM response includes tool_use (e.g., shell: git status)
2. Sidecar emits event: tool.requested { tool, args }
3. Tauri receives event:
   a. Checks approval rules
   b. If auto-approved → emits tool.approved, tells sidecar to proceed
   c. If needs approval → emits tool_approval_requested to UI
4. UI shows approval modal (or inline approval in Sessions)
5. User approves → UI calls invoke('approve_tool_request', { id, approve: true })
6. Tauri:
   a. Emits tool.approved
   b. Signals sidecar to execute
7. Sidecar:
   a. Executes tool
   b. Emits event: tool.completed { result, duration }
8. Tauri persists all events
9. UI updates (Inspector shows tool trace, Session shows result)
```

### Example 3: Opening Inspector for a past session

```
1. User navigates to Inspector, selects a session
2. UI calls: invoke('get_session_events', { sessionId })
3. Tauri queries SQLite for all events with that session_id
4. Returns ordered event list to UI
5. UI renders event timeline:
   - Messages (user + assistant)
   - Tool calls (with input/output/duration)
   - Token counts per turn
   - Total cost
   - Timing waterfall
```

---

## Security Architecture

### Defense in Depth

```
Layer 1 (UI):      Input sanitization, no raw HTML rendering
Layer 2 (Tauri):   Token auth, tool approval, path restrictions, CORS enforcement
Layer 3 (Sidecar): Tool safety modes (sandboxed/restricted/full), blocked commands
```

### Token Flow
1. Tauri generates UUID v4 token on sidecar start
2. Token passed to sidecar via environment variable
3. Every HTTP request from Tauri includes `x-ai-studio-token` header
4. Sidecar validates token in middleware
5. Token rotates on every sidecar restart

### Tool Security
| Mode | Shell | Filesystem | Browser |
|---|---|---|---|
| sandboxed | Whitelist only (ls, git, python...) | Workspace dir only | Not available |
| restricted | Block dangerous (rm -rf, sudo...) | Block sensitive paths (~/.ssh, /etc...) | Available with approval |
| full | No restrictions | No restrictions | Available |

### CORS Policy (Fix from Current)
```python
# Current (insecure when no token):
allow_origins=["*"] if not AI_STUDIO_TOKEN

# Fixed:
allow_origins=["tauri://localhost", "http://localhost:1420"]
# Always restricted. No wildcard. Token is always required when launched from Tauri.
```

---

## File System Layout

### Application Data
```
~/.ai-studio/
├── data.db              # SQLite database (agents, sessions, events, settings)
├── artifacts/           # Files created by tools (screenshots, outputs)
│   └── sessions/
│       └── <session-id>/
├── mcp-servers/         # MCP server configurations
└── logs/
    └── sidecar.log      # Sidecar process logs
```

### Project Repository (Source Code)
```
ai-studio-template/
├── apps/
│   ├── ui/              # React frontend
│   ├── desktop/         # Tauri (Rust) backend
│   └── sidecar/         # Python agent server
├── packages/
│   └── shared/          # Shared TypeScript types
├── docs/
│   └── specs/           # Design specifications (this file)
└── data/                # Sample data for development
```

---

## Development Modes

### Mode 1: Full Desktop (Primary)
```bash
npm run tauri:dev
```
Runs Tauri + React + spawns sidecar. Full tool approval, persistence, events.

### Mode 2: Web UI Only (Frontend Development)
```bash
npm run dev
```
Runs React at localhost:1420. No Tauri, no sidecar. Uses mock data. For UI/styling work only.

### Mode 3: Sidecar Only (API Development)
```bash
npm run sidecar
```
Runs FastAPI at localhost:8765. Swagger docs at /docs. For testing providers, tools, MCP integration.

### Mode 4: Docker (Deployment)
```bash
docker compose up
```
Runs sidecar + Ollama. For headless/server deployment (Runs without UI).

---

## Migration Path from Current Codebase

### What Stays (Keep As-Is)
- Tauri shell + sidecar lifecycle (`sidecar.rs`) — enhance, don't rewrite
- LLM providers (`agent/providers/`) — working and clean
- Tool implementations (`agent/tools/`) — working, add MCP bridge later
- UI framework (React + Vite + Tailwind + Zustand) — keep everything
- Build system (npm workspaces, Cargo) — working
- Tool approval modal — enhance with rules engine

### What Changes
| Current | New | Effort |
|---|---|---|
| 7 page modules | 5 page modules (cut Vision/Audio/Training) | Low — delete files |
| Mock data in Zustand | SQLite via Tauri IPC | Medium — new persistence layer |
| No event system | WebSocket event bus + Tauri bridge | Medium — new feature |
| No Inspector | Full Inspector page | High — flagship feature |
| REST-only tools | MCP client + REST fallback | Medium — additive |
| CORS wildcard | Strict CORS | Low — config change |
| No logging | Structured logging | Low — add library |

### What Gets Removed
- `VisionPage.tsx` — delete
- `AudioPage.tsx` — delete
- `TrainingPage.tsx` — delete
- `canvas/` directory — keep but don't expand (potential future use)
- `channels/` (Telegram) — keep in sidecar, remove from UI
- Mock fixtures — replace incrementally as real persistence comes online

---

## Decisions Made (With Rationale)

| Decision | Choice | Rationale |
|---|---|---|
| Persistence | SQLite in Tauri layer | Local-first, fast, single-file, no server needed. Tauri (Rust) has excellent SQLite support. |
| Event storage | SQLite (same DB) | Events are the core data model. Keeping them in SQLite means the Inspector can query them with SQL. |
| Event transport | WebSocket (sidecar → Tauri) | Real-time, bidirectional, supports streaming. |
| UI ↔ Backend | Tauri IPC only | Security boundary. UI never talks directly to sidecar. |
| Tool system | MCP-native | Interoperable with the growing MCP ecosystem. Future-proof. |
| Built-in tools | Keep as MCP-compatible fallback | Zero-config experience for new users. Migrate to MCP servers over time. |
| State management | Zustand (keep) | Already in place, works well, minimal boilerplate. |
| Module routing | Custom (switch/case in App.tsx) | Simple. No need for react-router when there's only 5 pages. |

---

## Open Architecture Questions

1. **SQLite access from Rust**: Use `rusqlite` (sync, simpler) or `sqlx` (async, type-checked queries)? Leaning `rusqlite` with `spawn_blocking` for simplicity.
2. **Event batching**: Should the Tauri layer batch event writes to SQLite, or write each immediately? Immediate is simpler; batching is better for high-frequency events (streaming tokens).
3. **MCP server lifecycle**: Should Tauri manage MCP server processes (like it manages the sidecar), or should the sidecar manage its own MCP connections? Leaning: sidecar manages MCP connections, Tauri provides the config.
4. **Session branching storage**: Tree structure in SQLite? Or separate sessions with a `parent_session_id` + `branch_point_event_id`? Leaning: parent reference approach — simpler, proven pattern.
