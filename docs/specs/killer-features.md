# AI Studio — Killer Features Spec

**Status**: PLANNED
**Phase**: 5+
**Goal**: Features that make AI Studio the only visual agent IDE worth using. Each one is a "holy shit" moment that competitors don't have.

---

## 1. Time-Travel Debug

**One-liner**: Click any node in a completed workflow, edit its output, re-run from that point forward.

### Problem
When a workflow fails at node 7 of 10, you currently re-run the entire thing. That wastes time, tokens, and money. You can't experiment with "what if this node returned something different?"

### Design
- Every completed node stores its output in `workflowNodeStates`
- **Rewind**: Click a completed node → "Edit Output" button appears in config panel
- **Edit**: Modify the output text/JSON in a modal (pre-filled with actual output)
- **Replay**: Click "Re-run from here" → engine skips nodes 1-6 (uses cached outputs), re-executes 7-10 with the edited output injected
- **Diff view**: Show what changed between original run and replayed run

### Engine changes
- `run_workflow` accepts optional `override_outputs: HashMap<NodeId, String>` — nodes with overrides skip execution and use the provided output
- `start_from_node: Option<NodeId>` — skip all nodes before this one, use cached outputs from previous run
- Previous run's `workflowNodeStates` stored per run (not just latest)

### UI
```
[Node 5: completed ✓] → click → Config Panel shows:
  Output: "The quarterly revenue increased by 12%..."
  [Edit Output]  [Re-run from here →]
```

### Why this wins
- Nobody has this. LangFlow, Flowise, n8n — all re-run from scratch.
- Saves real money on LLM calls during development
- Makes prompt iteration 10x faster — change one node's behavior, see downstream impact instantly

---

## 2. A/B Eval Node

**One-liner**: Run the same input through multiple LLMs, compare outputs side-by-side with scores.

### Problem
"Which model should I use for this task?" Currently: build separate workflows, run each, manually compare. Nobody does this because it's tedious.

### Design

**New node type: `eval`**

Config:
| Field | Type | Description |
|-------|------|-------------|
| providers | array | List of `{provider, model}` pairs to compare |
| criteria | array | Scoring dimensions (e.g. "accuracy", "conciseness", "cost") |
| judgeProvider | string | Optional: LLM that scores the outputs (LLM-as-judge) |
| judgePrompt | string | Prompt for the judge LLM |
| runs | int | Number of runs per model (for variance measurement) |

Input handle: text (the prompt to evaluate)
Output handle: json (results matrix)

**Output format**:
```json
{
  "results": [
    {
      "provider": "anthropic",
      "model": "claude-sonnet-4-5",
      "output": "...",
      "latencyMs": 1200,
      "tokens": 342,
      "costUsd": 0.004,
      "scores": { "accuracy": 9, "conciseness": 8 }
    },
    {
      "provider": "google",
      "model": "gemini-2.0-flash",
      "output": "...",
      "latencyMs": 400,
      "tokens": 289,
      "costUsd": 0.001,
      "scores": { "accuracy": 7, "conciseness": 9 }
    }
  ],
  "winner": "claude-sonnet-4-5",
  "summary": "Claude scored higher on accuracy, Gemini was faster and cheaper"
}
```

**UI: Eval Results Panel** (replaces standard output preview for eval nodes):
```
┌─────────────────────────────────────────────────┐
│  A/B EVAL RESULTS                               │
├──────────┬──────────┬──────────┬────────────────┤
│          │ Sonnet   │ Gemini   │ GPT-4o         │
├──────────┼──────────┼──────────┼────────────────┤
│ Accuracy │ ★★★★★   │ ★★★★     │ ★★★★           │
│ Speed    │ 1.2s     │ 0.4s     │ 0.8s           │
│ Cost     │ $0.004   │ $0.001   │ $0.003         │
│ Tokens   │ 342      │ 289      │ 310            │
├──────────┼──────────┼──────────┼────────────────┤
│ Output   │ [expand] │ [expand] │ [expand]       │
└──────────┴──────────┴──────────┴────────────────┘
  Winner: claude-sonnet-4-5 (highest accuracy)
```

