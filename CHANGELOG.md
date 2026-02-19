# Changelog

All notable changes to AI Studio are documented here.

## [0.1.0] - 2026-02-19

### Phase 4A: Universal Automation Canvas

**New Node Types**
- HTTP Request node: GET/POST/PUT/DELETE with headers, body, auth token support
- File Read node: text and binary modes with encoding selection
- File Write node: create/append/overwrite with directory auto-creation
- Shell Exec node: command execution with timeout, working directory, environment variables
- Validator node: JSON Schema validation with pass/fail routing

**Visual Overhaul**
- TypedEdge: colored bezier wires matching handle data types (magenta=text, orange=json, red=bool, green=float, gray=any)
- TypedConnectionLine: live drag preview with type-aware coloring
- Inline editing on all 8 node types (edit directly on canvas without config panel)
- CSS polish: shadows, glow effects, gradient backgrounds, edge animation
- Handle improvements: better spacing, hover states, type labels

**Vision Pipeline**
- File Read binary mode outputs base64 with mime_type detection
- LLM node accepts multiple image inputs from upstream nodes
- Sidecar builds OpenAI-compatible multi-image content blocks
- Validated end-to-end with Qwen3-VL (local vision model)

**Architecture**
- Monolith split: NodeEditorPage.tsx refactored into 16 focused modules
- Engine bug fixes: sourceHandle routing + clean_output preservation for structured data
- Validation relaxation: file_read counts as input source, file_write as output sink

**Testing**
- Playwright E2E infrastructure with Tauri IPC mock (30+ commands)
- 15 tests: 6 UI tests (sidebar, canvas, node palette) + 7 sidecar API tests + 2 integration
- Screenshots capture full UI state for visual regression
- Configurable via environment variables (SIDECAR_URL, LLM_BASE_URL, LLM_MODEL)

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
