# Prompt Version Control — Git for LLM Prompts

**Status**: DRAFT — pending peer review
**Phase**: 5B
**Priority**: P1 — solves universal prompt engineering pain
**Author**: AI Studio PM
**Date**: 2026-02-27
**Effort**: ~2 sessions

---

## Problem Statement

Prompt engineering is iterative. Every developer tweaks LLM prompts dozens of times per workflow. The problem:

1. **Lost history** — You edit a prompt, the old version is gone. No undo beyond Ctrl+Z.
2. **No comparison** — "Was version 2 or version 5 better?" You can't compare without manually saving copies.
3. **No rollback** — You broke a working prompt by "improving" it. Now what?
4. **No correlation** — "Which prompt version produced that great output I saw yesterday?"

Nobody solves this well. LangSmith has prompt registries, but they're external services. Dify has no versioning at all. AI Studio can build this natively into the node editor — zero config, always on.

---

## User Experience

### Automatic Versioning

Every time a user modifies an LLM node's prompt (system prompt or user template) and runs the workflow, a new version is automatically saved. No explicit "save version" step.

```
┌─ LLM Node Config ─────────────────────────────────────────┐
│                                                            │
│  System Prompt                              v7 │ ⏱ History│
│  ┌──────────────────────────────────────────────────────┐ │
│  │ You are a senior code reviewer. Analyze this code    │ │
│  │ change against our team's coding standards.          │ │
│  │                                                      │ │
│  │ Rate severity: CRITICAL or NORMAL.                   │ │
│  │ - CRITICAL: security vulns, breaking changes         │ │
│  │ - NORMAL: style issues, minor refactors              │ │
│  └──────────────────────────────────────────────────────┘ │
│                                                            │
└────────────────────────────────────────────────────────────┘
```

The **v7** badge shows the current version. Click **History** to open the version panel.

### Version History Panel

```
┌─ Prompt History: LLM · Analyzer ──────────────────────────┐
│                                                            │
│  v7 (current)  Today 2:30 PM                              │
│  "You are a senior code reviewer. Analyze this code..."   │
│  Last run: ✓ Success · "CRITICAL: SQL injection..."       │
│                                                            │
│  v6  Today 1:15 PM                                        │
│  "You are a code reviewer. Check this diff for..."        │
│  Last run: ✓ Success · "NORMAL: Minor style issues..."    │
│  Changed: -"senior code reviewer" +"code reviewer"        │
│                                                            │
│  v5  Today 12:00 PM                                       │
│  "Review the following code change. Be thorough..."       │
│  Last run: ✗ Failed · Router parse error                  │
│  Changed: Complete rewrite                                │
│                                                            │
│  v4  Yesterday 4:00 PM                                    │
│  "Analyze this PR for bugs and style issues..."           │
│  Last run: ✓ Success · "Found 2 issues..."                │
│                                                            │
│  [Load v4]  [Compare v4 ↔ v7]  [Delete v5]               │
└────────────────────────────────────────────────────────────┘
```

Each version shows:
- Version number + timestamp
- First 80 chars of the prompt
- Last run result (from that version) — success/fail + output preview
- Diff summary vs previous version

### Actions

| Action | What happens |
|--------|-------------|
| **Load** | Replace current prompt with that version. Creates a new version (v8 = copy of v4). |
| **Compare** | Side-by-side diff view (green = added, red = removed) |
| **Delete** | Remove a version from history |
| **Pin** | Mark a version as "known good" — pinned versions can't be accidentally deleted |
| **Copy** | Copy prompt text to clipboard |

### Diff View

```
┌─ Compare v6 ↔ v7 ─────────────────────────────────────────┐
│                                                            │
│  - You are a code reviewer. Check this diff for            │
│  + You are a senior code reviewer. Analyze this code       │
│  + change against our team's coding standards.             │
│                                                            │
│    Rate severity: CRITICAL or NORMAL.                      │
│  - - CRITICAL: security issues                             │
│  + - CRITICAL: security vulns, breaking changes            │
│  - - NORMAL: everything else                               │
│  + - NORMAL: style issues, minor refactors                 │
│                                                            │
│  Result with v6: "NORMAL: Minor style issues..."           │
│  Result with v7: "CRITICAL: SQL injection vulnerability.." │
│                                                            │
│  [Use v6]  [Use v7]  [Close]                               │
└────────────────────────────────────────────────────────────┘
```

The diff view also shows the output from each version — so you can see how the prompt change affected the result.

---

## Architecture

