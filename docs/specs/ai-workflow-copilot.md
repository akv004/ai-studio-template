# AI Workflow Copilot — Self-Optimizing Pipelines

**Status**: DRAFT — pending peer review
**Phase**: 5D (requires run history + cost tracking to be mature)
**Priority**: P2 — high impact but needs data from real usage
**Author**: AI Studio PM
**Date**: 2026-02-27
**Effort**: ~8 sessions (phased: advisor → optimizer → auto-pilot)
**Tagline**: "Your workflows get cheaper and better on their own"

---

## Problem Statement

Today, AI Studio workflows are static after creation. They run the same way every time regardless of:
- Whether a cheaper model would produce identical quality
- Whether certain branches are never taken (dead paths)
- Whether retrieval scores are consistently low (bad docs)
- Whether a node consistently fails and retries waste time/money
- Whether the same prompt works better at temperature 0.1 vs 0.7

Users must manually monitor runs, spot patterns, and tweak settings. For a tool that processes hundreds of automated runs (cron, webhooks), this doesn't scale.

**AI Workflow Copilot** watches your workflow runs over time and surfaces actionable insights — from passive suggestions to automatic optimizations.

---

## Three-Phase Design

### Phase A: Advisor (passive — suggestions only)
Analyzes run history, surfaces insights as cards in a "Copilot" panel. User decides whether to apply.

### Phase B: Optimizer (semi-active — one-click apply)
Same insights, but with a one-click "Apply" button that makes the change. Still human-in-the-loop.

### Phase C: Autopilot (active — auto-applies safe optimizations)
User enables autopilot per-workflow. Copilot automatically applies safe optimizations (model downgrade, timeout adjustment) and reports what it changed.

---

## Phase A: Advisor

### Insight Types

| Insight | Detection | Suggestion |
|---------|-----------|------------|
| **Model downgrade** | LLM node output quality is consistent across runs, and a cheaper model exists for this provider | "LLM 'Analyzer' produced similar outputs with gpt-4o-mini (est. 90% cost savings). Switch?" |
| **Dead branch** | Router branch has 0 executions in the last 20 runs | "'critical' branch in Router has never been taken. Remove or review the routing logic." |
| **Low retrieval** | KB node's average top score < 0.5 across last 10 runs | "Knowledge Base 'Standards' retrieval scores are low (avg 0.43). Your docs may not cover the query topics." |
| **Consistent failure** | Node fails > 50% of the time in last 10 runs | "HTTP Request 'Fetch Data' failed 7/10 times. Check the URL or add error handling." |
| **Slow node** | Node takes > 80% of total workflow duration | "LLM 'Summarizer' takes 8.2s avg (82% of total). Consider a faster model or shorter prompt." |
| **Cost outlier** | One node accounts for > 70% of workflow cost | "LLM 'Deep Analysis' costs $0.12/run (89% of total). Budget: $3.60/month at current rate." |
| **Unused output** | Node produces output that no downstream node consumes | "Transform 'Parse' output is not connected to any node. Remove or connect it." |
| **Prompt drift** | Prompt version changed but output quality decreased (via user feedback or automated eval) | "Prompt v7 produces shorter outputs than v5 (avg 120 chars vs 340). Intentional?" |

### UI: Copilot Panel

A collapsible panel on the right side of the workflow canvas (like GitHub Copilot suggestions):

```
┌─ Copilot ─────────────────────────────────────────┐
│                                                    │
│  3 insights from 47 runs                           │
│                                                    │
│  ┌─ 💰 Cost Optimization ───────────────────────┐ │
│  │ LLM "Analyzer" could use gpt-4o-mini         │ │
│  │ Est. savings: $0.09/run (90%)                 │ │
│  │ Confidence: High (consistent output quality)  │ │
│  │ [Apply]  [Dismiss]  [Details]                 │ │
│  └───────────────────────────────────────────────┘ │
│                                                    │
│  ┌─ ⚠ Reliability ──────────────────────────────┐ │
│  │ HTTP "Fetch Data" failing 70% of runs         │ │
│  │ Last error: Connection timeout                │ │
│  │ [Add Error Handler]  [Dismiss]  [Details]     │ │
│  └───────────────────────────────────────────────┘ │
│                                                    │
│  ┌─ 🔍 Retrieval Quality ───────────────────────┐ │
│  │ KB "Standards" avg score: 0.43 (low)          │ │
│  │ Queries not matching: "security", "auth"      │ │
│  │ [Re-index]  [Dismiss]  [Details]              │ │
│  └───────────────────────────────────────────────┘ │
│                                                    │
│  Last analyzed: 2 min ago │ [Refresh]              │
└────────────────────────────────────────────────────┘
```

### Data Collection

Copilot analyzes data that already exists:
- `sessions` table — workflow run history
- `events` table — per-node timing, cost, status
- `node_outputs` — output values (for quality comparison)
- `prompt_versions` table — prompt change history

New: aggregate statistics computed on-demand or cached:

