//! Semantic search and ranking for code intelligence RAG

use ndarray::{Array1, Array2};
use std::collections::HashMap;

/// Represents a code/document chunk with its embedding and metadata
#[derive(Debug, Clone)]
pub struct IndexedChunk {
    pub id: String,
    pub text: String,
    pub embedding: Array1<f32>,
    pub metadata: HashMap<String, String>,
    pub trust_score: Option<f32>,
}

/// In-memory vector index for semantic search
pub struct VectorIndex {
    pub chunks: Vec<IndexedChunk>,
}

impl VectorIndex {
    pub fn new() -> Self {
        Self { chunks: Vec::new() }
    }

    pub fn add_chunk(&mut self, chunk: IndexedChunk) {
        self.chunks.push(chunk);
    }

    /// Returns top-k most similar chunks to the query embedding, ranked by cosine similarity
    pub fn search(&self, query_embedding: &Array1<f32>, k: usize) -> Vec<(f32, &IndexedChunk)> {
        let mut scored: Vec<(f32, &IndexedChunk)> = self.chunks.iter()
            .map(|chunk| (cosine_similarity(query_embedding, &chunk.embedding), chunk))
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        scored.into_iter().take(k).collect()
    }
}

/// Compute cosine similarity between two vectors
pub fn cosine_similarity(a: &Array1<f32>, b: &Array1<f32>) -> f32 {
    let dot = a.dot(b);
    let norm_a = a.dot(a).sqrt();
    let norm_b = b.dot(b).sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}

pub mod embedding;
pub mod hybrid;
pub mod mmr;
