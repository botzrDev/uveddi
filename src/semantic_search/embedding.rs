//! Embedding model integration for semantic search

use ndarray::Array1;

/// Trait for embedding models (local or API)
pub trait EmbeddingModel {
    /// Generate an embedding for a given text/code chunk
    fn embed(&self, text: &str) -> Array1<f32>;
}

/// Dummy embedding model for testing (returns a fixed vector)
pub struct DummyEmbeddingModel;

impl EmbeddingModel for DummyEmbeddingModel {
    fn embed(&self, text: &str) -> Array1<f32> {
        // For demonstration, hash the text and fill a vector
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        text.hash(&mut hasher);
        let hash = hasher.finish();
        Array1::from(vec![hash as f32; 768])
    }
}
