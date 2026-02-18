# Docs Review (README + Reviews + Review Prompts)
**Date**: 2026-02-18
**Reviewer**: Codex (GPT-5.2)
**Status**: RESOLVED

### Findings Table
| Area | Priority | Verdict | Findings |
|------|----------|---------|----------|
| `README.md` quick start completeness | MED | WARN | `npm run tauri:dev` depends on `cargo tauri` (Tauri CLI) but the quick start path does not mention installing `tauri-cli` unless the reader expands the detailed setup section. |
| `README.md` Python guidance | LOW | WARN | `pip install -r apps/sidecar/requirements.txt` is fine, but there is no recommended virtualenv flow even though `apps/sidecar/.venv/` exists in-repo. Optional deps like Playwright may also require `playwright install`. |
| `docs/reviews/` documentation consistency | HIGH | FAIL | `docs/reviews/README.md` references `claude-config/rules/design-reviews.md`, but that path does not exist in this repo. |
| `docs/reviews/` file naming examples | MED | FAIL | The "Review response" example (`phase-2-review-detailed.md`) does not match the stated `{topic}-review-{date}.md` pattern used elsewhere (e.g., `node-editor-handles-review-2026-02-18.md`). |
| Review folder placement | LOW | PASS | Keeping reviews under `docs/reviews/` is a good default: they are versioned, discoverable, and close to specs. |
| `docs/reviews/node-editor-handles-review-prompt-2026-02-18.md` prompt structure | LOW | PASS | The prompt is self-contained: context, scope, ordered file list, explicit evaluation questions, and a strict output format that writes into `docs/reviews/`. |
| `docs/reviews/node-editor-handles-review-prompt-2026-02-18.md` prompt staleness risk | HIGH | FAIL | The prompt embeds specific claims about the current implementation (e.g., “LLM has only `prompt` in / `response` out”, “LLM executor ignores system/context”) which are not true in the current workspace (`apps/ui/src/app/pages/NodeEditorPage.tsx` shows `system/context/prompt` inputs and `response/usage/cost` outputs; `apps/desktop/src-tauri/src/workflow/executors/llm.rs` reads `system`/`context`). This can mislead reviewers if code changes after prompt generation. |
| `docs/reviews/node-editor-handles-review-prompt-2026-02-18.md` run instructions (“ACTION REQUIRED”) | MED | WARN | The prompt itself does not include the “Open Antigravity / Workspace / Say … / Save to …” steps. That instruction likely came from the generating agent’s chat output, not a file. If you want repeatability, put the run instructions directly in the prompt header (or standardize in `docs/reviews/README.md`). |
| `docs/reviews/node-editor-handles-review-prompt-2026-02-18.md` reproducibility anchor | MED | WARN | The prompt references approximate line ranges, but does not record a commit hash. Adding `git rev-parse HEAD` at prompt creation time avoids “reviewed different code than intended” situations. |

### Actionable Checklist
- [x] Update `docs/reviews/README.md` to point at an in-repo standard (or remove the external reference). (2026-02-18 — removed external ref)
- [x] Fix the naming examples in `docs/reviews/README.md` to match the stated patterns. (2026-02-18 — fixed example)
- [x] Consider adding a 1-line note to `README.md` Quick Start: install Tauri CLI (`cargo install tauri-cli`). (2026-02-18 — added step 2)
- [ ] Deferred: Python venv snippet in README — nice to have, not blocking
- [ ] Deferred: mention `npm run sidecar` in README — nice to have
- [ ] Rejected: Replace prompt claims — prompt is a point-in-time artifact, staleness is expected when code changes between prompt and review
- [ ] Rejected: ACTION REQUIRED in prompt file — instructions are given in chat output by design
- [x] Accepted for future: Add commit hash to review prompts — good practice, will apply to next `/peer-review`

### Notes
- The current `docs/reviews/` workflow is solid and already includes prompt/response separation. The main value add here is tightening internal references and examples so the docs are self-contained for new contributors.
- Consolidates findings from `docs/reviews/node-editor-handles-review-prompt-quality-review-2026-02-18.md` into this single file.

**Signature**: Codex (GPT-5.2) — 2026-02-18
