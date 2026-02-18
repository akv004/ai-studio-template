# AI Studio — Phase 4: Universal Automation Canvas

> **Version**: 1.1
> **Status**: DRAFT — NEEDS EXTERNAL REVIEW BEFORE IMPLEMENTATION
> **Phase**: 4 (4A → 4B → 4C)
> **Depends on**: node-editor.md (Phase 3 — DONE), architecture.md, event-system.md, mcp-integration.md
> **Addresses**: Deferred review items from Antigravity (Gemini) node editor critique, ChatGPT 5.2 hybrid intelligence review
> **Review plan**: Gemini (architecture + EIP patterns) → Codex (implementation gaps) → iterate → code

---

## Executive Summary

Phase 3 built an AI workflow builder — 8 node types, DAG engine, templates, hybrid intelligence. It works, but it's an AI toy. Phase 4 graduates the Node Editor into a **universal automation canvas** where data I/O nodes (HTTP, file, shell) sit alongside AI nodes, with control flow (loop, merge, error handling) for production-quality pipelines.

**The gap**: Our current node editor (see `docs/design-references/node-editor/Screenshot_2025-12-23_13-17-43.png`) has bright card-like nodes with anonymous handles, no inline editing, no data connectors, and no control flow. Compare with Blender's Shader Editor (labeled sockets, dark professional theme, collapsible sections) and Unreal Blueprints (comment boxes, dense layouts, execution pins). We look like a prototype; they look like production tools.

**What Phase 4 delivers**:
- 10+ new node types: HTTP Request, File Read, File Write, Shell Exec, Validator, Subworkflow (executor), Merge, Loop, Error Handler, Code, Notification
- 3 new handle types: number, rows, binary
- Engine refactoring: `execute_subgraph()` extraction for loop support
- **Canvas UX**: Comment boxes, reroute nodes, collapsed graphs, container nodes for scopes
- UI refactoring: 1632-line monolith → modular component architecture
- 6 new templates for real-world automation pipelines
- Notification node for webhooks (Slack/Discord)
- Token estimation for pre-run cost estimates

**What this means competitively**: Node-RED meets ComfyUI with AI-native observability. No other tool combines universal data connectivity + AI-first intelligence + full execution replay + local-first desktop architecture.

---

## Design Principles

### 1. Unix Philosophy on Canvas

Every node does one thing well. Complex behavior emerges from composition. Shell nodes turn every Unix tool into a visual building block. The canvas IS the pipeline.

### 2. Blender-Grade Visual Quality

Reference: `docs/design-references/node-editor/blender_node.png`

