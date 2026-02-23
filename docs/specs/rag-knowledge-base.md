# RAG Knowledge Base — Visual Retrieval-Augmented Generation

**Status**: DRAFT — peer reviewed (Gemini 3 Pro + GPT-5.2 Codex), fixes applied
**Phase**: 5A (next after streaming + rich output)
**Priority**: P0 — table stakes for production AI workflows, but our implementation must be differentiated
**Author**: AI Studio PM
**Date**: 2026-02-22
**Providers (first)**: Azure OpenAI, Local/OpenAI-Compatible (Qwen3-VL-8B at localhost:8003)
**Providers (later)**: Ollama, Google, Anthropic, Cohere, HuggingFace

---

## Problem Statement

RAG is the #1 production AI pattern. Every team building AI workflows needs: "give the LLM context from my documents." Today in AI Studio, the workaround is File Read → LLM context stuffing, which breaks when docs exceed the context window.

Every competitor (Dify, LangFlow, Flowise, n8n) already has RAG. A basic chunk-embed-search pipeline is commodity. To be a 10K-star feature, AI Studio's RAG must leverage what makes us unique: **visual pipeline transparency, Inspector-level observability, local-first architecture, and the node editor's educational power.**

### What competitors do (and we must match)
- Upload docs → auto-index → query with retrieval
- Multiple vector DB backends (Pinecone, Chroma, Qdrant)
- Embedding provider selection

### What competitors DON'T do (our differentiators)
1. **Visual RAG pipeline** — you SEE chunks flowing through nodes, not a black box
2. **Source citations with line numbers** — click a citation, see the exact source
3. **Inspector for RAG** — debug WHY a retrieval failed (scores, chunk matches, embedding distances)
4. **Live re-indexing** — edit a doc → index auto-updates → next query uses new content
5. **Zero-server local index** — no Chroma/Pinecone/Docker. A folder + binary file. Works offline.
6. **Workflow-scoped knowledge** — each workflow has its own knowledge base, not a global system

---

## Architecture Overview

### Two-Tier Design

**Tier 1: Knowledge Base Node (80% of users)**
Single node. Configure a folder, it handles everything. Point and shoot.

**Tier 2: Individual RAG Nodes (power users)**
Separate Chunker, Embedding, Index Store, Index Search nodes for custom pipelines.
These are the building blocks that the Knowledge Base node uses internally.
Available for power users who want fine-grained control.

Both tiers share the same underlying engine. Tier 1 is sugar over Tier 2.

### Layer Responsibilities

| Piece | Layer | Why |
|-------|-------|-----|
| Text chunking | **Rust** | Pure text splitting, zero deps, fast, no Python needed |
| Embedding API calls | **Sidecar (Python)** | Already has provider clients, async HTTP, streaming |
| Index storage (write) | **Rust** | Binary file I/O, efficient serialization |
| Vector search (read) | **Rust** | Cosine similarity, pure Rust, fast, no native deps |
| Index management | **Rust** | Timestamp checking, incremental updates, file watching |
| Citation formatting | **Rust** | Attach source metadata to search results |

### Data Flow

```
INDEXING (automatic on first run, incremental on subsequent runs):

  Docs Folder ──→ Rust: scan files ──→ Rust: chunk text
                                            ↓
                  Sidecar: POST /embed ←────┘
                       ↓
                  Rust: write vectors + chunks to .ai-studio-index/

QUERYING (every workflow run):

  Query text ──→ Sidecar: POST /embed (single query)
                       ↓
                 Rust: cosine similarity against index
                       ↓
                 Top-K results with {text, score, source_file, line_range}
                       ↓
                 Format as context string with citations
```

---

## Tier 1: Knowledge Base Node

### The User Experience

1. Drag `Knowledge Base` onto canvas
2. Set `Docs Folder` to `~/my-docs/`
3. Connect: `Input → Knowledge Base → LLM → Output`
4. Click Run
5. First run: node badge shows "Indexing 47 files..." → "Indexed 312 chunks" → then answers query
6. Next run: "Index fresh" → instant search → answer with citations

### Config Panel

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| docsFolder | string | (required) | Path to the folder of documents to index |
| indexLocation | string | `{docsFolder}/.ai-studio-index/` | Where to store the index. Defaults to inside the docs folder. |
| embeddingProvider | enum | `azure_openai` | Which provider for embeddings |
| embeddingModel | string | `text-embedding-3-small` | Embedding model name |
| chunkSize | int | 500 | Target characters per chunk |
| chunkOverlap | int | 50 | Overlap between adjacent chunks |
| chunkStrategy | enum | `recursive` | `fixed_size`, `sentence`, `paragraph`, `recursive` |
| topK | int | 5 | Number of results to return per query |
| scoreThreshold | float | 0.0 | Minimum similarity score (0.0 = return all top-K) |
| autoReindex | bool | true | Re-index when file timestamps change |
| fileTypes | string | `*.md,*.txt,*.rs,*.py,*.ts,*.js,*.json,*.yml,*.yaml,*.csv,*.toml,*.go,*.java` | Glob pattern for files to include. Safe default excludes binary/secret files. |
| maxFileSize | int | 10 | Max file size in MB (skip larger files) |
| label | string | — | Custom node label (e.g., "Deploy Docs") |

