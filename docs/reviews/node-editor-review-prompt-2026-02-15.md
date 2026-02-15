# Peer Review: Node Editor Architecture

**Date**: 2026-02-15
**Project**: AI Studio (open-source IDE for AI agents)
**Reviewer**: Gemini 3 Pro
**Review type**: Architecture

---

## Instructions for Reviewer

You are reviewing code in the workspace at `/home/amit/projects/myws/ws01/ai-studio-template`.
Read the files listed below, then provide your review in the output format specified at the bottom.

## Context

AI Studio is a desktop-native IDE for AI agents (Tauri 2 + React 19 + Python FastAPI sidecar). Phase 1 (core loop) and Phase 2 (polish) are complete. The Node Editor is the flagship Phase 3 feature — a visual pipeline builder described as "Unreal Blueprints for AI agents." This spec defines 8 node types, a DAG execution engine in Rust, React Flow for rendering, and integration with the existing Inspector, Runs, and MCP tool systems. We want the architecture reviewed before writing any code.

## Scope

Review the node editor architecture spec for correctness, completeness, and implementation feasibility. Focus on how it integrates with the existing 3-layer architecture (UI → Tauri/Rust → Python sidecar).

## Files to Read

Read these files in this order:

1. `docs/specs/node-editor.md` — **THE SPEC BEING REVIEWED**. Full node editor architecture: 8 node types, execution model, persistence, events, integration points, implementation plan.
2. `docs/specs/architecture.md` — Existing 3-layer architecture. Needed to understand how the node editor fits in.
3. `docs/specs/event-system.md` — Existing event system. The node editor adds new `workflow.*` event types that must integrate with this.
4. `docs/specs/data-model.md` — Existing SQLite schema. The node editor adds a `workflows` table (schema v5).
5. `apps/desktop/src-tauri/src/db.rs` — Current database migration code (v1-v4). The spec adds v5.
6. `apps/desktop/src-tauri/src/commands.rs` — Current Tauri IPC commands. The spec adds ~10 new workflow commands.
7. `apps/sidecar/server.py` — Current Python sidecar. The execution engine reuses existing `/chat` and `/tools/call` endpoints.
8. `docs/specs/mcp-integration.md` — MCP tool system. Tool nodes in the editor use MCP tools.

## What to Look For

1. **DAG Execution Engine**: The spec puts the DAG walker in Rust (topological sort). Is this the right layer? Should execution orchestration live in the sidecar (Python) instead? What are the tradeoffs?

2. **Sidecar Statelessness**: We recently fixed a bug where branched sessions lost LLM context because the sidecar held state in-memory. The node editor will make many sequential calls to `/chat` and `/tools/call`. Is the proposed execution flow correctly stateless, or will we hit the same class of bugs?

3. **Schema v5 Design**: The `workflows` table stores the full React Flow graph as a JSON blob. Is this the right approach vs. normalized tables for nodes/edges? What about query performance for large workflows?

4. **Event Integration**: The spec defines `workflow.node.started`, `workflow.node.completed`, `workflow.node.error`, etc. Do these integrate cleanly with the existing event system (WebSocket bridge, Inspector, cost calculation)?

5. **Router Node Complexity**: The Router node has two modes (LLM classify + pattern match). The LLM classify mode calls a model just for routing. Is this over-engineered for Phase 3A? Should it be deferred?

6. **Missing Pieces**: What's NOT in the spec that should be? Error recovery, retry logic, parallel execution, rate limiting, large workflow performance, edge cases with cycles or disconnected subgraphs?

## Output Format

Provide your review as a markdown file saved to:
**`docs/reviews/node-editor-review-2026-02-15.md`**

Use this structure:

### Header
```
# Node Editor Architecture Review
**Date**: 2026-02-15
**Reviewer**: Gemini 3 Pro
**Status**: Draft
```

### Findings Table
| Check | Priority | Verdict | Findings |
|-------|----------|---------|----------|
| {area} | {HIGH/MED/LOW} | {PASS/FAIL/WARN} | {1-2 sentence finding} |

### Actionable Checklist
- [ ] {Action item 1}
- [ ] {Action item 2}

### Notes (optional)
Any architecture recommendations, praise, or broader observations.
