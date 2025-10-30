//! Unit tests for semantic search modules

use super::*;
use ndarray::Array1;
use std::collections::HashMap;

#[test]
fn test_vector_index_add_and_search() {
    let mut index = VectorIndex::new();
    let chunk1 = IndexedChunk {
        id: "1".to_string(),
        text: "foo".to_string(),
        embedding: Array1::from(vec![1.0, 0.0, 0.0]),
        metadata: HashMap::new(),
        trust_score: None,
    };
    let chunk2 = IndexedChunk {
        id: "2".to_string(),
        text: "bar".to_string(),
        embedding: Array1::from(vec![0.0, 1.0, 0.0]),
        metadata: HashMap::new(),
        trust_score: None,
    };
    index.add_chunk(chunk1.clone());
    index.add_chunk(chunk2.clone());
    let query = Array1::from(vec![1.0, 0.0, 0.0]);
    let results = index.search(&query, 1);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].1.id, "1");
}

#[test]
fn test_cosine_similarity() {
    let a = Array1::from(vec![1.0, 0.0, 0.0]);
    let b = Array1::from(vec![1.0, 0.0, 0.0]);
    let c = Array1::from(vec![0.0, 1.0, 0.0]);
    assert!((cosine_similarity(&a, &b) - 1.0).abs() < 1e-6);
    assert!((cosine_similarity(&a, &c)).abs() < 1e-6);
}
