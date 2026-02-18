# AI Studio — Sprint Board

> This is the project's single source of truth for progress.
> New sessions: read this top-to-bottom, then you're caught up.

## Spec Roadmap

| # | Spec | Priority | Phase | Status | Notes |
|---|------|----------|-------|--------|-------|
| 1 | architecture.md | P0 | 1 | DONE | 3-layer system, IPC boundary |
| 2 | data-model.md | P0 | 1 | DONE | SQLite schema v2, WAL mode |
| 3 | event-system.md | P0 | 1 | DONE | Event bus + WS bridge + cost calc |
| 4 | api-contracts.md | P0 | 1 | DONE | All IPC commands implemented (v3.0 — workflows added) |
| 5 | agent-inspector.md | P0 | 1 | DONE | Timeline, detail, stats, filters, export |
| 6 | mcp-integration.md | P1 | 1 | DONE | Registry, tool calling, MCP client |
| 7 | ui-design.md | P1 | 2 | DONE | Polish pass — error handling, UX |
| 8 | hybrid-intelligence.md | P1 | 3 | DONE | Smart model routing, budget controls, savings tracking |
| 9 | phase-plan.md | — | — | REFERENCE | Implementation roadmap |
| 10 | use-cases.md | — | — | REFERENCE | Demo script, user scenarios |
| 11 | product-vision.md | — | — | REFERENCE | North star, positioning |
| 12 | node-editor.md | P0 | 3 | DONE | Visual pipeline builder — 8 node types, DAG engine, templates |
| 13 | plugin-system.md | P1 | 3 | DONE | Manifest, scanner, CRUD, Settings UI |

**Status key**: DONE | IN PROGRESS | PLANNED | BLOCKED | REFERENCE (non-implementable)

---

## Current Phase: 3 (Ecosystem + Node Editor)

**Goal**: Node editor (flagship Phase 3 feature), plugin system, templates, open-source launch prep.
**Status**: ALL TASKS COMPLETE. Phase 3 is ready for launch.
**Specs in scope**: `node-editor.md` (primary), `hybrid-intelligence.md`, `phase-plan.md` (3A-3C)

| Task | Spec | Status |
|------|------|--------|
| Node editor architecture spec | `node-editor.md` | DONE |
| Node editor architecture review | `node-editor.md` | DONE |
| CONTRIBUTING.md | — | DONE |
| Node editor foundation (3A) | `node-editor.md` | DONE |
| Node editor execution (3B) | `node-editor.md` | DONE |
| Node editor polish (3C) | `node-editor.md` | DONE |
| Hybrid intelligence routing | `hybrid-intelligence.md` | DONE |
| Rust module restructuring | — | DONE |
| Budget enforcement (deep critique fix) | `hybrid-intelligence.md` | DONE |
| Plugin system foundation | `plugin-system.md` | DONE |
| Plugin subprocess lifecycle | `plugin-system.md` | DONE |
| README update | — | DONE |
| One-click installers | `phase-plan.md` | DONE |
| Community templates | `phase-plan.md` | DONE (10 bundled + templates/README.md) |
| Open-source launch prep | — | DONE (CHANGELOG, SECURITY, Show HN, CI, issue templates) |

---

## Backlog (work top-down)

1. ~~Node editor foundation (3A)~~ DONE
2. ~~Node editor execution (3B)~~ DONE
3. ~~Node editor polish (3C)~~ DONE
4. ~~Hybrid intelligence routing~~ DONE
5. ~~Plugin system foundation~~ DONE (spec + schema v7 + CRUD + Settings UI)
6. ~~README update~~ DONE
7. ~~One-click installers~~ DONE (Tauri bundler config + MIT LICENSE)
8. ~~Plugin subprocess lifecycle~~ DONE (enable→spawn→MCP connect, disable→disconnect, startup auto-connect)
9. ~~Community template gallery~~ DONE (10 templates + templates/README.md contributor guide)
10. ~~Open-source launch prep~~ DONE (CHANGELOG.md, SECURITY.md, Show HN draft, CI, GitHub templates, package.json metadata)

---

## Done (Compressed)

**Phase 0** (5 sessions): Restructured to 5 pillars, removed old modules, wrote 11 specs.

