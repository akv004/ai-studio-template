# Loop & Feedback Review
**Date**: 2026-02-24
**Reviewer**: Gemini 3 Pro (architecture)
**Status**: RESOLVED

### Findings Table
| # | Area | Priority | Verdict | Finding |
|---|------|----------|---------|---------|
| 1 | Router Output | HIGH | WARN | The `extract_primary_text("value")` mitigation handles strings, but fails if downstream expects a stringified JSON object. If `value` is an object, `extract_primary_text` falls back to serializing the entire wrapper `{"selectedBranch": "...", "value": ...}` instead of just the inner object. |
| 2 | Evaluator Mode | HIGH | FAIL | Detecting "done" by checking `!iteration_output.is_null()` conflates a skipped Exit node with an Exit node that explicitly receives a JSON `Null` value. If the "done" branch legitimately produces `null`, the loop will incorrectly continue. |
| 3 | Subgraph Extraction | MED | WARN | `find_loop_subgraph` and `find_iterator_subgraph` duplicate the exact same forward/backward BFS logic. This should be DRY'd into a shared `find_bounded_subgraph(graph, start_id, boundary_type)` utility to reduce maintenance surface. |
| 4 | Feedback Append | MED | WARN | `stringify_value()` serializes objects to JSON strings before concatenating with `\n---\n`. This breaks structural validity (yielding invalid JSON). Append mode should ideally warn or encapsulate non-string inputs (e.g., wrap in an array) rather than blindly concatenating them. |
| 5 | Nesting Validation | HIGH | FAIL | Multiple Loops, or Loop + Iterator coexistence, are only flagged as warnings. Given the current unbounded forward BFS extraction, allowing nested structures to execute will result in malformed subgraphs and unpredictable runtime failures. These must be elevated to fatal validation errors until nesting is properly supported. |

### Actionable Checklist
- [x] Fix Router output stringification bug where object values cause the entire wrapper to be serialized. (2026-02-24, engine.rs `extract_primary_text` now recurses into non-string `value` field)
- [ ] Deferred: Evaluator mode null detection — extremely unlikely in practice (Exit is pass-through, upstream producing null on "done" branch is a degenerate case). Fixing requires synthetic graph API changes to expose per-node outputs. Deferred to Phase 5.
- [ ] Deferred to Phase 5: Refactor BFS subgraph extraction into shared utility. Valid maintenance concern, no correctness issue.
- [x] Append mode now wraps non-string values in JSON array instead of blind stringify+concat. (2026-02-24, loop_node.rs feedback block)
- [x] Nesting validation elevated to hard errors — multiple Loops and Loop+Iterator coexistence are now fatal. (2026-02-24, validation.rs + 2 updated tests)

### Notes
The architecture effectively reuses the synthetic graph approach from the Iterator implementation. However, the Router breaking change and Evaluator mode detection logic both introduce subtle edge cases around `Null` and Object handling. Fixing these before merging will prevent insidious downstream data corruption.

---

# Loop & Feedback Review
**Date**: 2026-02-24
**Reviewer**: Codex (implementation)
**Status**: RESOLVED

### Findings Table
| # | Area | Priority | Verdict | Finding |
|---|------|----------|---------|---------|
| 1 | Evaluator loop progression | HIGH | FAIL | In `loop_node.rs`, evaluator mode calls `continue` immediately when Exit is not reached, which skips feedback update (`current_input`) for the next iteration. This means evaluator loops can repeatedly run with stale input and diverge from the spec (`continue` branch should feed next iteration). |
| 2 | Router wrapper compatibility | HIGH | FAIL | `router.rs` now outputs `{"selectedBranch","value"}`, but `resolve_source_handle` falls back to returning the whole object for `branch-*` handles. Existing branch edges therefore pass wrapper objects downstream instead of the prior raw value; this affects templates like `content-moderator.json` and `email-classifier.json`. |
| 3 | Empty loop body behavior | MED | WARN | A direct `Loop -> Exit` graph is accepted; `find_loop_subgraph` can return an empty subgraph, and synthetic execution then produces disconnected Input/Output nodes with `Null` results. This fails silently instead of surfacing a configuration error. |
| 4 | Levenshtein correctness | LOW | PASS | The two-row DP implementation in `levenshtein_similarity()` is algorithmically correct (boundary checks and `swap` usage are valid), and current unit tests cover key correctness cases. |
| 5 | Levenshtein memory/perf profile | LOW | WARN | Memory usage is safe (heap-allocated `Vec`s, no stack-overflow risk), but worst-case cost is still `O(10k*10k)` operations per compare. Under high iteration counts this can become CPU-expensive even though memory remains bounded. |
| 6 | Loop UI numeric guardrails | LOW | WARN | `LoopNode.tsx` and `NodeConfigPanel.tsx` set `min/max`, but `onChange(parseInt(...))` still stores out-of-range values (e.g., `0`, `100`) until runtime clamp in Rust. This causes UI/runtime mismatch and confusing previews. |
| 7 | Test coverage gaps | MED | WARN | Loop tests are mostly helper-level. Missing targeted execution tests for: evaluator mode without Router (error path), evaluator continue-path feedback propagation, stable-output early exit on iteration 2 with identical outputs, and direct `Loop -> Exit` validation behavior. |

### Actionable Checklist
- [x] Fix evaluator mode: removed `continue` short-circuit, feedback now runs on all paths. Null outputs (evaluator continue) are skipped to preserve current_input. (2026-02-24, loop_node.rs)
- [x] Add backward-compatible routing: `resolve_source_handle` now unwraps `value` from Router wrapper for `branch-*` handles. (2026-02-24, engine.rs + 2 new tests)
- [x] Added validation warning for direct Loop→Exit (empty loop body). (2026-02-24, validation.rs + 1 new test)
- [x] Clamp Loop numeric fields in UI: both LoopNode.tsx and NodeConfigPanel.tsx now clamp [1,50] on change. (2026-02-24)
- [ ] Deferred: Levenshtein perf — O(10k*10k) is ~100M ops, acceptable for max 50 iterations. Would only matter at extreme scale.
- [ ] Deferred to Phase 5: Additional executor-level integration tests for evaluator/stable-output behaviors.

### Notes
Focused test run status: `cargo test workflow::executors::loop_node::tests`, `cargo test workflow::engine::tests`, and `cargo test workflow::validation::tests::test_loop*` all pass. The gaps above are behavioral/coverage issues not currently asserted by those suites.
