# Peer Review Workflow

Multi-model design reviews using Claude Code + external AI reviewers (Antigravity/Gemini, ChatGPT, etc).

## How It Works

### Step 1: Claude Code generates the prompt

Claude Code writes a self-contained review prompt to this directory:

```
docs/reviews/{topic}-review-prompt-{date}.md
```

The prompt includes:
- What to review (feature, phase, or specific concern)
- Files to read (exact paths — reviewer runs in same workspace)
- What to look for (bugs, architecture, security, perf, UX)
- Expected output format (table + checklist)

### Step 2: You run the review

Open the external AI tool in the **same workspace** and say:

> "Review per the prompt in `docs/reviews/{topic}-review-prompt-{date}.md`"

The tool reads the files, writes its response as a markdown file in this directory.

### Step 3: Claude Code triages and fixes

Tell Claude Code where the review response is. It will:
1. Read each finding
2. Triage: **Accept** (fix now), **Reject** (explain why), or **Defer** (to later phase)
3. Implement accepted fixes
4. Mark items closed with date and commit hash
5. Commit everything

## File Naming

| Type | Pattern | Example |
|------|---------|---------|
| Review prompt | `{topic}-review-prompt-{date}.md` | `phase-2-review-prompt-2026-02-15.md` |
| Review response | `{topic}-review-{date}.md` | `phase-2-review-2026-02-15.md` |
| Closed review | Same file, status updated at top | `**Status**: RESOLVED` |

## Item Lifecycle

```
- [ ] Open           → New finding from reviewer
- [x] Fixed          → Implemented (date, commit hash)
- [ ] Rejected: ...  → Invalid or incorrect finding
- [ ] Deferred to P3 → Valid but not for this phase
```

## Which Reviewer for What

**Rule of thumb:** Antigravity/Gemini = "zoom out" (architecture, strategy). Codex/GPT = "zoom in" (code bugs, edge cases).

| Review Type | Reviewer | Tool | Why |
|---|---|---|---|
| **Architecture / Data Model** | Gemini 3 Pro | Antigravity | System design, schema analysis, cross-layer consistency |
| **Code Quality / Bugs** | GPT-4.1 / Codex | VS Code Codex | Code-level bugs, edge cases, type errors, off-by-one |
| **Security** | Gemini 3 Pro | Antigravity | OWASP-style analysis, auth flows, injection risks |
| **Performance** | GPT-4.1 / Codex | VS Code Codex | Profiling suggestions, runtime analysis, N+1 queries |
| **UX / API Design** | Either | Either | Both good — use whichever is open |
| **Open-source readiness** | Gemini 3 Pro | Antigravity | Big-picture "would this impress people" evaluation |
| **Quick sanity check** | Claude Code | Claude Code | Self-review for code consistency, missed patterns |

**When in doubt:** The review prompt (Step 1) will specify which reviewer Claude Code recommends.

## Cross-Project Review Standard

This workflow follows a standard multi-model review pattern used across projects:

1. **Collect** — Reviews go in `docs/reviews/` with naming `design-review-YYYY-MM-DD.md`. Include reviewer name/tool, date, and status (Draft/Triaged/Closed). Each item is a checkbox `- [ ]`.
2. **Triage critically** — Not all feedback is correct. For each item: **Accept** (add to backlog), **Reject** (explain why with evidence), or **Already Planned** (reference existing spec/task).
3. **Build & Close** — When implemented, change `- [ ]` to `- [x]` with closing note: `(Closed YYYY-MM-DD, commit abc1234)`. Commit review doc update alongside implementation.
4. **Close the Review** — When all items resolved, update status from `Draft` to `Closed`. Final commit: `Docs: Close design review YYYY-MM-DD — all items resolved`.

**Rules:**
- Never blindly accept all review items — validate against specs and existing code
- If a reviewer claims something is missing but it exists, reject with evidence
- Cross-reference review items with STATUS.md backlog to avoid duplicates
- Reviews from any AI model or human follow the same process

## Past Reviews (Archived)

All resolved reviews have been deleted. Summary of completed reviews:

| Date | Topic | Reviewer | Outcome |
|------|-------|----------|---------|
| 2026-02-09 | Phase 1 design | Gemini 3 Pro | 5 items — all fixed |
| 2026-02-15 | Phase 2 branching | Gemini 3 Pro | 6 items — context loss + transaction safety fixed |
| 2026-02-15 | Node editor architecture | Gemini 3 Pro | 5 items — /chat/direct + parallelism fixed |
| 2026-02-18 | Runtime safety | Codex (GPT-5) | 8 items — 4 fixed, 4 deferred to 3C |
| 2026-02-18 | Platform architecture | Codex (GPT-5) | 8 items — 7 fixed in 3C (2380b83, 280be8c), 1 deferred (Windows fuser) |
| 2026-02-18 | Node editor 3C polish | Antigravity (Gemini) | APPROVED — 3 minor suggestions deferred to Phase 4 (skipped edge styling, merge node, template thumbnails) |
| 2026-02-18 | Node editor architecture critique | Antigravity (Gemini) | 4 findings — all accepted, deferred to Phase 4 (dynamic handles, type viz, loops, script nodes) |
| 2026-02-17 | Hybrid intelligence | Antigravity (Gemini) | 6 checks PASS, 1 WARN (timezone). 3 items: 1 accepted (model list TODO), 1 rejected, 1 deferred (retry fallback) |
| 2026-02-17 | Hybrid intelligence deep critique | Simulated ChatGPT 5.2 | 4 findings: 1 fixed (budget enforcement — 5117302), 2 deferred (token estimation, supports_tools), 1 rejected (PII) |
| 2026-02-18 | Node editor handle system | Antigravity (Gemini) | 6 checks: 2 PASS, 2 already fixed (LLM multi-handle), 1 accepted (type validation — ebeddbb), 1 deferred (tool schema handles) |
| 2026-02-18 | README + review workflow docs | Codex (GPT-5.2) | 8 findings: 3 fixed (tauri-cli, naming, external ref — ebeddbb), 2 deferred (venv, sidecar script), 2 rejected (prompt file format), 1 accepted for future (commit hash) |
| 2026-02-18 | Phase 4 spec — architecture | Antigravity (Gemini 3 Pro) | 5 findings — all accepted. Container model validated, engine refactoring confirmed, scope isolation added. |
| 2026-02-18 | Phase 4 spec — implementation | Codex (GPT-5.2) | 10 findings — 9 accepted (spec updated to v1.2), 1 deferred (F10 event consistency → 4C). Found 2 existing engine bugs (sourceHandle, clean_output). Best review yet. |
| 2026-02-18 | Phase 4 combined verdict | Codex (GPT-5.3) | 12 findings — CF1/CF2 already fixed (ff2b271), CF3/CF7/CF11 deferred (container model, router IDs, code sandbox). Verdict: "Continue Phase 4, prioritize engine fixes before more nodes." |
| 2026-02-18 | Node field model coherence | Codex (GPT-5.3) | 8 findings — F2 already fixed (ff2b271), F7 deferred (router branch IDs). Fields being wired incrementally per executor. |
