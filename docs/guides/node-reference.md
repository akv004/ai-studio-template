# Node Reference

AI Studio ships 20 node types organized into 6 categories. Each node can be placed on the canvas by clicking it in the Node Palette (left sidebar) and then clicking on the canvas, or by dragging it directly.

All nodes support custom labels — double-click the header to rename.

---

## Triggers

### Webhook

**Type ID:** `webhook_trigger`

Exposes an HTTP endpoint that starts a workflow run when a request arrives. Source-only node — no input handles.

**Output Handles**

| Handle | Type | Description |
|--------|------|-------------|
| body | json | Parsed request body |
| headers | json | Request headers as key-value pairs |
| query | json | URL query parameters |
| method | text | HTTP method (GET, POST, etc.) |

**Configuration**

| Field | Default | Description |
|-------|---------|-------------|
| Path | *(empty)* | URL path segment, e.g. `/my-hook` → `http://localhost:9876/hook/my-hook` |
| Methods | POST | Which HTTP methods to accept (POST, GET, PUT, DELETE) |
| Auth Mode | None | `None`, `Bearer Token`, or `HMAC-SHA256` |
| Auth Token | — | Bearer token value (shown when Auth Mode = Bearer Token) |
| HMAC Secret | — | Signing secret (shown when Auth Mode = HMAC-SHA256) |
| Response Mode | Immediate | `Immediate` returns 202 right away; `Wait` blocks until workflow completes |
| Timeout | 30s | Max wait time (shown when Response Mode = Wait) |
| Rate Limit | 60 req/min | Maximum requests per minute |

**Toolbar**

When a Webhook node is on the canvas, the toolbar shows Arm / Disarm / Test buttons:

- **Arm** — Saves the workflow, creates/updates the trigger record, and starts the webhook server. The node shows a green status dot when armed.
- **Disarm** — Stops listening for requests. Status dot turns gray.
- **Test** — Fires a mock request to trigger one execution (must be armed first).

**Example**

Connect `body` → LLM `prompt` to build a chatbot API, or connect `body` → Transform to extract fields from incoming webhooks.

---

## Inputs / Outputs

### Input

**Type ID:** `input`

Workflow entry point. Defines a named variable that can be set when the workflow runs.

**Output Handles**

| Handle | Type | Description |
|--------|------|-------------|
| value | text | The input value |

**Configuration**

| Field | Default | Description |
|-------|---------|-------------|
| Name | input | Variable name used in the Run dialog |
| Data Type | Text | Text, JSON, Boolean, or File |
| Default Value | *(empty)* | Pre-filled value |

**Tip:** Each Input node becomes a field in the Run Workflow dialog. Use meaningful names like `question` or `document`.

---

### Output

**Type ID:** `output`

Workflow exit point. Collects and displays the final result.

**Input Handles**

| Handle | Type | Description |
|--------|------|-------------|
| value | text | The result to output |

**Configuration**

| Field | Default | Description |
|-------|---------|-------------|
| Name | result | Output variable name |
| Format | Text | Text, Markdown, or JSON — controls how the result is rendered |

---

## AI

### LLM

**Type ID:** `llm`

Calls a language model. Supports streaming, multi-turn sessions, and vision.

**Input Handles**

| Handle | Type | Description |
|--------|------|-------------|
| system (opt) | text | System prompt override |
| context (opt) | json | Additional context (RAG results, previous outputs) |
| prompt | text | The user message to send |

**Output Handles**

| Handle | Type | Description |
|--------|------|-------------|
| response | text | Model's text response |
| usage | json | Token usage breakdown |
| cost | float | Estimated cost in USD |

**Configuration**

| Field | Default | Description |
|-------|---------|-------------|
| Provider | — | Anthropic, Google, Azure OpenAI, Ollama, Local |
| Model | — | Model name (filtered by provider) |
| System Prompt | *(empty)* | Default system instructions |
| Temperature | 0.7 | Creativity control (0 = deterministic, 2 = max random) |
| Session Mode | Stateless | `Stateless` (single turn) or `Session` (multi-turn conversation) |
| Max History | 20 | Number of conversation turns to keep (Session mode only) |
| Streaming | On | Enable token-by-token streaming preview |

**Tip:** Use the inline temperature slider on the canvas node for quick adjustments. For office/enterprise use, set Provider to Azure OpenAI with `gpt-4o-mini`.

---

### Knowledge Base

