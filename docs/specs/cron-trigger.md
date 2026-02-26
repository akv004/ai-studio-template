# Cron Trigger Node

**Status**: DRAFT — pending peer review
**Phase**: 4C (automation canvas)
**Priority**: P0 — critical for scheduled automation use cases
**Author**: AI Studio PM
**Date**: 2026-02-25

---

## Problem Statement

The Webhook Trigger node starts workflows on external HTTP requests. But many automation use cases are time-based: daily report generation, hourly data checks, weekly summaries. Without a Cron Trigger, users must manually run workflows or use external tools (cron, Task Scheduler) to POST to webhook endpoints — defeating the purpose of an integrated automation platform.

---

## Design

### Node Type: `cron_trigger`

A **source node** (like Input or Webhook Trigger) that fires on a time-based schedule. The workflow starts automatically when the schedule matches.

### Config

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| expression | string | required | Cron expression (standard 5-field: `min hour dom month dow`) |
| timezone | string | UTC | IANA timezone (`America/New_York`, `Asia/Kolkata`, etc) |
| staticInput | json | {} | Static data passed as workflow input on each trigger |
| maxConcurrent | int | 1 | Max overlapping runs (prevents pile-up if workflow is slow) |
| catchUpPolicy | enum | skip | skip / run_once / run_all — behavior for missed executions (app was closed) |

### Output Handles

| Handle | Type | Description |
|--------|------|-------------|
| timestamp | text | ISO 8601 timestamp when the trigger fired |
| iteration | number | Sequential run number since schedule was armed |
| input | json | The `staticInput` config value |
| schedule | text | The cron expression (for downstream logging) |

### Common Cron Patterns

| Pattern | Expression | Description |
|---------|-----------|-------------|
| Every 5 minutes | `*/5 * * * *` | Quick polling |
| Every hour | `0 * * * *` | Hourly check |
| Daily at 9am | `0 9 * * *` | Morning report |
| Weekdays at 6pm | `0 18 * * 1-5` | End-of-day summary |
| Weekly Monday 8am | `0 8 * * 1` | Weekly digest |
| First of month | `0 0 1 * *` | Monthly report |

---

## Node UI

```
+---------------------------+
|   CRON TRIGGER            |
|                           |
|   Every 5 minutes         |
|   Next: 10:05 AM          |
|   Runs: 42                |
|                           |
|      timestamp → [text]   |
|      iteration → [number] |
|          input → [json]   |
+---------------------------+
```

### Config Panel — Friendly Mode (Default)

```
┌──────────────────────────────────────┐
│  Cron Trigger Configuration          │
│                                      │
│  ┌─ Schedule ──────────────────────┐ │
│  │  Run every: [5 ▼] [minutes ▼]  │ │
│  │                                  │ │
│  │  ── OR ──                        │ │
│  │                                  │ │
│  │  Run: [Daily ▼] at [09:00 ▼]   │ │
│  └──────────────────────────────────┘ │
│                                      │
│  Timezone: [America/Chicago ▼]       │
│                                      │
│  Advanced: */5 * * * *  [edit]       │
│                                      │
│  Preview:                            │
│    Next 3: 10:05, 10:10, 10:15      │
│                                      │
│  ┌─ Options ───────────────────────┐ │
│  │  Max concurrent: [1       ]     │ │
│  │  If missed:      [Skip ▼]      │ │
│  │  Static input:   [{ }     ]     │ │
│  └──────────────────────────────────┘ │
└──────────────────────────────────────┘
```

### Config Panel — Advanced Mode

Raw cron expression editor with syntax highlighting and validation.

---

## Engine Architecture

### CronScheduler (Rust)

Lives inside the existing `TriggerManager` (from webhook implementation). Extends it with a scheduler thread.

```
TriggerManager
├── WebhookServer (Axum, :9876)        ← already built
└── CronScheduler (tokio interval)     ← NEW
    ├── armed_schedules: HashMap<trigger_id, CronSchedule>
    ├── tick every 1 second
    ├── on match: spawn workflow execution
    └── respects maxConcurrent + catchUpPolicy
```

### Tick Loop (pseudo-code)

```
loop {
    sleep(1 second)
    now = current_time_in(schedule.timezone)

    for each armed_schedule:
        if cron_matches(schedule.expression, now):
            if active_runs < schedule.max_concurrent:
                spawn execute_workflow(...)
            else:
                log "skipped: max concurrent reached"
}
```

### Missed Execution Handling

