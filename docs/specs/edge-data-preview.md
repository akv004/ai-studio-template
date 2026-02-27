# Edge Data Preview — X-Ray Mode for Data Flow

**Status**: DRAFT — pending peer review
**Phase**: 5A (pairs with step-through-debugging)
**Priority**: P1 — high impact, low effort, amplifies debugger value
**Author**: AI Studio PM
**Date**: 2026-02-27
**Effort**: ~1 session

---

## Problem Statement

After a workflow runs, users have no visual indication of what data flowed between nodes. The canvas shows nodes and edges, but edges are opaque — you must click each node and open the Inspector to see input/output. For a 10-node workflow, this means 10 clicks to understand the full data flow.

Every developer who has used network packet inspection (Chrome DevTools Network tab) or data pipeline monitoring (Spark UI, Airflow) expects to see data on the wires. AI Studio's canvas should show this.

---

## User Experience

### Toggle: X-Ray Mode

A toggle button on the toolbar (or keyboard shortcut `X`):

```
[▶ Run]  [🐛 Debug]  [🔍 X-Ray]  [Go Live ◉]  [⚙]
```

When enabled, every edge that has data (from the last run) shows a compact preview badge.

### Edge Badges

```
Input ──["Review this PR: diff --git a/au..."]──→ LLM ──["CRITICAL: SQL injection vu..."]──→ Router
                                                                                              │
                                                                                    ┌─────────┤
                                                                    ["critical"]    │  ["normal"]
                                                                         ▼          ▼
                                                                      Approval    Email Send
```

Badge rules:
- **Strings**: First 40 chars, ellipsis if longer
- **Numbers**: Full value
- **Arrays**: `[3 items]` with first item preview
- **Objects**: `{4 keys}` with first key preview
- **null/empty**: `(empty)` dimmed
- **Long text** (LLM output): First 30 chars + `...`

### Hover for Full Preview

Hover over any edge badge → tooltip with full value (up to 500 chars), formatted:
- JSON → syntax-highlighted, pretty-printed
- Plain text → wrapped with line breaks
- Arrays → first 5 items with `... and N more`

### Click for Detail

Click an edge badge → opens a popover with:
- Full value (scrollable, up to 5000 chars)
- Copy button
- "Open in Inspector" link (jumps to the source node's output event)
- Data type indicator (string, number, array, object)
- Size indicator (chars, items, keys)

---

## Architecture

### Data Source

Edge data comes from `node_outputs` — the HashMap that the DAG walker already populates during execution. After a workflow completes, node_outputs is returned to the UI via the `workflow.completed` event.

Currently: `workflow.completed` only includes `finalOutput`.

Change: Include `all_node_outputs` in the completed event (or a new `workflow.debug_data` event).

### New Event Payload

```rust
// In workflow.completed event, add:
{
    "node_outputs": {
        "input-1": { "value": "Review this PR..." },
        "kb-1": { "context": "...", "results": [...] },
        "llm-1": { "value": "CRITICAL: SQL injection..." },
        "router-1": { "branch": "critical", "value": "..." }
    }
}
```

### Store

```typescript
interface WorkflowStore {
    // Existing
    nodeStates: Record<string, NodeState>;

    // New
    lastRunNodeOutputs: Record<string, unknown>;  // node_id → output value
    xrayEnabled: boolean;
    toggleXray: () => void;
}
```

### Edge Rendering

React Flow edges support custom labels. Add a label component that reads from `lastRunNodeOutputs`:

```typescript
// EdgeDataBadge.tsx
function EdgeDataBadge({ sourceNodeId, sourceHandle }: Props) {
    const outputs = useAppStore(s => s.lastRunNodeOutputs);
    const xray = useAppStore(s => s.xrayEnabled);

    if (!xray || !outputs[sourceNodeId]) return null;

    const value = resolveHandle(outputs[sourceNodeId], sourceHandle);
    const preview = formatPreview(value, 40);

    return (
        <div className="edge-badge" title={JSON.stringify(value, null, 2)}>
            {preview}
        </div>
    );
}
```

### Edge Label Positioning

React Flow's `EdgeLabelRenderer` places labels at the edge midpoint. For edges with data badges:
- Badge appears as a small pill on the edge midpoint
- Semi-transparent background (matches edge color)
- Font: 10px monospace
- Max width: 200px, overflow: ellipsis

---

## Implementation Plan

### Single Session (~3 hours)

- [ ] Rust: Include `node_outputs` in `workflow.completed` event payload
- [ ] Store: `lastRunNodeOutputs`, `xrayEnabled`, `toggleXray()`
- [ ] `EdgeDataBadge.tsx` — compact preview component
- [ ] Wire badges into React Flow edge labels (custom edge component)
- [ ] Hover tooltip with full value (max 500 chars)
- [ ] Click popover with scrollable full value + copy button
- [ ] Toolbar X-Ray toggle button + `X` keyboard shortcut
- [ ] CSS: edge badge styling (pill, monospace, semi-transparent)
- [ ] `formatPreview()` utility — smart truncation per data type
- [ ] Clear `lastRunNodeOutputs` on new run start

---

## Integration with Step-Through Debugging

During debug mode, edge badges update in real-time as each node completes:
- Completed edges show data badges (green tint)
- Current edge pulses (data about to flow)
- Future edges remain empty

X-Ray mode is automatically enabled during debug sessions.

---

## Scope Boundaries

### In scope (v1)
- Edge data badges after workflow completion
- Hover tooltip with full value
- Click popover with copy
- Toolbar toggle + keyboard shortcut
- Smart formatting per data type
- Real-time updates during debug mode

### Out of scope (v2+)
- Data type icons on badges (string, array, object)
- Edge bandwidth visualization (thicker edge = more data)
- Data diff between runs (compare edge values across two runs)
- Filter badges by data type or content
- Export edge data as CSV/JSON

---

## Success Criteria

1. After running a workflow, toggle X-Ray → see data previews on every edge
2. Hover any edge → see full value
3. A user can trace "what data went where" without opening Inspector
4. During debug mode, edge badges appear as each node completes
5. Performance: no visible lag on a 20-node workflow with badges