### Input Handles

| Handle | Type | Description |
|--------|------|-------------|
| query | text | The search query (required) |
| folder | text | (Optional) Override docsFolder from upstream node |

### Output Handles

| Handle | Type | Description |
|--------|------|-------------|
| context | text | Formatted text with citations, ready to inject into LLM prompt |
| results | json | Structured array of `{text, score, source, lineStart, lineEnd, chunkId}` |
| indexStats | json | `{fileCount, chunkCount, indexSize, lastIndexed, staleFiles}` |

### Context Output Format

The `context` handle produces a formatted string optimized for LLM injection:

```
Relevant context from your knowledge base:

---
[Source: services/auth-service.md, lines 23-45, score: 0.92]
The auth service uses JWT tokens with a 15-minute expiry. Refresh tokens
are stored in Redis with a 7-day TTL. The /auth/refresh endpoint...

---
[Source: runbooks/deploy-checklist.md, lines 5-18, score: 0.87]
Deployment steps for auth-service:
1. Run database migrations: `make migrate`
2. Deploy to staging first: `gh workflow run deploy.yml -f env=staging`
3. Verify health check: curl https://staging.api.example.com/health
...

---
[Source: services/auth-service.md, lines 67-82, score: 0.81]
Error handling: The auth service returns 401 for expired tokens and 403
for insufficient permissions. Rate limiting is set to 100 req/min per IP.
```

### Node Badge Display

The node shows live status on the canvas:

| State | Badge |
|-------|-------|
| No index yet | `⚠ Not indexed` |
| Indexing | `⟳ Indexing 23/47 files...` |
| Index fresh | `✓ 312 chunks · 47 files` |
| Stale (files changed) | `↻ 3 files changed` |
| Searching | `⟳ Searching...` |
| Results | `✓ 5 results (best: 0.92)` |
| Error | `✗ Embedding failed` |

### Execution Flow (detailed)

```
execute_knowledge_base(node_config, query):

  1. RESOLVE index path
     index_dir = node_config.indexLocation
                 ?? format!("{}/.ai-studio-index", node_config.docsFolder)

  2. ACQUIRE cross-process lock
     lock = flock(index_dir/.lock, EXCLUSIVE)  // blocks if another workflow is indexing

  3. CHECK index freshness
     if index_dir/meta.json exists:
       meta = read meta.json
       if meta.embeddingModel != node_config.embeddingModel:
         → FULL re-index (model changed, vectors incompatible)

       // Detect ALL changes: new, modified, AND deleted files
       disk_files = glob docsFolder with fileTypes filter
       indexed_files = meta.indexedFiles keys
       stale_files = disk_files where mtime > meta.indexedFiles[file].modified
       new_files = disk_files NOT IN indexed_files
       deleted_files = indexed_files NOT IN disk_files

       if stale_files.is_empty() && new_files.is_empty() && deleted_files.is_empty():
         → SKIP to step 6 (index is fresh)
       else:
         // MVP: FULL REBUILD on any change (simple, correct, no tombstone complexity)
         // Phase 2: per-file shards with tombstones for true incremental updates
         → FULL re-index (step 4)
     else:
       → FULL index (step 4, all files)

  4. INDEX (full rebuild)
     files = glob docsFolder with fileTypes filter
     // Exclude .ai-studio-index/ directory and deny-listed paths (reuse is_path_denied)
     for each file:
       a. Read file content (skip if > maxFileSize)
       b. Normalize line endings (CRLF → LF)
       c. Chunk text using chunkStrategy (with hard cap per chunk)
          Each chunk = {text, source_file, line_start, line_end, byte_start, byte_end, chunk_id}
       d. Emit workflow.node.streaming event: "Indexing {filename}..."

     all_chunks = collect all chunks from all files

     e. Call sidecar POST /embed with all chunk texts (batched)
        → Returns array of vectors (one per chunk)

     f. Normalize all vectors (L2 normalize each vector for pre-normalized storage)

     g. ATOMIC WRITE to index_dir:
        // Write to temp directory first, then rename — crash-safe
        temp_dir = index_dir/.tmp-{uuid}/
        Write temp_dir/meta.json
        Write temp_dir/chunks.jsonl
        Write temp_dir/offsets.bin (u64 byte offset per chunk for O(1) lookup)
        Write temp_dir/vectors.bin (with magic + version header)
        // Atomic swap: rename old dir, rename temp → real, delete old
        rename(index_dir/meta.json, index_dir/meta.json.old)  // backup
        rename(temp_dir/*, index_dir/)                         // atomic per file
        delete temp files

  5. EMIT indexing complete event
     workflow.node.streaming: "Indexed {chunk_count} chunks from {file_count} files"

  6. SEARCH
     a. Call sidecar POST /embed with [query] → query_vector
     b. Normalize query_vector (L2)
     c. Load vectors.bin (mmap via memmap2 for large indexes)
     d. Validate vectors.bin header (magic, version, dims * count * 4 + 16 == file_len)
     e. Dot product: query_vector vs all stored vectors (pre-normalized = cosine)
     f. Use BinaryHeap (min-heap) to maintain top-K without full sort
     g. Filter by scoreThreshold
     h. Load corresponding chunks from chunks.jsonl via offsets.bin (O(1) seek)
     i. Format context string with citations
     j. Return {context, results, indexStats}

  7. RELEASE lock
```

