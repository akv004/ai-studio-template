# Enterprise Integration Patterns — Data I/O Nodes

**Status**: DRAFT — pending peer review
**Phase**: 4B (post 4A new node types)
**Inspired by**: Apache Camel File Component, MuleSoft File Connector, n8n
**Author**: AI Studio PM
**Date**: 2026-02-19

---

## Problem Statement

The current File Read node reads a single file. Real-world workflows need to:
1. Process multiple files from a directory (glob/wildcard filtering)
2. Watch directories for new files (polling trigger)
3. Stream file data through LLM with session context (multi-turn)
4. Handle file I/O as part of larger ETL/automation pipelines

These patterns are well-established in enterprise integration (Apache Camel EIPs) and should be adapted for AI-native workflows.

---

## New Node Types

### 1. File Glob Node (Priority: P0)

**Purpose**: Read all files matching a pattern from a directory. Equivalent to Camel's `file://dir?include=*.csv`.

**Config**:
| Field | Type | Default | Description |
|-------|------|---------|-------------|
| directory | string | required | Base directory path |
| pattern | string | `*` | Glob pattern (`*.csv`, `**/*.py`, `report_*.txt`) |
| mode | enum | text | text / json / csv / binary |
| recursive | bool | false | Search subdirectories |
| maxFiles | int | 100 | Limit to prevent accidental overload |
| sortBy | enum | name | name / modified / size |
| sortOrder | enum | asc | asc / desc |

**Output Handles**:
| Handle | Type | Description |
|--------|------|-------------|
| files | json | Array of `{path, name, content, size, modified}` |
| count | float | Number of matched files |
| paths | json | Array of file paths only (lightweight) |

**Security**: Same denied-paths list as File Read. Per-file size limit via `maxSize` (MB) — files exceeding the limit are skipped. Total file count capped at `maxFiles`.

**UI Node Design**:
```
+---------------------------+
|   FILE GLOB               |
| [path] ← directory       |
|   /tmp/data/              |
|   Pattern: *.csv          |
|   Mode: Text              |
|   Recursive: [ ]          |
|           files → [json]  |
|           count → [float] |
|           paths → [json]  |
+---------------------------+
```

**Executor Pseudocode** (Rust):
```
1. Validate directory exists and is readable
2. Apply glob pattern (use `glob` crate)
3. Sort results
4. For each file (up to maxFiles):
   a. Check security (denied paths)
   b. Read content per mode (text/json/csv/binary)
   c. Build file object {path, name, content, size, modified}
5. Return {files: [...], count: N}
```

### 2. File Watch Node (Priority: P2 — Future)

**Purpose**: Trigger workflow when files appear in a directory. Equivalent to Camel's `file://dir?noop=true` with polling consumer.

**Config**:
| Field | Type | Default | Description |
|-------|------|---------|-------------|
| directory | string | required | Watch directory |
| pattern | string | `*` | Glob filter |
| pollInterval | int | 5000 | Milliseconds between checks |
| processedAction | enum | none | none / move / delete |
| processedDir | string | - | Move-to directory (if processedAction=move) |

**Deferred**: Requires event-driven trigger architecture (not just DAG execution).

### 3. Iterator/Splitter Node (Priority: P1)

**Purpose**: Split an array into individual items and process each through a sub-pipeline. Equivalent to Camel's Splitter EIP.

**Config**:
| Field | Type | Default | Description |
|-------|------|---------|-------------|
| mode | enum | sequential | sequential / parallel |
| maxConcurrency | int | 5 | For parallel mode |
| expression | string | - | Optional JSONPath to extract array from input |

**Input Handles**:
| Handle | Type | Description |
|--------|------|-------------|
| items | json | Array to iterate over |

**Output Handles**:
| Handle | Type | Description |
|--------|------|-------------|
| item | any | Current item (emitted per iteration) |
| index | float | Current index |
| total | float | Total count |

**Engine Impact**: This requires extending the DAG walker to support sub-graph iteration. The Iterator node marks a "loop boundary" — downstream nodes until the Aggregator are re-executed per item.

**Implementation approach**:
- Option A: Inline loop — engine detects Iterator→Aggregator boundaries, re-executes subgraph per item.
- Option B: Subworkflow — Iterator spawns a subworkflow per item (reuses existing Subworkflow executor).
- **Recommended: Option B** — lower engine complexity, leverages existing subworkflow execution.

### 4. Aggregator Node (Priority: P1)

**Purpose**: Collect outputs from Iterator iterations into a single result. Equivalent to Camel's Aggregator EIP.

