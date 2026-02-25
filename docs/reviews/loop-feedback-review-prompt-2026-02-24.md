# Peer Review: Loop & Feedback Implementation

**Date**: 2026-02-24
**Project**: AI Studio (open-source IDE for AI agents)
**Reviewer**: Gemini 3 Pro (architecture) + GPT-4.1 / Codex (implementation)
**Review type**: Architecture + Code Quality

---

## Instructions for Reviewer

You are reviewing code in the workspace at `/home/amit/projects/myws/ws01/ai-studio-template`.
Read the files listed below, then provide your review in the output format specified.

## Context

AI Studio is a desktop-native IDE for AI agents built with Tauri 2 (Rust) + React 19 + Python FastAPI. The workflow engine executes visual DAG pipelines. We just added **Loop & Feedback** — 2 new node types (Loop + Exit) that enable iterative refinement: draft-critique-revise loops, agentic search, convergence detection. The implementation reuses 80% of the existing Iterator/Aggregator architecture (subgraph extraction, synthetic graph execution). This is a significant architectural addition because it changes how the Router node outputs data (adding `selectedBranch` to the value), which is a potentially breaking change mitigated by adding "value" to the engine's `extract_primary_text` key priority list.

## Scope

4 commits: `ab9b0fd` through `d8543fa`. New: `loop_node.rs` (~400 lines), `exit.rs` (~25 lines), `LoopNode.tsx`, `ExitNode.tsx`, 2 templates, validation changes, Router output restructure. 26 new Rust unit tests (188 total), 2 new E2E tests (8 total).

## Files to Read

Read these files in this order:

1. `docs/specs/loop-feedback.md` — The spec (peer reviewed). Understand what was designed before reading the implementation.
2. `apps/desktop/src-tauri/src/workflow/executors/loop_node.rs` — **Core file**. Subgraph extraction, synthetic graph builder, levenshtein similarity, 3 exit conditions, 2 feedback modes, 20 unit tests.
3. `apps/desktop/src-tauri/src/workflow/executors/exit.rs` — Pass-through stub (trivial, same pattern as aggregator.rs).
4. `apps/desktop/src-tauri/src/workflow/executors/router.rs` — **Breaking change**: Router output now includes `selectedBranch`. Check backward compatibility.
5. `apps/desktop/src-tauri/src/workflow/engine.rs` — Line 50: `"value"` added to `extract_primary_text` key list. This is the mitigation for the Router output change.
6. `apps/desktop/src-tauri/src/workflow/validation.rs` — Loop↔Exit pairing validation, nesting warnings (lines 98-120).
7. `apps/desktop/src-tauri/src/workflow/executors/iterator.rs` — Reference: the original subgraph extraction pattern that Loop copies from.
8. `apps/ui/src/app/pages/workflow/nodes/LoopNode.tsx` — UI component for inline controls.

## What to Look For

### For Gemini (Architecture)

1. **Router output breaking change**: The Router now wraps its output in `{"selectedBranch": "...", "value": ...}`. Does the `extract_primary_text("value")` mitigation fully cover all downstream template resolution cases? Are there edge cases where `{{router.output}}` or `{{router.result}}` would break?

2. **Evaluator mode correctness**: In `loop_node.rs`, the evaluator mode detects "done" by checking if the synthetic graph produced a non-null output (Exit was reached vs skipped by Router). Is this reliable? Could a Router select "done" but the Exit node still produce Null?

3. **Subgraph extraction reuse**: Loop copies Iterator's BFS pattern but stops at "exit" instead of "aggregator". Is there a way to DRY this into a shared function? Should we?

4. **Feedback mode semantics**: In "append" mode, text is concatenated with `\n---\n`. For non-string JSON values, `stringify_value()` converts to JSON text first. Is this the right behavior, or should append mode reject non-string inputs?

5. **Nesting validation**: The validation warns about Loop+Iterator coexistence and multiple Loops, but doesn't prevent execution. Should these be errors instead of warnings?

### For Codex (Implementation)

1. **Levenshtein correctness**: Two-row DP implementation in `levenshtein_similarity()`. Verify the algorithm is correct, especially the `std::mem::swap` pattern and boundary conditions.

2. **Memory safety in levenshtein**: With 10K char truncation, the DP matrix is 10K * 2 * 8 bytes = ~160KB. Is this acceptable? Any risk of stack overflow?

3. **Edge cases in loop execution**: What happens if the subgraph has no nodes between Loop and Exit (direct connection)? What if the synthetic graph execution returns an error on iteration 0?

4. **Router output type change**: All existing templates use Router output. Check `templates/content-moderator.json` and `templates/email-classifier.json` — do they still work with the new `{"selectedBranch", "value"}` output shape?

5. **UI number input**: LoopNode has `<input type="number" min={1} max={50}>` for maxIterations. What happens if the user types 0 or 100? The Rust side clamps, but the UI doesn't prevent invalid input.

6. **Test coverage gaps**: Are there tests for: empty input to Loop, evaluator mode with no Router in subgraph (error case), stable_output with identical first two outputs (should exit on iteration 2)?

## Output Format

Provide your review as a markdown file saved to:
**`docs/reviews/loop-feedback-review-2026-02-24.md`**

Use this structure:

### Header
```
# Loop & Feedback Review
**Date**: 2026-02-24
**Reviewer**: {Your model name}
**Status**: Draft
```

### Findings Table
| # | Area | Priority | Verdict | Finding |
|---|------|----------|---------|---------|
| 1 | {area} | HIGH/MED/LOW | PASS/FAIL/WARN | {1-2 sentence finding} |

### Actionable Checklist
- [ ] {Action item 1}
- [ ] {Action item 2}

### Notes (optional)
Any architecture recommendations, praise, or broader observations.
