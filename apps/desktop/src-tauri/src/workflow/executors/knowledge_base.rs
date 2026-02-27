use super::{ExecutionContext, NodeExecutor, NodeOutput};
use crate::workflow::engine::{emit_workflow_event, resolve_template};
use crate::workflow::executors::file_read::is_path_denied;
use crate::workflow::rag::{
    ChunkStrategy, chunk_text, write_index, read_meta, check_freshness, search, normalize,
    format_context_with_citations, IndexMeta, IndexStatus,
    index::scan_docs, index::IndexedFileInfo,
};
use serde_json::Value;
use std::collections::HashMap;

/// Default file types for Knowledge Base indexing.
const DEFAULT_FILE_TYPES: &str = "*.md,*.txt,*.rs,*.py,*.ts,*.js,*.json,*.yml,*.yaml,*.csv,*.toml,*.go,*.java,*.pdf,*.docx,*.xlsx,*.pptx";

/// Binary document formats that require sidecar extraction.
const BINARY_EXTENSIONS: &[&str] = &["pdf", "docx", "xlsx", "xls", "pptx"];

pub struct KnowledgeBaseExecutor;

#[async_trait::async_trait]
impl NodeExecutor for KnowledgeBaseExecutor {
    fn node_type(&self) -> &str { "knowledge_base" }

    async fn execute(
        &self,
        ctx: &ExecutionContext<'_>,
        node_id: &str,
        node_data: &Value,
        incoming: &Option<Value>,
    ) -> Result<NodeOutput, String> {
        // --- Resolve config ---
        let config_folder = node_data.get("docsFolder").and_then(|v| v.as_str()).unwrap_or("");
        let docs_folder = if let Some(inc) = incoming {
            if let Some(obj) = inc.as_object() {
                obj.get("folder").and_then(|v| v.as_str()).unwrap_or(config_folder).to_string()
            } else {
                config_folder.to_string()
            }
        } else {
            config_folder.to_string()
        };
        let docs_folder = resolve_template(&docs_folder, ctx.node_outputs, ctx.inputs);

        if docs_folder.is_empty() {
            return Err("Knowledge Base: docsFolder is empty".into());
        }

        let docs_path = std::path::Path::new(&docs_folder);
        if !docs_path.exists() || !docs_path.is_dir() {
            return Err(format!("Knowledge Base: docs folder not found: {docs_folder}"));
        }

        // Security check
        let canonical_base = docs_path.canonicalize()
            .map_err(|e| format!("Knowledge Base: cannot resolve docs folder: {e}"))?;
        if is_path_denied(&canonical_base) {
            return Err(format!("Knowledge Base: access denied to {docs_folder}"));
        }

        // Resolve query
        let query = if let Some(inc) = incoming {
            if let Some(obj) = inc.as_object() {
                obj.get("query").and_then(|v| v.as_str()).unwrap_or("").to_string()
            } else if let Some(s) = inc.as_str() {
                s.to_string()
            } else {
                String::new()
            }
        } else {
            String::new()
        };
        let query = resolve_template(&query, ctx.node_outputs, ctx.inputs);
        if query.is_empty() {
            return Err("Knowledge Base: query is empty".into());
        }

        // Config fields
        let index_location = node_data.get("indexLocation").and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("{}/.ai-studio-index", docs_folder));
        let index_dir = std::path::Path::new(&index_location);