---

## Tier 2: Individual RAG Nodes (Power Users)

For users who want to build custom RAG pipelines — different chunking per file type, multiple embedding models, custom retrieval logic.

### 2a. Text Chunker Node

Splits text into overlapping chunks with source metadata.

**Config**:
| Field | Type | Default | Description |
|-------|------|---------|-------------|
| strategy | enum | `recursive` | `fixed_size`, `sentence`, `paragraph`, `recursive` |
| chunkSize | int | 500 | Target characters per chunk |
| overlap | int | 50 | Overlap between adjacent chunks |

**Input**: `text` (document content), `source` (optional: source filename for metadata)
**Output**: `chunks` (json array of `{text, index, lineStart, lineEnd, source}`)

**Chunking Strategies**:
- `fixed_size`: Split at chunkSize boundaries, respecting word boundaries
- `sentence`: Split on sentence endings (`.` `!` `?` followed by space/newline)
- `paragraph`: Split on double newlines (`\n\n`)
- `recursive`: Try paragraph → sentence → fixed_size, keeping chunks near target size. Best for mixed content. This is the default because it handles most document types well.

**Implementation**: Pure Rust. No external dependencies.

### 2b. Embedding Node

Converts text chunks into vector embeddings via API.

**Config**:
| Field | Type | Default | Description |
|-------|------|---------|-------------|
| provider | enum | `azure_openai` | Embedding provider |
| model | string | `text-embedding-3-small` | Embedding model |
| batchSize | int | 100 | Chunks per API call |

**Input**: `texts` (json array of strings, or single text string)
**Output**: `vectors` (json array of `{text, vector, index}`)

**Supported providers** (Phase 1):
| Provider | Model | Dimensions | Notes |
|----------|-------|-----------|-------|
| Azure OpenAI | text-embedding-3-small | 1536 | Office demo default |
| Azure OpenAI | text-embedding-3-large | 3072 | Higher quality |
| Local/OpenAI-Compatible | any | varies | e.g., Qwen, nomic-embed-text via localhost |

**Supported providers** (Phase 2):
| Provider | Model | Dimensions | Notes |
|----------|-------|-----------|-------|
| OpenAI | text-embedding-3-small/large | 1536/3072 | Direct API |
| Ollama | nomic-embed-text, mxbai-embed-large | 768/1024 | Local, CPU-friendly |
| Google | text-embedding-004 | 768 | Vertex AI |
| Cohere | embed-v3.0 | 1024 | Multilingual |

**Sidecar endpoint**: `POST /embed`

```python
@app.post("/embed")
async def embed(request: EmbedRequest):
    """
    request.texts: list[str]       — texts to embed
    request.provider: str          — provider name
    request.model: str             — model name
    Returns: list[list[float]]     — array of embedding vectors
    """
```

**Implementation**: Sidecar (Python). Reuses existing provider config from Settings.

### 2c. Index Store Node

Writes vectors + metadata to a persistent index on disk.

**Config**:
| Field | Type | Default | Description |
|-------|------|---------|-------------|
| indexPath | string | (required) | Directory path for the index |
| name | string | `default` | Index name (allows multiple indexes in same dir) |
| mode | enum | `upsert` | `upsert` (add/update), `rebuild` (delete + recreate) |

**Input**: `vectors` (json — from Embedding node), `chunks` (json — from Chunker node)
**Output**: `stats` (json — `{chunkCount, fileSize, lastUpdated}`)

**Implementation**: Rust. Binary vector storage + JSONL chunk metadata.

### 2d. Index Search Node

Queries an existing index with a vector and returns top-K matches.

**Config**:
| Field | Type | Default | Description |
|-------|------|---------|-------------|
| indexPath | string | (required) | Directory path of the index |
| name | string | `default` | Index name |
| topK | int | 5 | Number of results |
| scoreThreshold | float | 0.0 | Minimum similarity score |

**Input**: `query` (text — will be embedded automatically), or `vector` (json — pre-computed vector)
**Output**: `results` (json), `context` (text — formatted with citations)

**Implementation**: Rust. Loads vectors via mmap, cosine similarity, returns results with metadata.

---

## Index Storage Format

### Directory Structure

