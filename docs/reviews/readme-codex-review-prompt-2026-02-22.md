# Peer Review: README — Technical Accuracy + Developer Experience

**Date**: 2026-02-22
**Project**: AI Studio (open-source IDE for AI agents)
**Reviewer**: GPT-4.1 / Codex
**Review type**: Technical accuracy + Developer onboarding

---

## Instructions for Reviewer

You are reviewing code in the workspace at `/home/amit/projects/myws/ws01/ai-studio-template`.
Read the files listed below, then provide your review in the output format specified.

## Context

AI Studio is a desktop-native IDE for AI agents built with Tauri 2 (Rust) + React 19 + Python FastAPI sidecar. The README hasn't been updated since Phase 3 — the project is now in Phase 4C with significantly more features. This review focuses on **technical accuracy** (are the numbers, commands, paths, and descriptions correct?) and **developer experience** (can a new contributor clone, build, and understand the project from the README alone?). The README should NOT explicitly name competitors — use capability-based language instead.

## Scope

Review `README.md` for:
- Every number, path, command, and technical claim — verify against actual code
- Quick Start instructions — do they actually work? Are dependencies correct?
- Project structure section — does it match the actual directory layout?
- Architecture diagram — does it reflect current state?
- Tech stack table — is it complete and accurate?
- Design specs table — are all specs listed?
- Keyboard shortcuts — are they correct?
- Any broken links or references to things that don't exist

## Files to Read

Read these files in this order:
1. `README.md` — **THE FILE TO REVIEW**
2. `package.json` — actual scripts, dependencies, monorepo config
3. `apps/ui/package.json` — UI dependencies and scripts
4. `apps/desktop/src-tauri/Cargo.toml` — Rust dependencies
5. `apps/desktop/src-tauri/src/commands/templates.rs` — actual template count and names
6. `apps/desktop/src-tauri/src/workflow/executors/` — list actual executor files (count node types)
7. `apps/sidecar/server.py` (first 50 lines) — actual sidecar endpoints and run command
8. `apps/sidecar/requirements.txt` — actual Python dependencies
9. `CONTRIBUTING.md` — does the README's contributing section match?
10. `docs/specs/` — list all spec files (does the Design Specs table match?)
11. `STATUS.md` (first 90 lines) — actual phase status, feature completeness

## What to Look For

1. **Wrong numbers**: README says "8 node types" — count the actual executor files in `src/workflow/executors/`. README says "10 bundled templates" — count in `templates.rs`. README says "31 unit tests" — how many are there now? README says "7 node executors" — count them. README says "13 command modules" — verify. README says "12 design specifications" — count spec files.

2. **Wrong commands**: Does `npm install` work (monorepo uses pnpm)? Does `npm run tauri:dev` exist? Does `npm run dev` work? Is `cd apps/sidecar && python server.py` the correct sidecar run command? Check `package.json` scripts.

3. **Wrong paths**: `src/app/pages/` — is `NodeEditorPage` still the name or was it renamed to `WorkflowsPage`? Does `src/workflow/` have "7 node executors" or more?

4. **Missing specs**: The Design Specs table lists 12 specs. Are there more now? (streaming-output.md, rich-output.md, eip-data-io-nodes.md, rag-knowledge-base.md, etc.)

5. **Architecture diagram**: Does it show the workflow engine? Does it mention streaming? Is "Agent Layer" still the right name for the sidecar?

6. **Quick Start pain points**: A developer cloning this for the first time — what would trip them up? Missing system dependencies (pnpm? tauri-cli version?)? Wrong Python version? Environment variables needed?

7. **Modules table**: README lists "Node Editor" as the module name. Was it renamed to "Workflows"? Are there 6 modules or has the count changed?

8. **Comparison table**: Currently names ChatGPT, Claude, Cursor, LangFlow explicitly. This needs to be rewritten WITHOUT naming competitors. Suggest capability-based alternatives.

9. **Roadmap section**: Shows Phase 3 as "in progress" and Phase 4 as "planned". Both are wrong — Phase 3 is complete, Phase 4 is in progress with 4A+4B done, 4C in progress.

10. **"What's Built" list**: Missing major features: SSE streaming (all 6 providers), live workflow mode, vision support, 16 node types, data I/O nodes (File Glob, Iterator, Aggregator), Transform with JSONPath, user templates, rich output rendering, ensemble intelligence.

## Output Format

Provide your review as a markdown file saved to:
**`docs/reviews/readme-codex-review-2026-02-22.md`**

Use this structure:

### Header
```
# README Review — Technical Accuracy
**Date**: 2026-02-22
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

### Notes (optional)
Specific corrections with exact line numbers, suggested rewrites for wrong commands, or structural recommendations.
