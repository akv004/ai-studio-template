use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;

use super::chunker::Chunk;

const VECTORS_MAGIC: u32 = 0x52414756; // "RAGV"
const VECTORS_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexMeta {
    pub version: u32,
    pub embedding_provider: String,
    pub embedding_model: String,
    pub dimensions: u32,
    pub chunk_size: usize,
    pub chunk_overlap: usize,
    pub chunk_strategy: String,
    pub file_count: usize,
    pub chunk_count: usize,
    pub total_chars: usize,
    pub indexed_files: HashMap<String, IndexedFileInfo>,
    pub last_indexed: String,
    pub index_size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedFileInfo {
    pub modified: String,
    pub chunks: usize,
}

#[derive(Debug, PartialEq)]
pub enum IndexStatus {
    Fresh,
    Stale,
    Missing,
    ModelChanged,
}

/// Write index files atomically: meta.json, chunks.jsonl, offsets.bin, vectors.bin.
/// Uses temp directory + rename for crash safety.
/// Acquires exclusive lock via fs2.
pub fn write_index(
    index_dir: &Path,
    chunks: &[Chunk],
    vectors: &[Vec<f32>],
    meta: &IndexMeta,
) -> Result<(), String> {
    use fs2::FileExt;

    std::fs::create_dir_all(index_dir)
        .map_err(|e| format!("Failed to create index dir: {e}"))?;

    // Set directory permissions on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(index_dir, std::fs::Permissions::from_mode(0o700));
    }

    // Acquire exclusive lock
    let lock_path = index_dir.join(".lock");
    let lock_file = std::fs::File::create(&lock_path)
        .map_err(|e| format!("Failed to create lock file: {e}"))?;
    lock_file.lock_exclusive()
        .map_err(|e| format!("Failed to acquire lock: {e}"))?;

    // Write to temp dir first
    let temp_dir = index_dir.join(format!(".tmp-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("Failed to create temp dir: {e}"))?;

    // 1. meta.json
    let meta_json = serde_json::to_string_pretty(meta)
        .map_err(|e| format!("Failed to serialize meta: {e}"))?;
    std::fs::write(temp_dir.join("meta.json"), &meta_json)
        .map_err(|e| format!("Failed to write meta.json: {e}"))?;

    // 2. chunks.jsonl + 3. offsets.bin
    let mut chunks_file = std::fs::File::create(temp_dir.join("chunks.jsonl"))
        .map_err(|e| format!("Failed to create chunks.jsonl: {e}"))?;
    let mut offsets: Vec<u64> = Vec::with_capacity(chunks.len());
    let mut byte_pos: u64 = 0;

    for chunk in chunks {
        offsets.push(byte_pos);
        let line = serde_json::to_string(chunk)
            .map_err(|e| format!("Failed to serialize chunk: {e}"))?;
        let line_bytes = format!("{}\n", line);
        chunks_file.write_all(line_bytes.as_bytes())
            .map_err(|e| format!("Failed to write chunk: {e}"))?;
        byte_pos += line_bytes.len() as u64;
    }

    // offsets.bin: array of u64 LE
    let mut offsets_file = std::fs::File::create(temp_dir.join("offsets.bin"))
        .map_err(|e| format!("Failed to create offsets.bin: {e}"))?;
    for offset in &offsets {
        offsets_file.write_all(&offset.to_le_bytes())
            .map_err(|e| format!("Failed to write offset: {e}"))?;
    }

    // 4. vectors.bin: magic(u32) + version(u32) + dims(u32) + count(u32) + f32[]
    let dims = if vectors.is_empty() { 0 } else { vectors[0].len() as u32 };
    let count = vectors.len() as u32;
    let mut vectors_file = std::fs::File::create(temp_dir.join("vectors.bin"))
        .map_err(|e| format!("Failed to create vectors.bin: {e}"))?;
    vectors_file.write_all(&VECTORS_MAGIC.to_le_bytes())
        .map_err(|e| format!("Failed to write magic: {e}"))?;
    vectors_file.write_all(&VECTORS_VERSION.to_le_bytes())
        .map_err(|e| format!("Failed to write version: {e}"))?;
    vectors_file.write_all(&dims.to_le_bytes())
        .map_err(|e| format!("Failed to write dims: {e}"))?;
    vectors_file.write_all(&count.to_le_bytes())
        .map_err(|e| format!("Failed to write count: {e}"))?;
    for vec in vectors {
        for &val in vec {
            vectors_file.write_all(&val.to_le_bytes())
                .map_err(|e| format!("Failed to write vector value: {e}"))?;
        }
    }

    // Atomic swap: rename old dir, rename temp dir into place, remove old
    // This is a single directory rename — all files swap together
    let old_backup = index_dir.join(format!(".old-{}", uuid::Uuid::new_v4()));
    let has_existing = index_dir.join("meta.json").exists();

    if has_existing {
        // Move existing files to backup dir
        std::fs::create_dir_all(&old_backup)
            .map_err(|e| format!("Failed to create backup dir: {e}"))?;
        for file_name in &["meta.json", "chunks.jsonl", "offsets.bin", "vectors.bin"] {
            let src = index_dir.join(file_name);
            if src.exists() {
                std::fs::rename(&src, old_backup.join(file_name))
                    .map_err(|e| format!("Failed to backup {file_name}: {e}"))?;
            }
        }
    }

    // Move new files into index dir
    for file_name in &["meta.json", "chunks.jsonl", "offsets.bin", "vectors.bin"] {
        let src = temp_dir.join(file_name);
        let dst = index_dir.join(file_name);
        std::fs::rename(&src, &dst)
            .map_err(|e| format!("Failed to move {file_name}: {e}"))?;
    }

    // Cleanup temp dir and backup
    let _ = std::fs::remove_dir_all(&temp_dir);
    if has_existing {
        let _ = std::fs::remove_dir_all(&old_backup);
    }

    // Create .gitignore
    let gitignore = index_dir.join(".gitignore");
    if !gitignore.exists() {
        let _ = std::fs::write(&gitignore, "# AI Studio RAG index — auto-generated, do not commit\n*\n");
    }

    // Set file permissions on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for file_name in &["meta.json", "chunks.jsonl", "offsets.bin", "vectors.bin"] {
            let _ = std::fs::set_permissions(
                index_dir.join(file_name),
                std::fs::Permissions::from_mode(0o600),
            );
        }
    }

    // Release lock
    let _ = lock_file.unlock();

    Ok(())
}

