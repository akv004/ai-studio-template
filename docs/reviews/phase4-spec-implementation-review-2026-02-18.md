## Phase 4 Implementation Review

**Reviewer**: Codex (GPT-5.2)
**Date**: 2026-02-18
**Status**: TRIAGED — 9 accepted (spec updated), 1 deferred (F10 event consistency → Phase 4C)

### Findings

- [x] **F1**: Engine ignores `sourceHandle` for data flow (output handles don’t work) — **severity: HIGH**
  - File/section: `apps/desktop/src-tauri/src/workflow/engine.rs` (incoming resolution), `apps/ui/src/app/pages/NodeEditorPage.tsx` (multi-output handles)
  - Description: Incoming values are built from `node_outputs.get(src_id)` and do **not** use `sourceHandle` to select a specific output. This means edges from e.g. LLM `usage`/`cost`, Approval `approved`/`rejected`, and all Phase 4 multi-output nodes will pass the same “whole node output” (or whatever was “cleaned”) regardless of which output handle is wired.
  - Impact: Phase 4 nodes with multiple outputs (HTTP/File/Shell/Validator/etc.) won’t behave correctly; some Phase 3 UI handles are already misleading.
  - Code suggestion: Treat node output as a map of handle → value. At edge-merge time, select the value using `(src_id, sourceHandle)`:
    - If the stored node output is an object and contains `sourceHandle`, use that field.
    - Otherwise, fall back to the whole node output for backward compatibility (keeps Router-style “flow” handles working).

- [x] **F2**: Engine "cleans" node outputs in a way that destroys structured/multi-output data — **severity: HIGH**
  - File/section: `apps/desktop/src-tauri/src/workflow/engine.rs` (`clean_output` logic)
  - Description: After execution, the engine stores `clean_output` into `node_outputs`, stripping `__usage` and often reducing structured outputs to a single string (`content`/`result`). The LLM executor returns `response/usage/cost/...`, but the engine persists only the “primary” string, making `{{llm_id.usage}}` and LLM output handles unusable.
  - Impact: Structured outputs cannot be routed by handle, cannot be referenced reliably by templates, and cannot support the Phase 4 multi-output node model.
  - Code suggestion: Store the full raw output for routing/template purposes, and compute a separate preview string for event logging/UI. If a “primary output” is desired, define it explicitly (e.g., `output` field) rather than inferring from `__usage`.

- [x] **F3**: Loop/Error Handler container model is not implementable without engine + validation changes beyond `execute_subgraph()` — **severity: HIGH**
  - File/section: `docs/specs/phase4-automation-canvas.md` (Loop/Error Handler), `apps/desktop/src-tauri/src/workflow/engine.rs`, `apps/desktop/src-tauri/src/workflow/validation.rs`
  - Description: Current execution topo-sorts and executes **all** nodes in the graph. Container nodes require skipping children in the main topo order and executing the child subgraph under container control (by `parentId`). Validation currently has no notion of container boundaries (or cross-boundary edges).
  - Impact: Without filtering and boundary validation, children may execute outside the container’s control, or interleave with the parent graph, breaking loop semantics and error catching/retry.
  - Code suggestion:
    - In `execute_workflow()`, topo-sort only top-level nodes (no `parentId`).
    - Add `execute_subgraph()` that accepts a node/edge subset + `initial_inputs`.
    - Add validation rules: (1) container must have ≥1 child, (2) single well-defined “entry” and “exit” for body, (3) forbid (or very explicitly define) edges that cross container boundaries.

- [x] **F4**: `ExecutionContext` lacks the data needed for Subworkflow + container executors — **severity: MEDIUM**
  - File/section: `apps/desktop/src-tauri/src/workflow/executors/mod.rs`
  - Description: Spec calls for `visited_workflows` (circular subworkflow detection) and container execution over a subset of nodes/edges. Current `ExecutionContext` exposes DB/sidecar/app/settings + `node_outputs` + `outgoing_by_handle`, but not the workflow graph, node metadata, or a way to invoke `execute_subgraph()`.
  - Impact: Subworkflow/Loop/Error Handler executors either can’t be implemented cleanly as executors, or they’ll require ad-hoc DB reloads / duplicated parsing / tightly coupled engine calls.
  - Code suggestion: Extend context with (a) `visited_workflows`, and for container execution either (b) an `EngineHandle`/closure to run subgraphs, or (c) pass the parsed graph (nodes/edges) into `ExecutionContext` for reuse.

