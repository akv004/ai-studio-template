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
| 14 | killer-features.md | P0 | 5 | PLANNED | Time-Travel, Auto-Pipeline, Guardrails, SQL |
| 25 | step-through-debugging.md | P0 | 5A | SPEC DONE | Breakpoints, F10/F5, Edit & Continue for AI workflows |
| 26 | edge-data-preview.md | P1 | 5A | DONE | X-Ray mode — data values on canvas edges (b31e610) |
| 27 | prompt-version-control.md | P1 | 5B | SPEC DONE | Auto-versioning LLM prompts, diff, rollback |
| 28 | natural-language-canvas.md | P0 | 5C | SPEC DONE | Chat-to-graph modification, preview-before-apply |
| 29 | ai-workflow-copilot.md | P2 | 5D | SPEC DONE | Run history insights, self-optimizing pipelines |
| 15 | connections-manager.md | P0 | 5B | PLANNED | Unified credential store, encrypted at rest, DB/HTTP/SMTP/Webhook |
| 16 | triggers-scheduling.md | P0 | 5B | PLANNED | Webhook, cron, file watch, event triggers |
| 17 | streaming-output.md | P1 | 5A | DONE | SSE token streaming — all 6 providers (Ollama, OpenAI, Azure, Google, Anthropic, Local) |
| 18 | batch-runs.md | P1 | 5B | PLANNED | Dataset import, batch execution, progress dashboard |
| 19 | rich-output.md | P1 | 5A | IN PROGRESS | Markdown, tables, JSON tree/table, code blocks — wired into 5 spots. Charts/images deferred. |
| 20 | workflow-versioning.md | P2 | 5B | PLANNED | Version history, diff view, rollback, run comparison |
| 21 | rag-knowledge-base.md | P0 | 5A | DONE | RAG Knowledge Base: 17th node type, full-stack (sidecar + Rust rag/ + executor + IPC + UI + templates + E2E). 160 tests. |
| 22 | loop-feedback.md | P0 | 5A | DONE | Loop & Exit nodes: iterative refinement, 3 exit conditions, 2 feedback modes, 2 templates. 193 tests. Peer reviewed (Gemini + Codex, 8 fixes). |
| 23 | scheduler-and-workflow-ux.md | P0 | 4C | DRAFT | Cron Trigger node, Workflow List UX upgrades, 3 demo templates |
| 24 | dual-mode-deployment.md | P1 | 5+ | PLANNED | Desktop + Server mode from same codebase. Core crate extraction, Axum HTTP, Docker, JWT auth. |

**Status key**: DONE | IN PROGRESS | PLANNED | BLOCKED | REFERENCE (non-implementable)

---

## Current Phase: 4 (Universal Automation Canvas)

**Goal**: Graduate node editor from AI workflow builder to universal automation platform. Data I/O nodes, control flow, engine refactoring, canvas UX.
**Status**: IN PROGRESS — 4A + 4B complete, 4C started (Live Workflow)
**Specs in scope**: `phase4-automation-canvas.md` (primary), `eip-data-io-nodes.md` (data I/O), `live-workflow.md` (4C), `node-editor.md` (reference)

### Phase 4A — Canvas + Node Types (DONE)
| Step | Status | Commit |
|------|--------|--------|
| Spec v1.2 (reviewed by Gemini + Codex) | DONE | — |
| Engine bug fixes (sourceHandle + clean_output) | DONE | ff2b271 |
| 4A.1 monolith split (NodeEditorPage → 16 modules) | DONE | bd6cbe1 |
| 4A.V visual overhaul (typed edges, inline editing, CSS) | DONE | 0924d2f |
| 4A.3-4A.8 new node types + executors | DONE | 2787a24 |
| Output truncation fix + vision pipeline | DONE | 4db3fd9 |
| Validation relaxation (file_read/file_write) | DONE | 4db3fd9 |
| Playwright E2E testing (15 tests, screenshots) | DONE | 378990e |

