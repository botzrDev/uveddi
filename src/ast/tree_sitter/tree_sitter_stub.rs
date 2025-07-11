// Stub implementation file for tree-sitter disabled builds
// UV-97: Tree-sitter feature gating implementation

use crate::error::UveddiError;
use std::collections::HashMap;
use std::path::Path;

/// Placeholder documentation for public items

// Stub types for tree-sitter when feature is disabled
// These maintain API compatibility but return errors or empty results

/// Stub for tree_sitter::Query
#[derive(Debug, Clone)]
pub struct Query;

/// Stub for tree_sitter::QueryCursor  
#[derive(Debug, Clone)]
pub struct QueryCursor;

/// Stub for tree_sitter::Node
/// A stub for `tree_sitter::Node<'a>`. This struct MUST include a lifetime
/// parameter to maintain API compatibility with the real tree-sitter Node.
#[derive(Debug, Clone, Copy)]
pub struct Node<'a> {
    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> Node<'a> {
    pub fn new() -> Self {
        Node {
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn child_count(&self) -> usize {
        0
    }
    pub fn child(&self, _index: usize) -> Option<Node<'a>> {
        None
    }
    pub fn kind(&self) -> &'static str {
        "stub"
    }
    pub fn start_byte(&self) -> usize {
        0
    }
    pub fn end_byte(&self) -> usize {
        0
    }
    pub fn start_position(&self) -> Point {
        Point { row: 0, column: 0 }
    }
    pub fn end_position(&self) -> Point {
        Point { row: 0, column: 0 }
    }
    pub fn utf8_text(&self, _source: &'a [u8]) -> Result<&'a str, std::str::Utf8Error> {
        Ok("")
    }
    pub fn parent(&self) -> Option<Node<'a>> {
        None
    }
    pub fn next_sibling(&self) -> Option<Node<'a>> {
        None
    }
    pub fn prev_sibling(&self) -> Option<Node<'a>> {
        None
    }
    pub fn child_by_field_name(&self, _name: &str) -> Option<Node<'a>> {
        None
    }
    pub fn walk(&self) -> TreeCursor {
        TreeCursor
    }
    pub fn id(&self) -> usize {
        0
    }
    pub fn name(&self) -> &'static str {
        "stub"
    }
}

/// Stub for tree_sitter::TreeCursor
#[derive(Debug, Clone)]
pub struct TreeCursor;

/// Stub for tree_sitter::Tree
#[derive(Debug, Clone)]
pub struct Tree;

impl Tree {
    pub fn root_node(&self) -> Node<'_> {
        Node::new()
    }

    pub fn language(&self) -> () {
        ()
    }
}

/// Stub for tree_sitter::Parser
#[derive(Debug, Clone)]
pub struct Parser;

/// Additional stub types needed
#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub row: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct QueryMatch<'a> {
    pub captures: Vec<QueryCapture<'a>>,
}

#[derive(Debug, Clone)]
pub struct QueryCapture<'a> {
    pub node: Node<'a>,
    pub index: u32,
}

/// Source language enumeration (always available)
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum SourceLanguage {
    Rust,
    Python,
    JavaScript,
}

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

/// Parsed file structure stub
#[derive(Debug, Clone)]
pub struct ParsedFile {
    pub file_path: String,
    pub language: SourceLanguage,
    pub content: String,
    pub tree: Option<Tree>,
    pub custom_ast: Option<CustomAst>,
    pub source: Option<String>,
}

/// Custom AST representation stub
#[derive(Debug, Clone)]
pub enum CustomAst {
    File {
        items: Vec<CustomAst>,
    },
    Struct {
        name: String,
        methods: Vec<String>,
    },
    Function {
        name: String,
        parameters: Vec<String>,
    },
    Variable {
        name: String,
        value_type: String,
    },
}

/// AST error types for stub implementation
#[derive(Debug, thiserror::Error)]
pub enum AstError {
    #[error("Tree-sitter feature not enabled: {0}")]
    FeatureNotEnabled(String),
    #[error("Tree-sitter functionality disabled")]
    TreeSitterDisabled,
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("IO error: {0}")]
    Io(std::io::Error),
    #[error("Cache error: {0}")]
    CacheError(String),
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),
}

// Stub implementations for tree-sitter types
impl Query {
    pub fn new(_language: &(), _query: &str) -> Result<Self, AstError> {
        Err(AstError::FeatureNotEnabled(
            "tree-sitter feature not enabled".to_string(),
        ))
    }

    pub fn capture_names(&self) -> &[&str] {
        &[]
    }
}

impl QueryCursor {
    pub fn new() -> Self {
        Self
    }

    pub fn captures<'a>(
        &'a mut self,
        _query: &'a Query,
        _node: Node<'a>,
        _source: &'a [u8],
    ) -> std::iter::Empty<(QueryMatch<'a>, usize)> {
        std::iter::empty()
    }

    pub fn matches<'a>(
        &'a mut self,
        _query: &'a Query,
        _node: Node<'a>,
        _source: &'a [u8],
    ) -> std::iter::Empty<QueryMatch<'a>> {
        std::iter::empty()
    }
}

// Note: Additional Node methods are implemented above in the main impl block

impl TreeCursor {
    pub fn node(&self) -> Node<'_> {
        Node::new()
    }

    pub fn goto_first_child(&mut self) -> bool {
        false
    }

    pub fn goto_next_sibling(&mut self) -> bool {
        false
    }

    pub fn goto_parent(&mut self) -> bool {
        false
    }

    pub fn goto_previous_sibling(&mut self) -> bool {
        false
    }
}

// Note: Tree implementation is above

impl Parser {
    pub fn new() -> Result<Self, AstError> {
        Err(AstError::FeatureNotEnabled(
            "tree-sitter feature not enabled".to_string(),
        ))
    }

    pub fn set_language(&mut self, _language: &()) -> Result<(), AstError> {
        Err(AstError::FeatureNotEnabled(
            "tree-sitter feature not enabled".to_string(),
        ))
    }

    pub fn parse(&mut self, _input: &str, _old_tree: Option<&Tree>) -> Option<Tree> {
        None
    }
}

impl AstParser {
    /// Create a new AST parser stub (always returns error)
    pub fn new() -> Result<Self, AstError> {
        Err(AstError::FeatureNotEnabled(
            "Tree-sitter feature not enabled".to_string(),
        ))
    }

    /// Parse a file (stub - returns error)
    pub fn parse_file(&self, _file_path: &Path) -> Result<ParsedFile, AstError> {
        Err(AstError::FeatureNotEnabled(
            "Tree-sitter feature not enabled".to_string(),
        ))
    }

    /// Parse content directly (stub - returns minimal ParsedFile)
    pub fn parse_content(
        &self,
        content: &str,
        file_path: &Path,
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
        cache_path.push(format!(
            "{}.cache",
            file_path.file_name().unwrap_or_default().to_string_lossy()
        ));
        cache_path
    }
}

/// Create a test AST parser stub for testing
pub fn create_test_ast_parser() -> Result<AstParser, AstError> {
    AstParser::new()
}
