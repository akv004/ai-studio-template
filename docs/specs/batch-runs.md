# Batch Runs & Dataset Processing

**Status**: PLANNED
**Phase**: 5B (production-ready)
**Priority**: P1 — essential for evaluation, testing, and data processing at scale
**Author**: AI Studio PM
**Date**: 2026-02-21

---

## Problem Statement

Currently each workflow run processes a single input. Real-world use cases require processing many inputs through the same workflow:
- "Run this prompt against 100 test cases and compare outputs"
- "Process all customer tickets from this CSV through the classifier"
- "Evaluate this workflow against a benchmark dataset"

Without batch processing, users manually re-run workflows per item — tedious and impossible to analyze at scale.

---

## Core Concepts

### Dataset

A tabular input source (CSV, JSON Lines, or manual table) where each row becomes one workflow run.

### Batch Run

A collection of individual workflow runs created from a dataset. Each row → one run. Runs execute with configurable parallelism.

### Results Table

Aggregated output from all runs in a batch, displayed as a sortable/filterable table with input columns + output columns + metadata (latency, cost, status).

---

## Dataset Import

### Supported Formats

| Format | Extension | How it maps |
|--------|-----------|-------------|
| CSV | `.csv` | Column headers → variable names. Each row → one run. |
| JSON Lines | `.jsonl` | Each line is a JSON object → one run. Keys → variable names. |
| JSON Array | `.json` | Array of objects → one run per object. |
| Manual | — | User fills rows in a table UI |

### Import Flow

```
┌──────────────────────────────────────────┐
│  Import Dataset                           │
│                                          │
│  [Drop CSV/JSON here or click to browse] │
│                                          │
│  Preview:                                │
│  ┌──────┬────────────┬──────────────┐    │
│  │ #    │ question   │ expected     │    │
│  ├──────┼────────────┼──────────────┤    │
│  │ 1    │ What is... │ Quantum...   │    │
│  │ 2    │ Explain... │ Machine...   │    │
│  │ 3    │ How does...│ Neural...    │    │
│  └──────┴────────────┴──────────────┘    │
│                                          │
│  Rows: 100  Columns: 2                   │
│                                          │
│  Map columns to workflow inputs:         │
│  Input node "prompt" ← [question ▼]     │
│  Input node "context" ← [— none — ▼]   │
│                                          │
│  [Cancel]                 [Start Batch]  │
└──────────────────────────────────────────┘
```

### Column Mapping

- Auto-map: if CSV column names match workflow Input node labels, map automatically
- Manual map: dropdown per Input node to select which column feeds it
- Unmapped columns: stored as metadata, available in results but not passed to workflow

---

## Batch Execution

### Config

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| parallelism | int | 3 | Max concurrent runs |
| stopOnError | enum | continue | continue / stop_after_n / stop_immediately |
| errorThreshold | int | 10 | For stop_after_n: halt after N failures |
| retryFailed | bool | false | Auto-retry failed runs once |
| budgetLimit | float | — | Max total cost ($) for the entire batch |
| timeout | int | 300 | Per-run timeout in seconds |

### Execution Engine

```
Dataset (100 rows)
    │
    ├── Run Queue (FIFO)
    │     │
    │     ├── Worker 1 → Row 1 → Execute Workflow → Result
    │     ├── Worker 2 → Row 2 → Execute Workflow → Result
    │     ├── Worker 3 → Row 3 → Execute Workflow → Result
    │     │
    │     │ (as workers complete, pull next row)
    │     │
    │     ├── Worker 1 → Row 4 → ...
    │     └── ...
    │
    └── Results Collector
          │
          ├── Progress: 47/100 (47%)
          ├── Passed: 44
          ├── Failed: 3
          ├── Running: 3
          └── Remaining: 50
```

**Engine implementation**:
- Rust: `tokio::sync::Semaphore` with `parallelism` permits
- Each run is a standard workflow execution (reuses existing engine)
- Budget enforcement: running cost total checked after each run completion
- Runs share no state — each gets a clean execution context

---

## Batch Dashboard UI

### Progress View (during execution)

```
┌─────────────────────────────────────────────────────┐
│  Batch Run: Customer Classifier                      │
│  Dataset: tickets.csv (500 rows)                     │
│                                                      │
│  ████████████████████░░░░░░░░░░  347/500 (69%)      │
│                                                      │
│  ✓ Completed: 340    ✗ Failed: 7    ◉ Running: 3    │
│  Avg latency: 1.2s   Total cost: $0.42               │
│  ETA: ~3 min                                         │
│                                                      │
│  [Pause]  [Cancel]  [View Results]                   │
└─────────────────────────────────────────────────────┘
```

### Results Table (after completion)

