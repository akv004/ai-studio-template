# Live Workflow Execution — Spec

**Status**: IN PROGRESS
**Phase**: 4C
**Priority**: P0

## Overview

Live workflow execution enables continuous, looping execution of a workflow with configurable interval. The user clicks "Go Live" and sees a real-time scrolling feed of results — like Otter.ai for AI workflows. "Live AI security camera in 5 nodes."

## Architecture

**Not a new node type.** A workflow-level execution mode. The existing DAG engine runs in a loop with configurable interval and cooperative cancellation.

```
User clicks "Go Live"
  → Rust spawns async loop (AtomicBool cancel token)
    → Each iteration: execute_workflow() with ephemeral=true (skip DB writes)
    → Emit live_workflow_feed event per iteration (summary only)
    → Sleep interval_ms between iterations (check cancel every 100ms)
  → UI appends feed items to scrolling panel
User clicks "Stop"
  → Sets cancel flag → loop breaks within 100ms
```

## Rust Backend

### LiveWorkflowManager

State: `HashMap<String, Arc<AtomicBool>>` keyed by workflow_id, wrapped in `Arc<Mutex<>>`.

- `start(workflow_id)` → creates cancel token, inserts into map
- `stop(workflow_id)` → sets cancel flag
- `stop_all()` → sets all cancel flags (app shutdown cleanup)

### IPC Commands

#### `start_live_workflow`

Params:
- `workflow_id: String`
- `inputs: HashMap<String, Value>`
- `interval_ms: u64` (default 5000)
- `max_iterations: u64` (default 1000)
- `error_policy: String` ("skip" | "stop", default "skip")

Behavior:
1. Creates single session row (for Inspector reference)
2. Spawns `tauri::async_runtime::spawn` loop
3. Each iteration calls `execute_workflow()` with `ephemeral=true`
4. Emits `live_workflow_feed` Tauri event per iteration
5. Returns immediately with `{ liveRunId, sessionId }`
6. 5 consecutive errors → auto-stop regardless of policy

#### `stop_live_workflow`

Params: `workflow_id: String`

Sets cancel flag, loop breaks within 100ms.

### Ephemeral Execution Mode

- `ExecutionContext` gains `pub ephemeral: bool`
- `record_event()` calls guarded with `if !ctx.ephemeral`
- `emit_workflow_event()` still fires (UI needs node state visuals)

## UI

### Zustand Store

```typescript
liveMode: boolean;
liveRunId: string | null;
liveFeedItems: LiveFeedItem[];  // max 500
liveConfig: { intervalMs: number; errorPolicy: 'skip' | 'stop'; maxIterations: number };
```

### LiveFeedPanel

Collapsible bottom panel showing scrolling feed of iteration results.

Features:
- Auto-scroll when at bottom, pause on scroll up, "Jump to latest"
- Green pulsing dot when active
- Running totals: iterations, tokens, cost, elapsed
- Each row: timestamp, iteration #, output summary, duration, tokens, cost
- Error rows in red
- Max 500 items (ring buffer)

### Toolbar Controls

- "Go Live" button next to "Run" (green play icon)
- When live: transforms to "Stop" (red square icon), "Run" disabled
- Settings popover: interval slider (1-60s), error policy, max iterations

## Live Feed Events

| Event | Payload |
|---|---|
| `live.started` | `{ liveRunId, workflowId, intervalMs }` |
| `live.iteration.completed` | `{ liveRunId, iteration, timestamp, outputSummary, tokens, costUsd, durationMs }` |
| `live.iteration.error` | `{ liveRunId, iteration, timestamp, error }` |
| `live.stopped` | `{ liveRunId, totalIterations, reason }` |

## Edge Cases

- **Iteration fails**: skip policy → red feed item, continue. stop policy → halt.
- **5 consecutive errors**: auto-stop regardless of policy
- **App close during live**: `stop_all()` in window close handler
- **Slow LLM**: interval is pause *between* iterations, not total cycle time
- **DB bloat**: ephemeral=true skips all record_event() calls
- **Memory**: feed capped at 500 items

## Files

| File | Action |
|---|---|
| `docs/specs/live-workflow.md` | CREATE |
| `apps/desktop/src-tauri/src/workflow/live.rs` | CREATE |
| `apps/desktop/src-tauri/src/workflow/mod.rs` | MODIFY |
| `apps/desktop/src-tauri/src/workflow/engine.rs` | MODIFY |
| `apps/desktop/src-tauri/src/workflow/executors/mod.rs` | MODIFY |
| `apps/desktop/src-tauri/src/lib.rs` | MODIFY |
| `apps/ui/src/state/store.ts` | MODIFY |
| `apps/ui/src/app/pages/workflow/LiveFeedPanel.tsx` | CREATE |
| `apps/ui/src/app/pages/workflow/WorkflowCanvas.tsx` | MODIFY |
