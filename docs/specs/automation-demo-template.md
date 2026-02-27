# Automation Demo Templates

**Status**: SUPERSEDED by `scheduler-and-workflow-ux.md` (consolidated spec with Cron Trigger + Workflow List UX + templates)
**Phase**: 4C (automation canvas)
**Priority**: P0 — needed for stakeholder demo
**Author**: AI Studio PM
**Date**: 2026-02-25

---

## Problem Statement

AI Studio needs a compelling demo that showcases end-to-end automation capabilities — webhook triggers, RAG, LLM analysis, approval gates, and email notifications — in a single workflow. The demo must be impressive to engineering leadership evaluating workflow automation tools, showing that AI Studio can handle the same use cases as web-based workflow builders while offering superior AI-native features (inspector, node editor, hybrid intelligence).

---

## Demo Scenario: Automated Code Review & Documentation Pipeline

### Story

> "When a developer pushes code, the system automatically analyzes the changes, checks them against team coding standards (RAG), generates a review summary with recommendations, routes for approval if critical, and emails the team lead with the results."

This maps directly to real use cases discussed in enterprise workflow automation evaluations: automated documentation generation for code changes + policy document lookup.

### Workflow Template: "Code Change Analyzer"

```
Webhook Trigger → Transform (parse payload)
                    → Knowledge Base (lookup coding standards)
                        → LLM (analyze changes vs standards)
                            → Router (critical?)
                                ├── critical → Approval → Email Send (urgent)
                                └── normal → Email Send (summary)
                                    → Output (report)
```

### Node-by-Node

| # | Node | Type | Purpose |
|---|------|------|---------|
| 1 | Webhook Trigger | webhook_trigger | Receives POST from CI/CD with code diff |
| 2 | Parse Payload | transform | Extract `repo`, `branch`, `diff`, `author` from webhook body |
| 3 | Standards Lookup | knowledge_base | RAG search (azure_openai / text-embedding-3-large, recursive, 500, topK=5) |
| 4 | Code Analyzer | llm | azure_openai / gpt-4o-mini / temp=0.2 — "Analyze this code change against our standards. Rate severity: critical/normal. List violations." |
| 5 | Severity Router | router | Route by LLM's severity rating |
| 6 | Approval Gate | approval | Human review required for critical changes |
| 7 | Urgent Email | email_send | Email team lead: "CRITICAL: Code review needs attention" |
| 8 | Summary Email | email_send | Email author: "Code review complete — see report" |
| 9 | Report Output | output | Final structured report |

### Template Config

```json
{
  "name": "Code Change Analyzer",
  "description": "Webhook-triggered code review: RAG standards lookup, LLM analysis, severity routing, email notification",
  "category": "automation",
  "tags": ["webhook", "rag", "email", "code-review", "enterprise"]
}
```

### Demo Script (2 minutes)

1. **Open template** — "Here's a pre-built automation pipeline"
2. **Walk the graph** — "Webhook receives code changes, RAG finds standards, LLM analyzes, routes by severity, emails results"
3. **Arm the webhook** — Click Arm button, show URL
4. **Fire it** — `curl -X POST http://localhost:9876/hook/code-review -H "Content-Type: application/json" -d '{"repo":"api-service","branch":"feature/auth","diff":"+ if user.role == admin:","author":"alice@team.com"}'`
5. **Watch execution** — Inspector shows each node lighting up in sequence
6. **Show email** — "Team lead got an email with the analysis"
7. **Key differentiator** — "Unlike web-based builders, everything runs locally. Your code never leaves your machine. And you can inspect every step."

---

## Demo Scenario 2: Scheduled Report Generator

### Story

> "Every morning at 9am, the system collects data, generates an AI summary, and emails the daily report."

### Workflow Template: "Daily AI Report"

```
Cron Trigger (daily 9am) → HTTP Request (fetch data)
                             → LLM (summarize)
                                 → Email Send (distribute)
                                     → Output (archive)
```