**Config**:
| Field | Type | Default | Description |
|-------|------|---------|-------------|
| strategy | enum | array | array / concat / merge (custom: future) |
| separator | string | `\n` | For concat mode |

**Input Handles**:
| Handle | Type | Description |
|--------|------|-------------|
| item | any | Individual result from Iterator |

**Output Handles**:
| Handle | Type | Description |
|--------|------|-------------|
| result | any | Aggregated output |
| count | float | Number of items aggregated |

---

## Multi-Image Vision Pipeline (Priority: P0)

### Problem
Current LLM node sends a single image to vision models. Real workflows need to send multiple images from a directory (e.g., "analyze all screenshots", "compare these diagrams", "OCR all scanned pages").

### Qwen3-VL API Reference
The project's local Qwen API (`qwen3-vl-api`) already supports multi-image natively:

**OpenAI-compatible endpoint** (`/v1/chat/completions`):
```json
{
  "model": "qwen3-vl",
  "messages": [{
    "role": "user",
    "content": [
      {"type": "text", "text": "Compare these two charts"},
      {"type": "image_url", "image_url": {"url": "data:image/png;base64,IMG1..."}},
      {"type": "image_url", "image_url": {"url": "data:image/png;base64,IMG2..."}}
    ]
  }]
}
```

**Native endpoint** (`/chat`):
```json
{
  "session_id": "workflow-123",
  "prompt": "Compare these charts",
  "images": ["/path/to/chart1.png", "/path/to/chart2.png"]
}
```

Both endpoints resize images automatically (max 1.5M pixels, max 1280px side) and handle OOM fallback with further downscaling.

### Design

#### File Glob → LLM Multi-Image Flow
```
File Glob (*.png, binary) → LLM (vision) → Output
```

The File Glob node in binary mode outputs:
```json
{
  "files": [
    {"path": "/dir/img1.png", "name": "img1.png", "content": "base64...", "mime_type": "image/png", "encoding": "base64", "size": 12345},
    {"path": "/dir/img2.png", "name": "img2.png", "content": "base64...", "mime_type": "image/png", "encoding": "base64", "size": 23456}
  ],
  "count": 2
}
```

#### LLM Executor Changes
The LLM executor detects multiple images from incoming data:
1. Scan `ctx.node_outputs` for any upstream output containing `files` array with `encoding: "base64"` + `mime_type: "image/*"` entries
2. Build multi-image content blocks: one `image_url` per image + one `text` block for the prompt
3. Send via the existing sidecar `/chat/direct` endpoint (which forwards to `/v1/chat/completions`)

#### Sidecar Changes
Update `/chat/direct` to accept `images: List[{data, mime_type}]` (already done for single image). The multimodal message builder already creates one `image_url` block per image — just ensure it loops over all images, not just the first.

### Memory Considerations
- Multiple high-res images can exhaust GPU VRAM (Qwen VL uses ~8GB for 3 images at 1280px)
- File Glob `maxFiles` should default to 10 for binary mode
- LLM executor should warn if total base64 data exceeds 50MB
- Qwen API already handles OOM with automatic downscale retry

### Example Workflows

**Batch Image Analysis**:
```
File Glob (screenshots/*.png, binary) → LLM ("Describe each screenshot") → Output
```

**Multi-Document Comparison**:
```
File Glob (contracts/*.png, binary) → LLM ("Compare these contracts, highlight differences") → Output
```

**OCR Pipeline**:
```
File Glob (scans/*.jpg, binary) → LLM ("Extract all text from these scanned pages") → File Write
```

---

## LLM Session Mode (Priority: P1)

### Problem
Current LLM node uses `/chat/direct` (stateless). Each invocation starts fresh with no memory. For multi-file workflows, the LLM should accumulate context across calls.

### Design

Add `session` mode to LLM node config:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| sessionMode | enum | stateless | stateless / session |
| maxHistory | int | 20 | Max messages to retain in session |

**Stateless** (current): Uses `/chat/direct`. Each call is independent.

**Session**: Uses `/chat/{conversation_id}`. The workflow engine creates a unique conversation_id per workflow run. All LLM nodes with `sessionMode=session` in the same run share the conversation, building up context.

**UI Change**: Toggle in LLM node config panel. When "session" is selected, show "Max History" field.

**Engine Change**:
- Generate `workflow_conversation_id = format!("wf-{}-{}", workflow_id, run_timestamp)`
- Pass to LLM executor via `ExecutionContext`
- LLM executor calls `/chat/{conversation_id}` instead of `/chat/direct`

