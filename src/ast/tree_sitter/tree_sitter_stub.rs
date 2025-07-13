// Stub implementation file for tree-sitter disabled builds
// UV-97: Tree-sitter feature gating implementation
// UV-178: Fix API compatibility issues

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

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
}

/// Stub for tree_sitter::TreeCursor
#[derive(Debug, Clone)]
pub struct TreeCursor;

/// Stub for tree_sitter::Tree
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceLanguage {
    Rust,
    Python,
    JavaScript,
}

impl SourceLanguage {
    pub fn from_path(_path: &Path) -> Option<Self> {
        None // Stub implementation
    }
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedFile {
    pub file_path: Arc<PathBuf>,
    pub language: SourceLanguage,
    pub source: Arc<String>,
    #[serde(skip)]
    pub tree: Option<Tree>,
    #[serde(skip)]
    pub custom_ast: Option<CustomAst>,
}

/// Custom AST representation stub
#[derive(Debug, Clone, Serialize, Deserialize)]
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
        parameters: Vec<String>, // Corrected from 'params' as per instructions
    },
    Variable {
        name: String,
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
    Io(#[from] std::io::Error),
    #[error("Tree-sitter language error: {0}")]
    TreeSitterLanguage(String),
    #[error("AST parsing failed")]
    ParseFailed,
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),
    #[error("Cache error: {0}")]
    CacheError(String),
    #[error("Anti-pattern detection error: {0}")]
    AntiPatternDetectionError(String),
    #[error("Other error: {0}")]
    Other(String),
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
    pub fn goto_previous_sibling(&mut self) -> bool {
        false
    }
    pub fn goto_parent(&mut self) -> bool {
        false
    }
}

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
    pub fn new() -> Result<Self, AstError> {
        Err(AstError::FeatureNotEnabled(
            "Tree-sitter feature not enabled".to_string(),
        ))
    }

    pub fn parse_file(&mut self, _path: &Path) -> Result<ParsedFile, AstError> {
        Err(AstError::FeatureNotEnabled(
            "Tree-sitter feature not enabled".to_string(),
        ))
    }

    pub fn parse_content(
        &mut self,
        _content: &str,
        _path: &Path,
        _language: SourceLanguage,
    ) -> Result<ParsedFile, AstError> {
        Err(AstError::FeatureNotEnabled(
            "Tree-sitter feature not enabled".to_string(),
        ))
    }

    pub fn clear_cache(&mut self) -> Result<(), AstError> {
        Ok(())
    }

    pub fn get_available_parsers(&self) -> Vec<SourceLanguage> {
        Vec::new()
    }
}

impl Default for AstParser {
    fn default() -> Self {
        // This will panic, which is acceptable for a stub that should not be called.
        Self::new().expect("Default AstParser stub should not be created")
    }
}

impl ParsedFile {
    pub fn cache_path(file_path: &Path) -> PathBuf {
        let mut cache_path = std::env::temp_dir();
        cache_path.push("uveddi_ast_cache");
        cache_path.push(format!(
            "{}.cache",
            file_path.file_name().unwrap_or_default().to_string_lossy()
        ));
        cache_path
    }

    /// Get the source code as a string slice
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Get the file path
    pub fn path(&self) -> &Path {
        &self.file_path
    }

    /// Extract a summary for debugging
    pub fn summary(&self) -> String {
        format!("Parsed {} (stub implementation)", self.file_path.display())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageThresholds {
    pub max_logical_loc: u32,
    pub max_methods: u32,
    pub max_fields: u32,
    pub max_cyclomatic_complexity: u32,
    pub max_cognitive_complexity: u32,
    pub max_lcom_score: f64,
    pub max_coupling: u32,
}

impl LanguageThresholds {
    pub fn rust() -> Self {
        Self {
            max_logical_loc: 100,
            max_methods: 10,
            max_fields: 10,
            max_cyclomatic_complexity: 15,
            max_cognitive_complexity: 20,
            max_lcom_score: 0.8,
            max_coupling: 5,
        }
    }
    pub fn python() -> Self {
        Self::rust()
    }
    pub fn javascript() -> Self {
        Self::rust()
    }
}

pub fn create_test_ast_parser() -> Result<AstParser, AstError> {
    AstParser::new()
}

pub mod language {
    pub fn rust() -> &'static () {
        &()
    }
    pub fn python() -> &'static () {
        &()
    }
    pub fn javascript() -> &'static () {
        &()
    }
}
