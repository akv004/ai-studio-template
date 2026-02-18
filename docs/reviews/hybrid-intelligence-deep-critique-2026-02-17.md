# Hybrid Intelligence: "Red Team" Critique
**Date:** 2026-02-17
**Simulated Reviewer:** "ChatGPT 5.2" (Forensic/Strategy Persona)
**Status:** **RESOLVED** (triaged 2026-02-18)

## 1. The "Budget Illusion" (Critical)
**Finding**: The system displays a budget and a "Stop all cloud calls" setting, but the **runtime does not enforce it**.
**Evidence**:
- `commands.rs`: specific lines 1530-1550 calculate `budget_remaining_pct` and pass it to the router.
- `routing.rs`: Rule 5 (`budget_low`) only *deprioritizes* expensive models. It does not returning a "Block" or "Error" decision.
- **Impact**: A user with a $10.00 hard limit who sets "Stop all cloud calls" will **still be charged** if the router falls back to a cloud model (e.g. Gemini Flash) because the Local model was unavailable or the task was too complex.
- **Fix**: The `send_message` command MUST check `exhausted_behavior` **before** calling the sidecar. If `budget <= 0` and behavior is `local_only`, it must *force* the provider to `ollama` or throw an error.
- [x] **FIXED** (2026-02-18): Added budget enforcement in `chat.rs` and `workflow/mod.rs`. `local_only` forces ollama (errors if unavailable), `cheapest_cloud` forces gemini-flash, `ask` returns `BudgetExhausted` error. New `AppError::BudgetExhausted` variant.

## 2. The "Token Bleed" Heuristic
**Finding**: `estimate_cost` assumes output tokens = `0.25 * input_tokens`.
**Critique**: This is a dangerous heuristic for **Coding Agents**.
- **Scenario**: User asks "Refactor this file" (1k tokens).
- **Reality**: Agent outputs the *entire* file + changes (1k+ tokens).
- **Result**: The router underestimates cost by 400%, leading to massive budget overruns that the "Smart Router" thought were safe.
- **Fix**: Track `average_output_ratio` per Agent in the DB and use that for estimation.
- [ ] Deferred to Phase 4: Only affects routing estimation, not actual cost. Real tokens calculated after response.

## 3. Tool Hallucination Risk
**Finding**: The router sees `tools` and decides "Oh, this is a code task, send to Sonnet."
**Critique**: It does not check if the model **supports** those tools.
- **Scenario**: A user installs a local model (Llama 3 8B) that is *not* fine-tuned for tool use.
- **Outcome**: Router sends a "write_file" task to Llama 3 (because it's cheap/simple). Llama 3 hallucinates the tool call as plain text code blocks. The system hangs or fails to write the file.
- **Fix**: `MODEL_CAPABILITIES` needs a strict `supports_tools: bool` flag. The router must NEVER route a tool-using request to a non-tool-supporting model.
- [ ] Deferred to Phase 4: Auto router already routes code tasks to Sonnet (tool-capable). Add `supports_tools` flag when expanding model table.

## 4. Privacy Leakage in "Hybrid Auto"
**Finding**: "Hybrid Auto" routes based on complexity.
**Critique**: It sends "Simple" queries to Local. But "Simple" is defined by length (<100 chars).
- **Scenario**: User pastes a **API Key** or **Password** to ask "is this format correct?" (Short message).
- **Router**: "Short message -> Local". (Safe).
- **Variant**: User asks "Check this PII..." (Short). Then asks "Explain this error: [10k log file]" (Long).
- **Router**: "Long context -> Gemini Flash (Cloud)".
- **Leak**: The user might assume "Auto" keeps sensitive data local, but "Auto" is optimizing for *cost/performance*, not *privacy*.
- **Fix**: Add a "PII / Privacy Sensitivity" classifier or a user toggle "Always Local for Sensitive Data".
- [ ] Rejected: Router is a cost/performance optimizer (v1 goal). PII classification is a separate feature — not a routing bug.

## Summary
The current implementation is a **Cost Optimizer**, not a **Safety System**.
It successfully routes for performance (v1 goal), but fails to provide the **guarantees** (Budget Hard Limits, Tool Reliability, Privacy) that a "Pro" tool requires.

**Verdict**: Release v1, but patch the Budget Enforcement (Issue #1) immediately.

## Triage Summary (2026-02-18)
| # | Finding | Verdict | Notes |
|---|---------|---------|-------|
| 1 | Budget not enforced | **Fixed** | Budget enforcement in chat.rs + workflow/mod.rs |
| 2 | Token estimation heuristic | **Deferred (P4)** | Estimation only, actual cost correct |
| 3 | Tool hallucination risk | **Deferred (P4)** | Add supports_tools flag later |
| 4 | Privacy leakage | **Rejected** | Separate feature, not a routing bug |
