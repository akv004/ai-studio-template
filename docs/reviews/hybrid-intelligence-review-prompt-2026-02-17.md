# Peer Review: Hybrid Intelligence

**Date**: 2026-02-17
**Project**: AI Studio (open-source IDE for AI agents)
**Reviewer**: Gemini 3 Pro
**Review type**: Architecture + Cross-layer consistency

---

## Instructions for Reviewer

You are reviewing code in the workspace at `/home/amit/projects/myws/ws01/ai-studio-template`.
Read the files listed below, then provide your review in the output format specified.

## Context

AI Studio is a desktop-native IDE for AI agents built with Tauri 2 (Rust) + React 19 + Python FastAPI sidecar. We just implemented "Hybrid Intelligence" — a smart model routing system that picks the best LLM model per request based on task type, budget constraints, and user preferences. The router lives in Rust (not sidecar) and supports 3 modes: single model, auto-routing (built-in rules), and manual routing (user-defined rules). This feature also adds monthly budget tracking with threshold warnings and Inspector integration for routing visibility.

## Scope

Review the full Hybrid Intelligence implementation (commit 8528f05):
- Schema v6 migration (routing_mode, routing_rules on agents table)
- Smart Router module in Rust (routing.rs — 3 modes, MODEL_CAPABILITIES table, 14 unit tests)
- Budget tracking (monthly cost aggregation from events, threshold warnings at 50/80/100%)
- IPC commands (get_budget_status, set_budget)
- UI: agent routing config (AgentsPage), budget settings tab (SettingsPage), Inspector routing events
- TypeScript types for routing, budget, and session stats

## Files to Read

Read these files in this order:
1. `docs/specs/hybrid-intelligence.md` — The spec this implementation follows. Compare implementation against spec.
2. `apps/desktop/src-tauri/src/routing.rs` — NEW: Smart Router module. Core routing logic, MODEL_CAPABILITIES table, 3 routing modes, cost estimation, 14 unit tests.
3. `apps/desktop/src-tauri/src/db.rs` — Schema migration. Look for `migrate_v6` near the bottom of the `impl Database` block.
4. `apps/desktop/src-tauri/src/commands.rs` — IPC commands. Search for: `send_message` (routing integration, budget thresholds), `get_budget_status`, `set_budget`, `SessionStats` struct, `get_session_stats`. This is a large file — focus on the routing-related sections.
5. `packages/shared/types/agent.ts` — TypeScript types for RoutingMode, RoutingRule, BudgetStatus, SessionStats extensions.
6. `apps/ui/src/app/pages/AgentsPage.tsx` — Agent routing mode selector UI. Search for `routingMode`.
7. `apps/ui/src/app/pages/SettingsPage.tsx` — Budget tab UI. Search for `budget`.
8. `apps/ui/src/app/pages/InspectorPage.tsx` — Routing event rendering. Search for `llm.routed`, `budget.warning`, `RoutingDetail`.

## What to Look For

1. **Cross-layer consistency**: Do the Rust structs, TypeScript types, and UI forms all agree on field names, types, and defaults? Are camelCase/snake_case conversions correct for Tauri v2 IPC?
2. **Router correctness**: Does the auto-routing chain (vision → simple → code → large_context → budget → default) have edge cases where multiple conditions fire simultaneously? Is the rule priority system in manual mode robust?
3. **Budget tracking accuracy**: The budget is computed by summing `cost_usd` from the events table for the current month. Are there race conditions or edge cases (timezone, month boundaries, missing cost data)?
4. **Security**: Can a user bypass budget limits? Are the settings table writes safe from injection? Does the `json_extract` in `get_session_stats` handle malformed payloads?
5. **Spec compliance**: Does the implementation match `hybrid-intelligence.md`? Are there spec features that were skipped or implemented differently?
6. **Integration completeness**: Is the routing decision properly passed through the full chain (agent config → router → sidecar request → event emission → Inspector display)?

## Output Format

Provide your review as a markdown file saved to:
**`docs/reviews/hybrid-intelligence-review-2026-02-17.md`**

Use this structure:

### Header
```
# Hybrid Intelligence Review
**Date**: 2026-02-17
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
Any architecture recommendations, praise, or broader observations.
