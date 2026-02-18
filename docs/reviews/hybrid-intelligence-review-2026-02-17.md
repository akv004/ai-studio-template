# Hybrid Intelligence Review
**Date**: 2026-02-17
**Reviewer**: Gemini 3 Pro
**Status**: RESOLVED
**Triaged by**: Claude Opus 4.6

### Findings Table
| Check | Priority | Verdict | Findings |
|-------|----------|---------|----------|
| Cross-layer consistency | HIGH | PASS | `RoutingDecision`, `BudgetStatus` structs in Rust match `agent.ts` interfaces and IPC commands. `camelCase` serialization is correctly handled. |
| Router correctness | HIGH | PASS | `route_auto` implements the 6 rules from spec correctly. `route_manual` sorts by priority. `try_route_to` correctly checks provider availability. |
| Budget tracking accuracy | HIGH | WARN | `get_budget_status` sums events from `month_start`. **Potential Issue**: Timezone handling in `chrono::Utc::now()` vs SQLite strings might cause slight mismatches at month boundaries, but it's acceptable for v1. |
| Security | MED | PASS | `set_budget` and `set_provider_key` use parameterized SQL queries (`?1`), preventing injection. API keys are marked `secret` in UI and not shown in plain text. |
| Spec compliance | HIGH | PASS | Implementation closely matches `hybrid-intelligence.md`. Timeline events, fallback chain, and budget warnings are all present. |
| Integration completeness | HIGH | PASS | `send_message` orchestrates the full flow: DB load -> Router -> Event Emit -> Sidecar -> Event Emit -> Budget Check. |

### Actionable Checklist
- [x] **Nit**: In `AgentsPage.tsx`, the `MODELS_BY_PROVIDER` list is hardcoded. It should ideally be fetched from the backend or `MODEL_CAPABILITIES` to avoid drift. — **Accepted**: Added TODO comment noting the sync requirement. Real fix (get_model_capabilities IPC) deferred.
- [ ] **Suggestion**: `estimate_cost` in Rust uses a hardcoded `0.25` ratio for output tokens. Consider making this configurable per model or refining the heuristic. — **Rejected**: This is a routing heuristic, not billing. Actual cost uses real token counts in `calculate_cost()`. Over-engineering the estimate adds complexity without value.
- [ ] **Test**: Verify that `get_available_providers` correctly handles the case where an API key is present but invalid. The fallback chain should handle the sidecar failure. — **Deferred**: Valid observation. `send_message` routes once, doesn't retry on sidecar failure. Implementing retry-with-next-alternative is a meaningful change — defer until real-world usage shows it's needed.

### Notes
The implementation is very solid and follows the "Visionary" architecture well.
- The `MODEL_CAPABILITIES` table in Rust is a great source of truth.
- The event sourcing pattern (`llm.routed`, `budget.warning`) makes the Inspector extremely powerful for debugging "why did it do that?".
- **Architecture Praise**: Keeping the router in Rust (synchronous, fast access to DB/Settings) instead of the Python sidecar was a good decision for performance and reliability.
