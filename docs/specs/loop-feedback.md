# Loop & Feedback Node

**Status**: DRAFT
**Phase**: 5A
**Author**: AI Studio PM
**Date**: 2026-02-22

---

## Problem Statement

The workflow engine is strictly DAG — no cycles. This prevents the most powerful AI pattern: **iterative refinement**. Users cannot build:

1. **Self-critique loops** — LLM drafts → evaluator grades → LLM revises with feedback → repeat until good
2. **Agentic reasoning** — LLM thinks → acts → observes result → thinks again → acts again
3. **Convergence workflows** — run until output stabilizes (diff between iterations < threshold)
4. **Multi-pass extraction** — first pass gets rough data → second pass fills gaps → third pass validates

Every serious AI framework supports this (LangGraph cycles, CrewAI loops, AutoGen multi-turn). Without it, AI Studio workflows are single-pass only — a hard ceiling on capability.

---

## Design

### Approach: Loop Node (reuse Iterator architecture)

The Iterator already solves 80% of this problem:
- Extracts a subgraph between Iterator and Aggregator
- Builds a synthetic workflow for isolated execution
- Runs it multiple times with different inputs
- Collects results via Aggregator

**Loop reuses this pattern** but changes the semantics:

| | Iterator | Loop |
|---|---|---|
| **Input** | Array of items | Single initial value |
| **Per-iteration input** | One item from array | Previous iteration's output (feedback) |
| **Termination** | All items processed | Condition met OR max iterations |
| **State** | Stateless (each item independent) | Stateful (output feeds next input) |
| **Downstream node** | Aggregator (collects all results) | Exit (passes final result through) |

### Why not relax DAG constraints?

Adding real cycles to the engine would require rewriting topological sort, handling infinite loops at the engine level, and rethinking the event system. The Iterator-style "virtual loop" approach is better:

- DAG engine stays unchanged — no architectural risk
- Loop is contained within a single node's execution (like Iterator)
- Cycle detection still works — the graph is still a DAG
- Cost controls are local to the Loop node

---

## Node Type: `loop`

### Config

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| maxIterations | int | 5 | Hard cap on iterations (safety valve, 1-50) |
| exitCondition | enum | `max_iterations` | When to stop: `max_iterations`, `evaluator`, `stable_output` |
| stabilityThreshold | float | 0.95 | For `stable_output`: stop when similarity between consecutive outputs exceeds this |
| feedbackMode | enum | `replace` | How to pass state: `replace` (output replaces input), `append` (text-only: output appended to input as conversation) |

### Handles

**Input handles:**

| Handle | Type | Required | Description |
|--------|------|----------|-------------|
| input | text/json | yes | Initial value for first iteration |

**Output handles:**

| Handle | Type | Description |
|--------|------|-------------|
| output | text/json | Final iteration result |
| iterations | json | Array of all intermediate results |
| count | number | How many iterations ran |

### Exit Condition Modes

#### 1. `max_iterations` (simplest)
Run exactly `maxIterations` times. Output = last iteration's result.

**Use case**: "Revise this draft 3 times, each time improving on the previous."

#### 2. `evaluator` (most powerful)
The subgraph must contain a **Router node** that outputs to either:
- `done` handle → Exit node receives value → loop stops, that value becomes final output
- `continue` handle → that value becomes next iteration's input

**Router branch detection**: The Router executor must include `selectedBranch` in its `NodeOutput.value` (e.g., `{"selectedBranch": "done", "value": "..."}`) so the Loop executor can inspect the synthetic graph's output to determine which branch fired. Today Router only records `selected_branch` to DB events — this must be added to the dataflow output.

**How Loop detects the branch**: After each synthetic execution, Loop reads the Router node's output from the synthetic graph results (not from `__loop_output__`). If the Router's `selectedBranch == "done"`, Loop reads the Exit node's output as the final result. If `selectedBranch == "continue"`, Loop reads the Router's output value as the next iteration's input.

The Router acts as the evaluator — it can use an LLM to decide whether the result is good enough.

**Use case**: "Keep refining until the evaluator LLM says the answer is complete and accurate."

#### 3. `stable_output` (convergence)
Compare consecutive outputs using text similarity. Stop when similarity exceeds `stabilityThreshold`.

**Algorithm**: Bounded normalized Levenshtein distance. Both outputs are stringified (JSON `to_string()` for non-string values) and truncated to 10,000 chars before comparison. Similarity = `1.0 - (edit_distance / max(len_a, len_b))`. No external crate needed — simple O(n*m) implementation with the truncation cap keeping it fast.

**Text-only**: This mode only works with text outputs. If outputs are structured JSON, they are serialized with `serde_json::to_string()` (compact, deterministic key order) before comparison.

**Use case**: "Keep summarizing until the summary stops changing."

