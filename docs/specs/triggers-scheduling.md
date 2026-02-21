# Triggers & Scheduling

**Status**: PLANNED
**Phase**: 5B (production-ready)
**Priority**: P0 — critical gap for automation use cases
**Author**: AI Studio PM
**Date**: 2026-02-21

---

## Problem Statement

Workflows only execute when a user clicks "Run" or enters Live Mode. Real automation requires workflows to start automatically — on a schedule, on incoming HTTP request, on file change, or on external event. Without triggers, AI Studio is a development tool but not an automation platform.

---

## Trigger Types

### 1. Webhook Trigger Node

**Purpose**: Start a workflow when an external service sends an HTTP request.

**How it works**:
- Workflow starts with a Webhook Trigger node instead of an Input node
- When the workflow is "armed" (published), AI Studio registers a local HTTP endpoint
- External services POST to this endpoint → workflow executes with the request body as input

**Config**:
| Field | Type | Default | Description |
|-------|------|---------|-------------|
| method | enum | POST | POST / GET / PUT |
| path | string | auto-generated | URL path suffix (e.g. `/hook/my-classifier`) |
| authMode | enum | token | none / token / hmac |
| authSecret | string | — | Bearer token or HMAC secret |
| responseMode | enum | immediate | immediate (202 + run_id) / wait (block until workflow completes, return output) |
| timeout | int | 30000 | For wait mode — max ms before returning 408 |

**Output handles**:
| Handle | Type | Description |
|--------|------|-------------|
| body | json | Request body (parsed) |
| headers | json | Request headers |
| query | json | Query string parameters |
| method | text | HTTP method |

**Webhook Server Architecture**:
```
External Service ──POST──→ Tauri Webhook Server (:9876)
                                │
                        Route lookup (path → workflow_id)
                                │
                        Spawn workflow execution
                                │
                        Return 202 {run_id} (immediate)
                        — or —
                        Wait for completion, return output (wait mode)
```

- Tauri starts a lightweight HTTP server (Axum) on a configurable port when any webhook workflow is armed
- Port configurable in Settings → Triggers (default: 9876)
- Server only binds to localhost by default (security)
- Optional: expose via ngrok/cloudflare tunnel for external access (link in UI)

**UI**:
- Webhook node shows the full URL: `http://localhost:9876/hook/{path}`
- "Copy URL" button
- "Arm" / "Disarm" toggle in toolbar (replaces Run button for webhook workflows)
- Activity log: recent webhook invocations with status

**Security**:
- Token auth: request must include `Authorization: Bearer {secret}`
- HMAC auth: validate `X-Signature` header against payload + secret
- Rate limiting: configurable max requests/minute (default: 60)
- Payload size limit: 10MB

### 2. Cron Schedule Trigger

**Purpose**: Run a workflow on a time-based schedule.

**Config**:
| Field | Type | Default | Description |
|-------|------|---------|-------------|
| expression | string | required | Cron expression (`*/5 * * * *` = every 5 min) |
| timezone | string | system | IANA timezone (e.g. `America/New_York`) |
| input | json | {} | Static input data passed to workflow |
| maxConcurrent | int | 1 | Max overlapping runs (prevent pile-up) |
| enabled | bool | true | Enable/disable without deleting |

**Cron expression helper** (UI):
```
┌──────────────────────────────────────┐
│  Schedule: Every [5] [minutes]  ▼    │
│                                      │
│  Preview: Runs at :00, :05, :10...   │
│  Next 3:  10:05, 10:10, 10:15       │
│                                      │
│  Advanced: */5 * * * *  [edit]       │
└──────────────────────────────────────┘
```

- Friendly dropdowns for common patterns (every N minutes/hours/days, daily at time, weekly on day)
- "Advanced" toggle for raw cron expression
- Next 3 execution times preview

**Engine**:
- Tauri spawns a scheduler thread (tokio cron) that checks armed schedules every second
- On trigger: creates a new run, executes workflow with the static input
- Missed executions (app was closed): configurable catch-up behavior (skip / run once / run all missed)

### 3. File Watch Trigger

**Purpose**: Start a workflow when files appear or change in a watched directory.

**Config**:
| Field | Type | Default | Description |
|-------|------|---------|-------------|
| directory | string | required | Directory to watch |
| pattern | string | `*` | Glob filter |
| events | array | [created] | created / modified / deleted |
| debounce | int | 1000 | ms to wait for batch changes |
| processedAction | enum | none | none / move / tag |
| processedDir | string | — | Move-to directory |

**Output handles**:
| Handle | Type | Description |
|--------|------|-------------|
| files | json | Array of changed files `{path, name, event, modified}` |
| event | text | Trigger event type |

**Engine**: Uses `notify` crate (Rust) for cross-platform filesystem events. Debounce groups rapid changes into a single trigger.

### 4. Event Trigger (Internal)

**Purpose**: One workflow triggers another based on internal events.

**Config**:
| Field | Type | Default | Description |
|-------|------|---------|-------------|
| eventType | string | required | Event pattern to match (e.g. `workflow.completed`, `agent.message`) |
| filter | string | — | JSONPath filter on event payload |
| sourceWorkflow | string | — | Optional: only trigger from specific workflow |

