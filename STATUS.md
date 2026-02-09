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
| 1D: Chat sessions | `api-contracts.md` | DONE — full flow verified (Google/Gemini) |
| 1E: Basic Inspector | `agent-inspector.md` | DONE — timeline, detail, stats, filters, export |
| 1F: MCP tools | `mcp-integration.md` | Not started |

---

## In Progress (Current Sprint)

> What we're actively working on right now.

- [x] ~~Verify chat flow end-to-end~~ — DONE, working with Google/Gemini
- [x] ~~Build Inspector timeline~~ — DONE (3285434), color-coded, type-specific details, filters, stats
- [ ] **MCP tool discovery + execution** (`mcp-integration.md`) — register tools, execute via sidecar

---

## Backlog (Prioritized — Next Up)

> Ordered by priority. Work top-down. Each item notes which spec to read.

### P0 — Must have for Phase 1 demo
1. ~~Chat flow fixes~~ — DONE
2. ~~Inspector timeline~~ — DONE (3285434)
3. ~~Inspector stats bar~~ — DONE (included in #2)

### P1 — Important for Phase 1 completeness
4. **MCP tool discovery + execution** (`mcp-integration.md`) — register tools, execute via sidecar, approval gate
5. **MCP settings UI** (`mcp-integration.md`) — tool list, enable/disable, approval rules
6. **Event WebSocket bridge** (`event-system.md`) — live event streaming to Inspector

### P2 — Nice to have before Phase 2
7. Cost calculation in events — populate `cost_usd` field based on model pricing
8. Runs execution — create/execute endpoints + UI
9. Error handling polish across IPC commands
10. **Dev: DB wipe command** — Tauri command + Settings UI button to reset SQLite (drop all data, re-init schema). Useful during dev.

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

**Date**: 2026-02-08 (session 3)
**What happened**:
- Chat flow verified working end-to-end (Google/Gemini)
- Built full Inspector (3285434): timeline, type-specific detail panels, filter chips, stats bar, keyboard nav, JSON export, "Inspect" button from Sessions
- All P0 items DONE. Moving to P1.

**Previous sessions**:
- Session 1: Phase 1 foundation (d3684bf), isTauri fix (a681b59), camelCase fix (8dbe4a8)
- Session 2: PM workflow (CLAUDE.md + STATUS.md), dogfooding insight (d7808e7)

**Next session should**:
1. Test Inspector with real session events via `pnpm tauri dev`
2. Start MCP tool discovery + execution (P1 #4, read `mcp-integration.md`)
3. If Inspector needs polish → fix it first
