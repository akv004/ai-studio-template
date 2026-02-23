# Peer Review: RAG Knowledge Base Spec

**Date**: 2026-02-22
**Project**: AI Studio (open-source IDE for AI agents)
**Reviewer**: Gemini 3 Pro
**Review type**: Architecture + Product Design

---

## Instructions for Reviewer

You are reviewing code in the workspace at `/home/amit/projects/myws/ws01/ai-studio-template`.
Read the files listed below, then provide your review in the output format specified.

## Context

AI Studio is a desktop-native IDE for AI agents built with Tauri 2 (Rust) + React 19 + Python FastAPI sidecar. Its flagship feature is a visual node editor ("Unreal Blueprints for AI agents") where users build AI pipelines by connecting nodes. We're designing RAG (Retrieval-Augmented Generation) support — the ability to point a workflow at a folder of documents and have the LLM answer questions using that knowledge. Every competitor (Dify, LangFlow, Flowise, n8n) already has basic RAG. Our implementation must be differentiated through visual transparency, local-first architecture, source citations, and Inspector-level observability.

## Scope

Review the RAG Knowledge Base spec (`docs/specs/rag-knowledge-base.md`) for:
- Architecture soundness (two-tier design, layer responsibilities, data flow)
- Product design (UX, competitive positioning, demoability)
- Storage format (index directory structure, binary vectors, incremental updates)
- API design (sidecar /embed endpoint, Tauri IPC commands)
- Missing features or edge cases
- Whether the differentiators are genuinely compelling vs competitors

## Files to Read

Read these files in this order:
1. `CLAUDE.md` — project context, architecture overview, build/run commands
2. `docs/specs/rag-knowledge-base.md` — **THE SPEC TO REVIEW** (full RAG design)
3. `docs/specs/killer-features.md` (lines 214-282) — original RAG sketch for comparison
4. `docs/specs/architecture.md` — 3-layer architecture (understand where RAG fits)
5. `docs/specs/eip-data-io-nodes.md` (first 60 lines) — existing node spec format for consistency
6. `apps/desktop/src-tauri/src/workflow/executors/llm.rs` — existing LLM executor pattern (RAG executor should follow same pattern)
7. `apps/sidecar/providers/` — existing provider implementations (embedding will follow same pattern)
8. `docs/specs/streaming-output.md` — recently built streaming spec (for consistency of spec quality)

## What to Look For

1. **Storage format**: Is the binary vectors.bin + JSONL chunks + meta.json format sound? Are there scalability concerns? Would mmap work reliably on macOS + Linux? Is there a better format?

2. **Two-tier design**: Is the Knowledge Base (single node) + individual RAG nodes (Tier 2) split the right call? Or is it over-engineering? Would users actually use Tier 2 nodes, or should we just build Tier 1?

3. **Incremental indexing**: The spec proposes timestamp-based stale file detection. Is this robust enough? What about file renames, deletions, moved files? What about content changes that don't update mtime?

4. **Embedding provider design**: The sidecar `/embed` endpoint reuses existing provider config. Is the batching strategy correct? Are there rate limiting concerns for Azure OpenAI embeddings? Should we handle token limits per embedding call?

5. **Competitive gaps**: The spec claims 8 differentiators vs Dify/LangFlow/Flowise/n8n. Are these genuine? Are any claimed as unique when competitors actually have them? Are there differentiators we're missing?

6. **Security**: Docs folder path containment, embedding data sent to external APIs, index file permissions. Any risks the spec misses?

7. **Product/UX**: Is the "4 nodes, 30-second demo" compelling? Would a first-time user understand the Knowledge Base node? Is the config panel too complex or too simple? Are the bundled templates the right ones?

8. **Missing pieces**: What's not in the spec that should be? Multi-language support? Index versioning? Concurrent index access? Index sharing between workflows?

## Output Format

Provide your review as a markdown file saved to:
**`docs/reviews/rag-knowledge-base-review-2026-02-22.md`**

Use this structure:

### Header
```
# RAG Knowledge Base Review
**Date**: 2026-02-22
**Reviewer**: {Your model name}
**Status**: Draft
```

### Findings Table
| Check | Priority | Verdict | Findings |
|-------|----------|---------|----------|
| {area} | {HIGH/MED/LOW} | {PASS/FAIL/WARN} | {1-2 sentence finding} |

### Actionable Checklist
- [ ] {Action item 1}
- [ ] {Action item 2}

### Notes (optional)
Any architecture recommendations, praise, or broader observations.
