# AI Studio — Node Editor Specification

> **Version**: 2.0 (expanded to universal automation canvas vision)
> **Status**: Phase 3 DONE — Phase 4 detailed spec in `phase4-automation-canvas.md`
> **Phase**: 3 (built), 4 (planned)
> **Depends on**: architecture.md, event-system.md, data-model.md, mcp-integration.md, hybrid-intelligence.md
> **Library**: React Flow (@xyflow/react)
> **This is the 10k-star feature.**

---

## What Is the Node Editor?

The Node Editor is a **universal automation canvas** — connect any input, any processing, any output, and watch data flow through the pipeline in real-time. AI (LLM nodes) is a first-class building block, not a bolt-on.

**Phase 3** (DONE): AI workflow builder — Input/Output/LLM/Tool/Router/Approval/Transform/Subworkflow nodes with DAG execution engine.

**Phase 4** (Vision): Universal automation — database connectors, file I/O, HTTP/webhook, message queues, IoT devices, code execution, validators, loops, caching. Any input source → any processing → any output destination.

This is where AI Studio graduates from "agent IDE" to "visual automation platform." Think **Node-RED meets ComfyUI** — but with AI-native observability, hybrid model routing, and full cost tracking on every node.

```
┌─────────────────────────────────────────────────────────────────────────┐
│  Node Editor                                    [Workflow: Code Review] │
├──────────┬──────────────────────────────────────────────────────────────┤
│          │                                                              │
│  Node    │  ┌──────────┐     ┌──────────┐     ┌──────────┐            │
│  Palette │  │  Input   │────▶│  Claude   │────▶│  GitHub  │            │
│          │  │  (PR URL)│     │  Sonnet   │     │  Comment │            │
│  ┌─────┐ │  └──────────┘     └────┬─────┘     └──────────┘            │
│  │ LLM │ │                        │                                    │
│  ├─────┤ │                   ┌────▼─────┐     ┌──────────┐            │
│  │Tool │ │                   │  Router   │────▶│  Slack   │            │
│  ├─────┤ │                   │ (severity)│     │  Notify  │            │
│  │Gate │ │                   └──────────┘     └──────────┘            │
│  ├─────┤ │                                                              │
│  │Data │ │  Status: Ready  │  Nodes: 5  │  Est Cost: $0.003           │
│  └─────┘ │                                                              │
└──────────┴──────────────────────────────────────────────────────────────┘
```

---

## Why This Matters

| Without Node Editor | With Node Editor |
|---------------------|-----------------|
| Agent = one LLM + one system prompt | Agent = a graph of steps, decisions, and tools |
| Single-shot conversations only | Multi-step pipelines with branching logic |
| Tool chains are implicit (LLM decides) | Tool chains are explicit and visual |
| Can't see the plan before execution | See the full workflow before running |
| Debugging = reading event logs | Debugging = clicking a node and seeing its I/O |
| No reuse — every agent starts from scratch | Workflows are composable — share, fork, remix |

**Competitive moat**: ComfyUI proved visual node editors create massive communities (15M+ users). Langflow proved it works for LLM pipelines. AI Studio combines both with local-first desktop-native execution, hybrid intelligence, and the Inspector for deep debugging. No other tool has this combination.

---

## Core Concepts

### Workflow

A **Workflow** is a directed acyclic graph (DAG) of nodes and edges that defines a multi-step AI pipeline. Workflows are stored as JSON and attached to agents.

```typescript
interface Workflow {
  id: string;
  name: string;
  description: string;
  nodes: WorkflowNode[];
  edges: WorkflowEdge[];
  viewport: { x: number; y: number; zoom: number };
  variables: WorkflowVariable[];    // User-defined workflow inputs
  createdAt: string;
  updatedAt: string;
}
```

### Node

A **Node** is a single step in the pipeline. Each node has a type, configuration, input handles, and output handles.

```typescript
interface WorkflowNode {
  id: string;
  type: NodeType;
  position: { x: number; y: number };
  data: NodeData;                   // Type-specific configuration
}

// Phase 3 — Built and working
type NodeType =
  | 'input'          // Workflow entry point (user prompt, file, data)
  | 'output'         // Workflow exit point (final result)
  | 'llm'            // LLM inference (any provider/model)
  | 'tool'           // MCP tool or built-in tool execution
  | 'router'         // Conditional branching (if/else, switch)
  | 'approval'       // Human-in-the-loop gate
  | 'transform'      // Data transformation (template, extract, merge)
  | 'subworkflow'    // Embed another workflow as a node
  // Phase 4 — Planned (universal automation canvas)
  | 'database_read'  // SQL query against any database
  | 'database_write' // INSERT/UPDATE/DELETE
  | 'file_read'      // Read CSV/JSON/text/PDF from filesystem
  | 'file_write'     // Write files to filesystem
  | 'http_request'   // GET/POST to REST/GraphQL APIs
  | 'http_post'      // Send data to external endpoints
  | 'webhook_listen' // HTTP endpoint triggers workflow
  | 'queue_consume'  // Read from message queues (Kafka, RabbitMQ, MQTT)
  | 'queue_publish'  // Push to message queues
  | 'iot_sensor'     // Read from IoT devices
  | 'iot_command'    // Send commands to IoT devices
  | 'cron_trigger'   // Scheduled execution
  | 'shell_input'    // Run shell command, capture stdout as data
  | 'stdin_pipe'     // Read from stdin or named pipe (streaming)
  | 'notification'   // Email, Slack, Discord, SMS
  | 'shell_exec'     // Execute commands with privilege controls (run_as, sandbox)
  | 'display'        // Rich visual output (charts, tables)
  | 'code'           // Python/JS sandboxed execution
  | 'validator'      // JSON Schema / data quality checks
  | 'merge'          // Wait for N branches (AND/OR logic)
  | 'loop'           // Iterate over array input
  | 'cache'          // Memoize expensive operations
  | 'rate_limiter'   // Throttle execution rate
  | 'error_handler'; // Catch errors, retry, fallback
```

