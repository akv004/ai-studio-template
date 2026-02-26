# Cron Trigger Implementation Review
**Date**: 2026-02-26
**Reviewer**: Codex (GPT-5)
**Status**: RESOLVED

### Findings Table
| # | Area | Priority | Verdict | Finding |
|---|------|----------|---------|---------|
| 1 | Minute matching logic | MED | PASS | For normalized 5-field cron (`0 ... *`), `schedule.after(local_now - 60s).take(1)` correctly matches the current minute and avoids same-minute double-fire via `last_fired_minute` (`apps/desktop/src-tauri/src/webhook/mod.rs:288`). |
| 2 | 6/7-field cron edge cases | HIGH | FAIL | Non-5-field expressions pass through unchanged (`apps/desktop/src-tauri/src/commands/triggers.rs:344`), but scheduler dedup is minute-based; second-level schedules can fire early/once-per-minute instead of at requested seconds (`apps/desktop/src-tauri/src/webhook/mod.rs:275`, `apps/desktop/src-tauri/src/webhook/mod.rs:290`). |
| 3 | Trigger bootstrap filtering | HIGH | FAIL | Workflow canvas calls `list_triggers` with `{ request: { workflowId } }` instead of `{ workflowId }`, so the filter can be ignored and the UI may bind to another workflow’s trigger (`apps/ui/src/app/pages/workflow/WorkflowCanvas.tsx:145`). |
| 4 | Dual-trigger workflow handling | HIGH | FAIL | Validation allows webhook+cron coexistence, but toolbar logic uses one `triggerId`, picks webhook node first, and exposes a single arm/disarm control; cron/webhook cannot be managed independently (`apps/ui/src/app/pages/workflow/WorkflowCanvas.tsx:523`, `apps/ui/src/app/pages/workflow/WorkflowCanvas.tsx:918`). |
| 5 | `catchUpPolicy` implementation gap | MED | WARN | UI captures and persists `catchUpPolicy` (`apps/ui/src/app/pages/workflow/NodeConfigPanel.tsx:818`, `apps/ui/src/app/pages/workflow/WorkflowCanvas.tsx:536`), but arm path and scheduler never read/apply it (`apps/desktop/src-tauri/src/commands/triggers.rs:372`, `apps/desktop/src-tauri/src/webhook/mod.rs:20`). |
| 6 | `fire_count` consistency | MED | WARN | In-memory `fire_count` increments before spawn (`apps/desktop/src-tauri/src/webhook/mod.rs:311`) while DB increments separately (`apps/desktop/src-tauri/src/webhook/mod.rs:459`); DB update failures are ignored, so iteration values can drift from persisted counts. |
| 7 | Max concurrency validation | MED | WARN | Backend clamps only upper bound (`min(10)`) and accepts `0` from API config (`apps/desktop/src-tauri/src/commands/triggers.rs:367`), which causes permanent skip (`active >= 0`) in scheduler (`apps/desktop/src-tauri/src/webhook/mod.rs:299`). |
| 8 | Executor test quality | HIGH | FAIL | `cron_trigger` tests do not execute `CronTriggerExecutor::execute`; they only reconstruct HashMap/JSON logic, so they miss real execution-path regressions (`apps/desktop/src-tauri/src/workflow/executors/cron_trigger.rs:65`). |
| 9 | Scheduler test coverage depth | MED | WARN | Cron tests validate parser/math primitives only; there are no tests for tick-loop matching, per-minute dedup behavior, max-concurrent with spawned tasks, or DST duplicate-minute behavior (`apps/desktop/src-tauri/src/webhook/mod.rs:586`). |
| 10 | Timezone UX completeness | LOW | WARN | Config panel has only 11 fixed timezone options (`apps/ui/src/app/pages/workflow/NodeConfigPanel.tsx:796`), so many common IANA zones are unavailable despite backend supporting them. |

### Actionable Checklist
- [x] Enforce strict 5-field cron input in backend; reject 6/7-field expressions (2026-02-26, triggers.rs)
- [x] Fix `list_triggers` invocation payload — `workflowId` directly, not wrapped in `request` (2026-02-26, WorkflowCanvas.tsx)
- [x] Split toolbar state by trigger type — per-type triggerId, armed state, arm/disarm handlers (2026-02-26, WorkflowCanvas.tsx)
- [ ] Deferred to Phase 5+: `catchUpPolicy` implementation (skip-only is fine for v1)
- [x] Harden `maxConcurrent` lower bound `.max(1)` to prevent permanent skip (2026-02-26, triggers.rs)
- [x] Rewrite cron executor tests with `build_cron_output()` pure function + 6 tests (2026-02-26, cron_trigger.rs)
- [x] Log DB update errors in execute_cron_run instead of silently ignoring (2026-02-26, webhook/mod.rs)
- [x] Expand timezone dropdown from 11 to 22 common IANA zones (2026-02-26, NodeConfigPanel.tsx)
- [ ] Deferred to Phase 5+: Scheduler integration tests for tick-loop, dedup, DST (requires Tauri runtime)

### Notes
- Targeted test run: `cargo test cron_trigger` and `cargo test webhook::tests::test_cron` both pass, but current suites do not exercise live tick-loop execution paths.
