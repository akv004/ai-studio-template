# Step-Through Debugging — Interactive AI Workflow Debugger

**Status**: DRAFT — pending peer review
**Phase**: 5A (killer feature)
**Priority**: P0 — primary differentiator, no competitor has this
**Author**: AI Studio PM
**Date**: 2026-02-27
**Tagline**: "F10 for AI workflows"

---

## Problem Statement

Every visual AI workflow tool (Dify, LangFlow, Flowise, n8n) is "run and pray." You build a pipeline, click Run, and hope the output is correct. When it's wrong — and it will be wrong — you have zero ability to:

1. **Pinpoint the failure node** — Was it the LLM prompt? The RAG retrieval? The router logic?
2. **Inspect intermediate data** — What did the Knowledge Base actually return? What did the LLM see as input?
3. **Test fixes without restarting** — Change one prompt, re-run one node, see if it's better
4. **Understand data flow** — How does data transform as it moves through 10 nodes?

Every developer already knows how to debug with breakpoints, step-over, watch expressions. **Nobody has applied this to AI workflows.** AI Studio's Inspector already captures per-node events — step-through debugging is the natural evolution that makes it interactive.

### Why this wins

- Zero learning curve — every developer knows debugger UX
- Solves the #1 pain in AI workflows (black-box failures)
- 30-second demo that makes people go "I need this"
- Builds on existing Inspector + Approval node patterns
- No competitor has anything close

---

## User Experience

### Setting Breakpoints

Right-click any node on the canvas → "Toggle Breakpoint". A red dot appears on the node corner (like VS Code).

```
┌──────────────────────┐
│ 🔴 LLM · Analyzer    │     ← red dot = breakpoint
│ azure_openai / gpt-4o│
│ Ready                 │
│         output ──→    │
└──────────────────────┘
```

Breakpoints can also be set via:
- Config panel toggle: "Break before execution"
- Keyboard shortcut: select node → `B` key
- Conditional breakpoints (Phase 2): break only when `{{node.output}}` contains "error"

### Debug Toolbar

When at least one breakpoint is set, the Run button gets a companion:

```
[▶ Run]  [🐛 Debug]  [Go Live ◉]  [⚙]
```

Clicking **Debug** starts execution in debug mode. Execution proceeds normally until it hits a breakpoint.

### Paused at Breakpoint

When execution reaches a breakpoint node, everything pauses. The canvas highlights the paused node with a pulsing yellow border. A **Debug Panel** appears below the canvas (or docked to the right):

```
┌─ Debug Panel ────────────────────────────────────────────────┐
│                                                              │
│  ⏸ Paused at: LLM · Analyzer          Step 4 of 8          │
│                                                              │
│  ┌─ Input ─────────────────────────────────────────────────┐ │
│  │ {                                                       │ │
│  │   "messages": [                                         │ │
│  │     {"role": "system", "content": "You are a senior..."}│ │
│  │     {"role": "user", "content": "Review this PR:\n..."}│ │
│  │   ],                                                    │ │
│  │   "context": "## Team Standards\n1. Always use..."      │ │
│  │ }                                                       │ │
│  └─────────────────────────────────────────────────────────┘ │
│                                                              │
│  ┌─ Node Config (read-only) ───────────────────────────────┐ │
│  │ Provider: azure_openai  Model: gpt-4o  Temp: 0.2       │ │
│  │ Prompt: "You are a senior code reviewer..."             │ │
│  └─────────────────────────────────────────────────────────┘ │
│                                                              │
│  ┌─ Previous Nodes ───────────────────────────────────────┐  │
│  │ ✓ Input         → "Review this PR: diff --git..."      │  │
│  │ ✓ Transform     → { body: "diff --git a/auth.rs..." }  │  │
│  │ ✓ Knowledge Base → [3 chunks, best: 0.87]              │  │
│  └────────────────────────────────────────────────────────┘  │
│                                                              │
│  [▶ Continue]  [⏭ Step Over]  [⏩ Step Out]  [⏹ Stop]       │
│  [✏ Edit Input]  [↩ Re-run Previous]  [⏭ Skip Node]        │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

### Debug Actions

| Action | Shortcut | What happens |
|--------|----------|-------------|
| **Continue (F5)** | `F5` | Resume execution until next breakpoint or end |
| **Step Over (F10)** | `F10` | Execute this node, pause at the next node |
| **Step Out** | `Shift+F10` | Execute remaining nodes in this branch, pause at merge point |
| **Stop (Shift+F5)** | `Shift+F5` | Abort execution entirely |
| **Edit Input** | `E` | Open editable JSON view of node's input. Modify, then Step Over with new input. |
| **Edit Output** | — | After Step Over, edit the node's output before continuing. "What if the LLM said X?" |
| **Re-run Node** | `R` | Re-execute the current node (useful after editing prompt in config panel) |
| **Skip Node** | `S` | Skip this node entirely, use a manually provided output value instead |

### After Step Over — Inspecting Output

After stepping over a node, the Debug Panel shows the result:

```
┌─ Debug Panel ────────────────────────────────────────────────┐
│                                                              │
│  ✓ Completed: LLM · Analyzer       1.8s · $0.003 · 1.2K tok│
│                                                              │
│  ┌─ Output ────────────────────────────────────────────────┐ │
│  │ "Severity: CRITICAL\n\nThe PR introduces a SQL         │ │
│  │ injection vulnerability in auth.rs line 42..."          │ │
│  └─────────────────────────────────────────────────────────┘ │
│                                                              │
│  [✏ Edit Output]  [↩ Re-run]                                │
│  [▶ Continue]  [⏭ Step Over]  [⏹ Stop]                      │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

