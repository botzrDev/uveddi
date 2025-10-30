#[cfg(test)]
mod tests {
    use crate::semantic_search::{IndexedChunk, embedding::DummyEmbeddingModel, embedding::EmbeddingModel};
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    /// Create an embedding for a code chunk with metadata
    /// 
    /// # Arguments
    /// * `text` - The code text to embed
    /// * `source_path` - Path to the source file
    /// 
    /// # Returns
    /// Result containing the IndexedChunk with embedding
    async fn create_embedding(text: &str, source_path: &str) -> Result<IndexedChunk, Box<dyn std::error::Error>> {
        let model = DummyEmbeddingModel;
        let embedding = model.embed(text);
        
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), source_path.to_string());
        metadata.insert("language".to_string(), "rust".to_string());
        
        // Generate a unique ID based on source path and content hash
        let id = format!("{}::{}", source_path, text.len());
        
        Ok(IndexedChunk {
            id,
            text: text.to_string(),
            embedding,
            metadata,
            trust_score: Some(0.8), // Default trust score
        })
    }

    #[tokio::test]
    async fn test_embedding_creation() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.rs");
        let mut file = File::create(&file_path).unwrap();
        let code_content = "fn main() {}";
        writeln!(file, "{}", code_content).unwrap();

        // Test embedding creation
        let embedding_result = create_embedding(code_content, &file_path.to_string_lossy()).await;
        assert!(embedding_result.is_ok(), "Failed to create embedding");
        
        let embedding = embedding_result.unwrap();
        
        // Verify the embedding structure
        assert_eq!(embedding.text, code_content);
        assert!(embedding.metadata.contains_key("source"));
        assert_eq!(embedding.metadata.get("source").unwrap(), &file_path.to_string_lossy());
        assert_eq!(embedding.metadata.get("language").unwrap(), "rust");
        assert!(embedding.trust_score.is_some());
        assert_eq!(embedding.trust_score.unwrap(), 0.8);
        
        // Verify embedding vector properties
        assert!(!embedding.embedding.is_empty(), "Embedding vector should not be empty");
        assert_eq!(embedding.embedding.len(), 768, "Expected 768-dimensional embedding from DummyEmbeddingModel");
        
        // Verify ID format
        let expected_id = format!("{}::{}", file_path.to_string_lossy(), code_content.len());
        assert_eq!(embedding.id, expected_id);
    }
}
