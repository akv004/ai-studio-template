# Natural Language Canvas — Talk to Your Workflow

**Status**: DRAFT — pending peer review
**Phase**: 5C (requires stable node editor + strong template system)
**Priority**: P0 — accessibility breakthrough, extends audience beyond developers
**Author**: AI Studio PM
**Date**: 2026-02-27
**Effort**: ~5 sessions
**Tagline**: "Cursor for visual AI pipelines"

---

## Problem Statement

Building a workflow today requires:
1. Knowing which node types exist (23 types)
2. Knowing which handles connect where
3. Dragging, dropping, wiring, configuring — manual for every node

This works for developers. But the people who MOST need AI workflows — product managers, analysts, ops teams — are intimidated by a blank canvas with 23 node types. They know WHAT they want ("fetch data, summarize it, email the team every morning") but not HOW to build it.

**Natural Language Canvas** lets users describe workflow changes in plain English. The AI modifies the graph in real-time.

---

## User Experience

### Chat Input on Canvas

A persistent chat input at the bottom of the workflow canvas:

```
┌─ Workflow Canvas ──────────────────────────────────────────┐
│                                                            │
│  [Input] ──→ [LLM] ──→ [Output]                          │
│                                                            │
│                                                            │
├────────────────────────────────────────────────────────────┤
│ 💬 "Add an email step after the LLM that sends the        │
│     summary to team@company.com"                    [Send] │
└────────────────────────────────────────────────────────────┘
```

### Response: Graph Modification

The AI processes the request and generates graph operations:

```
┌─ AI Response ──────────────────────────────────────────────┐
│                                                            │
│  I'll add an Email Send node between LLM and Output:      │
│                                                            │
│  + Add node: Email Send (email-1)                         │
│    - to: team@company.com                                  │
│    - subject: "Daily Summary"                              │
│    - body: connected from LLM output                       │
│  + Add edge: llm-1.output → email-1.body                  │
│  + Move edge: llm-1.output → output-1 to email-1 → output│
│                                                            │
│  [Apply]  [Edit]  [Cancel]                                 │
└────────────────────────────────────────────────────────────┘
```

User clicks **Apply** → the canvas updates with the new node and edges, animated into position.

### Example Interactions

| User says | AI does |
|---|---|
| "Add an email step after the LLM" | Inserts Email Send node, wires LLM→Email→Output |
| "Make the router check for 3 categories instead of 2" | Updates Router node config: adds third branch |
| "Replace the LLM with a cheaper model" | Changes LLM node's model field |
| "Add error handling — if the HTTP request fails, send an alert" | Adds Router after HTTP (success/error branches), wires error→Email |
| "This workflow should run every morning at 9am" | Adds Cron Trigger node at the start, wires to first node |
| "Remove the approval step, it's slowing things down" | Removes Approval node, reconnects upstream→downstream |
| "What does this workflow do?" | Describes the pipeline in plain English (read-only, no changes) |
| "Split the output into two — one for email, one for file" | Adds second Output node, wires from same source |

### Safety: Preview Before Apply

All modifications are shown as a preview (diff) before applying:
- Green: new nodes/edges
- Red: removed nodes/edges
- Yellow: modified node config
- User must click **Apply** to commit changes

This prevents "oops, the AI deleted my whole workflow."

---

## Architecture

### Graph Operation Language

The AI generates structured graph operations, not raw JSON:

```typescript
interface GraphOperation {
    type: 'add_node' | 'remove_node' | 'add_edge' | 'remove_edge' | 'update_config' | 'move_node';
    // For add_node:
    nodeType?: string;
    nodeId?: string;
    label?: string;
    config?: Record<string, unknown>;
    position?: { x: number; y: number };
    // For remove_node:
    targetNodeId?: string;
    // For add_edge / remove_edge:
    source?: string;
    sourceHandle?: string;
    target?: string;
    targetHandle?: string;
    // For update_config:
    field?: string;
    value?: unknown;
}
```

### LLM Prompt Design

The system prompt gives the LLM:
1. Current graph structure (simplified — node types, labels, edges, NOT full JSON)
2. Available node types with their handles
3. The user's request
4. Output format: list of `GraphOperation` objects