**Type ID:** `knowledge_base`

RAG node: indexes a folder of documents, then retrieves relevant chunks for a query. Handles embedding, chunking, and vector search automatically.

**Input Handles**

| Handle | Type | Description |
|--------|------|-------------|
| query | text | Search query |
| folder | text | Override docs folder path |

**Output Handles**

| Handle | Type | Description |
|--------|------|-------------|
| context | text | Formatted context string with citations |
| results | json | Raw search results with scores |
| indexStats | json | Index metadata (chunk count, file count) |

**Configuration**

| Field | Default | Description |
|-------|---------|-------------|
| Docs Folder | *(empty)* | Path to documents directory |
| Index Location | auto | Where to store the index (default: `{folder}/.ai-studio-index/`) |
| Embedding Provider | Azure OpenAI | Provider for embedding model |
| Embedding Model | text-embedding-3-small | Embedding model name |
| Chunk Strategy | Recursive | Recursive, Paragraph, Sentence, or Fixed Size |
| Chunk Size | 500 | Characters per chunk |
| Overlap | 50 | Overlap between chunks |
| Top K | 5 | Number of results to return |
| Min Score | 0.0 | Minimum similarity score threshold |
| File Types | *.md,*.txt,*.py,... | Glob patterns for files to index |
| Max File Size | 10 MB | Skip files larger than this |

**Example**

Connect `query` from an Input node, and pipe `context` into an LLM's `context` handle for grounded, cited answers.

---

### Router

**Type ID:** `router`

Conditional branching — sends data down one of multiple output paths.

**Input Handles**

| Handle | Type | Description |
|--------|------|-------------|
| input | text | Value to evaluate |

**Output Handles**

| Handle | Type | Description |
|--------|------|-------------|
| branch-0..N | bool | One handle per branch (dynamic) |

**Configuration**

| Field | Default | Description |
|-------|---------|-------------|
| Mode | Pattern | `Pattern Match` or `LLM Classify` |
| Branches | true, false | Comma-separated branch names |

**Tip:** In LLM Classify mode, the Router sends the input to an LLM which picks the best branch. In Pattern mode, it matches against branch names.

---

## Tools

### Tool

**Type ID:** `tool`

Invokes an MCP tool or built-in tool (shell, file, HTTP).

**Input Handles**

| Handle | Type | Description |
|--------|------|-------------|
| input | json | Tool arguments |

**Output Handles**

| Handle | Type | Description |
|--------|------|-------------|
| result | json | Tool execution result |

**Configuration**

| Field | Default | Description |
|-------|---------|-------------|
| Tool Name | *(empty)* | Fully qualified name, e.g. `builtin__shell` |
| Approval | Auto | `Auto` (run immediately), `Ask` (human approval), `Deny` (block) |

---

## Data I/O

### HTTP Request

**Type ID:** `http_request`

Makes HTTP API calls to external services.

**Input Handles**

| Handle | Type | Description |
|--------|------|-------------|
| url | text | Request URL |
| body | json | Request body |
| headers | json | Additional headers |

**Output Handles**

| Handle | Type | Description |
|--------|------|-------------|
| body | text | Response body |
| status | float | HTTP status code |
| headers | json | Response headers |

**Configuration**

| Field | Default | Description |
|-------|---------|-------------|
| URL | *(empty)* | Target endpoint |
| Method | GET | GET, POST, PUT, PATCH, DELETE, HEAD |
| Timeout | 30s | Request timeout |
| Auth | None | None, Bearer Token, Basic Auth, API Key |
| Auth Token Settings Key | — | Settings key for the credential (e.g. `provider.github.api_key`) |
| Request Body | *(empty)* | JSON body for POST/PUT requests |

---

### File Glob

**Type ID:** `file_glob`

Matches files by glob pattern. Returns file contents or metadata.

**Input Handles**

| Handle | Type | Description |
|--------|------|-------------|
| directory | text | Base directory to search |

**Output Handles**

| Handle | Type | Description |
|--------|------|-------------|
| files | json | Array of file contents/metadata |
| count | float | Number of matched files |
| paths | json | Array of file paths |

**Configuration**

| Field | Default | Description |
|-------|---------|-------------|
| Directory | *(empty)* | Base directory path |
| Pattern | * | Glob pattern (e.g. `*.csv`, `**/*.md`) |
| Mode | Text | Text, JSON, CSV, or Binary |
| Recursive | Off | Search subdirectories |
| Max Files | 100 | Limit number of results |
| Max File Size | 10 MB | Skip files larger than this |
| Sort By | Name | Name, Modified, or Size |
| Order | Ascending | Ascending or Descending |

