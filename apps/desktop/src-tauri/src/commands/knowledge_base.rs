use crate::error::AppError;
use crate::workflow::rag;
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexStats {
    pub file_count: usize,
    pub chunk_count: usize,
    pub dimensions: u32,
    pub embedding_model: String,
    pub last_indexed: String,
    pub index_size_bytes: u64,
}

#[tauri::command]
pub async fn index_folder(
    sidecar: tauri::State<'_, crate::sidecar::SidecarManager>,
    db: tauri::State<'_, crate::db::Database>,
    docs_folder: String,
    index_location: Option<String>,
    embedding_provider: String,
    embedding_model: String,
    chunk_size: Option<usize>,
    chunk_overlap: Option<usize>,
    chunk_strategy: Option<String>,
    file_types: Option<String>,
    max_file_size: Option<usize>,
) -> Result<IndexStats, AppError> {
    let docs_path = Path::new(&docs_folder);
    if !docs_path.exists() || !docs_path.is_dir() {
        return Err(AppError::NotFound(format!("Docs folder not found: {docs_folder}")));
    }

    let index_dir_str = index_location.unwrap_or_else(|| format!("{}/.ai-studio-index", docs_folder));
    let index_dir = Path::new(&index_dir_str);
    let chunk_size = chunk_size.unwrap_or(500);
    let chunk_overlap = chunk_overlap.unwrap_or(50);
    let strategy_str = chunk_strategy.as_deref().unwrap_or("recursive");
    let strategy = rag::ChunkStrategy::from_str(strategy_str);
    let file_types_str = file_types.as_deref().unwrap_or("*.md,*.txt,*.rs,*.py,*.ts,*.js,*.json,*.yml,*.yaml,*.csv,*.toml,*.go,*.java");
    let max_size_bytes = max_file_size.unwrap_or(10) * 1_048_576;

    // Scan and chunk files
    let file_paths = rag::index::scan_docs(docs_path, file_types_str);
    let mut all_chunks = Vec::new();
    let mut indexed_files = std::collections::HashMap::new();
    let mut total_chars = 0usize;

    for rel_path in &file_paths {
        let full_path = docs_path.join(rel_path);
        if let Ok(metadata) = std::fs::metadata(&full_path) {
            if metadata.len() > max_size_bytes as u64 {
                continue;
            }
        }
        let content = match std::fs::read_to_string(&full_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        total_chars += content.len();
        let chunks = rag::chunk_text(&content, rel_path, strategy.clone(), chunk_size, chunk_overlap);
        let chunk_count = chunks.len();

        let modified = std::fs::metadata(&full_path)
            .and_then(|m| m.modified())
            .ok()
            .map(|t| chrono::DateTime::<chrono::Utc>::from(t).format("%Y-%m-%dT%H:%M:%SZ").to_string())
            .unwrap_or_default();

        indexed_files.insert(rel_path.clone(), rag::index::IndexedFileInfo { modified, chunks: chunk_count });

        let base_id = all_chunks.len();
        for mut chunk in chunks {
            chunk.id = base_id + chunk.id;
            all_chunks.push(chunk);
        }
    }

    if all_chunks.is_empty() {
        return Err(AppError::Validation("No text content found".into()));
    }

    // Get provider config from settings
    let prefix = format!("provider.{}.", embedding_provider);
    let mut api_key = String::new();
    let mut base_url = String::new();
    let mut extra_config = serde_json::Map::new();
    {
        let conn = db.conn.lock()?;
        let mut stmt = conn.prepare("SELECT key, value FROM settings WHERE key LIKE ?1")?;
        let rows = stmt.query_map([format!("{}%", prefix)], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        for row in rows {
            let (k, v) = row.map_err(|e| AppError::Db(e.to_string()))?;
            if let Some(field) = k.strip_prefix(&prefix) {
                let clean_val = v.trim_matches('"').to_string();
                match field {
                    "api_key" => api_key = clean_val,
                    "base_url" | "endpoint" => base_url = clean_val,
                    _ => { extra_config.insert(field.to_string(), serde_json::Value::String(clean_val)); }
                }
            }
        }
    }

    // Embed via sidecar
    let texts: Vec<String> = all_chunks.iter().map(|c| c.text.clone()).collect();
    let embed_body = serde_json::json!({
        "texts": texts,
        "provider": embedding_provider,
        "model": embedding_model,
        "api_key": api_key,
        "base_url": base_url,
        "extra_config": extra_config,
    });

    let embed_resp = sidecar.proxy_request("POST", "/embed", Some(embed_body)).await
        .map_err(|e| AppError::Internal(format!("Embedding failed: {e}")))?;

    let raw_vectors: Vec<Vec<f32>> = embed_resp.get("vectors")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().map(|vec| {
            vec.as_array().unwrap_or(&vec![])
                .iter().map(|v| v.as_f64().unwrap_or(0.0) as f32).collect()
        }).collect())
        .unwrap_or_default();

    let dimensions = embed_resp.get("dimensions").and_then(|v| v.as_u64()).unwrap_or(0) as u32;

    let mut vectors = raw_vectors;
    for v in &mut vectors {
        rag::normalize(v);
    }

    let meta = rag::IndexMeta {
        version: 1,
        embedding_provider: embedding_provider.clone(),
        embedding_model: embedding_model.clone(),
        dimensions,
        chunk_size,
        chunk_overlap,
        chunk_strategy: strategy_str.to_string(),
        file_count: indexed_files.len(),
        chunk_count: all_chunks.len(),
        total_chars,
        indexed_files,
        last_indexed: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        index_size_bytes: 0,
    };

    rag::write_index(index_dir, &all_chunks, &vectors, &meta)
        .map_err(|e| AppError::Internal(format!("Failed to write index: {e}")))?;

    Ok(IndexStats {
        file_count: meta.file_count,
        chunk_count: meta.chunk_count,
        dimensions,
        embedding_model,
        last_indexed: meta.last_indexed,
        index_size_bytes: 0,
    })
}

#[tauri::command]
pub async fn search_index(
    sidecar: tauri::State<'_, crate::sidecar::SidecarManager>,
    db: tauri::State<'_, crate::db::Database>,
    index_location: String,
    query: String,
    top_k: Option<usize>,
    score_threshold: Option<f32>,
    embedding_provider: String,
    embedding_model: String,
) -> Result<Vec<serde_json::Value>, AppError> {
    let index_dir = Path::new(&index_location);
    if !index_dir.exists() {
        return Err(AppError::NotFound(format!("Index not found: {index_location}")));
    }

    let top_k = top_k.unwrap_or(5);
    let threshold = score_threshold.unwrap_or(0.0);

    // Get provider config
    let prefix = format!("provider.{}.", embedding_provider);
    let mut api_key = String::new();
    let mut base_url = String::new();
    let mut extra_config = serde_json::Map::new();
    {
        let conn = db.conn.lock()?;
        let mut stmt = conn.prepare("SELECT key, value FROM settings WHERE key LIKE ?1")?;
        let rows = stmt.query_map([format!("{}%", prefix)], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        for row in rows {
            let (k, v) = row.map_err(|e| AppError::Db(e.to_string()))?;
            if let Some(field) = k.strip_prefix(&prefix) {
                let clean_val = v.trim_matches('"').to_string();
                match field {
                    "api_key" => api_key = clean_val,
                    "base_url" | "endpoint" => base_url = clean_val,
                    _ => { extra_config.insert(field.to_string(), serde_json::Value::String(clean_val)); }
                }
            }
        }
    }

    // Embed query
    let embed_body = serde_json::json!({
        "texts": [query],
        "provider": embedding_provider,
        "model": embedding_model,
        "api_key": api_key,
        "base_url": base_url,
        "extra_config": extra_config,
    });

    let resp = sidecar.proxy_request("POST", "/embed", Some(embed_body)).await
        .map_err(|e| AppError::Internal(format!("Query embedding failed: {e}")))?;

    let mut query_vector: Vec<f32> = resp.get("vectors")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().map(|v| v.as_f64().unwrap_or(0.0) as f32).collect())
        .unwrap_or_default();

    rag::normalize(&mut query_vector);

    let results = rag::search(&query_vector, index_dir, top_k, threshold)
        .map_err(|e| AppError::Internal(format!("Search failed: {e}")))?;

    Ok(results.iter().map(|r| serde_json::json!({
        "text": r.text,
        "score": r.score,
        "source": r.source,
        "lineStart": r.line_start,
        "lineEnd": r.line_end,
        "chunkId": r.chunk_id,
    })).collect())
}

#[tauri::command]
pub fn get_index_stats(index_location: String) -> Result<IndexStats, AppError> {
    let index_dir = Path::new(&index_location);
    let meta = rag::read_meta(index_dir)
        .map_err(|e| AppError::NotFound(format!("Index not found: {e}")))?;

    Ok(IndexStats {
        file_count: meta.file_count,
        chunk_count: meta.chunk_count,
        dimensions: meta.dimensions,
        embedding_model: meta.embedding_model,
        last_indexed: meta.last_indexed,
        index_size_bytes: meta.index_size_bytes,
    })
}

#[tauri::command]
pub fn delete_index(index_location: String) -> Result<(), AppError> {
    let index_dir = Path::new(&index_location);
    if index_dir.exists() {
        std::fs::remove_dir_all(index_dir)
            .map_err(|e| AppError::Internal(format!("Failed to delete index: {e}")))?;
    }
    Ok(())
}
