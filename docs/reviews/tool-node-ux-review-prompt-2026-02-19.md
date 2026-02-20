# Peer Review: Tool Node UX Specification

**Recommended reviewer**: Antigravity (Gemini) — architecture + UX + open-source readiness
**Date**: 2026-02-19
**Status**: OPEN

---

## Context

AI Studio is an open-source desktop IDE for building AI agent workflows (Tauri 2 + React 19 + Python FastAPI sidecar). It has a visual node editor ("Unreal Blueprints for AI agents") with 16 node types.

The **Tool node** connects MCP (Model Context Protocol) tools into visual workflows. Currently it has a broken UX — a blank text field where users type a tool name manually. This spec designs the upgrade: Tool Browser + Dynamic Schema Form.

## Files to Read

Read these files in order:

1. **The spec under review**: `docs/specs/tool-node-ux.md` — the full spec (read this completely)
2. **Existing MCP spec**: `docs/specs/mcp-integration.md` — current MCP architecture (for context)
3. **Current implementation**:
   - `apps/desktop/src-tauri/src/workflow/executors/tool.rs` — Rust executor (80 lines)
   - `apps/ui/src/app/pages/workflow/nodes/ToolNode.tsx` — React node component (32 lines)
   - `apps/ui/src/app/pages/workflow/NodeConfigPanel.tsx` — Config panel, lines 124-142 (tool section)
   - `apps/sidecar/agent/mcp/registry.py` — Tool registry + `to_summary()` method
   - `apps/sidecar/agent/mcp/builtin.py` — Built-in tool definitions with schemas
4. **Node editor guide**: `docs/node-editor-guide.md` — for general node editor context
5. **Tool node docs**: `docs/tool-node.md` — usage guide we just wrote (shows the UX gap)

## What to Review

### 1. Architecture (High Priority)

- Is the `list_available_tools` IPC → sidecar proxy approach correct? Or should Rust cache tool schemas independently?
- Is the data flow for static vs dynamic input mode clear and correct?
- Will the migration path work for existing workflows?
- Is storing `toolSchema` on each node's data the right choice, or should it be centralized?

### 2. UX Design (High Priority)

- Does the Tool Picker dropdown design make sense for 5-50+ tools?
- Is the "Use incoming data" toggle the right abstraction for static vs dynamic input?
- Are there better patterns for the schema → form mapping (e.g., how does n8n, Retool, or React JSON Schema Form handle this)?
- Will the built-in tool hints ("use Shell Exec node instead") confuse users or help them?

### 3. Completeness (Medium Priority)

- Any missing edge cases? (tool disconnects mid-workflow, schema changes between runs, etc.)
- Should the spec address error states more explicitly? (MCP server down, schema validation failure, etc.)
- Is the Palette MCP Section (P2) worth the complexity, or is the dropdown enough?

### 4. Open-Source Readiness (Medium Priority)

- Would this feature excite contributors? Is it clearly scoped enough for someone to pick up?
- Does the component design (ToolPicker, SchemaForm) follow good reusable patterns?
- Any accessibility concerns (keyboard nav in dropdown, screen reader for schema forms)?

### 5. Competitive Analysis (Low Priority)

- How does this compare to LangFlow's tool node, n8n's node configuration, or Flowise?
- Are we missing anything they do well?

## Output Format

Please structure your review as:

```
## Summary
One paragraph overall assessment.

## Findings

### [CRITICAL/HIGH/MEDIUM/LOW] — Short title
**Category**: Architecture | UX | Completeness | Open-Source | Competitive
**Description**: What the issue is
**Recommendation**: How to fix it
```

Number each finding for easy reference.