```
~/my-docs/
├── services/
│   ├── auth-service.md
│   └── gateway.md
├── runbooks/
│   └── deploy.md
└── .ai-studio-index/              ← created by Knowledge Base node
    ├── meta.json                   ← index metadata
    ├── chunks.jsonl                ← text chunks with source metadata
    ├── offsets.bin                 ← byte offsets into chunks.jsonl (u64 LE per chunk, O(1) lookup)
    ├── vectors.bin                 ← binary f32 embedding vectors (pre-normalized)
    └── .lock                       ← cross-process lock file (flock/fs2)
```

### meta.json

```json
{
  "version": 1,
  "embeddingProvider": "azure_openai",
  "embeddingModel": "text-embedding-3-small",
  "dimensions": 1536,
  "chunkSize": 500,
  "chunkOverlap": 50,
  "chunkStrategy": "recursive",
  "fileCount": 3,
  "chunkCount": 47,
  "totalChars": 23500,
  "indexedFiles": {
    "services/auth-service.md": { "modified": "2026-02-22T10:30:00Z", "chunks": 15 },
    "services/gateway.md": { "modified": "2026-02-22T09:15:00Z", "chunks": 12 },
    "runbooks/deploy.md": { "modified": "2026-02-21T14:00:00Z", "chunks": 20 }
  },
  "lastIndexed": "2026-02-22T12:00:00Z",
  "indexSizeBytes": 287232
}
```

### chunks.jsonl

One JSON object per line:

```json
{"id": 0, "text": "The auth service uses JWT tokens...", "source": "services/auth-service.md", "lineStart": 23, "lineEnd": 45, "byteStart": 1200, "byteEnd": 1700}
{"id": 1, "text": "Refresh tokens are stored in Redis...", "source": "services/auth-service.md", "lineStart": 46, "lineEnd": 62, "byteStart": 1650, "byteEnd": 2150}
```

### vectors.bin

Binary format (little-endian throughout):

```
[magic: u32 = 0x52414756 ("RAGV")]  ← 4 bytes, identifies file type
[version: u32 = 1]                   ← 4 bytes, format version
[dimensions: u32]                    ← 4 bytes, embedding dimensions (e.g., 1536)
[count: u32]                         ← 4 bytes, number of vectors
[f32 * dimensions * count]           ← float region, row-major
```

- **Endianness**: All values little-endian (LE). Both ARM64 (macOS) and x86_64 (Linux) are natively LE.
- **Alignment**: Float region starts at byte offset 16 (4-byte aligned). Use `memmap2` crate for mmap.
- **Validation on load**: Verify magic + version, then check `16 + dims * count * 4 == file_len`. Reject if mismatch (corruption detection).
- **Vectors are pre-normalized** at index time. This means search is a simple dot product (no per-vector norm computation needed), improving both speed and numerical stability.
- **mmap safety**: Use `memmap2::MmapOptions`, gate `f32` slice casting on alignment checks. Never cast unaligned bytes.

This format supports memory-mapped reads — the OS handles paging, we don't load the entire file into memory for large indexes.

**Alternative considered**: `.npy` or `safetensors`. Custom format chosen for simplicity and because we control both reader and writer. The magic + version header allows future migration.

### .gitignore

The Knowledge Base node automatically creates `.ai-studio-index/.gitignore`:

```
# AI Studio RAG index — auto-generated, do not commit
*
```

---

## Sidecar API Changes

### New Endpoint: POST /embed

```python
class EmbedRequest(BaseModel):
    texts: list[str]             # 1 to N texts to embed
    provider: str                # "azure_openai", "local", etc. (matches Settings provider IDs)
    model: str                   # "text-embedding-3-small", etc.

class EmbedResponse(BaseModel):
    vectors: list[list[float]]   # one vector per input text
    model: str                   # model used
    dimensions: int              # vector dimensions
    usage: dict                  # {prompt_tokens, total_tokens}

@app.post("/embed")
async def embed(request: EmbedRequest) -> EmbedResponse:
    client = create_embedding_client(request.provider)  # separate from AgentProvider
    vectors = await client.embed(request.texts, request.model)
    return EmbedResponse(vectors=vectors, ...)
```

**Note**: Embedding uses a **separate client** from the chat `AgentProvider` (which only has `chat()`/`stream()`). The embedding client reads the same Settings config (api_key, base_url, extra_config) but calls the embeddings API. This follows the same per-request config pattern as `/chat/direct`.

**Provider ID standardization**: Use the same IDs everywhere:
- `azure_openai` (not `azure`, not `azureopenai`)
- `local` (not `local_openai`, not `local_openai_compatible`)
- `openai`, `google`, `anthropic`, `ollama`

### Provider embed() implementations

**Azure OpenAI**:
```python
async def embed(self, texts: list[str], model: str) -> list[list[float]]:
    response = await self.client.embeddings.create(
        input=texts,
        model=model,   # deployment name in Azure
    )
    return [item.embedding for item in response.data]
```

**Local/OpenAI-Compatible**:
```python
async def embed(self, texts: list[str], model: str) -> list[list[float]]:
    # Same OpenAI API format, different base_url
    response = await httpx.post(
        f"{self.base_url}/embeddings",
        json={"input": texts, "model": model},
    )
    return [item["embedding"] for item in response.json()["data"]]
```