/// Acquire a shared lock for reading. Returns the lock file handle.
pub(crate) fn acquire_shared_lock(index_dir: &Path) -> Result<Option<std::fs::File>, String> {
    use fs2::FileExt;
    let lock_path = index_dir.join(".lock");
    if !lock_path.exists() {
        return Ok(None);
    }
    let lock_file = std::fs::File::open(&lock_path)
        .map_err(|e| format!("Failed to open lock file: {e}"))?;
    lock_file.lock_shared()
        .map_err(|e| format!("Failed to acquire shared lock: {e}"))?;
    Ok(Some(lock_file))
}

/// Read index metadata.
pub fn read_meta(index_dir: &Path) -> Result<IndexMeta, String> {
    let _lock = acquire_shared_lock(index_dir)?;
    let path = index_dir.join("meta.json");
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read meta.json: {e}"))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse meta.json: {e}"))
}

/// Read a single chunk by ID using offsets.bin for O(1) lookup.
pub fn read_chunk(index_dir: &Path, chunk_id: usize) -> Result<Chunk, String> {
    let _lock = acquire_shared_lock(index_dir)?;
    let offsets_path = index_dir.join("offsets.bin");
    let offsets_data = std::fs::read(&offsets_path)
        .map_err(|e| format!("Failed to read offsets.bin: {e}"))?;

    if (chunk_id + 1) * 8 > offsets_data.len() {
        return Err(format!("Chunk ID {chunk_id} out of range"));
    }

    let offset_bytes = &offsets_data[chunk_id * 8..(chunk_id + 1) * 8];
    let offset = u64::from_le_bytes(offset_bytes.try_into().unwrap());

    let chunks_path = index_dir.join("chunks.jsonl");
    let chunks_data = std::fs::read(&chunks_path)
        .map_err(|e| format!("Failed to read chunks.jsonl: {e}"))?;

    // Read line starting at offset
    let start = offset as usize;
    if start >= chunks_data.len() {
        return Err(format!("Offset {offset} out of range"));
    }
    let rest = &chunks_data[start..];
    let end = rest.iter().position(|&b| b == b'\n').unwrap_or(rest.len());
    let line = std::str::from_utf8(&rest[..end])
        .map_err(|e| format!("Invalid UTF-8 in chunk: {e}"))?;

    serde_json::from_str(line)
        .map_err(|e| format!("Failed to parse chunk: {e}"))
}

