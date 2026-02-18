# Node Editor: Architectural Critique & Future Analysis
**Date:** 2026-02-18
**Reviewer:** Antigravity (Gemini 3 Pro)
**Triaged by:** Claude Opus 4.6
**Status:** RESOLVED — All 4 findings accepted, deferred to Phase 4
**Focus:** Long-term Viability, Extensibility, and Comparison to High-End Tools (Unreal/Blender)

## 1. The Core Limitation: Static vs. Dynamic Pins
**Current State**: Nodes like `LLMNode` have fixed inputs (`prompt`, `systemPrompt`).
**Unreal/Blender Standard**: High-end node editors allow **Dynamic Pins**. For example, a "Format String" node in Unreal lets you add as many input pins as you have variables (`{0}`, `{1}`, etc.).
**Critique**: The current implementation relies on `{{handlebars}}` parsing to "pull" data from other nodes. This is a **Web/SaaS pattern**, not a **Visual Programming pattern**.
- **Pros**: Easy to implement; familiar to web developers.
- **Cons**: Hides data dependencies inside text strings. You cannot visualize the flow of data because the "wire" is implicit in the text, not explicit on the graph.
**Recommendation**: Move to a **Dynamic Handle System**.
- If a user types `{{research_summary}}` in a prompt, the Node should *automatically generate* an input handle named `research_summary`.
- This forces the user to drag a wire to it, making the data flow explicit and debuggable.

## 2. Type Safety & Data Marshaling
**Current State**: All data is passed as `serde_json::Value`. It's "Stringly Typed."
**Unreal/Blender Standard**: Strictly typed pins (Color outputs connect only to Color inputs; Float to Float).
**Critique**:
- **Loose Coupling**: Good for LLMs (text in, text out).
- **Fragile Transforms**: If a user connects a `JSON Object` output to a node expecting `String`, the failure happens at *runtime* inside the Rust backend (`execute_workflow`).
**Recommendation**: Implement **Visual Type Coercion**.
- The UI should color-code connections (String=Gray, JSON=Gold, Image=Purple).
- If a mismatch occurs (connecting JSON to String), automatically insert a "To String" conversion node or visually warn the user *before* execution.

## 3. Execution Model: DAG vs. Graph
**Current State**: Directed Acyclic Graph (DAG). No loops allowed.
**Unreal/Blender Standard**: Full Turing-complete Graphs with `For Loop`, `While`, and State memory.
**Critique**:
- The current `execute_workflow` is a topological sort. It cannot handle "Refine this draft until it is good" (a loop).
- Useful for pipelines, limiting for agents.
**Recommendation**: Shift to a **Step-Based Execution Engine**.
- Instead of "Sort -> Run All", use a "Pointer" execution model (like a program counter).
- This allows for `Loop` nodes and `If/Else` blocks that can cycle back to previous nodes for self-correction.

## 4. Extensibility (The "Plugin" Problem)
**Current State**: Node types (`llm`, `router`) are hardcoded in Rust `commands.rs`. A user cannot add a logical node without recompiling the app.
**Unreal/Blender Standard**: Custom nodes via Python/C++.
**Critique**: As the tool grows, you will need 100+ node types (PDF Reader, Gmail, Slack, etc.). Hardcoding them in Rust is not scalable.
**Recommendation**: Define a **"Script Node" Protocol**.
- Allow users to write a small Python/JS script that defines:
  1. Inputs/Outputs
  2. Execution Logic
- The Rust backend executes this script in a sandbox V8/Python environment.
- This creates an ecosystem where the community can share custom nodes.

## 5. Summary
The current implementation is a solid **LLM Chain Builder** (like LangChain/Flowise). It is **not yet** a full **Visual Programming Environment** (like Unreal Blueprints).

To bridge the gap to "Blender-quality":
1.  **Dynamic Handles**: Make data dependencies explicit wires.
2.  **Type Visualization**: Color-coded data streams.
3.  **Cyclic Execution**: Allow loops for iterative refinement.
4.  **User-Defined Nodes**: Scriptable logic blocks.

This architecture is robust enough to *evolve* into that, but these are the specific pivots required to get there.

---

## Triage (Claude Opus 4.6, 2026-02-17)

| # | Finding | Verdict | Notes |
|---|---------|---------|-------|
| 1 | Dynamic Handles | **Accept — Deferred to Phase 4** | Strongest point. Parse `{{var}}` in prompt → auto-generate input handles. React Flow supports dynamic handles. Makes data flow explicit. |
| 2 | Type Visualization | **Accept — Deferred to Phase 4** | Handle dots already color-coded (ba82190). Extend to mismatch warnings + auto-coercion nodes. |
| 3 | Cyclic Execution | **Accept with caveat — Deferred to Phase 4** | Valid for agentic "refine until good" loops. Implement as Loop node with `max_iterations` wrapping a sub-DAG, NOT full Turing-complete graphs. DAG safety constraint preserved. |
| 4 | User-Defined Nodes | **Accept — Already in backlog** | Exactly the Plugin System planned for Phase 3 backlog. Sandbox execution (V8/Python) is the right call. |

**Overall**: Accurate diagnosis. We are a "solid LLM Chain Builder" evolving toward "Visual Programming Environment." All 4 pivots are the right direction. They define Phase 4's scope.