---

## Graph Structure

### Pattern: LLM Self-Refinement

```
Input → [Loop] → LLM (draft) → LLM (critique) → Transform (merge feedback) → [Exit] → Output
```

The Loop node extracts the subgraph between `[Loop]` and `[Exit]`, just like Iterator extracts between Iterator and Aggregator.

### Pattern: Evaluator-Controlled Loop

```
Input → [Loop] → LLM (draft) → Router (evaluate)
                                  ├─ done → [Exit] → Output
                                  └─ continue → Transform (add feedback) ──→ (feeds back to LLM)
```

**Important**: The `continue` → Transform → LLM path is inside the subgraph. It's not a real cycle in the DAG — the Loop node internally manages the feedback wiring.

### Pattern: Agentic Tool Loop

```
Input → [Loop] → LLM (think+act) → Tool (execute) → Transform (observe) → Router (done?)
                                                                             ├─ done → [Exit]
                                                                             └─ continue → (back to LLM)
```

---

## Exit Node

A new node type `exit` serves as the Loop's termination point (analogous to Aggregator for Iterator).

### Config

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| (none) | | | Exit is a pass-through marker — no config needed |

### Handles

| Handle | Type | Direction | Description |
|--------|------|-----------|-------------|
| input | text/json | target | Receives the final value to pass out of the loop |
| output | text/json | source | Passes the value to downstream nodes |

### Validation Rules

- Every Loop node must have exactly one Exit node downstream in its subgraph
- An Exit node without a paired Loop is a validation error
- Exactly like Iterator↔Aggregator pairing rules
- **Nesting validation**: Use `find_subgraph()` BFS during validation to detect if a Loop's subgraph contains another Loop or Iterator. Reject with error: "Nested loops and loops inside iterators are not supported (Phase 2)."

---

## Execution Flow

### Subgraph Extraction (reuse Iterator pattern)

