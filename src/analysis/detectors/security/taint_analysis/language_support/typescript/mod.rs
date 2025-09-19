//! TypeScript/JavaScript-specific taint analysis support

pub mod patterns;
pub mod analyzer;

pub use analyzer::TypeScriptTaintAnalyzer;