```
┌──────┬────────────┬──────────────┬──────────┬────────┬────────┐
│ #    │ question   │ output       │ status   │ time   │ cost   │
├──────┼────────────┼──────────────┼──────────┼────────┼────────┤
│ 1    │ What is... │ Quantum co...│ ✓        │ 1.1s   │ $0.001 │
│ 2    │ Explain... │ Machine le...│ ✓        │ 0.9s   │ $0.001 │
│ 3    │ How does...│ ERROR: time..│ ✗        │ 30.0s  │ $0.000 │
│ ...  │            │              │          │        │        │
└──────┴────────────┴──────────────┴──────────┴────────┴────────┘

Filters: [All ▼]  [Status: Failed ▼]  Search: [________]
Sort by: [# ▼]

[Export CSV]  [Export JSON]  [Retry Failed (7)]
```

### Results Features

- **Sort** by any column (input, output, status, latency, cost)
- **Filter** by status (all / passed / failed / slow)
- **Search** across input and output text
- **Expand row** to see full output + Inspector link for that run
- **Compare columns**: if dataset has an `expected` column, show diff vs actual output
- **Export**: CSV or JSON with all columns including metadata

---

## Evaluation Mode (A/B Eval Integration)

When a workflow contains an A/B Eval node, batch mode becomes an **evaluation suite**:

- Each dataset row runs through the eval node (multiple models)
- Results table adds columns per model: output, latency, cost, scores
- Summary statistics: win rate per model, average scores, cost comparison
- This is how you run "evaluate my prompt against 100 test cases across 3 models"

---

## Data Model Changes

### New table: `batch_runs`

| Column | Type | Description |
|--------|------|-------------|
| id | TEXT PK | Batch run ID |
| workflow_id | TEXT FK | Workflow being executed |
| dataset_name | TEXT | Original filename or "manual" |
| total_rows | INT | Total items in dataset |
| completed | INT | Completed runs |
| failed | INT | Failed runs |
| config | TEXT (JSON) | Parallelism, stopOnError, etc. |
| status | TEXT | pending / running / paused / completed / cancelled |
| total_cost | REAL | Accumulated cost |
| started_at | TEXT | ISO timestamp |
| completed_at | TEXT | ISO timestamp (nullable) |

### New table: `batch_items`

| Column | Type | Description |
|--------|------|-------------|
| id | TEXT PK | Item ID |
| batch_id | TEXT FK | Parent batch |
| row_index | INT | Position in dataset |
| input_data | TEXT (JSON) | Row data from dataset |
| run_id | TEXT FK | Linked workflow run (nullable until executed) |
| status | TEXT | pending / running / completed / failed / skipped |
| output | TEXT (JSON) | Workflow output (nullable) |
| latency_ms | INT | Execution time |
| cost_usd | REAL | Run cost |
| error | TEXT | Error message if failed |

---

## IPC Commands

```rust
// Dataset
import_dataset(file_path, format) -> Dataset  // Parse and validate
preview_dataset(file_path, format, limit: 5) -> DatasetPreview

// Batch CRUD
create_batch_run(workflow_id, dataset, column_mapping, config) -> BatchRun
get_batch_run(batch_id) -> BatchRun
list_batch_runs(workflow_id?) -> Vec<BatchRunSummary>
delete_batch_run(batch_id) -> ()

// Batch lifecycle
start_batch(batch_id) -> ()
pause_batch(batch_id) -> ()
resume_batch(batch_id) -> ()
cancel_batch(batch_id) -> ()
retry_failed(batch_id) -> ()

// Results
get_batch_results(batch_id, page, page_size, filter?, sort?) -> PagedResults
export_batch_results(batch_id, format: csv|json) -> FilePath
```

---

## Implementation Plan

### Phase 1: Dataset Import + Batch Engine (2 sessions)
- [ ] Rust: CSV/JSON/JSONL parser with preview
- [ ] Rust: `batch_runs` + `batch_items` tables
- [ ] Rust: Batch executor with semaphore-based parallelism
- [ ] Rust: CRUD + lifecycle commands
- [ ] 10 tests (import, execution, parallelism, budget limit, error handling)

### Phase 2: Batch Dashboard UI (1 session)
- [ ] UI: Dataset import modal with preview + column mapping
- [ ] UI: Progress bar + live stats
- [ ] UI: Pause/Resume/Cancel controls
- [ ] UI: Results table with sort/filter/search
- [ ] UI: Export CSV/JSON

### Phase 3: Evaluation Integration (1 session)
- [ ] Wire A/B Eval node outputs into batch results columns
- [ ] Summary statistics panel (win rates, averages)
- [ ] Side-by-side output comparison view
- [ ] 5 tests (eval batch, multi-model comparison, scoring)

---

## Example Workflows

### Prompt Testing
```
Input (from dataset row) → LLM → Output
Dataset: test_cases.csv with columns: prompt, expected_output
Results: Compare LLM output vs expected, measure quality
```

### Batch Classification
```
Input (ticket text) → LLM ("Classify: bug/feature/question") → Output
Dataset: tickets.csv with columns: id, title, description
Results: Classification per ticket, export for import into ticketing system
```

### Multi-Model Evaluation
```
Input (from dataset) → A/B Eval (Claude vs GPT vs Gemini) → Output
Dataset: eval_suite.jsonl with 100 test prompts
Results: Win rate, avg latency, cost per model across all prompts
```