### Edge

An **Edge** connects an output handle of one node to an input handle of another. Edges carry typed data.

```typescript
interface WorkflowEdge {
  id: string;
  source: string;        // Source node ID
  sourceHandle: string;  // Output handle ID
  target: string;        // Target node ID
  targetHandle: string;  // Input handle ID
  label?: string;        // Optional label (e.g., "if true")
}
```

### Handle

**Handles** are typed connection ports on nodes. Type checking prevents invalid connections.

```typescript
type HandleDataType =
  | 'text'        // String data
  | 'json'        // Structured data (objects, arrays)
  | 'boolean'     // True/false
  | 'file'        // File path or binary data
  | 'any'         // Accepts any type
  // Phase 4 additions:
  | 'rows'        // Database result set (array of objects)
  | 'stream'      // Continuous data stream (IoT, queue)
  | 'binary'      // Raw binary data (images, PDFs)
  | 'number';     // Numeric value (sensor readings, counts)

interface HandleDef {
  id: string;
  label: string;
  dataType: HandleDataType;
  required: boolean;
}
```

---

## Node Types (Detail)

### 1. Input Node

Entry point for the workflow. Defines what the user provides when running.

```
┌─────────────────────┐
│  INPUT               │
│                      │
│  Name: pr_url        │
│  Type: text          │
│  Default: (none)     │
│                      │
│              [text]──●
└─────────────────────┘
```

| Field | Type | Description |
|-------|------|-------------|
| name | string | Variable name (referenced by other nodes) |
| dataType | HandleDataType | Expected input type |
| default | string? | Default value if not provided |
| description | string? | Help text shown in run form |

**Outputs**: One handle matching the declared type.

### 2. Output Node

Exit point. Captures the final result of the workflow.

```
┌─────────────────────┐
│  OUTPUT              │
│                      │
●──[text]  Name: result│
│          Format: md  │
│                      │
└─────────────────────┘
```

| Field | Type | Description |
|-------|------|-------------|
| name | string | Result name |
| format | 'text' \| 'markdown' \| 'json' | How to display the result |

**Inputs**: One handle accepting any type.

### 3. LLM Node

Calls a language model. The core building block.

```
┌─────────────────────────┐
│  LLM: Claude Sonnet      │
│                          │
●──[text]  prompt          │
●──[text]  system          │
●──[json]  context         │
│                          │
│  Provider: anthropic     │
│  Model: claude-sonnet-4-5│
│  Temp: 0.7  MaxTok: 4096│
│  Mode: ○ Fixed ● Hybrid │
│                          │
│           response [text]──●
│           usage   [json]──●
│           cost    [text]──●
└─────────────────────────┘
```

| Field | Type | Description |
|-------|------|-------------|
| provider | string | Provider ID (anthropic, openai, ollama, google) |
| model | string | Model ID |
| systemPrompt | string | System prompt (can reference variables via `{{var}}`) |
| temperature | number | 0.0 - 2.0 |
| maxTokens | number | Max response tokens |
| routingMode | 'fixed' \| 'hybrid' | Use specified model or let hybrid router pick |

**Inputs**: `prompt` (text, required), `system` (text, optional), `context` (json, optional)
**Outputs**: `response` (text), `usage` (json: tokens, cost), `cost` (text: formatted USD)

### 4. Tool Node

Executes an MCP tool or built-in tool.

```
┌─────────────────────────┐
│  TOOL: github__create    │
│        _issue            │
│                          │
●──[text]  title           │
●──[text]  body            │
●──[text]  repo            │
│                          │
│  Server: github-mcp     │
│  Approval: auto         │
│                          │
│           result  [json]──●
│           success [bool]──●
└─────────────────────────┘
```

| Field | Type | Description |
|-------|------|-------------|
| toolName | string | Full tool name (e.g., `builtin__shell`, `github__create_issue`) |
| serverName | string? | MCP server name (null for built-in) |
| approval | 'auto' \| 'ask' \| 'deny' | Override approval behavior |
| inputMapping | Record<string, string> | Map input handles to tool parameters |

**Inputs**: Dynamic — generated from the tool's input schema (discovered via MCP).
**Outputs**: `result` (json), `success` (boolean)

### 5. Router Node

Conditional branching. Routes data based on conditions.

```
┌─────────────────────────┐
│  ROUTER                  │
│                          │
●──[any]  input            │
│                          │
│  Mode: ● LLM  ○ Pattern │
│                          │
│  Classify into:          │
│  ┌─ bug     → [any]────●│
│  ├─ feature → [any]────●│
│  └─ question→ [any]────●│
└─────────────────────────┘
```

