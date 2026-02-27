# Scheduler Node + Workflow UX + Demo Templates

**Status**: DRAFT — pending peer review
**Phase**: 4C (automation canvas)
**Priority**: P0 — critical gap for scheduled automation + workflow management UX
**Author**: AI Studio PM
**Date**: 2026-02-26
**Related specs**: `cron-trigger.md` (engine detail), `triggers-scheduling.md` (overview), `automation-demo-template.md` (demo scripts)

---

## Problem Statement

Three gaps prevent AI Studio from being a real automation platform:

1. **No time-based scheduling.** Users can't say "run this at 3pm Friday" or "every morning at 9am." The Cron Trigger spec exists but isn't built. Users must click Run manually or use external cron + webhook — defeating the "integrated IDE" value prop.

2. **Workflow List is view-only.** To run a workflow, you must open it first. No status indicators, no schedule badges, no quick-run. For an automation platform with 10+ workflows, this is unusable.

3. **No end-to-end demo template.** We have 17 templates but none that showcase the full automation loop: trigger → process → notify. The "Code Change Analyzer" and "Daily AI Report" templates are specced but not built.

This spec addresses all three as a cohesive feature set.

---

## Part 1: Cron Trigger Node (22nd node type)

> Full engine spec in `cron-trigger.md`. This section covers what's needed to build it.

### Node Type: `cron_trigger`

Category: **Triggers** (alongside Webhook Trigger)

### Canvas Node

```
┌─────────────────────────────┐
│  ⏰ CRON TRIGGER            │
│                             │
│  Daily at 9:00 AM           │
│  Timezone: America/Chicago  │
│  Next: Tomorrow 9:00 AM     │
│  Runs: 42                   │
│                             │
│       timestamp ──→         │
│       iteration ──→         │
│           input ──→         │
│        schedule ──→         │
└─────────────────────────────┘
```

- Icon: `Clock` (lucide) — distinguish from Webhook's `Zap`
- Header: schedule summary in human-readable form
- "Next" line: computed from cron expression + timezone (client-side via `cronstrue` npm package)
- "Runs" counter: from trigger DB `fire_count`
- No input handles (source node)
- 4 output handles: timestamp, iteration, input, schedule

### Config Panel

Two modes: **Friendly** (default) and **Advanced**.

**Friendly mode:**
```
┌──────────────────────────────────────────┐
│  Schedule                                │
│                                          │
│  Frequency: [Daily ▼]                   │
│  At: [09:00 ▼]                          │
│                                          │
│  Timezone: [America/Chicago ▼]          │
│                                          │
│  ── Preview ──                           │
│  Expression: 0 9 * * *                   │
│  Next 3 runs:                            │
│    Wed Feb 27, 9:00 AM                   │
│    Thu Feb 28, 9:00 AM                   │
│    Fri Mar 01, 9:00 AM                   │
│                                          │
│  [Switch to Advanced]                    │
│                                          │
│  ── Options ──                           │
│  Max concurrent runs: [1   ]             │
│  If app was closed:   [Skip missed ▼]   │
│  Static input JSON:   [{ }          ]   │
└──────────────────────────────────────────┘
```

**Friendly frequency presets:**

| Preset | Cron | UI fields shown |
|--------|------|-----------------|
| Every N minutes | `*/N * * * *` | Interval: [N] minutes |
| Hourly | `0 * * * *` | At minute: [0] |
| Daily | `0 H * * *` | At time: [HH:MM] |
| Weekdays | `0 H * * 1-5` | At time: [HH:MM] |
| Weekly | `0 H * * D` | Day: [Mon-Sun], At: [HH:MM] |
| Monthly | `0 H D * *` | Day of month: [1-28], At: [HH:MM] |
| Custom | raw | [Switch to Advanced] |

**Advanced mode:**
- Raw cron expression text input with 5-field syntax hint
- Real-time validation (green check / red error)
- `cronstrue` renders human-readable description below
- Next 3 execution times preview

**Timezone dropdown:**
- Top entries: UTC, user's system timezone (auto-detected), `America/Chicago`, `Asia/Kolkata`
- Full IANA list below, searchable

### Toolbar Integration

When a workflow contains a `cron_trigger` node, the toolbar shows:

