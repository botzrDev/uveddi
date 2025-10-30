//! Error handling and robustness tests
//!
//! This module tests error handling for malformed files, unsupported languages, and missing configuration.
//! It ensures that the analysis engine and AST parser fail gracefully and return appropriate errors.
//! Any new error scenarios should be added here as separate tests.

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;
    use std::sync::{Arc, Mutex};
    use tempfile::tempdir;
    use uveddi::analysis::errors::AnalysisError;
    use uveddi::analysis::DependencyExtractor;
    use uveddi::ast::tree_sitter_impl::{AstError, AstParser};

    fn create_temp_file(dir: &tempfile::TempDir, name: &str, content: &str) -> std::path::PathBuf {
        let file_path = dir.path().join(name);
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "{content}").unwrap();
        file_path
    }

    #[test]
    fn handles_malformed_files() {
        // Test that parsing a malformed file returns an error
        let dir = tempdir().unwrap();
        let file_path = create_temp_file(&dir, "bad.rs", "mod { bad syntax");
        let mut parser = AstParser::new().unwrap();
        let parsed = parser.parse_file(&file_path);
        assert!(parsed.is_err());
    }

    #[test]
    fn handles_unsupported_languages() {
        // Test that parsing a file with an unsupported language returns the correct error
        let dir = tempdir().unwrap();
        let file_path = create_temp_file(&dir, "file.unknown", "some content");
        let mut parser = AstParser::new().unwrap();
        let parsed = parser.parse_file(&file_path);
        match parsed {
            Err(AstError::UnsupportedLanguage(_)) => (),
            Err(AstError::Other(msg)) if msg.contains("Unsupported file type") => (),
            Err(e) => panic!("Expected UnsupportedLanguage or security error for unsupported file type, got: {:?}", e),
            Ok(_) => panic!("Expected error, got Ok"),
        }
    }

    #[test]
    fn handles_timeouts_and_missing_config() {
        // Test that missing config or API keys do not cause panics and return Result
        // For now, just ensure DependencyExtractor::new() does not panic and returns Result
        let extractor = DependencyExtractor::new();
        assert!(extractor.is_ok());
        // If you add config checks in the future, add more assertions here
    }

    // UV-276: Unwrap replacement tests

    #[test]
    fn test_parse_error_creation() {
        let error = AnalysisError::parse_error("Failed to parse invalid syntax");
        assert!(matches!(error, AnalysisError::ParseError { .. }));
        assert_eq!(
            error.to_string(),
            "Parse operation failed: Failed to parse invalid syntax"
        );
    }

    #[test]
    fn test_query_error_creation() {
        let error = AnalysisError::query_error("Missing capture group");
        assert!(matches!(error, AnalysisError::QueryError(_)));
        assert_eq!(
            error.to_string(),
            "Tree-sitter query error: Missing capture group"
        );
    }

    #[test]
    fn test_collection_access_error_creation() {
        let error = AnalysisError::collection_access_error("Index out of bounds");
        assert!(matches!(error, AnalysisError::CollectionAccessError { .. }));
        assert_eq!(
            error.to_string(),
            "Collection access failed: Index out of bounds"
        );
    }

    #[test]
    fn test_lock_error_creation() {
        let error = AnalysisError::lock_error("Mutex poisoned");
        assert!(matches!(error, AnalysisError::LockError { .. }));
        assert_eq!(error.to_string(), "Mutex lock failed: Mutex poisoned");
    }

    #[test]
    fn test_safe_collection_access_patterns() {
        let empty_vec: Vec<String> = vec![];

        // Test safe indexing pattern
        let result = empty_vec
            .get(0)
            .ok_or_else(|| AnalysisError::collection_access_error("Empty collection"));

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AnalysisError::CollectionAccessError { .. }
        ));
    }

    #[test]
    fn test_poisoned_mutex_error_handling() {
        // Create a mutex that we'll poison
        let mutex = Arc::new(Mutex::new(42));
        let mutex_clone = Arc::clone(&mutex);

        // Simulate poison by panicking while holding the lock
        let handle = std::thread::spawn(move || {
            let _guard = mutex_clone.lock().unwrap();
            panic!("Simulated panic to poison mutex");
        });

        // Wait for thread to panic
        let _ = handle.join();

        // Now the mutex should be poisoned
        let result = mutex
            .lock()
            .map_err(|_| AnalysisError::lock_error("Mutex poisoned"));
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AnalysisError::LockError { .. }
        ));
    }
}
