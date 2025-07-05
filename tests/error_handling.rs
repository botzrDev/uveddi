//! Error handling and robustness tests
//!
//! This module tests error handling for malformed files, unsupported languages, and missing configuration.
//! It ensures that the analysis engine and AST parser fail gracefully and return appropriate errors.
//! Any new error scenarios should be added here as separate tests.

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;
    use uveddi::analysis::DependencyExtractor;
    use uveddi::ast::tree_sitter::{AstError, AstParser};

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
            _ => panic!("Expected UnsupportedLanguage error"),
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
}