```
... | [Run▶] [Arm Schedule ⏰ / Disarm ■] [Go Live◉] [⚙] [⚡Webhook] |
```

- **Arm Schedule** button: Arms the cron trigger. Shows pulsing green clock icon when armed.
- **Disarm** button: Stops the schedule. Red square icon.
- Run button stays (manual test run, ignores schedule)
- Can coexist with Webhook Arm (both can be armed simultaneously)

### Arm/Disarm Behavior

**Arm:**
1. Validate cron expression
2. Call `arm_trigger` IPC (existing command, extended for cron type)
3. CronScheduler registers the schedule
4. Node shows "Armed" badge, toolbar shows green indicator
5. `next_fire` calculated and stored in DB

**Disarm:**
1. Call `disarm_trigger` IPC
2. CronScheduler removes the schedule
3. Node shows "Paused" badge

**App restart:**
1. On Tauri startup, load all armed triggers from DB
2. For each `cron` trigger: re-arm in CronScheduler
3. Apply `catchUpPolicy` for missed executions

### Executor: `CronTriggerExecutor`

Minimal — just formats the trigger context as output:

```rust
fn execute(&self, node, inputs, config) -> Result<NodeOutput> {
    // inputs come from the CronScheduler when it fires
    Ok(json!({
        "timestamp": inputs.get("_trigger_timestamp"),
        "iteration": inputs.get("_trigger_iteration"),
        "input": config.static_input,
        "schedule": config.expression,
    }))
}
```

The CronScheduler injects `_trigger_timestamp` and `_trigger_iteration` as synthetic inputs when spawning the workflow.

---

## Part 2: Workflow List UX Upgrades

### Current State (Problems)

```
┌─ Workflows ──────────────────────────────────┐
│ [↻] [↑Import] | Templates [▼] | [+ New]     │
├──────────────────────────────────────────────┤
│ ┌─ Card ──────────────────────────────────┐  │
│ │ My Workflow                [✏] [📋] [🗑] │  │
│ │ Description                              │  │
│ │ 8 nodes                                  │  │
│ └──────────────────────────────────────────┘  │
└──────────────────────────────────────────────┘
```

Problems:
- No way to run from list — must open each workflow
- No status: is it running? scheduled? idle? erroring?
- No schedule info: when does it run next?
- No last-run info: did it succeed? when?
- Cards are static — no operational awareness

### Proposed Design