The user can now:
1. **Accept and continue** — output looks good, proceed
2. **Edit output** — change "CRITICAL" to "NORMAL", see how the Router behaves
3. **Re-run** — tweak the prompt in config panel, re-run this node with same input

### Edit & Continue

This is the "holy shit" moment. User pauses at the Router node, sees the LLM output fed into it. The Router chose "critical" branch. User wonders: "What if the analysis said NORMAL?"

1. Click **Edit Output** on the LLM node (previous node)
2. Change "Severity: CRITICAL" → "Severity: NORMAL"
3. Click **Continue** — Router now takes the "normal" branch
4. User sees the entire downstream behavior change in real-time

This is **interactive prompt debugging**. No restart, no re-running the expensive LLM call, no guessing.

### Watch Expressions

Pin values to monitor across the entire execution:

```
┌─ Watch ──────────────────────────────────┐
│ {{kb-1.output.results[0].score}}  → 0.87 │
│ {{llm-1.output}}                  → ...  │
│ {{router-1.branch}}               → —    │ (not yet executed)
│ + Add watch expression                   │
└──────────────────────────────────────────┘
```

Watch values update as execution progresses through nodes.

---

## Architecture

### Debug Mode in DAG Walker

The existing DAG walker in `workflow/engine.rs` executes nodes in topological order. Debug mode adds a pause-check before each node.

```rust
// In execute_workflow / execute_dag
for node_id in &execution_order {
    // NEW: Check if debug mode + breakpoint
    if debug_mode {
        if has_breakpoint(node_id, &node_map) || step_mode == StepMode::StepOver {
            // Emit debug.paused event with node input data
            emit_debug_event(app, session_id, "debug.paused", json!({
                "node_id": node_id,
                "input": &resolved_input,
                "node_config": &node_data,
                "completed_nodes": &node_outputs,
                "step": current_step,
                "total_steps": execution_order.len(),
            }));

            // Wait for debug command (continue/step/edit/skip/stop)
            let command = debug_receiver.recv().await?;
            match command {
                DebugCommand::Continue => { step_mode = StepMode::Continue; }
                DebugCommand::StepOver => { step_mode = StepMode::StepOver; }
                DebugCommand::EditInput(new_input) => { resolved_input = new_input; }
                DebugCommand::EditOutput(output) => {
                    node_outputs.insert(node_id, output);
                    continue; // Skip execution, use provided output
                }
                DebugCommand::Skip(output) => {
                    node_outputs.insert(node_id, output);
                    continue;
                }
                DebugCommand::Stop => { return Err("Debug: execution stopped by user"); }
            }
        }
    }

    // ... existing node execution logic ...

    // NEW: After execution, if step mode, pause with output
    if debug_mode && step_mode == StepMode::StepOver {
        emit_debug_event(app, session_id, "debug.node_completed", json!({
            "node_id": node_id,
            "output": &node_outputs[node_id],
            "duration_ms": duration,
            "cost": cost,
        }));
        // Wait for next command (edit output / continue / step)
        let command = debug_receiver.recv().await?;
        // ... handle post-execution commands ...
    }
}
```

