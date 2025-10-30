//! Test helpers for security tests

use uveddi::ast::{ParsedFile, SourceLanguage};
use std::path::PathBuf;
use std::sync::Arc;

/// Create a test ParsedFile with minimal required fields
pub fn create_test_parsed_file(path: &str, language: SourceLanguage) -> ParsedFile {
    ParsedFile {
        file_path: Arc::new(PathBuf::from(path)),
        language,
        tree: None,
        source: Arc::new(String::new()),
        custom_ast: Arc::new(None),
        modified_at: uveddi::analysis::cache::wrappers::ArchivableSystemTime::now(),
    }
}

/// Create a test ParsedFile with source content
pub fn create_test_parsed_file_with_content(path: &str, language: SourceLanguage, content: &str) -> ParsedFile {
    ParsedFile {
        file_path: Arc::new(PathBuf::from(path)),
        language,
        tree: None,
        source: Arc::new(content.to_string()),
        custom_ast: Arc::new(None),
        modified_at: uveddi::analysis::cache::wrappers::ArchivableSystemTime::now(),
    }
}