```
┌─ Workflows ──────────────────────────────────────────────────────────┐
│ [↻] [↑Import] | Templates [▼] | [+ New]     ● Active  ○ All       │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│ ┌─────────────────────────────────────────────────────────────────┐  │
│ │ ● Daily AI Report                              [▶Run] [✏] [🗑] │  │
│ │ Fetch data, AI summarize, email team                            │  │
│ │ 5 nodes │ ⏰ Daily 9:00 AM │ Next: Tomorrow 9:00 AM            │  │
│ │ Last run: Today 9:00 AM — ✓ Success (2.3s)                    │  │
│ └─────────────────────────────────────────────────────────────────┘  │
│                                                                      │
│ ┌─────────────────────────────────────────────────────────────────┐  │
│ │ ◉ Code Change Analyzer                    [■Stop] [✏] [🗑]    │  │
│ │ Webhook-triggered code review pipeline                          │  │
│ │ 9 nodes │ ⚡ Webhook /code-review │ Armed                       │  │
│ │ Last run: 10 min ago — ✓ Success (4.1s) │ 23 total runs       │  │
│ └─────────────────────────────────────────────────────────────────┘  │
│                                                                      │
│ ┌─────────────────────────────────────────────────────────────────┐  │
│ │ ○ Research Assistant                           [▶Run] [✏] [🗑] │  │
│ │ Research a topic and produce a formatted report                 │  │
│ │ 4 nodes │ Manual                                                │  │
│ │ Last run: 3 days ago — ✓ Success                               │  │
│ └─────────────────────────────────────────────────────────────────┘  │
│                                                                      │
│ ┌─────────────────────────────────────────────────────────────────┐  │
│ │ ○ Data Pipeline                                [▶Run] [✏] [🗑] │  │
│ │ Extract structured data from raw input                          │  │
│ │ 3 nodes │ Manual                                                │  │
│ │ No runs yet                                                     │  │
│ └─────────────────────────────────────────────────────────────────┘  │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

### Card Anatomy

```
┌──────────────────────────────────────────────────────────┐
│ [status dot] [Name]                      [Actions]       │ ← Row 1: identity + actions
│ [Description]                                            │ ← Row 2: description
│ [N nodes] │ [trigger info] │ [schedule info]             │ ← Row 3: metadata
│ [Last run: time — status (duration)] │ [total runs]      │ ← Row 4: operational
└──────────────────────────────────────────────────────────┘
```

**Status dot (left of name):**

| State | Dot | Meaning |
|-------|-----|---------|
| Idle | `○` gray | No triggers armed, not running |
| Scheduled | `●` blue | Cron trigger armed, waiting for next fire |
| Armed | `◉` yellow | Webhook armed, waiting for requests |
| Running | `◉` green pulsing | Currently executing |
| Error | `●` red | Last run failed |

**Actions (right side):**

| Button | When | Action |
|--------|------|--------|
| `▶ Run` | Not running | Quick-run with default inputs (no modal). If workflow has Input nodes with no defaults, opens mini-modal. |
| `■ Stop` | Running or Live | Stop current execution |
| `✏` | Always | Open in canvas editor |
| `📋` | Hover | Duplicate |
| `🗑` | Hover | Delete (with confirmation) |

**Trigger info (row 3):**

| Trigger type | Display |
|---|---|
| None (manual only) | `Manual` |
| Cron armed | `⏰ Daily 9:00 AM` (human-readable) |
| Cron paused | `⏰ Daily 9:00 AM (paused)` dimmed |
| Webhook armed | `⚡ Webhook /path` |
| Webhook + Cron | `⏰ Daily 9AM │ ⚡ /path` |
| Live mode | `◉ Live (5s interval)` |

**Last run info (row 4):**

| State | Display |
|---|---|
| Never run | `No runs yet` (dimmed) |
| Success | `Last run: Today 9:00 AM — ✓ Success (2.3s)` |
| Failed | `Last run: Today 9:05 AM — ✗ Failed: SMTP timeout` (red) |
| Running now | `Running... 1.2s elapsed` (green, animated) |

### Filter Tabs

```
● Active (3)  ○ All (12)  ○ Scheduled (2)  ○ Errors (1)
```

| Tab | Shows |
|-----|-------|
| Active | Running + Armed + Scheduled (not idle) |
| All | Everything |
| Scheduled | Only workflows with armed cron triggers |
| Errors | Last run failed |

### Quick Run from List

When user clicks `▶ Run` on a card:

1. Check if workflow has Input nodes with no default values
2. **If all inputs have defaults** (or no Input nodes): run immediately with defaults, show toast "Running..."
3. **If inputs need values**: show a compact modal (same fields as canvas Run modal, but smaller)
4. On completion: update card's "Last run" row in-place, show toast with result
5. Click the toast or "Last run" link → opens Inspector for that session

### Data Requirements (Rust → UI)

The workflow list needs additional data that doesn't currently flow to the UI:

```typescript
interface WorkflowListItem {
    // Existing
    id: string;
    name: string;
    description: string;
    graphJson: string;
    createdAt: string;
    updatedAt: string;

    // NEW — populated by join queries
    nodeCount: number;           // COUNT from graph JSON (computed)
    triggerType: string | null;  // 'cron' | 'webhook' | null
    triggerEnabled: boolean;     // Is trigger armed?
    triggerConfig: object | null; // Cron expression, webhook path, etc
    nextFireAt: string | null;   // ISO timestamp of next scheduled run
    lastRunAt: string | null;    // When last run started
    lastRunStatus: string | null; // 'completed' | 'failed' | null
    lastRunDuration: number | null; // ms
    lastRunError: string | null;  // Error message if failed
    totalRuns: number;           // COUNT of runs for this workflow
    isRunning: boolean;          // Currently executing?
}
```

New IPC command:
```rust
#[tauri::command]
async fn list_workflows_with_status(db: State<Db>) -> Result<Vec<WorkflowListItem>>
```

This replaces the simple `list_workflows` with a richer query that joins `workflows` + `triggers` + `workflow_runs` tables.

---

## Part 3: Demo Templates (3 new templates)

### Template #18: "Daily AI Report" (Cron + HTTP + LLM + Email)

**Story**: Every morning at 9am, fetch data from an API, have AI summarize it, and email the report.

**Graph:**
```
Cron Trigger ──→ HTTP Request ──→ LLM ──→ Email Send ──→ Output
 (daily 9am)     (GET data)      (summarize)  (distribute)  (archive)
