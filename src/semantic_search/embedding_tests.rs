//! Unit tests for DummyEmbeddingModel

use super::embedding::{EmbeddingModel, DummyEmbeddingModel};

#[test]
fn test_dummy_embedding_model_returns_fixed_length() {
    let model = DummyEmbeddingModel;
    let emb = model.embed("hello world");
    assert_eq!(emb.len(), 768);
}

#[test]
fn test_dummy_embedding_model_different_texts() {
    let model = DummyEmbeddingModel;
    let emb1 = model.embed("foo");
    let emb2 = model.embed("bar");
    assert_eq!(emb1.len(), emb2.len());
    assert_ne!(emb1, emb2);
}
