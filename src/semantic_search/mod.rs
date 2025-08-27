//! Semantic Search and Vector-Based Code Intelligence
//!
//! This module provides semantic search capabilities for Retrieval-Augmented Generation (RAG)
//! in code analysis contexts. It enables finding semantically similar code chunks, functions,
//! and documentation based on vector embeddings rather than just text matching.
//!
//! # Key Components
//!
//! ## Vector Index
//! An in-memory vector database that stores code chunks with their embeddings and metadata.
//! Supports efficient similarity search using cosine similarity.
//!
//! ## Indexed Chunks
//! Code segments (functions, classes, modules) that have been processed into vector
//! representations along with contextual metadata for enhanced search results.
//!
//! # Use Cases
//!
//! - **Code Pattern Detection**: Find similar code patterns across the codebase
//! - **Example Retrieval**: Locate relevant code examples for AI explanations
//! - **Context Enhancement**: Provide relevant context for AI-powered analysis
//! - **Duplicate Detection**: Identify potential code duplication through semantic similarity
//!
//! # Example Usage
//!
//! ```rust
//! use uveddi::semantic_search::{VectorIndex, IndexedChunk};
//! use ndarray::Array1;
//! use std::collections::HashMap;
//!
//! // Create a vector index
//! let mut index = VectorIndex::new();
//!
//! // Add code chunks with embeddings
//! let chunk = IndexedChunk {
//!     id: "function_001".to_string(),
//!     text: "fn calculate_score(items: &[Item]) -> f64 { ... }".to_string(),
//!     embedding: Array1::zeros(384), // Would be actual embedding vector
//!     metadata: {
//!         let mut meta = HashMap::new();
//!         meta.insert("file_path".to_string(), "src/scoring.rs".to_string());
//!         meta.insert("function_name".to_string(), "calculate_score".to_string());
//!         meta
//!     },
//!     trust_score: Some(0.9),
//! };
//!
//! index.add_chunk(chunk);
//!
//! // Search for similar chunks
//! let query_embedding = Array1::zeros(384); // Query vector
//! let results = index.search(&query_embedding, 5);
//! ```
//!
//! # Architecture Integration
//!
//! The semantic search module integrates with:
//! - **AI Engine**: Provides context for AI-powered explanations
//! - **Analysis Engine**: Enhances pattern detection capabilities  
//! - **Report Generator**: Finds relevant examples for issue explanations

use ndarray::Array1;
use std::collections::HashMap;
// Removed unused tracing imports

/// Code chunk with vector embedding and contextual metadata
///
/// Represents a piece of code (function, class, module, etc.) that has been
/// processed into a vector embedding for semantic search. Includes metadata
/// for context and optional trust scoring for result ranking.
///
/// # Fields
///
/// * `id` - Unique identifier for this chunk (e.g., "file::function_name")
/// * `text` - The actual code or text content
/// * `embedding` - Vector representation of the content for similarity search
/// * `metadata` - Additional context like file path, function name, language
/// * `trust_score` - Optional quality/reliability score (0.0 to 1.0)
#[derive(Debug, Clone)]
pub struct IndexedChunk {
    /// Unique identifier for this code chunk
    pub id: String,
    /// The raw text content of the code chunk
    pub text: String,
    /// Vector embedding representation for semantic similarity
    pub embedding: Array1<f32>,
    /// Contextual metadata (file path, function name, language, etc.)
    pub metadata: HashMap<String, String>,
    /// Optional trust/quality score for ranking results (0.0 to 1.0)
    pub trust_score: Option<f32>,
}

/// In-memory vector database for semantic code search
///
/// Provides efficient storage and retrieval of code chunks based on
/// vector similarity. Uses cosine similarity for ranking results
/// and supports metadata-based filtering.
///
/// # Performance Characteristics
///
/// - **Search Time**: O(n * d) where n is chunk count, d is embedding dimension
/// - **Memory Usage**: Linear with number of chunks and embedding size
/// - **Scalability**: Suitable for projects with thousands of code chunks
///
/// For larger codebases, consider using external vector databases like
/// Pinecone, Weaviate, or Chroma.
/// Alias for a single semantic search result: a borrowed chunk and its similarity score
pub type SearchResult<'a> = (&'a IndexedChunk, f32);

/// In-memory vector index for semantic search over code chunks
pub struct VectorIndex {
    /// Collection of indexed code chunks with embeddings
    pub chunks: Vec<IndexedChunk>,
}