```

**Nodes:**

| # | Node ID | Type | Label | Config |
|---|---------|------|-------|--------|
| 1 | cron-1 | cron_trigger | Daily Schedule | expression: `0 9 * * *`, timezone: `America/Chicago`, staticInput: `{}` |
| 2 | http-1 | http_request | Fetch Data | method: GET, url: `https://jsonplaceholder.typicode.com/posts?_limit=5` (demo API) |
| 3 | llm-1 | llm | Summarize | provider: azure_openai, model: gpt-4o-mini, temp: 0.2, prompt: "Summarize the following data into a concise daily briefing with key highlights and action items:\n\n{{http-1.output}}" |
| 4 | email-1 | email_send | Email Report | smtpHost: `(configure)`, smtpPort: 587, encryption: tls, fromAddress: `(configure)`, bodyType: plain |
| 5 | output-1 | output | Archive | — |

**Edges:**
- cron-1 → http-1 (timestamp → url, passthrough trigger context)
- http-1 → llm-1 (output → input)
- llm-1 → email-1 (output → body)
- llm-1 → email-1 (output → subject, via Transform or static: "Daily AI Report — {{cron-1.timestamp}}")
- email-1 → output-1 (output → input)

**Input handles for email:**
- `to`: user configures in Email Send config (e.g., "team@company.com")
- `subject`: "Daily AI Report"
- `body`: LLM output (connected)

**Why this is demoable:**
- Shows time-based automation (no clicking Run)
- Shows external data integration (HTTP)
- Shows AI processing (LLM)
- Shows real-world output (email delivery)
- 5 nodes — simple enough to explain in 30 seconds

---

### Template #19: "Code Change Analyzer" (Webhook + RAG + LLM + Router + Approval + Email)

**Story**: CI/CD pushes code changes via webhook. AI Studio looks up team coding standards (RAG), analyzes the diff, routes by severity, and emails results — with human approval for critical changes.

**Graph:**
```
Webhook Trigger ──→ Transform ──→ Knowledge Base ──→ LLM ──→ Router
   (POST /code-review)  (parse)     (standards)      (analyze)  (severity?)
                                                                   │
                                                        ┌──────────┤
                                                        ▼          ▼
                                                    Approval    Email Send
                                                    (review)    (summary)
                                                        │          │
                                                        ▼          ▼
                                                    Email Send   Output
                                                    (urgent)     (report)
                                                        │
                                                        ▼
                                                      Output
                                                      (report)
```

**Nodes:**

| # | Node ID | Type | Label | Config |
|---|---------|------|-------|--------|
| 1 | webhook-1 | webhook_trigger | CI/CD Hook | path: `/code-review`, method: POST, authMode: hmac |
| 2 | transform-1 | transform | Parse Payload | mode: jsonpath, expression: `$.body` (extract webhook body) |
| 3 | kb-1 | knowledge_base | Standards | folderPath: `(configure)`, chunkStrategy: recursive, chunkSize: 500, topK: 5 |
| 4 | llm-1 | llm | Code Analyzer | provider: azure_openai, model: gpt-4o-mini, temp: 0.2, prompt: (see below) |
| 5 | router-1 | router | Severity? | branches: ["critical", "normal"], prompt: "Based on the analysis, is this critical or normal?" |
| 6 | approval-1 | approval | Review Gate | prompt: "Critical code change detected. Review the analysis and approve or reject." |
| 7 | email-urgent | email_send | Urgent Alert | subject: "CRITICAL: Code review needs attention" |
| 8 | email-summary | email_send | Summary | subject: "Code review complete" |
| 9 | output-1 | output | Report | — |
| 10 | output-2 | output | Report | — |

**LLM Prompt (node 4):**
```
You are a senior code reviewer. Analyze this code change against our team's coding standards.

## Code Change
{{transform-1.output}}

## Team Standards (from knowledge base)
{{kb-1.output}}

Rate the severity: CRITICAL or NORMAL.
- CRITICAL: security vulnerabilities, breaking changes, data loss risks
- NORMAL: style issues, minor refactors, documentation gaps

Provide:
1. Severity: CRITICAL or NORMAL
2. Summary (2-3 sentences)
3. Specific violations found
4. Recommendations
```

