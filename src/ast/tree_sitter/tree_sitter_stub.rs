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

impl SourceLanguage {
    /// Determine language from file path
    pub fn from_path(path: &Path) -> Self {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("rs") => SourceLanguage::Rust,
            Some("py") => SourceLanguage::Python,
            Some("js") | Some("jsx") | Some("ts") | Some("tsx") => SourceLanguage::JavaScript,
            _ => SourceLanguage::Rust, // Default to Rust
        }
    }
}

/// Parsed file structure stub
#[derive(Debug, Clone)]
pub struct ParsedFile {
    pub file_path: String,
    pub language: SourceLanguage,
    pub content: String,
    pub tree: Option<StubTree>,
    pub custom_ast: Option<CustomAst>,
    // Add source field to match real implementation
    pub source: Option<String>,
}

/// Custom AST representation stub
#[derive(Debug, Clone)]
pub enum CustomAst {
    File { items: Vec<CustomAst> },
    Struct { name: String, methods: Vec<String> },
    Function { name: String, params: Vec<String> },
    Variable { name: String },
}

#[derive(Debug, Clone)]
pub struct StubTree;

impl StubTree {
    pub fn as_ref(&self) -> Option<&Self> { None }
    pub fn root_node(&self) -> StubNode { StubNode }
    // Add language method for API compatibility
    pub fn language(&self) -> StubLanguage { StubLanguage }
}

#[derive(Debug, Clone)]
pub struct StubNode;

impl StubNode {
    pub fn kind(&self) -> &str {
        ""
    }
    
    pub fn start_byte(&self) -> usize {
        0
    }
    
    pub fn end_byte(&self) -> usize {
        0
    }
    
    pub fn start_position(&self) -> StubPoint {
        StubPoint { row: 0, column: 0 }
    }
    
    pub fn end_position(&self) -> StubPoint {
        StubPoint { row: 0, column: 0 }
    }
    
    pub fn parent(&self) -> Option<StubNode> {
        None
    }
    
    pub fn child(&self, _index: usize) -> Option<StubNode> {
        None
    }
    
    pub fn child_count(&self) -> usize {
        0
    }
    
    pub fn utf8_text<'a>(&self, _source: &'a [u8]) -> Result<&'a str, std::str::Utf8Error> {
        Ok("")
    }
    
    pub fn walk(&self) -> StubTreeCursor {
        StubTreeCursor
    }
}

#[derive(Debug, Clone)]
pub struct StubPoint {
    pub row: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct StubTreeCursor;

impl StubTreeCursor {
    pub fn goto_first_child(&mut self) -> bool {
        false
    }
    
    pub fn goto_next_sibling(&mut self) -> bool {
        false
    }
    
    pub fn node(&self) -> StubNode {
        StubNode
    }
}

#[derive(Debug, Clone)]
pub struct StubLanguage;

impl StubLanguage {
    pub fn node_kind_for_id(&self, _id: u16) -> Option<&str> {
        None
    }
}

#[derive(Debug, Clone)]
pub struct StubCapture {
    pub index: u32,
    pub node: StubNode,
}

#[derive(Debug, Clone)]
pub struct StubMatch {
    pub pattern_index: usize,
    pub captures: Vec<StubCapture>,
}

// Re-export stub types with tree-sitter names for compatibility
pub use StubNode as Node;
pub use StubTree as Tree;

#[derive(Debug, Clone)]
pub struct Query;

impl Query {
    pub fn new(_lang: &StubLanguage, _pattern: &str) -> Result<Self, AstError> {
        Err(AstError::TreeSitterDisabled)
    }
}

#[derive(Debug, Clone)]
pub struct QueryCursor;

impl QueryCursor {
    pub fn new() -> Self {
        QueryCursor
    }

    pub fn matches<'a>(&'a mut self, _query: &Query, _node: StubNode, _source: &[u8]) -> Vec<StubMatch> {
        Vec::new()
    }
}

// Re-export stub match type
pub use StubMatch as Match;

#[derive(Debug, Clone)]
pub struct Parser;

impl Parser {
    pub fn new() -> Result<Self, AstError> {
        Err(AstError::TreeSitterDisabled)
    }

    pub fn set_language(&mut self, _language: ()) -> Result<(), AstError> {
        Err(AstError::TreeSitterDisabled)
    }

    pub fn parse(&mut self, _source: &[u8], _old_tree: Option<&StubTree>) -> Result<StubTree, AstError> {
        Err(AstError::TreeSitterDisabled)
    }
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
    
    // Add parse_file method for API compatibility
    pub fn parse_file(&self, file_path: &Path) -> Result<ParsedFile, AstError> {
        let content = std::fs::read_to_string(file_path)?;
        Ok(ParsedFile {
            file_path: file_path.to_string_lossy().to_string(),
            language: SourceLanguage::from_path(file_path),
            content: content.clone(),
            source: Some(content),
            tree: None,
            custom_ast: None,
        })
    }

    /// Add a language parser (stub - always returns error)
    pub fn add_language(&mut self, _lang: SourceLanguage, _parser: Parser) {
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
            source: Some(content.to_string()),
            tree: None,
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