```
You are an AI workflow editor. The user describes changes to their workflow.
Given the current graph and available node types, produce a list of graph operations.

Current workflow:
- input-1 (Input, label: "Code Diff") → llm-1 (LLM, label: "Analyzer", model: gpt-4o)
- llm-1 → output-1 (Output, label: "Report")

Available node types: [Input, Output, LLM, Tool, Router, Approval, Transform, HTTP Request, ...]
Each type has handles: [list of input/output handles per type]

User request: "Add an email step after the LLM that sends the summary to team@company.com"

Respond with JSON: { "explanation": "...", "operations": [...] }
```

### Execution Flow

```
User types request
    ↓
serialize_current_graph() → simplified graph description
    ↓
Build LLM prompt (graph + node types + user request)
    ↓
Call LLM (via sidecar /chat/direct, using configured provider)
    ↓
Parse response → GraphOperation[]
    ↓
Show preview (diff visualization on canvas)
    ↓
User clicks Apply
    ↓
apply_operations(graph, operations) → new graph
    ↓
Update React Flow state + auto-layout affected nodes
```

### Auto-Layout

When nodes are added/removed, the layout may need adjustment. Use a simple algorithm:
- New nodes are placed between their source and target (midpoint)
- If inserting between two connected nodes, shift downstream nodes right by 250px
- Use React Flow's `fitView()` after changes

---

## Implementation Plan

### Session 1: Graph Operations Engine
- [ ] `GraphOperation` type definition
- [ ] `serialize_graph_for_llm()` — convert React Flow JSON to simplified description
- [ ] `apply_operations()` — apply operations to React Flow graph state
- [ ] Position calculation for inserted nodes
- [ ] 10 unit tests (add/remove/update/insert-between operations)

### Session 2: LLM Integration
- [ ] System prompt template with graph context + node type reference
- [ ] Call sidecar `/chat/direct` with graph prompt
- [ ] Parse structured JSON response
- [ ] Error handling (malformed response, unknown node types, invalid edges)
- [ ] Fallback: if JSON parse fails, show error + raw response

### Session 3: Canvas Chat UI
- [ ] `CanvasChat.tsx` — input bar at bottom of canvas
- [ ] Response panel with explanation + operation list
- [ ] Preview mode: highlight new/removed/modified elements on canvas
- [ ] Apply / Edit / Cancel buttons
- [ ] Loading state while LLM processes
- [ ] Chat history (last 10 interactions, session-scoped)

### Session 4: Polish + Edge Cases
- [ ] "What does this workflow do?" — read-only describe mode
- [ ] "Undo last change" — revert most recent Apply
- [ ] Auto-layout after insert/remove
- [ ] Handle complex requests (multi-step: "add error handling with email alerts")
- [ ] Keyboard shortcut: `/` to focus chat input (like Slack)
- [ ] Playwright E2E: type request → preview → apply → verify graph changed

### Session 5: Templates + Examples
- [ ] Pre-built example prompts in chat input placeholder
- [ ] "Try: Add an email notification" / "Try: Make it run on a schedule"
- [ ] Quick-action buttons for common operations (Add LLM, Add Email, Add Trigger)
- [ ] Tutorial overlay on first use

---

## Scope Boundaries

### In scope (v1)
- Add/remove/modify nodes via natural language
- Add/remove edges
- Update node config (model, prompt, parameters)
- Preview before apply
- Describe workflow (read-only)
- Single-turn interactions (one request → one set of changes)

### Out of scope (v2+)
- Multi-turn conversations ("now connect that to..." referring to previous turn)
- Voice input ("Hey AI Studio, add an email step")
- Generate entire workflow from scratch (overlap with Auto-Pipeline Generator)
- Suggest improvements proactively ("This workflow could be optimized by...")
- Collaborative editing (multiple users + AI modifying simultaneously)
- Learning from user patterns ("You always add email after LLM — do it automatically?")

---

## Success Criteria

1. User types "add an email after the LLM" → sees preview with new Email node → clicks Apply → graph updates
2. User types "what does this workflow do?" → gets plain English description
3. Non-developer can build a 5-node workflow entirely through chat (no drag-and-drop)
4. Preview clearly shows what will change (green/red/yellow highlighting)
5. Handles edge cases gracefully (unknown request → "I don't understand, try...")