1. **Forward BFS** from Loop node → find all reachable Exit nodes (stop at Exit, don't traverse past)
2. **Backward BFS** from Exit → find all nodes that can reach it (stop at Loop)
3. **Intersection** = subgraph body
4. Validate exactly one Exit found

### Synthetic Graph Construction

For each iteration, build a synthetic workflow:
- Replace Loop's outgoing edge with `__loop_input__` (Input node)
- Copy all subgraph nodes
- Replace Exit node with `__loop_output__` (Output node)
- Preserve all internal edges

### Per-Iteration Execution

```
iteration_0:
  input = original_input
  result = execute_synthetic_graph(input)

iteration_1:
  input = result_0  (feedbackMode=replace, works with text or JSON)
     OR  input + "\n---\n" + result_0  (feedbackMode=append, text-only — error if input is non-string JSON)
  result = execute_synthetic_graph(input)

...repeat until exit condition met...

final_output = last result
```

### For `evaluator` mode:

The Router inside the subgraph controls flow:
- If Router outputs `selectedBranch: "done"` → Exit node receives value → loop stops
- If Router outputs `selectedBranch: "continue"` → Router's output value becomes next iteration's input
- Loop executor inspects Router's `selectedBranch` field in the synthetic graph's `node_outputs` after each execution
- **Requires**: Router executor must include `selectedBranch` in its `NodeOutput.value` (implementation change to `router.rs`)

### Skip Pattern (same as Iterator)

Loop returns:
- `skip_nodes` = all subgraph node IDs + Exit node ID
- `extra_outputs` = `{exit_id: final_result}`
- Engine pre-inserts Exit result into `node_outputs`
- When engine reaches Exit in topo order, it's already computed

---

## Events

| Event | Data | When |
|-------|------|------|
| `workflow.node.iteration` | `{node_id, index, total, input_preview}` | Each iteration starts (reuses Iterator's `index`/`total` field names for consistency) |
| `workflow.node.streaming` | `{node_id, tokens}` | Progress updates during iteration |
| `workflow.node.completed` | `{node_id, iterations_run, exit_reason}` | Loop finishes |

`exit_reason` enum: `max_iterations`, `evaluator_done`, `stable_output`, `error`

---

## UI

### Loop Node Appearance

```
+-------------------------------+
|  ↻  LOOP                     |
| [input] ← input              |
|                               |
|  Max Iterations: [5]         |
|  Exit: [evaluator ▼]         |
|  Feedback: [replace ▼]       |
|                               |
|          output → [text]      |
|       iterations → [json]     |
|            count → [number]   |
+-------------------------------+
```

Icon: `RefreshCw` from lucide-react (circular arrows)
Color: `#4a2a6a` (purple — control flow, matches Iterator/Aggregator family)

### Exit Node Appearance

Minimal — same style as Output but marked as loop exit:

```
+--------------------+
|  ◉  EXIT           |
| [input] ← input    |
|        output →     |
+--------------------+
```

Icon: `LogOut` from lucide-react
Color: Same as Loop node

### Node Palette

Add to LOGIC category (alongside Iterator, Aggregator):

```
LOGIC
  ├─ Approval
  ├─ Transform
  ├─ Validator
  ├─ Iterator
  ├─ Aggregator
  ├─ Loop        ← NEW
  └─ Exit        ← NEW
```

---

## Bundled Templates

### Template: Self-Refine

**Description**: "Draft → critique → revise loop — LLM improves its own output iteratively"

```
Input → Loop → LLM (draft/revise) → LLM (critique) → Transform (merge) → Exit → Output
```

- Loop config: maxIterations=3, exitCondition=max_iterations, feedbackMode=replace
- First LLM: "Write/revise based on feedback: {{input}}"
- Second LLM: "Critique this draft. List specific improvements needed."
- Transform: template merging critique + draft for next iteration

### Template: Agentic Search

**Description**: "LLM decides what to search, evaluates results, searches again if needed"

```
Input → Loop → LLM (plan search) → Tool (search) → Router (enough?)
                                                      ├─ done → Exit → Output
                                                      └─ continue → Transform (what's missing)
```

- Loop config: maxIterations=5, exitCondition=evaluator
- Router LLM system prompt: "Evaluate if the search results answer the question. Output 'done' or 'continue' with explanation of what's missing."

---

## Safety & Cost Controls

| Control | Implementation |
|---------|---------------|
| **Max iterations hard cap** | `maxIterations` clamped to [1, 50]. Configurable, default 5. |
| **Per-node timeout** | Existing workflow timeout applies per synthetic execution |
| **Cost tracking** | Each synthetic execution emits cost events. Loop node sums them. |
| **Infinite loop prevention** | DAG engine unchanged — no real cycles exist. Loop manages iteration count internally. |
| **Budget enforcement** | Existing budget check runs before each sidecar call inside the loop |

---

## Implementation Plan

### New Files

| File | Description |
|------|-------------|
| `executors/loop_node.rs` | Loop executor — subgraph extraction, synthetic graph, iteration loop |
| `executors/exit.rs` | Exit executor — pass-through (trivial, like Aggregator's stub) |
| `nodes/LoopNode.tsx` | UI component |
| `nodes/ExitNode.tsx` | UI component |
| `templates/self-refine.json` | Bundled template |
| `templates/agentic-search.json` | Bundled template |

### Modified Files

| File | Change |
|------|--------|
| `executors/mod.rs` | Register `loop_node::LoopExecutor`, `exit::ExitExecutor` |
| `executors/router.rs` | Add `selectedBranch` to `NodeOutput.value` (for evaluator mode detection) |
| `nodeTypes.ts` | Add LoopNode, ExitNode |
| `nodeCategories.ts` | Add to LOGIC |
| `nodeColors.ts` | Add loop + exit colors |
| `NodeConfigPanel.tsx` | Add loop config section |
| `templates.rs` | Add 2 templates |
| `tauri-mock.ts` | Add to E2E mock |
| `workflow-canvas.spec.ts` | Add E2E tests |
| `validation.rs` | Add Loop↔Exit pairing validation (same pattern as Iterator↔Aggregator) |

### Estimated Scope

- **Executor**: ~400 lines (heavily reuses Iterator's subgraph extraction — copy + adapt)
- **UI**: ~150 lines (2 node components + config panel section)
- **Templates**: ~50 lines each
- **Validation**: ~30 lines (pairing check)
- **Tests**: ~20 unit tests (iteration count, evaluator exit, stable output, feedback modes, max cap)
- **Total**: ~800 lines of new code

### Not in Scope (Phase 2)

- Nested loops (Loop inside Loop) — validate and reject for now
- Parallel loop iterations — always sequential
- Loop inside Iterator subgraph — validate and reject
- Visual iteration counter on canvas during execution
- Replay/inspect individual loop iterations in Inspector

---

## Comparison with Competitors

| Feature | AI Studio (this spec) | LangGraph | n8n | ComfyUI |
|---------|----------------------|-----------|-----|---------|
| Visual loop node | Yes (drag-and-drop) | No (code only) | Limited (webhook loop) | No |
| Evaluator-controlled exit | Yes (Router integration) | Yes (conditional edges) | No | No |
| Convergence detection | Yes (stable_output) | Manual | No | No |
| Cost controls per loop | Yes (budget enforcement) | No | No | No |
| Max iteration safety | Yes (clamped 1-50) | Manual | Manual | N/A |
| Feedback mode options | replace / append | Manual | N/A | N/A |

**Differentiator**: Only visual workflow builder with native loop support + cost controls + evaluator-based exit. LangGraph has cycles but requires code. n8n has no real loop primitive.
