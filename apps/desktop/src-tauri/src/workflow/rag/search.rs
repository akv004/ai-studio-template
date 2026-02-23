use std::collections::BinaryHeap;
use std::cmp::Ordering;
use std::path::Path;

use super::chunker::Chunk;
use super::index::{read_chunk, acquire_shared_lock};

const VECTORS_MAGIC: u32 = 0x52414756;
const VECTORS_VERSION: u32 = 1;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub chunk_id: usize,
    pub score: f32,
    pub text: String,
    pub source: String,
    pub line_start: usize,
    pub line_end: usize,
}

/// L2 normalize a vector in-place. Called once at index time per vector,
/// and once at query time for the query vector.
pub fn normalize(v: &mut [f32]) {
    let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        v.iter_mut().for_each(|x| *x /= norm);
    }
}

/// Dot product similarity. On pre-normalized vectors, this equals cosine similarity.
pub fn dot_similarity(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

/// Min-heap entry for top-K selection.
#[derive(Debug)]
struct HeapEntry {
    score: f32,
    chunk_id: usize,
}

impl PartialEq for HeapEntry {
    fn eq(&self, other: &Self) -> bool { self.score == other.score }
}

impl Eq for HeapEntry {}

impl PartialOrd for HeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HeapEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap (smallest score on top)
        // Tiebreak by chunk_id (lower ID first = larger in reversed heap)
        match other.score.partial_cmp(&self.score) {
            Some(Ordering::Equal) | None => other.chunk_id.cmp(&self.chunk_id),
            Some(ord) => ord,
        }
    }
}

