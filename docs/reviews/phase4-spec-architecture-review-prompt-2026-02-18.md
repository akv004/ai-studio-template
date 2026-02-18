# Phase 4 Architecture Review — Prompt for Gemini 3 Pro

> **Recommended reviewer**: Antigravity (Gemini 3 Pro) — architecture-level review
> **Date**: 2026-02-18
> **Topic**: Phase 4 Universal Automation Canvas specification

## What to Review

AI Studio's Node Editor is expanding from 8 AI workflow node types to 20+ universal automation nodes (HTTP, file I/O, shell exec, loops, merges, error handling, code execution). This is the most ambitious expansion since the project started. The spec needs architecture-level validation before implementation.

## Files to Read

1. **The Phase 4 spec** (PRIMARY): `docs/specs/phase4-automation-canvas.md`
2. **The Phase 3 spec** (REFERENCE — what's already built): `docs/specs/node-editor.md`
3. **Current engine** (REFERENCE — what we're refactoring): `apps/desktop/src-tauri/src/workflow/engine.rs`
4. **Current executors** (REFERENCE — existing pattern): `apps/desktop/src-tauri/src/workflow/executors/mod.rs`
5. **Current executor types**: `apps/desktop/src-tauri/src/workflow/types.rs`
6. **Design references**: `docs/design-references/node-editor/README.md`

## What to Look For

### 1. Container/Scope Architecture (CRITICAL)

The spec proposes Loop and Error Handler as **visual container nodes** (MuleSoft-style scopes) instead of standalone nodes with handle connections. This is the biggest architecture decision in the spec.

- Is `parentId`-based child node identification the right approach?
- How does it interact with React Flow's group node behavior?
- What happens with edges that cross container boundaries (node inside loop connected to node outside)?
- Does the engine correctly handle nested containers (loop inside error handler)?
- Should containers use `parentId` or a separate `scopeId` field to avoid conflicting with React Flow's native group behavior?
- Alternative: keep body_in/body_out handles (simpler, no React Flow group complexity). Which is better?

### 2. Engine Refactoring: `execute_subgraph()`

The spec extracts a subgraph executor from the main engine. This affects the core execution path.

- Is the proposed signature sufficient for both Loop and Error Handler use cases?
- How does the main topological sort interact with container nodes? (Spec says: skip child nodes in main sort, let container handle them.)
- What about node outputs from inside a container that are referenced by nodes outside? (Template resolution: `{{node_inside_loop.output}}` — which iteration's output?)
- Can this cause deadlocks or infinite recursion? What safeguards exist?

### 3. EIP Pattern Mapping

The spec maps 15+ Enterprise Integration Patterns to our node types.

- Are any critical EIP patterns missing from the "built-in" list that should NOT be deferred to plugins?
- Is the Merge node's quorum mode correctly specified? (Note: our engine is sequential via topo sort, not parallel-first-arrival.)
- Is the Error Handler's retry model correct? (Retry the entire subgraph, not individual nodes.)

### 4. Handle Type System

3 new types: number, rows, binary. Extended coercion matrix.

- Is `number` (integer) distinct enough from `float` to warrant a separate type? Rust has i64 vs f64, but at the JSON level both are `Number`.
- Is the coercion matrix complete? Any missing conversions?
- The `rows` type (array of objects) — should this just be `json` with a convention, or does it need a distinct type for validation?

### 5. Security Model

Shell Exec, File I/O, HTTP Request, and Code nodes all interact with the system.

- Is default approval="ask" for Shell Exec sufficient?
- File operations scoped to Tauri FS scope — is this enough, or do we need additional path validation?
- HTTP Request: auth tokens in config — security risk if graph JSON is shared?
- Code node: subprocess with minimal env — what about filesystem access?

### 6. Canvas UX

Comment boxes, reroute nodes, graph search, collapsed graphs.

- Are comment boxes (annotation nodes in graph JSON) the right abstraction?
- How should container nodes (loop/error_handler) differ visually from comment boxes?
- Any React Flow-specific gotchas with group nodes and custom node types?

## Expected Output Format

```markdown
## Phase 4 Architecture Review

**Reviewer**: [Your name/model]
**Date**: [Date]
**Status**: Draft

### Checks

| # | Area | Check | Status | Notes |
|---|------|-------|--------|-------|
| 1 | Container model | parentId vs scopeId vs handles | PASS/WARN/FAIL | ... |
| 2 | Engine refactor | execute_subgraph safety | PASS/WARN/FAIL | ... |
| ... | ... | ... | ... | ... |

### Findings

- [ ] **F1**: [Finding title] — [severity: HIGH/MEDIUM/LOW]
  - Description: ...
  - Impact: ...
  - Recommendation: ...

- [ ] **F2**: ...
```
