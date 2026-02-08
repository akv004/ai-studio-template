# AI Studio — Phase Plan

> **Version**: 2.0
> **Status**: Draft
> **Depends on**: All other specs

---

## Current State (What Exists Today)

| Component | Status | Notes |
|---|---|---|
| Tauri desktop shell | Working | Window, build, cross-platform |
| React UI with 7 pages | Working | All mock data, needs restructure to 5 pillars |
| Python sidecar lifecycle | Working | Spawn, health check, token auth, kill |
| Multi-provider LLM chat | Working | Ollama, Anthropic, OpenAI, Google |
| Tool execution (shell, fs, browser) | Working | REST endpoints, safety modes |
| Tool approval modal | Working | Tauri event → UI modal → approve/deny |
| Keyboard shortcuts + command palette | Working | Cmd+K, Cmd+1-6 navigation |
| Zustand state (mock data) | Working | Needs migration to SQLite-backed |

---

## Phase 0: Foundation (Specs + Cleanup)

**Goal**: Lock all specs, clean up codebase, prepare for real implementation.

**Status**: In progress (this document completes it).

### Tasks

| # | Task | Effort | Output |
|---|---|---|---|
| 0.1 | Write all design specs | Done | 8 spec documents |
| 0.2 | Remove Vision, Audio, Training pages | Low | Delete 3 page files + sidebar entries |
| 0.3 | Rename "Projects" page → restructure to Agents | Low | Update routing, sidebar |
| 0.4 | Update sidebar to 5 pillars | Low | Agents, Sessions, Runs, Inspector, Settings |
| 0.5 | Update shared types to match new data model | Low | New Agent, Session, Event types |
| 0.6 | Fix CORS wildcard security issue | Low | Strict origins in sidecar |
| 0.7 | Update README to reflect new direction | Medium | New screenshots, architecture, features |

**Deliverable**: Clean codebase with 5-pillar navigation and all specs finalized.

---

## Phase 1: Core Loop

**Goal**: A working product where you can create an agent, chat with it, see tool calls happen, and inspect the session afterward. Real data, real persistence, real events.

### 1A — SQLite Persistence (Tauri/Rust)

| # | Task | Effort | Notes |
|---|---|---|---|
| 1A.1 | Add rusqlite dependency to Cargo.toml | Low | |
| 1A.2 | Create `db/` module with schema.sql + migrations | Medium | Tables: _meta, agents, sessions, messages, events, settings, mcp_servers, provider_keys, approval_rules |
| 1A.3 | DB init on app startup (create/migrate) | Medium | Check version, run migrations |
| 1A.4 | Implement agent CRUD commands | Medium | list, get, create, update, delete |
| 1A.5 | Implement session CRUD commands | Medium | list, get, create, delete |
| 1A.6 | Implement message read/write | Medium | Insert on send, query on load |
| 1A.7 | Implement event write + query | Medium | Insert from event bridge, query for Inspector |
| 1A.8 | Implement settings get/set | Low | Key-value store |
| 1A.9 | Implement provider key management | Low | Store API keys |
| 1A.10 | Remove mock data from UI store | Low | Replace with IPC calls |

### 1B — Event System

| # | Task | Effort | Notes |
|---|---|---|---|
| 1B.1 | Define event types in Python (Pydantic models) | Low | Based on event-system.md |
| 1B.2 | Define event types in TypeScript | Low | Mirror Python types |
| 1B.3 | Add WebSocket `/events` endpoint to sidecar | Medium | FastAPI WebSocket handler |
| 1B.4 | Add event emission to chat flow | Medium | llm.request.started, llm.response.chunk, llm.response.completed |
| 1B.5 | Add event emission to tool flow | Low | tool.requested, tool.completed, tool.error |
| 1B.6 | Build Tauri WebSocket client (event bridge) | Medium | Connect to /events, persist to SQLite, emit to UI |
| 1B.7 | Add seq number management in Tauri | Medium | Master counter per session |

### 1C — Agent CRUD UI

| # | Task | Effort | Notes |
|---|---|---|---|
| 1C.1 | AgentsPage: list view with cards | Medium | Name, model, provider, tool count, last used |
| 1C.2 | Agent create/edit form | Medium | All fields from CreateAgentRequest |
| 1C.3 | Agent delete with confirmation | Low | |
| 1C.4 | Provider + model selector dropdown | Medium | Query available providers/models from sidecar |

