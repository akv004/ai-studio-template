## Phase 4 Architecture Review

**Reviewer**: Antigravity (Gemini 3 Pro)
**Date**: 2026-02-18
**Status**: TRIAGED — All 5 findings accepted, spec updated

### Checks

| # | Area | Check | Status | Notes |
|---|------|-------|--------|-------|
| 1 | Container model | parentId vs scopeId vs handles | **WARN** | `parentId` is cleaner for UI but requires significant engine refactoring from current flat-topology model. |
| 2 | Engine refactor | execute_subgraph safety | **WARN** | Recursive execution needs strict depth limits to prevent stack overflow/infinite loops. |
| 3 | Handle Type System | Coercion matrix | **PASS** | `number` vs `float` distinction is valid for UI validation even if storage is identical. |
| 4 | Security | Shell/IO safeguards | **PASS** | "Ask" by default is safe, but relies on a robust UI implementation of the approval dialog. |
| 5 | EIP Mapping | Coverage completeness | **PASS** | Good coverage of core integration patterns. |
| 6 | Subworkflow | Circular dependency check | **FAIL** | Current `engine.rs` and `ExecutionContext` lack `visited_workflows` tracking. |

### Findings

- [x] **F1**: **Engine Topology is Flat** — [severity: HIGH]
  - **Description**: The current `execute_workflow` constructs a flat dependency graph using Kahn's algorithm on *all* nodes. It does not respect `parentId`. If a Loop node contains 5 children, the flat sort might try to execute the children before the Loop node itself, or interleave them incorrectly.
  - **Impact**: Container nodes (Loop, ErrorHandler) will not function as true scopes.
  - **Recommendation**: Refactor `engine.rs` to:
    1. Filter the main topological sort to only include top-level nodes (nodes with no `parentId`).
    2. Implement `execute_subgraph(nodes, context)` which is called by the Loop executor for its children.
    3. Ensure `parentId` is indexed for fast lookup of children.

- [x] **F2**: **Missing Circular Dependency Protection in Subworkflows** — [severity: MEDIUM]
  - **Description**: The Spec 3.6 notes the need for `visited_workflows` in `ExecutionContext`, but the valid current code in `executors/mod.rs` does not have this field.
  - **Impact**: A user could create A -> B -> A, causing an infinite recursion that crashes the desktop app stack.
  - **Recommendation**: Add `pub visited_workflows: &'a HashSet<String>` to `ExecutionContext` struct in `executors/mod.rs` and plumbing in `engine.rs`.

- [x] **F3**: **Loop Variable Scoping** — [severity: MEDIUM]
  - **Description**: The spec mentions `{{node_inside_loop.output}}`. If a node *outside* the loop tries to access a node *inside* the loop, which iteration's value does it get? The last one? An array of all?
  - **Impact**: Ambiguous behavior for downstream nodes.
  - **Recommendation**: Enforce "Scope Isolation". Nodes outside the loop cannot access nodes inside the loop directly. They must use the Loop Node's aggregated output (`results` handle). This matches standard block-scoping rules in programming.

- [x] **F4**: **Security: Shell Execution Environment** — [severity: LOW]
  - **Description**: `ShellExecNode` runs commands in the parent process environment (potentially inheriting API keys/env vars).
  - **Impact**: A malicious template could print environment variables to stdout/logs.
  - **Recommendation**: Explicitly clear environment variables in `tokio::process::Command` and only inject the allowed `envVars` from config. Use `env_clear()` before `envs()`.

- [x] **F5**: **Missing `rows` Type Validation** — [severity: LOW]
  - **Description**: The `rows` handle type is visual-only.
  - **Impact**: No backend validation that `rows` input is actually an array of objects.
  - **Recommendation**: Add a helper `ensure_rows(val: &Value) -> Result<Vec<Value>>` in `types.rs` to fail fast if incorrect data is passed.

### Summary
The Phase 4 Spec is ambitious and moves AI Studio towards a professional automation tool. The visual container model (`parentId`) is the superior UX choice but mandates the heaviest backend refactoring (breaking the flat DAG assumption). The security model is adequate for a v1 desktop app but should tighten environment isolation.

**Approval Status**: **APPROVED with Required Changes (F1, F2)**