**Two modes:**

| Mode | How It Works |
|------|-------------|
| **LLM Classify** | Sends input to a small/fast LLM with instructions to classify into one of N categories. Cost-efficient — uses local model or cheapest cloud model. |
| **Pattern Match** | Regex or keyword matching on the input text. Free, instant, deterministic. |

| Field | Type | Description |
|-------|------|-------------|
| mode | 'llm' \| 'pattern' | Classification method |
| branches | Branch[] | Named output branches |
| classifyModel | string? | Model for LLM classification (if mode=llm) |
| fallbackBranch | string | Branch for unmatched inputs |

**Inputs**: `input` (any, required)
**Outputs**: One handle per branch (dynamic).

### 6. Approval Node

Human-in-the-loop gate. Pauses execution until a human approves.

```
┌─────────────────────────┐
│  APPROVAL GATE           │
│                          │
●──[any]  data             │
│                          │
│  Show to user:           │
│  "Review before posting" │
│  Timeout: 5 min          │
│                          │
│           approved [any]──●
│           rejected [any]──●
└─────────────────────────┘
```

| Field | Type | Description |
|-------|------|-------------|
| message | string | What to show the user |
| showData | boolean | Display the incoming data for review |
| timeout | number? | Auto-reject after N seconds (null = wait forever) |
| autoApproveInRuns | boolean | Skip in headless runs (use with caution) |

**Inputs**: `data` (any, required)
**Outputs**: `approved` (any — passes data through), `rejected` (any — passes data through)

### 7. Transform Node

Data manipulation without calling an LLM. Fast, free, deterministic.

```
┌─────────────────────────┐
│  TRANSFORM               │
│                          │
●──[text]  input_a         │
●──[json]  input_b         │
│                          │
│  Mode: template          │
│  ┌──────────────────┐   │
│  │Review {{input_a}} │   │
│  │Context: {{input_b.│   │
│  │summary}}          │   │
│  └──────────────────┘   │
│           output  [text]──●
└─────────────────────────┘
```

**Three modes:**

| Mode | Description |
|------|-------------|
| **Template** | Mustache-style string interpolation (`{{var}}`, `{{var.field}}`) |
| **JSONPath** | Extract fields from JSON input |
| **Script** | JavaScript expression (sandboxed, no side effects) |

**Inputs**: Dynamic — user names them.
**Outputs**: `output` (type depends on mode).

### 8. Subworkflow Node

Embeds another workflow as a single node. Enables composition and reuse.

```
┌─────────────────────────┐
│  SUBWORKFLOW             │
│  "Summarize Document"    │
│                          │
●──[text]  document        │
│                          │
│  Workflow: doc-summarize │
│  ⟳ Click to expand      │
│                          │
│           summary [text]──●
└─────────────────────────┘
```

| Field | Type | Description |
|-------|------|-------------|
| workflowId | string | Referenced workflow ID |

**Inputs/Outputs**: Mapped from the referenced workflow's Input/Output nodes.

---

## Canvas UX

### Node Palette (Left Sidebar)

Draggable list of node types. Grouped by category:

```
Inputs / Outputs
  ├─ Input
  └─ Output

AI
  ├─ LLM
  └─ Router (LLM Classify)

Tools
  ├─ Shell
  ├─ Filesystem
  ├─ Browser
  └─ [MCP tools from connected servers]

Logic
  ├─ Router (Pattern)
  ├─ Approval Gate
  └─ Transform

Composition
  └─ Subworkflow
```

MCP tools are listed dynamically based on connected MCP servers. Each MCP tool becomes a pre-configured Tool node.

### Canvas Controls

| Control | Action |
|---------|--------|
| Drag from palette | Add node to canvas |
| Drag between handles | Create edge |
| Click node | Select (show config in side panel) |
| Delete/Backspace | Remove selected node or edge |
| Cmd+Z / Cmd+Shift+Z | Undo / Redo |
| Scroll wheel | Zoom |
| Click+drag canvas | Pan |
| Cmd+A | Select all |
| Cmd+C / Cmd+V | Copy / Paste nodes |
| Cmd+S | Save workflow |
| Cmd+Shift+R | Run workflow |

### Minimap

Bottom-right corner. Shows bird's-eye view of the full graph. Click to navigate.

### Validation

Real-time validation as users build:

| Validation | Visual |
|-----------|--------|
| Required input not connected | Red handle + tooltip |
| Type mismatch on edge | Dashed red edge + tooltip |
| Cycle detected | Red edges in cycle + toast |
| Subworkflow cycle (A embeds B embeds A) | Red subworkflow node + toast |
| No Input node | Warning banner at top |
| No Output node | Warning banner at top |
| Disconnected nodes | Yellow outline on orphaned nodes |

### Node Config Panel (Right Sidebar)

When a node is selected, the right panel shows its full configuration form. Same panel that the Inspector uses — keeps UX consistent.

---

## Execution Model

### How Workflows Run

Workflow execution reuses the existing sidecar architecture. The Tauri layer orchestrates — the sidecar executes individual steps.

**Critical design rule**: Workflow execution uses `POST /chat/direct` (stateless), NOT `POST /chat` (stateful). Rust owns all state — it builds the context window for each LLM node based on the DAG traversal path and sends the full message history. The sidecar is a pure compute engine for workflows, with zero in-memory conversation state. This prevents the split-brain bugs seen in session branching where Python held state that diverged from Rust's persisted state.

