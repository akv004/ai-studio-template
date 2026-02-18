# Runtime & Safety Review: Chat, Tools, Runs, and Events

**Date**: 2026-02-18  
**Reviewer**: Codex (GPT-5)  
**Status**: RESOLVED
**Scope**: Desktop (`apps/desktop/src-tauri`), Sidecar (`apps/sidecar`), UI (`apps/ui`)

## Executive Summary

The project builds cleanly, but there are multiple high-severity runtime and safety defects in core chat/tool paths.  
Most critical: duplicate user prompts, ineffective run cancellation, shell safety bypass risk, and policy/approval mismatch for LLM-driven tool calls.

## Verification Performed

- `npm run build` ✅
- `cd apps/desktop/src-tauri && cargo check` ✅
- `python -m py_compile ...` on key sidecar files ✅

## Findings Matrix

| ID | Severity | Status | Finding |
|---|---|---|---|
| R1 | High | Open | User messages are duplicated before LLM call |
| R2 | High | Open | Cancelled runs can still be marked completed/failed |
| R3 | High | Open | `local` provider may fail when API key is empty |
| R4 | High | Open | Shell tool sandbox can be bypassed via shell interpretation |
| R5 | High | Open | LLM tool execution bypasses approval/rule intent |
| R6 | Medium | Open | Event sequencing/counters can drift due to dual sequence sources |
| R7 | Medium | Open | Inspector tool detail view expects wrong payload keys |
| R8 | Medium | Open | Missing targeted tests for critical runtime paths |

## Detailed Findings

### R1 - Duplicate user message in chat context (High)

**Evidence**
- Desktop persists user message: `apps/desktop/src-tauri/src/commands.rs:1327`
- Desktop then sends full history including new user message: `apps/desktop/src-tauri/src/commands.rs:1354`
- Sidecar hydrates history from request: `apps/sidecar/server.py:323`
- Chat service appends `user_message` again: `apps/sidecar/agent/chat.py:120`, `apps/sidecar/agent/chat.py:163`

**Impact**
- Inflated prompt tokens/cost
- Lower response quality from repeated user content
- Hard-to-debug conversation drift

**Expected fix direction**
- Ensure each user turn enters provider message list exactly once for both simple and tool loops.

---

### R2 - Run cancellation not enforced (High)

**Evidence**
- Cancellation updates run status: `apps/desktop/src-tauri/src/commands.rs:1058`
- Background execution still writes terminal status unconditionally:
  - success path: `apps/desktop/src-tauri/src/commands.rs:1026`
  - error path: `apps/desktop/src-tauri/src/commands.rs:1039`

**Impact**
- User sees cancelled runs later become completed/failed
- Incorrect run history/state

**Expected fix direction**
- Guard terminal updates so they apply only if run is still `pending`/`running`.

---

### R3 - `local` provider bootstrap gap when no API key (High)

**Evidence**
- Dynamic provider registration only happens when `api_key` is present: `apps/sidecar/server.py:305`
- Provider lookup fails if not registered: `apps/sidecar/agent/chat.py:73`

**Impact**
- Common local setup (`base_url`, no key) can fail unexpectedly

**Expected fix direction**
- Register/create provider when provider-specific config exists, not only when API key exists.

---

### R4 - Shell sandbox bypass via shell execution mode (High)

**Evidence**
- Safety check in sandbox mode only inspects parsed base command: `apps/sidecar/agent/tools/shell.py:86`
- Command executes through shell interpreter: `apps/sidecar/agent/tools/shell.py:126`

**Impact**
- Metacharacter/chaining payloads can bypass intended restrictions

**Expected fix direction**
- In protected modes (`sandboxed`, `restricted`), avoid shell interpreter (`create_subprocess_exec` + strict parsing/validation).

---

### R5 - Approval/rule intent not applied to LLM-driven tool calls (High)

**Evidence**
- Desktop approval modal gates only `/tools/*` proxy path: `apps/desktop/src-tauri/src/sidecar.rs:487`
- Main chat uses sidecar direct proxy path: `apps/desktop/src-tauri/src/sidecar.rs:223`, `apps/desktop/src-tauri/src/commands.rs:1411`
- Sidecar executes tool calls directly in loop: `apps/sidecar/agent/chat.py:275`
- UI wording implies stronger restriction semantics: `apps/ui/src/app/pages/AgentsPage.tsx:232`

**Impact**
- Policy mismatch and higher prompt-injection blast radius

**Expected fix direction**
- Enforce a real policy boundary for LLM tool execution in `restricted` mode (or safely disable until approval path exists).

---

### R6 - Event sequence/counter drift (Medium)

**Evidence**
- DB enforces unique `(session_id, seq)`: `apps/desktop/src-tauri/src/db.rs:159`
- Desktop assigns sequence from DB max+1: `apps/desktop/src-tauri/src/commands.rs:1535`
- Sidecar assigns independent in-memory sequence: `apps/sidecar/agent/events.py:32`
- Bridge inserts with `INSERT OR IGNORE`: `apps/desktop/src-tauri/src/sidecar.rs:395`
- Bridge also mutates session token/cost totals: `apps/desktop/src-tauri/src/sidecar.rs:402`

**Impact**
- Silent event drops on collisions
- Inspector timeline inconsistencies
- Session counters can drift

**Expected fix direction**
- Centralize sequence assignment at persistence boundary and avoid silent ignore behavior for expected collisions.

---

### R7 - Inspector tool details mismatch (Medium)

**Evidence**
- UI tool summary/detail reads `tool`, `input`, `output`: `apps/ui/src/app/pages/InspectorPage.tsx:125`, `apps/ui/src/app/pages/InspectorPage.tsx:414`
- Producers emit `tool_name`, `tool_input`, `tool_output`:
  - desktop: `apps/desktop/src-tauri/src/commands.rs:1438`, `apps/desktop/src-tauri/src/commands.rs:1455`
  - sidecar: `apps/sidecar/server.py:267`

**Impact**
- Missing tool detail display in inspector

**Expected fix direction**
- Add backward-compatible key handling in inspector (`tool_name`/`tool_input`/`tool_output` plus legacy keys).

---

### R8 - Test coverage gap on critical paths (Medium)

**Evidence**
- No targeted automated tests found for chat duplication, run cancellation semantics, tool policy gating, or event consistency.

**Impact**
- High regression probability in critical runtime flows

**Expected fix direction**
- Add focused tests/unit checks for each high-severity fix.

## Implementation Priority

1. R1 (prompt duplication)
2. R2 (run cancellation correctness)
3. R4 + R5 (tool safety/policy)
4. R3 (provider bootstrap)
5. R6 + R7 (observability consistency)
6. R8 (tests)

## Triage Checklist

- [x] R1 **Accept** — Fixed 2026-02-17. Strip last user msg from hydrated history to avoid double-append.
- [x] R2 **Accept** — Fixed 2026-02-17. Added `AND status = 'running'` to terminal UPDATE statements.
- [x] R3 **Accept** — Fixed 2026-02-17. Provider registered when `api_key or base_url or extra_config` present.
- [ ] R4 **Defer to 3C** — Valid concern. Shell sandbox bypass via metacharacters. Security hardening, not blocking.
- [ ] R5 **Defer to 3C** — Known limitation. LLM-driven tool approval needs async approval flow (Phase 3 work).
- [ ] R6 **Defer to 3C** — Low practical impact. Centralized sequencing is a larger refactor.
- [x] R7 **Accept** — Fixed 2026-02-17. Inspector now reads `tool_name`/`tool_input`/`tool_output` with fallbacks.
- [ ] R8 **Defer to 3C** — Tests planned for polish phase.