### 1D — Sessions (Real Chat)

| # | Task | Effort | Notes |
|---|---|---|---|
| 1D.1 | SessionsPage: session list sidebar | Medium | List sessions, select to open |
| 1D.2 | Chat interface (message input + display) | Medium | Based on existing AgentsPage chat, but real |
| 1D.3 | Wire send_message → Tauri → sidecar | Medium | Full flow with events |
| 1D.4 | Streaming response display | Medium | Listen to llm.response.chunk events |
| 1D.5 | Tool approval inline (in chat, not just modal) | Medium | Show approval request in chat flow |
| 1D.6 | Session persistence (resume on reopen) | Medium | Load messages from SQLite |
| 1D.7 | New session creation from agent | Low | "Start session" button on agent card |

### 1E — Basic Inspector

| # | Task | Effort | Notes |
|---|---|---|---|
| 1E.1 | InspectorPage: session selector | Low | Dropdown or list of sessions |
| 1E.2 | Event timeline (left panel) | High | Vertical list, color-coded, grouped |
| 1E.3 | Detail panel (right panel) | High | Adaptive content per event type |
| 1E.4 | Stats bar (bottom) | Medium | Token counts, cost, duration, tool counts |
| 1E.5 | Filter chips (Messages, LLM, Tools, Errors) | Low | |
| 1E.6 | Live inspection (real-time for active sessions) | Medium | Listen to agent_event, append to timeline |

### 1F — MCP Integration

| # | Task | Effort | Notes |
|---|---|---|---|
| 1F.1 | Add `mcp` SDK to sidecar requirements | Low | |
| 1F.2 | MCP client manager (connect, disconnect, lifecycle) | Medium | stdio transport first |
| 1F.3 | Tool discovery from MCP servers | Low | tools/list → registry |
| 1F.4 | Wrap built-in tools as MCP-compatible handlers | Low | Same interface, no subprocess |
| 1F.5 | Wire MCP tools into LLM tool definitions | Medium | Merge built-in + MCP tools per agent config |
| 1F.6 | MCP tool execution with approval flow | Medium | Same approval pipeline as built-in tools |
| 1F.7 | Settings UI: MCP server management | Medium | Add, edit, remove, test connection |
| 1F.8 | MCP server config persistence (SQLite) | Low | Already in schema |

### Phase 1 Deliverable

A real, working product:
- Create agents with specific models, prompts, and tools
- Chat with agents in persistent sessions
- Tools execute with approval (built-in + MCP)
- Every action is recorded as events
- Inspector shows full session timeline with stats
- All data persisted in SQLite

---

## Phase 2: Power Features

**Goal**: The Inspector becomes the flagship. Replay, branching, cost tracking, export. Runs work headlessly.

### 2A — Full Inspector

| # | Task | Effort | Notes |
|---|---|---|---|
| 2A.1 | Event grouping (tool call groups, LLM groups) | Medium | Collapsible groups in timeline |
| 2A.2 | Search across events (Cmd+F) | Medium | Search content, tool inputs/outputs |
| 2A.3 | Keyboard navigation (arrow keys, vim-style) | Medium | |
| 2A.4 | Cost calculation from pricing table | Low | Settings-stored pricing × token counts |
| 2A.5 | Cost breakdown in stats bar | Low | Per-model, per-provider |
| 2A.6 | Export as JSON | Medium | Full event log + metadata |
| 2A.7 | Export as Markdown | Medium | Human-readable transcript |
| 2A.8 | Virtualized timeline (react-window) | Medium | Performance for 1000+ events |

### 2B — Replay & Branch

| # | Task | Effort | Notes |
|---|---|---|---|
| 2B.1 | Branch session (create from point) | High | Copy messages, create new session |
| 2B.2 | Replay from point (re-execute with same/modified context) | High | Replay button in Inspector |
| 2B.3 | Compare view (side-by-side sessions) | High | Two timeline panels, diff highlighting |
| 2B.4 | Branch indicator in session list | Low | Show parent relationship |

### 2C — Runs