```
User clicks "Run"
    │
    ▼
UI sends: invoke('run_workflow', { workflowId, inputs })
    │
    ▼
Tauri (Rust):
    1. Validates workflow DAG (no cycles, all required inputs, no subworkflow cycles)
    2. Creates a Session for this run
    3. Topologically sorts nodes
    4. Walks the DAG:
       │
       ├─ Input node → inject user-provided values
       ├─ LLM node → POST /chat/direct to sidecar (Rust builds full context)
       ├─ Tool node → POST /tools/* to sidecar → approve if needed
       ├─ Router node → evaluate condition → pick branch
       ├─ Approval node → emit to UI → wait for response
       ├─ Transform node → evaluate locally in Rust (no sidecar call)
       ├─ Subworkflow → recursive execution (cycle check prevents A→B→A)
       └─ Output node → collect result
    │
    5. Each node execution emits events:
       - workflow.node.started { node_id, node_type, inputs }
       - workflow.node.completed { node_id, outputs, duration_ms, cost_usd }
       - workflow.node.error { node_id, error }
       - (plus all existing events: llm.*, tool.*, etc.)
    │
    6. Session stores all events (Inspector can replay)
    │
    ▼
UI receives events → updates node states in real-time
```

### Parallel Execution

When the DAG has independent branches (nodes with no data dependency between them), Rust executes them concurrently using `tokio::join_all`. This applies to:
- Router output branches (only the selected branch executes, but if multiple independent paths exist downstream they can run in parallel)
- Independent subgraphs within the workflow

**Concurrency limit**: Configurable per-workflow, default 4. This prevents overwhelming the sidecar with concurrent LLM requests. The sidecar must run with `uvicorn --workers 4` (or matching concurrency limit) for production workflows.

**Sidecar compatibility**: Since workflow execution uses `/chat/direct` (stateless), concurrent requests are safe — each request is independent with no shared conversation state.

### Node Execution States

Each node has a visual state during execution:

| State | Visual | Meaning |
|-------|--------|---------|
| `idle` | Default styling | Not yet reached |
| `running` | Blue pulsing border + spinner | Currently executing |
| `completed` | Green border + checkmark | Finished successfully |
| `error` | Red border + X icon | Failed |
| `waiting` | Yellow border + hourglass | Waiting for approval |
| `skipped` | Gray + dashed border | Branch not taken |

### Live Data Preview

During execution, each completed node shows a data preview badge:

```
┌─────────────────────────┐
│  LLM: Claude Sonnet  ✓  │
│  ─────────────────────  │
│  "The code has 3 issues │
│   that need..."         │
│  ─────────────────────  │
│  1,247 tokens · $0.004  │
│  2.3s                   │
└─────────────────────────┘
```

Clicking the node during or after execution opens the Inspector detail panel for that node's events.

---

## Data Flow

### Edge Data

Data flows through edges as typed values:

```typescript
interface EdgeData {
  type: HandleDataType;    // text, json, boolean, file, any
  value: unknown;          // The actual data
}
```

### Variable References

Nodes can reference workflow variables and upstream node outputs using template syntax:

```
{{input.pr_url}}              → Value from Input node named "pr_url"
{{node_abc123.response}}      → Output "response" from node abc123
{{node_abc123.usage.tokens}}  → Nested field access
```

### Type Checking

Edges enforce type compatibility at connection time:

| Source Type | Compatible Targets |
|-------------|-------------------|
| text | text, any |
| json | json, any |
| boolean | boolean, any |
| file | file, any |
| any | text, json, boolean, file, any |

Incompatible connections are rejected with a tooltip explaining the mismatch.

---

## Persistence

### Schema Changes

New table for workflows:

```sql
CREATE TABLE workflows (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT DEFAULT '',
    graph_json TEXT NOT NULL,     -- Serialized { nodes, edges, viewport }
    variables_json TEXT DEFAULT '[]',
    agent_id TEXT,                -- Optional: link to agent for config inheritance
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    is_archived INTEGER DEFAULT 0,
    FOREIGN KEY (agent_id) REFERENCES agents(id)
);
```

Migration: v5 (next after current v4 which added session branching fixes).

### Serialization Format

`graph_json` stores the React Flow graph directly:

```json
{
  "nodes": [
    {
      "id": "input_1",
      "type": "input",
      "position": { "x": 100, "y": 200 },
      "data": {
        "name": "pr_url",
        "dataType": "text",
        "default": "",
        "description": "GitHub PR URL to review"
      }
    },
    {
      "id": "llm_1",
      "type": "llm",
      "position": { "x": 400, "y": 200 },
      "data": {
        "provider": "anthropic",
        "model": "claude-sonnet-4-5",
        "systemPrompt": "You are a code reviewer...",
        "temperature": 0.3,
        "maxTokens": 4096,
        "routingMode": "fixed"
      }
    }
  ],
  "edges": [
    {
      "id": "e_input1_llm1",
      "source": "input_1",
      "sourceHandle": "output",
      "target": "llm_1",
      "targetHandle": "prompt"
    }
  ],
  "viewport": { "x": 0, "y": 0, "zoom": 1 }
}
```

This maps directly to React Flow's `toObject()` / `setNodes()` / `setEdges()` — zero translation layer.

