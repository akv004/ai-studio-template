# Changelog

All notable changes to AI Studio are documented here.

## [0.1.0] - 2026-02-18

### Phase 3: Advanced Features

**Node Editor — Visual AI Pipelines**
- 8 custom node types: Input, Output, LLM, Tool, Router, Approval, Transform, Subworkflow
- React Flow canvas with drag-and-drop from palette, config panel, sidebar
- DAG execution engine in Rust (topological sort, parallel branches via `tokio::join_all`)
- Live execution view with node status badges and per-node cost tracking
- Blender-inspired dark theme with labeled handles, collapsible nodes
- Context menu + keyboard shortcuts (Ctrl+D/A/C/V)
- 10 bundled workflow templates (Code Review, Research, Data Pipeline, Multi-Model Compare, Safe Executor, Email Classifier, Content Moderator, Translation Pipeline, Meeting Notes, RAG Search)
- JSON export/import for workflow sharing

**Hybrid Intelligence**
- Smart model router with 3 modes: single, auto (built-in rules), manual (user rules)
- Auto-routing: vision tasks → GPT-4o, code → Claude, large context → Gemini, simple → local
- Monthly budget tracking with threshold warnings at 50/80/100%
- Budget enforcement: `local_only`, `cheapest_cloud`, or `ask` when exhausted
- Inspector integration: routing events, savings tracking, per-model stats

**Plugin System**
- Plugin manifest format (`plugin.json`) with permission declarations
- Directory scanner for `~/.ai-studio/plugins/`
- Enable/disable/remove from Settings UI
- MCP subprocess lifecycle: plugins communicate via stdio JSON-RPC
- Auto-connect enabled plugins on app startup

**Open-Source Infrastructure**
- MIT license
- CONTRIBUTING.md with architecture overview and development guide
- GitHub Actions CI (Rust check + tests, TypeScript type check)
- Issue templates (bug report, feature request)
- Pull request template
- SECURITY.md with responsible disclosure policy
- Cross-platform installers via Tauri bundler (DMG, MSI, DEB, AppImage)

### Phase 2: Polish

- Session branching (fork from any message, compare approaches)
- Inspector improvements (event grouping, keyboard navigation, markdown export)
- Onboarding wizard for first-time setup
- Toast notification system with auto-dismiss
- Error handling across all 19 store actions
- Sidecar error event propagation

### Phase 1: Core Loop

- SQLite local-first persistence (WAL mode, 7 schema migrations)
- 5 LLM providers: Ollama, Anthropic, OpenAI, Google AI, Azure OpenAI
- Agent CRUD with model configuration, system prompts, tool permissions
- Interactive chat sessions with real-time streaming
- Agent Inspector: event timeline, detail panel, stats, filters, export
- MCP-native tool system with registry, approval workflow, stdio client
- Built-in tools: shell, filesystem
- Multi-turn tool calling with event-sourced audit trail
- Headless runs for batch execution
- Event WebSocket bridge between Rust and Python sidecar
- Cost calculation per model (Claude, GPT, Gemini, local)

### Phase 0: Foundation

- Project restructure to 5-pillar architecture (Agents, Sessions, Runs, Inspector, Settings)
- 12 design specifications covering all features
- Tauri 2 + React 19 + Python FastAPI stack
- Monorepo with shared TypeScript types
