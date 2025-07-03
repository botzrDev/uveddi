//! AST parsing and manipulation module for Uveddi
//!
//! This module provides utilities for working with abstract syntax trees (ASTs),
//! including language-agnostic parsing via Tree-sitter.
//!
//! The main types and functions include:
//! - `CustomAst`: A structure representing the abstract syntax tree.
//! - Tree-sitter based parsing functions.

pub mod tree_sitter;
pub use tree_sitter::CustomAst;
