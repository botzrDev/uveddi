//! Maximal Marginal Relevance (MMR) for diversity in context selection

use super::IndexedChunk;
use ndarray::Array1;

/// Selects a diverse set of top-k chunks using Maximal Marginal Relevance (MMR).
///
/// MMR balances relevance to the query with diversity among selected chunks, helping
/// avoid redundancy in context selection for retrieval-augmented generation (RAG).
///
/// # Arguments
///
/// * `query_embedding` - Embedding vector representing the search query.
/// * `candidates` - Slice of references to candidate `IndexedChunk`s.
/// * `lambda` - Trade-off parameter (0.0 = only diversity, 1.0 = only relevance).
/// * `k` - Number of results to select.
///
/// # Returns
///
/// * `Vec<&IndexedChunk>` - Top-k diverse and relevant chunks.
///
/// # Example
/// ```rust
/// use uveddi::semantic_search::{maximal_marginal_relevance, IndexedChunk};
/// // ... setup query_embedding and candidates ...
/// let selected = maximal_marginal_relevance(&query_embedding, &candidates, 0.5, 5);
/// ```
pub fn maximal_marginal_relevance<'a>(
    query_embedding: &Array1<f32>,
    candidates: &[&'a IndexedChunk],
    lambda: f32,
    k: usize,
) -> Vec<&'a IndexedChunk> {
    let mut selected = Vec::new();
    let mut remaining: Vec<_> = candidates.to_vec();
    while selected.len() < k && !remaining.is_empty() {
        let mut best_score = f32::MIN;
        let mut best_idx = 0;
        for (i, chunk) in remaining.iter().enumerate() {
            let relevance = super::cosine_similarity(query_embedding, &chunk.embedding);
            let diversity = selected
                .iter()
                .map(|s: &IndexedChunk| super::cosine_similarity(&s.embedding, &chunk.embedding))
                .fold(0.0, f32::max);
            let mmr_score = lambda * relevance - (1.0 - lambda) * diversity;
            if mmr_score > best_score {
                best_score = mmr_score;
                best_idx = i;
            }
        }
        selected.push(remaining.remove(best_idx));
    }
    selected
}
