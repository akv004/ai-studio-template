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

## Current State

- Phase 0 (Specs): Done — 11 design specs in `docs/specs/`
- Phase 1 (Core Loop): Next — SQLite, events, agent CRUD, sessions, basic inspector, MCP
- UI currently has old 7-module structure (needs cleanup to 5 pillars)
- All data is mocked — no real persistence yet

## Key Specs

All specs are in `docs/specs/`:
- product-vision.md, architecture.md, event-system.md, data-model.md
- agent-inspector.md, mcp-integration.md, hybrid-intelligence.md
- api-contracts.md, ui-design.md, use-cases.md, phase-plan.md