- [x] **F5**: File Read/Write security model assumes Tauri FS scope, but Rust `std::fs` bypasses it — **severity: HIGH**
  - File/section: `docs/specs/phase4-automation-canvas.md` (3.2/3.3), `apps/desktop/src-tauri/tauri.conf.json`
  - Description: The spec states “Scoped to Tauri FS scope,” but the proposed executors use `std::fs::{read,write,...}` in Rust. Tauri scope restrictions primarily govern frontend APIs/plugins; Rust backend code can read/write anywhere the user can. “No symlink following by default” is also not true for `std::fs` without explicit checks.
  - Impact: A workflow could read `~/.ssh/*`, cloud credentials, or overwrite arbitrary user files unless explicit scoping/approval is implemented in Rust.
  - Code suggestion:
    - Define an explicit allowlist root (workspace, app data dir, user-selected folders) and enforce via `canonicalize()` + `starts_with()` checks.
    - Consider a “deny sensitive paths” policy similar to `apps/sidecar/agent/tools/filesystem.py`.
    - Treat File Write as `"ask"` by default; strongly consider File Read as `"ask"` unless path is within an allowlisted root.

- [x] **F6**: HTTP Request node needs SSRF + secret-handling clarifications; “runtime-only authToken” conflicts with persisted graph_json — **severity: HIGH**
  - File/section: `docs/specs/phase4-automation-canvas.md` (3.1), `apps/desktop/src-tauri/src/workflow/mod.rs` (graph_json stored in SQLite)
  - Description:
    - SSRF: allowing arbitrary URLs enables access to localhost/private networks (including sidecar port scanning) from the desktop app context.
    - Secrets: node config lives in `graph_json` persisted to SQLite and exportable; a literal `authToken` field is not “runtime-only” unless a vault/secret reference is implemented.
    - Response body: spec says output is text; real APIs may return binary or huge bodies.
  - Impact: Security and privacy risk (exfiltration, internal-service probing) and UX risk (large response memory blowups).
  - Code suggestion:
    - Add policy: block private IP ranges by default (or require approval), and enforce `max_response_bytes`.
    - Make auth fields reference settings keys/secret IDs, not raw tokens in graph JSON (Phase 4 should at least avoid accidental export leakage).
    - Decode as UTF-8 with replacement; optionally surface `content_type`/`is_binary` as extra outputs.

- [x] **F7**: Shell Exec executor requires new tokio features + careful process cleanup; “no inherited env” is tricky cross-platform — **severity: HIGH**
  - File/section: `docs/specs/phase4-automation-canvas.md` (3.4), `apps/desktop/src-tauri/Cargo.toml` (`tokio` features)
  - Description:
    - Current `tokio` features are `time,sync`; `tokio::process` requires enabling `process` (and often `io-util`).
    - `wait_with_output()` + stdin piping needs correct ordering (write stdin, drop it, then await output).
    - Timeout kill: `child.kill()` may not terminate grandchildren; process-group kill is needed for robust cleanup on Unix.
    - `env_clear()` removes PATH; shells may not be found unless absolute paths or a minimal PATH is re-injected.
    - Shell availability (`bash`/`zsh`) is not guaranteed on Windows.
  - Impact: Non-compiling build until tokio features updated; potential zombie processes; platform-specific failures; env leakage if not cleared.
  - Code suggestion:
    - Enable tokio `process` (+ `io-util` as needed).
    - On Unix: set process group and kill group on timeout.
    - Apply output-size caps and `String::from_utf8_lossy`.
    - Gate behind `"ask"` approval with the fully resolved command + cwd + env preview; consider restricting to Unix shells or adding a Windows plan (PowerShell).