### Communication: IPC Channel

Debug commands flow through a dedicated channel, similar to the Approval pattern:

```
UI                    Rust Engine
│                         │
│  run_workflow_debug()   │
│ ───────────────────────→│
│                         │ (executes until breakpoint)
│  debug.paused event     │
│ ←───────────────────────│
│                         │
│  (user clicks Step Over)│
│                         │
│  debug_command(StepOver)│
│ ───────────────────────→│
│                         │ (executes one node)
│  debug.node_completed   │
│ ←───────────────────────│
│                         │
│  debug_command(Continue) │
│ ───────────────────────→│
│                         │ (runs to completion)
│  workflow.completed      │
│ ←───────────────────────│
```

### New Types

```rust
#[derive(Debug, Clone)]
pub enum DebugCommand {
    Continue,           // Resume until next breakpoint
    StepOver,           // Execute one node, pause
    StepOut,            // Execute until branch merge
    Stop,               // Abort execution
    EditInput(Value),   // Replace current node's input
    EditOutput(Value),  // Skip execution, inject this output
    Skip(Value),        // Skip node, use provided value
    RerunNode,          // Re-execute current node
}

#[derive(Debug, Clone, PartialEq)]
pub enum StepMode {
    Continue,   // Run until breakpoint
    StepOver,   // Pause after every node
}
```

### New IPC Commands

```rust
#[tauri::command]
async fn run_workflow_debug(
    db: State<Db>, sidecar: State<Sidecar>,
    workflow_id: String, inputs: Value,
    breakpoints: Vec<String>,  // node IDs with breakpoints
) -> Result<String, String>

#[tauri::command]
async fn debug_command(
    session_id: String,
    command: String,       // "continue" | "step_over" | "step_out" | "stop" | "edit_input" | "edit_output" | "skip" | "rerun"
    payload: Option<Value>, // For edit_input/edit_output/skip: the new value
) -> Result<(), String>
```

### New Events

| Event | When | Payload |
|-------|------|---------|
| `debug.paused` | Execution paused at breakpoint/step | node_id, input, config, completed_nodes, step/total |
| `debug.node_completed` | Node finished in step mode | node_id, output, duration_ms, cost |
| `debug.resumed` | Execution continuing | command type |
| `debug.input_edited` | User modified node input | node_id, original, modified |
| `debug.output_edited` | User modified node output | node_id, original, modified |
| `debug.stopped` | User aborted debug session | node_id (where stopped) |

---

## UI Components

### DebugPanel.tsx

Docked below the canvas (like browser DevTools). Three tabs:

1. **Context** — Current node input, config, output (after step)
2. **Watch** — Pinned expressions with live values
3. **Call Stack** — List of completed nodes with expandable input/output

### Breakpoint Indicators

- **Red dot** on node corner (top-left) — breakpoint set
- **Yellow pulsing border** — paused at this node
- **Green checkmark** — completed in this debug session
- **Gray** — not yet reached
- **Dimmed** — will be skipped (downstream of a skip/edit)

### Debug Toolbar (replaces standard toolbar during debug)

```
[▶ Continue F5]  [⏭ Step F10]  [⏩ Out ⇧F10]  [⏹ Stop ⇧F5]  │  Step 4/8  │  ⏱ 3.2s  │  💰 $0.006
```

### Canvas Enhancements

During debug mode:
- Completed edges glow green (data flowed through)
- Current node has yellow pulsing border
- Future nodes are slightly dimmed
- Clicking a completed node shows its input/output in the Debug Panel
- Data preview on edges: hover an edge to see the value that flowed through it

---

## Implementation Plan

