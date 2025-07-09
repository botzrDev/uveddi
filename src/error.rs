//! Error types and error handling for Uveddi
//!
//! This module defines the unified error type [`UveddiError`] used throughout the codebase.
//! All fallible operations should return `Result<T, UveddiError>`. Error variants cover
//! filesystem, database, AST parsing, AI provider, report generation, and configuration errors.
//!
//! [`UveddiError`]: enum.UveddiError.html

use crate::analysis::detectors::dependency::ExtractionError;
use thiserror::Error;

/// Unified error type for all Uveddi operations with comprehensive documentation
#[derive(Debug, Error)]
pub enum UveddiError {
    // === Path and File System Errors ===
    #[error("Path not found: {0}")]
    PathNotFound(String),
    #[error("Invalid input path: {path}")]
    InvalidInputPath { path: std::path::PathBuf },
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    // === Database Errors ===
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    // === AST Parsing Errors ===
    #[error("AST parsing error: {0}")]
    AstParsing(String),
    #[error("Tree-sitter query error: {0}")]
    QueryError(String),
    #[error("Language not supported: {0}")]
    UnsupportedLanguage(String),

    #[error("Analysis error: {0}")]
    Analysis(String),

    #[error("Anti-pattern detection error: {0}")]
    AntiPatternDetection(String),

    // === AI Provider Errors ===
    #[error("AI API error: {provider}: {message}")]
    AiApi { provider: String, message: String },
    #[error("AI context building error: {0}")]
    AiContext(String),
    #[error("AI response parsing error: {0}")]
    AiResponseParsing(String),
    #[error("No AI providers available")]
    NoAiProviders,

    // === Report Generation Errors ===
    #[error("Report generation error: {0}")]
    ReportGeneration(String),
    #[error("Unsupported output format: {0}")]
    UnsupportedOutputFormat(String),
    #[error("JSON serialization error: {0}")]
    JsonSerialization(#[from] serde_json::Error),

    // === Configuration Errors ===
    #[error("Configuration error: {0}")]
    Configuration(String),
    #[error("Missing required configuration: {0}")]
    MissingConfiguration(String),
    #[error("CLI argument error: {0}")]
    Clap(#[from] clap::Error),
    #[error("TOML parsing error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("Plugin error: {0}")]
    PluginError(String),
    // WASM error conversion temporarily disabled for debugging
    // #[error("WASM runtime error: {0}")]
    // WasmRuntimeError(#[from] wasmtime::Error),

    // === Cache Errors ===
    #[error("Cache error: {0}")]
    Cache(String),

    // === Network and External Service Errors ===
    #[error("Network error: {0}")]
    #[cfg(feature = "ai")]
    Network(#[from] reqwest::Error),

    // === Validation Errors ===
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Other error: {0}")]
    Other(String),
}

// Comprehensive From implementations for common error types
impl From<crate::ast::tree_sitter::AstError> for UveddiError {
    fn from(err: crate::ast::tree_sitter::AstError) -> Self {
        match err {
            crate::ast::tree_sitter::AstError::Io(io_err) => UveddiError::Io(io_err),
            crate::ast::tree_sitter::AstError::UnsupportedLanguage(lang) => {
                UveddiError::UnsupportedLanguage(lang)
            }
            _ => UveddiError::AstParsing(err.to_string()),
        }
    }
}

impl From<ExtractionError> for UveddiError {
    fn from(err: ExtractionError) -> Self {
        match err {
            ExtractionError::AstError(ast_err) => ast_err.into(),
            ExtractionError::IoError(_path, io_err) => UveddiError::Io(io_err),
            _ => UveddiError::Analysis(err.to_string()),
        }
    }
}

impl From<anyhow::Error> for UveddiError {
    fn from(err: anyhow::Error) -> Self {
        UveddiError::Other(err.to_string())
    }
}
