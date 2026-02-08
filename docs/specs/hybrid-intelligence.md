# AI Studio — Hybrid Intelligence Specification

> **Version**: 1.0
> **Status**: Draft
> **Depends on**: product-vision.md, architecture.md

---

## What Is Hybrid Intelligence?

Most AI tools force you to pick ONE model. Claude OR GPT OR local. Then you use that model for everything — even when it's overkill or underpowered for the task.

Hybrid Intelligence means: **the right model for each step, automatically.**

```
Simple question     → Local Llama (free, fast, private)
Complex code        → Claude Sonnet (best at code, moderate cost)
Vision/screenshots  → GPT-4o (strong vision, moderate cost)
Quick classification → Gemini Flash (cheapest cloud, fastest)
Critical reasoning  → Claude Opus (most capable, expensive)
```

The user doesn't think about models. They think about tasks. AI Studio handles the routing.

---

## Why This Is a Killer Feature

**Nobody else does this well:**

| Tool | Model Routing |
|---|---|
| ChatGPT | GPT only |
| Claude | Claude only |
| Cursor | One model at a time, manual switch |
| OpenClaw | One model per agent, manual config |
| LM Studio | Local only |
| **AI Studio** | **Auto-routes across local + cloud + multiple providers** |

**Real money savings:**

| Task | Without Hybrid | With Hybrid | Savings |
|---|---|---|---|
| Read a file, answer simple question | Claude Opus ($0.02) | Local Llama ($0.00) | 100% |
| Generate 50 unit tests | Opus for all ($1.00) | Flash for simple, Sonnet for complex ($0.15) | 85% |
| Full-day coding session | Sonnet for everything ($2.40) | Local for reads, Sonnet for writes ($0.80) | 67% |

---

## How It Works

### Three Intelligence Modes

Users pick one when creating an agent:

#### Mode 1: Single Model (Simple)
```
Always use: Claude Sonnet
```
What exists today. Pick one model, use it for everything. Good for beginners or when you want predictability.

#### Mode 2: Hybrid Auto (Smart — Default)
```
AI Studio picks the best model for each step based on:
  → Task complexity (simple lookup vs complex reasoning)
  → Required capabilities (vision, code, math)
  → Cost target (stay within budget)
  → Speed preference (fast vs thorough)
```

#### Mode 3: Hybrid Manual (Custom Rules)
```
You define the rules:
  Code generation     → Claude Sonnet
  Code review         → Claude Opus
  Simple questions    → Ollama Llama 3
  Anything with image → GPT-4o
  Fallback            → Gemini Flash
```

---

## Smart Router: How Auto Mode Decides

The router runs BEFORE each LLM call and picks the best model.

### Decision Factors

```
┌─────────────────────────────────────────────┐
│              Smart Router                    │
│                                             │
│  Input:                                     │
│    → Message content                        │
│    → Conversation history length            │
│    → Attached tools (what's available)      │
│    → Has images/files? (vision needed?)     │
│    → User's budget remaining                │
│    → Available models (what's online)       │
│                                             │
│  Decision:                                  │
│    → Model to use                           │
│    → Reason (logged in event)               │
│                                             │
│  Output:                                    │
│    → Route to chosen provider               │
│    → Emit event: llm.routed { model, reason }│
└─────────────────────────────────────────────┘
```

### Routing Rules (Built-in Defaults)

These are the default rules for Hybrid Auto mode. Users can override in settings.

