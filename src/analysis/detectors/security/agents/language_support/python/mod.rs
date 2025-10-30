//! Python-specific security analysis module
//!
//! This module provides comprehensive Python security analysis including
//! dangerous import detection, code execution patterns, obfuscation analysis,
//! and persistence mechanism detection.

pub mod analyzer;
pub mod patterns;

// Re-export main analyzer
pub use analyzer::PythonAgentAnalyzer;