- [x] **F8**: Code node sidecar endpoint `/code/execute` is missing; “Node.js guaranteed” and “no network access” are not true by default — **severity: HIGH**
  - File/section: `docs/specs/phase4-automation-canvas.md` (4.4), `apps/sidecar/server.py`
  - Description:
    - Sidecar currently has `/chat/direct` and tool endpoints, but no `/code/execute`.
    - `node -e` assumes Node.js exists on the user machine or in the bundle.
    - “No network access” cannot be enforced without OS sandboxing (containers/seccomp/firewall); subprocesses can still access network by default.
    - “Minimal env” needs explicit `env={...}`; sidecar’s existing shell tool inherits `os.environ`.
  - Impact: Spec as written is not implementable on all platforms without packaging decisions; security claims could be misleading.
  - Code suggestion:
    - Start with Python-only (guaranteed by sidecar packaging) or embed a JS engine (e.g., QuickJS) if JS is required.
    - Implement `/code/execute` with strict timeouts, output caps, and explicit `env` (HOME, PATH, AI_STUDIO_INPUT only).
    - Update spec language: treat “no network” as Phase 5 sandbox work; Phase 4 can only “best-effort” restrict.

- [x] **F9**: Validator executor: `jsonschema` integration needs caching strategy + draft support confirmation — **severity: MEDIUM**
  - File/section: `docs/specs/phase4-automation-canvas.md` (3.5), `apps/desktop/src-tauri/Cargo.toml`
  - Description: Validating is straightforward with `jsonschema`, but compiling the schema on every node execution can be expensive. Also confirm the crate’s supported drafts match “draft 7” expectations.
  - Impact: Performance regressions on loops/retries; confusing validation gaps if draft mismatches.
  - Code suggestion: Cache compiled schemas keyed by schema string (or hash) in the executor (e.g., `Mutex<LruCache<...>>`), and emit errors as structured array output when `failOnError=false`.

- [ ] **F10**: Workflow events don’t yet match the spec’s “inputs/outputs + node-specific fields” model — **severity: LOW**
  - File/section: `apps/desktop/src-tauri/src/workflow/engine.rs` (event payloads), `apps/desktop/src-tauri/src/workflow/executors/router.rs` (extra event emission)
  - Description: Engine emits `workflow.node.*` events with `output_preview` only; some executors also emit their own `workflow.node.completed`, causing duplicates with different payload shapes.
  - Impact: Inspector data is inconsistent; Phase 4 “extra fields in completed payload” will be hard to implement cleanly without a unified event envelope.
  - Code suggestion: Make the engine the single place that emits `workflow.node.*` events; allow executors to return structured metadata (e.g., `NodeOutput { value, meta }`) so the engine can attach node-specific fields consistently.

### Security Audit

| Area | Risk Level | Finding | Mitigation |
|------|-----------|---------|------------|
| Shell Exec | HIGH | Arbitrary command execution + command injection via template-resolved inputs; potential env leakage; incomplete timeout cleanup | Default `"ask"` approval with resolved command/cwd/env; `env_clear` + minimal PATH; process-group kill on timeout; consider restricted mode/allowlist (Phase 5) |
| File I/O | HIGH | `std::fs` in Rust bypasses any frontend scope; path traversal + symlink tricks; write can overwrite user files | Explicit allowlisted roots + canonicalization; deny sensitive paths; File Write `"ask"`; optionally require File Read approval outside allowlist |
| HTTP Request | MEDIUM/HIGH | SSRF to localhost/private IPs; secret tokens likely persisted in `graph_json`; large/binary bodies | Block/ask for private IP ranges; response size limits; token indirection via settings/secret IDs; lossy UTF-8 decode + surface content-type |
| Code Exec | HIGH | Arbitrary code execution; cannot truly block network/filesystem without sandbox; Node.js availability not guaranteed | Default `"ask"`; start with Python-only or embed JS; strict env + timeouts + output caps; document sandbox as Phase 5 work |

### Dependency Review

| Crate | Version | Verdict | Notes |
|-------|---------|---------|-------|
| jsonschema | 0.28 | WARN | Likely workable, but confirm draft-7 behavior + dependency size; add schema compile caching to avoid per-execution overhead |
| csv | TBD | OK (Recommended) | A “basic split” CSV parser will be brittle (quotes/newlines/BOM). Using `csv` reduces correctness risk and code complexity |
| base64 | TBD | OK (Likely needed) | Needed for File Read `binary` mode if implemented in Rust |
| encoding_rs (or similar) | TBD | WARN | Spec exposes `encoding`, but `std::fs::read_to_string` is UTF-8 only; either constrain to UTF-8 in v1 or add an encoding crate |

