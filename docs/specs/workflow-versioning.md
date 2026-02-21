# Workflow Versioning & History

**Status**: PLANNED
**Phase**: 5B (production-ready)
**Priority**: P2 — important for iterative development and team use
**Author**: AI Studio PM
**Date**: 2026-02-21

---

## Problem Statement

When users edit a workflow, the previous version is lost. There's no way to:
- See what changed between iterations
- Roll back to a version that worked before
- Compare outputs between two versions of the same workflow
- Track who changed what and why (for team scenarios)

This makes iterative prompt engineering and workflow development risky — one bad edit can break a workflow with no undo path beyond Ctrl+Z.

---

## Core Concepts

### Version

A snapshot of a workflow's complete state (nodes, edges, config) at a point in time. Immutable once created.

### Version Creation Strategy

Versions are created **automatically** at meaningful checkpoints:
1. **Before each run** — auto-save current state as a version (ensures you can always go back to "the version I just ran")
2. **Manual save** — user explicitly saves a named version ("v2 - added guardrails")
3. **On import** — when loading a template, the pre-import state is versioned

This avoids excessive versioning (not every keystroke) while ensuring important states are always captured.

### Diff

A structured comparison showing what changed between two versions:
- Added nodes (green)
- Removed nodes (red)
- Modified nodes (yellow) — changed config, prompt, model, etc.
- Changed edges (connections added/removed)

---

## Data Model

### New table: `workflow_versions`

| Column | Type | Description |
|--------|------|-------------|
| id | TEXT PK | Version ID |
| workflow_id | TEXT FK | Parent workflow |
| version_number | INT | Auto-increment per workflow (1, 2, 3...) |
| label | TEXT | Optional user-provided name ("v2 - added guardrails") |
| trigger | TEXT | auto_pre_run / manual / import / rollback |
| graph_data | TEXT (JSON) | Full nodes + edges snapshot |
| node_count | INT | Number of nodes (for quick display) |
| edge_count | INT | Number of edges |
| created_at | TEXT | ISO timestamp |

### Storage Efficiency

- Each version stores the **full graph JSON** (not a diff)
- Typical workflow graph: 2-20KB — even 1000 versions = 20MB (trivial for SQLite)
- No compression needed at this scale
- Old versions auto-pruned after configurable limit (default: 100 per workflow)

---

## Version History Panel

### UI Location

New tab in the workflow toolbar area, alongside existing tabs:

```
[Canvas]  [Config]  [History]  [Runs]
```

### History View

```
┌──────────────────────────────────────────────┐
│  Version History                              │
│                                              │
│  ● v12 — "Added retry logic"      just now   │
│  │  Manual save · 8 nodes, 9 edges           │
│  │  [View] [Restore] [Compare ▼]            │
│  │                                           │
│  ○ v11 — Auto-save (pre-run)      2 min ago  │
│  │  7 nodes, 8 edges                         │
│  │  [View] [Restore] [Compare ▼]            │
│  │                                           │
│  ○ v10 — Auto-save (pre-run)      1 hr ago   │
│  │  7 nodes, 7 edges                         │
│  │  [View] [Restore] [Compare ▼]            │
│  │                                           │
│  ○ v9 — "Initial working version"  yesterday │
│  │  6 nodes, 6 edges                         │
│  │                                           │
│  ···  (load more)                            │
└──────────────────────────────────────────────┘
```

### Actions per Version

| Action | Description |
|--------|-------------|
| **View** | Load this version onto canvas in read-only mode (nodes grayed, "Viewing v9" banner) |
| **Restore** | Replace current workflow with this version (creates a new version first as backup) |
| **Compare** | Open diff view between this version and another (default: current) |
| **Label** | Add/edit a descriptive name |
| **Delete** | Remove this version (with confirmation) |

---

## Diff View

### Split-pane Comparison

When comparing two versions, show a split view:

```
┌───────────────────────┬───────────────────────┐
│  v9 (baseline)        │  v12 (current)        │
│                       │                       │
│  ┌───┐    ┌───┐      │  ┌───┐    ┌───┐      │
│  │INP│───→│LLM│──→   │  │INP│───→│LLM│──→   │
│  └───┘    └───┘  │   │  └───┘    └───┘  │   │
│              ↓   │   │      ↓     ↓     │   │
│           ┌───┐  │   │  ┌─────┐ ┌───┐  │   │
│           │OUT│←─┘   │  │GUARD│ │RET│  │   │
│           └───┘      │  └──┬──┘ └─┬─┘  │   │
│                       │     ↓      ↓    │   │
│                       │  ┌───┐  ┌───┐   │   │
│                       │  │OUT│  │OUT│←──┘   │
│                       │  └───┘  └───┘       │
├───────────────────────┴───────────────────────┤
│  Changes: +2 nodes, +2 edges, ~1 modified     │
│                                               │
│  + Guardrail node (new)                        │
│  + Retry node (new)                            │
│  ~ LLM node: model changed (gpt-4 → claude)   │
│  ~ LLM node: prompt modified                   │
│  + Edge: LLM → Guardrail                       │
│  + Edge: Guardrail → Output                    │
│  - Edge: LLM → Output (removed)               │
└───────────────────────────────────────────────┘
```