```python
ROUTING_RULES = [
    # Rule 1: Vision tasks need a vision model
    {
        "condition": "message_has_image",
        "route_to": "gpt-4o",          # Best vision model
        "reason": "vision_required"
    },

    # Rule 2: Short, simple messages → local model
    {
        "condition": "message_tokens < 100 AND no_tools_needed",
        "route_to": "ollama:llama3.2",
        "reason": "simple_query_local"
    },

    # Rule 3: Code generation/editing → best code model
    {
        "condition": "tools_include_filesystem_write OR tools_include_shell",
        "route_to": "claude-sonnet-4-5",
        "reason": "code_task"
    },

    # Rule 4: Long context (big codebase) → large context model
    {
        "condition": "context_tokens > 50000",
        "route_to": "gemini-2.0-flash",  # 1M context, cheap
        "reason": "large_context"
    },

    # Rule 5: Budget is low → cheapest available
    {
        "condition": "monthly_budget_remaining < 20%",
        "route_to": "ollama:llama3.2",   # Free
        "reason": "budget_conservation"
    },

    # Rule 6: Fallback → balanced default
    {
        "condition": "always",
        "route_to": "claude-sonnet-4-5",
        "reason": "default"
    }
]
```

### What Gets Logged (Inspector Visibility)

Every routing decision is an event:

```json
{
  "type": "llm.routed",
  "payload": {
    "chosen_model": "ollama:llama3.2",
    "reason": "simple_query_local",
    "alternatives_considered": [
      { "model": "claude-sonnet-4-5", "estimated_cost": 0.003 },
      { "model": "gpt-4o", "estimated_cost": 0.004 }
    ],
    "estimated_savings": 0.003
  }
}
```

The Inspector shows:
```
● llm.routed → Llama 3 (local)
  Reason: Simple query — routed locally
  Saved: $0.003 vs Claude Sonnet
```

Users can see WHY each model was chosen. Full transparency.

---

## Budget Controls

### Monthly Budget

```
Settings → Intelligence:

  Monthly AI budget:  [$10.00            ]

  When budget runs out:
    ○ Stop all cloud calls (local only)
    ● Fall back to cheapest cloud model
    ○ Ask me each time
    ○ No limit

  Current month:
  ████████░░░░░░░░░░░░  $4.12 / $10.00 (41%)

  Breakdown:
    Claude:  $3.20  (78%)
    GPT-4o:  $0.72  (17%)
    Gemini:  $0.20  (5%)
    Local:   $0.00
```

### Per-Agent Budget

```
Agent "Code Assistant":
  Budget: $2.00/month
  Used: $0.84

Agent "Data Analyst":
  Budget: $1.00/month
  Used: $0.31
```

### Budget Events

When budget thresholds are hit:

```json
{
  "type": "budget.warning",
  "payload": {
    "level": "80_percent",
    "budget": 10.00,
    "used": 8.12,
    "remaining": 1.88,
    "suggestion": "Switching to local models for simple tasks"
  }
}
```

---

## Capability Matrix

The router needs to know what each model can do:

```python
MODEL_CAPABILITIES = {
    "claude-opus-4-6": {
        "strengths": ["reasoning", "code", "analysis", "long_output"],
        "context_window": 200000,
        "vision": False,
        "speed": "slow",
        "cost_tier": "expensive",
        "input_per_1m": 15.00,
        "output_per_1m": 75.00,
    },
    "claude-sonnet-4-5": {
        "strengths": ["code", "balanced", "tool_use"],
        "context_window": 200000,
        "vision": True,
        "speed": "medium",
        "cost_tier": "moderate",
        "input_per_1m": 3.00,
        "output_per_1m": 15.00,
    },
    "gpt-4o": {
        "strengths": ["vision", "balanced", "multilingual"],
        "context_window": 128000,
        "vision": True,
        "speed": "medium",
        "cost_tier": "moderate",
        "input_per_1m": 2.50,
        "output_per_1m": 10.00,
    },
    "gemini-2.0-flash": {
        "strengths": ["speed", "large_context", "cheap"],
        "context_window": 1000000,
        "vision": True,
        "speed": "fast",
        "cost_tier": "cheap",
        "input_per_1m": 0.10,
        "output_per_1m": 0.40,
    },
    "ollama:llama3.2": {
        "strengths": ["free", "private", "fast_local"],
        "context_window": 128000,
        "vision": False,
        "speed": "varies",  # depends on hardware
        "cost_tier": "free",
        "input_per_1m": 0.00,
        "output_per_1m": 0.00,
    },
}
```

