# Streaming Output

**Status**: PLANNED
**Phase**: 5A (demo-ready polish)
**Priority**: P1 — expected UX for any LLM tool
**Author**: AI Studio PM
**Date**: 2026-02-21
**Supersedes**: Brief streaming section in `eip-data-io-nodes.md`

---

## Problem Statement

When an LLM node executes, the UI shows a spinner for 5-60 seconds, then the entire response appears at once. This feels broken — every modern LLM interface shows tokens appearing in real-time. The lack of streaming is especially painful in Live Mode where users monitor continuous execution.

---

## Architecture

### Current Flow (synchronous)
```
UI → [Run] → Rust Engine → POST /chat/direct → Sidecar → Provider API
                                                    ↓ (wait 5-60s)
                                              Full response JSON
                                                    ↓
                                        workflow.node.completed event
                                                    ↓
                                            UI shows full output
```

### Streaming Flow (proposed)
```
UI → [Run] → Rust Engine → POST /chat/stream → Sidecar → Provider API
                                                    ↓ (token-by-token)
                                              SSE: data: {"token": "The"}
                                              SSE: data: {"token": " quarterly"}
                                              SSE: data: {"token": " revenue"}
                                              ...
                                              SSE: data: {"done": true}
                                                    ↓
                                        workflow.node.streaming events (batched)
                                                    ↓
                                            UI updates incrementally
                                                    ↓
                                        workflow.node.completed (final)
```

---

## Sidecar Changes

### New Endpoint: `POST /chat/stream`

Same request schema as `/chat/direct`, returns `StreamingResponse` with SSE format.

**Request** (identical to `/chat/direct`):
```json
{
  "provider": "ollama",
  "model": "qwen3:8b",
  "messages": [{"role": "user", "content": "Explain quantum computing"}],
  "temperature": 0.7
}
```

**Response** (SSE stream):
```
data: {"type": "token", "content": "Quantum", "index": 0}

data: {"type": "token", "content": " computing", "index": 1}

data: {"type": "token", "content": " is", "index": 2}

data: {"type": "done", "content": "Quantum computing is...", "usage": {"prompt_tokens": 12, "completion_tokens": 89, "total_tokens": 101}}
```

**Error mid-stream**:
```
data: {"type": "error", "message": "Provider connection lost", "partial": "Quantum computing is..."}
```

### Provider Changes

| Provider | Current | Change |
|----------|---------|--------|
| Ollama | `stream=False` | Set `stream=True`, yield chunks from async generator |
| Anthropic | Non-streaming | Use `client.messages.stream()`, yield `text_delta` events |
| OpenAI | Non-streaming | Use `stream=True` in `chat.completions.create()`, yield delta content |
| Google AI | Non-streaming | Use `generate_content(stream=True)`, yield chunks |

**Implementation pattern** (all providers):
```python
async def stream_chat(self, messages, **kwargs) -> AsyncGenerator[dict, None]:
    # Provider-specific streaming call
    async for chunk in provider_stream:
        yield {"type": "token", "content": chunk.text, "index": i}
    yield {"type": "done", "content": full_text, "usage": usage}
```

**FastAPI endpoint**:
```python
@app.post("/chat/stream")
async def chat_stream(req: ChatRequest):
    provider = get_provider(req.provider)
    async def generate():
        async for chunk in provider.stream_chat(req.messages, **req.params):
            yield f"data: {json.dumps(chunk)}\n\n"
    return StreamingResponse(generate(), media_type="text/event-stream")
```

---

## Rust Engine Changes

### Streaming Proxy in Sidecar Manager

New method alongside existing `proxy_request()`:

```rust
async fn proxy_request_stream(
    &self,
    path: &str,
    body: serde_json::Value,
    on_token: impl Fn(StreamChunk) + Send,
) -> Result<String> {
    let response = self.client
        .post(format!("{}{}", self.base_url, path))
        .header("x-ai-studio-token", &self.token)
        .json(&body)
        .send()
        .await?;

    let mut stream = response.bytes_stream();
    let mut accumulated = String::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        let text = String::from_utf8_lossy(&chunk);
        for line in text.lines() {
            if let Some(data) = line.strip_prefix("data: ") {
                let parsed: StreamChunk = serde_json::from_str(data)?;
                match &parsed {
                    StreamChunk::Token { content, .. } => {
                        accumulated.push_str(content);
                        on_token(parsed);
                    }
                    StreamChunk::Done { content, usage } => {
                        return Ok(content.clone());
                    }
                    StreamChunk::Error { message, partial } => {
                        return Err(anyhow!("Stream error: {}", message));
                    }
                }
            }
        }
    }
    Ok(accumulated)
}
```