### Phase 4B — Data I/O + EIP Patterns (DONE)
| Step | Status | Description |
|------|--------|-------------|
| EIP spec written | DONE | `docs/specs/eip-data-io-nodes.md` — File Glob, Iterator, Aggregator, LLM Session, Streaming |
| **v0.1.0 tagged** | DONE | Phase 1-3 + 4A release tag (91007cc) |
| **Transform jsonpath + script modes** | DONE | RFC 9535 JSONPath + pipe expressions, 24 tests (b765061) |
| EIP spec peer review (Gemini + Codex) | DONE | 7+12 findings triaged, 8 accepted + fixed |
| 4B.1 File Glob node | DONE | glob executor + UI node, 8 tests (e352d0c) |
| **4B.2 Iterator + 4B.3 Aggregator** | DONE | Subgraph extraction, synthetic workflow execution per item, 3 aggregation strategies, 23 tests |
| **4B.4 LLM Session mode** | DONE | Stateful multi-turn conversations via sidecar session accumulation, 5 tests |
| **Codex review fixes** | DONE | UTF-8 safe truncation, File Glob containment, multi-aggregator detection, maxHistory clamp, cycle detection |
| **Agent edit mode** | DONE | Edit provider, model, prompt after agent creation (4f49165) |
| **Click-to-place nodes** | DONE | macOS WebKit drag-and-drop fix — click palette then click canvas (335f166) |
| **Custom node labels** | DONE | Double-click header to name any node (e.g. "LLM · Summarizer"), config panel field (11a166a) |
| **Toolbar + list UI polish** | DONE | Dividers, node count badge, hover card actions, btn-icon-sm utility (8f4906a) |
| **Node editor guide** | DONE | Step-by-step guide for all 16 nodes, 7 patterns, LLM picker (82795b8) |
| **Rename Node Editor → Workflows** | DONE | Sidebar label + Workflow icon + page title + command palette (18648b5) |

