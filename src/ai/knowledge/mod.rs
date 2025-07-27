//! AI Knowledge Library Schema and Format
//! 
//! This module defines the foundational schema for Uveddi's AI Knowledge Library,
//! a compressed, hierarchical knowledge base containing anti-patterns, code quality
//! issues, and architectural problems across multiple programming languages.
//!
//! Key Features:
//! - Hierarchical schema optimized for AI consumption and zstd compression
//! - Perfect Hash Function (PHF) indexing for O(1) lookups
//! - Dictionary-trained compression achieving 70-75% size reduction
//! - Extensible design supporting language-specific and custom knowledge
//! - Build-time preprocessing for compression and index generation

pub mod schema;
pub mod compression;
pub mod indexing;
pub mod loader;

pub use schema::*;
pub use compression::*;
pub use indexing::*;
pub use loader::*;