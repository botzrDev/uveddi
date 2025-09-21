//! Code Duplication anti-pattern detector
//!
//! This module provides a comprehensive code duplication detection system that combines
//! multiple algorithms for identifying different types of code clones:
//!
//! - **Type-1 (Exact)**: Identical code except whitespace/comments
//! - **Type-2 (Renamed)**: Identical structure, different identifiers
//! - **Type-3 (Near-miss)**: Similar structure with minor modifications
//! - **Type-4 (Semantic)**: Functionally equivalent but different syntax
//!
//! ## Architecture
//!
//! The detector is organized into several specialized modules:
//! - `algorithms`: Different detection algorithms (token-based, AST-based, semantic)
//! - `language_support`: Language-specific parsing and analysis
//! - `metrics`: Similarity calculation and threshold management
//! - `detector`: Main detector implementation
//! - `config`: Configuration management

pub mod algorithms;
pub mod config;
pub mod detector;
pub mod helpers;
pub mod language_support;
pub mod metrics;
pub mod types;

// Re-export the main types and detector
pub use config::DuplicationConfig;
pub use detector::CodeDuplicationDetector;
pub use types::{ClonePair, CloneType, CodeBlock};