**Why this is demoable:**
- **9 nodes** — impressive visual complexity
- Uses 7 different node types (webhook, transform, KB, LLM, router, approval, email)
- Shows human-in-the-loop (approval gate)
- Shows RAG (knowledge base lookup)
- Shows conditional routing (critical vs normal paths)
- Realistic enterprise use case
- Can demo with a curl command — no external setup needed

---

### Template #20: "Smart Alert Pipeline" (Cron + HTTP + LLM + Loop + Router + Email)

**Story**: Every hour, check a monitoring endpoint. If issues detected, AI triages severity. Loop refines the analysis until stable. Route critical alerts via email, log normal ones.

**Graph:**
```
Cron Trigger ──→ HTTP Request ──→ LLM ──→ Loop ──→ Router ──→ Email Send
 (hourly)        (health check)   (triage)  (refine)  (critical?)   (alert)
                                                          │
                                                          ▼
                                                        Output
                                                        (log)
```

**Nodes:**

| # | Node ID | Type | Label | Config |
|---|---------|------|-------|--------|
| 1 | cron-1 | cron_trigger | Hourly Check | expression: `0 * * * *`, timezone: `America/Chicago` |
| 2 | http-1 | http_request | Health Check | method: GET, url: `https://jsonplaceholder.typicode.com/todos?_limit=3` (demo) |
| 3 | llm-1 | llm | Triage | provider: azure_openai, model: gpt-4o-mini, temp: 0.2, prompt: "Analyze this system health data. Identify any issues, rate each as critical/warning/ok. Provide a concise triage summary." |
| 4 | loop-1 | loop | Refine Analysis | maxIterations: 3, exitCondition: stable_output, stabilityThreshold: 0.9, feedbackMode: replace |
| 5 | llm-2 | llm | Refiner | provider: azure_openai, model: gpt-4o-mini, temp: 0.1, prompt: "Review and refine this triage analysis. Make it more precise and actionable. If nothing to improve, return it unchanged." |
| 6 | exit-1 | exit | Done | — |
| 7 | router-1 | router | Critical? | branches: ["critical", "normal"], prompt: "Does this analysis contain any critical issues?" |
| 8 | email-1 | email_send | Alert Email | subject: "ALERT: System issues detected", bodyType: plain |
| 9 | output-1 | output | Log | — |
| 10 | output-2 | output | Log | — |

**Why this is demoable:**
- Shows **scheduled monitoring** (cron every hour)
- Shows **iterative refinement** (Loop node — unique differentiator)
- Shows **conditional alerting** (Router + Email only for critical)
- Uses 8 different node types
- Practical DevOps/SRE use case

---

## Part 4: Implementation Plan

### Session 1: Cron Engine + Executor (Rust)
- [ ] Add `cron = "0.13"`, `chrono-tz = "0.10"` to Cargo.toml
- [ ] Create `src/workflow/executors/cron_trigger.rs`
- [ ] Extend `TriggerManager` with `CronScheduler` (tick loop, arm/disarm)
- [ ] Extend `arm_trigger`/`disarm_trigger` for `trigger_type = "cron"`
- [ ] Validation: max 1 cron_trigger, coexists with webhook
- [ ] 10 unit tests (from cron-trigger.md test plan)
- [ ] Register executor in engine

### Session 2: Cron Node UI + Config Panel
- [ ] Add `cronstrue` npm package (cron → human-readable)
- [ ] `CronTriggerNode.tsx` — Clock icon, schedule summary, 4 output handles
- [ ] Config panel: friendly mode (presets) + advanced mode (raw expression)
- [ ] Next-execution preview (client-side)
- [ ] Toolbar: Arm Schedule / Disarm button (alongside existing Webhook arm)
- [ ] Node palette: add to "Triggers" category

### Session 3: Workflow List UX Upgrades
- [ ] `list_workflows_with_status` IPC command (join workflows + triggers + runs)
- [ ] Redesign `WorkflowList.tsx` cards with status dot, trigger info, last run
- [ ] Quick Run button on cards (run with defaults or mini-modal)
- [ ] Filter tabs: Active / All / Scheduled / Errors
- [ ] Real-time status updates via Tauri events (running → completed)