### LLM Executor Changes

```rust
// In llm.rs executor
let streaming_enabled = node_config.get("streaming").unwrap_or(true);

if streaming_enabled {
    let result = ctx.sidecar.proxy_request_stream(
        "/chat/stream",
        request_body,
        |chunk| {
            // Emit streaming event (batched — every 3 tokens or 100ms)
            ctx.emit_event(WorkflowEvent::NodeStreaming {
                node_id: node_id.clone(),
                token: chunk.content,
                accumulated_length: chunk.index,
            });
        },
    ).await?;
    Ok(result)
} else {
    // Existing synchronous path (fallback)
    ctx.sidecar.proxy_request("POST", "/chat/direct", request_body).await
}
```

### Token Batching

Emitting a Tauri event per token is expensive. Batch tokens:
- Accumulate tokens in a buffer
- Flush every **3 tokens** or **100ms** (whichever comes first)
- Each flush emits one `workflow.node.streaming` event with the batch

---

## New Event Types

### `workflow.node.streaming`

```json
{
  "type": "workflow.node.streaming",
  "node_id": "llm_1",
  "run_id": "run_abc",
  "tokens": "Quantum computing is a ",
  "accumulated_length": 24,
  "ts": "2026-02-21T10:30:00.123Z"
}
```

### Updated `workflow.node.completed`

Add `usage` field to completion event:
```json
{
  "type": "workflow.node.completed",
  "node_id": "llm_1",
  "output": "Quantum computing is...",
  "usage": {
    "prompt_tokens": 12,
    "completion_tokens": 89,
    "cost_usd": 0.0003
  }
}
```

---

## UI Changes

### Node Output Preview (streaming state)

```
┌─────────────────────────────┐
│  LLM · Summarizer    ◉ ▊   │  ← blinking cursor during stream
│─────────────────────────────│
│  Quantum computing is a     │
│  fundamentally different    │
│  approach to computation    │
│  that leverages▊            │  ← text growing in real-time
│─────────────────────────────│
│  ⚡ Streaming · 24 tokens   │  ← live counter
└─────────────────────────────┘
```

After completion:
```
┌─────────────────────────────┐
│  LLM · Summarizer    ✓     │
│─────────────────────────────│
│  Quantum computing is a     │
│  fundamentally different    │
│  approach to computation... │
│─────────────────────────────│
│  ✓ 89 tokens · 1.2s · $0.0003 │
└─────────────────────────────┘
```

### Config Panel Output Section

- Live text area that updates during streaming
- Shows accumulated text with a cursor
- Token count + elapsed time live counter
- "Stop Generation" button (sends cancel signal)

### Cancel Support

- User clicks "Stop" → Rust drops the SSE connection → sidecar detects disconnect → stops generation
- Partial output is kept as the node's output (usable by downstream nodes)
- Node status: `completed_partial` (yellow check instead of green)

---

## LLM Node Config Addition

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| streaming | bool | true | Enable token streaming |

Streaming is **on by default**. Users can disable it per-node if they prefer the synchronous behavior (e.g., when output is consumed programmatically and streaming adds no value).

---

## Implementation Plan

### Phase 1: Sidecar SSE endpoint (1 session)
- [ ] FastAPI `StreamingResponse` endpoint at `/chat/stream`
- [ ] Ollama provider: `stream=True` + async generator
- [ ] 5 tests (stream, cancel, error mid-stream, empty response, timeout)

### Phase 2: Rust streaming proxy (1 session)
- [ ] `proxy_request_stream()` in sidecar manager
- [ ] Token batching (3 tokens / 100ms flush)
- [ ] LLM executor streaming mode
- [ ] `workflow.node.streaming` event emission
- [ ] 8 tests (batching, cancel, partial output, fallback to sync)

### Phase 3: UI rendering (1 session)
- [ ] OutputPreview streaming state (incremental text + cursor)
- [ ] Config panel live output area
- [ ] Token count + elapsed time counter
- [ ] "Stop Generation" button
- [ ] 3 E2E tests (stream renders, cancel works, completion shows stats)

### Phase 4: Remaining providers (1 session)
- [ ] Anthropic streaming
- [ ] OpenAI streaming
- [ ] Google AI streaming
- [ ] Provider capability detection (fallback to sync if provider doesn't support streaming)

---

## Performance Considerations

- Token batching prevents UI event flooding (max ~10 events/sec instead of ~50)
- SSE connection reuse: keep-alive between LLM nodes in same workflow run
- Memory: streaming doesn't buffer full response in multiple places — accumulate once in Rust, emit increments to UI
- Backpressure: if UI can't keep up, tokens still accumulate in Rust buffer (UI catches up on next batch)