        let embedding_provider = node_data.get("embeddingProvider").and_then(|v| v.as_str()).unwrap_or("azure_openai");
        let embedding_model = node_data.get("embeddingModel").and_then(|v| v.as_str()).unwrap_or("text-embedding-3-small");
        let chunk_size = node_data.get("chunkSize").and_then(|v| v.as_u64()).unwrap_or(500) as usize;
        let chunk_overlap = node_data.get("chunkOverlap").and_then(|v| v.as_u64()).unwrap_or(50) as usize;
        let chunk_strategy_str = node_data.get("chunkStrategy").and_then(|v| v.as_str()).unwrap_or("recursive");
        let chunk_strategy = ChunkStrategy::from_str(chunk_strategy_str);
        let top_k = node_data.get("topK").and_then(|v| v.as_u64()).unwrap_or(5) as usize;
        let score_threshold = node_data.get("scoreThreshold").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
        let file_types = node_data.get("fileTypes").and_then(|v| v.as_str()).unwrap_or(DEFAULT_FILE_TYPES);
        let max_file_size = node_data.get("maxFileSize").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
        let max_file_size_bytes = max_file_size * 1_048_576;

        eprintln!("[workflow] KnowledgeBase node '{}': folder={}, model={}, chunks={}",
            node_id, docs_folder, embedding_model, chunk_size);

        // --- Check index freshness ---
        let status = check_freshness(index_dir, docs_path, file_types, embedding_model);
        eprintln!("[workflow] KnowledgeBase node '{}': index status = {:?}", node_id, status);

