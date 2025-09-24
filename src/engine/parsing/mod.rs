//! # Parsing Module
//!
//! Pure parsing logic without business rules. Handles AST generation and
//! language detection for multiple programming languages.

pub mod ast_builder;
pub mod language_detection;
pub mod parsers;

// Re-export main types
pub use ast_builder::{AstBuilder, ParseResult};
pub use language_detection::detect_language;

// Language parser trait that all parsers must implement
use crate::ast::SourceLanguage;
#[cfg(feature = "tree-sitter")]
use tree_sitter::Tree;
#[cfg(not(feature = "tree-sitter"))]
use crate::ast::tree_sitter::Tree;

/// Common interface for all language-specific parsers
pub trait LanguageParser: Send + Sync {
    /// Returns the language this parser handles
    fn language(&self) -> SourceLanguage;

    /// Parse source code into a syntax tree
    fn parse(&self, source: &str) -> Result<Tree, ParseError>;

    /// Extract symbols from the parsed tree
    fn extract_symbols(&self, tree: &Tree, source: &str) -> Vec<Symbol>;

    /// Build relations between symbols
    fn build_relations(&self, tree: &Tree, source: &str) -> Vec<Relation>;
}

/// Parsing errors
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Language not supported: {0:?}")]
    UnsupportedLanguage(SourceLanguage),

    #[error("Parse failed: {0}")]
    ParseFailed(String),

    #[error("Invalid source code")]
    InvalidSource,
}

/// Symbol extracted from source code
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub line: usize,
    pub column: usize,
}

/// Symbol types
#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    Function,
    Class,
    Module,
    Variable,
    Constant,
    Type,
}

/// Relations between code elements
#[derive(Debug, Clone)]
pub struct Relation {
    pub from: String,
    pub to: String,
    pub kind: RelationKind,
}

/// Relation types
#[derive(Debug, Clone, PartialEq)]
pub enum RelationKind {
    Imports,
    Extends,
    Implements,
    Uses,
    Calls,
}