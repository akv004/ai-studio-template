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

This project spans many sessions. Follow this workflow every time:

### On Session Start
1. **Read `STATUS.md`** first — it has what's done, what's not, bugs fixed, and what to do next
2. Don't re-explore the codebase if STATUS.md already covers it
3. Ask the user what they want to work on, or pick up from the "Next" items in STATUS.md

### While Working
4. **Commit after each meaningful chunk** — don't let uncommitted changes pile up. Small, frequent commits.
5. When you hit a bug or learn something non-obvious, note it in STATUS.md under "Bugs Fixed" or "Gotchas"

### On Session End (or after completing a task)
6. **Update `STATUS.md`** — move completed items to "Done", add new items to "Not Done", update "Last Session Notes"
7. **Commit STATUS.md** — so the next session sees it immediately
8. Keep STATUS.md concise — it's a handoff note, not documentation

### Key Rules
- Never let a session end with a large amount of uncommitted work
- STATUS.md is the source of truth for "where are we" — keep it accurate
- Specs in `docs/specs/` are the source of truth for "what should we build"
- If something in STATUS.md conflicts with code reality, update STATUS.md to match reality

## Architecture (3 Layers)

```
UI (React 19) → Tauri IPC → Rust Backend (SQLite) → Python Sidecar (LLM providers)
```

- UI never talks to sidecar directly — all through Tauri IPC
- Tauri owns the security boundary (tool approval, token auth, CORS)
- Sidecar is spawned by Tauri with an auth token; all requests must include it
- Event-sourced: every action emits typed events stored in SQLite

## Key Specs

All specs in `docs/specs/`:
- product-vision.md, architecture.md, event-system.md, data-model.md
- agent-inspector.md, mcp-integration.md, hybrid-intelligence.md
- api-contracts.md, ui-design.md, use-cases.md, phase-plan.md

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