### Workflow example
```
Input ("Explain quantum computing to a 10-year-old")
  → Eval (Sonnet vs Gemini vs GPT-4o, judge: Claude)
  → Output (results matrix)
```

### Why this wins
- Prompt engineers desperately need this
- Built-in, visual, one-click — not a separate tool
- LLM-as-judge scoring is cutting edge
- The eval results table is screenshot-worthy (Twitter/X viral potential)

---

## 3. Auto-Pipeline Generator

**One-liner**: Describe what you want in English, AI generates the workflow graph.

### Problem
New users see an empty canvas and don't know where to start. Power users want to prototype faster. Both benefit from "describe it, get a graph."

### Design

**Trigger**: Text input at top of canvas or command palette
```
"Build a pipeline that reads CSV files, translates each row to Spanish, and writes the output"
```

**Backend**: Single LLM call with structured output
- System prompt: "You are a workflow generator for AI Studio. Given a description, output a React Flow graph JSON with nodes and edges. Available node types: [list all 16+]. Each node needs an id, type, position, and data fields."
- Output: Valid `{nodes, edges}` JSON
- Engine validates the generated graph (type compatibility, required fields)
- Graph loads onto canvas with `fitView`

**UX flow**:
1. User types description in a text box (or command palette `Ctrl+K` → "Generate workflow")
2. Loading spinner: "Generating workflow..."
3. Graph appears on canvas (animated node placement)
4. User tweaks nodes, connects missing edges, adjusts config
5. Run

**Fallback**: If generated graph is invalid, show error + the raw JSON for manual fix

### Why this wins
- The "wow" demo moment. "Watch me build a RAG pipeline in 10 seconds."
- Show HN title writes itself: "AI Studio: describe an AI pipeline, watch it build itself"
- Lowers barrier to entry dramatically
- Meta: using AI to build AI pipelines

---

## 4. Guardrails Node

**One-liner**: Drop-in safety checks — PII detection, content filtering, hallucination detection, schema enforcement.

### Problem
AI outputs are unpredictable. Before sending LLM output to a database, API, or user, you need safety checks. Currently: build custom validation logic per workflow. Nobody does it because it's tedious.

### Design

**New node type: `guardrail`**

Config:
| Field | Type | Description |
|-------|------|-------------|
| checks | array | Which guardrails to enable |
| action | enum | `block` (stop flow), `flag` (continue with warning), `redact` (remove and continue) |
| customRules | array | User-defined regex patterns or LLM-based checks |

**Built-in checks**:
| Check | What it detects |
|-------|----------------|
| `pii_email` | Email addresses |
| `pii_phone` | Phone numbers |
| `pii_ssn` | Social security numbers |
| `pii_credit_card` | Credit card numbers |
| `profanity` | Offensive language |
| `prompt_injection` | Attempts to override system prompt |
| `hallucination` | Claims not supported by input context (LLM-based) |
| `off_topic` | Response doesn't address the input (LLM-based) |
| `json_schema` | Output doesn't match expected schema |
| `length` | Response too short or too long |

**Output handles**:
- `passed` — content that cleared all checks
- `blocked` — content that failed (with reasons)
- `report` — JSON report of all checks and results

**Workflow**:
```
Input → LLM → Guardrail (PII + profanity + length)
                 ├── passed → Output
                 └── blocked → LLM ("Rewrite without PII") → Output
```

### Why this wins
- Enterprise credibility — companies can't ship AI without safety checks
- Visual guardrails in the pipeline = easy to audit, easy to explain to compliance
- "Add safety in 2 seconds" — drag, drop, connect
- Open-source guardrails library = community contributions

---

## 5. RAG Pipeline Nodes

**One-liner**: First-class nodes for building RAG (Retrieval Augmented Generation) pipelines visually.

### Problem
RAG is the #1 AI use case in production. Currently: hack it with HTTP Request → external vector DB. No native support for chunking, embedding, or vector search.

### New node types

#### 5a. Text Chunker

Split documents into overlapping chunks for embedding.

Config:
| Field | Type | Description |
|-------|------|-------------|
| strategy | enum | `fixed_size`, `sentence`, `paragraph`, `recursive` |
| chunkSize | int | Target characters per chunk (default 500) |
| overlap | int | Characters of overlap between chunks (default 50) |
| separator | string | Custom split character (for fixed_size) |