### Phase 4C — Streaming, Live Mode, UX (IN PROGRESS)
| Step | Status | Description |
|------|--------|-------------|
| **Live Workflow execution** | DONE | Continuous loop mode with cooperative cancellation, ephemeral execution (skip DB), LiveFeedPanel, Go Live/Stop toolbar, settings popover, 119 Rust tests passing |
| **Vision pipeline fix** | DONE | MCP client preserves image data (was dropping with placeholder), webcam template uses `webcam_capture` (raw frame), LLM vision prompt safety net for unresolved templates |
| **Vision OOM fix** | DONE | Image dedup + better prompt safety net — fixes 500 OOM on Qwen3-VL (6178cd9) |
| Webcam Monitor live demo | DONE | Verified webcam → Qwen3-VL pipeline working end-to-end |
| **Node variable interpolation fixes** | DONE | Shell Exec incoming JSON merge, resolve_template array indexing, Transform "Script"→"Expression" rename (81b42d9) |
| **Input node auto-resize textarea** | DONE | Auto-expanding textarea (1→5 lines), config panel Default Value field (81b42d9) |
| **User Templates (Save & Load)** | DONE | Save workflow as reusable template, filesystem-based `~/.ai-studio/templates/`, merged into Templates dropdown with badge + delete |
| **Streaming node output** | DONE | SSE token streaming: sidecar /chat/stream, Rust proxy_request_stream with batching, UI live preview with cursor (cd3b84d). All 6 providers: Ollama, OpenAI, Azure, Google Gemini, Anthropic, LocalOpenAI |
| **Rich Output wiring** | DONE | RichOutput component bug fixes (broken CopyButton, CSV export, compact prop). Wired into 5 spots: NodeShell canvas preview, Inspector event details, Sessions chat, Runs output (35988bb) |
| **Hybrid Intelligence template** | DONE | Ensemble synthesis: Qwen (engineer) + Gemini (architect) in parallel → synthesizer merges best of both. Template #12 (f6f5587) |
| **Smart Deployer template** | DONE | Natural language microservice deployment: File Read → LLM Plan Builder → Approval → Iterator → Shell Exec. Template #13 |
| **RAG Knowledge Base spec** | DONE | Full spec: Knowledge Base node, 2-tier design, binary index format, sidecar /embed, security, templates. Peer reviewed by Gemini + Codex, all 21 findings accepted and fixed. |
| **README overhaul** | DONE | Complete rewrite: 16 node types, 13 templates, 129 tests, 6 providers, capability-based positioning (no competitor names), correct shortcuts, RAG "Coming Next". Peer reviewed by Gemini + Codex (c7c835e) |
| **v0.1.2 tagged** | DONE | Streaming + Rich Output + RAG spec + README overhaul |
| **RAG Knowledge Base** | DONE | Full-stack: sidecar /embed, Rust rag/ (chunker+index+search+format, 31 tests), KnowledgeBaseExecutor, 4 IPC commands, UI node+config, 3 templates (#14-16), 2 E2E tests. 160 total tests. (09ca3d0) |
| **Loop & Feedback nodes** | DONE | Loop + Exit node types: 3 exit conditions (max_iterations, evaluator, stable_output), 2 feedback modes (replace, append), Router selectedBranch for evaluator mode, 2 templates (self-refine, agentic-search). 188 Rust tests, 8 E2E tests. |
| **Loop peer review fixes** | DONE | Gemini + Codex review: 8 fixes applied. Router value unwrap, branch-* backward compat, evaluator feedback fix, append array wrap, nesting errors, empty body warning, UI clamp. 193 tests. (9dba659) |
| **Webhook trigger** | DONE | HTTP webhook entry point, HMAC auth, rate limiting, response modes. 20th node type. |
| **Toolbar UX redesign** | DONE | Icon-driven toolbar per Gemini UX review (767a670) |
| **Email Send node** | DONE | SMTP integration via lettre crate: TLS/SSL/plain, template resolution, address validation, error→extra_outputs. 21st node type, "Communication" palette category. 229 tests. (6483d21) |
| **Cron Trigger node** | DONE | Time-based schedule automation. CronScheduler (1s tick loop in TriggerManager), arm/disarm IPC, validation (max 1, cron+webhook coexist). UI: CronTriggerNode, config panel (expression, presets, timezone, max concurrent, catch-up policy), toolbar arm/disarm. 22nd node type, Triggers category. 251 tests (+14 new). (9eb843d) |
| **Cron Trigger peer review** | DONE | Gemini (arch) + Codex (impl): 8 fixes — last_fired_minute init from DB, 5-field enforcement, list_triggers payload fix, dual-trigger toolbar split, maxConcurrent≥1, executor tests rewrite, DB error logging, 22 timezones. 253 tests. (a8649fe) |
| **Note node** | DONE | Documentation-only canvas node (23rd node type). StickyNote icon, text preview on canvas, full textarea in config panel. Utility category. Orphan warning suppressed in validation. 254 tests. (5621c26) |
| **Daily Meeting Digest template** | DONE | Cron (9 AM) → File Glob (transcripts) → LLM (summarize) → Email Send (digest). 19th bundled template. Includes Note node explaining setup. (5621c26) |
| **Tilde expansion fix** | DONE | `expand_tilde()` shared helper in file_read.rs — applied to File Read, File Glob, File Write executors. `~/path` now resolves correctly. (db16b37) |
| **Tool Picker dropdown** | DONE | Replace hardcoded text input with grouped `<select>` that discovers MCP tools via sidecar `GET /mcp/tools`. Grouped by server, shows description, custom fallback. ToolNode shows friendly name + server subtitle. (49be3ad) |
| **Edge Data Preview (X-Ray)** | DONE | X-Ray mode: toggle data preview badges on edges after workflow run. Hover tooltip, click popover with copy. Toolbar Scan button + X shortcut. (b31e610) |
| Container/group nodes | TODO | Visual grouping on canvas |

---

## Phase 3 (COMPLETE)

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
| Community templates | `phase-plan.md` | DONE (12 bundled + templates/README.md) |
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

### Phase 5+ Backlog (killer features — work top-down)

**Killer Features (specced):**
11. ~~A/B Eval Node~~ REMOVED — Multi-Model Compare template covers this adequately
12. **Step-Through Debugging** — Breakpoints, F10/F5, Edit & Continue for AI workflows. Spec: `step-through-debugging.md`. ~4 sessions.
13. ~~Edge Data Preview (X-Ray Mode)~~ DONE (b31e610) — Toggle to see data values on every edge after a run.
14. **Prompt Version Control** — Auto-versioning LLM prompts with diff, rollback, pin. Spec: `prompt-version-control.md`. ~2 sessions.
15. **Natural Language Canvas** — Chat input to modify workflow graphs via LLM. Spec: `natural-language-canvas.md`. ~5 sessions.
16. **AI Workflow Copilot** — Insight engine analyzing run history for optimizations. Spec: `ai-workflow-copilot.md`. ~8 sessions.

**Infrastructure:**
17. **Dual-Mode Deployment** — Desktop + Server mode from same Rust codebase. Spec: `dual-mode-deployment.md`. ~7 sessions.

**Future Nodes:**
18. **Time-Travel Debug** — Click any completed node → edit output → re-run from that point.
19. **Auto-Pipeline Generator** — English → AI generates graph JSON. Overlap with Natural Language Canvas.
20. **Guardrails Node** — PII detection, content filtering, hallucination check.
21. ~~RAG Knowledge Base~~ DONE (09ca3d0)
22. **EIP: Error Handler / Dead Letter** — Node-level `onError` output handle.
23. **EIP: Content Enricher** — Merge external data into message.
24. **EIP: Wire Tap** — Side-channel logging without affecting flow.
25. **EIP: Recipient List** — Dynamic runtime routing.
26. **SQL Query Node** — Postgres/MySQL/SQLite queries.
27. ~~Phase 4C~~ DONE — v0.2.0 tagged
28. ~~v0.2.0 tag~~ DONE (d5446d2)

**Moonshot Ideas (no specs yet):**
29. **Autonomous Agent Mode** — LLM dynamically picks which nodes to call, visualized on canvas in real-time. "ReAct on Canvas."
30. **Workflow as API** — One-click deploy workflow as REST endpoint. Local-first production serving.
31. **Workflow Recording** — Record manual process → AI generates workflow automatically.

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
- **Vision OOM**: When same image arrives on multiple LLM handles (input+prompt), dedup is needed. Also, stringified image JSON as prompt text (`{"encoding":"base64",...}`) bypasses the old `!contains(' ')` safety net — pretty-printed JSON has spaces. Fixed in 6178cd9.

---

## Last Session Notes

**Date**: 2026-02-28 (session 47)
**What happened**:
- **Edge Data Preview (X-Ray Mode)** (b31e610): First Phase 5 killer feature implemented in a single session. 7 files changed (2 new, 5 modified):
  - Rust: Removed `#[serde(skip_serializing)]` from `node_outputs` in `WorkflowRunResult` — now serialized to UI via IPC
  - TS types: Added `nodeOutputs?: Record<string, unknown>` to `WorkflowRunResult`
  - Store: `lastRunNodeOutputs`, `xrayEnabled`, `toggleXray()` — captures outputs after run, clears on reset
  - New `edgeDataUtils.ts`: `formatPreview()`, `resolveHandleValue()` (mirrors Rust `resolve_source_handle`), `formatFullPreview()`, `getDataTypeLabel()`
  - New `EdgeDataBadge.tsx`: Pill badge on edge midpoint, hover tooltip (500 chars), click popover (5000 chars) with copy button + data type indicator
  - `TypedEdge.tsx`: Added `EdgeLabelRenderer` with `EdgeDataBadge` at `labelX`/`labelY` from `getBezierPath`
  - `WorkflowCanvas.tsx`: Toolbar Scan button (cyan active state), `X` keyboard shortcut
  - 254 Rust tests pass, TS type check clean

**Previous session (46)**:
- RAG document extraction, corporate cleanup, v0.2.0 re-tag, 5 killer feature specs

**Previous sessions**:
- Session 45: Note node (23rd), Daily Meeting Digest template, tilde expansion fix, Tool Picker dropdown
- Session 44: Cron Trigger peer review — 8 fixes, 253 tests
- Sessions 1-43: See git log

**Next session should**:
1. **Step-Through Debugging** — highest differentiation killer feature (~4 sessions)
2. Or **Prompt Version Control** (~2 sessions) — auto-versioning LLM prompts
3. Or add tests to 0-test nodes (Shell Exec, HTTP Request, Router)
4. Or start **Dual-Mode Deployment** (desktop + server from same codebase)
