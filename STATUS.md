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
| 14 | killer-features.md | P0 | 5 | PLANNED | A/B Eval, Time-Travel, Auto-Pipeline, Guardrails, RAG, SQL |
| 15 | connections-manager.md | P0 | 5B | PLANNED | Unified credential store, encrypted at rest, DB/HTTP/SMTP/Webhook |
| 16 | triggers-scheduling.md | P0 | 5B | PLANNED | Webhook, cron, file watch, event triggers |
| 17 | streaming-output.md | P1 | 5A | DONE | SSE token streaming — all 6 providers (Ollama, OpenAI, Azure, Google, Anthropic, Local) |
| 18 | batch-runs.md | P1 | 5B | PLANNED | Dataset import, batch execution, progress dashboard |
| 19 | rich-output.md | P1 | 5A | IN PROGRESS | Markdown, tables, JSON tree/table, code blocks — wired into 5 spots. Charts/images deferred. |
| 20 | workflow-versioning.md | P2 | 5B | PLANNED | Version history, diff view, rollback, run comparison |
| 21 | rag-knowledge-base.md | P0 | 5A | DONE | RAG Knowledge Base: 17th node type, full-stack (sidecar + Rust rag/ + executor + IPC + UI + templates + E2E). 160 tests. |
| 22 | loop-feedback.md | P0 | 5A | DONE | Loop & Exit nodes: iterative refinement, 3 exit conditions, 2 feedback modes, 2 templates. 193 tests. Peer reviewed (Gemini + Codex, 8 fixes). |

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

11. **A/B Eval Node** — Split input to multiple LLMs in parallel, score outputs side-by-side. Built-in eval grid: latency, cost, quality rating. "Which model is best for this task" in one click. Easiest to build (parallel LLM calls exist), highest demo impact.
12. **Time-Travel Debug** — Click any completed node → edit its output → re-run from that point forward. Don't restart the whole workflow. Inspector + node states already exist — this is an evolution. Unique differentiator, no competitor has this.
13. **Auto-Pipeline Generator** — Describe a workflow in English → AI generates the graph JSON → canvas fills itself. Meta: use AI to build AI pipelines. The "wow" demo moment for Show HN.
14. **Guardrails Node** — Built-in safety: PII detection, content filtering, hallucination check, schema enforcement. Drop anywhere in pipeline. Enterprise magnet, huge credibility for production use.
15. ~~RAG Knowledge Base~~ DONE (09ca3d0) — 17th node type, full-stack implementation
16. **EIP: Error Handler / Dead Letter** — Route errors to a fallback path instead of stopping the workflow. Node-level `onError` output handle that connects to recovery logic. Table stakes for production automation.
17. **EIP: Content Enricher** — Merge data from an external source (DB, API) into the current message. Ties into SQL Query node — "enrich this record with customer data from the DB."
18. **EIP: Wire Tap** — Copy node output to a side channel (log, file, webhook) without affecting the main flow. Non-blocking audit/debugging.
19. **EIP: Recipient List** — Dynamic routing to multiple destinations based on message content. Router is static branches; this evaluates at runtime.
20. **SQL Query Node** — Connect to Postgres/MySQL/SQLite, run queries, return rows. Settings gets a Connections tab. Enables natural language → SQL → results → LLM summary pipelines.
21. Phase 4C: Streaming, containers, UX polish
22. v0.2.0 tag for Phase 4B completion

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

**Date**: 2026-02-25 (session 42)
**What happened**:
- **Email Send node — full implementation** (6483d21):
  - Rust `EmailSendExecutor` using `lettre` crate: async SMTP with TLS/SSL/plain modes
  - Template resolution on all 6 email fields (to, subject, body, cc, bcc, replyTo)
  - Address validation via lettre's RFC 5321 `Address::from_str`
  - Error→extra_outputs pattern (node does NOT stop workflow on failure)
  - UI: `EmailSendNode.tsx` with 6 input + 2 output handles, SMTP/From preview
  - New "Communication" palette category (Mail icon)
  - Config panel: SMTP server section + email section with body textarea
  - 8 unit tests (parse addresses, validation, error shape, success shape, body type)
  - **229 total Rust tests** passing (193 existing + 36 new across recent sessions)
  - This is the **21st node type** and first in the Communication category

**Previous session (41)**:
- Webhook Chat API template + toolbar UX redesign per Gemini review

**Previous session (40)**:
- Loop & Feedback peer review — Gemini + Codex, 8 fixes, 193 tests

