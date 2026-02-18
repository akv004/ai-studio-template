# Phase 4 Implementation Review — Prompt for Codex / GPT

> **Recommended reviewer**: VS Code Codex (GPT-5) — code-level review
> **Date**: 2026-02-18
> **Topic**: Phase 4 Rust executor implementation details + security model

## What to Review

AI Studio's Phase 4 spec defines 10+ new Rust executors and engine changes. This review validates the implementation feasibility, edge cases, and security of the proposed code.

## Files to Read

1. **The Phase 4 spec** (PRIMARY): `docs/specs/phase4-automation-canvas.md`
   - Focus on: Part 3 (Data I/O nodes), Part 4 (Control Flow nodes), Part 6 (Executor Registry)
2. **Current engine implementation**: `apps/desktop/src-tauri/src/workflow/engine.rs`
3. **Current executor trait + registry**: `apps/desktop/src-tauri/src/workflow/executors/mod.rs`
4. **Example executor** (LLM — most complex): `apps/desktop/src-tauri/src/workflow/executors/llm.rs`
5. **Example executor** (Transform — template resolution): `apps/desktop/src-tauri/src/workflow/executors/transform.rs`
6. **Example executor** (Router — skip_nodes): `apps/desktop/src-tauri/src/workflow/executors/router.rs`
7. **Cargo.toml** (dependencies): `apps/desktop/src-tauri/Cargo.toml`
8. **Sidecar server** (for Code node endpoint): `apps/sidecar/server.py`

## What to Look For

### 1. Executor Implementation Feasibility

For each new executor in the spec, check:
- Can it be implemented with the current `NodeExecutor` trait signature?
- Does it need changes to `ExecutionContext`?
- Are the proposed Rust crate dependencies sufficient?
- Any edge cases in input/output handling?

Specific executors to scrutinize:

**HTTP Request (`executors/http_request.rs`)**:
- `reqwest` is already in Cargo.toml — can it handle all auth modes (bearer, basic, API key)?
- Template resolution of URL — what if the URL template evaluates to an invalid URL?
- Headers: merging config headers with incoming edge headers — precedence correct?
- Response body: what if it's binary (image, PDF)? Spec says output is text.

**Shell Exec (`executors/shell_exec.rs`)**:
- `tokio::process::Command` — does the current tokio version support all features?
- Stdin piping via `child.stdin` — does this work with `wait_with_output()`?
- Timeout: `tokio::time::timeout` + process kill — is the process properly cleaned up?
- UTF-8 assumption for stdout/stderr — what about binary output?
- Command injection: if command comes from template resolution (LLM output → shell), is this a security risk?

**Validator (`executors/validator.rs`)**:
- `jsonschema` crate: is version 0.28 compatible with our Rust edition?
- JSON Schema draft 7 — does the crate support this draft?
- Performance: schema compilation for every node execution, or should schemas be cached?

**Subworkflow (`executors/subworkflow.rs`)**:
- `visited_workflows: HashSet<String>` in ExecutionContext — how is this threaded through recursive calls?
- The current ExecutionContext uses references — can it be extended without lifetime issues?

**Code node + sidecar endpoint**:
- New sidecar endpoint `POST /code/execute` — fits the existing sidecar architecture?
- Subprocess security: minimal env (HOME, PATH, AI_STUDIO_INPUT) — is this sufficient?
- Timeout: `subprocess.run()` with timeout — what happens on timeout? Zombie processes?
- JavaScript execution: `node -e "..."` — is Node.js guaranteed to be available?

### 2. Engine Refactoring Risk

The spec proposes extracting `execute_subgraph()` from `execute_workflow()`.

- Read `engine.rs` carefully. How coupled is the current execute function?
- Can the topological sort, output collection, event emission, and skip_nodes logic be cleanly factored?
- The spec says "container nodes skip their children in the main topo sort." How is this implemented without breaking existing graph validation?
- Template resolution (`{{node_id.output}}`) — does it work correctly when node_id refers to a node inside a container?

### 3. Security Audit

New nodes interact with the system (shell, filesystem, network, code execution).

- **Shell Exec**: Command injection via template variables? If an LLM generates a command containing `; rm -rf /`, is it passed verbatim to bash?
- **File Read/Write**: Path traversal? If path is `../../etc/passwd`, does Tauri FS scope catch it?
- **HTTP Request**: SSRF? Can a template-resolved URL point to `http://localhost:8765` (our own sidecar)?
- **Code node**: Sandbox escape? Can Python code access the filesystem, network, or Tauri APIs?
- **Auth tokens**: Graph JSON is stored in SQLite and can be exported. Auth tokens in HTTP node config are visible.

### 4. New Cargo Dependency

`jsonschema = "0.28"` — is this the right crate?
- Is it maintained?
- What's its dependency tree size?
- Are there alternatives (e.g., `boon`, `valico`)?

### 5. CSV Parsing

The spec proposes "basic split implementation (no external crate dependency)" for CSV in File Read.

- Is a basic split reliable for real-world CSV files?
- Quoted fields with embedded delimiters, newlines in quoted fields, BOM bytes — handled?
- Should we just add the `csv` crate instead?

## Expected Output Format

```markdown
## Phase 4 Implementation Review

**Reviewer**: [Your name/model]
**Date**: [Date]
**Status**: Draft

### Findings

- [ ] **F1**: [Finding title] — [severity: HIGH/MEDIUM/LOW]
  - File/section: ...
  - Description: ...
  - Impact: ...
  - Code suggestion: ...

- [ ] **F2**: ...

### Security Audit

| Area | Risk Level | Finding | Mitigation |
|------|-----------|---------|-----------|
| Shell Exec | HIGH/MED/LOW | ... | ... |
| File I/O | ... | ... | ... |
| HTTP Request | ... | ... | ... |
| Code Exec | ... | ... | ... |

### Dependency Review

| Crate | Version | Verdict | Notes |
|-------|---------|---------|-------|
| jsonschema | 0.28 | OK/WARN/REJECT | ... |
```