### IPC Commands

```rust
// Workflow CRUD
#[tauri::command] fn list_workflows() -> Vec<WorkflowSummary>;
#[tauri::command] fn get_workflow(workflow_id: String) -> Workflow;
#[tauri::command] fn create_workflow(workflow: CreateWorkflowRequest) -> Workflow;
#[tauri::command] fn update_workflow(workflow_id: String, updates: UpdateWorkflowRequest) -> Workflow;
#[tauri::command] fn delete_workflow(workflow_id: String) -> ();
#[tauri::command] fn duplicate_workflow(workflow_id: String) -> Workflow;

// Execution
#[tauri::command] fn run_workflow(workflow_id: String, inputs: HashMap<String, String>) -> RunResult;
#[tauri::command] fn validate_workflow(workflow_id: String) -> ValidationResult;
// ValidationResult includes: DAG cycle check, required inputs, type compatibility,
// subworkflow cycle detection (A→B→A), disconnected node warnings
```

---

## New Event Types

```
workflow.started         { workflow_id, workflow_name, inputs }
workflow.node.started    { workflow_id, node_id, node_type, inputs }
workflow.node.completed  { workflow_id, node_id, outputs, duration_ms, cost_usd }
workflow.node.error      { workflow_id, node_id, error }
workflow.node.waiting    { workflow_id, node_id, message }     (approval gate)
workflow.node.skipped    { workflow_id, node_id, reason }      (branch not taken)
workflow.completed       { workflow_id, total_duration_ms, total_cost_usd, output }
workflow.failed          { workflow_id, failed_node_id, error }
```

These events nest within the existing event system. A workflow run creates a Session, and all events (including existing `llm.*`, `tool.*`) are recorded with their `seq` numbers. The Inspector can display workflow runs with full node-level granularity.

---

## Integration with Existing Features

### Inspector

The Inspector gains a **workflow view** when viewing a session that ran a workflow:

- Node graph rendered read-only with execution state overlaid
- Click any node to see its events in the timeline
- Cost breakdown per node (not just per LLM call)
- Critical path highlighted (longest chain of sequential nodes)

### Agents

An Agent can optionally have a workflow attached:

```typescript
interface Agent {
  // ... existing fields ...
  workflowId?: string;      // If set, sessions use this workflow instead of free-form chat
}
```

When an agent has a workflow:
- "New Session" shows a form for workflow inputs (from Input nodes) instead of a chat box
- The session runs the workflow, then shows results
- The user can still chat after workflow completion (hybrid mode)

### Runs

Headless runs with workflows become powerful batch processors:

```
Run: "Review 10 PRs"
  Agent: "Code Reviewer" (workflow: code-review-pipeline)
  Inputs: [pr_url_1, pr_url_2, ..., pr_url_10]
  → 10 parallel workflow executions
  → Results collected, costs aggregated
  → Inspector shows each run's node graph
```

### Hybrid Intelligence

LLM nodes with `routingMode: 'hybrid'` use the smart router:

- The router picks the best model based on the prompt content and budget
- Node displays which model was chosen and why
- Cost comparison shown: "Used Llama locally (free) — Claude would have cost $0.003"

---

## Templates

Bundled workflow templates for first-run experience:

| Template | Description | Nodes |
|----------|-------------|-------|
| **Code Review** | Analyze PR, classify issues, comment on GitHub | Input → LLM → Router → Tool (GitHub) |
| **Research Assistant** | Search web, summarize, compile report | Input → Tool (Brave Search) → LLM → Transform → Output |
| **Data Pipeline** | Read file, extract data, transform, output | Input → Tool (FS Read) → LLM (Extract) → Transform → Output |
| **Multi-Model Compare** | Same prompt to 3 models, compare outputs | Input → 3x LLM (parallel) → Transform (compare) → Output |
| **Safe Executor** | Run shell commands with human approval | Input → LLM (plan) → Approval → Tool (Shell) → Output |

Templates are stored as JSON workflow files and imported via the "New Workflow" dialog.

---

## Implementation Plan

### Phase 3A: Foundation (Build First)

| # | Task | Effort | Notes |
|---|------|--------|-------|
| 3A.1 | Install @xyflow/react, add to apps/ui | Low | `npm install @xyflow/react` |
| 3A.2 | SQLite schema v4: add workflows table | Low | Migration from v3 |
| 3A.3 | Workflow CRUD IPC commands in Rust | Medium | list, get, create, update, delete |
| 3A.4 | Zustand store: workflows slice | Low | Same pattern as agents slice |
| 3A.5 | NodeEditorPage: basic canvas with pan/zoom | Medium | React Flow canvas + minimap |
| 3A.6 | Node palette sidebar (drag to add) | Medium | All 8 node types |
| 3A.7 | Custom node components (LLM, Tool, Router, etc.) | High | React components with handles |
| 3A.8 | Node config panel (right sidebar) | Medium | Form for selected node |
| 3A.9 | Edge validation (type checking) | Low | Prevent incompatible connections |
| 3A.10 | Save/load workflow (serialize ↔ SQLite) | Medium | React Flow toObject() ↔ graph_json |
| 3A.11 | Undo/redo with snapshot history | Medium | Zustand-based undo stack |

**Deliverable**: Users can visually build workflows and save them. No execution yet.