---

### File Read

**Type ID:** `file_read`

Reads a single file from the local filesystem.

**Input Handles**

| Handle | Type | Description |
|--------|------|-------------|
| path | text | File path |

**Output Handles**

| Handle | Type | Description |
|--------|------|-------------|
| content | text | File contents |
| rows | json | Parsed rows (CSV mode only) |
| size | float | File size in bytes |

**Configuration**

| Field | Default | Description |
|-------|---------|-------------|
| File Path | *(empty)* | Absolute or relative path |
| Mode | Text | Text, JSON, CSV, or Binary |
| Max Size | 10 MB | Reject files larger than this |
| CSV Delimiter | , | Column separator (CSV mode) |
| First Row Header | On | Treat first row as column names (CSV mode) |

---

### File Write

**Type ID:** `file_write`

Writes content to a local file.

**Input Handles**

| Handle | Type | Description |
|--------|------|-------------|
| path | text | Output file path |
| content | any | Content to write |

**Output Handles**

| Handle | Type | Description |
|--------|------|-------------|
| path | text | Path of written file |
| bytes | float | Bytes written |

**Configuration**

| Field | Default | Description |
|-------|---------|-------------|
| File Path | *(empty)* | Output path |
| Mode | Text | Text, JSON, or CSV |
| Write Mode | Overwrite | Overwrite or Append |
| Create Dirs | On | Create parent directories if missing |

---

### Shell Exec

**Type ID:** `shell_exec`

Runs a shell command and captures output.

**Input Handles**

| Handle | Type | Description |
|--------|------|-------------|
| command | text | Command to execute |
| stdin | text | Standard input |

**Output Handles**

| Handle | Type | Description |
|--------|------|-------------|
| stdout | text | Standard output |
| stderr | text | Standard error |
| exit_code | float | Process exit code |

**Configuration**

| Field | Default | Description |
|-------|---------|-------------|
| Command | *(empty)* | Shell command to run |
| Shell | bash | bash, sh, or zsh |
| Working Dir | *(empty)* | Working directory (default: app directory) |
| Timeout | 30s | Kill process after this time |

**Tip:** Use template syntax `{{input}}` in the command field to inject values from upstream nodes.

---

## Logic

### Approval

**Type ID:** `approval`

Human-in-the-loop gate. Pauses execution until a human approves or rejects.

**Input Handles**

| Handle | Type | Description |
|--------|------|-------------|
| data | any | Data to review |

**Output Handles**

| Handle | Type | Description |
|--------|------|-------------|
| approved | bool | Fires when approved |
| rejected | bool | Fires when rejected |

**Configuration**

| Field | Default | Description |
|-------|---------|-------------|
| Message | "Review before continuing" | Prompt shown to the reviewer |

---

### Transform

**Type ID:** `transform`

Data transformation — template strings, JSONPath extraction, or JavaScript expressions.

**Input Handles**

| Handle | Type | Description |
|--------|------|-------------|
| *dynamic* | any | One handle per named input (configurable) |

**Output Handles**

| Handle | Type | Description |
|--------|------|-------------|
| output | any | Transformed result |

**Configuration**

| Field | Default | Description |
|-------|---------|-------------|
| Mode | Template | Template, JSONPath, or Expression |
| Inputs | [input] | Named input handles (add/remove dynamically) |
| Template | `{{input}}` | Template string with `{{varName}}` placeholders, JSONPath query, or JS expression |

**Examples**

- **Template:** `Hello, {{name}}! You said: {{message}}`
- **JSONPath:** `$.data.items[0].title`
- **Expression:** `inputs.price * inputs.quantity`

---

### Validator

**Type ID:** `validator`

Validates JSON data against a JSON Schema.

**Input Handles**

| Handle | Type | Description |
|--------|------|-------------|
| data | json | Data to validate |

**Output Handles**

| Handle | Type | Description |
|--------|------|-------------|
| valid | bool | True if validation passes |
| data | json | Pass-through of input data |
| errors | json | Array of validation error messages |

**Configuration**

| Field | Default | Description |
|-------|---------|-------------|
| JSON Schema | `{}` | JSON Schema definition |
| Fail on Error | Off | If on, the node errors instead of outputting `valid: false` |