| # | Node | Type | Purpose |
|---|------|------|---------|
| 1 | Daily Schedule | cron_trigger | Fires at 9:00 AM daily |
| 2 | Fetch Data | http_request | GET from internal API / data source |
| 3 | Summarize | llm | azure_openai / gpt-4o-mini / temp=0.2 — "Summarize this data into a concise daily report" |
| 4 | Email Report | email_send | Send to distribution list |
| 5 | Archive | output | Store report output |

### Template Config

```json
{
  "name": "Daily AI Report",
  "description": "Cron-scheduled: fetch data, AI summarize, email distribution",
  "category": "automation",
  "tags": ["cron", "email", "report", "scheduled"]
}
```

---

## Demo Scenario 3: Webhook → RAG → Response (Simplest)

### Story

> "An external app sends a question via webhook, AI Studio looks up the knowledge base and responds instantly."

### Workflow Template: "Smart API Endpoint"

```
Webhook Trigger (wait mode) → Knowledge Base (search)
                                → LLM (answer with context)
                                    → Output (return to caller)
```

This template uses **wait mode** — the webhook caller gets the LLM response directly in the HTTP response body. Turns AI Studio into a smart API endpoint.

| # | Node | Type | Purpose |
|---|------|------|---------|
| 1 | API Endpoint | webhook_trigger | Receives question, wait mode returns answer |
| 2 | Knowledge Search | knowledge_base | RAG lookup (azure_openai / text-embedding-3-large, recursive, 500, topK=5) |
| 3 | Answer Generator | llm | azure_openai / gpt-4o-mini / temp=0.2 — Generate answer using RAG context |
| 4 | Response | output | Returns to webhook caller |

### Template Config

```json
{
  "name": "Smart API Endpoint",
  "description": "Webhook receives question, RAG finds context, LLM answers, response returned to caller",
  "category": "automation",
  "tags": ["webhook", "rag", "api", "wait-mode"]
}
```

---

## LLM Provider Settings (Proven Working)

All templates MUST use these exact settings (verified working in enterprise environment):

| Setting | Value |
|---------|-------|
| LLM Provider | `azure_openai` |
| LLM Model | `gpt-4o-mini` |
| Temperature | `0.2` |
| Embedding Provider | `azure_openai` |
| Embedding Model | `text-embedding-3-large` |
| KB Chunk Strategy | `recursive` |
| KB Chunk Size | `500` |
| KB Top K | `5` |

Do NOT use Ollama, local providers, or any other model in enterprise-facing templates.

---

## Implementation Plan

### Prerequisites
- [ ] Webhook Trigger backend — DONE (9ee19c7)
- [ ] Webhook Trigger UI — needed
- [ ] Email Send node — needed (new spec: `email-node.md`)
- [ ] Cron Trigger — needed (new spec: `cron-trigger.md`)

### Template Implementation (after prerequisites)
- [ ] Template #16: "Code Change Analyzer" — webhook + RAG + LLM + router + approval + email
- [ ] Template #17: "Daily AI Report" — cron + HTTP + LLM + email
- [ ] Template #18: "Smart API Endpoint" — webhook (wait) + RAG + LLM

### Demo Prep
- [ ] Index sample coding standards documents in Knowledge Base
- [ ] Prepare curl commands for webhook testing
- [ ] Test end-to-end with Azure OpenAI provider
- [ ] Record 2-minute demo video

---

## What This Proves (vs Web-Based Builders)

| Capability | AI Studio | Web-Based Builders |
|-----------|-----------|-------------------|
| **Runs locally** | Desktop-native, data never leaves machine | Cloud-hosted, data goes through vendor |
| **AI-native** | LLM nodes, RAG, hybrid intelligence built-in | LLM as add-on integration |
| **Visual debugging** | Inspector: event timeline, cost tracking, replay | Basic execution logs |
| **Enterprise security** | No cloud dependency, localhost webhooks, approval gates | Requires cloud subscription + data agreements |
| **Trigger types** | Webhook, Cron, File Watch (planned) | Webhook, Cron, many integrations |
| **Node types** | 20+ AI-focused nodes | 400+ integration-focused nodes |

**AI Studio's pitch**: "Fewer integrations, deeper AI capabilities. Every node is AI-aware."