**Phase 1** (COMPLETE): SQLite + CRUD (d3684bf) → chat sessions verified w/ Gemini → Inspector flagship (3285434) → MCP tool system (827e514) → event bridge + cost calc (ed629cf) → runs + DB wipe (ac9803d).

**Phase 2** (COMPLETE): Error handling polish + toasts (e4a8567). Agents schema alignment (8d370f0). Sidecar error events (30cd467). Onboarding wizard (b786c8b). Session branching (d3f22d9). Session branching review fixes (5778124). Inspector improvements (0a5895c).

**Phase 3** (COMPLETE): CONTRIBUTING.md (fe8ba6a). Node editor spec. Node editor review triaged (Gemini 3 Pro — 4/5 items fixed in spec, 1 deferred to 3B). **3A foundation DONE**: Schema v5 + workflow CRUD (3e6c277), Node Editor UI — 8 custom nodes, React Flow canvas, palette, config panel (d2eb98d). **3B execution DONE**: DAG walker engine, 7 node executors, validation, live node states, approval dialog, run button + input form. **3C polish DONE**: Codex review fixes (H1/H2/M1-M4 — 2380b83, 280be8c), Blender-inspired node restyling + collapse (ba82190), context menu + keyboard shortcuts (74d97df), 5 bundled templates + export/import (bb37147). **Hybrid Intelligence DONE**: Schema v6 (routing_mode, routing_rules on agents), Smart Router in Rust (3 modes: single/auto/manual, 14 unit tests), budget tracking (monthly cost aggregation, threshold warnings), UI (agent routing config, Settings budget tab), Inspector integration (llm.routed + budget.warning events, routing stats, savings tracking). **Rust refactoring** (6e338c9): monolithic commands.rs → 13 domain modules + workflow/. **Budget enforcement** (5117302): BudgetExhausted error, enforcement in chat + workflow before sidecar calls. **Plugin system** (750b4f6): Spec, schema v7, CRUD commands, scanner, Settings UI. **Plugin lifecycle** (0823cc8): Subprocess spawning via MCP connect, tool discovery, auto-connect on startup. **README** (9a630e8): Full update reflecting Phase 3 features. **Installers** (425f85c): Tauri bundler config + MIT LICENSE. **Template gallery** (c665ac6): 10 bundled templates + contributor guide. **Launch prep** (803724a): CHANGELOG, SECURITY, Show HN draft, package.json metadata.

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
| 2026-02-15 | **ChatGPT Codex scan triaged** | 6 findings: 4 false positives (spec is consistent), 2 valid low-priority (competitive-roadmap.md marked legacy, workflow commands added to api-contracts.md v3.0). |

---

## Gotchas

- `isTauri()`: Must check `__TAURI_INTERNALS__` (v2), not `__TAURI__` (v1) — fixed in a681b59
- **Tauri v2 IPC args are camelCase**: Rust `agent_id` → JS `{ agentId }`. NOT snake_case! This caused "missing required key" errors.
- Store errors: Every store action that calls `invoke` should set `error` state on failure, not swallow silently
- Sidecar provider config: Values stored as JSON strings with quotes — strip with `trim_matches('"')` in Rust

---

## Last Session Notes

**Date**: 2026-02-18 (session 20)
**What happened**:
- **Docker cleanup** (f702d14): Removed stale Telegram service, added Google AI key env var
- **Template gallery** (c665ac6): 5 new templates (Email Classifier, Content Moderator, Translation Pipeline, Meeting Notes, RAG Search) bringing total to 10. Added templates/README.md with format docs and contribution guide.
- **Launch prep** (803724a): CHANGELOG.md (full Phase 0-3 history), SECURITY.md (architecture security + disclosure policy), Show HN draft, package.json metadata (keywords, homepage, bugs).
- All 31 Rust tests pass, TypeScript clean
- **Phase 3 backlog is now fully complete.**

**Previous sessions**:
- Sessions 1-17: See git log for full history
- Session 18: Rust refactoring, budget enforcement, plugin foundation, README update
- Session 19: One-click installers, plugin subprocess lifecycle, open-source infra (CI, GitHub templates, CONTRIBUTING)

**Next session should**:
1. Phase 4 planning (universal automation canvas — custom node types, loop/merge, plugin marketplace)
2. Take fresh screenshots with running app for README
3. Consider GitHub Releases / v0.1.0 tag
