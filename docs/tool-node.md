# Tool Node — Reference Guide

The Tool node calls an MCP tool or built-in tool as a step in your workflow. It's the bridge between AI thinking (LLM nodes) and real-world actions (shell, files, APIs, external services).

**Tool Name format**: `server__tool_name` (double underscore separates server from tool)

---

## Built-in Tools

Available out of the box — no setup required.

| Tool Name | Input | What it does |
|-----------|-------|-------------|
| `builtin__shell` | `{"command": "..."}` | Run any shell command |
| `builtin__read_file` | `{"path": "..."}` | Read a file's contents |
| `builtin__write_file` | `{"path": "...", "content": "..."}` | Write content to a file |
| `builtin__list_directory` | `{"path": "..."}` | List files in a directory |

## MCP Server Tools

Connect an MCP server in Settings, and all its tools become available. Examples:

| Server | Tool Name | What it does |
|--------|-----------|-------------|
| GitHub | `github__create_issue` | Create a GitHub issue |
| GitHub | `github__search_repos` | Search repositories |
| Slack | `slack__send_message` | Post a Slack message |
| Postgres | `postgres__query` | Run a SQL query |

To see available tools: open **Settings > MCP Servers** — each connected server lists its tools.

---

## Configuration

| Field | What it does | Example |
|-------|-------------|---------|
| **Tool Name** | Qualified name of the tool to call | `builtin__shell` |
| **Tool Input** | Static JSON input (optional — if empty, uses incoming data) | `{"command": "git status"}` |
| **Approval** | `auto` (run immediately), `ask` (pause for human approval), `deny` (block) | `auto` |

