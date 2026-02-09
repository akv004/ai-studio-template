# AI Studio — Session Status

> This file is updated by Claude after every meaningful change.
> New sessions: read this first to know where things stand.

## Last Updated
2026-02-08 19:15 — fixed isTauri() + createSession error handling (a681b59)

## What's Done

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
- [x] Removed: Audio, Vision, Training, Projects, Canvas
- [x] Agents page — full CRUD UI with provider/model selection
- [x] Settings page — provider API key forms with save + test connection
- [x] Zustand store wired to real Tauri IPC commands (no more mocks)
- [x] Sidebar updated to 5 pillars
- [x] Skeleton pages: Inspector, Sessions

### Shared Types
- [x] Updated agent, run types for 5-pillar model
- [x] Removed project.ts, training.ts

## What's NOT Done Yet

### High Priority (Phase 1 remaining)
- [ ] **Chat UI** — no message input or display anywhere yet (send_message backend works, no frontend)
- [ ] **Inspector timeline** — page exists but no event visualization
- [ ] **Sessions page** — skeleton only, needs session list + chat view
- [ ] **Test the full flow** — create agent with API key → start session → send message → see response
- [ ] **Verify provider test connection** — isTauri() bug fixed (a681b59), needs retest

### Medium Priority
- [ ] Runs execution — only list_runs exists, no create/execute
- [ ] Session branching UI — DB supports it, no UI
- [ ] Cost calculation in events — field exists, not populated
- [ ] Error handling polish across all IPC commands

### Lower Priority (Phase 2+)
- [ ] MCP tool integration
- [ ] Agent Inspector replay/branching
- [ ] Export/import sessions
- [ ] Hybrid intelligence (smart model routing)

## Bugs Fixed This Session
- **isTauri() wrong check** (a681b59): Was checking `window.__TAURI__` (v1) instead of `__TAURI_INTERNALS__` (v2). Caused test connection to bypass Tauri IPC → direct HTTP to sidecar without auth token → 401.
- **createSession silent failure** (a681b59): Store didn't set error state on failure. Now errors are visible in UI.

## Last Session Notes
- Fixed two bugs: test connection 401 and create session doing nothing
- Next: retest both flows with `npm run tauri:dev`
- Previous session was working on: adding agents with Google API key and testing provider integration

## Build / Run
```bash
# UI dev
cd apps/ui && pnpm dev

# Tauri desktop (runs Rust + UI)
cd apps/desktop && pnpm tauri dev

# Sidecar standalone (for testing)
cd apps/sidecar && python -m uvicorn server:app --port 8420
```