### Batching + Token Safety

For large document sets, the sidecar batches embedding calls:
- Azure OpenAI: max 2048 texts per call, max 8191 tokens per text
- Local: depends on server, default batch of 32
- Sidecar handles retry with exponential backoff on rate limits

**Per-input token validation**: Before sending to the embedding API:
1. Estimate token count per text (heuristic: `len(text) / 4` for English, `len(text) / 2` for CJK)
2. If any text exceeds the model's token limit (8191 for text-embedding-3), **truncate** it and log a warning (do NOT fail the entire batch)
3. Return truncation warnings in the response metadata so the caller knows which chunks were affected

**Partial batch failure handling**: If a batch call fails (rate limit, server error):
1. Retry with exponential backoff (3 attempts)
2. If a specific input causes the error (malformed text), skip it and return a null vector + error message for that index
3. Never fail the entire indexing operation because of one bad chunk

---

## Rust Changes

### New Module: `src/workflow/executors/knowledge_base.rs`

Implements the Knowledge Base node executor:

```rust
pub struct KnowledgeBaseExecutor;

impl KnowledgeBaseExecutor {
    /// Follows the same pattern as LlmExecutor::execute and FileGlobExecutor::execute:
    /// - Input resolution: incoming > config (resolve_template for interpolation)
    /// - Sidecar calls via ctx.sidecar_url + reqwest
    /// - Event emission via emit_workflow_event (knowledge.index.*, knowledge.search.*)
    pub async fn execute(
        ctx: &ExecutionContext,
        node_id: &str,
        node_data: &Value,
        incoming: &HashMap<String, Value>,
    ) -> Result<NodeOutput> {
        let config = parse_kb_config(node_data, incoming);  // incoming > config
        let query = resolve_input("query", incoming, node_data)?;

        // 1. Ensure index is fresh
        let index_dir = resolve_index_dir(&config);
        let index_status = check_index_freshness(&index_dir, &config)?;

        match index_status {
            IndexStatus::Missing => full_index(&config, &ctx).await?,
            IndexStatus::Stale(files) => incremental_index(&config, &ctx, files).await?,
            IndexStatus::Fresh => { /* skip */ }
            IndexStatus::ModelChanged => full_index(&config, &ctx).await?,
        }

        // 2. Search
        let results = search_index(&index_dir, &query, &config, &ctx).await?;

        // 3. Format outputs
        let context = format_context_with_citations(&results);
        let stats = read_index_stats(&index_dir)?;

        Ok(NodeOutput {
            output: json!({
                "context": context,
                "results": results,
                "indexStats": stats,
            }),
            handles: HashMap::from([
                ("context".into(), json!(context)),
                ("results".into(), json!(results)),
                ("indexStats".into(), json!(stats)),
            ]),
        })
    }
}
```

### New Module: `src/workflow/rag/`

Shared RAG engine used by both Knowledge Base node and individual Tier 2 nodes.

```
src/workflow/rag/
├── mod.rs           — public API
├── chunker.rs       — text chunking strategies (recursive, sentence, paragraph, fixed)
├── index.rs         — index read/write (meta.json, chunks.jsonl, offsets.bin, vectors.bin)
│                      Atomic writes (temp dir + rename), cross-process lock (fs2 crate)
├── search.rs        — dot product on pre-normalized vectors, BinaryHeap top-K, mmap via memmap2
└── format.rs        — citation formatting for context output
```

### Chunking (Pure Rust)

```rust
pub enum ChunkStrategy {
    FixedSize,
    Sentence,
    Paragraph,
    Recursive,
}

pub struct Chunk {
    pub id: usize,
    pub text: String,
    pub source: String,
    pub line_start: usize,
    pub line_end: usize,
    pub byte_start: usize,  // byte offset (not char offset — avoids ambiguity)
    pub byte_end: usize,
}

pub fn chunk_text(
    content: &str,
    source: &str,
    strategy: ChunkStrategy,
    chunk_size: usize,
    overlap: usize,
) -> Vec<Chunk>
```

**Chunking safety requirements**:
- **UTF-8 safe splitting**: Always split at char boundaries (use `char_indices()`). Never slice mid-codepoint.
- **CRLF handling**: Normalize `\r\n` → `\n` before chunking. Adjust byte offsets to map back to original.
- **CJK punctuation**: Sentence strategy must recognize `。！？` (CJK period, exclamation, question) in addition to ASCII `.!?`.
- **Abbreviation awareness**: Sentence splitter must not split on common abbreviations (e.g., "Dr.", "U.S.", "etc."). Use a simple heuristic: don't split on `.` preceded by a single uppercase letter or known abbreviation.
- **Hard cap per chunk**: Clamp every chunk to `max(chunk_size * 2, 2000)` characters to prevent overlong inputs to the embed endpoint.
- **Overlap constraint**: Enforce `overlap < chunk_size`. Clamp if user provides invalid value.
- **Line number computation**: Precompute line-start byte offsets once per file. Derive `lineStart/lineEnd` from chunk byte range via binary search on the offset array.

