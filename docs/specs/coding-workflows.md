# Coding Workflows

**Status**: DRAFT
**Phase**: 5A
**Author**: AI Studio PM
**Date**: 2026-02-25

---

## Problem Statement

AI Studio can build AI pipelines (summarize, translate, search, moderate) but cannot do **coding tasks** — the #1 use case for AI power users. Tools like Cursor, Claude Code, and Copilot Workspace prove the demand: developers want AI to read code, generate fixes, run tests, and iterate until green.

AI Studio has 80% of the building blocks already (File Read/Write/Glob, Shell Exec, LLM, Loop, Knowledge Base). The missing pieces prevent the last mile:

1. **No surgical file editing** — File Write overwrites entire files. An LLM can't change line 47 without regenerating 500 lines. This is fragile, token-expensive, and error-prone on large files.
2. **No diff/patch support** — The standard way to express code changes (unified diff) has no node support. LLMs are trained to output diffs — we can't consume them.
3. **No git operations** — Commit, branch, diff, status are all shell commands today. A dedicated node would be safer (validation, confirmation) and more discoverable.
4. **No test feedback loop template** — The pattern "edit → test → check → fix → repeat" doesn't exist as a ready-made template.

---

## Design

### New Node: `code_edit`

A node that applies surgical edits to a file without rewriting the whole thing.

**Why not just use File Write?**
File Write = overwrite. For a 500-line file where the LLM wants to change 3 lines, you'd need the LLM to output all 500 lines perfectly. One missed import, one wrong indent, and the file is corrupted. Code Edit takes a targeted change instruction instead.

#### Config

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| mode | enum | `search_replace` | Edit strategy (see below) |
| createIfMissing | bool | false | Create file if it doesn't exist |

#### Edit Modes

**`search_replace`** (default, safest)
- Input: `{ "file": "/path/to/file.rs", "old": "fn foo() {", "new": "fn foo() -> Result<()> {" }`
- Exact string match → replace. Fails if `old` not found (no silent corruption).
- Multiple edits: accept array of `{old, new}` pairs, applied top-to-bottom.
- This is the same pattern Claude Code and Aider use — proven to work with LLMs.

**`unified_diff`**
- Input: `{ "file": "/path/to/file.rs", "diff": "--- a/file.rs\n+++ b/file.rs\n@@ -47,3 +47,3 @@..." }`
- Parse unified diff format, apply hunks with context matching.
- Fuzzy context matching (±3 lines offset tolerance) for when line numbers shift.
- This is how `git apply` works — LLMs are well-trained on this format.

**`line_range`**
- Input: `{ "file": "/path/to/file.rs", "startLine": 47, "endLine": 49, "content": "new content here" }`
- Replace specific line range. Simplest mode, works when LLM knows exact line numbers (e.g., from File Read output that includes line numbers).

#### Handles

| Handle | Direction | Type | Description |
|--------|-----------|------|-------------|
| input | target | json | Edit instruction(s) — format depends on mode |
| file | target | text | File path (alternative to including in input JSON) |
| output | source | text | The modified file content (full file after edit) |
| diff | source | text | What actually changed (unified diff of before→after) |

#### Executor (`code_edit.rs`)

```
1. Read file from disk (or create if createIfMissing)
2. Apply edit based on mode:
   - search_replace: find exact `old` string, replace with `new`
   - unified_diff: parse hunks, apply with context matching
   - line_range: splice content at line range
3. Write modified file back to disk
4. Generate unified diff of before→after
5. Return { content: modified_file, diff: unified_diff }
```

Safety:
- Reuse `is_path_denied()` from File Read/Write — same security boundary
- Atomic write (write to temp, rename) — same as File Write
- Backup original to `.ai-studio-backup/{filename}.{timestamp}` before edit
- Fail loudly if `old` string not found (search_replace) or context doesn't match (unified_diff)

---

### New Node: `git`

Dedicated node for git operations with validation and confirmation.

#### Config

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| operation | enum | `status` | Git operation to perform |
| requireApproval | bool | true | Show approval dialog before destructive ops |

#### Operations

| Operation | Input | Output | Destructive? |
|-----------|-------|--------|-------------|
| `status` | (none) | Modified/staged/untracked files | No |
| `diff` | optional: file path, ref | Unified diff output | No |
| `log` | optional: count, ref | Commit history | No |
| `commit` | message (from LLM or input) | Commit hash | Yes — requires approval |
| `branch` | branch name | Success/failure | No |
| `checkout` | branch/ref | Success/failure | Yes — requires approval |
| `add` | file paths (array or glob) | Staged file list | No |

#### Handles

| Handle | Direction | Type |
|--------|-----------|------|
| input | target | json/text |
| output | source | text |
| status | source | json |

#### Executor (`git.rs`)

- Uses `std::process::Command` to run git CLI (same as Shell Exec but constrained)
- Working directory: from node config or workflow-level `workingDirectory` setting
- Destructive operations (`commit`, `checkout`) trigger approval dialog via existing approval event system
- Never runs: `push`, `force`, `reset --hard`, `clean` — these are intentionally excluded for safety
- Timeout: 30 seconds per operation

---

### Updated Templates

#### Template: "Code Fix Loop"

```
Input (error message + file path)
  → File Read (read the broken file)
  → LLM (analyze error, generate fix)
  → Code Edit (apply fix, search_replace mode)
  → Shell Exec (run tests)
  → Router (tests pass? parse exit code)
     → pass: Git (commit) → Output (success + diff)
     → fail: Loop back (Shell Exec stderr → LLM context)
```

- Loop config: maxIterations=5, exitCondition=evaluator, feedbackMode=append
- LLM system prompt instructs search_replace JSON output format
- Router checks Shell Exec exit code (0 = pass, non-zero = fail)

#### Template: "Code Review + Auto-Fix"

```
Input (PR description or diff)
  → Knowledge Base (index codebase for context)
  → LLM (review code, find issues)
  → Router (issues found?)
     → no issues: Output (clean review)
     → issues: Iterator (over each issue)
       → LLM (generate fix per issue)
       → Code Edit (apply fix)
     → Aggregator (collect all fixes)
     → Shell Exec (run tests)
     → Output (review + fixes + test results)
```

#### Template: "Codebase Q&A"

```
Input (question about code)
  → Knowledge Base (index + search codebase)
  → File Read (read top-K relevant files)
  → LLM (answer question with full file context)
  → Output (answer with file citations)
```

This already works today with existing nodes — just needs a template.

---

## Implementation

### Build Order

1. **Code Edit executor + node** (~200 lines Rust, ~50 lines TSX)
   - Start with `search_replace` mode only — simplest, most reliable
   - Add `unified_diff` and `line_range` in follow-up
2. **Git executor + node** (~150 lines Rust, ~40 lines TSX)
   - Start with read-only ops (status, diff, log)
   - Add write ops (add, commit) with approval gate
3. **Templates** (3 JSON files + register in templates.rs)
4. **Tests** (~20 unit tests for Code Edit parsing, ~10 for Git executor)

### What We DON'T Need

- **LSP integration** — too complex, marginal benefit for workflows. LLMs work fine without type info.
- **Terminal/REPL node** — Shell Exec covers this. Interactive sessions are a different product.
- **Language-aware parsing** — tree-sitter etc. is overkill. Text-based search_replace works for all languages.

---

## Success Criteria

1. User can build "edit file → run tests → loop until green" workflow from templates
2. Code Edit reliably applies LLM-generated fixes without corrupting files
3. Backup system prevents data loss from bad edits
4. Git node provides safe commit workflow with approval gate
