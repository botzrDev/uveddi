//! Caching functionality for analysis operations
//!
//! This module provides various caching mechanisms to improve analysis performance,
//! including AST caching and result caching.

pub mod ast;

pub use ast::AstCache;