/// Search an index for the top-K most similar chunks to the query vector.
/// Query vector must be pre-normalized.
pub fn search(
    query_vector: &[f32],
    index_dir: &Path,
    top_k: usize,
    threshold: f32,
) -> Result<Vec<SearchResult>, String> {
    let vectors_path = index_dir.join("vectors.bin");
    if !vectors_path.exists() {
        return Ok(Vec::new());
    }

    // Acquire shared lock for consistent reads during search
    let _lock = acquire_shared_lock(index_dir)?;

    let file = std::fs::File::open(&vectors_path)
        .map_err(|e| format!("Failed to open vectors.bin: {e}"))?;

    let file_len = file.metadata()
        .map_err(|e| format!("Failed to get file metadata: {e}"))?
        .len() as usize;

    if file_len < 16 {
        return Err("vectors.bin too small (no header)".into());
    }

    // Memory-map the file
    let mmap = unsafe {
        memmap2::MmapOptions::new()
            .map(&file)
            .map_err(|e| format!("Failed to mmap vectors.bin: {e}"))?
    };

    // Validate header
    let magic = u32::from_le_bytes(mmap[0..4].try_into().unwrap());
    let version = u32::from_le_bytes(mmap[4..8].try_into().unwrap());
    let dims = u32::from_le_bytes(mmap[8..12].try_into().unwrap()) as usize;
    let count = u32::from_le_bytes(mmap[12..16].try_into().unwrap()) as usize;

    if magic != VECTORS_MAGIC {
        return Err(format!("Invalid vectors.bin magic: {magic:#X} (expected {VECTORS_MAGIC:#X})"));
    }
    if version != VECTORS_VERSION {
        return Err(format!("Unsupported vectors.bin version: {version}"));
    }

    let expected_len = 16 + dims * count * 4;
    if file_len != expected_len {
        return Err(format!(
            "vectors.bin size mismatch: got {file_len}, expected {expected_len} (dims={dims}, count={count})"
        ));
    }

    if count == 0 {
        return Ok(Vec::new());
    }

    if dims != query_vector.len() {
        return Err(format!(
            "Query vector dimension mismatch: query has {}, index has {dims}",
            query_vector.len()
        ));
    }

    // Verify mmap alignment for f32 reads (mmap is always page-aligned, but be safe)
    let float_data = &mmap[16..];
    assert!(
        (float_data.as_ptr() as usize) % std::mem::align_of::<u8>() == 0,
        "mmap data not byte-aligned"
    );

    // BinaryHeap min-heap for top-K
    let mut heap: BinaryHeap<HeapEntry> = BinaryHeap::with_capacity(top_k + 1);

    for i in 0..count {
        let offset = i * dims * 4;
        // Compute dot product directly over byte slice — no per-vector allocation
        let mut score: f32 = 0.0;
        for j in 0..dims {
            let start = offset + j * 4;
            let val = f32::from_le_bytes(float_data[start..start + 4].try_into().unwrap());
            score += query_vector[j] * val;
        }

        // Filter non-finite scores (NaN, Inf)
        if !score.is_finite() || score < threshold {
            continue;
        }

        heap.push(HeapEntry { score, chunk_id: i });
        if heap.len() > top_k {
            heap.pop(); // Remove the lowest score
        }
    }

    // Extract results and sort by score descending
    let mut results: Vec<(f32, usize)> = heap.into_iter()
        .map(|e| (e.score, e.chunk_id))
        .collect();
    results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(Ordering::Equal));

    // Load chunk data for top results
    let mut search_results = Vec::new();
    for (score, chunk_id) in results {
        match read_chunk(index_dir, chunk_id) {
            Ok(chunk) => {
                search_results.push(SearchResult {
                    chunk_id,
                    score,
                    text: chunk.text,
                    source: chunk.source,
                    line_start: chunk.line_start,
                    line_end: chunk.line_end,
                });
            }
            Err(e) => {
                eprintln!("[rag] Warning: failed to read chunk {chunk_id}: {e}");
            }
        }
    }

    Ok(search_results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::index::{write_index, IndexMeta};
    use super::super::chunker::Chunk;
    use tempfile::TempDir;
    use std::collections::HashMap;

    fn make_normalized_vectors(dims: usize, count: usize) -> Vec<Vec<f32>> {
        (0..count).map(|i| {
            let mut v: Vec<f32> = (0..dims).map(|j| ((i * dims + j + 1) as f32)).collect();
            normalize(&mut v);
            v
        }).collect()
    }

    fn setup_test_index(dir: &Path, dims: usize, count: usize) -> Vec<Vec<f32>> {
        let chunks: Vec<Chunk> = (0..count).map(|i| {
            Chunk { id: i, text: format!("Chunk {i}"), source: "test.md".into(), line_start: i + 1, line_end: i + 1, byte_start: i * 10, byte_end: (i + 1) * 10 }
        }).collect();
        let vectors = make_normalized_vectors(dims, count);
        let meta = IndexMeta {
            version: 1, embedding_provider: "local".into(), embedding_model: "test".into(),
            dimensions: dims as u32, chunk_size: 500, chunk_overlap: 50, chunk_strategy: "recursive".into(),
            file_count: 1, chunk_count: count, total_chars: count * 10,
            indexed_files: HashMap::new(), last_indexed: "2026-02-22T12:00:00Z".into(), index_size_bytes: 0,
        };
        write_index(dir, &chunks, &vectors, &meta).unwrap();
        vectors
    }

    #[test]
    fn test_normalize() {
        let mut v = vec![3.0, 4.0];
        normalize(&mut v);
        let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_normalize_zero_vector() {
        let mut v = vec![0.0, 0.0, 0.0];
        normalize(&mut v);
        assert_eq!(v, vec![0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_dot_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((dot_similarity(&a, &b) - 1.0).abs() < 1e-6);

        let c = vec![0.0, 1.0, 0.0];
        assert!((dot_similarity(&a, &c)).abs() < 1e-6);
    }

    #[test]
    fn test_search_top_k() {
        let dir = TempDir::new().unwrap();
        let index_dir = dir.path().join("idx");
        let vectors = setup_test_index(&index_dir, 4, 10);

        // Query with the first vector → should match itself best
        let results = search(&vectors[0], &index_dir, 3, 0.0).unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].chunk_id, 0); // Best match is itself
        assert!(results[0].score > results[1].score);
    }

    #[test]
    fn test_search_threshold_filter() {
        let dir = TempDir::new().unwrap();
        let index_dir = dir.path().join("idx");
        let _vectors = setup_test_index(&index_dir, 4, 5);

        // Query with a very different vector, high threshold
        let mut query = vec![1.0, 0.0, 0.0, 0.0];
        normalize(&mut query);
        let results = search(&query, &index_dir, 10, 0.999).unwrap();
        // Should filter out low scores
        for r in &results {
            assert!(r.score >= 0.999);
        }
    }

    #[test]
    fn test_search_empty_index() {
        let dir = TempDir::new().unwrap();
        let index_dir = dir.path().join("idx");
        setup_test_index(&index_dir, 4, 0);
        let query = vec![1.0, 0.0, 0.0, 0.0];
        let results = search(&query, &index_dir, 5, 0.0).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_nonexistent_index() {
        let dir = TempDir::new().unwrap();
        let query = vec![1.0, 0.0, 0.0, 0.0];
        let results = search(&query, dir.path(), 5, 0.0).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_bad_magic() {
        let dir = TempDir::new().unwrap();
        let vectors_path = dir.path().join("vectors.bin");
        let mut data = vec![0u8; 16 + 4 * 4]; // header + one 4-dim vector
        // Wrong magic
        data[0..4].copy_from_slice(&0xDEADBEEFu32.to_le_bytes());
        data[4..8].copy_from_slice(&1u32.to_le_bytes()); // version
        data[8..12].copy_from_slice(&4u32.to_le_bytes()); // dims
        data[12..16].copy_from_slice(&1u32.to_le_bytes()); // count
        std::fs::write(&vectors_path, &data).unwrap();

        let query = vec![1.0, 0.0, 0.0, 0.0];
        let result = search(&query, dir.path(), 5, 0.0);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("magic"));
    }

    #[test]
    fn test_search_truncated_file() {
        let dir = TempDir::new().unwrap();
        let vectors_path = dir.path().join("vectors.bin");
        let mut data = vec![0u8; 16];
        data[0..4].copy_from_slice(&0x52414756u32.to_le_bytes()); // correct magic
        data[4..8].copy_from_slice(&1u32.to_le_bytes());
        data[8..12].copy_from_slice(&4u32.to_le_bytes()); // 4 dims
        data[12..16].copy_from_slice(&10u32.to_le_bytes()); // claims 10 vectors but file is too small
        std::fs::write(&vectors_path, &data).unwrap();

        let query = vec![1.0, 0.0, 0.0, 0.0];
        let result = search(&query, dir.path(), 5, 0.0);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("mismatch"));
    }

    #[test]
    fn test_zero_norm_query() {
        let dir = TempDir::new().unwrap();
        let index_dir = dir.path().join("idx");
        let _vectors = setup_test_index(&index_dir, 4, 5);

        let query = vec![0.0, 0.0, 0.0, 0.0];
        let results = search(&query, &index_dir, 5, 0.0).unwrap();
        // All scores should be 0.0
        for r in &results {
            assert!((r.score).abs() < 1e-6);
        }
    }
}
