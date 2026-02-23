pub mod chunker;
pub mod index;
pub mod search;
pub mod format;

pub use chunker::{ChunkStrategy, Chunk, chunk_text};
pub use index::{IndexMeta, IndexStatus, write_index, read_meta, read_chunk, check_freshness};
pub use search::{SearchResult, normalize, dot_similarity, search};
pub use format::format_context_with_citations;
