## Phase 4 Combined Review

**Reviewer**: Codex (GPT-5.3)
**Date**: 2026-02-18
**Status**: RESOLVED (triaged 2026-02-18, session 24)
**Source Reviews**:
- `docs/reviews/phase4-spec-implementation-review-2026-02-18.md`
- `docs/reviews/phase4-node-field-model-review-2026-02-18.md`
- `docs/reviews/phase4-spec-architecture-review-2026-02-18.md`

### Executive Verdict

Phase 4 direction is correct, but execution is not stable enough to safely scale node count yet.

Current state:
- Good architecture intent (container scopes, typed handles, richer node library)
- Incomplete runtime contracts (handle routing, field-model coherence, container execution boundaries)
- UX not yet at Maya/Blender/Unreal workflow speed

Recommendation:
- Continue Phase 4, but prioritize core engine/contract fixes before adding many more node types.

### Findings

- [x] **CF1**: Edge routing ignores `sourceHandle` at runtime - **severity: HIGH**
  - **ALREADY FIXED** in ff2b271 (pre-Phase 4 engine fixes). sourceHandle now resolves from handle-keyed output map.

- [x] **CF2**: Engine output normalization drops structured output data - **severity: HIGH**
  - **ALREADY FIXED** in ff2b271. clean_output now preserves full structured data for routing; preview computed separately for events.

- [ ] **CF3**: Container node model requires deeper engine and validation refactor - **severity: HIGH**
  - **Deferred to 4B** (Session 5). Already spec'd as `execute_subgraph()` extraction. Requires dedicated session for careful engine refactoring.

- [x] **CF4**: Field model has multiple UI-only or partially wired fields - **severity: HIGH**
  - **Accepted**. Addressing incrementally as each node type gets its executor in 4A.3-4A.8. Each new executor will wire all config fields.

- [x] **CF5**: Subworkflow is not yet a full reusable module path - **severity: HIGH**
  - **Accepted**. Planned as 4A.3 — workflow picker, workflowId binding, executor registration, cycle checks.

- [x] **CF6**: Type fields and socket types are disconnected - **severity: MEDIUM**
  - **Accepted**. Will wire `dataType`/`format` to handle CSS classes during Input/Output node rework.

- [ ] **CF7**: Router branch handles are index-coupled - **severity: MEDIUM**
  - **Deferred to 4C**. Low risk — branch reordering is rare in practice. Stable IDs are a nice-to-have.

- [x] **CF8**: File I/O security assumptions are too weak for Rust executors - **severity: HIGH**
  - **Accepted**. Will implement allowlist + denylist + canonicalization during 4A.5.

- [x] **CF9**: HTTP Request security and secret storage contract needs hardening - **severity: HIGH**
  - **Accepted**. Will implement SSRF protection + authTokenSettingsKey during 4A.4.

- [x] **CF10**: Shell Exec portability and cleanup details are underdefined - **severity: HIGH**
  - **Accepted**. Will implement with tokio process features, env_clear, setsid during 4A.6.

- [ ] **CF11**: Code node safety claims exceed actual sandbox guarantees - **severity: HIGH**
  - **Accepted**. Deferred to 4B naturally. Will document limitations honestly as spec already does.

- [x] **CF12**: Canvas UX is not yet as efficient as Maya/Blender/Unreal - **severity: MEDIUM**
  - **Accepted**, ongoing improvement. Partially addressed in 4A.V (visual overhaul). Continues in 4B-4C.