---

### Iterator

**Type ID:** `iterator`

Loops over an array, executing downstream nodes once per item. Must be paired with an Aggregator node.

**Input Handles**

| Handle | Type | Description |
|--------|------|-------------|
| items | json | Array to iterate over |

**Output Handles**

| Handle | Type | Description |
|--------|------|-------------|
| item | any | Current item in the loop |

**Configuration**

| Field | Default | Description |
|-------|---------|-------------|
| Mode | Sequential | Sequential or Parallel (future) |
| Expression | *(empty)* | Optional JSONPath to extract array (e.g. `$.data[*]`) |

**Rule:** Every Iterator must have exactly one downstream Aggregator. Using Iterator + Loop in the same workflow is not allowed.

---

### Aggregator

**Type ID:** `aggregator`

Collects results from an Iterator's loop body into a single output.

**Input Handles**

| Handle | Type | Description |
|--------|------|-------------|
| input | any | Result from each iteration |

**Output Handles**

| Handle | Type | Description |
|--------|------|-------------|
| result | any | Aggregated results |
| count | float | Number of items processed |

**Configuration**

| Field | Default | Description |
|-------|---------|-------------|
| Strategy | Array | `Array` (collect all), `Concat` (join as text), `Merge` (combine objects) |
| Separator | \n | Text separator (Concat mode only) |

---

### Loop

**Type ID:** `loop`

Iterative refinement — runs a subgraph repeatedly until an exit condition is met. Must contain an Exit node.

**Input Handles**

| Handle | Type | Description |
|--------|------|-------------|
| input | any | Initial input for first iteration |

**Output Handles**

| Handle | Type | Description |
|--------|------|-------------|
| output | any | Final result when loop exits |
| iterations | json | Array of all iteration outputs |
| count | float | Number of iterations executed |

**Configuration**

| Field | Default | Description |
|-------|---------|-------------|
| Max Iterations | 5 | Hard limit (1-50) |
| Exit Condition | Max Iterations | `Max Iterations`, `Evaluator` (Router decides), `Stable Output` (convergence) |
| Stability Threshold | 0.95 | Similarity threshold for Stable Output mode |
| Feedback Mode | Replace | `Replace` (output becomes next input) or `Append` (concat into array) |

**Rule:** Every Loop must contain exactly one Exit node. Loop + Iterator in the same workflow is not allowed.

---

### Exit

**Type ID:** `exit`

Loop exit point. Pass-through node that marks where the loop body ends.

**Input Handles**

| Handle | Type | Description |
|--------|------|-------------|
| input | any | Result from this iteration |

**Output Handles**

| Handle | Type | Description |
|--------|------|-------------|
| output | any | Pass-through (Loop reads this) |

**Tip:** Place the Exit node as the last node inside a Loop's body. The Loop node reads Exit's output to decide whether to continue.

---

## Composition

### Subworkflow

**Type ID:** `subworkflow`

Embeds another workflow as a single node. Useful for reusing common patterns.

**Input Handles**

| Handle | Type | Description |
|--------|------|-------------|
| input | any | Input to the sub-workflow |

**Output Handles**

| Handle | Type | Description |
|--------|------|-------------|
| output | any | Sub-workflow's result |

**Configuration**

| Field | Default | Description |
|-------|---------|-------------|
| Workflow Name | *(empty)* | Name of the workflow to embed |

---

## Common Patterns

### Chat API (Webhook + LLM)
```
Webhook → body → LLM → response → Output
```
Expose an LLM as an HTTP endpoint. Set Response Mode to "Wait" to return the LLM response directly.

### RAG Pipeline (Input + Knowledge Base + LLM)
```
Input → query → Knowledge Base → context → LLM → response → Output
```
User asks a question, KB retrieves relevant docs, LLM answers with citations.

### Data Processing (File Glob + Iterator + Transform + Aggregator)
```
File Glob → files → Iterator → item → Transform → output → Aggregator → result → Output
```
Process a batch of files through a transformation pipeline.

### Self-Refining (Loop + LLM + Router)
```
Input → Loop → LLM → Router → [continue/done]
                              → Exit → Output
```
LLM generates, Router evaluates quality, loops until good enough.

### Approval Gate (LLM + Approval + Output)
```
LLM → response → Approval → approved → Output
                           → rejected → (discard)
```
Human reviews LLM output before it goes anywhere.
