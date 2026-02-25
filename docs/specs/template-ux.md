# Template UX & Discoverability

**Status**: DRAFT
**Phase**: 5A
**Author**: AI Studio PM
**Date**: 2026-02-25

---

## Problem Statement

AI Studio's workflow engine is powerful — 19 node types, loops, iterators, RAG, routers. But **new users can't use it** without deep knowledge of internal mechanics:

1. **Transform `{{node_id.field}}` is invisible magic** — Users don't know node IDs exist, don't know what fields each node outputs, and don't know that templates can reference any upstream node without an edge. There's zero discoverability.

2. **Loop feedback is incomprehensible** — "feedbackMode: append" means the Transform output goes back through the Loop as input next iteration. But nothing on the canvas shows this. Users see a linear chain and have no idea output wraps back to input.

3. **Edge ≠ data flow** — Edges serve two purposes (data passing AND execution ordering) but look the same. A Transform can read from nodes it has no edge to. This breaks the mental model of "lines = data flow."

4. **Node IDs are hidden** — Every node has an ID (e.g., `llm_draft`, `transform_merge`) that's critical for templates, but it's only visible in the JSON. The canvas shows labels ("Draft / Revise") which are different from IDs.

5. **No guidance for patterns** — Building a self-refine loop, an agentic search, or a code fix pipeline requires knowing which nodes to combine and how. Templates exist but users can't modify them without understanding the underlying mechanics.

**Result**: Only the person who built the product can use the advanced features. This is a ceiling on adoption.

---

## Design

### 1. Template Autocomplete (Highest Impact)

**Where**: Any text field that supports `{{}}` syntax — Transform template, LLM system prompt, Shell Exec command, HTTP URL.

**How it works**:
1. User types `{{` in a text field
2. Dropdown appears showing all nodes in the workflow:
   ```
   ┌─────────────────────────────────┐
   │ 🔤 input_1      (Input)        │
   │ 🔁 loop_1       (Loop)         │
   │ 🤖 llm_draft    (LLM)          │  ← cursor here
   │ 🤖 llm_critique (LLM)          │
   │ 📐 transform_1  (Transform)    │
   └─────────────────────────────────┘
   ```
3. User selects a node → expands to show available output fields:
   ```
   ┌─────────────────────────────────┐
   │ llm_draft.response              │
   │ llm_draft.output                │
   │ llm_draft.tokens                │
   └─────────────────────────────────┐
   ```
4. Click inserts `{{llm_draft.response}}` at cursor position
5. Typing after `{{` filters the list (fuzzy match on node ID and label)

**Output fields per node type** (hardcoded list in UI):

| Node Type | Available Fields |
|-----------|-----------------|
| LLM | `response`, `output`, `tokens` |
| Input | `value`, `output` |
| Transform | `output` |
| Router | `output`, `selectedBranch` |
| HTTP Request | `body`, `status`, `headers` |
| Shell Exec | `stdout`, `stderr`, `exitCode` |
| File Read | `content`, `output` |
| File Glob | `files`, `output` |
| File Write | `path`, `output` |
| Iterator | `output`, `item`, `index` |
| Knowledge Base | `context`, `results`, `indexStats` |
| Validator | `output`, `valid`, `errors` |

**Implementation**:
- New component: `TemplateAutocomplete.tsx` — wraps `<textarea>` with dropdown
- Uses React Flow's `useNodes()` hook to get all nodes in current graph
- Triggered by `{{` keystroke detection
- Positioned relative to cursor (like VS Code intellisense)
- Used in: NodeConfigPanel (template fields, system prompt, command fields)

---

### 2. Node ID Visible on Canvas

**Current**: Nodes show only their label ("Draft / Revise") or type ("LLM").
**Proposed**: Show node ID as small muted text below the label.

```
┌─────────────────────┐
│  🤖 LLM             │
│  Draft / Revise      │  ← label (user-editable)
│  llm_draft           │  ← node ID (small, muted, read-only)
└─────────────────────┘
```

**Why**: Users need to see the ID to understand what `{{llm_draft.response}}` references. Currently they have to open the JSON to find this.

**Toggle**: Settings checkbox "Show node IDs on canvas" — default ON for new users, existing users unaffected.

**Implementation**:
- Add `<div className="node-id">` to `NodeShell.tsx` — 3 lines of TSX + CSS
- Read setting from Zustand store
- Node IDs are already in React Flow node data — just display them

---

### 3. Implicit Dependency Indicators

**Problem**: Transform references `{{llm_draft.output}}` but has no edge from `llm_draft`. On the canvas, it looks disconnected. Users wonder "where does this data come from?"

**Proposed**: When a node's template references another node via `{{node_id.field}}`, draw a **dashed line** from the referenced node to the referencing node.

```
llm_draft --------→ transform_merge    (solid = edge, carries data via handle)
llm_draft - - - -→ transform_merge    (dashed = implicit reference via template)
```

Visual rules:
- Dashed line, 50% opacity, same color as node type
- Only shown when the referencing node is selected (to avoid clutter)
- Tooltip on hover: "Referenced via {{llm_draft.output}} in template"
- Non-interactive (can't click or delete — it's not a real edge)

**Implementation**:
- Parse `{{...}}` patterns from node config (template, systemPrompt, command, url fields)
- Extract referenced node IDs
- Render as SVG dashed paths in a custom React Flow layer
- Recompute on node config change (debounced)
- New component: `ImplicitDependencyLayer.tsx` — ~80 lines

---

### 4. Loop Feedback Visualization