### Phase 3B: Execution (Build Second)

| # | Task | Effort | Notes |
|---|------|--------|-------|
| 3B.1 | Workflow validation command | Medium | DAG check, required inputs, type compat |
| 3B.2 | Workflow execution engine in Rust | High | Topological sort, DAG walker |
| 3B.3 | LLM node execution (reuse send_message) | Medium | Wire to existing sidecar /chat |
| 3B.4 | Tool node execution (reuse tool pipeline) | Medium | Wire to existing tool approval flow |
| 3B.5 | Router node execution (LLM + pattern) | Medium | New classify endpoint or reuse /chat |
| 3B.6 | Approval node (UI prompt + wait) | Medium | Tauri event to UI, wait for response |
| 3B.7 | Transform node (template + JSONPath) | Low | Evaluate in Rust, no sidecar needed |
| 3B.8 | Workflow event types | Low | workflow.node.started/completed/error |
| 3B.9 | Live node state updates during execution | Medium | Events → UI → node visual states |
| 3B.10 | Data preview on completed nodes | Low | Show output snippet on node |
| 3B.11 | Run workflow button + input form | Medium | UI for triggering execution |

**Deliverable**: Workflows execute end-to-end. Users see live data flow through nodes.

### Phase 3C: Polish (Build Third)

| # | Task | Effort | Notes |
|---|------|--------|-------|
| 3C.1 | Workflow templates (5 bundled) | Medium | JSON files, import UI |
| 3C.2 | Inspector workflow view | Medium | Node graph overlay on session |
| 3C.3 | Agent ↔ workflow linking | Low | workflowId on agent config |
| 3C.4 | Subworkflow node | High | Recursive execution |
| 3C.5 | Copy/paste nodes | Low | Cmd+C/V with position offset |
| 3C.6 | Export/import workflow as JSON | Low | File save/load dialog |
| 3C.7 | Workflow-aware runs (batch + parallel) | Medium | Multiple inputs → multiple executions |
| 3C.8 | Cost estimation before execution | Medium | Sum estimated costs per LLM node |
| 3C.9 | Add "Node Editor" to sidebar navigation | Low | 6th module (or replace Runs tab) |
| 3C.10 | Keyboard shortcuts (all canvas controls) | Low | Register in command palette |

**Deliverable**: Full-featured node editor with templates, Inspector integration, and batch execution.

---

## UI Module Integration

The Node Editor becomes the 6th module in the sidebar:

```
Sidebar:
  ├─ Agents
  ├─ Sessions
  ├─ Runs
  ├─ Inspector
  ├─ Node Editor    ← NEW
  └─ Settings
```

Navigation shortcut: `Cmd+5` → Node Editor.

The Node Editor page has three zones:
1. **Left**: Node palette (collapsible)
2. **Center**: React Flow canvas
3. **Right**: Node config panel (shows when a node is selected)

Top bar shows: workflow name, save status, run button, validation status.

---

## Technology Choice: React Flow (@xyflow/react)

| Criteria | React Flow |
|----------|-----------|
| Stars | 35K+ |
| NPM downloads/wk | ~3M |
| License | MIT |
| TypeScript | Written in TS, full type exports |
| React 19 | Supported |
| Tailwind | Official templates use it |
| Custom nodes | Any React component |
| Serialization | Built-in toObject() → JSON |
| Performance | 60 FPS at 100+ nodes (DOM-based, memoized) |
| AI precedent | Langflow, Firecrawl Open Agent Builder |

Install: `npm install @xyflow/react`

---

## Open Questions

1. **Module placement**: Should Node Editor be the 6th module, or should it replace the Runs module (since workflow runs are a superset of headless runs)?
2. ~~**Execution location**~~: **RESOLVED** — DAG walker lives in Rust. Sidecar is stateless compute only (uses `/chat/direct`). Confirmed by Gemini 3 Pro review (2026-02-15).
3. ~~**Parallel execution**~~: **RESOLVED** — Yes, independent branches execute concurrently via `tokio::join_all`. Default concurrency limit: 4. Sidecar needs matching worker count.
4. **Versioning**: Should workflows have version history (like git for graphs)? Useful but adds complexity. Defer to post-launch.
5. **Hybrid mode**: After a workflow completes, should the user be able to continue chatting in the same session? Leaning yes — the workflow output becomes context for free-form conversation.

---

## Phase 4: Universal Automation Canvas (Vision)

Phase 3 built the AI workflow engine. Phase 4 expands the node type system to make AI Studio a universal automation platform. The architecture already supports this — each node type is an executor function in Rust + a React component. Adding new node types is the same pattern.

### Node Type Roadmap

The current 8 node types handle AI workflows. Phase 4 adds connector and processing nodes that handle everything else.

#### Data Source Nodes (Inputs)

