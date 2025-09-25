//! Abstract Syntax Tree (AST) parsing and manipulation module for Uveddi.
//!
//! This module provides the core infrastructure for working with abstract syntax trees (ASTs),
//! which are fundamental to all code analysis operations in Uveddi. It abstracts the
//! underlying parsing technology (Tree-sitter) to provide a consistent, language-agnostic
//! interface for traversing and analyzing source code.
//!
//! # Key Components
//!
//! - **`tree_sitter`**: A submodule that contains the low-level Tree-sitter parsing logic,
//!   including language-specific grammar handling and node traversal.
//! - **`CustomAst`**: A high-level struct representing a parsed source file. It encapsulates
//!   the Tree-sitter tree and provides convenient methods for accessing the root node and
//!   other metadata.
//!
//! # Usage
//!
//! The primary entry point for using this module is typically through the `AstParser`
//! in the `analysis` module, which handles file parsing and returns a `CustomAst` instance.
//!
//! ```no_run
//! use uveddi::ast::tree_sitter::AstParser;
//! use std::path::Path;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let parser = AstParser::new()?;
//! let ast = parser.parse_file(Path::new("src/main.rs"))?;
//! // Now you can analyze the `ast`
//! # Ok(())
//! # }
//! ```

pub mod compatibility_shim;
pub mod tree_sitter;
pub mod tree_sitter_impl;

pub use tree_sitter_impl::{
    AstError, AstParser, CacheStats, CustomAst, SourceLanguage,
};

// Re-export compatibility shim for migration
pub use compatibility_shim::{AstParserCompat, ParsedFileCompat};

// Type alias for backward compatibility with legacy detectors
pub type ParsedFile = ParsedFileCompat;

// Add missing error types for compatibility
#[derive(Debug, Clone)]
pub struct SyntaxError {
    pub start_byte: usize,
    pub end_byte: usize,
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
    pub message: String,
}

pub type ParseError = AstError;
