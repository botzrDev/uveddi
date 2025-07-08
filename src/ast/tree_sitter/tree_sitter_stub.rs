// Stub implementation file for tree-sitter disabled builds
// UV-97: Tree-sitter feature gating implementation

use std::collections::HashMap;
use std::path::Path;
use crate::error::UveddiError;

/// Tree-sitter parser stub (feature disabled)
#[derive(Debug, Clone)]
pub struct AstParser {
    // Empty struct when tree-sitter is disabled
}

/// Cached AST structure stub
#[derive(Debug, Clone)]
pub struct CachedAst {
    pub timestamp: std::time::SystemTime,
}

/// Source language enumeration (always available)
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum SourceLanguage {
    Rust,
    Python,
    JavaScript,
}

/// Parsed file structure stub
#[derive(Debug, Clone)]
pub struct ParsedFile {
    pub file_path: String,
    pub language: SourceLanguage,
    pub content: String,
    pub custom_ast: Option<CustomAst>,
}

/// Custom AST representation stub
#[derive(Debug, Clone)]
pub enum CustomAst {
    File { items: Vec<CustomAst> },
    Struct { name: String, methods: Vec<String> },
    Function { name: String, params: Vec<String> },
    Variable { name: String },
}

/// AST error types for stub operations
#[derive(Debug, thiserror::Error)]
pub enum AstError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Tree-sitter feature not enabled")]
    TreeSitterDisabled,
    #[error("Parse failed: tree-sitter not available")]
    ParseFailed,
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),
    #[error("Other error: {0}")]
    Other(String),
}

impl AstParser {
    /// Create a new AST parser stub
    pub fn new() -> Result<Self, AstError> {
        Ok(AstParser {})
    }

    /// Add a language parser (stub - always returns error)
    pub fn add_language(&mut self, _lang: SourceLanguage, _parser: ()) {
        // No-op in stub implementation
    }

    /// Parse a file with caching (stub - always returns error)
    pub fn parse_with_cache(
        &mut self,
        file_path: &Path,
        content: &str,
        language: SourceLanguage,
    ) -> Result<ParsedFile, AstError> {
        // Return a minimal ParsedFile without tree-sitter functionality
        Ok(ParsedFile {
            file_path: file_path.to_string_lossy().to_string(),
            language,
            content: content.to_string(),
            custom_ast: None,
        })
    }

    /// Clear the AST cache (stub - no-op)
    pub fn clear_cache(&mut self) -> Result<(), AstError> {
        Ok(())
    }

    /// Get available parsers (stub - returns empty list)
    pub fn get_available_parsers(&self) -> Vec<SourceLanguage> {
        Vec::new()
    }
}

impl Default for AstParser {
    fn default() -> Self {
        Self::new().expect("Failed to create default AstParser stub")
    }
}

impl ParsedFile {
    /// Get cache path for a file (stub implementation)
    pub fn cache_path(file_path: &Path) -> std::path::PathBuf {
        let mut cache_path = std::env::temp_dir();
        cache_path.push("uveddi_ast_cache_stub");
        cache_path.push(format!("{}.cache", 
            file_path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
        ));
        cache_path
    }
}

/// Create a test AST parser stub for testing
pub fn create_test_ast_parser() -> Result<AstParser, AstError> {
    AstParser::new()
}
