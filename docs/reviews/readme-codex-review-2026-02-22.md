# README Review — Technical Accuracy
**Date**: 2026-02-22
**Reviewer**: GPT-5.2 (Codex CLI)
**Status**: RESOLVED — all findings accepted, README rewritten

### Findings Table
| Check | Priority | Verdict | Findings |
|-------|----------|---------|----------|
| Quick Start commands | HIGH | PASS | `npm install`, `npm run dev`, and `npm run tauri:dev` exist in `package.json`; sidecar can run via `cd apps/sidecar && python server.py` and Docker via `docker compose up`. |
| Node type count | HIGH | FAIL | README claims “8 Node Types”, but `apps/desktop/src-tauri/src/workflow/executors/` contains 16 actual executors (Input/Output/LLM/Tool/Router/Approval/Transform/Subworkflow plus HTTP Request, File Read/Write/Glob, Shell Exec, Validator, Iterator, Aggregator). |
| DAG execution concurrency claim | HIGH | FAIL | README claims “parallel branches via `tokio::join_all`”, but the workflow engine is explicitly “sequential node execution” (topological sort + for-loop) and there is no `join_all` usage. |
| Bundled template count | HIGH | FAIL | README claims “10 Bundled Templates”, but `apps/desktop/src-tauri/src/commands/templates.rs` defines 13 bundled templates (adds Webcam Monitor, Hybrid Intelligence, Smart Deployer). |
| LLM provider list/count | MED | WARN | README lists 5 providers (omits Local/OpenAI-compatible). Sidecar provider set includes `LocalOpenAIProvider` in addition to Ollama/Anthropic/OpenAI/Google/Azure. |
| MCP tools accuracy | HIGH | WARN | README says “Built-in tools: shell, filesystem, browser”, but `/tools/execute` (used by workflow Tool nodes) resolves only the MCP registry tools (shell + read/write/list). Browser exists as legacy HTTP endpoints but is not registered as an MCP tool. |
| Project structure numbers | HIGH | FAIL | README correctly says 13 command modules and `routing.rs` has 14 tests, but it incorrectly says “7 node executors” and “12 design specifications”; both counts are materially higher now. |
| Design Specs table | MED | FAIL | README lists 12 specs, but `docs/specs/` currently contains 30+ spec docs (e.g., streaming output, rich output, EIP data I/O nodes, live workflow, RAG knowledge base). |
| Roadmap phase status | HIGH | FAIL | README shows Phase 3 “In progress” and Phase 4 “Planned”, but `STATUS.md` states Phase 3 is complete and Phase 4 is in progress (4A + 4B done, 4C in progress). |
| “What’s Built” accuracy | MED | WARN | Several bullets are outdated (node type count, provider count, template count, test count) and the list omits major shipped features (SSE streaming, live workflow mode, vision pipeline, user templates, iterator/aggregator patterns). |
| Keyboard shortcuts | HIGH | FAIL | README shortcuts don’t match actual bindings: Inspector is `⌘4` (not `⌘I`), send message is `Enter` (not `⌘Enter`), WorkflowCanvas uses `Ctrl+D` for Duplicate (not Delete) and `Del` for Delete. |
| Competitor naming | MED | WARN | README comparison table explicitly names competitors; prompt requirement is capability-based language only. |

### Actionable Checklist
- [ ] Update Node Editor section to “Workflows” and fix node-type count/list to the 16 current executors.
- [ ] Fix DAG engine description: keep “topological sort”, remove “parallel branches via `tokio::join_all`” (or implement parallel execution if that’s truly intended).
- [ ] Update bundled templates count and list to match `apps/desktop/src-tauri/src/commands/templates.rs` (13 bundled + user templates).
- [ ] Update provider list/count everywhere (Tech Stack + “What’s Built”) to include Local/OpenAI-compatible, and reconcile “5 vs 6 providers”.
- [ ] Clarify MCP/tooling: either register browser as an MCP tool (so Tool nodes can call it) or remove “browser” from “built-in tools” claims.
- [ ] Replace “12 design specifications” and the 12-row Design Specs table with an auto-maintained list (or link to a corrected `docs/specs/README.md` index).
- [ ] Fix Roadmap to match `STATUS.md` (Phase 3 complete; Phase 4 in progress; optionally call out 4A/4B/4C).
- [ ] Fix Keyboard Shortcuts table to match actual bindings (⌘1-5, ⌘,, ⌘K, ⌘N, ⌘⇧N, Enter-to-send, Ctrl+D duplicate, Del delete).
- [ ] Rewrite the comparison table without naming competitors (use categories like “chat-only apps”, “code IDE copilots”, “flow-based workflow builders”, etc.).
- [ ] Add missing Quick Start prerequisites that commonly break first-run (Tauri OS deps, recommended Python venv workflow).

### Notes (optional)
- Node types + DAG engine claims are in `README.md:54`-`README.md:63`; executor source of truth is `apps/desktop/src-tauri/src/workflow/executors/` (16 node types).
- Templates claim is in `README.md:61`; source of truth is `apps/desktop/src-tauri/src/commands/templates.rs:33` (13 bundled templates).
- Roadmap mismatch is in `README.md:324`; source of truth is `STATUS.md:1`.
- Project structure count mismatches are in `README.md:280`-`README.md:299`; command modules count is correct (`apps/desktop/src-tauri/src/commands/` has 13), but executor/spec counts are not.
- Keyboard shortcuts mismatch is in `README.md:355`-`README.md:369`; bindings are defined in `apps/ui/src/commands/index.ts:1` and WorkflowCanvas context menu in `apps/ui/src/app/pages/workflow/WorkflowCanvas.tsx:909`.
