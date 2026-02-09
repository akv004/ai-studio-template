# AI Studio — Sprint Board

> This is the project's single source of truth for progress.
> New sessions: read this top-to-bottom, then you're caught up.

## Spec Roadmap

| # | Spec | Priority | Phase | Status | Notes |
|---|------|----------|-------|--------|-------|
| 1 | architecture.md | P0 | 1 | DONE | 3-layer system, IPC boundary |
| 2 | data-model.md | P0 | 1 | DONE | SQLite schema v2, WAL mode |
| 3 | event-system.md | P0 | 1 | DONE | Event bus + WS bridge + cost calc |
| 4 | api-contracts.md | P0 | 1 | DONE | All IPC commands implemented |
| 5 | agent-inspector.md | P0 | 1 | DONE | Timeline, detail, stats, filters, export |
| 6 | mcp-integration.md | P1 | 1 | DONE | Registry, tool calling, MCP client |
| 7 | ui-design.md | P1 | 2 | IN PROGRESS | Polish pass — error handling, UX |
| 8 | hybrid-intelligence.md | P1 | 3 | PLANNED | Smart model routing, budget controls |
| 9 | phase-plan.md | — | — | REFERENCE | Implementation roadmap |
| 10 | use-cases.md | — | — | REFERENCE | Demo script, user scenarios |
| 11 | product-vision.md | — | — | REFERENCE | North star, positioning |

**Status key**: DONE | IN PROGRESS | PLANNED | BLOCKED | REFERENCE (non-implementable)

---

## Current Phase: 2 (Polish + Power Features)

**Goal**: Polish UX, error handling, runs, session branching. Prep for open-source launch.
**Specs in scope**: `ui-design.md` (polish pass)

| Task | Spec | Status |
|------|------|--------|
| Runs execution | `api-contracts.md` | DONE |
| DB wipe command | — | DONE |
| Error handling polish | `ui-design.md` | DONE |
| Agents schema alignment | `data-model.md` | Backlog |
| Sidecar error events | `event-system.md` | Backlog |
| Onboarding / first-run UX | `ui-design.md` | Backlog |
| Session branching | `data-model.md` | Backlog |
| Inspector improvements | `agent-inspector.md` | Backlog |

---

## Backlog (work top-down)

1. Agents schema alignment — `data-model.md` — split `tools` JSON into `tools_mode`, `mcp_servers`, `approval_rules` per spec (do before branching touches DB)
2. Sidecar error events — `event-system.md` — emit `agent.error` / `system.error` for LLM-level crashes (only `tool.error` exists today)
3. Onboarding / first-run UX — `ui-design.md` — welcome modal or guided empty state when no agents exist (spec has 3-step wizard)
4. Session branching — `data-model.md`
5. Inspector improvements (replay, better detail panels) — `agent-inspector.md`
6. Phase 3: Node editor architecture — `product-vision.md`

---

## Done (Compressed)

**Phase 0** (5 sessions): Restructured to 5 pillars, removed old modules, wrote 11 specs.

**Phase 1** (COMPLETE): SQLite + CRUD (d3684bf) → chat sessions verified w/ Gemini → Inspector flagship (3285434) → MCP tool system (827e514) → event bridge + cost calc (ed629cf) → runs + DB wipe (ac9803d).

**Phase 2** (IN PROGRESS): Error handling polish + toasts (e4a8567).

Built: SQLite WAL schema v2, 5 LLM providers, MCP registry + stdio client, multi-turn tool calling, event-sourced persistence, WS bridge, cost calc (Claude/GPT/Gemini/local), Inspector (timeline/detail/stats/filters/export/keyboard nav), Runs (async bg execution + UI), DB wipe, all CRUD UIs, Zustand→IPC store, toast notification system, full error handling across all IPC calls.

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
| 2026-02-09 | Gemini 3 Pro design review — triaged | 3 items added to P2 backlog (schema alignment, sidecar error events, onboarding). 2 claims rejected: Command Palette already exists; INSERT OR IGNORE is correct. Hybrid intelligence & approval_rules already planned for P3. |

---

## Gotchas

- `isTauri()`: Must check `__TAURI_INTERNALS__` (v2), not `__TAURI__` (v1) — fixed in a681b59
- **Tauri v2 IPC args are camelCase**: Rust `agent_id` → JS `{ agentId }`. NOT snake_case! This caused "missing required key" errors.
- Store errors: Every store action that calls `invoke` should set `error` state on failure, not swallow silently
- Sidecar provider config: Values stored as JSON strings with quotes — strip with `trim_matches('"')` in Rust

---

## Last Session Notes

**Date**: 2026-02-09 (session 6)
**What happened**:
- Error handling polish (P2 #9): wrapped 9 unhandled store functions with try/catch + error state
- Toast notification system: Toasts.tsx component, slide-in-right animation (200ms per spec), auto-dismiss 5s
- Error auto-clear on page navigation (setActiveModule clears error state)
- SettingsPage: testConnection errors now surface via toast (was console.error only)
- RunsPage: cancelRun handler wrapped to catch rejected promises
- All MCP operations (add/update/remove) now show success/error toasts
- Build verified: TypeScript + Vite both pass clean
- Triaged Gemini 3 Pro design review → 3 items added to backlog, 2 rejected

**Previous sessions**:
- Session 1: Phase 1 foundation (d3684bf), isTauri fix (a681b59), camelCase fix (8dbe4a8)
- Session 2: PM workflow (CLAUDE.md + STATUS.md), dogfooding insight (d7808e7)
- Session 3: Inspector (3285434), node editor decision (755575e)
- Session 4a: MCP tool system (827e514)
- Session 4b: Event bridge + cost calc (ed629cf)
- Session 5: Runs execution + DB wipe (ac9803d)
- Session 6: Error handling polish + toasts (e4a8567), design review triage

**Next session should**:
1. Agents schema alignment — split `tools` → `tools_mode`/`mcp_servers`/`approval_rules` in db.rs + migration
2. Sidecar error events — emit `agent.error`/`system.error` for LLM crashes
3. Onboarding / first-run UX — welcome state or guided flow
4. Session branching — `data-model.md`
5. Inspector improvements — `agent-inspector.md`