Input: text (document content)
Output: json (array of `{text, index, startChar, endChar}`)

#### 5b. Embedding

Convert text chunks to vector embeddings.

Config:
| Field | Type | Description |
|-------|------|-------------|
| provider | enum | `openai`, `cohere`, `local`, `huggingface` |
| model | string | Embedding model name (e.g. `text-embedding-3-small`) |
| batchSize | int | Chunks per API call (default 100) |

Input: json (array of text chunks)
Output: json (array of `{text, vector, index}`)

#### 5c. Vector Search

Query a vector database for similar chunks.

Config:
| Field | Type | Description |
|-------|------|-------------|
| provider | enum | `pinecone`, `chroma`, `qdrant`, `pgvector`, `local` |
| connectionId | string | Reference to Settings connection |
| topK | int | Number of results (default 5) |
| scoreThreshold | float | Minimum similarity score (0-1) |
| namespace | string | Vector DB namespace/collection |

Input: text (query to search for) or json (query vector)
Output: json (array of `{text, score, metadata}`)

### Full RAG workflow
```
                                    ┌─ Chunker → Embedding → Vector DB (index)
File Glob (docs/*.pdf) ────────────┤
                                    └─ (indexing done once, stored in vector DB)

Input (question) → Vector Search (top 5) → Transform (format context) → LLM ("Answer using: {{context}}") → Output
```

### Why this wins
- RAG is the #1 production AI pattern — every team builds this
- Visual RAG builder = "build a RAG pipeline in 60 seconds, no code"
- Supports local vector DBs (Chroma) — works offline, privacy-first
- Combines with Iterator for batch indexing

---

## 6. EIP: Error Handler (Dead Letter Channel)

**One-liner**: Route errors to a fallback path instead of killing the workflow.

### Problem
Node fails → workflow stops. In production, you want: retry, fallback to a different model, log the error and continue, or alert someone.

### Design

Every node gets an optional `onError` output handle (red, only visible when enabled in config).

Config addition (all nodes):
| Field | Type | Description |
|-------|------|-------------|
| onError | enum | `stop` (default), `fallback`, `retry`, `skip` |
| retryCount | int | For retry mode (default 3) |
| retryDelay | int | Milliseconds between retries |

**Fallback mode**: Error output handle activates. Connect it to recovery logic:
```
LLM (Claude) ──── onError → LLM (Ollama local) → Output
      │
      └── success → Output
```

**Skip mode**: Node is marked as skipped, downstream gets null input. Useful for optional enrichment steps.

### Why this wins
- Table stakes for production workflows — n8n and Zapier both have this
- Visual error handling = easy to understand and debug
- Retry with different models = hybrid intelligence at the error level

---

## 7. EIP: Content Enricher

**One-liner**: Merge data from an external source into the current message.

### Design

**New node type: `enricher`**

Config:
| Field | Type | Description |
|-------|------|-------------|
| source | enum | `http`, `sql`, `file`, `static` |
| query | string | URL / SQL query / file path / static JSON |
| mergeStrategy | enum | `merge` (deep merge), `append` (add field), `replace` |
| targetField | string | Where to put the enrichment data |

Input: json (message to enrich)
Output: json (enriched message)

**Example**: Enrich a customer record with data from a DB
```
Input (customer_id) → SQL Query (SELECT * FROM customers WHERE id = {{input}})
                        → Enricher (merge customer data with order history)
                          → LLM ("Summarize this customer's profile")
                            → Output
```

---

## 8. EIP: Wire Tap

**One-liner**: Copy node output to a side channel without affecting the main flow.

### Design

**New node type: `wiretap`**

Config:
| Field | Type | Description |
|-------|------|-------------|
| destination | enum | `file`, `http`, `console`, `inspector` |
| path | string | File path or webhook URL |
| format | enum | `json`, `text`, `csv` |
| async | bool | Non-blocking (default true) |

Connects as a **branch** off any edge — data flows through to the next node AND copies to the wire tap.

```
LLM → [main flow continues] → Output
  └── Wire Tap (log to /tmp/audit.jsonl)
```

### Why this wins
- Essential for debugging production workflows
- Audit trail without modifying the pipeline logic
- Non-blocking — doesn't slow down execution