**Input handle**: json (tool arguments from upstream node)
**Output handle**: json (tool's result)

### How Input Is Resolved

1. If **Tool Input** is set in config → uses that (static)
2. Else if upstream node sends data → uses that (dynamic)
3. Else → sends `{}` (empty object)

This means you can either hardcode the input or wire it dynamically from upstream nodes.

### Approval Modes

| Mode | Behavior | Use when |
|------|----------|----------|
| `auto` | Runs immediately, no pause | Safe operations (reads, local commands) |
| `ask` | Pauses workflow, shows approval dialog with data preview | Destructive ops, external API calls, writes |
| `deny` | Blocks execution, workflow errors | Disabled tools you want to keep in the graph |

When approval mode is `ask`, the UI shows a dialog with the tool name and a preview of the input data. You have 5 minutes to approve or deny. Timeout = denied.

---

## Examples: Simple to Killer

### 1. Hello Shell

Run a command, see the output.

```
[Input] → [Tool] → [Output]
```

| Node | Config |
|------|--------|
| Input | name=`cmd`, dataType=`json`, default=`{"command": "echo Hello from AI Studio"}` |
| Tool | toolName=`builtin__shell`, approval=`auto` |
| Output | name=`result` |

**Important**: The Input data type must be `json`, not `text`. Tool nodes require a JSON object matching the tool's parameter names. `builtin__shell` expects `{"command": "..."}`, so that's what the Input must produce.

If you wire a text Input (plain string) into a Tool node, you'll get an error. Use a **Transform** node between them to reshape: template mode with `{"command": "{{input}}"}`.

---

### 2. Read + Summarize a File

Read a file with a Tool node, then summarize with an LLM.

```
[Tool: read_file] → [LLM] → [Output]
```

| Node | Config |
|------|--------|
| Tool | toolName=`builtin__read_file`, toolInput=`{"path": "/home/user/notes.txt"}` |
| LLM | provider=`anthropic`, model=`claude-haiku-4-5`, systemPrompt=`Summarize this text in 3 bullet points.` |
| Output | name=`summary` |

The Tool reads the file, passes the content to the LLM, which returns a summary.

---

### 3. LLM Writes, Tool Saves

Generate content with an LLM, reshape it, save to disk.

```
[Input] → [LLM] → [Transform] → [Tool: write_file] → [Output]
```

| Node | Config |
|------|--------|
| Input | name=`topic`, default=`Benefits of Rust` |
| LLM | systemPrompt=`Write a short blog post about the given topic.` |
| Transform | mode=`template`, template=`{"path": "/tmp/blog.md", "content": "{{input}}"}` |
| Tool | toolName=`builtin__write_file`, approval=`auto` |
| Output | name=`status` |

The Transform node reshapes the LLM's text output into the `{"path", "content"}` JSON that `write_file` expects.

---

### 4. Git Status Reporter

Run git commands, have an LLM explain them to a non-technical PM.

```
[Tool: shell] → [LLM] → [Output]
```

| Node | Config |
|------|--------|
| Tool | toolName=`builtin__shell`, toolInput=`{"command": "cd /path/to/repo && git status --short && echo '---' && git log --oneline -5"}` |
| LLM | systemPrompt=`You are a friendly project assistant. Summarize this git status for a non-technical PM. What files changed? Any concerns? Use bullet points.` |
| Output | name=`report` |

---

### 5. Directory Scanner with Iterator

List a directory, iterate over each file, read and summarize each one.

```
[Tool: list_directory] → [Transform] → [Iterator] → [Tool: read_file] → [LLM] → [Aggregator] → [Output]
```

| Node | Config |
|------|--------|
| Tool 1 | toolName=`builtin__list_directory`, toolInput=`{"path": "./src"}` |
| Transform | mode=`script`, template=`split('\n') \| filter(nonempty)` — converts the string output to an array |
| Iterator | mode=`sequential` |
| Tool 2 | toolName=`builtin__read_file` — receives `{"path": "..."}` from iterator |
| LLM | systemPrompt=`Summarize what this file does in one sentence.` |
| Aggregator | strategy=`concat`, separator=`\n` |
| Output | name=`codebase_summary` |

Result: a one-line summary of every file in the directory.

---

### 6. Code Review Pipeline

Read source code, run an LLM code review, save the report.

```
[Input] → [Tool: read_file] → [LLM: review] → [LLM: format] → [Tool: write_file] → [Output]
```

| Node | Config |
|------|--------|
| Input | name=`file_path`, default=`./src/main.rs` |
| Transform 1 | mode=`template`, template=`{"path": "{{input}}"}` |
| Tool 1 | toolName=`builtin__read_file`, approval=`auto` |
| LLM 1 | systemPrompt=`Review this code for: bugs, security issues, performance problems, and style. List each finding with severity (critical/warning/info).` |
| LLM 2 | systemPrompt=`Format the code review as a clean markdown report with sections for Critical, Warnings, and Info.` |
| Transform 2 | mode=`template`, template=`{"path": "/tmp/code-review.md", "content": "{{input}}"}` |
| Tool 2 | toolName=`builtin__write_file`, approval=`auto` |
| Output | name=`review_path` |

---

### 7. DevOps: Test → Auto-File Issues

Run tests, check if they pass, auto-create a GitHub issue on failure.

```
[Tool: shell] → [Router] → (fail) → [LLM] → [Transform] → [Tool: github__create_issue] → [Output]
                          → (pass) → [Output: "All tests passing"]
```

| Node | Config |
|------|--------|
| Tool 1 | toolName=`builtin__shell`, toolInput=`{"command": "cd /repo && cargo test 2>&1"}` |
| Router | mode=`pattern`, branches=`pass,fail` — matches "FAILED" → fail, else → pass |
| LLM | systemPrompt=`Analyze these test failures. Return JSON: {"title": "...", "body": "..."}` |
| Transform | mode=`jsonpath`, template=`$` — pass through the JSON |
| Tool 2 | toolName=`github__create_issue`, approval=`ask` — human confirms before creating |
| Output 1 | name=`issue_created` |
| Output 2 | name=`all_clear`, connected to Router's "pass" branch |

**Key**: The GitHub Tool node uses `approval=ask` so a human reviews the issue before it's posted.

---

### 8. Self-Healing Deploy Pipeline (Killer)

Deploy, health check, auto-rollback on failure, generate incident report.

```
[Input: deploy cmd]
  → [Tool: shell (deploy)]
    → [Router: exit code]
       ├─ success → [Tool: shell (health check)]
       │              → [Router: HTTP 200?]
       │                  ├─ healthy → [Output: "Deploy successful"]
       │                  └─ unhealthy → [Tool: shell (rollback)]
       │                                   → [LLM: analyze]
       │                                     → [Tool: write_file (incident report)]
       │                                       → [Output: incident report]
       └─ failure → [Tool: shell (rollback)]
                      → [LLM: analyze build error]
                        → [Tool: write_file (incident report)]
                          → [Output: incident report]
```

| Node | Config |
|------|--------|
| Input | `{"deploy_cmd": "kubectl apply -f deploy.yaml", "rollback_cmd": "kubectl rollout undo deployment/app", "health_url": "https://app.example.com/health"}` |
| Tool 1 | toolName=`builtin__shell`, approval=`ask` — confirm before deploying |
| Router 1 | mode=`pattern`, branches=`success,failure` |
| Tool 2 | toolName=`builtin__shell`, toolInput uses health_url via Transform |
| Router 2 | mode=`pattern`, branches=`healthy,unhealthy` — matches "200" |
| Rollback Tools | toolName=`builtin__shell`, approval=`ask` |
| LLM | systemPrompt=`Analyze this deployment failure. Write an incident report with: what happened, root cause hypothesis, and next steps.` |
| File Tool | toolName=`builtin__write_file` — saves incident report |

**Why this is killer**: The workflow handles success, health check failure, AND build failure — all with auto-rollback and AI-generated incident reports. The `ask` approval on deploy and rollback keeps a human in the loop for destructive actions.

---

### 9. Webcam Security Monitor (Real-World MCP Example)

Uses a custom Vision MCP server (YOLO object detection + webcam capture) to build an AI-powered security camera.

**Prerequisites**: Vision MCP server connected in Settings:
- Name: `vision`
- Transport: `stdio`
- Command: `python3 /path/to/vision-mcp-server/server.py`
- Requires: YOLO detection API running on port 8004

```
[Tool: webcam_detect] → [Router: person detected?]
                             ├── person → [LLM: describe scene] → [Output: alert]
                             └── empty  → [Output: skip]
```

| Node | Config |
|------|--------|
| Tool | toolName=`vision__webcam_detect`, approval=`auto` — captures webcam frame + runs YOLO detection in one call |
| Router | mode=`pattern`, branches=`person,empty` — matches "person" in detection text |
| LLM | systemPrompt=`You are a security camera analyst. Describe what was detected: how many people, positions, other objects. Be concise.` |
| Output 1 | name=`alert`, format=`markdown` — the LLM's scene description |
| Output 2 | name=`skip`, format=`text` — empty frame, nothing detected |

**What makes this special**:
- **Hardware integration** — webcam + GPU (RTX 5090 YOLO inference in ~30ms)
- **MCP tool** — the Vision server exposes 3 tools (`webcam_capture`, `detect_objects`, `webcam_detect`), all available in the Tool picker
- **Local-first** — no cloud, no API keys, all runs on your machine
- **Extensible** — add Iterator for continuous monitoring, add File Write to save alerts, add HTTP Request to send Slack notifications

This template is bundled in AI Studio as **Webcam Monitor**.

---

## Tips

- **Use Transform before Tool** when you need to reshape data into the format the tool expects
- **Use `ask` approval** for any tool that modifies external state (writes, API calls, deploys)
- **Chain Tool → LLM** to have AI analyze tool output (logs, test results, API responses)
- **Chain LLM → Tool** to have AI decide what action to take, then execute it
- **Use Iterator + Tool** to run the same tool on multiple inputs (batch file processing, multi-repo operations)
- **MCP servers** are the killer feature — connect GitHub, Slack, databases, and every tool they expose becomes a drag-and-drop node
- Tool nodes show the **server name** below the tool name when connected to an MCP server
- Check **Settings > MCP Servers** to see all available tools and their expected input schemas

## Common Mistakes

### "Input should be a valid dictionary"

**Cause**: You wired a text output (plain string) directly into a Tool node. Tool nodes require a JSON object.

**Fix**: Add a **Transform** node between the text source and the Tool node:
- Mode: `template`
- Template: `{"command": "{{input}}"}` (for shell) or `{"path": "{{input}}"}` (for read_file)

### "Tool not found"

**Cause**: Wrong tool name format. Must be `server__tool_name` (double underscore).

**Fix**: Check the exact name in Settings > MCP Servers. Built-in tools are `builtin__shell`, `builtin__read_file`, `builtin__write_file`, `builtin__list_directory`.

### "Tool execution denied by user"

**Cause**: Approval mode is `ask` and you didn't approve within 5 minutes (or clicked Deny).

**Fix**: Watch for the approval dialog in the UI. If you want auto-execution, set approval to `auto`.

---

## Tool Name Quick Reference

| Category | Tool Name | Input Schema |
|----------|-----------|-------------|
| Shell | `builtin__shell` | `{"command": str, "timeout?": number, "cwd?": str}` |
| Read | `builtin__read_file` | `{"path": str}` |
| Write | `builtin__write_file` | `{"path": str, "content": str}` |
| List | `builtin__list_directory` | `{"path": str}` |
| MCP | `{server}__{tool}` | Varies — check MCP server docs |