impl VectorIndex {
    /// Create a new empty vector index
    ///
    /// # Examples
    ///
    /// ```rust
    /// use uveddi::semantic_search::VectorIndex;
    ///
    /// let index = VectorIndex::new();
    /// assert_eq!(index.chunks.len(), 0);
    /// ```
    pub fn new() -> Self {
        Self { chunks: Vec::new() }
    }

    /// Add a code chunk to the searchable index
    ///
    /// # Arguments
    ///
    /// * `chunk` - The indexed chunk to add to the search index
    ///
    /// # Examples
    ///
    /// ```rust
    /// use uveddi::semantic_search::{VectorIndex, IndexedChunk};
    /// use ndarray::Array1;
    /// use std::collections::HashMap;
    ///
    /// let mut index = VectorIndex::new();
    /// let chunk = IndexedChunk {
    ///     id: "test_chunk".to_string(),
    ///     text: "fn test() {}".to_string(),
    ///     embedding: Array1::zeros(10),
    ///     metadata: HashMap::new(),
    ///     trust_score: None,
    /// };
    ///
    /// index.add_chunk(chunk);
    /// assert_eq!(index.chunks.len(), 1);
    /// ```
    pub fn add_chunk(&mut self, chunk: IndexedChunk) {
        self.chunks.push(chunk);
    }

    /// Search for the most semantically similar code chunks
    ///
    /// Returns the top-k most similar chunks ranked by cosine similarity
    /// to the query embedding. Results are sorted by similarity score
    /// in descending order.
    ///
    /// # Arguments
    ///
    /// * `query_embedding` - Vector representation of the search query
    /// * `k` - Maximum number of results to return
    ///
    /// # Returns
    ///
    /// Vector of tuples containing (chunk_reference, similarity_score)
    /// sorted by similarity in descending order.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use uveddi::semantic_search::{VectorIndex, IndexedChunk};
    /// # use ndarray::Array1;
    /// # use std::collections::HashMap;
    /// # let mut index = VectorIndex::new();
    /// # let chunk = IndexedChunk {
    /// #     id: "test".to_string(),
    /// #     text: "test".to_string(),
    /// #     embedding: Array1::ones(5),
    /// #     metadata: HashMap::new(),
    /// #     trust_score: None,
    /// # };
    /// # index.add_chunk(chunk);
    /// let query = Array1::ones(5);
    /// let results = index.search(&query, 3);
    ///
    /// // Results are ordered by similarity score
    /// for (chunk, score) in results {
    ///     println!("Found chunk '{}' with similarity: {:.3}", chunk.id, score);
    /// }
    /// ```
    pub fn search(&self, query_embedding: &Array1<f32>, k: usize) -> Vec<SearchResult<'_>> {
        let mut scored: Vec<(f32, &IndexedChunk)> = self
            .chunks
            .iter()
            .map(|chunk| (cosine_similarity(query_embedding, &chunk.embedding), chunk))
            .collect::<Vec<(f32, &IndexedChunk)>>(); // Materialize as Vec of (score, &IndexedChunk) pairs for clarity
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        scored
            .into_iter()
            .take(k)
            .map(|(score, chunk)| (chunk, score))
            .collect::<Vec<SearchResult<'_>>>()
    }
}

/// Compute cosine similarity between two vectors
///
/// Calculates the cosine similarity between two embedding vectors, which measures
/// the cosine of the angle between them. Values range from -1 to 1, where:
/// - 1.0 indicates identical direction (perfect similarity)
/// - 0.0 indicates orthogonal vectors (no similarity)  
/// - -1.0 indicates opposite direction (perfect dissimilarity)
///
/// # Arguments
///
/// * `a` - First vector for comparison
/// * `b` - Second vector for comparison
///
/// # Returns
///
/// Cosine similarity score between -1.0 and 1.0
///
/// # Examples
///
/// ```rust
/// use ndarray::Array1;
/// use uveddi::semantic_search::cosine_similarity;
///
/// let vec1 = Array1::from(vec![1.0, 0.0, 0.0]);
/// let vec2 = Array1::from(vec![1.0, 0.0, 0.0]);
/// let similarity = cosine_similarity(&vec1, &vec2);
/// assert!((similarity - 1.0).abs() < 1e-6); // Perfect similarity
/// ```
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

#[cfg(test)]
mod embedding_tests;