- Dark node bodies that blend with the canvas (#1e1e1e on #0d0d0d)
- Thin colored header strips, not chunky blocks
- Labeled sockets with colored dots per data type
- Collapsible sections for complex nodes
- Smooth bezier noodles with type-based colors
- Professional, minimal, high signal-to-noise

### 3. Security by Default

New I/O nodes (shell, file, HTTP) interact with the system. Defaults are always safe:
- Shell Exec: approval = "ask", timeout = 30s
- File operations: scoped to Tauri FS scope by default
- HTTP: no auth headers stored in plain text in graph JSON

### 4. Additive Architecture

No schema migration needed. New node types are just new executor functions + React components. Graph JSON accommodates any node type naturally. Existing workflows continue to work unchanged.

---

## Architectural Influences

### Enterprise Integration Patterns (Apache Camel / MuleSoft)

Our node editor IS an integration platform. We should learn from 20+ years of EIP evolution rather than reinventing patterns. Key mappings:

| EIP Pattern | AI Studio Implementation | Status |
|-------------|-------------------------|--------|
| **Content-Based Router** | Router node (pattern + LLM modes) | BUILT |
| **Scatter-Gather** | Parallel branches + Merge node | Phase 4 (merge) |
| **Splitter** | Loop node (iterate over array) | Phase 4 (loop) |
| **Aggregator** | Merge node (all/any/quorum modes) | Phase 4 (merge) |
| **Message Translator** | Transform node (template/JSONPath/script) | BUILT |
| **Content Filter** | Transform node (JSONPath mode) | BUILT |
| **Dead Letter Channel** | Error Handler node (fallback value) | Phase 4 |
| **Retry** | Error Handler node (retry count + delay) | Phase 4 |
| **Wire Tap** | Built-in: every node emits events to Inspector | BUILT (automatic) |
| **Guaranteed Delivery** | SQLite WAL + event-sourced persistence | BUILT |
| **Validate** | Validator node (JSON Schema) | Phase 4 |
| **Process Manager** | DAG engine (topological sort walker) | BUILT |
| **Multicast** | Implicit: one node → multiple downstream edges | BUILT |

**Key insight from MuleSoft**: Scope containers. MuleSoft's Try scope, For-Each scope, and Scatter-Gather scope **visually wrap** their child nodes. Our Loop and Error Handler should do the same — not be standalone nodes, but visual containers that enclose their body.

**Key insight from Apache Camel Karavan**: Properties panel on the right, flow visualization at runtime, YAML under the hood. We already have this pattern (config panel, live node states, JSON serialization).

**Patterns intentionally deferred**:
- Circuit Breaker (add to Error Handler in Phase 5 — tracks failure rates)
- Saga/Compensation (complex rollback for external writes — Phase 5+)
- Routing Slip (runtime-determined routing — niche)
- Recipient List (dynamic fan-out — can compose from Router + multiple branches)

### Unreal Engine Blueprint Architecture

Unreal Blueprints is the gold standard for visual programming. Key architectural concepts and how they map to our system:

#### Execution Model: Dual-Wire System

Unreal has TWO wire types: **execution pins** (white triangles — control flow) and **data pins** (colored circles — data flow). Our system has only data edges, with execution order derived from DAG topology.

| Aspect | Unreal | AI Studio | Gap |
|--------|--------|-----------|-----|
| Execution ordering | Explicit white wires | Implicit via topology | Low gap for pipelines. Becomes limiting for imperative logic. |
| Data transfer | Colored typed wires | Typed edges | Equivalent |
| Pure computation | Pure functions (no exec pins) | Transform node | Equivalent |
| Side-effect nodes | Require exec pin connection | All nodes run when data arrives | Equivalent for DAGs |

**Decision**: We do NOT add execution pins in Phase 4. Our DAG model works well for automation pipelines. Execution pins add visual complexity without proportional value for our use cases. Re-evaluate in Phase 5 if users need imperative control flow.

#### Graph Organization Features We Need

| UE Feature | Description | Priority | Phase |
|------------|-------------|----------|-------|
| **Comment Boxes** | Colored rectangular regions grouping related nodes with titles | HIGH | 4A |
| **Collapsed Graphs** | Select nodes → collapse into single visual node. Expand on double-click. | MEDIUM | 4B |
| **Reroute Nodes** | Invisible routing points on wires to prevent spaghetti | LOW | 4C |
| **Graph Search** | Find nodes by name/type/content within the canvas | MEDIUM | 4B |
| **Bookmarks** | Save camera positions for quick navigation in large graphs | LOW | 4C |

#### Container/Scope Nodes (Critical Architecture Decision)

Both MuleSoft and Unreal use **visual containers** for scope-based constructs:
- MuleSoft: Try scope wraps error-handling body; For-Each scope wraps iteration body
- Unreal: Sequence node, Branch node contain sub-execution paths

**Phase 4 decision**: Loop and Error Handler are implemented as **container nodes** that visually enclose child nodes. This is more intuitive than standalone nodes with abstract "body_in/body_out" handles.

```
┌─── LOOP (items: [...]) ─────────────────────┐
│                                               │
│  ┌──────────┐    ┌──────────┐    ┌────────┐ │
│  │  Transform │──▶│   LLM    │──▶│Validate│ │
│  └──────────┘    └──────────┘    └────────┘ │
│                                               │
│  Iterations: 0/10  ▓▓▓▓▓░░░░░  50%          │
└──────────────── results [json] ──●───────────┘
```

**Implementation**: React Flow's `groupNode` feature. Loop/Error Handler nodes have `type: 'group'` styling with special rendering. Child nodes have `parentId` pointing to the container. The engine identifies loop body nodes by `parentId` rather than handle connections.

#### Abstraction Levels

| UE Concept | AI Studio Equivalent | Status |
|------------|---------------------|--------|
| Function (separate execution scope) | Subworkflow | BUILT |
| Macro (inline expansion, reusable) | **Snippet Library** (Phase 4C) | PLANNED |
| Collapsed Graph (visual compression) | Comment Box + Collapse | Phase 4B |
| Function Library (shared utils) | Template Gallery | BUILT |

### What We Do Better Than Everyone

| Capability | n8n/Node-RED/MuleSoft | Unreal Blueprints | ComfyUI/Langflow | **AI Studio** |
|------------|----------------------|-------------------|------------------|---------------|
| AI/LLM as first-class nodes | Bolt-on integration | N/A | Yes | **Yes — hybrid routing, cost tracking, multi-model** |
| Execution Inspector | No replay | No cost tracking | No event timeline | **Yes — replay, branch, diff, per-node cost** |
| Human-in-the-loop | Limited | N/A | No | **Yes — Approval gates anywhere** |
| Local-first / offline | No (cloud) | Yes (but game engine) | Yes | **Yes — all data on disk, SQLite** |
| MCP-native tools | No | No | No | **Yes — open standard** |
| Typed handle system | Schema-based | Pin types | Limited | **Yes — 8 types with coercion rules** |
| Plugin-provided nodes | Yes (community) | Yes (marketplace) | Yes (custom nodes) | **Phase 4C — same model** |

---

## Part 1: UI Architecture Refactoring

### Problem

`NodeEditorPage.tsx` is a 1632-line monolith containing:
- 8 custom node components (InputNode, OutputNode, LLMNode, etc.)
- Node palette with categories
- Node configuration panel with per-type forms
- Workflow list view with templates and import/export
- Workflow canvas with drag/drop, context menu, keyboard shortcuts
- Run modal, approval dialog, debug panel
- Execution state management
- Connection validation logic
- Default data definitions
- Node type registry

Adding 10+ new node types to this file is unsustainable. Before Phase 4 adds any code, the monolith must be split.

### Target Architecture

```
apps/ui/src/app/pages/
├── NodeEditorPage.tsx              ← Thin entry point (router between list/canvas)
└── workflow/
    ├── index.ts                    ← Re-exports
    ├── nodeTypes.ts                ← NODE_CATEGORIES, customNodeTypes registry, nodeColors
    ├── nodeDefaults.ts             ← defaultDataForType(), handle type definitions
    ├── connectionRules.ts          ← isValidConnection(), getHandleType(), type coercion rules
    ├── WorkflowList.tsx            ← Workflow list + template picker + import
    ├── WorkflowCanvas.tsx          ← React Flow canvas + toolbar + run/approval/debug
    ├── NodePalette.tsx             ← Left sidebar palette (draggable node types)
    ├── NodeConfigPanel.tsx         ← Right sidebar config form (per-type routing)
    ├── RunModal.tsx                ← Run input form modal
    ├── ApprovalDialog.tsx          ← Approval request dialog
    ├── ContextMenu.tsx             ← Right-click context menu
    └── nodes/                      ← One file per node component
        ├── NodeShell.tsx           ← Shared wrapper (header, collapse, exec badge)
        ├── ExecutionBadge.tsx      ← Status badge component
        ├── OutputPreview.tsx       ← Output preview component
        ├── InputNode.tsx
        ├── OutputNode.tsx
        ├── LLMNode.tsx
        ├── ToolNode.tsx
        ├── RouterNode.tsx
        ├── ApprovalNode.tsx
        ├── TransformNode.tsx
        ├── SubworkflowNode.tsx
        ├── HttpRequestNode.tsx     ← Phase 4A
        ├── FileReadNode.tsx        ← Phase 4A
        ├── FileWriteNode.tsx       ← Phase 4A
        ├── ShellExecNode.tsx       ← Phase 4A
        ├── ValidatorNode.tsx       ← Phase 4A
        ├── MergeNode.tsx           ← Phase 4B
        ├── LoopNode.tsx            ← Phase 4B
        ├── ErrorHandlerNode.tsx    ← Phase 4B
        ├── CodeNode.tsx            ← Phase 4B
        └── NotificationNode.tsx    ← Phase 4B
```

### Split Rules

1. **Zero behavior change** — pure refactoring, no new features
2. **Each file exports one named component** (no default exports)
3. **Shared state stays in Zustand store** — components import `useAppStore`
4. **Node registration in `nodeTypes.ts`** — add entry to `customNodeTypes` and `NODE_CATEGORIES`
5. **Config panel routing in `NodeConfigPanel.tsx`** — switch on `node.type`, delegate to per-type sections
6. **Constants extracted** — `nodeColors`, `PROVIDER_MODELS`, `execBadgeConfig` go to `nodeTypes.ts`/`nodeDefaults.ts`

---

## Part 2: New Handle Types

### Current Handle Types (Phase 3)

| Type | CSS Class | Color | Meaning |
|------|-----------|-------|---------|
| text | `handle-text` | `#E879F9` (magenta) | String data |
| json | `handle-json` | `#F59E0B` (orange) | Structured objects |
| bool | `handle-bool` | `#EF4444` (red) | True/false |
| float | `handle-float` | `#10B981` (green) | Decimal numbers |
| any | `handle-any` | `#9CA3AF` (gray) | Universal connector |

### New Handle Types (Phase 4)

| Type | CSS Class | Color | Meaning | Used By |
|------|-----------|-------|---------|---------|
| number | `handle-number` | `#00FF00` (bright green) | Integer values (exit codes, counts, status codes) | Shell Exec, HTTP Request, Merge, Loop |
| rows | `handle-rows` | `#00CED1` (dark turquoise) | Array of objects (tabular data) | File Read (CSV), future DB nodes |
| binary | `handle-binary` | `#4B0082` (indigo) | Raw binary data (files, images) | File Read (binary mode), future image nodes |

### Type Coercion Matrix

Extended `isValidConnection()` rules:

| Source → Target | Allowed? | Reason |
|-----------------|----------|--------|
| number → text | Yes | Implicit `.toString()` |
| number → float | Yes | Int to float is lossless |
| text → number | Yes | `parseInt()` — may fail at runtime |
| rows → json | Yes | Rows is a JSON array |
| json → rows | Yes | If JSON is an array of objects |
| binary → any | Yes | `any` accepts everything |
| float → number | No | Lossy (decimals truncated) |
| binary → text | No | Must use explicit transform |

### Implementation

- CSS: Add 3 new `.handle-*` classes in `index.css`
- `connectionRules.ts`: Update coercion matrix
- DOM class detection: `getHandleType()` already reads CSS classes — new types work automatically

---

## Part 2B: Canvas UX Features

### Comment Boxes (Phase 4A — HIGH priority)

Visual rectangular regions that group related nodes. From Unreal Blueprints.

```
┌─── Data Preprocessing ──────────────────────┐
│                                               │
│  ┌──────────┐    ┌──────────┐    ┌────────┐ │
│  │ File Read │──▶│ Transform │──▶│Validate│ │
│  └──────────┘    └──────────┘    └────────┘ │
│                                               │
└───────────────────────────────────────────────┘
```

**Implementation**:
- React Flow annotation node type: `type: 'annotation'` with custom rendering
- Config: title (string), color (hex — from a preset palette), description (optional)
- Move behavior: when a comment box moves, all enclosed nodes move with it
- Resize: drag edges to resize
- Create: right-click canvas → "Add Comment Box", or select nodes + keyboard shortcut (Ctrl+G)
- Z-order: always behind other nodes

**Stored in graph JSON** as a regular node with `type: 'annotation'`:
```json
{
  "id": "comment_1",
  "type": "annotation",
  "position": { "x": 50, "y": 50 },
  "data": { "title": "Data Preprocessing", "color": "#2d5a27", "description": "" },
  "style": { "width": 500, "height": 300 }
}
```

No executor needed — annotation nodes are skipped by the engine (already handled by validation filtering).

### Reroute Nodes (Phase 4C — LOW priority)

Invisible routing points on wires to prevent spaghetti in complex graphs. From Unreal Blueprints.

- Double-click an edge → inserts a reroute point
- Reroute points can be dragged to organize wire paths
- Implementation: React Flow custom edge type with intermediate waypoints
- Stored as edge metadata: `edge.data.reroutes: [{ x, y }]`

### Graph Search (Phase 4B — MEDIUM priority)

Find nodes by name, type, or content within the canvas.

- Keyboard shortcut: Ctrl+F
- Popup search bar at top of canvas
- Filters: node type, node name, config values
- Results: highlights matching nodes, navigates to first match
- Implementation: filter `nodes` array, use `setCenter()` to navigate

### Palette Search (Phase 4C — LOW priority)

Filter the node palette by typing. Already identified in the plan.

- Text input at top of palette
- Filters categories and node types as user types
- Shows matching nodes across all categories

---

## Part 3: New Node Types — Data I/O (Phase 4A)

### 3.1 HTTP Request Node

**Purpose**: Make HTTP requests to any REST/GraphQL API. Equivalent of `curl` as a visual node.

```
┌────────────────────────────────┐
│  HTTP REQUEST                   │
│                                 │
●──[text]     url                 │
●──[json]     body (opt)          │
●──[json]     headers (opt)       │
│                                 │
│  Method: GET ▼                  │
│  Timeout: 30s                   │
│  Auth: None ▼                   │
│                                 │
│              body     [text]──● │
│              status   [number]──● │
│              headers  [json]──● │
└────────────────────────────────┘
```

**Config Fields**:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| url | string | `""` | Request URL (can use template vars `{{input.url}}`) |
| method | enum | `"GET"` | `GET`, `POST`, `PUT`, `PATCH`, `DELETE`, `HEAD` |
| headers | json | `{}` | Static headers (merged with incoming edge) |
| body | string | `""` | Request body (for POST/PUT/PATCH) |
| timeout | number | `30` | Timeout in seconds |
| auth | enum | `"none"` | `none`, `bearer`, `basic`, `api_key` |
| authToken | string | `""` | Token/key value (for bearer/basic/api_key) |
| authHeader | string | `"Authorization"` | Header name for api_key auth |
| followRedirects | bool | `true` | Follow HTTP redirects |
| validateCerts | bool | `true` | Validate TLS certificates |

**Inputs**:
- `url` (text, optional) — overrides config URL if connected
- `body` (json, optional) — request body
- `headers` (json, optional) — merged with config headers (edge wins on conflict)

**Outputs**:
- `body` (text) — response body as text
- `status` (number) — HTTP status code (200, 404, 500, etc.)
- `headers` (json) — response headers as key-value object

**Executor** (`executors/http_request.rs`):
- Uses existing `reqwest` crate (already in Cargo.toml)
- Template-resolves URL and body before making request
- Merges headers: config headers → incoming edge headers (edge wins)
- Auth handling: prepends `Bearer`, `Basic base64(user:pass)`, or raw header
- Returns error string on timeout/connection failure (does NOT panic)
- Reports `cost_usd: 0.0` (HTTP calls are free from AI Studio's perspective)

**Security**:
- No auth tokens stored in graph JSON — use `authToken` config field which is runtime-only
- TODO (Phase 5): Credential vault integration for secure token storage
- Default approval: `"auto"` (HTTP requests don't need approval by default)

---

### 3.2 File Read Node

**Purpose**: Read files from the local filesystem. Supports text, JSON, CSV, and binary modes.

```
┌────────────────────────────────┐
│  FILE READ                      │
│                                 │
●──[text]     path (opt)          │
│                                 │
│  Path: /data/input.csv          │
│  Mode: csv ▼                    │
│  Encoding: utf-8                │
│                                 │
│              content  [text]──● │
│              rows     [rows]──● │
│              size     [number]──● │
└────────────────────────────────┘
```

**Config Fields**:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| path | string | `""` | File path (absolute or relative to working dir) |
| mode | enum | `"text"` | `text`, `json`, `csv`, `binary` |
| encoding | string | `"utf-8"` | Text encoding |
| csvDelimiter | string | `","` | CSV delimiter character |
| csvHasHeader | bool | `true` | First row is header |
| maxSize | number | `10` | Max file size in MB (safety limit) |

**Inputs**:
- `path` (text, optional) — overrides config path if connected

**Outputs** (varies by mode):
- **text mode**: `content` (text), `size` (number — bytes)
- **json mode**: `content` (json — parsed), `size` (number)
- **csv mode**: `rows` (rows — array of objects with header keys), `content` (text — raw CSV), `size` (number)
- **binary mode**: `content` (binary — base64 encoded), `size` (number)

**Executor** (`executors/file_read.rs`):
- Uses `std::fs::read` / `std::fs::read_to_string`
- CSV parsing: basic split implementation (no external crate dependency)
  - Split by newlines → split each line by delimiter → map to objects using header row
  - Handles quoted fields containing delimiters
- JSON parsing: `serde_json::from_str`
- Binary: read bytes, base64-encode as string
- Checks `maxSize` before reading (reject files over limit)
- Returns descriptive errors: "File not found: /path", "File too large: 25MB > 10MB limit"

**Security**:
- Scoped to Tauri FS scope by default (`scope.allow` in `tauri.conf.json`)
- Paths outside scope return a permission error
- No symlink following by default (prevents path traversal)
- Default approval: `"auto"` (reading is safe within scope)

---

### 3.3 File Write Node

**Purpose**: Write data to the local filesystem. Supports text, JSON, CSV modes.

```
┌────────────────────────────────┐
│  FILE WRITE                     │
│                                 │
●──[text]     path (opt)          │
●──[any]      content             │
│                                 │
│  Path: /data/output.json        │
│  Mode: json ▼                   │
│  Write: overwrite ▼             │
│                                 │
│              path     [text]──● │
│              bytes    [number]──● │
└────────────────────────────────┘
```

**Config Fields**:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| path | string | `""` | Output file path |
| mode | enum | `"text"` | `text`, `json`, `csv` |
| writeMode | enum | `"overwrite"` | `overwrite`, `append` |
| createDirs | bool | `true` | Create parent directories if missing |
| csvDelimiter | string | `","` | CSV delimiter |
| jsonPretty | bool | `true` | Pretty-print JSON output |

**Inputs**:
- `path` (text, optional) — overrides config path
- `content` (any, required) — data to write

**Outputs**:
- `path` (text) — the actual path written to (for chaining)
- `bytes` (number) — bytes written

**Executor** (`executors/file_write.rs`):
- Uses `std::fs::write` / `std::fs::OpenOptions` for append
- If `createDirs`: `std::fs::create_dir_all(parent)`
- JSON mode: `serde_json::to_string_pretty` or `to_string`
- CSV mode: if input is `rows` (array of objects), convert to CSV string with header row
- Text mode: write as-is (convert non-string to `.to_string()`)

**Security**:
- Scoped to Tauri FS scope
- Default approval: `"ask"` — writing files should be confirmed
- Cannot overwrite system files (FS scope prevents this)

---

### 3.4 Shell Exec Node

**Purpose**: Execute shell commands directly in Rust. The Unix philosophy on canvas — every CLI tool becomes a node.

```
┌────────────────────────────────┐
│  SHELL EXEC                     │
│                                 │
●──[text]     command (opt)       │
●──[text]     stdin (opt)         │
│                                 │
│  Command: echo "hello"          │
│  Working Dir: /home/user        │
│  Timeout: 30s                   │
│  Shell: bash ▼                  │
│                                 │
│              stdout   [text]──● │
│              stderr   [text]──● │
│              exit_code[number]──● │
└────────────────────────────────┘
```

**Config Fields**:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| command | string | `""` | Shell command to execute |
| workingDir | string | `""` | Working directory (empty = home dir) |
| timeout | number | `30` | Timeout in seconds |
| shell | enum | `"bash"` | `bash`, `sh`, `zsh` |
| envVars | json | `{}` | Additional environment variables |

**Inputs**:
- `command` (text, optional) — overrides config command (enables LLM → Shell pipelines)
- `stdin` (text, optional) — piped to process stdin

**Outputs**:
- `stdout` (text) — standard output
- `stderr` (text) — standard error
- `exit_code` (number) — process exit code (0 = success)

**Executor** (`executors/shell_exec.rs`):
- Uses `tokio::process::Command` — runs directly in Rust, NO sidecar roundtrip
- Command format: `shell -c "command"` (e.g., `bash -c "echo hello"`)
- Template-resolves command string before execution
- Stdin: if connected, writes to child process stdin
- Timeout: `tokio::time::timeout(Duration::from_secs(timeout), child.wait_with_output())`
- On timeout: kill process, return error "Command timed out after 30s"
- Output: captures stdout + stderr as UTF-8 strings
- Reports exit code as output handle value

**Security Model** (Critical):
- Default approval: `"ask"` — all shell commands require human confirmation
- The approval dialog shows: the exact command, working directory, timeout, and environment variables
- No `run_as` or `sudo` in v1 — too dangerous without proper privilege management
- No sandbox mode in v1 — deferred to Phase 5 (needs container/seccomp integration)
- Blocked commands: TODO for Phase 5 (allowlist/blocklist configuration)
- Environment variables: only those explicitly set in config — no inherited env leakage

**Why Rust, not sidecar**:
- Faster: no HTTP roundtrip to Python
- Simpler: `tokio::process` is well-tested, async-native
- More control: direct process management, proper signal handling
- Consistent: file and shell nodes both run in Rust layer

---

### 3.5 Validator Node

**Purpose**: Validate data against a JSON Schema. Ensures LLM outputs match expected structure before passing downstream.

```
┌────────────────────────────────┐
│  VALIDATOR                      │
│                                 │
●──[json]     data                │
│                                 │
│  Schema: {                      │
│    "type": "object",            │
│    "required": ["name"]         │
│  }                              │
│                                 │
│              valid    [bool]──● │
│              data     [json]──● │
│              errors   [json]──● │
└────────────────────────────────┘
```

**Config Fields**:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| schema | string | `"{}"` | JSON Schema (draft 7) as string |
| failOnError | bool | `false` | If true, node reports error status instead of outputting `valid: false` |

**Inputs**:
- `data` (json, required) — data to validate

**Outputs**:
- `valid` (bool) — whether validation passed
- `data` (json) — passthrough of input data (unchanged)
- `errors` (json) — array of validation error strings (empty if valid)

**Executor** (`executors/validator.rs`):
- New Cargo dependency: `jsonschema` crate
- Parses schema string as JSON Schema
- Validates input data against schema
- If `failOnError` and validation fails: return `Err(...)` (node goes to error state)
- Otherwise: return `{ valid, data, errors }` as output

**Why this matters**: LLM outputs are unpredictable. Validator + Router creates a retry loop:
```
LLM → Validator → Router(valid?) → [true: Output] / [false: LLM (retry)]
```

---

### 3.6 Subworkflow Executor

**Purpose**: Execute embedded workflows. The node type already exists in Phase 3 UI, but the Rust executor is a stub. Phase 4 implements it properly.

**Current state**: `SubworkflowNode.tsx` exists with `workflowId` and `workflowName` config. No executor in Rust — selecting a subworkflow node does nothing at runtime.

**Config Fields** (unchanged from Phase 3):

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| workflowId | string | `""` | ID of the workflow to embed |
| workflowName | string | `""` | Display name (cached for UI) |

**Inputs**:
- `input` (any, required) — passed as the primary input to the sub-workflow

**Outputs**:
- `output` (any) — the sub-workflow's Output node value

**Executor** (`executors/subworkflow.rs`):
- Load referenced workflow from DB by `workflowId`
- Check for circular references: `visited_workflows: HashSet<String>` in `ExecutionContext`
  - Before executing, check if `workflowId` is in visited set
  - If yes: return `Err("Circular subworkflow reference: A → B → A")`
  - If no: add to visited set, execute, remove after
- Create input map from incoming data: `{ "input": incoming_value }`
- Call `execute_workflow()` recursively with extended visited set
- Return the sub-workflow's output(s)

**ExecutionContext change**:
```rust
pub struct ExecutionContext<'a> {
    // ... existing fields ...
    pub visited_workflows: &'a HashSet<String>,  // NEW — circular ref detection
}
```

**Events**: Sub-workflow execution emits its own `workflow.node.*` events. The parent's `workflow.node.completed` for the subworkflow node includes total duration and cost of the sub-execution.

---

### 3.7 Notification Node (Phase 4B, but simple enough for 4A)

**Purpose**: Send notifications via webhooks. Supports Slack, Discord, and generic webhooks.

```
┌────────────────────────────────┐
│  NOTIFICATION                   │
│                                 │
●──[text]     message             │
●──[json]     data (opt)          │
│                                 │
│  Channel: slack ▼               │
│  Webhook: https://hooks...      │
│  Template: {{message}}          │
│                                 │
│              success  [bool]──● │
│              status   [number]──● │
└────────────────────────────────┘
```

**Config Fields**:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| channel | enum | `"webhook"` | `slack`, `discord`, `webhook` (generic) |
| webhookUrl | string | `""` | Webhook URL |
| messageTemplate | string | `"{{message}}"` | Message template |
| username | string | `"AI Studio"` | Bot username (Slack/Discord) |

**Inputs**:
- `message` (text, required) — notification message
- `data` (json, optional) — additional data for template resolution

**Outputs**:
- `success` (bool) — whether the notification was sent
- `status` (number) — HTTP status code from webhook

**Executor** (`executors/notification.rs`):
- Uses `reqwest` to POST to webhook URL
- Slack format: `{ "text": resolved_message, "username": username }`
- Discord format: `{ "content": resolved_message, "username": username }`
- Generic webhook: `{ "message": resolved_message, "data": data }`
- Returns `{ success: status.is_success(), status: status_code }`

---

## Part 4: New Node Types — Control Flow (Phase 4B)

### 4.1 Merge Node

**Purpose**: Collect results from multiple upstream branches into a single output. Essential for parallel workflows.

```
┌────────────────────────────────┐
│  MERGE                          │
│                                 │
●──[any]      input_1             │
●──[any]      input_2             │
●──[any]      input_3             │
│  [+ Add Input]                  │
│                                 │
│  Mode: all ▼                    │
│                                 │
│              results  [json]──● │
│              count    [number]──● │
└────────────────────────────────┘
```

**Config Fields**:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| mode | enum | `"all"` | `all` (wait for all), `any` (first non-null), `quorum` (first N) |
| quorum | number | `2` | Required count for quorum mode |
| inputs | string[] | `["input_1", "input_2"]` | Dynamic input handle names |

**Inputs**: Dynamic — user names them (like Transform node pattern).

**Outputs**:
- `results` (json) — array of all collected values `[{ input: "input_1", value: ... }, ...]`
- `count` (number) — number of non-null inputs received

**Executor** (`executors/merge.rs`):
- Collects all incoming edge values into a results array
- Mode logic:
  - `all`: returns all inputs (current behavior — topo sort guarantees all predecessors complete first)
  - `any`: returns only the first non-null value
  - `quorum`: returns when `quorum` inputs are non-null (in v1, topo sort means all are available; quorum filtering is post-hoc)
- Output is always a JSON array of `{ input, value }` objects

**Note**: In the current sequential engine, merge is straightforward because topo sort ensures all predecessors finish before the merge node executes. True parallel-first-arrival semantics require engine changes deferred to Phase 5.

---

### 4.2 Loop Node (Container/Scope Pattern)

**Purpose**: Iterate over an array, executing a subgraph for each element. The most requested Phase 4 feature.

**Architecture**: Following MuleSoft's For-Each scope pattern, the Loop node is a **visual container** that encloses its body nodes, not a standalone node with abstract handles.

```
┌─── LOOP (items: [...]) ───────────────────────────────┐
│                                                         │
│  ●──[json] items                                       │
│                                                         │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐         │
│  │ Transform │──▶│   LLM    │──▶│ Validator │         │
│  │ (format)  │   │ (process)│    │ (check)   │         │
│  └──────────┘    └──────────┘    └──────────┘         │
│                                                         │
│  Max: 100  Parallel: false  Collect: array             │
│  Progress: ▓▓▓▓▓░░░░░  5/10                           │
│                                                         │
│                               results  [json]──●       │
│                               count    [number]──●     │
└─────────────────────────────────────────────────────────┘
```

**Config Fields**:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| maxIterations | number | `100` | Safety limit |
| parallel | bool | `false` | Execute iterations concurrently (v1: false only) |
| collectMode | enum | `"array"` | `array` (collect all), `last` (keep last), `concat` (flatten) |

**Inputs**:
- `items` (json, required) — array to iterate over

**Outputs**:
- `results` (json) — collected results from all iterations
- `count` (number) — number of iterations executed

**Container Model** (replaces body_in/body_out handles):
- Loop is a React Flow **group node** (`type: 'group'` behavior)
- Child nodes have `parentId` pointing to the loop node ID
- The engine identifies loop body nodes via `parentId`, NOT handle connections
- First node in body receives current array element as input
- Last node in body (no outgoing edges within the group) provides iteration result
- Visual: dark border with title bar showing config + progress bar during execution

**Engine Refactoring Required**:

The loop node requires extracting `execute_subgraph()` from the current `execute_workflow()`:

```rust
// Current: execute_workflow() runs all nodes in topo order
// New: extract a function that runs a subset of nodes

async fn execute_subgraph(
    ctx: &ExecutionContext<'_>,
    nodes: &[WorkflowNode],      // Only body nodes
    edges: &[WorkflowEdge],       // Only edges between body nodes
    initial_inputs: &HashMap<String, serde_json::Value>,
) -> Result<serde_json::Value, String>;
```

**Loop Execution Flow**:
1. Parse `items` input as JSON array
2. Identify loop body nodes (all nodes with `parentId == loop_node_id`)
3. Identify body edges (edges between body nodes only)
4. For each element in array (up to `maxIterations`):
   a. Create fresh input map with current element
   b. Call `execute_subgraph()` with body nodes + edges
   c. Emit progress event with iteration index
   d. Collect last node's output as iteration result
5. Assemble `results` based on `collectMode`
6. Emit `workflow.node.completed` with iteration count and total duration

**Safety**:
- `maxIterations` prevents runaway loops
- Each iteration emits its own set of node events (with iteration index in payload)
- Total cost is sum of all iteration costs
- Progress bar on the container shows `current/total` iterations

**v1 Limitations** (documented, not hidden):
- No nested loops (would require recursive subgraph extraction — Phase 5)
- No early exit / break condition (Phase 5: add `breakCondition` config)
- No parallel iteration in v1 (Phase 5: `tokio::join_all` with concurrency limit)
- Container must have at least one child node (validated at run time)

---

### 4.3 Error Handler Node (Container/Scope Pattern)

**Purpose**: Catch errors from enclosed nodes, with retry logic and fallback. Maps to EIP Dead Letter Channel + Retry patterns.

**Architecture**: Following MuleSoft's Try scope pattern, the Error Handler is a **visual container** that wraps nodes it protects. If any enclosed node fails, the error handler catches it.

```
┌─── ERROR HANDLER ─────────────────────────────────────┐
│                                                         │
│  ●──[any] input                                        │
│                                                         │
│  ┌──────────┐    ┌──────────┐                          │
│  │   LLM    │──▶│  HTTP    │   ← protected nodes      │
│  │ (analyze)│    │ (post)   │                          │
│  └──────────┘    └──────────┘                          │
│                                                         │
│  Retries: 2  Delay: 1000ms                             │
│  Fallback: "Service unavailable"                        │
│                                                         │
│                               data     [any]──●        │
│                               had_error[bool]──●       │
│                               error_msg[text]──●       │
└─────────────────────────────────────────────────────────┘
```

**Config Fields**:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| retryCount | number | `0` | Number of retries on error |
| retryDelayMs | number | `1000` | Delay between retries in milliseconds |
| fallbackValue | string | `""` | Value to use if all retries fail |

**Inputs**:
- `input` (any, required) — data to pass to first enclosed node

**Outputs**:
- `data` (any) — last enclosed node output (success) or fallback value (failure)
- `had_error` (bool) — whether an error occurred
- `error_msg` (text) — error message (empty if no error)

**Container Model** (same pattern as Loop):
- Error Handler is a React Flow group node
- Child nodes have `parentId` pointing to the error handler node ID
- Engine identifies protected nodes via `parentId`
- Execution: runs enclosed subgraph via `execute_subgraph()`
- On error: retry the entire subgraph (not individual nodes) `retryCount` times
- On all retries exhausted: output fallback value, set `had_error: true`
- On success: output last node's result, set `had_error: false`

**Engine Integration**:
- Container nodes (loop, error_handler) are executed by the engine INSTEAD of their children
- The engine skips child nodes in the main topo sort (they're handled by the container executor)
- This is a fundamental change: `execute_workflow()` must filter out child nodes and let container executors handle them via `execute_subgraph()`

**Events**:
- Error handler emits `workflow.node.started` once
- Each retry emits events for the enclosed nodes (with retry index in payload)
- Final `workflow.node.completed` includes `had_error`, `retry_count`, `error_msg`

**Future: Circuit Breaker** (Phase 5):
- Track failure rate per enclosed node type across workflow runs
- States: closed (normal), open (rejecting — too many failures), half-open (testing)
- Open state: skip execution, return fallback immediately
- Config: `failureThreshold` (count), `resetTimeout` (ms)
- Stored in settings: `circuit_breaker.{node_type}.state`

---

### 4.4 Code Node

**Purpose**: Execute arbitrary Python or JavaScript code. For transformations too complex for templates.

```
┌────────────────────────────────┐
│  CODE                           │
│                                 │
●──[json]     data                │
│                                 │
│  Language: python ▼             │
│  ┌──────────────────────────┐  │
│  │ import json              │  │
│  │ data = json.loads(input) │  │
│  │ result = {               │  │
│  │   "count": len(data),    │  │
│  │   "sum": sum(data)       │  │
│  │ }                        │  │
│  │ print(json.dumps(result))│  │
│  └──────────────────────────┘  │
│  Timeout: 10s                   │
│                                 │
│              result   [json]──● │
│              stdout   [text]──● │
└────────────────────────────────┘
```

**Config Fields**:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| language | enum | `"python"` | `python`, `javascript` |
| code | string | `""` | Code to execute |
| timeout | number | `10` | Timeout in seconds |

**Inputs**:
- `data` (json, required) — passed as `input` variable in the code context

**Outputs**:
- `result` (json) — parsed from stdout (last line must be valid JSON) or from `result` variable
- `stdout` (text) — full stdout output

**Execution Architecture**:

New sidecar endpoint: `POST /code/execute`

```python
# Sidecar endpoint
@router.post("/code/execute")
async def execute_code(request: CodeExecuteRequest):
    # request: { language, code, input_data, timeout }
    # Runs code in subprocess with:
    #   - Python: python3 -c "..." with input as env var
    #   - JavaScript: node -e "..." with input as env var
    # Returns: { stdout, stderr, exit_code, result }
```

**Executor** (`executors/code.rs`):
- Calls sidecar `/code/execute` endpoint (code runs in sidecar, not Rust)
- Input data passed as JSON string in environment variable `AI_STUDIO_INPUT`
- Code can read: `import json; data = json.loads(os.environ['AI_STUDIO_INPUT'])`
- Result: parsed from stdout if valid JSON, otherwise raw stdout as text

**Sidecar implementation**:
- Uses `subprocess.run()` with timeout
- Captures stdout + stderr
- Tries to parse stdout as JSON for `result` output
- Falls back to raw string

**Security**:
- Default approval: `"ask"` — code execution requires confirmation
- Subprocess has no network access (TODO Phase 5: network sandbox)
- Subprocess inherits minimal env (HOME, PATH, AI_STUDIO_INPUT only)
- Timeout kills process

---

### 4.5 Token Estimation (Phase 4B polish)

**Purpose**: Pre-run cost estimate displayed in the Run dialog.

**How it works**:
- Count LLM nodes in the workflow
- For each LLM node: estimate tokens from system prompt length + average input size
- Use the pricing table in `routing.rs` (`MODEL_CAPABILITIES`) to calculate cost per node
- Sum all node costs → display in Run dialog: "Estimated cost: ~$0.05 (3 LLM calls)"

**Implementation**:
- New Rust command: `estimate_workflow_cost(workflow_id: String) -> CostEstimate`
- `CostEstimate { total_usd: f64, node_estimates: Vec<NodeEstimate>, llm_node_count: usize }`
- UI: call before showing Run modal, display in modal header

---

## Part 5: Templates

### New Templates (Phase 4A)

#### API Scraper
**Nodes**: Input(url) → HTTP Request(GET) → LLM(extract data) → Validator(JSON Schema) → Output
**Use case**: Fetch a web page, use LLM to extract structured data, validate output.

#### DevOps Pipeline
**Nodes**: Input(command) → Shell Exec(plan) → LLM(review) → Approval → Shell Exec(apply) → Notification(Slack)
**Use case**: Run a planning command, have AI review it, human approves, execute.

#### Data Validator
**Nodes**: File Read(CSV) → Transform(per-row) → Validator(schema) → Router(valid?) → [File Write(valid.csv) / File Write(errors.csv)]
**Use case**: Read a data file, validate each record, split into valid and invalid.

### New Templates (Phase 4B)

#### Batch Processor
**Nodes**: File Read(CSV) → Loop(per-row) → [LLM(process) → Validator] → Merge → File Write(results.json)
**Use case**: Process each row of a CSV through an LLM pipeline.

#### Resilient API Client
**Nodes**: Input(url) → Error Handler → HTTP Request → Validator → Output
**Use case**: Call an API with automatic retry and fallback on failure.

#### Multi-Step Code Pipeline
**Nodes**: Input(data) → Code(Python: preprocess) → LLM(analyze) → Code(Python: postprocess) → Output
**Use case**: Custom pre/post processing around LLM analysis.

---

## Part 6: Rust Executor Registration

### Current Executor Registry

```rust
// workflow/executors/mod.rs
pub fn create_executor_registry() -> HashMap<String, Box<dyn NodeExecutor>> {
    let mut registry = HashMap::new();
    registry.insert("input".into(), Box::new(InputExecutor));
    registry.insert("output".into(), Box::new(OutputExecutor));
    registry.insert("llm".into(), Box::new(LlmExecutor));
    registry.insert("tool".into(), Box::new(ToolExecutor));
    registry.insert("router".into(), Box::new(RouterExecutor));
    registry.insert("approval".into(), Box::new(ApprovalExecutor));
    registry.insert("transform".into(), Box::new(TransformExecutor));
    registry
}
```

### Phase 4 Additions

```rust
// Phase 4A
registry.insert("subworkflow".into(), Box::new(SubworkflowExecutor));
registry.insert("http_request".into(), Box::new(HttpRequestExecutor));
registry.insert("file_read".into(), Box::new(FileReadExecutor));
registry.insert("file_write".into(), Box::new(FileWriteExecutor));
registry.insert("shell_exec".into(), Box::new(ShellExecExecutor));
registry.insert("validator".into(), Box::new(ValidatorExecutor));
registry.insert("notification".into(), Box::new(NotificationExecutor));

// Phase 4B
registry.insert("merge".into(), Box::new(MergeExecutor));
registry.insert("loop".into(), Box::new(LoopExecutor));
registry.insert("error_handler".into(), Box::new(ErrorHandlerExecutor));
registry.insert("code".into(), Box::new(CodeExecutor));
```

### New Cargo Dependencies

```toml
jsonschema = "0.28"  # For Validator node — JSON Schema draft 7 validation
```

All other functionality uses existing deps (`reqwest`, `tokio`, `serde_json`, `std::fs`).

---

## Part 7: Event Extensions

### New Events (All Node Types)

All new node types emit the standard workflow node events:

```
workflow.node.started    { workflow_id, node_id, node_type, inputs }
workflow.node.completed  { workflow_id, node_id, outputs, duration_ms, cost_usd }
workflow.node.error      { workflow_id, node_id, error }
```

### Node-Specific Event Extensions

| Node Type | Extra Fields in `completed` payload |
|-----------|-------------------------------------|
| http_request | `status_code`, `response_size_bytes` |
| file_read | `file_path`, `file_size_bytes`, `mode` |
| file_write | `file_path`, `bytes_written` |
| shell_exec | `exit_code`, `command` (truncated) |
| validator | `valid`, `error_count` |
| subworkflow | `sub_workflow_id`, `sub_session_id`, `sub_node_count` |
| notification | `channel`, `webhook_status` |
| merge | `input_count`, `mode` |
| loop | `iteration_count`, `total_duration_ms` |
| error_handler | `had_error`, `retry_count` |
| code | `language`, `exit_code` |

These extra fields appear in the Inspector for richer debugging.

---

## Part 8: Plugin-Provided Nodes (Phase 4C — Vision)

### Goal

Community developers can create custom node types via plugins. This is how database connectors, IoT nodes, and niche integrations get built without bloating the core.

### Plugin Node Registration

Extend `plugin.json` manifest:

```json
{
  "id": "postgres-connector",
  "name": "PostgreSQL Connector",
  "version": "1.0.0",
  "provides": {
    "node_types": [
      {
        "id": "postgres_read",
        "name": "PostgreSQL Read",
        "category": "Data I/O",
        "icon": "database",
        "inputs": [
          { "id": "query", "label": "SQL Query", "type": "text", "required": true }
        ],
        "outputs": [
          { "id": "rows", "label": "Results", "type": "rows" },
          { "id": "count", "label": "Row Count", "type": "number" }
        ],
        "config": [
          { "id": "connectionString", "label": "Connection String", "type": "text" },
          { "id": "timeout", "label": "Timeout (s)", "type": "number", "default": 30 }
        ]
      }
    ]
  }
}
```

### Plugin Node Execution

- Plugin registers MCP tool: `node_execute__postgres_read`
- When the engine encounters a plugin node type, it calls the plugin's MCP tool
- Input data passed as tool arguments, config as additional params
- Tool returns result JSON that maps to output handles

### Plugin Node UI

- Generic `PluginNode.tsx` component renders handles + config from schema
- No custom React components from plugins in v1 — schema-driven rendering only
- Plugin icon rendered from a built-in icon set (lucide icons by name)

---

## Build Order

```
Phase 4A (4-5 sessions):
  Session 1: UI refactoring (4A.1) + handle types (4A.2) + Comment Boxes
  Session 2: Subworkflow executor (4A.3) + HTTP Request (4A.4)
  Session 3: File Read/Write (4A.5) + Shell Exec (4A.6)
  Session 4: Validator (4A.7) + Notification + palette + templates (4A.8)

Phase 4B (4-5 sessions):
  Session 5: Engine refactoring — extract execute_subgraph() + container node support
  Session 6: Merge (4B.1) + Graph Search
  Session 7: Loop container (4B.2) — the hardest task
  Session 8: Error Handler container (4B.3) + Code node + sidecar endpoint (4B.4)
  Session 9: Token estimation (4B.6) + collapsed graphs

Phase 4C (3 sessions):
  Session 10: Plugin node type registration (4C.1)
  Session 11: Tool schema handles (4C.2) + skipped edge styling (4C.3)
  Session 12: Reroute nodes + palette search + template thumbnails (4C.4-5)
```

**Critical path**: 4A.1 (UI refactor) → all 4A nodes. 4B engine refactoring → Loop + Error Handler containers.

**NEW: Engine refactoring is now a dedicated session** (Session 5) because container nodes require fundamental changes to how `execute_workflow()` works. This must be done carefully before Loop/Error Handler can be built.

---

## Verification Criteria

After each sub-phase:

1. **`cargo test`** — all existing 31+ tests pass + new tests for each executor
2. **`npm run build`** — TypeScript compiles clean with zero errors
3. **Manual test per new node type**:
   - Create workflow with new node
   - Configure all config fields
   - Connect to other nodes (verify type validation)
   - Run end-to-end
   - Verify Inspector shows events with node-specific extra fields
4. **Template test**: load each new template, execute, verify output
5. **Regression**: existing Phase 3 workflows continue to work unchanged

---

## What's Deferred (Phase 5 or Community Plugins)

| Item | Reason |
|------|--------|
| `database_read/write` | Needs driver deps (postgres, mysql, sqlite) — community plugin |
| `webhook_listen` | Needs HTTP server in Rust — complex lifecycle |
| `queue_consume/publish` | Needs Kafka/RabbitMQ drivers — community plugin |
| `iot_sensor/command` | Niche — community plugin |
| `cron_trigger` | Needs scheduler daemon — Phase 5 |
| `stdin_pipe` | Niche streaming — Phase 5 |
| `display` | Needs chart library — Phase 5 |
| `cache` | Nice-to-have — Phase 5 |
| `rate_limiter` | Nice-to-have — Phase 5 |
| Nested loops | Requires recursive subgraph extraction |
| Loop `break` condition | Adds complexity |
| Parallel loop iteration | Needs concurrency control |
| Shell sandbox mode | Needs container/seccomp integration |
| Credential vault | Secure token storage for HTTP auth |
| Plugin custom UI components | V1 uses schema-driven rendering only |
| Skipped edge styling | CSS for inactive router branches — Phase 4C |
| Template thumbnails | SVG miniatures — Phase 4C |
| Tool schema-driven handles | Auto-generate from MCP schema — Phase 4C |

---

## Open Questions

1. **Notification node in 4A or 4B?** — Spec places it as 4A.7+ since it's simple (`reqwest` POST). The plan had it in 4B. Recommend 4A since it's just an HTTP POST wrapper.
2. **Code node security**: Should code execution require Tauri permission scope? Currently inherits sidecar permissions.
3. ~~**Loop body identification**~~: RESOLVED — Container/scope pattern. Child nodes have `parentId` pointing to the loop node. Engine identifies body nodes via `parentId`.
4. ~~**Error handler placement**~~: RESOLVED — Container/scope pattern. Error handler wraps enclosed nodes, not a specific upstream.
5. **Container node nesting**: Can a loop contain an error handler? Can an error handler contain a loop? Proposal: yes, with a max nesting depth of 3.
6. **Container node UI complexity**: React Flow group nodes have limitations (drag behavior, resize, z-ordering). Need prototype to validate UX before committing to container model.
7. **Comment box + container overlap**: Both use group-like visuals. How to visually distinguish annotation groups (no execution meaning) from container nodes (execution scope)? Proposal: containers have thick colored borders + header bar; comment boxes have dashed thin borders + background tint.
8. **Snippet Library scope**: Should snippets be per-workflow or global? Proposal: global — stored in `~/.ai-studio/snippets/` as JSON files.

---

## Review Plan

This spec MUST be externally reviewed before implementation. The review cycle:

### Round 1: Architecture Review (Gemini 3 Pro via Antigravity)

**Focus**: EIP pattern mapping correctness, container/scope architecture, engine refactoring risks, handle type coercion completeness.

**Key questions for reviewer**:
- Is the container/scope model (parentId-based) the right approach for Loop and Error Handler? Or should we use the body_in/body_out handle approach?
- Are we missing critical EIP patterns that should be built-in (not community plugins)?
- Is `execute_subgraph()` extraction from `execute_workflow()` a clean refactoring, or does it risk breaking the existing engine?
- Does the handle type coercion matrix cover all reasonable conversions?

### Round 2: Implementation Review (Codex / GPT via VS Code)

**Focus**: Rust executor implementation details, edge cases, type safety, security model for shell/file/HTTP nodes.

**Key questions for reviewer**:
- Shell Exec security: is the default approval model sufficient? What about command injection via template variables?
- File Read/Write: is the Tauri FS scope sufficient for security, or do we need additional path validation?
- Loop engine: does the `parentId`-based body node identification handle edge cases (nodes connected both inside and outside the loop)?
- CSV parsing without external crate: is the basic split approach reliable enough, or should we add the `csv` crate?

### Round 3: UX Review (Either)

**Focus**: Canvas UX features (comment boxes, containers, reroute nodes), palette organization, template quality.

After all reviews are triaged and addressed, proceed to implementation.
