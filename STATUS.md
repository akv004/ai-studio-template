# AI Studio — Sprint Board

> This is the project's single source of truth for progress.
> New sessions: read this top-to-bottom, then you're caught up.

## Current Phase: 1 (Core Working Product)

**Demo goal**: Create agent → chat with it → see tools execute → inspect session with events & cost.

| Sub-task | Spec | Status |
|----------|------|--------|
| 1A: SQLite + CRUD | `data-model.md` | DONE |
| 1B: Event system | `event-system.md` | DONE — WS bridge, live streaming, cost calc |
| 1C: Agent CRUD UI | `ui-design.md` | DONE |
| 1D: Chat sessions | `api-contracts.md` | DONE — full flow verified (Google/Gemini) |
| 1E: Basic Inspector | `agent-inspector.md` | DONE — timeline, detail, stats, filters, export |
| 1F: MCP tools | `mcp-integration.md` | DONE — tool calling, MCP client, Settings UI |

---

## In Progress (Current Sprint)

> What we're actively working on right now.

- [x] ~~Verify chat flow end-to-end~~ — DONE, working with Google/Gemini
- [x] ~~Build Inspector timeline~~ — DONE (3285434), color-coded, type-specific details, filters, stats
- [x] ~~MCP tool discovery + execution~~ — DONE: tool registry, multi-turn tool loop, MCP client, Settings UI
- [x] ~~Event WebSocket bridge + cost calc~~ — DONE: sidecar EventBus/WS, Rust WS client, UI listener, model pricing

---

## Backlog (Prioritized — Next Up)

> Ordered by priority. Work top-down. Each item notes which spec to read.

### P0 — Must have for Phase 1 demo
1. ~~Chat flow fixes~~ — DONE
2. ~~Inspector timeline~~ — DONE (3285434)
3. ~~Inspector stats bar~~ — DONE (included in #2)

### P1 — Important for Phase 1 completeness
4. ~~MCP tool discovery + execution~~ — DONE
5. ~~MCP settings UI~~ — DONE (Settings → MCP Servers tab)
6. ~~Event WebSocket bridge~~ — DONE (sidecar WS, Rust bridge, UI listener, cost calc)

### P2 — Nice to have before Phase 2
7. ~~Cost calculation~~ — DONE (in event bridge, pricing table for Claude/GPT/Gemini/local)
8. Runs execution — create/execute endpoints + UI
9. Error handling polish across IPC commands
10. **Dev: DB wipe command** — Tauri command + Settings UI button to reset SQLite (drop all data, re-init schema). Useful during dev.

---

## Done (Compressed)

**Phase 0**: Restructured to 5 pillars, removed old modules, wrote 11 specs.

**Phase 1 (so far)**:
- SQLite DB (WAL mode, full schema v2), all CRUD commands, send_message chat loop, event recording
- Sidecar: 5 providers (Anthropic, Google, Azure, Local, Ollama), chat/test endpoints, tool stubs
- MCP: Tool registry, built-in tools (shell/fs), MCP stdio client, multi-turn tool calling loop
- Provider tool calling: Anthropic + Google with full tool_use support, others interface-ready
- DB: mcp_servers table, CRUD commands, shared TypeScript types
- Event bridge: Sidecar EventBus + WS /events → Tauri WS client → SQLite + UI emit
- Cost calculation: model pricing table (Claude/GPT/Gemini/local), auto-calc on llm.response.completed
- UI: Agents page CRUD, Settings page (provider keys + MCP servers), Sessions page (chat UI), sidebar, Zustand→IPC
- Inspector: timeline, detail panels, stats, filters, export, keyboard nav, live event push
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
| 2026-02-08 | **NODE EDITOR = core product direction** (Phase 3 build, but shapes all architecture) | "Unreal Blueprints for AI agents." Visual node graph where: LLM models, MCP tools, routers, approval gates, data transforms are all pluggable nodes. Users build AI pipelines by connecting nodes — no code. Live execution shows data flowing through nodes with cost/tokens per node. Inspiration: Maya Hypershade, UE Blueprints, Houdini, ComfyUI. This is the 10k-star feature. Current timeline Inspector evolves INTO this. Everything we build (MCP tools, hybrid routing, events) must be node-compatible. |

---

## Gotchas

- `isTauri()`: Must check `__TAURI_INTERNALS__` (v2), not `__TAURI__` (v1) — fixed in a681b59
- **Tauri v2 IPC args are camelCase**: Rust `agent_id` → JS `{ agentId }`. NOT snake_case! This caused "missing required key" errors.
- Store errors: Every store action that calls `invoke` should set `error` state on failure, not swallow silently
- Sidecar provider config: Values stored as JSON strings with quotes — strip with `trim_matches('"')` in Rust

---

## Last Session Notes

**Date**: 2026-02-08 (session 4 continued)
**What happened**:
- Full MCP implementation: tool registry, built-in tools, MCP stdio client, multi-turn tool loop
- Provider tool calling: Anthropic (tool_use blocks) + Google (functionCall parts)
- DB schema v2: mcp_servers table with CRUD commands
- Rust send_message: sends tools_enabled, records tool.requested + tool.completed events
- Sidecar: /mcp/connect, /mcp/disconnect, /mcp/tools endpoints, chat_with_tools loop
- UI: MCP Servers tab in Settings (add/remove/enable/disable), Zustand store wired
- Event WebSocket bridge: Sidecar EventBus + /events WS → Tauri WS client → SQLite + UI emit
- Cost calculation: model pricing table in Rust, auto-calc on llm.response.completed events
- Live Inspector: UI subscribes to agent_event, pushes events to store in real-time

**Previous sessions**:
- Session 1: Phase 1 foundation (d3684bf), isTauri fix (a681b59), camelCase fix (8dbe4a8)
- Session 2: PM workflow (CLAUDE.md + STATUS.md), dogfooding insight (d7808e7)
- Session 3: Inspector (3285434), node editor decision (755575e)
- Session 4a: MCP tool system (827e514)

**Next session should**:
1. Test full flow via `pnpm tauri dev` — MCP + events + cost
2. Runs execution (P2 #8) — create/execute endpoints + UI
3. Error handling polish (P2 #9)
4. DB wipe command (P2 #10)