**Use cases**:
- Workflow A completes → triggers Workflow B with A's output
- Agent receives a message matching a pattern → triggers a workflow
- Any workflow error → triggers an alert/notification workflow

---

## Trigger Management UI

### Settings → Triggers Tab

```
┌─────────────────────────────────────────────────────┐
│  Triggers                                            │
├─────────────────────────────────────────────────────┤
│  ● Active  ○ All  ○ Errors                          │
│                                                      │
│  ┌────────┬─────────────────┬──────────┬──────────┐ │
│  │ Type   │ Workflow        │ Schedule │ Status   │ │
│  ├────────┼─────────────────┼──────────┼──────────┤ │
│  │ Cron   │ Daily Report    │ 0 9 * *  │ ● Armed  │ │
│  │ Webhook│ Slack Notifier  │ POST /sl │ ● Armed  │ │
│  │ Watch  │ Image Processor │ ~/inbox  │ ○ Paused │ │
│  └────────┴─────────────────┴──────────┴──────────┘ │
│                                                      │
│  Webhook Server: localhost:9876  [Configure]         │
│  Next scheduled run: Daily Report in 4h 23m          │
└─────────────────────────────────────────────────────┘
```

### Workflow Toolbar Changes

When a workflow has a trigger node:
- "Run" button stays (manual test run)
- New "Arm" toggle appears: arms the trigger for automatic execution
- Status indicator: Armed (green) / Paused (yellow) / Error (red)

---

## Execution History

All trigger-initiated runs appear in the existing Runs tab with additional metadata:

| Field | Description |
|-------|-------------|
| trigger_type | webhook / cron / file_watch / event |
| trigger_data | Webhook: request info. Cron: schedule. Watch: file paths. |
| triggered_at | When the trigger fired |

This integrates with the existing Inspector — click any triggered run to see full event timeline.

---

## Data Model Changes

### New table: `triggers`

| Column | Type | Description |
|--------|------|-------------|
| id | TEXT PK | Trigger ID |
| workflow_id | TEXT FK | Associated workflow |
| trigger_type | TEXT | webhook / cron / file_watch / event |
| config | TEXT (JSON) | Type-specific configuration |
| enabled | BOOL | Active flag |
| last_fired | TEXT | ISO timestamp |
| fire_count | INT | Total invocations |
| created_at | TEXT | ISO timestamp |

### New table: `trigger_log`

| Column | Type | Description |
|--------|------|-------------|
| id | TEXT PK | Log entry ID |
| trigger_id | TEXT FK | Which trigger |
| run_id | TEXT FK | Resulting workflow run (nullable if failed) |
| fired_at | TEXT | ISO timestamp |
| status | TEXT | success / error / skipped |
| metadata | TEXT (JSON) | Request body, error message, etc. |

---

## IPC Commands

```rust
// CRUD
create_trigger(workflow_id, trigger_type, config) -> Trigger
update_trigger(trigger_id, config) -> Trigger
delete_trigger(trigger_id) -> ()
list_triggers(workflow_id?) -> Vec<Trigger>

// Lifecycle
arm_trigger(trigger_id) -> ()      // Start listening
disarm_trigger(trigger_id) -> ()   // Stop listening
test_trigger(trigger_id) -> Run    // Fire once manually

// Webhook server
get_webhook_server_status() -> {port, active_hooks, uptime}
```

---

## Implementation Plan

### Phase 1: Webhook Trigger (2 sessions)
- [ ] Rust: Axum HTTP server embedded in Tauri (configurable port)
- [ ] Rust: Trigger table + CRUD commands
- [ ] Rust: Route registration (arm/disarm)
- [ ] UI: WebhookTriggerNode.tsx with URL display + copy
- [ ] UI: Arm/Disarm toggle in workflow toolbar
- [ ] 10 tests (routing, auth, rate limiting, concurrent)

### Phase 2: Cron Scheduler (1 session)
- [ ] Rust: Tokio cron scheduler (check armed schedules)
- [ ] UI: CronTriggerNode.tsx with friendly schedule builder
- [ ] UI: Settings → Triggers tab
- [ ] 5 tests (schedule parsing, concurrent limits, missed execution)

### Phase 3: File Watch (1 session)
- [ ] Rust: `notify` crate watcher with debounce
- [ ] UI: FileWatchTriggerNode.tsx
- [ ] 5 tests (create/modify/delete events, debounce, glob filter)

### Phase 4: Event Trigger (1 session)
- [ ] Rust: Event bus subscription for internal triggers
- [ ] UI: EventTriggerNode.tsx with event type picker
- [ ] 3 tests (workflow chain, filter, source restriction)

---

## Dependencies

| Feature | Depends On | Blocks |
|---------|-----------|--------|
| Webhook Trigger | Axum server (new) | External integrations |
| Cron Scheduler | Trigger table (Phase 1) | Scheduled automation |
| File Watch | `notify` crate (new dep) | File-based automation |
| Event Trigger | Event bus (exists) | Workflow chaining |

---

## Security Considerations

- Webhook server binds to localhost only by default
- All trigger-initiated runs are logged with full audit trail
- Cron jobs respect budget limits (existing budget enforcement applies)
- File watch: same denied-paths security as File Read node
- Rate limiting on all trigger types to prevent runaway execution