### Vector Search (Pure Rust)

```rust
pub struct SearchResult {
    pub chunk_id: usize,
    pub score: f32,
    pub text: String,
    pub source: String,
    pub line_start: usize,
    pub line_end: usize,
}

/// Since vectors are pre-normalized at index time, cosine similarity
/// reduces to a simple dot product. This is both faster and more
/// numerically stable than computing norms at query time.
pub fn dot_similarity(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

/// Normalize a vector in-place. Called once at index time per vector,
/// and once at query time for the query vector.
pub fn normalize(v: &mut [f32]) {
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 { v.iter_mut().for_each(|x| *x /= norm); }
}

/// Search uses a BinaryHeap (min-heap by score) to maintain top-K
/// without sorting all results. For 50K vectors this avoids a full sort.
/// Consider spawn_blocking/Rayon for CPU-bound loop on large indexes.
pub fn search(
    query_vector: &[f32],  // must be pre-normalized
    index_dir: &Path,
    top_k: usize,
    threshold: f32,
) -> Result<Vec<SearchResult>>
```

---

## Tauri IPC Commands

### New Commands

```rust
#[tauri::command]
pub async fn index_folder(
    docs_folder: String,
    index_location: Option<String>,
    embedding_provider: String,
    embedding_model: String,
    chunk_size: Option<usize>,
    chunk_overlap: Option<usize>,
    chunk_strategy: Option<String>,
    file_types: Option<String>,
) -> Result<IndexStats, AppError>

#[tauri::command]
pub async fn search_index(
    index_location: String,
    query: String,
    top_k: Option<usize>,
    score_threshold: Option<f32>,
    embedding_provider: String,
    embedding_model: String,
) -> Result<Vec<SearchResult>, AppError>

#[tauri::command]
pub async fn get_index_stats(
    index_location: String,
) -> Result<IndexStats, AppError>

#[tauri::command]
pub async fn delete_index(
    index_location: String,
) -> Result<(), AppError>
```

These commands are also used by the UI for index management (see Settings section).

---

## UI Changes

### Knowledge Base Node (Canvas)

```
+─────────────────────────────────────+
│  KNOWLEDGE BASE                     │
│  "Deploy Docs"                      │
│                                     │
│  📁 ~/projects/deploy-docs/         │
│  🔢 312 chunks · 47 files           │
│                                     │
│  [query] ──────────→ [context]      │
│                      [results]      │
+─────────────────────────────────────+
```

### Config Panel

Standard config panel layout matching existing nodes:
- **Docs Folder**: text input with folder picker button
- **Index Location**: text input (auto-filled, editable)
- **Embedding**: provider dropdown + model text input (same pattern as LLM node)
- **Chunking**: collapsible section with strategy, size, overlap
- **Search**: topK, scoreThreshold
- **Index Status**: read-only display of current index stats
- **Actions**: [Re-index Now] button, [Delete Index] button

### RichOutput Citations (Enhancement)

When RichOutput renders text containing citation markers like `[Source: file.md, lines 23-45, score: 0.92]`, it renders them as styled citation blocks:

```
┌─ auth-service.md:23-45 ─────────── score: 0.92 ─┐
│ The auth service uses JWT tokens with a 15-minute │
│ expiry. Refresh tokens are stored in Redis...      │
└────────────────────────────────────────────────────┘
```

This is a RichOutput enhancement, not a new component. Detection: regex for `[Source: ...]` pattern in the text mode/markdown mode.

### Settings: Index Management

New section in Settings page: **Knowledge Bases**

Shows all indexes found in `~/.ai-studio/` and any workflow-referenced index directories:

| Name | Location | Files | Chunks | Size | Last Indexed | Actions |
|------|----------|-------|--------|------|-------------|---------|
| Deploy Docs | ~/deploy-docs/.ai-studio-index/ | 47 | 312 | 2.3 MB | 2 min ago | [Re-index] [Delete] |
| API Specs | ~/api-specs/.ai-studio-index/ | 12 | 89 | 0.8 MB | 1 hour ago | [Re-index] [Delete] |

---

## Workflow Events

### New Event Types

```
knowledge.index.started    — {docsFolder, fileCount}
knowledge.index.progress   — {file, filesProcessed, filesTotal, chunksTotal}
knowledge.index.completed  — {fileCount, chunkCount, durationMs, indexSize}
knowledge.index.error      — {error, file?}

knowledge.search.started   — {query, topK}
knowledge.search.completed — {query, resultCount, bestScore, durationMs}
```

These events are emitted during workflow execution and appear in the Inspector timeline, giving full visibility into RAG performance.

### Inspector Integration

The Inspector shows RAG events with:
- **Index events**: file-by-file progress, total chunks, duration
- **Search events**: query text, result count, best score, latency
- **Retrieval quality**: score distribution across results (are we finding relevant content?)

---

## Bundled Templates

### Template: Knowledge Q&A (the 30-second demo)