/// Check freshness of an index against the docs folder.
pub fn check_freshness(
    index_dir: &Path,
    docs_folder: &Path,
    file_types: &str,
    embedding_model: &str,
) -> IndexStatus {
    let meta_path = index_dir.join("meta.json");
    if !meta_path.exists() {
        return IndexStatus::Missing;
    }

    let meta = match read_meta(index_dir) {
        Ok(m) => m,
        Err(_) => return IndexStatus::Missing,
    };

    // Model changed → full rebuild
    if meta.embedding_model != embedding_model {
        return IndexStatus::ModelChanged;
    }

    // Scan disk files
    let disk_files = scan_docs(docs_folder, file_types);

    // Check for new files
    for path in &disk_files {
        if !meta.indexed_files.contains_key(path) {
            return IndexStatus::Stale;
        }
    }

    // Check for deleted files
    for path in meta.indexed_files.keys() {
        if !disk_files.contains(path) {
            return IndexStatus::Stale;
        }
    }

    // Check for modified files
    for path in &disk_files {
        if let Some(info) = meta.indexed_files.get(path) {
            let full_path = docs_folder.join(path);
            if let Ok(metadata) = std::fs::metadata(&full_path) {
                if let Ok(modified) = metadata.modified() {
                    let mtime_str = chrono::DateTime::<chrono::Utc>::from(modified)
                        .format("%Y-%m-%dT%H:%M:%SZ")
                        .to_string();
                    if mtime_str != info.modified {
                        return IndexStatus::Stale;
                    }
                }
            }
        }
    }

    IndexStatus::Fresh
}