---

## 9. SQL Query Node

**One-liner**: Query databases directly from workflows.

### Design

**New node type: `sql_query`**

Config:
| Field | Type | Description |
|-------|------|-------------|
| connectionId | string | Reference to a saved connection in Settings |
| query | string | SQL with `{{input}}` template substitution |
| mode | enum | `query` (SELECT → rows) or `execute` (INSERT/UPDATE/DELETE → affected count) |
| timeout | int | Seconds (default 30) |
| maxRows | int | Limit results (default 1000) |
| parameterized | bool | Use parameterized queries to prevent SQL injection |

Input: text or json (for template substitution / parameters)
Output: rows (query results) or json (affected count + metadata)

**Settings: Connections tab**
```
+ Add Connection
  Name:     prod-db
  Type:     PostgreSQL / MySQL / SQLite / MSSQL
  Host:     localhost
  Port:     5432
  Database: myapp
  Username: ****
  Password: ****  (encrypted at rest)
  SSL:      true
  Test Connection: [Test]
```

Connections stored in SQLite settings table as `connection.{id}.{field}` — same pattern as provider config.

**Security**:
- Parameterized queries by default (prevent SQL injection)
- Read-only mode option (blocks INSERT/UPDATE/DELETE)
- Connection credentials encrypted at rest
- Query timeout enforced

### Killer workflow: Natural Language → SQL
```
Input ("Show users who signed up this week")
  → LLM ("Convert to PostgreSQL query")
  → Approval ("Review this SQL before running")
  → SQL Query (execute)
  → LLM ("Summarize these results")
  → Output
```

---

## 10. Agent-Workflow Unification

**One-liner**: An agent IS a workflow. Build agent behavior visually, then chat with it.

### Problem
"Agents" and "Workflows" feel like separate concepts. Agents are just LLM config (boring). Workflows are where the real agent behavior lives.

### Design

- Every agent has a `workflowId` (already exists in schema)
- "Create Agent" → picks a starter template → opens workflow editor
- "Chat with Agent" → runs the agent's workflow with the chat message as Input
- Simple agents: just Input → LLM → Output (equivalent to current agent)
- Complex agents: full graph with tools, routing, approval, memory

**Agent card becomes**:
```
┌──────────────────────────────┐
│  My RAG Agent                │
│  12 nodes · claude-sonnet    │
│  Last run: 2 min ago         │
│  [Chat]  [Edit Blueprint]    │
└──────────────────────────────┘
```

**Chat with Agent** runs the workflow per message:
1. User sends "What were our Q4 earnings?"
2. Workflow runs: Input → Vector Search → LLM → Output
3. Response appears in chat
4. Inspector shows exactly what happened inside

### Why this wins
- One coherent product story: "Build agents visually. Chat with them. See inside every decision."
- That's the Show HN title
- Nobody else does all three in one tool

---

## Implementation Priority

| # | Feature | Effort | Impact | Priority Score |
|---|---------|--------|--------|---------------|
| 1 | A/B Eval Node | Medium | Very High | **P0** — demo showstopper |
| 2 | Time-Travel Debug | Medium | Very High | **P0** — unique differentiator |
| 3 | Auto-Pipeline Generator | Low | Very High | **P0** — single LLM call, massive wow factor |
| 4 | Error Handler (Dead Letter) | Low | High | **P1** — production readiness |
| 5 | Guardrails Node | Medium | High | **P1** — enterprise credibility |
| 6 | SQL Query Node | Medium | High | **P1** — backend connectivity |
| 7 | Agent-Workflow Unification | High | Very High | **P1** — product coherence |
| 8 | RAG Pipeline Nodes | High | High | **P2** — 3 new nodes + vector DB integration |
| 9 | Content Enricher | Low | Medium | **P2** — nice EIP addition |
| 10 | Wire Tap | Low | Medium | **P2** — debugging/audit |

**Phase 5A** (demo-ready): Items 1-3 (A/B Eval, Time-Travel, Auto-Pipeline)
**Phase 5B** (production-ready): Items 4-7 (Error Handler, Guardrails, SQL, Unification)
**Phase 5C** (ecosystem): Items 8-10 (RAG, Enricher, Wire Tap)