### Session 4: Demo Templates + Polish
- [ ] Template #18: "Daily AI Report" (cron + HTTP + LLM + email)
- [ ] Template #19: "Code Change Analyzer" (webhook + RAG + LLM + router + approval + email)
- [ ] Template #20: "Smart Alert Pipeline" (cron + HTTP + LLM + loop + router + email)
- [ ] Playwright E2E: cron node on canvas, list status display
- [ ] Peer review prompt for Gemini + Codex

### Session 5 (optional): Settings > Triggers Tab
- [ ] Unified trigger management page in Settings
- [ ] All armed triggers with status, next fire, fire count
- [ ] Arm/disarm from Settings (without opening workflow)
- [ ] Trigger execution log (last 50 runs per trigger)

---

## Dependencies

| Feature | Depends On | Status |
|---------|-----------|--------|
| Cron Trigger engine | TriggerManager (webhook) | DONE |
| Cron Trigger executor | Engine executor registry | DONE |
| Cron Trigger UI | Node palette system | DONE |
| Workflow List upgrades | `list_workflows` IPC | DONE (needs extension) |
| Quick Run from list | `run_workflow` IPC | DONE |
| Template #18 (Daily AI Report) | Cron Trigger node | Session 1-2 |
| Template #19 (Code Change) | Webhook Trigger UI, Email Send, KB | All DONE |
| Template #20 (Smart Alert) | Cron Trigger, Loop, Email Send | Session 1-2 |

---

## New Crate Dependencies

| Crate | Version | Size | Purpose |
|-------|---------|------|---------|
| `cron` | 0.13 | ~50KB | Cron expression parsing |
| `chrono-tz` | 0.10 | ~200KB | IANA timezone data |

| npm Package | Version | Purpose |
|-------------|---------|---------|
| `cronstrue` | ^2.50 | Cron → human-readable text |

---

## Data Model Changes

### Existing `triggers` table (from webhook implementation)

No schema changes needed — cron triggers use the same table:

```sql
-- trigger_type = 'cron' uses config JSON:
{
  "expression": "0 9 * * *",
  "timezone": "America/Chicago",
  "staticInput": {},
  "maxConcurrent": 1,
  "catchUpPolicy": "skip"
}
```

### New: `workflow_runs` summary query

For the list page, we need a lightweight query:

```sql
SELECT
    w.id, w.name, w.description, w.graph_json, w.updated_at,
    t.trigger_type, t.enabled as trigger_enabled, t.config as trigger_config,
    t.next_fire, t.fire_count,
    (SELECT COUNT(*) FROM sessions s WHERE s.workflow_id = w.id) as total_runs,
    latest.started_at as last_run_at,
    latest.status as last_run_status,
    latest.duration_ms as last_run_duration,
    latest.error as last_run_error
FROM workflows w
LEFT JOIN triggers t ON t.workflow_id = w.id AND t.enabled = 1
LEFT JOIN LATERAL (
    SELECT started_at, status, duration_ms, error
    FROM sessions WHERE workflow_id = w.id
    ORDER BY started_at DESC LIMIT 1
) latest ON 1=1
ORDER BY w.updated_at DESC
```

(SQLite doesn't support LATERAL — will use correlated subquery or CTE instead.)

---

## Security Considerations

| Concern | Mitigation |
|---------|-----------|
| Runaway cron execution | maxConcurrent (default 1) + budget enforcement |
| Quick Run from list with sensitive inputs | Only auto-runs if all Input nodes have defaults. Otherwise shows modal. |
| Missed runs catch-up flood | catchUpPolicy default: skip. `run_all` capped at 20. |
| Multiple armed triggers | Each trigger type limited to 1 per workflow. Both can coexist. |
| Resource exhaustion | Minimum cron interval: 1 minute (5-field cron enforces this) |

---

## Success Criteria

1. User can drag Cron Trigger onto canvas, set "Daily at 9am", arm it, and the workflow runs automatically
2. Workflow List shows status dots, trigger info, and last run status for all workflows
3. User can click Run on any workflow card without opening the canvas
4. All 3 demo templates work end-to-end with Azure OpenAI + SMTP
5. A stakeholder can watch the "Daily AI Report" template demo in under 60 seconds and understand the value