/// Scan a docs folder for files matching file type patterns.
pub fn scan_docs(docs_folder: &Path, file_types: &str) -> Vec<String> {
    let patterns: Vec<&str> = file_types.split(',').map(|s| s.trim()).collect();
    let mut files = Vec::new();

    for pattern in patterns {
        let glob_pattern = format!("{}/**/{}", docs_folder.display(), pattern);
        if let Ok(entries) = glob::glob(&glob_pattern) {
            for entry in entries.flatten() {
                if entry.is_file() {
                    // Skip .ai-studio-index/ directory
                    let entry_str = entry.to_string_lossy();
                    if entry_str.contains(".ai-studio-index") {
                        continue;
                    }
                    if let Ok(rel) = entry.strip_prefix(docs_folder) {
                        files.push(rel.to_string_lossy().to_string());
                    }
                }
            }
        }
    }

    files.sort();
    files.dedup();
    files
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_test_chunks() -> Vec<Chunk> {
        vec![
            Chunk { id: 0, text: "Hello world".into(), source: "test.md".into(), line_start: 1, line_end: 1, byte_start: 0, byte_end: 11 },
            Chunk { id: 1, text: "Second chunk".into(), source: "test.md".into(), line_start: 2, line_end: 3, byte_start: 12, byte_end: 24 },
            Chunk { id: 2, text: "Third chunk".into(), source: "other.md".into(), line_start: 1, line_end: 2, byte_start: 0, byte_end: 11 },
        ]
    }

    fn make_test_vectors(dims: usize, count: usize) -> Vec<Vec<f32>> {
        (0..count).map(|i| {
            (0..dims).map(|j| ((i * dims + j) as f32) * 0.01).collect()
        }).collect()
    }

    #[test]
    fn test_write_read_roundtrip() {
        let dir = TempDir::new().unwrap();
        let index_dir = dir.path().join(".ai-studio-index");
        let chunks = make_test_chunks();
        let vectors = make_test_vectors(4, 3);
        let meta = IndexMeta {
            version: 1,
            embedding_provider: "local".into(),
            embedding_model: "nomic-embed-text".into(),
            dimensions: 4,
            chunk_size: 500,
            chunk_overlap: 50,
            chunk_strategy: "recursive".into(),
            file_count: 2,
            chunk_count: 3,
            total_chars: 34,
            indexed_files: HashMap::new(),
            last_indexed: "2026-02-22T12:00:00Z".into(),
            index_size_bytes: 0,
        };

        write_index(&index_dir, &chunks, &vectors, &meta).unwrap();

        // Read meta
        let read_m = read_meta(&index_dir).unwrap();
        assert_eq!(read_m.chunk_count, 3);
        assert_eq!(read_m.dimensions, 4);
        assert_eq!(read_m.embedding_model, "nomic-embed-text");

        // Read chunks
        let c0 = read_chunk(&index_dir, 0).unwrap();
        assert_eq!(c0.text, "Hello world");
        let c1 = read_chunk(&index_dir, 1).unwrap();
        assert_eq!(c1.text, "Second chunk");
        let c2 = read_chunk(&index_dir, 2).unwrap();
        assert_eq!(c2.text, "Third chunk");

        // Chunk out of range
        assert!(read_chunk(&index_dir, 99).is_err());

        // .gitignore created
        assert!(index_dir.join(".gitignore").exists());
    }

    #[test]
    fn test_vectors_bin_format() {
        let dir = TempDir::new().unwrap();
        let index_dir = dir.path().join(".ai-studio-index");
        let chunks = make_test_chunks();
        let vectors = make_test_vectors(4, 3);
        let meta = IndexMeta {
            version: 1, embedding_provider: "local".into(), embedding_model: "test".into(),
            dimensions: 4, chunk_size: 500, chunk_overlap: 50, chunk_strategy: "recursive".into(),
            file_count: 1, chunk_count: 3, total_chars: 34,
            indexed_files: HashMap::new(), last_indexed: "2026-02-22T12:00:00Z".into(), index_size_bytes: 0,
        };
        write_index(&index_dir, &chunks, &vectors, &meta).unwrap();

        let data = std::fs::read(index_dir.join("vectors.bin")).unwrap();
        // Header: 16 bytes
        assert!(data.len() >= 16);
        let magic = u32::from_le_bytes(data[0..4].try_into().unwrap());
        let version = u32::from_le_bytes(data[4..8].try_into().unwrap());
        let dims = u32::from_le_bytes(data[8..12].try_into().unwrap());
        let count = u32::from_le_bytes(data[12..16].try_into().unwrap());
        assert_eq!(magic, VECTORS_MAGIC);
        assert_eq!(version, VECTORS_VERSION);
        assert_eq!(dims, 4);
        assert_eq!(count, 3);
        assert_eq!(data.len(), 16 + 4 * 3 * 4); // header + dims * count * sizeof(f32)
    }

    #[test]
    fn test_check_freshness_missing() {
        let dir = TempDir::new().unwrap();
        let status = check_freshness(dir.path(), dir.path(), "*.md", "test-model");
        assert_eq!(status, IndexStatus::Missing);
    }

    #[test]
    fn test_scan_docs_excludes_index() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("readme.md"), "hello").unwrap();
        let idx_dir = dir.path().join(".ai-studio-index");
        std::fs::create_dir(&idx_dir).unwrap();
        std::fs::write(idx_dir.join("meta.json"), "{}").unwrap();

        let files = scan_docs(dir.path(), "*.md,*.json");
        assert!(files.contains(&"readme.md".to_string()));
        assert!(!files.iter().any(|f| f.contains(".ai-studio-index")));
    }

    #[test]
    fn test_offsets_correctness() {
        let dir = TempDir::new().unwrap();
        let index_dir = dir.path().join("idx");
        let chunks = make_test_chunks();
        let vectors = make_test_vectors(4, 3);
        let meta = IndexMeta {
            version: 1, embedding_provider: "local".into(), embedding_model: "test".into(),
            dimensions: 4, chunk_size: 500, chunk_overlap: 50, chunk_strategy: "recursive".into(),
            file_count: 1, chunk_count: 3, total_chars: 34,
            indexed_files: HashMap::new(), last_indexed: "2026-02-22T12:00:00Z".into(), index_size_bytes: 0,
        };
        write_index(&index_dir, &chunks, &vectors, &meta).unwrap();

        // Verify each chunk can be read via offset
        for i in 0..3 {
            let chunk = read_chunk(&index_dir, i).unwrap();
            assert_eq!(chunk.id, i);
        }
    }
}