### Session 1: Debug Infrastructure (Rust)
- [ ] `DebugCommand` enum and `StepMode` in `workflow/engine.rs`
- [ ] Debug channel (`tokio::sync::mpsc`) per debug session
- [ ] `run_workflow_debug` IPC command (wraps `execute_workflow` with debug hooks)
- [ ] `debug_command` IPC command (sends commands to channel)
- [ ] Breakpoint check before node execution
- [ ] `debug.paused` and `debug.node_completed` events
- [ ] 10 unit tests (pause, continue, step, stop, edit input, edit output, skip)

### Session 2: Debug Panel UI
- [ ] `DebugPanel.tsx` — docked panel with Context/Watch/Call Stack tabs
- [ ] Debug toolbar (Continue, Step, Stop buttons with keyboard shortcuts)
- [ ] Breakpoint toggle (right-click menu + `B` shortcut + config panel toggle)
- [ ] Red dot breakpoint indicators on nodes
- [ ] Yellow pulsing border for paused node
- [ ] Green/gray status for completed/pending nodes
- [ ] Store: `debugSession`, `breakpoints`, `debugPaused`, `debugNodeData`

### Session 3: Edit & Continue + Watch
- [ ] Edit Input modal (JSON editor with syntax highlighting)
- [ ] Edit Output modal (post-execution, before continuing)
- [ ] Re-run Node command (re-execute with same or modified input)
- [ ] Skip Node command (inject user-provided output)
- [ ] Watch expressions panel (resolve `{{node.field}}` patterns against node_outputs)
- [ ] Edge data preview on hover during debug

### Session 4: Polish + Testing
- [ ] Keyboard shortcuts (F5, F10, Shift+F5, Shift+F10, B, E, R, S)
- [ ] Canvas visual enhancements (green edges, dimmed future nodes)
- [ ] Call Stack tab (expandable completed nodes)
- [ ] Debug session cleanup (cancel channel on stop/complete)
- [ ] Playwright E2E tests (set breakpoint, step through, edit output)
- [ ] Peer review prompt (Gemini architecture + Codex implementation)

---

## Leveraging Existing Patterns

| New feature | Existing code to build on |
|---|---|
| Pause execution | `ApprovalExecutor` — already pauses and waits for user response via channel |
| Debug events | `emit_workflow_event()` — same event bus, new event types |
| Node input/output display | `Inspector > MessageDetail` — already renders node data |
| JSON editing | Config panel textarea — reuse for input/output editing |
| Breakpoint storage | `data.breakpoint` on node — same pattern as `data.label` |
| Keyboard shortcuts | Existing shortcut system in workflow canvas |
| Step tracking | `node_outputs` HashMap — already tracks all completed node outputs |

The Approval node is the strongest precedent. Its flow:
1. Engine reaches approval node → emits `workflow.node.waiting`
2. UI shows approval dialog
3. User clicks approve → sends IPC `resolve_approval`
4. Engine receives response, continues

Debug mode generalizes this to: ANY node can pause, show data, accept commands.

---

## Scope Boundaries

### In scope (v1)
- Breakpoints on any node
- Step Over (one node at a time)
- Continue (run to next breakpoint)
- Stop (abort)
- View input/output at each step
- Edit Input before execution
- Edit Output after execution (Edit & Continue)
- Skip node with custom output
- Re-run node
- Watch expressions
- Keyboard shortcuts

### Out of scope (v2+)
- **Conditional breakpoints** — break when `{{output}}` matches a pattern
- **Logpoints** — emit a log message without pausing (like VS Code logpoints)
- **Debug history** — save/replay debug sessions
- **Remote debugging** — debug a server-mode workflow from another machine
- **Multi-branch stepping** — step into parallel branches independently
- **Performance profiling** — flame chart of node execution times
- **Memory inspection** — view sidecar conversation state mid-run

---

## Success Criteria

1. User sets breakpoint on LLM node, clicks Debug, execution pauses before the LLM runs
2. User sees exact input (messages + context) the LLM will receive
3. User clicks Step Over, sees LLM output with cost/time
4. User edits the output, continues, sees Router take a different branch
5. A first-time user understands the debug UX within 30 seconds (it's just a debugger)
6. Demo GIF: set breakpoint → step through 5 nodes → edit LLM output → different downstream path (under 60 seconds)
