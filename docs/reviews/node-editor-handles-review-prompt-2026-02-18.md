# Peer Review: Node Editor Handle System — Spec vs Implementation Gap

**Date**: 2026-02-18
**Project**: AI Studio (open-source IDE for AI agents)
**Reviewer**: Gemini 3 Pro
**Review type**: Architecture — cross-layer consistency, design gap analysis

---

## Instructions for Reviewer

You are reviewing code in the workspace at `/home/amit/projects/myws/ws01/ai-studio-template`.
Read the files listed below, then provide your review in the output format specified.

## Context

AI Studio's Node Editor is a visual pipeline builder ("Unreal Blueprints for AI agents") with 8 node types, a DAG execution engine in Rust, and a React Flow UI. The spec defines rich multi-input/multi-output handles for nodes (e.g., LLM has 3 inputs: prompt/system/context, and 3 outputs: response/usage/cost). However, the current implementation uses simplified single-handle nodes for most types. Transform and Router nodes already have dynamic handles. We need to close this gap for Phase 4.

## Scope

Review the handle system across three layers:
1. **Spec** (what's designed) — the node-editor.md spec's handle definitions
2. **Engine** (Rust) — how the DAG walker resolves and passes multi-input data
3. **UI** (React) — what handles are actually rendered and connectable

Focus on the gap between spec and implementation, and recommend the best approach to close it.

## Files to Read

Read these files in this order:

1. `docs/specs/node-editor.md` (lines 140-400) — Handle type definitions, all 8 node type specs with their declared inputs/outputs. This is the blueprint.
2. `apps/desktop/src-tauri/src/workflow/engine.rs` (lines 130-250) — DAG walker: how edges are parsed, how `incoming_edges` collects multi-input by target handle name, how `incoming_value` is passed to executors.
3. `apps/desktop/src-tauri/src/workflow/executors/llm.rs` — LLM executor: takes single `incoming` value, builds prompt. Note how it ignores system/context handles.
4. `apps/desktop/src-tauri/src/workflow/executors/tool.rs` — Tool executor: takes single `incoming` value as tool_input. No schema-driven parameter mapping.
5. `apps/desktop/src-tauri/src/workflow/executors/transform.rs` — Transform executor: merges incoming object keys with global inputs, resolves `{{ref}}` templates. This already handles multi-input correctly.
6. `apps/ui/src/app/pages/NodeEditorPage.tsx` (lines 150-320) — Custom node React components. Check which handles are rendered: LLM has only `prompt` in / `response` out. Transform has dynamic inputs from `data.inputs[]`. Router has dynamic branch outputs.
7. `apps/desktop/src-tauri/templates/translation-pipeline.json` — Example template showing a multi-input Transform node in practice.
8. `apps/desktop/src-tauri/templates/meeting-notes.json` — Example of parallel LLM nodes feeding into a Transform (demonstrates the workaround pattern for missing LLM multi-output).

## What to Look For

1. **Engine readiness**: The Rust engine (`engine.rs` lines 228-239) already builds a JSON object from multiple incoming edges keyed by target handle name. Is this sufficient for full multi-handle support, or does it need changes? Are there edge cases (e.g., multiple edges to the same handle, type mismatches)?

2. **LLM executor gap**: The spec defines `system` and `context` as separate input handles. Currently, `systemPrompt` is a static config field. Should `system` become a connectable handle (wired from another node's output), a static config field, or both? What's the best UX — separate handles or a merged approach?

3. **LLM output gap**: The spec defines 3 outputs (response, usage, cost). The executor already returns `{ content, __usage }` but the engine flattens to `node_outputs[node_id] = value`. Should outputs be keyed by handle name (e.g., `response`, `usage`, `cost`) so downstream nodes can reference `{{llm_1.usage.total_tokens}}`?

4. **Tool dynamic handles**: The spec says Tool node inputs should be "dynamic — generated from the tool's input schema (discovered via MCP)." This requires: (a) fetching tool schema at design time, (b) rendering per-parameter handles in the UI, (c) mapping handles to tool_input fields at execution time. Is this the right approach, or should Tool nodes keep a single JSON input and let the user wire a Transform node to build the JSON?

5. **Handle type checking**: The spec defines `HandleDataType` (text, json, image, boolean, etc.). Currently there's zero type checking — any handle connects to any handle. Should we add validation? If so, should it be at connection time (UI), at validation time (Rust `validate_workflow`), or both?

6. **Backward compatibility**: There are 10 bundled templates and users may have saved workflows using single-handle nodes. How should we handle migration? Should old edges with `sourceHandle: "output"` / `targetHandle: "input"` still work when nodes gain multiple handles?

## Output Format

Provide your review as a markdown file saved to:
**`docs/reviews/node-editor-handles-review-2026-02-18.md`**

Use this structure:

### Header
```
# Node Editor Handle System Review
**Date**: 2026-02-18
**Reviewer**: {Your model name}
**Status**: Draft
```

### Findings Table
| Check | Priority | Verdict | Findings |
|-------|----------|---------|----------|
| {area} | {HIGH/MED/LOW} | {PASS/FAIL/WARN} | {1-2 sentence finding} |

### Actionable Checklist
- [ ] {Action item 1}
- [ ] {Action item 2}

### Architecture Recommendation
For each of the 6 questions above, provide a clear recommendation with rationale. Include code-level suggestions where possible (e.g., "change `NodeOutput::value()` to `NodeOutput::named(HashMap<String, Value>)`").

### Notes (optional)
Any broader observations about the handle system design, comparisons to other node editors (React Flow best practices, Blender, ComfyUI, Node-RED), or Phase 4 implications.
