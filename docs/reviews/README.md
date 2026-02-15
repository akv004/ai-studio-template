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
| Review response | `{topic}-review-{date}.md` | `phase-2-review-detailed.md` |
| Closed review | Same file, status updated at top | `**Status**: RESOLVED` |

## Item Lifecycle

```
- [ ] Open           → New finding from reviewer
- [x] Fixed          → Implemented (date, commit hash)
- [ ] Rejected: ...  → Invalid or incorrect finding
- [ ] Deferred to P3 → Valid but not for this phase
```

## Which Reviewers to Use

| Reviewer | Best for | How to invoke |
|----------|----------|---------------|
| Gemini 3 Pro (Antigravity) | Architecture, data model, API design | Open Antigravity in same workspace |
| ChatGPT 5.2 | Security review, edge cases, UX critique | Paste prompt or open in workspace |
| Claude Code (self-review) | Quick sanity checks, code consistency | Ask Claude Code to review its own changes |

## Past Reviews

| Date | Topic | Reviewer | Status | File |
|------|-------|----------|--------|------|
| 2026-02-09 | Phase 1 design | Gemini 3 Pro | Closed | `design-review-2026-02-09.md` |
| 2026-02-15 | Phase 2 branching | Gemini 3 Pro | Resolved | `phase-2-review-detailed.md` |
