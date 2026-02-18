# AI Studio Productization Review

**Date**: 2026-02-09
**Status**: Closed (2026-02-15)
**Reviewer**: Antigravity

---

## 1. Executive Summary

The AI Studio architecture (UI -> Tauri -> Python Sidecar) is solid and well-aligned with the `architecture.md` specification. The 3-layer separation provides good security boundaries and independent testing capabilities.

However, there are discrepancies between the `data-model.md` specification and the actual database implementation in `db.rs` that need to be addressed before a production release. Specifically, the `agents` table schema and the global `approval_rules` system need alignment.

## 2. Implementation Gaps (Code Analysis)

### A. Database Service (`db.rs`) vs Spec (`data-model.md`)
- [x] **Schema Mismatch**: The `agents` table in `db.rs` uses a generic `tools` JSON column, whereas the spec requires `tools_mode`, `mcp_servers`, and `approval_rules` columns.
- [x] **Missing Table**: The `approval_rules` table (for global rules) is defined in the spec but missing in `db.rs`.
- [x] **Persistence**: `persist_ws_event` uses `INSERT OR IGNORE`, which might silently swallow errors.

### B. Sidecar (`server.py`, `chat.py`)
- [x] **Error Handling**: `ChatService.chat_with_tools` catches exceptions and returns them as tool outputs. This is good for resilience but should ensure `tool.error` events are always emitted (currently seems covered).
- [x] **Auth**: `server.py` implements the `x-ai-studio-token` check and `auth` message over WebSocket. Correct.

### C. Desktop (`sidecar.rs`)
- [x] **Event Bridge**: Retry logic with backoff is properly implemented.
- [x] **Tool Approval**: Implemented via `tool_approval_requested` event.

## 3. Productization Recommendations-dangerously-skip-permissions 

### High Priority (Stability & Data Integrity)
1.  **Align DB Schema**: Update `db.rs` to match `data-model.md`. specifically splitting the `agents.tools` column into `mcp_servers` and `approval_rules`. (Closed 2026-02-13, commit 8d370f0)
2.  **Global Approval Rules**: Implement the `approval_rules` table to allow system-wide security policies (e.g., "Always deny `rm -rf`"). (Closed 2026-02-13, commit 8d370f0)
3.  **Structured Error Events**: Ensure the Sidecar emits a dedicated `agent.error` or `system.error` event for top-level crashes, not just tool errors. (Closed 2026-02-14, commit 30cd467)

### Medium Priority (DX & Observability)
4.  **Log Rotation**: The sidecar logs to stdout. For a product, these should be piped to a file in `~/.ai-studio/logs/` with rotation. (Deferred to P3)
5.  **Config Validation**: Add Pydantic validation for values stored in the `settings` table to prevent invalid JSON from breaking the UI. (Deferred to P3)

## 4. Product Vision & UX Review

### Strengths ("The Moat")
- **The Inspector**: `InspectorPage.tsx` is visually rich and delivers on the "Chrome DevTools for Agents" promise. The timeline, token stats, and cost breakdowns are clear differentiators against chat-only tools like OpenClaw or LM Studio.
- **3-Layer Architecture**: The separation of concerns (UI / Tauri / Sidecar) is excellent for stability and matches the "Pro Tool" vibe.

### Critical Gaps (Vision vs Reality)
1.  **Missing Onboarding**: The vision targets "< 3 minutes to first run". Currently, a new user lands on an empty `AgentsPage` with no guidance.
    - *Recommendation*: Build a "Welcome to AI Studio" modal or empty state that guides them to "Create your first Agent" or offers a "Try Demo Agent" button.
    - (Closed 2026-02-15, commit b786c8b — 3-step onboarding wizard)
2.  **Hybrid Intelligence Visibility**: The vision highlights "auto-routing" as a key feature. However, `AgentsPage.tsx` only allows picking a single static model.
    - *Recommendation*: Add a "Auto (Hybrid)" option in the model selector that uses the cost/complexity routing logic defined in `hybrid-intelligence.md`.
    - (Deferred to P3 — hybrid intelligence spec)
3.  **Keyboard-First UX**: The vision claims "Every action reachable by keyboard". The current UI relies heavily on mouse clicks (standard React buttons).
    - *Recommendation*: Implement a global Command Palette (`Cmd+K`) to jump between Agents, Sessions, and Inspector.
    - (Rejected — Command Palette already exists, reviewer missed it)

---

*Signed,*
*Antigravity*
*2026-02-09*
