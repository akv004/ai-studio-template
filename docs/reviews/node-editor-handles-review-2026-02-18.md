# Node Editor Handle System Review
**Date**: 2026-02-18
**Reviewer**: Antigravity (Gemini)
**Status**: RESOLVED

### Findings Table
| Check | Priority | Verdict | Findings |
|-------|----------|---------|----------|
| **Engine readiness** | HIGH | PASS | The Rust engine (`engine.rs`) correctly aggregates multiple incoming edges into a JSON object derived from `targetHandle` names. It also maintains backward compatibility by flattening single inputs named "input". |
| **LLM executor gap** | HIGH | FAIL | The `LLMNode` UI only exposes a `prompt` input. The `LlmExecutor` logic ignores any incoming keys other than "input" (implicitly) and relies on static configuration for `systemPrompt`. |
| **LLM output gap** | MED | WARN | The executor returns a bundled object `{ content, __usage }`. While downstream nodes can access fields via templates (e.g., `{{llm.usage}}`), there are no explicit output handles for `usage` or `cost` to visually route these values. |
| **Tool dynamic handles** | HIGH | FAIL | The `ToolNode` UI renders a single generic `input` handle. There is no mechanism to fetch tool schemas and generate dynamic per-parameter handles. The executor expects a single JSON input object. |
| **Handle type checking** | MED | FAIL | There is no enforcement of handle types (e.g., preventing a text output from connecting to a JSON input). Visual classes (`handle-text`) exist but are decorative. |
| **Backward compatibility** | HIGH | PASS | The engine's flattening logic ensures that existing workflows with single-input nodes continue to function without migration. |

### Actionable Checklist
- [x] **LLM Node UI**: Already implemented — 3 inputs (system/context/prompt) + 3 outputs (response/usage/cost). (Pre-review, by user)
- [x] **LLM Executor**: Already implemented — reads system/context from incoming edges, falls back to config. (Pre-review, by user)
- [x] **Tool Node**: Accepted recommendation — keep single JSON input, defer schema-driven handles to Phase 4.
- [x] **Validation**: Implemented `isValidConnection` with type-aware connection rules. (2026-02-18)

### Architecture Recommendation

#### 1. Engine Readiness
**Recommendation**: **Approved / No Change Required.**
The recent updates to `engine.rs` (checking `inc.len() == 1` vs multiple edges) provide the necessary foundation. The logic correctly handles both the "simple single input" case (flattening) and the "multi-input map" case. This is sufficient for Phase 4.

#### 2. LLM Executor Gap
**Recommendation**: **Implement Hybrid Configuration.**
We should support both static config and dynamic inputs.
- **UI**: Add toggles in the LLM config panel: "Show System Input" and "Show Context Input". When enabled, render the handles.
- **Executor**: Update `LlmExecutor` to prioritize incoming edge data over static config.
  ```rust
  let system_prompt = incoming.get("system").or(previous_config).unwrap_or("");
  ```
- **Rationale**: This closes the gap with the spec while keeping the simple case easy to use.

#### 3. LLM Output Gap
**Recommendation**: **Defer Explicit Handles.**
While separate handles for `usage` and `cost` would be "clean", they add visual clutter for data that is rarely routed independently.
- **Approach**: Continue returning the bundled object. Reliance on template access (e.g., `{{node.usage.total_tokens}}`) is consistent with how `Transform` nodes work.
- **Phase 4**: If we introduce a "Math" node that strictly requires number types, we can add a specific `cost` output handle then.

#### 4. Tool Dynamic Handles
**Recommendation**: **Defer to Phase 4.**
Implementing schema-driven handles requires:
1.  Frontend fetching of MCP tool schemas (currently not exposed to the node graph).
2.  Dynamic UI generation of handles.
3.  Executor logic to map handles back to arguments.
**Decision**: Stick to the "Transform -> Tool" pattern for now. Users use a Transform node to build the specific JSON input required by the tool.

#### 5. Handle Type Checking
**Recommendation**: **UI-Only Validation.**
Strict backend type checks are difficult due to the loose nature of JSON.
- **Immediate Fix**: Implement `isValidConnection` prop in React Flow to check that `sourceHandle` and `targetHandle` have compatible "types" (encoded in their ID or class).
- **Long Term**: Add a `validate` pass in Rust that checks edge compatibility before execution.

#### 6. Backward Compatibility
**Recommendation**: **Implicit Migration.**
The engine's current behavior handles migration transparently.
- Old Workflows: Single edge to "input" -> Engine flattens -> Executor sees direct value.
- New Workflows: Multiple edges to "named_handle" -> Engine creates object -> Executor sees map.
No database migration is required.

### Notes
The implementation of dynamic inputs for `Transform` nodes serves as the reference implementation for "Un-deferring" the LLM inputs. We should apply the same pattern to the LLM node immediately. Tool nodes are significantly more complex and should remain in the backlog for Phase 4.
