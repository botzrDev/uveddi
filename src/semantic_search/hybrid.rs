//! Hybrid search and Reciprocal Rank Fusion (RRF) for code intelligence

use super::IndexedChunk;

/// Placeholder for BM25 or sparse keyword search result
#[derive(Debug)]
pub struct SparseResult<'a> {
    /// BM25 or keyword search score
    pub score: f32,
    /// Reference to the matched chunk
    pub chunk: &'a IndexedChunk,
}

/// Fuses dense and sparse search results using Reciprocal Rank Fusion (RRF).
///
/// RRF combines rankings from multiple sources (e.g., dense vector search and sparse keyword search)
/// to produce a single, more robust ranking. This is useful for hybrid code search and retrieval-augmented generation.
///
/// # Arguments
///
/// * `dense` - Slice of tuples of (`IndexedChunk`, score) from dense (embedding) search.
/// * `sparse` - Slice of `SparseResult` from sparse (keyword/BM25) search.
/// * `k` - Number of top results to return.
/// * `rrf_k` - RRF parameter controlling the influence of rank position.
///
/// # Returns
///
/// * `Vec<&IndexedChunk>` - Top-k fused results ranked by combined RRF score.
///
/// # Example
/// ```rust
/// use uveddi::semantic_search::{reciprocal_rank_fusion, IndexedChunk, SparseResult};
/// // ... setup dense and sparse results ...
/// let fused = reciprocal_rank_fusion(&dense, &sparse, 10, 60);
/// ```
pub fn reciprocal_rank_fusion<'a>(
    dense: &[(&'a IndexedChunk, f32)],
    sparse: &[SparseResult<'a>],
    k: usize,
    rrf_k: usize,
) -> Vec<&'a IndexedChunk> {
    use std::collections::HashMap;
    let mut scores: HashMap<&'a str, f32> = HashMap::new();
    for (rank, (chunk, _)) in dense.iter().enumerate() {
        let rrf_score = 1.0 / (rrf_k as f32 + rank as f32);
        *scores.entry(&chunk.id).or_insert(0.0) += rrf_score;
    }
    for (rank, result) in sparse.iter().enumerate() {
        let rrf_score = 1.0 / (rrf_k as f32 + rank as f32);
        *scores.entry(&result.chunk.id).or_insert(0.0) += rrf_score;
    }
    // Sort by combined RRF score
    let mut ranked: Vec<_> = scores.into_iter().collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    ranked
        .into_iter()
        .take(k)
        .filter_map(|(id, _)| {
            dense
                .iter()
                .find(|(chunk, _)| &chunk.id == id)
                .map(|(chunk, _)| *chunk)
                .or_else(|| sparse.iter().find(|r| &r.chunk.id == id).map(|r| r.chunk))
        })
        .collect()
}
