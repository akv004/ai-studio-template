# Detailed Phase 2 Review: Session Branching & Architecture

**Date**: 2026-02-15
**Reviewer**: AI Assistant (Antigravity)
**Reference**: User Request ID 49

## Executive Summary

Phase 2 functionality ("Session Branching") has been implemented but contains **critical architectural regressions** that will cause the feature to fail in practice (Context Loss). The implementations of persistence and state management between the Desktop (Rust) and Agent (Python) layers are currently incorrectly synchronized for branched sessions.

**Recommendation**: 🛑 **DO NOT SHIP Phase 2** until the "Context Loss" and "Transaction Safety" issues are resolved.

**Status**: **RESOLVED** — All 6 items fixed (2026-02-15). See triage notes below.

---

## 1. Session Branching Analysis

| Check | Priority | Verdict | Findings |
|-------|----------|---------|----------|
| **Sidecar State Consistency** | 🔴 **HIGH** | **FAIL** | **Critical Bug**: The Python Sidecar holds conversation history in-memory (`chat.py`: `self.conversations`). `branch_session` in Rust copies messages in SQLite but **does not** inform the Sidecar. When the user sends a message in the new branch, the Sidecar creates a fresh, empty conversation. **The LLM will not see the branched history.** |
| **Transaction Safety** | 🔴 **HIGH** | **FAIL** | **Data Integrity Risk**: `branch_session` in `commands.rs` executes multiple SQL statements (`INSERT session`, loop `INSERT messages`, `UPDATE count`) on a raw connection without an explicit transaction. A failure mid-loop will leave a corrupt session state. |
| **Token/Cost Counting** | 🟠 **MED** | **FAIL** | **Logic Error**: The new session is initialized with 0 tokens/cost (`commands.rs` L597-600), but it inherits messages that have costs. Use `INSERT INTO ... SELECT SUM(...)` to pre-calculate these totals, or the Session List will show "$0.00" for a session that actually contains expensive history. |
| **Parent Deletion** | 🟠 **MED** | **WARN** | **Orphan Risk**: `parent_session_id` foreign key (db.rs L108) lacks `ON DELETE SET NULL` or `CASCADE`. Deleting a parent session will either fail (FK constraint) or leave ghost references depending on SQLite version nuances. |
| **Database Indices** | 🔵 **LOW** | **PASS** | Missing `idx_sessions_parent` is a minor perf optimization, not a blocker. Can be added in V4. |
| **Naming UX** | 🔵 **LOW** | **WARN** | "Branch of Branch of Branch of..." pattern detected. Suggestion: `format!("Branch of {}", parent_title.replace("Branch of ", ""))` or similar logic. |

### Suggested Fixes

#### 1. Fix Sidecar Context Loss
**Option A (Stateless Sidecar - Recommended for Phase 3)**:
Change the `POST /chat` endpoint to accept `messages: List[Message]` instead of `conversation_id`. Let Rust own the history and send the full context window on every turn. This aligns with a "Node Editor" architecture where nodes are stateless processors.

**Option B (Hydration - Quick Fix)**:
Add a `POST /chat/{id}/hydrate` endpoint to the Sidecar. Call this from Rust immediately after `branch_session` to push the copied messages into the Sidecar's memory.

#### 2. Fix Transaction Safety
Wrap the Rust logic in a transaction:
```rust
let mut conn = db.conn.lock().map_err(...)?;
let tx = conn.transaction().map_err(...)?;
// ... execute all queries on tx ...
tx.commit().map_err(...)?;
```

---

## 2. Phase 2 Gaps (Completeness)

*   **CONTRIBUTING.md**: Missing (Verified). Essential for open-source launch.
*   **Automated Testing**: No integration test exists that verifies a branched session actually sends history to the LLM. This would have caught the Context Loss bug immediately.

---

## 3. Phase 3 Architecture: Node Editor Readiness

**Is the Event-Sourced architecture ready for Node Graphs?**

*   **Yes:** The `event` schema (`type`, `payload`, `source`) is generic enough to represent node execution (`node.started`, `data.transformed`).
*   **No:** The `Agent` data model is too rigid.
    *   Current: `Agent` = `System Prompt` + `Model` + `Tools`.
    *   Node Editor: `Agent` = `Graph Definition` (JSON).
    *   **Gap**: We need a `graphs` table and a way to execute a graph. The Sidecar currently runs a hardcoded "Chat Loop" (`ChatService.chat_with_tools`). This loop needs to be extracted into a `GraphExecutor` engine.

---

## 4. Open Source Readiness

*   **DX**: `npm run tauri:dev` is smooth.
*   **Docs**: `README.md` is strong.
*   **First Run**: Onboarding wizard (Phase 2 feature) handles the initial "Connect Provider" step well.
*   **Blocker**: The bugs identified in Section 1. Shipping a "Branching" feature that forgets context will hurt the project's reputation on Day 1.

---

## Final Checklist for User

- [x] **Fix Critical**: Sidecar Context Loss — Rust now sends full message history to sidecar on every `/chat` call. Sidecar hydrates conversation from `history` field. (Fixed 2026-02-15)
- [x] **Fix Critical**: Wrap `branch_session` in SQL Transaction — uses `conn.transaction()` + `tx.commit()`. (Fixed 2026-02-15)
- [x] **Fix Logic**: Calculate initial cost/tokens for branched sessions — accumulates during message copy loop. (Fixed 2026-02-15)
- [x] **Fix DB**: Parent deletion orphan protection — V4 migration adds trigger `trg_sessions_parent_delete` (SET NULL). (Fixed 2026-02-15)
- [x] **Fix DB**: Missing `idx_sessions_parent` index — added in V4 migration. (Fixed 2026-02-15)
- [x] **Fix UX**: Branch title nesting — `strip_prefix("Branch of ")` prevents "Branch of Branch of..." chains. (Fixed 2026-02-15)
- [ ] **Feature**: Add `CONTRIBUTING.md`. (Deferred to open-source launch prep)
- [ ] **Architecture**: Plan the `ChatService` refactor to support stateless execution (pre-req for Node Editor). (Phase 3)