### Diff Detail

For each changed node, show the specific config changes:

```
~ LLM · Summarizer (node_llm_1)
  model:  "gpt-4o"  →  "claude-sonnet-4-5"
  prompt: "Summarize the following text:"
       →  "Summarize the following text concisely in 3 bullet points:"
  temperature: 0.7  →  0.5
```

### Diff Algorithm

1. Match nodes between versions by `node_id`
2. Nodes present in B but not A → added
3. Nodes present in A but not B → removed
4. Nodes present in both → deep-compare `data` object, list changed fields
5. Same logic for edges (match by `source+target+sourceHandle+targetHandle`)

---

## Run Comparison

### Compare Outputs Across Versions

Select two runs (from different versions) and compare:

```
┌──────────────────────┬──────────────────────┐
│  Run #45 (v9)        │  Run #52 (v12)       │
├──────────────────────┼──────────────────────┤
│  Input: "Explain..." │  Input: "Explain..." │
│                      │                      │
│  LLM output:         │  LLM output:         │
│  "Quantum computing  │  "• Quantum bits     │
│  is a paradigm..."   │  • Superposition     │
│  (342 tokens)        │  • Entanglement"     │
│                      │  (89 tokens)          │
│                      │                      │
│  Cost: $0.004        │  Cost: $0.001        │
│  Time: 1.2s          │  Time: 0.8s          │
├──────────────────────┴──────────────────────┤
│  Δ Tokens: -253 (74% reduction)             │
│  Δ Cost:   -$0.003 (75% savings)            │
│  Δ Time:   -0.4s (33% faster)              │
└─────────────────────────────────────────────┘
```

This is especially powerful combined with Batch Runs — compare aggregate metrics (avg latency, cost, quality) between workflow versions.

---

## Manual Version Save

### Save Version Dialog

Toolbar button: 💾 (or Ctrl+S when in workflow editor)

```
┌──────────────────────────────┐
│  Save Version                │
│                              │
│  Label: [v2 - added retry  ]│
│                              │
│  Note: Auto-saved versions   │
│  are created before each run │
│                              │
│  [Cancel]      [Save]       │
└──────────────────────────────┘
```

---

## IPC Commands

```rust
// Version CRUD
create_version(workflow_id, label?, trigger) -> WorkflowVersion
list_versions(workflow_id, page, page_size) -> PagedVersions
get_version(version_id) -> WorkflowVersion
delete_version(version_id) -> ()
update_version_label(version_id, label) -> ()

// Restore
restore_version(workflow_id, version_id) -> WorkflowVersion  // creates backup, restores

// Diff
diff_versions(version_a_id, version_b_id) -> VersionDiff

// Run comparison
compare_runs(run_a_id, run_b_id) -> RunComparison

// Auto-save hook (called by engine before run)
auto_save_version(workflow_id) -> WorkflowVersion
```

---

## Implementation Plan

### Phase 1: Version Storage + Auto-save (1 session)
- [ ] Rust: `workflow_versions` table + migration
- [ ] Rust: `create_version`, `list_versions`, `get_version`
- [ ] Rust: Auto-save hook before `run_workflow`
- [ ] Rust: Auto-prune (keep last 100)
- [ ] 8 tests (create, list, auto-save, prune, restore)

### Phase 2: History Panel UI (1 session)
- [ ] UI: History tab in workflow toolbar
- [ ] UI: Version list with timeline view
- [ ] UI: View version (read-only canvas mode)
- [ ] UI: Restore with confirmation
- [ ] UI: Manual save dialog (Ctrl+S)

### Phase 3: Diff View (1 session)
- [ ] Rust: `diff_versions` command (node/edge comparison)
- [ ] UI: Split-pane diff view
- [ ] UI: Change list (added/removed/modified nodes)
- [ ] UI: Per-node config diff (field-by-field)
- [ ] Color-coded canvas overlay (green=added, red=removed, yellow=modified)

### Phase 4: Run Comparison (1 session)
- [ ] Rust: `compare_runs` command (output + metrics diff)
- [ ] UI: Side-by-side run output view
- [ ] UI: Delta metrics (tokens, cost, time)
- [ ] Integration with Batch Runs (compare version A batch vs version B batch)

---

## Edge Cases

- **Restore creates a backup**: Before restoring v5, auto-save current state as a new version (labeled "Auto-save before restore to v5")
- **Concurrent editing**: Not applicable (single-user desktop app). Future team collaboration would need conflict resolution.
- **Large workflows**: Version diff is computed on-demand, not stored. Even 50-node workflows diff in <10ms.
- **Version pruning**: Keep all manually-labeled versions forever. Only prune auto-saved versions beyond the limit.