**Problem**: Loop feedback is the most confusing concept. Output of iteration N becomes input of iteration N+1. But the canvas shows a linear chain with no visual indication of the backward flow.

**Proposed**: When a Loop node is present, draw a **curved feedback arrow** from Exit back to Loop, with annotation.

```
                    ┌──── feedback (replace) ◄────┐
                    ▼                              │
Input → [Loop] → Draft → Critic → Merge → [Exit] → Output
          │        iteration 1, 2, 3...        │
          └────────── loop body ───────────────┘
```

Visual elements:
- **Loop body highlight**: Light purple background behind all nodes between Loop and Exit
- **Feedback arrow**: Curved dashed arrow from Exit back to Loop, labeled with feedback mode ("replace" / "append")
- **Iteration badge**: Small counter on Loop node showing "3 iterations" (from config)
- **Runtime**: During execution, badge updates to "Iteration 2/3" with a spinner

**Implementation**:
- Detect Loop↔Exit pairs from edges (same algorithm as `find_loop_subgraph` in Rust)
- Render background highlight as SVG rect encompassing subgraph node positions
- Render feedback arrow as SVG curved path
- New component: `LoopVisualization.tsx` — ~120 lines
- Runtime updates via existing workflow execution events (`workflow.node.iteration`)

---

### 5. Pattern Presets (Loop Wizard)

**Problem**: Building a self-refine loop from scratch requires knowing: you need a Loop, an Exit, the right feedback mode, at least 2 LLMs (draft + critic), a Transform to merge, and the right edge connections. That's 6 nodes and 6 edges to get right.

**Proposed**: When adding a Loop node from the palette, offer a preset picker:

```
┌─────────────────────────────────────────┐
│  Add Loop Pattern                        │
│                                          │
│  ○ Empty Loop                            │
│    Just Loop + Exit, you wire the body   │
│                                          │
│  ● Self-Refine                           │
│    Draft → Critique → Merge → repeat     │
│    3 iterations, feedbackMode: replace   │
│                                          │
│  ○ Search + Evaluate                     │
│    Plan → Search → Answer → Router eval  │
│    Up to 5 iterations, exits when done   │
│                                          │
│  ○ Converge                              │
│    Run until output stabilizes           │
│    stabilityThreshold: 0.95             │
│                                          │
│         [Add to Canvas]                  │
└─────────────────────────────────────────┘
```

Selecting a preset drops the full subgraph onto the canvas — all nodes positioned, all edges connected, all configs set. User can then customize (change LLM models, edit prompts, add nodes).

**Implementation**:
- Preset definitions as JSON (same format as templates, but partial — no Input/Output)
- Modal component: `LoopPresetPicker.tsx` — ~100 lines
- Triggered when Loop is dropped/clicked from palette
- Each preset is a mini-graph that gets merged into the current workflow at drop position

---

### 6. Inline Node Help

**Problem**: Users don't know what each node does or what its outputs are.

**Proposed**: Small `?` button on each node type that shows a tooltip/popover:

```
┌─────────────────────────────────────────┐
│ 📐 Transform                            │
│                                          │
│ Converts data using templates or         │
│ expressions. Can reference any upstream  │
│ node by ID: {{node_id.field}}            │
│                                          │
│ Outputs: output (any)                    │
│                                          │
│ Example:                                 │
│   Template: "Hello {{input_1.value}}"    │
│   → "Hello World"                        │
│                                          │
│ Tip: Type {{ in the template field to    │
│ see available nodes and fields.          │
└─────────────────────────────────────────┘
```

**Implementation**:
- Help content: static map of node type → { description, outputs, example, tip }
- Component: `NodeHelp.tsx` — tooltip/popover triggered by `?` icon in NodeShell header
- ~200 lines for the component + help content for all 19 node types
- Also accessible from palette (hover on node type before adding)

---

## Implementation Priority

| # | Feature | Impact | Effort | Priority |
|---|---------|--------|--------|----------|
| 1 | **Template Autocomplete** | HIGH — eliminates the biggest pain point | Medium (~200 lines) | P0 |
| 2 | **Node ID on Canvas** | HIGH — 5 minutes of work, instant clarity | Tiny (~10 lines) | P0 |
| 3 | **Inline Node Help** | HIGH — self-service learning | Small (~200 lines) | P0 |
| 4 | **Loop Feedback Visualization** | MEDIUM — helps understand loops | Medium (~150 lines) | P1 |
| 5 | **Implicit Dependency Lines** | MEDIUM — explains Transform magic | Medium (~100 lines) | P1 |
| 6 | **Pattern Presets** | MEDIUM — faster loop building | Medium (~200 lines) | P2 |

**Recommended build order**: 2 → 1 → 3 → 4 → 5 → 6

Node ID on canvas is a 10-minute win. Template autocomplete is the biggest unlock. Inline help makes everything self-documenting. The visual features (4, 5, 6) are polish after the core discoverability is solved.

---

## What This Does NOT Cover

- **Tutorial / onboarding wizard** — a guided "build your first workflow" experience. Valuable but separate scope.
- **Node documentation page** — a full docs site with examples. The inline help is a lighter-weight solution.
- **AI-assisted workflow building** — "describe what you want, AI builds the graph." That's the Auto-Pipeline Generator from killer-features.md.

---

## Success Criteria

1. New user can build a Transform node that references upstream LLM output **without reading docs** — autocomplete guides them
2. New user can understand what `{{llm_draft.response}}` means by seeing the node ID on the canvas
3. Loop workflows show visible feedback path — no more "where does the data go back?"
4. Every node type has inline help accessible in one click from the canvas
