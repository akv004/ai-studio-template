# Architecture Review - 2026-02-18

**Reviewer**: Codex (GPT-5)
**Status**: Triaged
**Scope**: Current platform architecture and readiness for generalized automation (not a single-use GitHub workflow).
**Triaged by**: Claude Opus 4.6 (2026-02-17)

## Findings (ordered by severity)

### H1. Router node data contract mismatch can crash workflow execution

- UI stores router branches as `string[]` (for example `["true","false"]`): `apps/ui/src/app/pages/NodeEditorPage.tsx:221`, `apps/ui/src/app/pages/NodeEditorPage.tsx:330`, `apps/ui/src/app/pages/NodeEditorPage.tsx:487`.
- Runtime expects branch objects with `.name`: `apps/desktop/src-tauri/src/commands.rs:3105`.
- Fallback indexes first branch name without guarding empty `branch_names`: `apps/desktop/src-tauri/src/commands.rs:3155`.

**Impact**: Router nodes can fail/panic despite valid-looking UI config, blocking a core "platform logic" primitive.

- [ ] **Accept — fix in 3C**: Fix Rust executor to handle `string[]` branches directly. Add empty guard on `branch_names`.

### H2. Tool node execution bypasses approval engine and weakens security/audit model

- Workflow tool nodes call `/tools/execute` directly: `apps/desktop/src-tauri/src/commands.rs:3203`.
- Sidecar endpoint executes handler directly without Tauri approval check path: `apps/sidecar/server.py:508`, `apps/sidecar/server.py:514`.

**Impact**: Workflow tool calls do not follow the same approval guarantees as interactive chat, which conflicts with the product's safety posture and makes unattended automation risky.

- [ ] **Accept — fix in 3C**: Add approval check to tool node executor when node config has `approval: "ask"`. Route through same ApprovalManager used by approval nodes.

### M1. Workflow approval preview data is dropped in UI due to payload key mismatch

- Backend emits `dataPreview`: `apps/desktop/src-tauri/src/commands.rs:3248`.
- UI listens for `data` and renders `approvalRequest.data`: `apps/ui/src/app/pages/NodeEditorPage.tsx:768`, `apps/ui/src/app/pages/NodeEditorPage.tsx:1223`.

**Impact**: Human-in-the-loop approval is less trustworthy because users cannot see the previewed payload they are approving.

- [ ] **Accept — fix in 3C**: Align key names — change UI to read `dataPreview` from event payload.

### M2. Event envelope inconsistency between workflow events and global event pipeline

- Workflow executor emits partial `agent_event` payloads (`type`, `session_id`, `payload`) without full envelope fields: `apps/desktop/src-tauri/src/commands.rs:2731`, `apps/desktop/src-tauri/src/commands.rs:2824`.
- Global listener expects full envelope (`event_id`, `ts`, `source`, `seq`, `cost_usd`): `apps/ui/src/App.tsx:72`.
- Store dedupes using `eventId`: `apps/ui/src/state/store.ts:349`.

**Impact**: Live workflow events can be dropped or malformed in shared inspector/event state, reducing observability consistency.

- [ ] **Accept — fix in 3C**: Add `event_id` (uuid), `ts` (ISO), `source: "workflow"`, `seq`, `cost_usd: null` to all workflow event emissions.

### M3. Run input UX currently exposes internal IDs and duplicates fields

- Run defaults include both node ID and logical name: `apps/ui/src/app/pages/NodeEditorPage.tsx:865`, `apps/ui/src/app/pages/NodeEditorPage.tsx:872`.
- Modal renders all keys as editable inputs: `apps/ui/src/app/pages/NodeEditorPage.tsx:1170`.

**Impact**: Users see duplicate/internal keys, increasing operator error and making workflows harder to use as a reusable platform.

- [ ] **Accept — fix in 3C**: Only send logical name as key. Rust resolver already handles name→node_id fallback.

### M4. Router branching behavior still marked placeholder in runtime

- Code explicitly leaves downstream branch skipping as placeholder: `apps/desktop/src-tauri/src/commands.rs:3160`.

**Impact**: Conditional control flow is not deterministic yet, limiting real multi-branch automation use cases.

- [ ] **Accept — fix in 3C**: Implement handle-based branch routing — skip downstream nodes not on selected branch.

### L1. STATUS and implementation have drift on router/tool execution state

- STATUS says router/tool are still "currently skipped": `STATUS.md:140`.
- Runtime includes router/tool executors: `apps/desktop/src-tauri/src/commands.rs:2763`, `apps/desktop/src-tauri/src/commands.rs:2768`.

**Impact**: Planning and external communication risk (team may optimize against outdated status).

- [x] **Accept — fix now**: Update STATUS.md to reflect that router/tool executors exist. (2026-02-17)

### L2. Sidecar orphan cleanup uses Unix-only `fuser` dependency

- Orphan cleanup depends on `fuser` under unix cfg: `apps/desktop/src-tauri/src/sidecar.rs:176`, `apps/desktop/src-tauri/src/sidecar.rs:180`.

**Impact**: Operational behavior may vary across environments where `fuser` is unavailable, especially dev machines.

- [ ] **Deferred**: Known limitation. Windows support not in current scope. Will add `#[cfg(windows)]` fallback (`netstat` + `taskkill`) when Windows CI is set up.

## Platform Readiness Summary

This project is now clearly a **platform foundation** (agents + events + inspector + MCP + node workflows), not a single-use tool.

What is strong now:

- Clear 3-layer architecture and persistence backbone.
- Evented runtime + inspector base.
- Workflow execution path exists and is already useful for iterative platform development.

What blocks "office automation platform" confidence:

1. Router data contract and branching correctness.
2. Unified approval/security semantics for workflow tool calls.
3. Event contract consistency across all emitters.
4. Trigger/reliability layer (scheduler/webhook/retry/idempotency) for unattended operations.

## Suggested Fix Order

1. Fix router branch schema + safe fallback guard (H1).
2. Route workflow tool calls through approval policy path (H2).
3. Normalize workflow `agent_event` envelope to canonical schema (M2).
4. Fix approval payload key mismatch (M1).
5. Clean run-input UX and update STATUS to current truth (M3, L1).
