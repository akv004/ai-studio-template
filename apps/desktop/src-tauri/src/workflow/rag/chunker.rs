use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ChunkStrategy {
    FixedSize,
    Sentence,
    Paragraph,
    Recursive,
}

impl ChunkStrategy {
    pub fn from_str(s: &str) -> Self {
        match s {
            "fixed_size" => Self::FixedSize,
            "sentence" => Self::Sentence,
            "paragraph" => Self::Paragraph,
            _ => Self::Recursive,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub id: usize,
    pub text: String,
    pub source: String,
    #[serde(rename = "lineStart")]
    pub line_start: usize,
    #[serde(rename = "lineEnd")]
    pub line_end: usize,
    #[serde(rename = "byteStart")]
    pub byte_start: usize,
    #[serde(rename = "byteEnd")]
    pub byte_end: usize,
}

/// Precompute byte offsets where each line starts (0-indexed line numbers).
fn line_offsets(text: &str) -> Vec<usize> {
    let mut offsets = vec![0usize];
    for (i, b) in text.bytes().enumerate() {
        if b == b'\n' {
            offsets.push(i + 1);
        }
    }
    offsets
}

/// Given a byte offset, find the 1-based line number via binary search.
fn byte_to_line(offsets: &[usize], byte_pos: usize) -> usize {
    match offsets.binary_search(&byte_pos) {
        Ok(idx) => idx + 1,
        Err(idx) => idx, // idx is the line that starts after byte_pos
    }
}

/// Split text into chunks using the specified strategy.
pub fn chunk_text(
    content: &str,
    source: &str,
    strategy: ChunkStrategy,
    chunk_size: usize,
    overlap: usize,
) -> Vec<Chunk> {
    if content.is_empty() {
        return Vec::new();
    }

    let chunk_size = chunk_size.max(10);
    let overlap = overlap.min(chunk_size.saturating_sub(1));
    let hard_cap = (chunk_size * 2).max(2000);

    // Normalize CRLF → LF
    let normalized = content.replace("\r\n", "\n");
    let offsets = line_offsets(&normalized);

    let raw_chunks = match strategy {
        ChunkStrategy::FixedSize => split_fixed(&normalized, chunk_size, overlap),
        ChunkStrategy::Sentence => split_sentence(&normalized, chunk_size, overlap),
        ChunkStrategy::Paragraph => split_paragraph(&normalized, chunk_size, overlap),
        ChunkStrategy::Recursive => split_recursive(&normalized, chunk_size, overlap),
    };

    raw_chunks
        .into_iter()
        .enumerate()
        .map(|(id, (text, byte_start, byte_end))| {
            // Apply hard cap — adjust byte_end to match truncated text
            let (text, byte_end) = if text.chars().count() > hard_cap {
                let truncated = truncate_chars(&text, hard_cap).to_string();
                let adjusted_end = byte_start + truncated.len();
                (truncated, adjusted_end)
            } else {
                (text, byte_end)
            };
            let line_start = byte_to_line(&offsets, byte_start);
            let line_end = byte_to_line(&offsets, byte_end.saturating_sub(1).max(byte_start));

            Chunk {
                id,
                text,
                source: source.to_string(),
                line_start,
                line_end,
                byte_start,
                byte_end,
            }
        })
        .collect()
}

/// UTF-8 safe truncation to N characters.
fn truncate_chars(s: &str, max: usize) -> &str {
    match s.char_indices().nth(max) {
        Some((idx, _)) => &s[..idx],
        None => s,
    }
}

/// Find the nearest char boundary at or before `pos`.
fn safe_boundary(text: &str, pos: usize) -> usize {
    if pos >= text.len() {
        return text.len();
    }
    // Walk backwards to find char boundary
    let mut p = pos;
    while p > 0 && !text.is_char_boundary(p) {
        p -= 1;
    }
    p
}

/// Find a word boundary near `pos` (prefer splitting on whitespace).
fn word_boundary(text: &str, pos: usize) -> usize {
    let safe_pos = safe_boundary(text, pos);
    // Look back from safe_pos for whitespace — search_start must also be a char boundary
    let search_start = safe_boundary(text, safe_pos.saturating_sub(50));
    if let Some(last_ws) = text[search_start..safe_pos].rfind(|c: char| c.is_whitespace()) {
        search_start + last_ws + 1
    } else {
        safe_pos
    }
}

fn split_fixed(text: &str, chunk_size: usize, overlap: usize) -> Vec<(String, usize, usize)> {
    let mut chunks = Vec::new();
    let mut pos = 0;
    let len = text.len();

    while pos < len {
        // Use char_indices to find the byte offset for chunk_size chars from pos
        let actual_end = match text[pos..].char_indices().nth(chunk_size) {
            Some((idx, _)) => {
                let abs = pos + idx;
                word_boundary(text, abs)
            }
            None => len,
        };

        let chunk = text[pos..actual_end].to_string();
        if !chunk.trim().is_empty() {
            chunks.push((chunk, pos, actual_end));
        }

        // Advance by (chunk_chars - overlap_chars) using char_indices for UTF-8 safety
        let advance_chars = chunk_size.saturating_sub(overlap);
        let advance_bytes = match text[pos..].char_indices().nth(advance_chars) {
            Some((idx, _)) => idx,
            None => actual_end - pos,
        };
        let new_pos = pos + advance_bytes.max(1);
        pos = safe_boundary(text, new_pos);
    }
    chunks
}

fn is_sentence_end(text: &str, byte_pos: usize) -> bool {
    if byte_pos >= text.len() {
        return false;
    }
    let c = text.as_bytes()[byte_pos];
    // ASCII sentence enders
    if matches!(c, b'.' | b'!' | b'?') {
        // Check for abbreviation: single uppercase letter before dot
        if c == b'.' && byte_pos >= 2 {
            let prev = text.as_bytes()[byte_pos - 1];
            let prev2 = text.as_bytes()[byte_pos - 2];
            if prev.is_ascii_uppercase() && (prev2 == b' ' || prev2 == b'\n') {
                return false; // "U.S.", "Dr.", etc.
            }
        }
        // Must be followed by whitespace or end
        let next_pos = byte_pos + 1;
        if next_pos >= text.len() {
            return true;
        }
        let next = text.as_bytes()[next_pos];
        return next == b' ' || next == b'\n' || next == b'\r';
    }
    // CJK sentence enders: 。！？
    // These are 3-byte UTF-8 sequences
    if byte_pos + 2 < text.len() {
        let bytes = &text.as_bytes()[byte_pos..byte_pos + 3];
        // 。= E3 80 82, ！= EF BC 81, ？= EF BC 9F
        if bytes == [0xE3, 0x80, 0x82] || bytes == [0xEF, 0xBC, 0x81] || bytes == [0xEF, 0xBC, 0x9F] {
            return true;
        }
    }
    false
}

fn split_sentence(text: &str, chunk_size: usize, overlap: usize) -> Vec<(String, usize, usize)> {
    // Find all sentence boundaries
    let mut boundaries = Vec::new();
    for (i, _) in text.char_indices() {
        if is_sentence_end(text, i) {
            // End byte is after the punctuation char
            let end = i + text[i..].chars().next().map(|c| c.len_utf8()).unwrap_or(1);
            boundaries.push(end);
        }
    }
    if boundaries.is_empty() || *boundaries.last().unwrap() < text.len() {
        boundaries.push(text.len());
    }

    merge_segments_by_size(text, &boundaries, chunk_size, overlap)
}

fn split_paragraph(text: &str, chunk_size: usize, overlap: usize) -> Vec<(String, usize, usize)> {
    // Find paragraph boundaries (\n\n)
    let mut boundaries = Vec::new();
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len().saturating_sub(1) {
        if bytes[i] == b'\n' && bytes[i + 1] == b'\n' {
            boundaries.push(i + 2);
            i += 2;
        } else {
            i += 1;
        }
    }
    if boundaries.is_empty() || *boundaries.last().unwrap() < text.len() {
        boundaries.push(text.len());
    }

    // If paragraphs are too large, fall through to sentence splitting
    let result = merge_segments_by_size(text, &boundaries, chunk_size, overlap);
    if result.iter().any(|(t, _, _)| t.chars().count() > chunk_size * 2) {
        return split_sentence(text, chunk_size, overlap);
    }
    result
}

fn split_recursive(text: &str, chunk_size: usize, overlap: usize) -> Vec<(String, usize, usize)> {
    // Try paragraph first
    let result = split_paragraph(text, chunk_size, overlap);
    // If any chunk is still too large, re-split those with sentence
    let mut final_chunks = Vec::new();
    for (chunk_text, byte_start, _byte_end) in result {
        if chunk_text.chars().count() > chunk_size * 2 {
            let sub = split_sentence(&chunk_text, chunk_size, overlap);
            for (sub_text, sub_start, sub_end) in sub {
                final_chunks.push((sub_text, byte_start + sub_start, byte_start + sub_end));
            }
        } else {
            final_chunks.push((chunk_text.clone(), byte_start, byte_start + chunk_text.len()));
        }
    }
    final_chunks
}

/// Merge segment boundaries into chunks near target size.
fn merge_segments_by_size(
    text: &str,
    boundaries: &[usize],
    chunk_size: usize,
    overlap: usize,
) -> Vec<(String, usize, usize)> {
    let mut chunks = Vec::new();
    let mut start = 0;

    let mut bi = 0;
    while bi < boundaries.len() {
        // Accumulate segments until char count exceeds chunk_size
        let mut end = boundaries[bi];
        while bi + 1 < boundaries.len() && text[start..end].chars().count() < chunk_size {
            bi += 1;
            end = boundaries[bi];
        }

        let chunk = text[start..end].to_string();
        if !chunk.trim().is_empty() {
            chunks.push((chunk, start, end));
        }

        // Advance with char-based overlap
        if overlap > 0 && end > start {
            // Count overlap chars backwards from end
            let chunk_chars: Vec<(usize, char)> = text[start..end].char_indices().collect();
            let overlap_start_idx = chunk_chars.len().saturating_sub(overlap);
            start = start + chunk_chars[overlap_start_idx].0;
        } else {
            start = end;
        }

        bi += 1;
    }
    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_content() {
        let chunks = chunk_text("", "test.md", ChunkStrategy::Recursive, 500, 50);
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_single_char() {
        let chunks = chunk_text("a", "test.md", ChunkStrategy::Recursive, 500, 50);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].text, "a");
        assert_eq!(chunks[0].line_start, 1);
        assert_eq!(chunks[0].line_end, 1);
    }

    #[test]
    fn test_cjk_text() {
        let text = "这是一个测试。这是第二句话。这是第三句话！";
        let chunks = chunk_text(text, "cjk.md", ChunkStrategy::Sentence, 20, 0);
        assert!(!chunks.is_empty());
        // Each chunk should be valid UTF-8 (no mid-codepoint splits)
        for chunk in &chunks {
            assert!(std::str::from_utf8(chunk.text.as_bytes()).is_ok());
        }
    }

    #[test]
    fn test_crlf_normalization() {
        let text = "Line 1\r\nLine 2\r\nLine 3";
        let chunks = chunk_text(text, "crlf.txt", ChunkStrategy::FixedSize, 500, 0);
        assert!(!chunks.is_empty());
        // Should not contain \r
        for chunk in &chunks {
            assert!(!chunk.text.contains('\r'));
        }
    }

    #[test]
    fn test_no_paragraph_fallthrough() {
        // Text with no paragraph breaks should still chunk
        let text = "This is a sentence. Another sentence. Third sentence. Fourth sentence. Fifth sentence.";
        let chunks = chunk_text(text, "test.md", ChunkStrategy::Paragraph, 30, 0);
        assert!(!chunks.is_empty());
    }

    #[test]
    fn test_hard_cap() {
        // Create a very long single line
        let text = "a".repeat(10000);
        let chunks = chunk_text(&text, "big.txt", ChunkStrategy::FixedSize, 100, 0);
        for chunk in &chunks {
            let char_count = chunk.text.chars().count();
            let hard_cap = (100usize * 2).max(2000);
            assert!(char_count <= hard_cap, "chunk has {} chars, cap is {}", char_count, hard_cap);
        }
    }

    #[test]
    fn test_overlap_clamping() {
        // Overlap > chunk_size should be clamped
        let text = "Hello world. This is a test. Another sentence here.";
        let chunks = chunk_text(text, "test.md", ChunkStrategy::FixedSize, 10, 100);
        assert!(!chunks.is_empty());
    }

    #[test]
    fn test_line_numbers_correct() {
        let text = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5";
        let chunks = chunk_text(text, "test.md", ChunkStrategy::FixedSize, 500, 0);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].line_start, 1);
        assert_eq!(chunks[0].line_end, 5);
    }

    #[test]
    fn test_recursive_strategy() {
        let text = "Paragraph one has text.\n\nParagraph two has more text.\n\nParagraph three.";
        let chunks = chunk_text(text, "test.md", ChunkStrategy::Recursive, 500, 0);
        assert!(!chunks.is_empty());
    }

    #[test]
    fn test_emoji_content() {
        let text = "Hello 🌍! This is a test 🚀. Another line 💡.";
        let chunks = chunk_text(text, "emoji.md", ChunkStrategy::Sentence, 20, 0);
        for chunk in &chunks {
            assert!(std::str::from_utf8(chunk.text.as_bytes()).is_ok());
        }
    }

    #[test]
    fn test_chunk_ids_sequential() {
        let text = "A. B. C. D. E. F. G. H.";
        let chunks = chunk_text(text, "test.md", ChunkStrategy::Sentence, 5, 0);
        for (i, chunk) in chunks.iter().enumerate() {
            assert_eq!(chunk.id, i);
        }
    }

    #[test]
    fn test_source_preserved() {
        let chunks = chunk_text("hello", "my/file.md", ChunkStrategy::FixedSize, 500, 0);
        assert_eq!(chunks[0].source, "my/file.md");
    }

    #[test]
    fn test_fixed_size_emoji_with_overlap() {
        // Regression test: FixedSize with multibyte chars + overlap must not panic
        let text = "Hello 🚀🌍💡 world! Testing 🎉 emoji overlap. More text here with 🐱 cats.";
        let chunks = chunk_text(text, "emoji.md", ChunkStrategy::FixedSize, 10, 3);
        assert!(!chunks.is_empty());
        for chunk in &chunks {
            assert!(std::str::from_utf8(chunk.text.as_bytes()).is_ok());
            // byte_end should never exceed byte_start + text.len()
            assert!(chunk.byte_end - chunk.byte_start <= chunk.text.len() + 4);
        }
    }

    #[test]
    fn test_hard_cap_adjusts_byte_end() {
        // Verify hard_cap adjusts byte_end to match truncated text
        let text = "a".repeat(5000);
        let chunks = chunk_text(&text, "big.txt", ChunkStrategy::FixedSize, 100, 0);
        for chunk in &chunks {
            assert_eq!(chunk.byte_end - chunk.byte_start, chunk.text.len(),
                "byte range should match text length after hard cap");
        }
    }

    #[test]
    fn test_strategy_from_str() {
        assert_eq!(ChunkStrategy::from_str("fixed_size"), ChunkStrategy::FixedSize);
        assert_eq!(ChunkStrategy::from_str("sentence"), ChunkStrategy::Sentence);
        assert_eq!(ChunkStrategy::from_str("paragraph"), ChunkStrategy::Paragraph);
        assert_eq!(ChunkStrategy::from_str("recursive"), ChunkStrategy::Recursive);
        assert_eq!(ChunkStrategy::from_str("unknown"), ChunkStrategy::Recursive);
    }
}
