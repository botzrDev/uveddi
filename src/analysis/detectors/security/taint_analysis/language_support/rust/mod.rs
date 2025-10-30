//! Rust-specific taint analysis support

pub mod analyzer;
pub mod patterns;

pub use analyzer::RustTaintAnalyzer;
