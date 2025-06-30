//! Unit tests for hybrid RRF and MMR modules

use super::*;
use crate::semantic_search::{IndexedChunk};
use crate::semantic_search::hybrid::{reciprocal_rank_fusion, SparseResult};
use crate::semantic_search::mmr::maximal_marginal_relevance;
use ndarray::Array1;
use std::collections::HashMap;

#[test]
fn test_reciprocal_rank_fusion() {
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
    let dense = vec![(&chunk1, 0.9), (&chunk2, 0.8)];
    let sparse = vec![SparseResult { score: 0.7, chunk: &chunk2 }];
    let fused = reciprocal_rank_fusion(&dense, &sparse, 2, 60);
    assert_eq!(fused.len(), 2);
    assert!(fused.iter().any(|c| c.id == "1"));
    assert!(fused.iter().any(|c| c.id == "2"));
}

#[test]
fn test_maximal_marginal_relevance() {
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
    let query = Array1::from(vec![1.0, 0.0, 0.0]);
    let candidates = vec![&chunk1, &chunk2];
    let selected = maximal_marginal_relevance(&query, &candidates, 0.5, 2);
    assert_eq!(selected.len(), 2);
    assert!(selected.iter().any(|c| c.id == "1"));
    assert!(selected.iter().any(|c| c.id == "2"));
}