When the app starts, for each armed cron trigger:
1. Check `last_fired` timestamp from DB
2. Calculate how many executions were missed
3. Based on `catchUpPolicy`:
   - `skip`: Do nothing, resume from next match
   - `run_once`: Fire one execution immediately
   - `run_all`: Fire all missed executions sequentially (with 1s delay between), **capped at 20 runs max**. If more than 20 executions were missed, log an audit entry: `"Capped catch-up: {missed} missed, executing 20"` and skip the oldest excess.

### DST (Daylight Saving Time) Handling

DST transitions are handled by the `chrono-tz` crate automatically. Specific behaviors:
- **Spring forward** (e.g., 2:00 AM skipped): If a scheduled time falls in the skipped hour, the execution is skipped for that tick (no double-fire).
- **Fall back** (e.g., 2:00 AM repeated): The execution fires on the **first** occurrence of the ambiguous time only (no duplicate fire).
- All cron matching uses the user-configured IANA timezone. Internal storage (`last_fired`, `next_fire`) uses UTC ISO-8601.

---

## Cron Expression Parsing

Use the `cron` crate (Rust): parses standard 5-field cron expressions, provides `next()` iterator for upcoming matches, timezone-aware via `chrono-tz`.

| Crate | Version | Purpose |
|-------|---------|---------|
| `cron` | 0.13 | Cron expression parsing + matching |
| `chrono-tz` | 0.10 | IANA timezone support |

---

## TriggerManager Extensions

### New methods on existing `TriggerManager`

```rust
// Cron-specific
arm_cron(&self, trigger_id, expression, timezone, config) -> Result<()>
disarm_cron(&self, trigger_id) -> Result<()>

// The tick loop
start_cron_scheduler(&self, db, sidecar, app) -> Result<()>
stop_cron_scheduler(&self) -> Result<()>
```

### Integration with existing arm/disarm IPC

The existing `arm_trigger` and `disarm_trigger` IPC commands (from webhook implementation) already dispatch by `trigger_type`. Adding cron support means extending the match:

```rust
match trigger.trigger_type.as_str() {
    "webhook" => trigger_mgr.arm_webhook(...),
    "cron" => trigger_mgr.arm_cron(...),
    _ => Err("Unsupported trigger type"),
}
```

---

## Validation

- `cron_trigger` counts as an input source (same as `webhook_trigger`, `input`, etc)
- Max 1 cron_trigger per workflow (same rule as webhook_trigger)
- A workflow can have BOTH a cron_trigger AND a webhook_trigger (e.g., scheduled daily + manual/webhook override). Each trigger type is still limited to max 1 per workflow.
- Cron expression validated on save (reject invalid expressions)

---

## Tests (Rust unit tests)

| # | Test | What |
|---|------|------|
| 1 | Parse valid expression | `*/5 * * * *` parses without error |
| 2 | Reject invalid expression | `*/5 * * *` (4 fields) returns error |
| 3 | Next occurrence calculation | Given a time, next match is correct |
| 4 | Timezone conversion | Expression + timezone gives correct local time |
| 5 | Max concurrent enforcement | Skip when limit reached |
| 6 | Catch-up: skip policy | No runs fired for missed period |
| 7 | Catch-up: run_once policy | Exactly 1 run fired |
| 8 | Output format | Correct JSON shape with timestamp/iteration/input |
| 9 | Executor: static input passthrough | Config staticInput appears in output |
| 10 | Validation: max 1 cron trigger | Error on 2 cron nodes |

---

## Implementation Plan

### Session 1: Cron Engine + Executor (1 session)
- [ ] Add `cron = "0.13"`, `chrono-tz = "0.10"` to Cargo.toml
- [ ] Create `src/workflow/executors/cron_trigger.rs` — NodeExecutor
- [ ] Extend `TriggerManager` with CronScheduler (tick loop, arm/disarm)
- [ ] Extend `arm_trigger`/`disarm_trigger` IPC to handle `trigger_type = "cron"`
- [ ] Add validation: cron_trigger as input source, max 1, mutex with webhook_trigger
- [ ] 10 unit tests
- [ ] Register executor

### Session 2: UI (1 session)
- [ ] `CronTriggerNode.tsx` — node with friendly schedule display
- [ ] Config panel: friendly mode (dropdowns) + advanced mode (raw expression)
- [ ] Next-execution preview (calculated client-side from cron expression)
- [ ] Settings > Triggers tab: show cron schedules with next-fire time

---

## Security Considerations

| Concern | Mitigation |
|---------|-----------|
| Runaway execution | maxConcurrent limit (default 1) + budget enforcement |
| App closed → missed runs | catchUpPolicy gives user control. Default: skip (safest) |
| Timezone confusion | Always display in user's configured timezone. Store as IANA string. |
| Resource exhaustion | Minimum interval: 1 minute (reject `* * * * *` every-second patterns via 5-field cron) |