| Node Type | What It Does | Example Use Case |
|-----------|-------------|-----------------|
| `database_read` | Execute SQL query against SQLite, PostgreSQL, MySQL | Read customer records → LLM summarizes → Output |
| `file_read` | Read CSV, JSON, XML, text, PDF from filesystem | Ingest CSV → LLM extracts insights → file_write results |
| `http_request` | GET/POST to any REST/GraphQL API | Fetch GitHub issues → LLM triages → Slack notify |
| `webhook_listen` | HTTP endpoint that triggers workflow on request | Receive Stripe webhook → LLM processes → database_write |
| `queue_consume` | Read from Kafka, RabbitMQ, Redis Streams, MQTT | Process message queue → LLM classifies → Router → different queues |
| `iot_sensor` | Read from IoT devices via MQTT, serial, GPIO | Temperature sensor → LLM anomaly detection → notification |
| `cron_trigger` | Time-based scheduled execution | Every hour: query DB → LLM generates report → email |
| `shell_input` | Run a shell command, capture stdout/stderr as data | `grep -r "ERROR" /var/log/` → LLM classifies errors → notification |
| `stdin_pipe` | Read from stdin or named pipe (streaming) | Tail a log file → LLM detects anomalies in real-time |

#### Data Destination Nodes (Outputs)

| Node Type | What It Does | Example Use Case |
|-----------|-------------|-----------------|
| `database_write` | INSERT/UPDATE/DELETE against any database | LLM extracts entities → database_write structured records |
| `file_write` | Write CSV, JSON, text, PDF, image to filesystem | LLM generates report → file_write as PDF |
| `http_post` | Send data to any REST/GraphQL/webhook endpoint | LLM response → POST to Slack/Discord/custom API |
| `queue_publish` | Push messages to Kafka, RabbitMQ, Redis, MQTT | Router classifies → queue_publish to appropriate topic |
| `iot_command` | Send commands to IoT devices | LLM decides action → iot_command turns on/off device |
| `notification` | Email, Slack, Discord, Teams, SMS, push notification | Any workflow → notification on completion/error |
| `display` | Rich visual output (charts, tables, formatted reports) | Data pipeline → display as chart in UI |
| `shell_exec` | Execute a shell command with configurable privileges | LLM generates migration script → shell_exec runs as `postgres` user |

#### Processing Nodes

| Node Type | What It Does | Example Use Case |
|-----------|-------------|-----------------|
| `code` | Execute Python/JavaScript snippets (sandboxed) | Custom data transformation that template can't handle |
| `validator` | JSON Schema validation, data quality checks | Validate LLM output matches expected schema before writing to DB |
| `merge` | Wait for N branches, combine results (AND/OR logic) | 3 parallel LLMs → merge when any 2 complete (quorum) |
| `loop` | Iterate over array, execute subgraph per item | File with 100 records → loop: LLM processes each → collect results |
| `cache` | Memoize expensive operations by input hash | Cache LLM responses for identical prompts (save cost) |
| `rate_limiter` | Throttle execution rate | API with 100 req/min limit → rate_limiter before http_post |
| `error_handler` | Catch errors from upstream, provide fallback | LLM fails → error_handler retries with different model |

### Shell Nodes: The Unix Philosophy Applied

The `shell_input` and `shell_exec` nodes deserve special attention because they turn every Unix tool into a node. The Unix philosophy — small tools that do one thing well, connected via pipes — maps directly to the node editor canvas.

**`shell_input`** — Any command's output becomes pipeline data:

```
┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐
│  shell   │    │   LLM    │    │  Router  │    │  Slack   │
│  _input  │───▶│  Claude  │───▶│ severity │───▶│  notify  │
│          │    │ (classify│    │          │    │ (#alerts)│
│ grep -r  │    │  errors) │    │          │    │          │
│ "ERROR"  │    └──────────┘    └──────────┘    └──────────┘
│ /var/log │                         │
└──────────┘                    ┌────▼─────┐
                                │  file    │
                                │  _write  │
                                │(report)  │
                                └──────────┘
```

Config: `command`, `working_dir`, `timeout`, `env_vars`, `shell` (bash/zsh/sh).
Outputs: `stdout` (text), `stderr` (text), `exit_code` (number).

**`shell_exec`** — LLM output becomes executable commands with privilege controls:

```
┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐
│  Input   │    │   LLM    │    │ Approval │    │  shell   │    │  Output  │
│  (task)  │───▶│  (plan   │───▶│  Gate    │───▶│  _exec   │───▶│ (result) │
│          │    │ command) │    │ (review) │    │          │    │          │
└──────────┘    └──────────┘    └──────────┘    │ run_as:  │    └──────────┘
                                                │  deploy  │
                                                │ sandbox: │
                                                │  true    │
                                                └──────────┘
```

Config fields for `shell_exec`:

| Field | Type | Description |
|-------|------|-------------|
| `run_as` | string? | Unix user to execute as (via `sudo -u`). Null = current user. |
| `sandbox` | boolean | Run in restricted sandbox (no network, limited fs) |
| `allowed_commands` | string[]? | Whitelist of allowed executables. Null = any. |
| `blocked_commands` | string[]? | Blacklist (e.g., `["rm -rf", "mkfs", "dd"]`) |
| `working_dir` | string? | Working directory for execution |
| `timeout` | number | Max execution time in seconds |
| `env_vars` | Record<string, string> | Environment variables to set |
| `capabilities` | string[]? | Linux capabilities to grant/restrict |

**Security model**: Shell exec nodes always default to `sandbox: true` and `approval: "ask"`. The approval gate shows the exact command that will run, the user ID, and the sandbox restrictions. This is the Safe Executor template pattern — but generalized to any command with fine-grained privilege control.