---

## Streaming Support (Priority: P2)

### Current
LLM node waits for full response before completing. UI shows "Running..." then jumps to "Done" with full output.

### Proposed
LLM node streams tokens to UI in real-time via WebSocket events.

**New events**:
- `workflow.node.streaming` — partial token received
- Payload: `{node_id, token, accumulated_text}`

**Engine change**:
- Use SSE/streaming endpoint instead of synchronous POST
- Sidecar already supports streaming for Ollama/local models
- Buffer tokens, emit events, finalize on completion

**UI change**:
- Node output preview updates incrementally during streaming
- Config panel output section shows live streaming text
- Spinner replaced with "Streaming..." + token count

**Deferred**: Requires async event streaming architecture in the DAG walker. Currently the walker is synchronous per-node — needs to become event-driven for streaming.

---

## Example Workflows

### Workflow A: Batch CSV Analysis
```
File Glob (*.csv) → Iterator → LLM (session) → Aggregator → Output
                                  ↑
                          "Analyze this CSV and
                           add to your running summary"
```
LLM accumulates knowledge across all CSV files, final output is comprehensive analysis.

### Workflow B: Code Security Audit
```
File Glob (**/*.py) → Iterator → LLM → Aggregator (concat) → Output
                                  ↑
                        "Review for OWASP vulnerabilities"
```
Each Python file gets individual security review, aggregated into single report.

### Workflow C: Multi-File Summarization
```
File Glob (*.txt) → Transform (concat all) → LLM → Output
```
Simpler approach: concat all file contents, send as single prompt. Works for small file sets.

### Workflow D: Image Batch Processing (Vision)
```
File Glob (*.png, binary) → Iterator → LLM (vision) → Aggregator → File Write
                                         ↑
                               "Extract text from this image"
```
OCR-like pipeline: read all images, extract text via vision LLM, write results.

---

## Implementation Plan

### Phase 4B.1: File Glob Node (1 session)
- [ ] Rust executor: `file_glob.rs` — glob crate, security checks, multi-file read
- [ ] UI node: `FileGlobNode.tsx` — directory, pattern, mode, recursive toggle
- [ ] Config panel fields
- [ ] 5 unit tests
- [ ] Wire to LLM and test with `/tmp/ai-studio-samples/*.csv`

### Phase 4B.2: Iterator + Aggregator Nodes (2 sessions)
- [ ] Design: subworkflow-based iteration (Option B)
- [ ] Rust: Iterator executor — splits array, spawns subworkflow per item
- [ ] Rust: Aggregator executor — collects results per strategy
- [ ] UI nodes for both
- [ ] Engine support for iteration boundaries
- [ ] 8 unit tests

### Phase 4B.3: LLM Session Mode (1 session)
- [ ] LLM executor: session vs stateless toggle
- [ ] Engine: generate workflow_conversation_id
- [ ] Sidecar: verify `/chat/{id}` handles multi-turn correctly
- [ ] UI: session toggle in LLM config
- [ ] Test: multi-file session workflow

### Phase 4B.4: Streaming (2 sessions — deferred)
- [ ] Sidecar: SSE streaming endpoint
- [ ] Engine: async token forwarding
- [ ] UI: live token rendering in node preview
- [ ] All providers: streaming support check

---

## Dependencies

| Feature | Depends On | Blocks |
|---------|-----------|--------|
| File Glob | File Read executor (done) | Iterator workflows |
| Iterator | Subworkflow executor (done) | Batch processing |
| Aggregator | Iterator node | Complete batch pipelines |
| LLM Session | Sidecar conversation API (done) | Multi-turn workflows |
| Streaming | SSE in sidecar + WS bridge | Real-time UX |

---

## Open Questions

1. **Iterator vs Transform**: For small file sets (<10), a Transform node that concatenates all files might be simpler than Iterator. Should we optimize for this common case?
2. **Error handling in Iterator**: If one item fails, should the whole workflow fail or continue with remaining items? (Camel uses `stopOnException` flag)
3. **Memory limits**: Large glob results could exhaust memory. Should we stream files one at a time through the Iterator instead of loading all into memory?
4. **File Watch trigger**: How does this fit into the current "run" model? Currently workflows are manually triggered. File Watch would need a persistent polling loop. Defer to Phase 5?

---

## Review Checklist

- [ ] Architecture review (Gemini/Antigravity) — focus on Iterator/Aggregator engine impact
- [ ] Code review (Codex/GPT) — focus on File Glob security, memory limits
- [ ] UX review — node layout, config fields, workflow templates
