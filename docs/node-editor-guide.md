# Node Editor — Step-by-Step Guide

How to build workflows in AI Studio's Node Editor.

---

## Quick Start: Your First Workflow

### Step 1: Create a Workflow

1. Open **Node Editor** from the sidebar
2. Click **+ New Workflow** (or pick a template from **Templates**)
3. You're on the canvas — empty grid, palette on the left

### Step 2: Add Nodes

Two ways to add nodes:
- **Drag** from the left palette onto the canvas
- **Click** a palette item, then **click** on the canvas to place it (works better on macOS)

### Step 3: Connect Nodes

- Drag from an **output handle** (right side of a node) to an **input handle** (left side of another node)
- Handles are color-coded by data type — compatible types snap together
- Invalid connections are rejected automatically

### Step 4: Configure Nodes

- Click any node to open the **Config Panel** (right sidebar)
- Fill in the fields for that node type (see reference below)
- Double-click a node header to give it a custom label (e.g. "LLM · Summarizer")

### Step 5: Run

1. Click **Run** (top-right)
2. If you have Input nodes, a dialog asks for values
3. Watch nodes light up as they execute (blue = running, green = done, red = error)
4. Click any completed node to see its output in the config panel

---

## Simplest Possible Workflow

```
[Input] → [LLM] → [Output]
```

**3 nodes, 2 edges.** Here's exactly what to configure:

### Input Node
| Field | Value |
|-------|-------|
| Name | `prompt` |
| Data Type | `text` |
| Default | `Summarize the benefits of exercise` |

### LLM Node
| Field | Value |
|-------|-------|
| Provider | Pick one you've configured in Settings (e.g. `anthropic`, `google`, `ollama`) |
| Model | Pick a model (e.g. `claude-sonnet-4-5-20250929`, `gemini-2.0-flash`, `llama3.2`) |
| System Prompt | `You are a helpful assistant. Be concise.` |
| Temperature | `0.7` (default) |

### Output Node
| Field | Value |
|-------|-------|
| Name | `result` |
| Format | `text` (or `markdown`) |

**Connect**: Input → LLM → Output. Click Run. Done.

---

## Available Providers & Models

Configure API keys in **Settings** before using a provider.

| Provider | Models | Setup |
|----------|--------|-------|
| **Anthropic** | claude-sonnet-4-5, claude-haiku-4-5, claude-opus-4-6 | API key in Settings |
| **Google** | gemini-2.0-flash, gemini-2.5-pro, gemini-2.5-flash | API key in Settings |
| **Azure OpenAI** | gpt-4o, gpt-4o-mini, gpt-4.1 | Endpoint + key in Settings |
| **Ollama** | llama3.2, llama3.1, mistral, codellama, qwen2.5 | Run Ollama locally on port 11434 |
| **Local** | qwen3-vl, qwen2.5-vl, llama3.2 | vLLM/Docker on custom port |

**Which LLM to pick?**
- Quick/cheap tasks → `gemini-2.0-flash` or `claude-haiku-4-5`
- Quality tasks → `claude-sonnet-4-5` or `gpt-4o`
- Local/free → `ollama` with `llama3.2`
- Vision/images → `qwen3-vl` (local) or `gemini-2.5-pro`

---

## All 16 Node Types — What They Do & How to Configure

### Inputs/Outputs

#### Input
Entry point for the workflow. Each Input becomes a field in the Run dialog.

| Field | What it does |
|-------|-------------|
| **Name** | Variable name (used by downstream nodes) |
| **Data Type** | `text`, `json`, `boolean`, or `file` |
| **Default** | Pre-filled value in the Run dialog |

**Output handle**: text, json, bool, or binary (depends on data type)

#### Output
Exit point. Captures the workflow's final result.

| Field | What it does |
|-------|-------------|
| **Name** | Label shown in results |
| **Format** | `text`, `markdown`, or `json` |

**Input handle**: any

---

### AI

#### LLM
Calls a language model. The core node for AI workflows.

| Field | What it does |
|-------|-------------|
| **Provider** | Which AI service (anthropic, google, azure_openai, ollama, local) |
| **Model** | Which model from that provider |
| **System Prompt** | Instructions for the LLM |
| **Temperature** | 0 = deterministic, 2 = creative. Default 0.7 |
| **Max Tokens** | Response length limit. 0 = provider default |
| **Session Mode** | `stateless` (default) or `session` (multi-turn memory) |
| **Max History** | If session mode: how many past turns to keep (1-100) |

