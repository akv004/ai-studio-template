# AI Studio — Session Status

> Updated by Claude after every meaningful change.
> New sessions: read this first, then check CLAUDE.md for workflow rules.

## Current Phase: 1 (Core Working Product)

**Success criteria**: Create agent → chat with it → see tools execute → inspect session with events & cost.

### Phase 1 Progress

| Sub-task | Spec to read | Status |
|----------|-------------|--------|
| **1A** SQLite schema + CRUD | `data-model.md`, `api-contracts.md` | DONE |
| **1B** Event system (types, recording) | `event-system.md` | PARTIAL — recording works, no WebSocket bridge yet |
| **1C** Agent CRUD UI | `ui-design.md`, `api-contracts.md` | DONE |
| **1D** Chat sessions + persistence | `api-contracts.md`, `data-model.md` | Backend DONE, **UI not wired** |
| **1E** Basic Inspector | `agent-inspector.md` | Skeleton page only, **no timeline/detail/stats** |
| **1F** MCP tool integration | `mcp-integration.md` | Not started |

## Last Updated
2026-02-08 19:30 — updated CLAUDE.md with spec-driven workflow (6e3cde9)

## What's Done (Detail)

### Backend (Rust/Tauri)
- [x] SQLite DB with WAL mode (`db.rs`) — agents, sessions, messages, events, runs, settings, provider_keys
- [x] Agent CRUD commands (list, get, create, update, delete/archive)
- [x] Session CRUD commands (list, create, get messages, delete)
- [x] `send_message` — full chat loop: persist user msg → call sidecar → persist response → record events
- [x] Event recording with typed events (message.user, llm.request.started, llm.response.completed, message.assistant)
- [x] Settings key-value store (get_all, set)
- [x] Provider key management (list, set, delete)
- [x] Sidecar lifecycle management (spawn, health check, auth token)

### Sidecar (Python/FastAPI)
- [x] Multi-provider support: Anthropic, Google, Azure OpenAI, Local/OpenAI-compatible, Ollama
- [x] `/chat` endpoint (persistent conversations)
- [x] `/chat/direct` endpoint (one-off queries)
- [x] `/providers/test` endpoint (test API key connectivity)
- [x] Tool system stubs (shell, filesystem, browser)

### UI (React)
- [x] Restructured from 7 modules → 5 pillars (Agents, Sessions, Runs, Inspector, Settings)
- [x] Agents page — full CRUD UI with provider/model selection
- [x] Settings page — provider API key forms with save + test connection
- [x] Zustand store wired to real Tauri IPC (no more mocks)
- [x] Sessions page with chat UI (messages, input, create/delete)
- [x] Skeleton Inspector page

## What's Next (Priority Order)

### Immediate — finish Phase 1D (Chat sessions UI)
1. **Retest provider test connection** — isTauri() bug was fixed (a681b59), verify it works
2. **Retest create session + send message** — createSession error handling added, verify end-to-end flow
3. Fix any remaining issues in the chat flow

### Then — Phase 1E (Basic Inspector) → read `agent-inspector.md`
4. Inspector timeline — show events for a selected session
5. Inspector detail panel — click event to see payload
6. Inspector stats bar — token counts, cost, duration

### Then — Phase 1F (MCP) → read `mcp-integration.md`
7. MCP tool discovery + registration
8. Tool execution through sidecar with approval gate
9. MCP settings UI

### Medium Priority
- Event system WebSocket bridge (live event streaming to Inspector)
- Runs execution — create/execute endpoints
- Cost calculation in events
- Error handling polish

## Bugs Fixed
- **isTauri() wrong check** (a681b59): `window.__TAURI__` (v1) → `__TAURI_INTERNALS__` (v2). Caused 401 on test connection.
- **createSession silent failure** (a681b59): Store didn't set error state. Now visible in UI.

## Last Session Notes
- Fixed two bugs: test connection 401 and create session silent failure
- **Next session should**: retest both flows with `pnpm tauri dev`, then move to Inspector if chat works
- Google API key was being tested — verify it connects after the isTauri fix
