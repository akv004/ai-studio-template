# AI Studio

**The open-source IDE for AI agents.** "Chrome DevTools for AI agents."

## Quick Context

- **Goal**: Open-source, fill the gap in AI/ML tooling
- **Stack**: Tauri 2 (Rust) + React 19 + Python FastAPI sidecar
- **5 Pillars**: Agents, Sessions, Runs, Inspector (flagship), Settings
- **Flagship Feature**: Agent Inspector — event timeline, replay, branching, cost tracking
- **Persistence**: SQLite local-first via Tauri/Rust layer
- **Tool System**: MCP-native from day one
- **Intelligence**: Hybrid — smart router picks best model per step with budget controls

## Session Workflow (IMPORTANT)

This is a spec-driven, multi-session project. Follow this workflow every time:

### On Session Start
1. **Read `STATUS.md`** — what's done, what's not, current phase, bugs, and what to do next
2. **Check the current phase** in STATUS.md — read the relevant specs before coding
3. Don't re-explore the codebase if STATUS.md already covers it
4. Ask the user what to work on, or pick up from "Next" items in STATUS.md

### Before Implementing a Feature
5. **Read the relevant spec(s)** from `docs/specs/` — they define the contracts, schemas, and behavior
6. Follow the spec. If something in the spec is wrong or outdated, flag it to the user before diverging.

### While Working
7. **Commit after each meaningful chunk** — don't let uncommitted changes pile up
8. When you hit a bug or learn something non-obvious, note it in STATUS.md "Gotchas" or CLAUDE.md "Gotchas"

### On Session End (or after completing a task)
9. **Update `STATUS.md`** — check off completed items, add new items, update "Last Session Notes" with what to do next
10. **Commit STATUS.md** — so the next session sees it immediately

### Key Rules
- Never let a session end with large uncommitted work
- `STATUS.md` = "where are we" (dynamic, updated every session)
- `docs/specs/` = "what should we build" (stable, the blueprint)
- `CLAUDE.md` = "how to work on this project" (workflow + gotchas)
- If STATUS.md conflicts with code reality, update STATUS.md to match reality

## Spec-Driven Development

### Specs → Code Mapping

When implementing a feature, read the spec first. Here's which spec covers what:

| Working on... | Read this spec |
|---------------|---------------|
| DB schema, migrations, branching | `data-model.md` |
| Tauri IPC commands, sidecar endpoints | `api-contracts.md` |
| Event types, envelope format, cost calc | `event-system.md` |
| Inspector timeline, detail panel, replay | `agent-inspector.md` |
| MCP tools, discovery, approval flow | `mcp-integration.md` |
| Smart model routing, budgets | `hybrid-intelligence.md` |
| UI layout, colors, components | `ui-design.md` |
| Overall architecture, layer responsibilities | `architecture.md` |
| User scenarios, demo script | `use-cases.md` |
| Phase plan, task ordering | `phase-plan.md` |
| Product direction, positioning | `product-vision.md` |

### Phase Plan (from `docs/specs/phase-plan.md`)

| Phase | Goal | Key Deliverables |
|-------|------|-----------------|
| **0** | Foundation | Restructure to 5 pillars, remove old modules, update types |
| **1** | Core working product | SQLite + events + agent CRUD + chat sessions + basic inspector + MCP |
| **2** | Power features | Replay, branching, headless runs, cost breakdown, export |
| **3** | Open-source launch | Plugin system, templates, installers, community |

**Phase 1 success criteria**: Create agent → chat with it → see tools execute → inspect session with events & cost.

**Phase 1 sub-tasks** (build order):
- 1A: SQLite schema + CRUD commands ✅
- 1B: Event system (types, recording) ✅ (basic — needs WebSocket bridge)
- 1C: Agent CRUD UI ✅
- 1D: Real chat sessions with persistence (backend ✅, UI in progress)
- 1E: Basic Inspector (timeline, detail, stats, filters)
- 1F: MCP tool discovery + execution + settings UI

## Architecture (3 Layers)

```
UI (React 19) → Tauri IPC → Rust Backend (SQLite) → Python Sidecar (LLM providers)
```

- UI never talks to sidecar directly — all through Tauri IPC
- Tauri owns the security boundary (tool approval, token auth, CORS)
- Sidecar is spawned by Tauri with an auth token; all requests must include it
- Event-sourced: every action emits typed events stored in SQLite

## Build / Run

```bash
# Full desktop app (Rust + UI + auto-spawns sidecar)
cd apps/desktop && pnpm tauri dev

# UI only (for frontend work, no Tauri IPC)
cd apps/ui && pnpm dev

# Sidecar standalone (for testing providers)
cd apps/sidecar && python -m uvicorn server:app --port 8765
```

## Gotchas

- `isTauri()` check: Tauri v2 uses `window.__TAURI_INTERNALS__`, not `window.__TAURI__` (v1)
- Tauri IPC args use snake_case matching Rust param names (e.g. `{ agent_id: '...' }`)
- Sidecar auth: every non-health request needs `x-ai-studio-token` header
- Settings stored as `provider.{id}.{field}` keys in SQLite settings table
- Values stored as JSON strings — strip quotes when reading (`value.trim_matches('"')`)
