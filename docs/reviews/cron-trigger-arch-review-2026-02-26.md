# Cron Trigger Architecture Review
**Date**: 2026-02-26
**Reviewer**: Gemini 3 Pro
**Status**: RESOLVED

### Findings Table
| # | Area | Priority | Verdict | Finding |
|---|------|----------|---------|---------|
| 1 | Tick loop correctness | HIGH | PASS | The `take(1)` logic safely captures the minute's occurrence because the internal schedule is capped at 1-minute frequency (by prepending `0 `). The use of monotonic UTC timestamps for minute-matching (`now.timestamp()`) is completely robust against DST timezone duplication. |
| 2 | In-memory Duplicate Fire | HIGH | FAIL | On startup or trigger re-arming, `last_fired_minute` is initialized to `None` in memory. If the app restarts within the same minute a trigger already fired, it will fire again. The scheduler must initialize `last_fired_minute` using the `last_fired` timestamp loaded from the DB. |
| 3 | Concurrency safety | LOW | PASS | `active_runs` TOCTOU race is harmless. The increment is single-threaded (from the centralized tick loop), ensuring `max_concurrent` is never exceeded. The decrement is concurrent but only produces conservatively high estimates. |
| 4 | Lock ordering | MED | PASS | There is no deadlock scenario. The tick loop clones the `cron_schedules` entries and drops the outer `MutexGuard` *before* iterating and acquiring the inner `last_fired_minute` lock, so `disarm_cron` can safely acquire the schedules lock independently. |
| 5 | Shutdown safety | MED | WARN | Dropping the tick loop oneshot leaves `spawn` worker executions completely detached. The app shutdown might halt in-flight `execute_workflow_ephemeral` calls unexpectedly. A `tokio::task::JoinSet` should be introduced to await active runs gracefully. |
| 6 | Catch-Up Policy Missing | HIGH | FAIL | The `catchUpPolicy` strategy (skip, run_once, run_all) mandated by the spec is completely unimplemented. Missed executions are silently ignored, acting purely as an implicit "skip". |
| 7 | Expression conversion security | LOW | PASS | User cron inputs are safely mapped to the `cron::Schedule` crate parser. The fallback logic correctly mitigates arbitrary execution, bounds the parsing, and safely drops bad syntax with an error without panicking. |

### Actionable Checklist
- [x] Initialize `last_fired_minute` from DB `last_fired` timestamp (2026-02-26, triggers.rs)
- [ ] Deferred to Phase 5+: `catchUpPolicy` handling (skip-only is fine for v1, spec feature)
- [ ] Deferred to Phase 5+: `JoinSet` for graceful drain (low risk for desktop app)
- [ ] Deferred to Phase 5+: `catchUpPolicy` parsing/enforcement (same as above)

### Notes
The overall integration into the pre-existing trigger architecture is excellent. `TriggerManager` natively aligns with the IPC channels used by webhook triggers, maintaining strong codebase coherence.