**Previous session (39)**:
- **Loop & Feedback nodes — full implementation** (4 commits):
  - `exit.rs`: pass-through stub (same pattern as Aggregator)
  - `loop_node.rs` (~400 lines): subgraph extraction, synthetic graph builder, levenshtein similarity, 3 exit conditions (max_iterations, evaluator, stable_output), 2 feedback modes (replace, append), 20 unit tests
  - Router output now includes `selectedBranch` for evaluator mode detection
  - Engine `extract_primary_text` adds "value" key for Router output unwrapping
  - Validation: Loop↔Exit pairing warnings, nesting warnings (loop+loop, loop+iterator), 5 new tests
  - UI: LoopNode (RefreshCw icon, 3 outputs) + ExitNode (LogOut icon, pass-through)
  - NodeConfigPanel: Loop config section with conditional stabilityThreshold
  - 2 templates: Self-Refine (#15), Agentic Search (#16) — 16 bundled total
  - 2 Playwright E2E tests (canvas render + palette presence)
  - **188 total Rust tests** (162 existing + 26 new), **8 E2E tests** passing
  - This is the **18th and 19th node types** (loop + exit)

**Previous session (38)**:
- **RAG Knowledge Base — full-stack implementation** (09ca3d0):
  - Sidecar: `POST /embed` endpoint with `EmbeddingClient` (Azure OpenAI + OpenAI-compatible), batching, token validation, retry
  - Rust `rag/` module: 4 chunking strategies, binary vector index (memmap2), dot-product search with BinaryHeap top-K, atomic writes (fs2), citation formatting. 31 unit tests.
  - `KnowledgeBaseExecutor`: auto-index on stale, streaming progress events, path security
  - 4 IPC commands: `index_folder`, `search_index`, `get_index_stats`, `delete_index`
  - UI: `KnowledgeBaseNode` component (BookOpen icon, 3 handles), full config panel
  - 3 templates: Knowledge Q&A (#14), Smart Deployer + RAG (#15), Codebase Explorer (#16)
  - 2 Playwright E2E tests (canvas render + palette presence), screenshots captured
  - **160 total Rust tests** (129 existing + 31 new), **10 E2E tests** passing
  - This is the **17th node type** and completes the RAG spec

**Previous sessions**:
- Sessions 1-17: See git log for full history
- Session 18: Rust refactoring, budget enforcement, plugin foundation, README update
- Session 19: One-click installers, plugin subprocess lifecycle, open-source infra
- Session 20: Docker cleanup, template gallery (10 total), launch prep
- Session 21: Phase 4 spec v1.1 written (10+ nodes, EIP patterns, Unreal architecture, containers)
- Session 22: Phase 4 spec v1.2, peer reviews triaged, engine bugs fixed, 4A.1 monolith split
- Session 23: Phase 4A.V visual overhaul (TypedEdge, TypedConnectionLine, inline editing, CSS polish)
- Session 24: Output truncation fix, vision pipeline, EIP spec, Playwright E2E (15 tests)
- Session 25: v0.1.0 tag, Transform jsonpath+script, File Glob, Iterator+Aggregator
- Session 26: LLM Session mode (4B.4), 5 tests, Playwright verification
- Session 27: EIP peer reviews (Gemini+Codex), 5 code fixes (UTF-8, containment, cycle detection)
- Session 28: Agent edit mode, click-to-place nodes, custom node labels
- Session 29: Toolbar polish, node editor guide, Phase 5+ backlog, v0.1.1 tag, rename → Workflows
- Session 30: Live Workflow execution, vision pipeline fix, multi-provider vision support
- Session 31: Vision OOM fix (image dedup + prompt safety net for Qwen3-VL)
- Session 32: Node variable interpolation fixes, Input node textarea, repo cleanup
- Session 33: User templates (Save & Load)
- Session 34: Competitive gap analysis + 6 Phase 5 specs
- Session 35: SSE token streaming (Ollama), Playwright test fix
- Session 36: Streaming for all remaining providers (OpenAI, Azure, Google, Anthropic, Local)
- Session 37: Rich Output wiring + Hybrid Intelligence template (ensemble synthesis)
- Session 38: RAG Knowledge Base full-stack implementation (17th node type, 31 tests, 3 templates)
- Session 39: Loop & Feedback nodes (18th+19th node types, 26 new tests, 2 templates)
- Session 40: Loop & Feedback peer review — Gemini (architecture) + Codex (implementation), 8 fixes applied, 193 tests

**Next session should**:
1. **Email Send E2E test** + Mailpit integration test
2. Consider **peer review** for email_send (Gemini architecture + Codex implementation)
3. Consider v0.2.0 tag for Phase 4 completion
4. Or start **A/B Eval Node** (Phase 5 #11 — highest demo impact, parallel LLM calls already exist)
5. Or start **connections-manager** (P0 — SMTP creds currently in node config, needs encrypted store)