        // --- Index if needed ---
        if status != IndexStatus::Fresh {
            emit_workflow_event(ctx.app, ctx.session_id, "workflow.node.streaming",
                serde_json::json!({ "node_id": node_id, "tokens": "Indexing documents..." }),
                ctx.seq_counter);

            // Scan files
            let file_paths = scan_docs(docs_path, file_types);
            let file_count = file_paths.len();
            eprintln!("[workflow] KnowledgeBase node '{}': found {} files to index", node_id, file_count);

            // Read and chunk all files
            let mut all_chunks = Vec::new();
            let mut indexed_files: HashMap<String, IndexedFileInfo> = HashMap::new();
            let mut total_chars = 0usize;

            for (idx, rel_path) in file_paths.iter().enumerate() {
                let full_path = docs_path.join(rel_path);

                // Security check
                if let Ok(canonical) = full_path.canonicalize() {
                    if is_path_denied(&canonical) {
                        continue;
                    }
                    if !canonical.starts_with(&canonical_base) {
                        continue;
                    }
                }

                // Skip files exceeding size limit
                if let Ok(metadata) = std::fs::metadata(&full_path) {
                    if metadata.len() > max_file_size_bytes as u64 {
                        continue;
                    }
                }

                // Check if this is a binary document that needs sidecar extraction
                let ext = full_path.extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                let content = if BINARY_EXTENSIONS.contains(&ext.as_str()) {
                    // Extract text via sidecar POST /extract
                    let extract_body = serde_json::json!({
                        "path": full_path.to_string_lossy(),
                        "format": ext,
                    });
                    match ctx.sidecar.proxy_request("POST", "/extract", Some(extract_body)).await {
                        Ok(resp) => {
                            resp.get("text")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string()
                        }
                        Err(e) => {
                            eprintln!("[workflow] KnowledgeBase: failed to extract {}: {}", rel_path, e);
                            continue;
                        }
                    }
                } else {
                    match std::fs::read_to_string(&full_path) {
                        Ok(c) => c,
                        Err(_) => continue,
                    }
                };

                total_chars += content.len();
                let chunks = chunk_text(&content, rel_path, chunk_strategy.clone(), chunk_size, chunk_overlap);
                let chunk_count = chunks.len();

                // Record file info
                let modified = std::fs::metadata(&full_path)
                    .and_then(|m| m.modified())
                    .ok()
                    .map(|t| chrono::DateTime::<chrono::Utc>::from(t).format("%Y-%m-%dT%H:%M:%SZ").to_string())
                    .unwrap_or_default();

                indexed_files.insert(rel_path.clone(), IndexedFileInfo {
                    modified,
                    chunks: chunk_count,
                });

                // Re-number chunk IDs globally
                let base_id = all_chunks.len();
                for mut chunk in chunks {
                    chunk.id = base_id + chunk.id;
                    all_chunks.push(chunk);
                }

                emit_workflow_event(ctx.app, ctx.session_id, "workflow.node.streaming",
                    serde_json::json!({
                        "node_id": node_id,
                        "tokens": format!("Indexing {}/{} files: {}...", idx + 1, file_count, rel_path),
                    }),
                    ctx.seq_counter);
            }

            eprintln!("[workflow] KnowledgeBase node '{}': {} chunks from {} files", node_id, all_chunks.len(), file_count);

            if all_chunks.is_empty() {
                return Err(format!("Knowledge Base: no text content found in {docs_folder}"));
            }

            // Embed all chunks via sidecar
            emit_workflow_event(ctx.app, ctx.session_id, "workflow.node.streaming",
                serde_json::json!({ "node_id": node_id, "tokens": format!("Embedding {} chunks...", all_chunks.len()) }),
                ctx.seq_counter);

            let texts: Vec<String> = all_chunks.iter().map(|c| c.text.clone()).collect();

            // Build provider config for embedding
            let prefix = format!("provider.{}.", embedding_provider);
            let mut api_key = String::new();
            let mut base_url = String::new();
            let mut extra_config = serde_json::Map::new();
            for (k, v) in ctx.all_settings {
                if let Some(field) = k.strip_prefix(&prefix) {
                    let clean_val = v.trim_matches('"').to_string();
                    match field {
                        "api_key" => api_key = clean_val,
                        "base_url" | "endpoint" => base_url = clean_val,
                        _ => { extra_config.insert(field.to_string(), Value::String(clean_val)); }
                    }
                }
            }

            // Use node-level embeddingDeployment if set, otherwise fall back to embeddingModel
            // This prevents the chat deployment name (e.g. gpt-4o-mini) from being used for embeddings
            let embed_deploy = node_data.get("embeddingDeployment")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .unwrap_or(embedding_model);
            extra_config.insert("deployment".to_string(), Value::String(embed_deploy.to_string()));

            let embed_body = serde_json::json!({
                "texts": texts,
                "provider": embedding_provider,
                "model": embedding_model,
                "api_key": api_key,
                "base_url": base_url,
                "extra_config": extra_config,
            });

            let embed_resp = ctx.sidecar.proxy_request("POST", "/embed", Some(embed_body)).await
                .map_err(|e| format!("Knowledge Base: embedding failed: {e}"))?;

            let raw_vectors: Vec<Vec<f32>> = embed_resp.get("vectors")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter().map(|vec| {
                        vec.as_array().unwrap_or(&vec![])
                            .iter()
                            .map(|v| v.as_f64().unwrap_or(0.0) as f32)
                            .collect()
                    }).collect()
                })
                .unwrap_or_default();

            if raw_vectors.len() != all_chunks.len() {
                return Err(format!(
                    "Knowledge Base: vector count mismatch: got {}, expected {}",
                    raw_vectors.len(), all_chunks.len()
                ));
            }

            let dimensions = embed_resp.get("dimensions").and_then(|v| v.as_u64()).unwrap_or(0) as u32;

            // Validate all vectors have consistent dimensions and finite values
            if let Some(expected_dims) = raw_vectors.first().map(|v| v.len()) {
                for (i, vec) in raw_vectors.iter().enumerate() {
                    if vec.len() != expected_dims {
                        return Err(format!(
                            "Knowledge Base: vector {} has {} dims, expected {}",
                            i, vec.len(), expected_dims
                        ));
                    }
                    if vec.iter().any(|v| !v.is_finite()) {
                        return Err(format!(
                            "Knowledge Base: vector {} contains non-finite values", i
                        ));
                    }
                }
            }

            // Normalize all vectors
            let mut vectors = raw_vectors;
            for v in &mut vectors {
                normalize(v);
            }

            // Write index
            let meta = IndexMeta {
                version: 1,
                embedding_provider: embedding_provider.to_string(),
                embedding_model: embedding_model.to_string(),
                dimensions,
                chunk_size,
                chunk_overlap,
                chunk_strategy: chunk_strategy_str.to_string(),
                file_count: indexed_files.len(),
                chunk_count: all_chunks.len(),
                total_chars,
                indexed_files,
                last_indexed: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                index_size_bytes: 0, // Will be set after write
            };

            write_index(index_dir, &all_chunks, &vectors, &meta)
                .map_err(|e| format!("Knowledge Base: failed to write index: {e}"))?;

            emit_workflow_event(ctx.app, ctx.session_id, "workflow.node.streaming",
                serde_json::json!({
                    "node_id": node_id,
                    "tokens": format!("Indexed {} chunks from {} files", all_chunks.len(), file_count),
                }),
                ctx.seq_counter);
        }

        // --- Search ---
        emit_workflow_event(ctx.app, ctx.session_id, "workflow.node.streaming",
            serde_json::json!({ "node_id": node_id, "tokens": "Searching..." }),
            ctx.seq_counter);

        // Embed query
        let prefix = format!("provider.{}.", embedding_provider);
        let mut api_key = String::new();
        let mut base_url = String::new();
        let mut extra_config = serde_json::Map::new();
        for (k, v) in ctx.all_settings {
            if let Some(field) = k.strip_prefix(&prefix) {
                let clean_val = v.trim_matches('"').to_string();
                match field {
                    "api_key" => api_key = clean_val,
                    "base_url" | "endpoint" => base_url = clean_val,
                    _ => { extra_config.insert(field.to_string(), Value::String(clean_val)); }
                }
            }
        }

        // Override deployment for embedding (don't use chat deployment name)
        let embed_deploy = node_data.get("embeddingDeployment")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .unwrap_or(embedding_model);
        extra_config.insert("deployment".to_string(), Value::String(embed_deploy.to_string()));

        let query_embed_body = serde_json::json!({
            "texts": [query],
            "provider": embedding_provider,
            "model": embedding_model,
            "api_key": api_key,
            "base_url": base_url,
            "extra_config": extra_config,
        });

        let query_resp = ctx.sidecar.proxy_request("POST", "/embed", Some(query_embed_body)).await
            .map_err(|e| format!("Knowledge Base: query embedding failed: {e}"))?;

        let mut query_vector: Vec<f32> = query_resp.get("vectors")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().map(|v| v.as_f64().unwrap_or(0.0) as f32).collect())
            .unwrap_or_default();

        normalize(&mut query_vector);

        // Search index
        let results = search(&query_vector, index_dir, top_k, score_threshold)
            .map_err(|e| format!("Knowledge Base: search failed: {e}"))?;

        let context = format_context_with_citations(&results);

        // Read index stats
        let stats = match read_meta(index_dir) {
            Ok(meta) => serde_json::json!({
                "fileCount": meta.file_count,
                "chunkCount": meta.chunk_count,
                "indexSizeBytes": meta.index_size_bytes,
                "lastIndexed": meta.last_indexed,
                "dimensions": meta.dimensions,
            }),
            Err(_) => serde_json::json!({}),
        };

        let results_json: Vec<Value> = results.iter().map(|r| serde_json::json!({
            "text": r.text,
            "score": r.score,
            "source": r.source,
            "lineStart": r.line_start,
            "lineEnd": r.line_end,
            "chunkId": r.chunk_id,
        })).collect();

        eprintln!("[workflow] KnowledgeBase node '{}': {} results, best score = {}",
            node_id, results_json.len(),
            results.first().map(|r| r.score).unwrap_or(0.0));

        let mut extra_outputs = HashMap::new();
        extra_outputs.insert("context".to_string(), Value::String(context.clone()));
        extra_outputs.insert("results".to_string(), Value::Array(results_json.clone()));
        extra_outputs.insert("indexStats".to_string(), stats.clone());

        Ok(NodeOutput {
            value: serde_json::json!({
                "context": context,
                "results": results_json,
                "indexStats": stats,
            }),
            skip_nodes: Vec::new(),
            extra_outputs,
        })
    }
}
