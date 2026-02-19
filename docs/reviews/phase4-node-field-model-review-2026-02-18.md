## Node Field Model Review

**Reviewer**: Codex (GPT-5.3)
**Date**: 2026-02-18
**Status**: RESOLVED (triaged 2026-02-18, session 24)

### Summary

The overall node set is directionally correct, but the current field model is not yet coherent across UI, runtime, and execution semantics.
Right now several fields are UI-only, several handles are cosmetic-only, and some critical fields are missing where they are needed.

### Findings

- [x] **F1**: Multiple node fields are not consumed by runtime — **severity: HIGH**
  - **Accepted**. Addressing incrementally during 4A.3-4A.8 as executors are built/updated. Each field will be wired or hidden.

- [x] **F2**: Output handles do not align with executor outputs — **severity: HIGH**
  - **ALREADY FIXED** in ff2b271. sourceHandle resolution now uses handle-keyed output map.

- [x] **F3**: Subworkflow configuration is incomplete in UI and unsupported in runtime — **severity: HIGH**
  - **Accepted**. Planned as 4A.3 — workflow picker, workflowId binding, executor registration.

- [x] **F4**: Input/output typing fields are disconnected from socket typing — **severity: MEDIUM**
  - **Accepted**. Will wire dataType/format to handle CSS classes during implementation.

- [x] **F5**: Input default exists in runtime path but has no clear editor control — **severity: MEDIUM**
  - **Accepted**. Will add explicit Default Value editor for Input nodes.

- [x] **F6**: Enum contracts are inconsistent (`boolean` vs `bool`, `file` type without semantics) — **severity: MEDIUM**
  - **Accepted**, low priority. Will normalize type vocabulary during implementation.

- [ ] **F7**: Router branch field model is index-coupled and brittle during edits — **severity: LOW**
  - **Deferred to 4C**. Same as CF7. Low risk for current usage patterns.

- [x] **F8**: Canvas nodes carry too many editable fields compared to Unreal/Blender ergonomics — **severity: LOW**
  - **Accepted**, ongoing. Partially addressed in 4A.V. Will continue moving advanced fields to side panel.