Users can add custom models and tag their capabilities in Settings.

---

## Hybrid Intelligence in the Inspector

This is where hybrid becomes visible and valuable:

### Session Stats Show Model Mix

```
┌──────────────────────────────────────────────────────────────┐
│  Session Stats                                               │
│                                                              │
│  Models used:                                                │
│    Claude Sonnet  ████████████░░░░  6 calls  $0.042          │
│    Llama 3 local  ████░░░░░░░░░░░░  3 calls  $0.000          │
│    Gemini Flash   ██░░░░░░░░░░░░░░  1 call   $0.001          │
│                                                              │
│  Total: $0.043                                               │
│  If all Claude Sonnet: $0.070 — Hybrid saved 39%             │
│  If all Claude Opus:   $0.350 — Hybrid saved 88%             │
└──────────────────────────────────────────────────────────────┘
```

### Timeline Shows Model Per Step

```
  ● llm.routed    → Llama 3 (local) — simple file read
  ● llm.response  content: "The file contains..." — 0.0s, $0.000

  ● llm.routed    → Claude Sonnet — code modification needed
  ● tool.requested  filesystem_write: src/auth.ts
  ● llm.response  content: "Here's the fix..." — 1.8s, $0.007

  ● llm.routed    → Llama 3 (local) — confirmation message
  ● llm.response  content: "Done! Tests pass." — 0.3s, $0.000
```

---

## Fallback Chain

If a model is unavailable, the router tries the next best option:

```
Chosen: Claude Sonnet
  ↓ unavailable (API error)
Fallback 1: GPT-4o (similar capability)
  ↓ unavailable (no API key)
Fallback 2: Gemini Flash (reduced capability)
  ↓ unavailable (no API key)
Fallback 3: Ollama local (always available if installed)
  ↓ unavailable (Ollama not running)
Error: No models available. Check your provider settings.
```

Each fallback emits an event so the Inspector shows what happened.

---

## Implementation Notes

### Where the Router Lives

In the **Python sidecar**, as a module between the chat handler and the provider:

```
apps/sidecar/
├── agent/
│   ├── router/
│   │   ├── smart_router.py     # Main routing logic
│   │   ├── rules.py            # Default + custom routing rules
│   │   ├── capabilities.py     # Model capability matrix
│   │   └── budget.py           # Budget tracking + limits
│   ├── chat.py                 # Calls router before calling provider
│   └── providers/              # Existing provider implementations
```

### Router API

Tauri sends the full agent config (including routing mode + rules) with each chat request. The sidecar's router uses this to decide.

```python
class SmartRouter:
    async def route(
        self,
        message: str,
        context_tokens: int,
        has_images: bool,
        tools: list[str],
        routing_mode: str,        # "single", "hybrid_auto", "hybrid_manual"
        routing_rules: list[dict],
        budget_remaining: float,
        available_models: list[str],
    ) -> RoutingDecision:
        """Pick the best model for this request."""
```

### Budget Tracking

Budget data stored in SQLite (Tauri layer). Sidecar receives `budget_remaining` with each request. After each response, Tauri updates the budget based on actual token usage.

---

## Phase Mapping

| Component | Phase | Effort |
|---|---|---|
| Single model mode (exists) | Done | — |
| Hybrid Manual (user-defined rules) | Phase 1 | Low |
| Model capability matrix | Phase 1 | Low |
| Budget tracking (monthly, per-agent) | Phase 1 | Medium |
| Budget UI in Settings | Phase 1 | Low |
| Hybrid Auto (smart routing) | Phase 2 | Medium |
| Routing events in Inspector | Phase 2 | Low |
| Savings calculation ("hybrid saved X%") | Phase 2 | Low |
| Fallback chain | Phase 2 | Low |
| Custom model capability tagging | Phase 2 | Low |
