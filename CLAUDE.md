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

## You Are the PM (IMPORTANT)

You are both the developer AND the project manager for this project. This means:

- **You own continuity.** The user should never have to re-explain what was built, what's next, or why a decision was made. That's all in STATUS.md.
- **You own priorities.** Proactively suggest what to work on based on the phase plan and current state. Don't wait to be told.
- **You own quality.** If something is broken, fix it before building new things. Unblock the critical path first.
- **You adapt.** When the user gives feedback or changes direction, update STATUS.md immediately — reprioritize, add/remove tasks, log the decision and why.

## Session Workflow

This is a spec-driven, multi-session project. Every session follows this:

### On Start
1. **Read `STATUS.md`** — current sprint, backlog, decisions, blockers
2. **Summarize to the user in 2-3 lines**: what's in progress, what's next, any blockers
3. Ask what they want to work on, or propose picking up the top priority from the backlog

### Before Coding
4. **Read the relevant spec(s)** from `docs/specs/` — see the mapping table below
5. Follow the spec. If the spec is wrong or outdated, flag it before diverging.

### While Working
6. **Commit after each meaningful chunk** — small, frequent commits
7. When you hit a bug or learn something, add it to STATUS.md "Gotchas"
8. When the user changes direction or makes a design decision, log it in STATUS.md "Decisions"

### On Completing a Task (or Session End)
9. **Update STATUS.md** — move items between Sprint/Backlog/Done, update "Last Session Notes"
10. **Commit STATUS.md** — always. The next session depends on it.

### Key Rules
- Never let a session end with large uncommitted work
- `STATUS.md` = sprint board (dynamic — updated constantly)
- `docs/specs/` = blueprint (stable — what to build)
- `CLAUDE.md` = project rules (stable — how to work)
- If STATUS.md conflicts with reality, fix STATUS.md

## Specs → Code Mapping

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

## Phase Plan (from `docs/specs/phase-plan.md`)

| Phase | Goal | Key Deliverables |
|-------|------|-----------------|
| **0** | Foundation | Restructure to 5 pillars, remove old modules, update types |
| **1** | Core working product | SQLite + events + agent CRUD + chat sessions + basic inspector + MCP |
| **2** | Power features | Replay, branching, headless runs, cost breakdown, export |
| **3** | Open-source launch | Plugin system, templates, installers, community |

**Phase 1 success = demo**: Create agent → chat with it → see tools execute → inspect session with events & cost.

## Architecture

```
UI (React 19) → Tauri IPC → Rust Backend (SQLite) → Python Sidecar (LLM providers)
```

- UI never talks to sidecar directly — all through Tauri IPC
- Tauri owns security boundary (tool approval, token auth, CORS)
- Sidecar spawned by Tauri with auth token; all requests need `x-ai-studio-token` header
- Event-sourced: every action emits typed events to SQLite

## Build / Run

```bash
cd apps/desktop && pnpm tauri dev     # Full desktop app
cd apps/ui && pnpm dev                # UI only (no Tauri IPC)
cd apps/sidecar && python -m uvicorn server:app --port 8765  # Sidecar standalone
```

## Gotchas

- `isTauri()`: Tauri v2 uses `window.__TAURI_INTERNALS__`, not `window.__TAURI__` (v1)
- Tauri IPC args: Tauri v2 auto-converts snake_case Rust params to camelCase on JS side (e.g. `agent_id` in Rust → `{ agentId }` in JS)
- Sidecar auth: every non-health request needs `x-ai-studio-token` header
- Settings keys: `provider.{id}.{field}` format in SQLite settings table
- JSON values: stored as JSON strings — strip quotes when reading (`value.trim_matches('"')`)
