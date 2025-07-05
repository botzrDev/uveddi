#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_embedding_creation() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.rs");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "fn main() {{}}").unwrap();

        let embedding_result =
            create_embedding("fn main() {{}}", &file_path.to_string_lossy()).await;
        assert!(embedding_result.is_ok());
        let embedding = embedding_result.unwrap();
        assert_eq!(embedding.source, file_path.to_string_lossy());
        assert!(!embedding.vector.is_empty());
    }
}