**Input handle**: text (the user/input message)
**Output handle**: text (LLM's response)

#### Router
Branches the workflow based on conditions.

| Field | What it does |
|-------|-------------|
| **Mode** | `pattern` (regex matching) or `llm` (AI classification) |
| **Branches** | Comma-separated branch names (e.g. `positive,negative,neutral`) |

**Input handle**: text
**Output handles**: one per branch name

---

### Tools

#### Tool
Calls an MCP tool or built-in tool.

| Field | What it does |
|-------|-------------|
| **Tool Name** | Name of the tool to call |
| **Server Name** | MCP server (leave empty for built-in tools) |
| **Approval** | `auto` (run immediately), `ask` (pause for approval), `deny` (block) |

**Input handle**: json (tool arguments)
**Output handle**: json (tool result)

---

### Data I/O

#### HTTP Request
Makes HTTP API calls.

| Field | What it does |
|-------|-------------|
| **URL** | Endpoint (supports `{{input}}` template substitution) |
| **Method** | GET, POST, PUT, PATCH, DELETE, HEAD |
| **Headers** | JSON object of headers |
| **Body** | Request body (for POST/PUT/PATCH) |
| **Auth** | `none`, `bearer`, `basic`, or `api_key` |
| **Timeout** | Seconds before timeout (default 30) |

**Input handle**: text or json
**Output handle**: json (response body)

#### File Glob
Find files matching a pattern.

| Field | What it does |
|-------|-------------|
| **Directory** | Base directory to search |
| **Pattern** | Glob pattern (e.g. `*.csv`, `**/*.json`) |
| **Mode** | `text`, `json`, `csv`, or `binary` |
| **Recursive** | Search subdirectories |
| **Max Files** | Limit number of results |
| **Sort By** | `name`, `modified`, or `size` |

**Input handle**: text (optional override for directory)
**Output handle**: rows (array of file contents) or json

#### File Read
Read a single file.

| Field | What it does |
|-------|-------------|
| **Path** | File path to read |
| **Mode** | `text`, `json`, `csv`, or `binary` |
| **Max Size** | Max file size in MB |

**Input handle**: text (path override)
**Output handle**: text, json, or rows (depends on mode)

#### File Write
Write content to a file.

| Field | What it does |
|-------|-------------|
| **Path** | Destination file path |
| **Write Mode** | `overwrite` or `append` |
| **Create Dirs** | Create parent directories if missing |

**Input handle**: text or json (content to write)
**Output handle**: text (written file path)

#### Shell Exec
Run a shell command.

| Field | What it does |
|-------|-------------|
| **Command** | Shell command to execute |
| **Shell** | `bash`, `sh`, or `zsh` |
| **Working Dir** | Directory to run in |
| **Timeout** | Seconds before timeout |

**Input handle**: text (appended to command or used as stdin)
**Output handle**: text (stdout)

---

### Logic

#### Transform
Reshape data between nodes. Three modes:

| Mode | What it does | Example |
|------|-------------|---------|
| **template** | String interpolation | `Hello {{name}}, you have {{count}} items` |
| **jsonpath** | Extract from JSON (RFC 9535) | `$.results[0].name` |
| **script** | Pipe operations | `split(',') \| map(trim) \| filter(nonempty) \| join('\n')` |

| Field | What it does |
|-------|-------------|
| **Mode** | `template`, `jsonpath`, or `script` |
| **Template** | The expression (varies by mode) |
| **Inputs** | Named inputs from upstream nodes (for template mode) |

**Input handle**: text or json
**Output handle**: text or json

#### Validator
Validates data against a JSON Schema.

| Field | What it does |
|-------|-------------|
| **Schema** | JSON Schema string (e.g. `{"type": "object", "required": ["name"]}`) |
| **Fail On Error** | Stop workflow on validation failure (checkbox) |

**Input handle**: json
**Output handles**: `valid` (json) and `invalid` (json)

#### Approval
Pauses workflow for human review.

| Field | What it does |
|-------|-------------|
| **Message** | What to show the user |
| **Show Data** | Include upstream data in the approval dialog |
| **Timeout** | Auto-reject after N seconds (0 = wait forever) |

**Input handle**: any
**Output handle**: any (passes through on approval)

#### Iterator
Loops over an array, running a subgraph for each item.

| Field | What it does |
|-------|-------------|
| **Mode** | `sequential` (one at a time) or `parallel` |
| **Expression** | JSONPath to extract array from input (e.g. `$.items`) |
| **Max Concurrency** | For parallel mode |

**Input handle**: json (must contain an array)
**Output handle**: connects to the subgraph that runs per item

**Rule**: Must have exactly one Aggregator downstream to collect results.

#### Aggregator
Collects results from an Iterator's subgraph.

| Field | What it does |
|-------|-------------|
| **Strategy** | `array` (collect as JSON array), `concat` (join as text), `merge` (deep merge objects) |
| **Separator** | For concat mode (e.g. `\n`) |

**Input handle**: from the subgraph's last node
**Output handle**: json or text (aggregated results)

---

### Composition

#### Subworkflow
Embeds another saved workflow as a single node.

| Field | What it does |
|-------|-------------|
| **Workflow ID** | Which saved workflow to run |

**Input/Output handles**: match the embedded workflow's Input/Output nodes

---

## Common Workflow Patterns

### 1. Simple Q&A
```
Input → LLM → Output
```

### 2. Summarize a File
```
File Read → LLM ("Summarize this") → Output
```

### 3. Multi-Step Chain
```
Input → LLM ("Extract keywords") → LLM ("Write article from keywords") → Output
```

### 4. Conditional Routing
```
Input → Router (positive/negative/neutral)
  ├── positive → LLM ("Expand on the praise")  → Output
  ├── negative → LLM ("Address the concern")   → Output
  └── neutral  → LLM ("Ask follow-up question") → Output
```

### 5. Batch Processing (Iterator)
```
File Glob (*.txt) → Iterator → LLM ("Summarize") → Aggregator → File Write
```

### 6. API → AI → File
```
HTTP Request (GET /api/data) → Transform (extract field) → LLM ("Analyze") → File Write
```

### 7. Human-in-the-Loop
```
Input → LLM ("Draft email") → Approval ("Review before sending") → HTTP Request (send)
```

---

## Tips

- **Ctrl+S** to save, **Ctrl+D** to duplicate a node, **Delete** to remove
- **Right-click** canvas for quick node insertion
- **Ctrl+A** to select all, **Ctrl+C/V** to copy/paste nodes
- Graph auto-saves before every run — you won't lose work
- Click a completed node to see its output, tokens used, and duration
- Use **Transform** nodes between incompatible types (e.g. JSON → text)
- The **Export** button (download icon) saves the workflow as JSON for sharing
- **Templates** button has 10 pre-built workflows to learn from
