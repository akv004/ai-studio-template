# AI Studio — Sprint Board

> This is the project's single source of truth for progress.
> New sessions: read this top-to-bottom, then you're caught up.

## Current Phase: 1 (Core Working Product)

**Demo goal**: Create agent → chat with it → see tools execute → inspect session with events & cost.

| Sub-task | Spec | Status |
|----------|------|--------|
| 1A: SQLite + CRUD | `data-model.md` | DONE |
| 1B: Event system | `event-system.md` | PARTIAL — recording works, no WebSocket |
| 1C: Agent CRUD UI | `ui-design.md` | DONE |
| 1D: Chat sessions | `api-contracts.md` | Backend DONE, **UI needs testing** |
| 1E: Basic Inspector | `agent-inspector.md` | Skeleton only |
| 1F: MCP tools | `mcp-integration.md` | Not started |

---

## In Progress (Current Sprint)

> What we're actively working on right now.

- [ ] **Verify chat flow end-to-end** — isTauri() and createSession bugs were fixed (a681b59), needs retest with `pnpm tauri dev`
  - Test: Settings → save Google API key → test connection (should not 401 now)
  - Test: Agents → create agent with Google provider → Sessions → create session → send message → see response
  - If errors appear, they should now be visible in the UI (error state was added)

---

## Backlog (Prioritized — Next Up)

> Ordered by priority. Work top-down. Each item notes which spec to read.

### P0 — Must have for Phase 1 demo
1. **Chat flow fixes** — whatever breaks during testing above
2. **Inspector timeline** (`agent-inspector.md`) — show events for a session, click to see detail
3. **Inspector stats bar** (`agent-inspector.md`) — token counts, cost, duration, models used

### P1 — Important for Phase 1 completeness
4. **MCP tool discovery + execution** (`mcp-integration.md`) — register tools, execute via sidecar, approval gate
5. **MCP settings UI** (`mcp-integration.md`) — tool list, enable/disable, approval rules
6. **Event WebSocket bridge** (`event-system.md`) — live event streaming to Inspector

### P2 — Nice to have before Phase 2
7. Cost calculation in events — populate `cost_usd` field based on model pricing
8. Runs execution — create/execute endpoints + UI
9. Error handling polish across IPC commands

---

## Done (Compressed)

**Phase 0**: Restructured to 5 pillars, removed old modules, wrote 11 specs.

**Phase 1 (so far)**:
- SQLite DB (WAL mode, full schema), all CRUD commands, send_message chat loop, event recording
- Sidecar: 5 providers (Anthropic, Google, Azure, Local, Ollama), chat/test endpoints, tool stubs
- UI: Agents page CRUD, Settings page (provider keys + test), Sessions page (chat UI), sidebar, Zustand→IPC
- Shared types updated, old types removed

---

## Decisions Log

> Design decisions and pivots. New sessions: read this to understand WHY things are the way they are.

| Date | Decision | Why |
|------|----------|-----|
| 2026-02-08 | Settings stored as `provider.{id}.{field}` in generic settings table | Simpler than separate provider_keys table for each field; one table for all config |
| 2026-02-08 | Sidecar auth via random token in env var | Tauri spawns sidecar with token; prevents localhost port scanning attacks |
| 2026-02-08 | UI routes all sidecar calls through Tauri IPC | Security boundary — Tauri adds auth token, UI never has direct sidecar access |
| 2026-02-08 | Chat sessions store messages in SQLite, not sidecar | Sidecar is stateless per-request; Rust owns persistence and history replay |
| 2026-02-08 | Dogfooding validation: STATUS.md/CLAUDE.md workflow mirrors what AI Studio solves | We hit the exact pain points (lost session context, no visibility, no replay) that the product addresses. Proves the market need. Priority: get to Inspector ASAP — it's the core value prop. |

---

## Gotchas

- `isTauri()`: Must check `__TAURI_INTERNALS__` (v2), not `__TAURI__` (v1) — fixed in a681b59
- **Tauri v2 IPC args are camelCase**: Rust `agent_id` → JS `{ agentId }`. NOT snake_case! This caused "missing required key" errors.
- Store errors: Every store action that calls `invoke` should set `error` state on failure, not swallow silently
- Sidecar provider config: Values stored as JSON strings with quotes — strip with `trim_matches('"')` in Rust

---

## Last Session Notes

**Date**: 2026-02-08
**What happened**:
- Committed Phase 1 foundation (d3684bf) — was all uncommitted from previous session
- Fixed isTauri() bug (was causing 401 on test connection)
- Fixed createSession silent failure (errors now visible)
- Set up project management workflow (CLAUDE.md + STATUS.md)

**Next session should**:
1. Run `pnpm tauri dev` and test the full chat flow (Google provider)
2. If chat works → start Inspector timeline (read `agent-inspector.md` first)
3. If chat breaks → fix it, that's the critical path