```
[Input "question"] → [Knowledge Base ~/my-docs/] →context→ [LLM "Answer from docs"] → [Output]
```

4 nodes. User sets the folder, asks a question, gets an answer with citations. First run indexes automatically.

### Template: Smart Deployer with RAG (upgraded)

```
[Knowledge Base ~/deploy-docs/] →context→
                                          ↘
[Input "deploy auth to staging"] ──────→ [LLM "Plan Builder"] → [Approval] → [Iterator] → [Shell Exec] → [Output]
```

The LLM has full knowledge of all service configs, runbooks, and deployment docs — no matter how many there are.

### Template: Codebase Explorer

```
[Knowledge Base ~/my-project/src/] →context→
                                             ↘
[Input "How does auth work?"] ──────────→ [LLM "Code Expert"] → [Output]
```

Index an entire codebase. Ask questions about how things work. Source citations point to exact files and line numbers.

---

## Supported File Types

| Extension | How it's read | Chunking notes |
|-----------|--------------|----------------|
| `.md`, `.txt` | Direct text | Standard chunking |
| `.py`, `.js`, `.ts`, `.rs`, `.java`, `.go` | Direct text | Chunks respect function boundaries where possible |
| `.json`, `.yaml`, `.yml`, `.toml` | Direct text | Smaller chunks (config files are dense) |
| `.csv` | Direct text | Row-aware chunking (don't split mid-row) |
| `.pdf` | Python extraction (sidecar) | Requires `pymupdf` or `pdfplumber` |
| `.docx` | Python extraction (sidecar) | Requires `python-docx` |

Phase 1: `.md`, `.txt`, code files, `.json`, `.yaml`, `.csv`
Phase 2: `.pdf`, `.docx` (sidecar extraction endpoint)

---

## Performance Considerations

### Index Size Estimates

| Docs | Chunks (~500 chars) | Vectors (1536-dim f32) | Index Size |
|------|---------------------|----------------------|------------|
| 10 files, 50 pages | ~500 | 3 MB | ~4 MB |
| 100 files, 500 pages | ~5,000 | 30 MB | ~35 MB |
| 1,000 files, 5,000 pages | ~50,000 | 300 MB | ~350 MB |

For 50K+ chunks, consider:
- Memory-mapped vector reads (don't load all into RAM)
- Approximate nearest neighbor (HNSW) instead of brute-force cosine
- This is a Phase 2 optimization — brute-force is fine up to ~10K chunks

### Embedding API Costs

| Provider | Model | Cost per 1M tokens | 500-page corpus cost |
|----------|-------|--------------------|--------------------|
| Azure OpenAI | text-embedding-3-small | $0.02 | ~$0.005 |
| Azure OpenAI | text-embedding-3-large | $0.13 | ~$0.03 |
| Local (Qwen/Ollama) | any | $0.00 | Free |

Re-indexing cost is negligible. Incremental indexing only re-embeds changed files.

### Search Latency

- Dot product over 5,000 pre-normalized vectors (1536-dim) with BinaryHeap top-K: ~1-2ms on modern CPU
- Index load from SSD: ~10ms for 30MB vectors file (mmap, OS handles paging)
- Chunk fetch via offsets.bin: <1ms per chunk (seek, not scan)
- Embedding one query: ~100-500ms (API call)
- Total search latency: **~200-600ms** (dominated by embedding API call)
- For large indexes (>10K chunks): consider `spawn_blocking`/Rayon to avoid stalling async executor

---

## Security

- **Path containment**: Same security model as File Read / File Glob nodes. Denied paths list applies to docsFolder. Reuse `is_path_denied()` from File Glob executor.
- **Self-exclusion**: The indexer always excludes `.ai-studio-index/` from scanning to prevent indexing its own output.
- **Index write safety**: Index is only written to the configured indexLocation. Never writes outside it.
- **File permissions**: Create `.ai-studio-index/` directory with mode `0700` and all files within with mode `0600`. This prevents other OS users from reading potentially sensitive document chunks.
- **No data exfiltration**: Embedding API calls send text chunks to the configured provider. User controls which provider (local = nothing leaves the machine).
- **Sidecar auth**: `/embed` endpoint requires the same `x-ai-studio-token` header as all other sidecar endpoints.
- **.gitignore**: Auto-created in index directory to prevent accidental commit of binary index files.
- **Concurrent access**: Cross-process file lock (`.lock` file with `flock`/`fs2` crate) prevents two workflows from writing the same index simultaneously. Readers acquire shared locks; writers acquire exclusive locks.

---

## Implementation Plan

### Phase 1 (MVP — this spec)
- [ ] Sidecar: `/embed` endpoint (Azure OpenAI + Local) with batching, token validation, retry
- [ ] Rust: `rag/` module (chunker, index, search, format) with `memmap2` + `fs2` deps
- [ ] Rust: vectors.bin with magic/version/LE header + pre-normalized vectors
- [ ] Rust: offsets.bin for O(1) chunk lookup
- [ ] Rust: Atomic index writes (temp dir + rename) + cross-process lock
- [ ] Rust: UTF-8 safe chunking with CRLF handling, CJK punctuation, hard cap
- [ ] Rust: Knowledge Base node executor (matching NodeExecutor API pattern)
- [ ] Rust: Tauri IPC commands (index_folder, search_index, get_index_stats, delete_index)
- [ ] Rust: Deletion detection (diff disk files vs meta.indexedFiles → full rebuild)
- [ ] UI: Knowledge Base node (canvas + config panel, safe default fileTypes)
- [ ] UI: RichOutput citation rendering
- [ ] Security: index dir 0700, files 0600, exclude .ai-studio-index from scanning
- [ ] Template: Knowledge Q&A
- [ ] Template: Smart Deployer with RAG
- [ ] Template: Codebase Explorer
- [ ] Tests: chunker edge cases, index round-trip, mmap validation, /embed contract

### Phase 2 (Polish)
- [ ] True incremental indexing (per-file shards + tombstones, skip full rebuild)
- [ ] Tier 2 individual nodes (Chunker, Embedding, Index Store, Index Search)
- [ ] PDF + DOCX support (sidecar extraction)
- [ ] Settings: Index Management page
- [ ] Inspector: RAG event types + timeline rendering
- [ ] Live mode: auto re-index on file change (file watcher)
- [ ] HNSW approximate search for large indexes (>10K chunks)
- [ ] Additional embedding providers (Ollama, Google, Cohere)
- [ ] Rayon/spawn_blocking for CPU-bound search on large indexes

---

## Competitive Positioning

| Feature | Dify | LangFlow | Flowise | n8n | **AI Studio** |
|---------|------|----------|---------|-----|---------------|
| RAG pipeline | ✓ | ✓ | ✓ | ✓ | ✓ |
| Visual pipeline | ✗ (config UI) | ✓ | ✓ | ✓ | **✓ (canvas + node badges)** |
| Source citations | Basic | ✗ | ✗ | ✗ | **✓ (file:line + score)** |
| RAG in Inspector | ✗ | ✗ | ✗ | ✗ | **✓ (events, scores, debug)** |
| Zero-server index | ✗ (needs DB) | ✗ | ✗ | ✗ | **✓ (file-based, portable)** |
| Auto re-index | Manual | Manual | Manual | Manual | **✓ (timestamp-based)** |
| Local-first | Partial | Partial | ✗ | ✗ | **✓ (local embed + local index)** |
| Workflow-scoped KB | ✗ (global) | ✗ (global) | ✗ | ✗ | **✓ (per-workflow)** |
| Live re-index | ✗ | ✗ | ✗ | ✗ | **✓ (Phase 2)** |

**Our story**: "Every other tool hides RAG behind a config panel. AI Studio shows you the full pipeline — watch your docs get chunked, embedded, and searched. Debug retrieval quality in the Inspector. All local, no servers, zero setup."

---

## Test Strategy

### Rust Unit Tests (chunker)
- Empty file → returns zero chunks
- Single character → returns one chunk
- Unicode: CJK text, emojis, multi-byte characters → splits at char boundaries
- CRLF file → line numbers correct after normalization
- File with no paragraph breaks → falls through to sentence → fixed_size
- Huge file (10MB) → chunks don't exceed hard cap
- Overlap > chunkSize → clamped to chunkSize - 1
- Code file → chunks respect function-boundary heuristic (if implemented)

### Rust Unit Tests (index)
- Write/read round-trip: chunks.jsonl + offsets.bin + vectors.bin
- vectors.bin magic + version validation (reject bad magic)
- vectors.bin length validation (reject truncated file = crash safety)
- mmap alignment check passes on valid file
- offsets.bin enables O(1) chunk lookup (verify byte offset correctness)

### Rust Unit Tests (search)
- Pre-normalized vectors: dot product matches expected cosine similarity
- Top-K heap returns correct top results (not just sorted)
- Score threshold filtering works
- Empty index → returns empty results (not error)
- Query vector with zero norm → returns score 0.0 for all

### Sidecar Tests (/embed endpoint)
- Single text embedding → returns vector of correct dimensions
- Batch of 100 texts → returns 100 vectors
- Overlong text (>8191 tokens) → truncated, warning returned
- Invalid provider → 400 error with clear message
- Auth token required (no token → 401)

### Integration Tests
- Full pipeline: write docs → index → search → verify citations have correct file + line
- Incremental: index, delete a file, re-index → deleted file's chunks are gone
- Model change: re-index with different model → full rebuild triggered
- Concurrent lock: two index operations on same dir → second blocks, not corrupts

---

## Open Questions

1. Should the Knowledge Base node support a "manual index" mode where you explicitly trigger indexing vs auto-index on run?
2. Do we need a dedicated "PDF Reader" node, or should the Knowledge Base handle PDF extraction internally?
3. For the Settings index management page — should we show all `.ai-studio-index/` directories found on disk, or only ones referenced by saved workflows?
4. Should citation format be configurable (compact vs verbose) or is the default format sufficient?
