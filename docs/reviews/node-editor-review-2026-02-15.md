# Node Editor Architecture Review
**Date**: 2026-02-15
**Reviewer**: Gemini 3 Pro
**Status**: Triaged

### Findings Table

| Check | Priority | Verdict | Findings |
|-------|----------|---------|----------|
| **Execution Engine** | HIGH | **WARN** | Placing the DAG walker in Rust is correct for performance/stability, BUT it creates a split-brain state problem. Rust knows the graph state, but Python owns the conversation state. |
| **Sidecar Statelessness** | HIGH | **FAIL** | **Critical**: The spec relies on `POST /chat` (stateful). In a DAG, efficient context management is key. Reusing a single session ID pollutes context (Branch A sees Branch B's messages). Using unique IDs means zero context. **Fix**: Use `POST /chat/direct` (stateless) and have Rust build the context window per node. |
| **Schema Design** | MED | **PASS** | Storing `graph_json` is the industry standard for node-based tools (like ComfyUI). Normalizing nodes/edges into SQL tables creates strict schema coupling that breaks easily with UI updates. JSON is correct here. |
| **Event Integration** | MED | **PASS** | `workflow.node.*` events nest perfectly into the existing schema. The `seq` contract ensures the Inspector can replay the graph execution linearly. |
| **Router Complexity** | LOW | **PASS** | "LLM Classify" is not over-engineered; it's essential for "fuzzy" logic (e.g., "is this sentiment positive?"). Using a small local model (Ollama) makes this fast and free. |
| **Missing Pieces** | HIGH | **WARN** | **Concurrency**: Spec doesn't mention parallel execution of branches. Rust `async` supports it, but the Python sidecar is single-threaded (GIL) unless using multiple workers. We need to ensure the sidecar can handle concurrent requests from the same workflow run. |

### Actionable Checklist

- [x] **Redesign Sidecar Interface**: Switch Node Editor execution to use `POST /chat/direct` (stateless). Rust should aggregate the history for each node based on the graph traversal path and send the full context window. (Fixed 2026-02-15 — spec updated: execution model mandates `/chat/direct`, Rust owns all context)
- [x] **Define Parallelism**: Explicitly state if the Rust walker executes branches in parallel (`join_all`) or sequence. If parallel, verify Sidecar `uvicorn` worker count. (Fixed 2026-02-15 — spec updated: parallel via `tokio::join_all`, default concurrency 4, sidecar worker count documented)
- [x] **Version Clarification**: Spec says Schema v4, Review Prompt says v5. Resolve the version number conflict. (Fixed 2026-02-15 — spec is correct: v4. Review prompt had a typo.)
- [x] **Subworkflow Cycles**: Add a check in `validate_workflow` to prevent infinite recursion (A -> B -> A) since subworkflows are just nodes. (Fixed 2026-02-15 — spec updated: validation table + validate_workflow command both include subworkflow cycle detection)
- [ ] **Mock Tests**: Add a specific requirement for "Headless Workflow Tests" where we run a mock workflow without the UI to verify the Rust->Python loop. (Deferred to Phase 3B — test infrastructure comes when execution engine is built)

### Triage Notes

**All HIGH items addressed.** The reviewer correctly identified the most critical issue: sidecar statelessness. We already have `/chat/direct` in the sidecar — the spec just wasn't using it. Now it mandates stateless execution for all workflow LLM calls, with Rust building context windows per node. This aligns with the lesson from session branching bugs.

The reviewer's overall note about data flowing through edges (Rust) while chat context lives in Python is exactly right — and exactly what the spec fix resolves. The sidecar becomes a pure compute engine for workflow execution.

### Notes
The architecture is solid overall, but the **State Management** strategy is the weak point. In valid node graph architectures (like LangChain or ComfyUI), data flows *through* the edges. In the proposed design, data flows through edges (Rust) but "chat context" lives invisibly in Python variables. This disconnect will lead to bugs. Make the Sidecar purely a compute engine (stateless) for the Node Editor.
