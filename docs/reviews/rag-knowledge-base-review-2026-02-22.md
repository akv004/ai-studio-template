# RAG Knowledge Base Review
**Date**: 2026-02-22
**Reviewer**: Gemini 3 Pro
**Status**: RESOLVED — all findings accepted and applied to spec

### Findings Table
| Check | Priority | Verdict | Findings |
|-------|----------|---------|----------|
| Storage format | HIGH | FAIL | Flat `vectors.bin` with `chunks.jsonl` does not support in-place deletions. Since vectors are stored contiguously, updating an existing file or removing a deleted file means there's no efficient way to clear its old chunks, leading to ghost context or corrupted index state. |
| Two-tier design | LOW | PASS | The split is an excellent product decision. Tier 1 serves the "just make it work" users (80%), while Tier 2 respects the node-based visual philosophy for power users building customized processing workflows. |
| Incremental indexing | HIGH | FAIL | Timestamps alone are brittle and the spec ignores file trashing entirely. If a file in `meta.json`'s `indexedFiles` is deleted or moved, the current incremental logic won't detect it, allowing deleted data to remain queryable indefinitely. |
| Embedding API | MED | WARN | Hard token limits. Context windows vary heavily locally vs Cloud (e.g. 8191 on `text-embedding-3`). The Python sidecar or the API should proactively warn or truncate if a single chunk's token length exceeds provider limits. |
| Competitive gaps | MED | PASS | Zero-server local indexing is a very strong, genuine differentiator (eschewing Docker/Pinecone). Visual RAG pipelines are cool, but LangFlow/Flowise also provide this; the **Inspector RAG trace/observability** is the actual killer feature. |
| Security | HIGH | WARN | Index files contain embedded vectors and plaintext chunks of user documents. `meta.json` and `chunks.jsonl` must have strict OS file permissions (e.g. `chmod 600`) upon creation so other users on a shared OS can't read potentially sensitive data. |
| Product/UX | LOW | PASS | The 4-node 30-second demo is highly compelling out of the box. Giving users a generalized `recursive` chunking strategy as the default provides a robust first-time experience. |
| Missing pieces | HIGH | FAIL | Concurrent read/write access. Since the app relies on flat files and mmap, multiple workflows reading simultaneously—or one workflow querying while another is indexing the same folder—will trigger torn reads or mmap crashes if the underlying file is truncated. File locks (e.g. `flock`) are required. |

### Actionable Checklist
- [x] Define the tombstone or rewrite mechanism for deleting old vectors/chunks when a file is modified/deleted during an incremental update. → **MVP: full rebuild on any change. Per-file shards deferred to Phase 2.** (2026-02-22, spec updated)
- [x] Add a deletion detection step in the incremental indexer (scan disk vs `meta.indexedFiles`). → **Added: diff disk_files vs indexed_files, detect new/stale/deleted.** (2026-02-22, spec updated)
- [x] Implement POSIX file locks (`flock`) across the `.ai-studio-index` directory so that index queries block safely during an index rebuild. → **Added: .lock file + fs2 crate, exclusive for writers, shared for readers.** (2026-02-22, spec updated)
- [x] Ensure the `.ai-studio-index` directory and its files are created with mode `0700`/`0600` (`rwx------`/`rw-------`). → **Added to Security section.** (2026-02-22, spec updated)
- [x] Enhance the sidecar's `/embed` endpoint to validate text lengths (token estimating) and gracefully emit errors or chunk sub-splits before passing to the API. → **Added: per-input token estimation + truncation + partial batch failure handling.** (2026-02-22, spec updated)

### Notes
The overall architecture is highly pragmatic. Relying on basic memory-mapping for vector search skips the operational nightmare of bundling entire vector databases like ChromaDB or Qdrant inside a local desktop app. However, storage engines exist for a reason—building a custom CRUD vector store on flat files exposes painful edge cases (primarily deletions and concurrency). Fix the deletion and concurrency flaws, and this will be an incredibly powerful local-first RAG implementation!