### Storage

Prompt versions are stored in a new SQLite table:

```sql
CREATE TABLE IF NOT EXISTS prompt_versions (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    workflow_id TEXT NOT NULL REFERENCES workflows(id) ON DELETE CASCADE,
    node_id     TEXT NOT NULL,
    field       TEXT NOT NULL DEFAULT 'system_prompt',  -- 'system_prompt' | 'user_template'
    version     INTEGER NOT NULL,
    content     TEXT NOT NULL,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    pinned      INTEGER NOT NULL DEFAULT 0,
    run_output  TEXT,          -- output preview from first run with this version
    run_status  TEXT,          -- 'success' | 'failed' | null
    UNIQUE(workflow_id, node_id, field, version)
);
```

### Versioning Logic

```rust
// Before executing an LLM node, check if prompt changed
fn maybe_save_prompt_version(db, workflow_id, node_id, current_prompt) {
    let latest = db.get_latest_prompt_version(workflow_id, node_id, "system_prompt");

    if latest.is_none() || latest.content != current_prompt {
        let next_version = latest.map(|v| v.version + 1).unwrap_or(1);
        db.insert_prompt_version(workflow_id, node_id, "system_prompt", next_version, current_prompt);
    }
}

// After LLM node completes, record output against current version
fn record_prompt_result(db, workflow_id, node_id, output, status) {
    db.update_latest_prompt_version_result(workflow_id, node_id, output_preview, status);
}
```

### New IPC Commands

```rust
#[tauri::command]
fn list_prompt_versions(db, workflow_id, node_id, field) -> Vec<PromptVersion>

#[tauri::command]
fn load_prompt_version(db, workflow_id, node_id, field, version) -> String

#[tauri::command]
fn delete_prompt_version(db, workflow_id, node_id, field, version) -> ()

#[tauri::command]
fn pin_prompt_version(db, workflow_id, node_id, field, version, pinned: bool) -> ()
```

### Schema Migration

Add to `migrate()` in `db.rs` as schema v9:

```rust
// v9: Prompt version control
conn.execute_batch("
    CREATE TABLE IF NOT EXISTS prompt_versions (
        id          INTEGER PRIMARY KEY AUTOINCREMENT,
        workflow_id TEXT NOT NULL REFERENCES workflows(id) ON DELETE CASCADE,
        node_id     TEXT NOT NULL,
        field       TEXT NOT NULL DEFAULT 'system_prompt',
        version     INTEGER NOT NULL,
        content     TEXT NOT NULL,
        created_at  TEXT NOT NULL DEFAULT (datetime('now')),
        pinned      INTEGER NOT NULL DEFAULT 0,
        run_output  TEXT,
        run_status  TEXT,
        UNIQUE(workflow_id, node_id, field, version)
    );
")?;
```

---

## Implementation Plan

### Session 1: Backend + Storage
- [ ] Schema v9: `prompt_versions` table
- [ ] `maybe_save_prompt_version()` — called before LLM execution
- [ ] `record_prompt_result()` — called after LLM completion
- [ ] 4 IPC commands: list, load, delete, pin
- [ ] 8 unit tests (versioning, dedup, load, pin, cascade delete)

### Session 2: UI
- [ ] Version badge on LLM node config panel (`v7`)
- [ ] History button → `PromptHistoryPanel.tsx`
- [ ] Version list with timestamp, preview, run result
- [ ] Load version action (creates new version)
- [ ] Diff view component (simple line-by-line diff)
- [ ] Pin/unpin toggle
- [ ] Delete version (with confirmation, pinned versions protected)

---

## Scope Boundaries

### In scope (v1)
- Auto-versioning on prompt change + run
- Version list with timestamps and run results
- Load (rollback), compare (diff), delete, pin
- System prompt and user template fields
- Per-node, per-workflow isolation

### Out of scope (v2+)
- Named versions ("production", "experimental")
- Version branches (fork from v3 to try two directions)
- Prompt A/B testing (run same input with two prompt versions, compare)
- Cross-workflow prompt sharing (template library for prompts)
- Prompt analytics (which version has best success rate over N runs)
- Export/import prompt history

---

## Success Criteria

1. User edits an LLM prompt and runs → version auto-saved, no explicit action needed
2. User opens History → sees all previous versions with timestamps and run outputs
3. User compares v3 vs v7 → sees inline diff + output from each version
4. User loads v3 → prompt reverts, new version v8 created (non-destructive)
5. User pins v7 as "known good" → cannot be accidentally deleted
