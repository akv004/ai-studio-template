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
| 7 | ui-design.md | P1 | 2 | DONE | Polish pass — error handling, UX |
| 8 | hybrid-intelligence.md | P1 | 3 | PLANNED | Smart model routing, budget controls |
| 9 | phase-plan.md | — | — | REFERENCE | Implementation roadmap |
| 10 | use-cases.md | — | — | REFERENCE | Demo script, user scenarios |
| 11 | product-vision.md | — | — | REFERENCE | North star, positioning |
| 12 | node-editor.md | P0 | 3 | IN PROGRESS | Visual pipeline builder — the 10k-star feature |

**Status key**: DONE | IN PROGRESS | PLANNED | BLOCKED | REFERENCE (non-implementable)

---

## Current Phase: 3 (Ecosystem + Node Editor)

**Goal**: Node editor (flagship Phase 3 feature), plugin system, templates, open-source launch prep.
**Specs in scope**: `node-editor.md` (primary), `hybrid-intelligence.md`, `phase-plan.md` (3A-3C)

| Task | Spec | Status |
|------|------|--------|
| Node editor architecture spec | `node-editor.md` | DONE |
| Node editor architecture review | `node-editor.md` | DONE |
| CONTRIBUTING.md | — | DONE |
| Node editor foundation (3A) | `node-editor.md` | TODO |
| Node editor execution (3B) | `node-editor.md` | TODO |
| Node editor polish (3C) | `node-editor.md` | TODO |
| Hybrid intelligence routing | `hybrid-intelligence.md` | TODO |
| Plugin system | `phase-plan.md` | TODO |
| Community templates | `phase-plan.md` | TODO |
| One-click installers | `phase-plan.md` | TODO |

---

## Backlog (work top-down)

1. **Node editor foundation (3A)** — `node-editor.md` — Install React Flow, schema v4, workflow CRUD, canvas, node palette, custom node components, save/load
2. **Node editor execution (3B)** — `node-editor.md` — DAG walker in Rust, LLM/tool/router node execution, live node states, data preview
3. **Node editor polish (3C)** — `node-editor.md` — Templates, Inspector integration, agent↔workflow linking, subworkflows, batch runs
4. Hybrid intelligence routing — `hybrid-intelligence.md` — Smart model router, budget controls, savings tracking
5. Plugin system — `phase-plan.md` — Plugin manifest, loader, permissions, UI panels
6. Community templates — `phase-plan.md` — Bundled templates, import/export, gallery
7. One-click installers — `phase-plan.md` — DMG, MSI, AppImage via Tauri bundler
8. README update — Update roadmap status, add node editor screenshots

---

## Done (Compressed)

**Phase 0** (5 sessions): Restructured to 5 pillars, removed old modules, wrote 11 specs.

**Phase 1** (COMPLETE): SQLite + CRUD (d3684bf) → chat sessions verified w/ Gemini → Inspector flagship (3285434) → MCP tool system (827e514) → event bridge + cost calc (ed629cf) → runs + DB wipe (ac9803d).

**Phase 2** (COMPLETE): Error handling polish + toasts (e4a8567). Agents schema alignment (8d370f0). Sidecar error events (30cd467). Onboarding wizard (b786c8b). Session branching (d3f22d9). Session branching review fixes (5778124). Inspector improvements (0a5895c).

**Phase 3** (IN PROGRESS): CONTRIBUTING.md (fe8ba6a). Node editor spec. Node editor review triaged (Gemini 3 Pro — 4/5 items fixed in spec, 1 deferred to 3B).

Built: SQLite WAL schema v3, 5 LLM providers, MCP registry + stdio client, multi-turn tool calling, event-sourced persistence, WS bridge, cost calc (Claude/GPT/Gemini/local), Inspector (timeline/detail/stats/filters/export/keyboard nav/grouping/actions), Runs (async bg execution + UI), DB wipe, all CRUD UIs, Zustand→IPC store, toast notification system, full error handling, onboarding wizard, session branching, peer review workflow.

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
| 2026-02-15 | **React Flow (@xyflow/react) for node editor** | 35K stars, 3M weekly downloads, MIT, native React 19 + TS + Tailwind, built-in JSON serialization, proven in AI workflows (Langflow, Firecrawl). Evaluated: Rete.js (smaller, styled-components conflict), Litegraph (archived), Butterfly (abandoned). React Flow is the only production-ready option. |
| 2026-02-15 | **Workflow execution uses `/chat/direct` (stateless)** | Gemini 3 Pro review caught that `/chat` (stateful) would cause split-brain bugs in DAG execution. Rust owns all context — builds message history per node. Sidecar is pure compute. Same class of bug as session branching context loss. |
| 2026-02-15 | **Parallel branch execution via `tokio::join_all`** | Independent DAG branches run concurrently (default limit: 4). Sidecar needs matching `uvicorn --workers` count. |

---

## Gotchas

- `isTauri()`: Must check `__TAURI_INTERNALS__` (v2), not `__TAURI__` (v1) — fixed in a681b59
- **Tauri v2 IPC args are camelCase**: Rust `agent_id` → JS `{ agentId }`. NOT snake_case! This caused "missing required key" errors.
- Store errors: Every store action that calls `invoke` should set `error` state on failure, not swallow silently
- Sidecar provider config: Values stored as JSON strings with quotes — strip with `trim_matches('"')` in Rust

---

## Last Session Notes

**Date**: 2026-02-15 (session 11)
**What happened**:
- Triaged Gemini 3 Pro node editor architecture review
  - 4/5 items fixed in spec (stateless sidecar, parallelism, version clarification, subworkflow cycles)
  - 1 item deferred to 3B (mock tests — comes with execution engine)
  - Key fix: workflow execution mandates `/chat/direct` (stateless), Rust builds context per node
  - Added parallel execution model: `tokio::join_all`, default concurrency 4
  - Added subworkflow cycle detection to validation

**Previous sessions**:
- Session 1: Phase 1 foundation (d3684bf), isTauri fix (a681b59), camelCase fix (8dbe4a8)
- Session 2: PM workflow (CLAUDE.md + STATUS.md), dogfooding insight (d7808e7)
- Session 3: Inspector (3285434), node editor decision (755575e)
- Session 4a: MCP tool system (827e514)
- Session 4b: Event bridge + cost calc (ed629cf)
- Session 5: Runs execution + DB wipe (ac9803d)
- Session 6: Error handling polish + toasts (e4a8567), design review triage
- Session 7: Agents schema alignment (8d370f0) + sidecar error events (30cd467)
- Session 8: Onboarding wizard (b786c8b)
- Session 9: Session branching (d3f22d9) + review fixes (5778124) + inspector improvements (0a5895c)
- Session 10: CONTRIBUTING.md + node editor spec (Phase 3 start)
- Session 11: Node editor review triage (Gemini 3 Pro)

**Next session should**:
1. Phase 3A: Start building node editor foundation — install React Flow, schema v4 migration, workflow CRUD, basic canvas
2. Spec is now reviewed and fixed — ready to build