**Why this matters**: This makes AI Studio the glue between AI and existing infrastructure. Any cron job, deploy script, monitoring command, or DevOps task becomes a visual node with approval gates, cost tracking, and full audit trail. "Terraform plan → LLM reviews → approval gate → terraform apply" is a 4-node workflow.

### Example: Full-Stack Automation Pipeline

```
┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐
│ database │    │   LLM    │    │ validator│    │ database │    │  Slack   │
│  _read   │───▶│  Claude  │───▶│  JSON    │───▶│  _write  │───▶│  notify  │
│ (orders) │    │ (analyze)│    │  Schema  │    │ (results)│    │          │
└──────────┘    └──────────┘    └──────────┘    └──────────┘    └──────────┘
     │                                               │
     │           ┌──────────┐                        │
     └──────────▶│  file    │◀───────────────────────┘
                 │  _write  │
                 │  (CSV)   │
                 └──────────┘
```

This pipeline: reads orders from a database → LLM analyzes patterns → validates the output → writes structured results back to DB → notifies Slack → also exports as CSV. Every node shows its data, cost, and latency. The Inspector replays the entire flow.

### Example: DevOps with Shell Nodes

```
┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐
│  shell   │    │   LLM    │    │ Approval │    │  shell   │    │  Slack   │
│  _input  │───▶│  Claude  │───▶│  Gate    │───▶│  _exec   │───▶│  notify  │
│          │    │          │    │          │    │          │    │          │
│ terraform│    │ "Review  │    │ "Apply   │    │ terraform│    │ #deploys │
│ plan     │    │  this    │    │  these   │    │ apply    │    │          │
│ -no-color│    │  plan"   │    │  changes │    │ run_as:  │    │          │
│          │    │          │    │  ?"      │    │  deploy  │    │          │
└──────────┘    └──────────┘    └──────────┘    └──────────┘    └──────────┘
```

`terraform plan` output → LLM reviews for risks → human approves → `terraform apply` runs as `deploy` user → Slack notification. Full audit trail in Inspector.

### Architecture: How New Node Types Plug In

Each node type is a self-contained unit:

```
Node Type = {
    executor:   Rust async fn(inputs, config) -> Result<outputs>
    component:  React component (handles, config panel, data preview)
    schema:     JSON (input handles, output handles, config fields)
    category:   "data_source" | "data_dest" | "processing" | "ai" | "logic"
}
```

**Phase 4 implementation path:**
1. Define the `NodeTypePlugin` trait in Rust (executor interface)
2. Build ~6 high-value connector nodes as built-in (database_read/write, file_read/write, http_request/post)
3. Ship the plugin system so the community can build the rest
4. Each community node type is a crate (Rust executor) + npm package (React component)

**What stays the same for all node types:**
- Events: every node emits `workflow.node.started/completed/error`
- Inspector: full visibility into every node's I/O
- Cost tracking: nodes that cost money (LLM, API calls) report `cost_usd`
- Approval gates: can be placed before any node
- Validation: DAG cycle check, type compatibility, required inputs

### Why This Wins

| Capability | n8n | Node-RED | Make.com | AI Studio |
|---|---|---|---|---|
| Visual canvas | Yes | Yes | Yes | Yes |
| 500+ integrations | Yes | Yes | Yes | Plugin system (community) |
| AI/LLM as first-class | No (bolt-on) | No (bolt-on) | No (bolt-on) | **Yes — hybrid routing, cost tracking, multi-model** |
| Execution Inspector | No | No | No | **Yes — replay, branch, diff** |
| Human-in-the-loop | No | No | Limited | **Yes — approval gates anywhere** |
| Cost tracking per node | No | No | No | **Yes** |
| Local-first / offline | No | Partial | No | **Yes — all data on disk** |
| Open standard tools | No | No | No | **Yes — MCP** |

The insight: n8n/Node-RED have 500+ connectors but treat AI as an afterthought. ComfyUI/Langflow are AI-native but can't connect to databases or IoT devices. AI Studio is both: **universal connectivity + AI-native intelligence + full observability**.

---

## What Success Looks Like

### Phase 3 (NOW — AI Workflows)

A developer:
1. Opens Node Editor
2. Drags an Input node, an LLM node, a Tool node (GitHub), and an Output node
3. Connects them: Input → LLM → Tool → Output
4. Configures the LLM node with Claude Sonnet and a code review prompt
5. Clicks "Run", enters a PR URL
6. Watches data flow through each node in real-time
7. Sees the review posted as a GitHub comment
8. Opens Inspector → sees the full event log with costs per node
9. Shares the workflow JSON with a colleague who imports it in 2 clicks
10. Screenshots it and posts on X. AI Studio gets 500 new stars.

### Phase 4 (NEXT — Universal Automation)

A developer:
1. Drags a `database_read` node → connects to their PostgreSQL
2. Adds an LLM node that summarizes each row
3. Adds a `validator` node to check the LLM output matches their JSON schema
4. Adds a `database_write` node that writes results back
5. Adds a `notification` node for Slack alerts on completion
6. Adds an `error_handler` that retries with a different model on failure
7. Sets it on a `cron_trigger` to run every morning
8. Opens Inspector → sees the full pipeline with cost breakdown
9. Shares the workflow → teammate imports it, swaps the database connection, running in 60 seconds
10. Posts the screenshot. "This replaced our entire data pipeline." AI Studio gets 10,000 stars.