```sql
-- Per-node run statistics
SELECT
    e.node_id,
    COUNT(*) as run_count,
    AVG(CASE WHEN e.event_type = 'workflow.node.completed'
        THEN json_extract(e.data, '$.duration_ms') END) as avg_duration_ms,
    SUM(CASE WHEN e.event_type = 'workflow.node.error' THEN 1 ELSE 0 END) as error_count,
    AVG(json_extract(e.data, '$.cost')) as avg_cost
FROM events e
JOIN sessions s ON e.session_id = s.id
WHERE s.workflow_id = ?1
  AND s.created_at > datetime('now', '-7 days')
GROUP BY e.node_id
```

---

## Architecture

### Insight Engine (Rust)

```rust
pub struct CopilotEngine;

impl CopilotEngine {
    /// Analyze a workflow's recent runs and return insights
    pub fn analyze(db: &Database, workflow_id: &str) -> Vec<Insight> {
        let stats = db.get_node_run_stats(workflow_id, 20); // last 20 runs
        let mut insights = Vec::new();

        for (node_id, stat) in &stats {
            // Dead branch detection
            if stat.node_type == "router" {
                for branch in &stat.branches {
                    if branch.execution_count == 0 && stat.total_runs > 10 {
                        insights.push(Insight::dead_branch(node_id, &branch.name));
                    }
                }
            }

            // Failure rate
            if stat.total_runs > 5 && stat.error_rate > 0.5 {
                insights.push(Insight::high_failure(node_id, stat.error_rate, &stat.last_error));
            }

            // Cost outlier
            if stat.cost_share > 0.7 {
                insights.push(Insight::cost_outlier(node_id, stat.avg_cost, stat.cost_share));
            }

            // Slow node
            if stat.duration_share > 0.8 {
                insights.push(Insight::slow_node(node_id, stat.avg_duration_ms, stat.duration_share));
            }
        }

        insights
    }
}
```

### Model Downgrade Detection

This is the most valuable insight. Approach:

1. Collect last N outputs from an LLM node
2. For each output, compute a quality fingerprint (length, structure, key phrases)
3. If fingerprint variance is low (consistent outputs), suggest a cheaper model
4. Confidence levels: Low (5-10 runs), Medium (10-20), High (20+)

For v1, use simple heuristics (output length variance, structure similarity). For v2, use embeddings to compare output quality.

### New IPC Commands

```rust
#[tauri::command]
fn get_copilot_insights(db: State<Db>, workflow_id: String) -> Vec<Insight>

#[tauri::command]
fn dismiss_insight(db: State<Db>, workflow_id: String, insight_id: String) -> ()

#[tauri::command]
fn apply_insight(db: State<Db>, workflow_id: String, insight_id: String) -> ()
```

---

## Implementation Plan

### Phase A — Advisor (3 sessions)

**Session 1: Data Collection + Stats**
- [ ] `get_node_run_stats()` DB query — aggregate per-node stats from events
- [ ] `Insight` struct with type, severity, message, action
- [ ] Detection: failure rate, cost outlier, slow node, unused output
- [ ] `get_copilot_insights` IPC command
- [ ] 10 unit tests

**Session 2: Advanced Insights**
- [ ] Dead branch detection (Router branches with 0 executions)
- [ ] Low retrieval score detection (KB avg score)
- [ ] Model downgrade suggestion (output length/structure variance)
- [ ] `dismiss_insight` / `apply_insight` IPC commands
- [ ] Dismissed insights stored in settings (don't re-show)

**Session 3: Copilot UI**
- [ ] `CopilotPanel.tsx` — collapsible right panel
- [ ] Insight cards with severity icons, descriptions, actions
- [ ] Apply button (changes node config via store)
- [ ] Dismiss button (hides insight)
- [ ] Details expansion (run data, trend chart)
- [ ] Toolbar toggle: Copilot icon with insight count badge

### Phase B — Optimizer (2 sessions)
- [ ] One-click Apply for model downgrade (swap model in node config + save)
- [ ] One-click "Add Error Handler" (insert Router after failing node)
- [ ] "Re-index" action for KB nodes
- [ ] Before/after cost projection
- [ ] Undo applied optimizations

### Phase C — Autopilot (3 sessions)
- [ ] Per-workflow autopilot toggle in config
- [ ] Safe optimization rules (model downgrade only, timeout only)
- [ ] Autopilot log (what was changed, when, why)
- [ ] Rollback to pre-autopilot state
- [ ] Notifications for applied changes

---

## Scope Boundaries

### In scope (Phase A — v1)
- Passive insights from run history
- 8 insight types (see table above)
- Copilot panel UI with cards
- Dismiss/apply actions
- Per-workflow analysis

### Out of scope (Phase B/C — v2+)
- Automatic optimization without user confirmation
- Cross-workflow insights ("Workflow A and B use the same prompt — deduplicate?")
- Predictive insights ("Based on usage trends, you'll hit your budget in 5 days")
- Prompt quality scoring (needs LLM-as-judge or user feedback)
- Integration with external monitoring (Datadog, Grafana)

---

## Success Criteria

1. After 20 runs, Copilot panel shows at least 1 actionable insight
2. Model downgrade suggestion includes confidence level and estimated savings
3. User clicks "Apply" on model downgrade → node config updates, next run uses cheaper model
4. Dead branch detection correctly identifies Router branches with 0 hits
5. Copilot doesn't spam — max 5 insights shown, prioritized by impact
