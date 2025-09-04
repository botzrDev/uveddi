//! Tree-sitter AST parsing functionality for Uveddi
//!
//! This module provides tree-sitter-based parsing capabilities for various programming languages.
//! All functionality in this module is gated behind the "tree-sitter" feature flag.
//!
//! UV-97: Proper tree-sitter feature gating implementation using the canonical shim pattern

// Conditional module imports based on feature flag
#[cfg(feature = "tree-sitter")]
mod tree_sitter_impl;
#[cfg(not(feature = "tree-sitter"))]
mod tree_sitter_stub;

// Re-export the appropriate implementation
#[cfg(feature = "tree-sitter")]
pub use tree_sitter_impl::*;
#[cfg(not(feature = "tree-sitter"))]
pub use tree_sitter_stub::*;

// Re-export tree-sitter types for compatibility
#[cfg(feature = "tree-sitter")]
pub use tree_sitter::{Language, Node, Query, QueryCursor, QueryMatch, Tree, TreeCursor};
#[cfg(feature = "tree-sitter")]
pub use streaming_iterator::StreamingIterator;
#[cfg(not(feature = "tree-sitter"))]
pub use tree_sitter_stub::{Language, Node, Query, QueryCursor, QueryMatch, Tree, TreeCursor, StreamingIterator};

// Re-export tree-sitter language modules
#[cfg(not(feature = "tree-sitter"))]
pub use tree_sitter_stub::{
    tree_sitter_javascript, tree_sitter_python, tree_sitter_rust, tree_sitter_typescript,
};
#[cfg(feature = "tree-sitter")]
pub use {tree_sitter_javascript, tree_sitter_python, tree_sitter_rust, tree_sitter_typescript};

pub mod queries;
