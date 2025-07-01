//! Error handling and robustness tests

#[cfg(test)]
mod tests {
    use uveddi::analysis::dependency_extractor::DependencyExtractor;
    use uveddi::ast::tree_sitter::{AstParser, AstError};
    use tempfile::tempdir;
    use std::fs::File;
    use std::io::Write;

    fn create_temp_file(dir: &tempfile::TempDir, name: &str, content: &str) -> std::path::PathBuf {
        let file_path = dir.path().join(name);
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "{}", content).unwrap();
        file_path
    }

    #[test]
    fn handles_malformed_files() {
        let dir = tempdir().unwrap();
        let file_path = create_temp_file(&dir, "bad.rs", "mod { bad syntax");
        let mut parser = AstParser::new().unwrap();
        let parsed = parser.parse_file(&file_path);
        assert!(parsed.is_err());
    }

    #[test]
    fn handles_unsupported_languages() {
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
        // Simulate missing config/API key by trying to create a DependencyExtractor if it required config
        // For now, just ensure DependencyExtractor::new() does not panic and returns Result
        let extractor = DependencyExtractor::new();
        assert!(extractor.is_ok());
        // If you add config checks in the future, add more assertions here
    }
}