| # | Task | Effort | Notes |
|---|---|---|---|
| 2C.1 | RunsPage: run list with status | Medium | |
| 2C.2 | Create run form (select agent, input prompt, auto-approve rules) | Medium | |
| 2C.3 | Run execution engine (Tauri → sidecar, headless) | High | Same as session but no interactive approval |
| 2C.4 | Run status tracking (real-time via events) | Medium | |
| 2C.5 | Run → Inspector link | Low | Open Inspector for run's session |
| 2C.6 | Cancel run | Low | |

### 2D — Auto-Approval Rules

| # | Task | Effort | Notes |
|---|---|---|---|
| 2D.1 | Global approval rules UI in Settings | Medium | |
| 2D.2 | Per-agent approval rules in agent config | Low | Already in schema |
| 2D.3 | Rule evaluation engine in Tauri | Medium | Pattern matching, priority ordering |
| 2D.4 | "Auto-approved by rule: X" display in Inspector | Low | |

### Phase 2 Deliverable

Feature-complete for developers:
- Inspector with replay, branching, side-by-side compare
- Cost tracking with per-model pricing
- Export sessions as JSON or Markdown
- Headless runs with auto-approval
- Full keyboard navigation

---

## Phase 3: Ecosystem

**Goal**: Plugin system, community templates, polish for open-source launch.

### 3A — Plugin System

| # | Task | Effort | Notes |
|---|---|---|---|
| 3A.1 | Plugin manifest format (plugin.toml) | Medium | Based on future-capabilities.md |
| 3A.2 | Plugin loader (subprocess isolation) | High | |
| 3A.3 | Plugin permission declarations | Medium | |
| 3A.4 | Plugin UI panels | High | Render plugin-provided React components |

### 3B — Community & Templates

| # | Task | Effort | Notes |
|---|---|---|---|
| 3B.1 | Bundled agent templates | Low | Code Assistant, Research Bot, Data Analyst |
| 3B.2 | Import/export agent configs | Low | JSON files |
| 3B.3 | Community template gallery (GitHub-based) | Medium | |

### 3C — Polish & Launch

| # | Task | Effort | Notes |
|---|---|---|---|
| 3C.1 | One-click installer (DMG, MSI, AppImage) | Medium | Tauri bundler config |
| 3C.2 | First-run onboarding flow | Medium | Provider setup wizard |
| 3C.3 | Product Hunt / Show HN assets | Medium | Demo video, screenshots, description |
| 3C.4 | Contributing guide (CONTRIBUTING.md) | Low | |
| 3C.5 | Discord community setup | Low | |

### Phase 3 Deliverable

Open-source launch-ready product with plugin extensibility and community features.

---

## Effort Summary

| Phase | Tasks | Estimated Effort |
|---|---|---|
| Phase 0: Foundation | 7 tasks | Small (cleanup + specs) |
| Phase 1: Core Loop | ~30 tasks | Large (the real build) |
| Phase 2: Power Features | ~15 tasks | Medium-Large (flagship polish) |
| Phase 3: Ecosystem | ~10 tasks | Medium (community + plugins) |

---

## Implementation Order Within Phase 1

This is the recommended build order (dependencies flow downward):

```
1A (SQLite) ──────────────────────────────┐
    │                                     │
1B (Events) ─────────────────────┐        │
    │                            │        │
1C (Agent CRUD UI) ──────┐      │        │
    │                    │      │        │
1F (MCP Integration) ────┤      │        │
    │                    │      │        │
1D (Sessions / Chat) ────┴──────┴────────┘
    │
1E (Basic Inspector) ────────────────────
```

Start with SQLite + Events in parallel, then Agent UI + MCP in parallel, then Sessions (which depends on everything), then Inspector (which reads what Sessions produce).

---

## What Success Looks Like

### After Phase 1
A developer can:
1. Launch AI Studio
2. Add their Anthropic API key
3. Create an agent ("Code Helper" with Claude Sonnet)
4. Add a GitHub MCP server
5. Chat: "Create an issue for the login bug"
6. See tool approval → approve → issue created
7. Open Inspector → see every event, tokens, cost

### After Phase 2
Same developer can:
8. Replay the session with a different prompt
9. Branch and try GPT-4o instead of Claude
10. Compare cost and quality side-by-side
11. Run a batch of 10 prompts headlessly
12. Export results as JSON for analysis

### After Phase 3
The developer shares it:
13. Installs via one-click DMG
14. Finds a "Code Reviewer" template in the gallery
15. Installs a community Jira MCP plugin
16. Stars the repo on GitHub
