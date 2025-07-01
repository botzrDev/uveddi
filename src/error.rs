use thiserror::Error;
use anyhow::Context;

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
    #[error("Language not supported: {0}")]
    UnsupportedLanguage(String),
    
    // === Analysis Errors ===
    #[error("Analysis error: {0}")]
    Analysis(String),
    
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
    
    // === Plugin Errors ===
    #[error("Plugin error: {0}")]
    Plugin(String),
    #[error("WASM runtime error: {0}")]
    WasmRuntimeError(#[from] wasmtime::Error),
    
    // === Cache Errors ===
    #[error("Cache error: {0}")]
    Cache(String),
    
    // === Network and External Service Errors ===
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    
    // === Validation Errors ===
    #[error("Validation error: {0}")]
    Validation(String),
}

// Comprehensive From implementations for common error types
impl From<crate::ast::tree_sitter::AstError> for UveddiError {
    fn from(err: crate::ast::tree_sitter::AstError) -> Self {
        match err {
            crate::ast::tree_sitter::AstError::Io(io_err) => UveddiError::Io(io_err),
            crate::ast::tree_sitter::AstError::UnsupportedLanguage(lang) => 
                UveddiError::UnsupportedLanguage(lang),
            _ => UveddiError::AstParsing(err.to_string()),
        }
    }
}

impl From<crate::analysis::dependency_extractor::ExtractionError> for UveddiError {
    fn from(err: crate::analysis::dependency_extractor::ExtractionError) -> Self {
        match err {
            crate::analysis::dependency_extractor::ExtractionError::AstError(ast_err) => ast_err.into(),
            crate::analysis::dependency_extractor::ExtractionError::IoError(_path, io_err) => 
                UveddiError::Io(io_err),
            _ => UveddiError::Analysis(err.to_string()),
        }
    }
}

// Context support for adding contextual information to errors
pub trait ErrContext<T> {
    fn err_context(self, context: &str) -> Result<T, UveddiError>;
    fn err_context_with(self, context: impl FnOnce() -> String) -> Result<T, UveddiError>;
}

impl<T, E> ErrContext<T> for Result<T, E>
where
    E: Into<UveddiError>,
{
    fn err_context(self, context: &str) -> Result<T, UveddiError> {
        self.map_err(|e| {
            let mut err: UveddiError = e.into();
            match &mut err {
                UveddiError::Io(e) => *e = std::io::Error::new(e.kind(), format!("{}: {}", context, e)),
                UveddiError::Analysis(s) => *s = format!("{}: {}", context, s),
                UveddiError::Plugin(s) => *s = format!("{}: {}", context, s),
                UveddiError::Configuration(s) => *s = format!("{}: {}", context, s),
                UveddiError::PathNotFound(s) => *s = format!("{}: {}", context, s),
                UveddiError::UnsupportedOutputFormat(s) => *s = format!("{}: {}", context, s),
                // Add more variants as needed
                _ => {} // For other variants, we don't modify the error message
            }
            err
        })
    }

    fn err_context_with(self, context: impl FnOnce() -> String) -> Result<T, UveddiError> {
        self.map_err(|e| {
            let context_str = context();
            let mut err: UveddiError = e.into();
            match &mut err {
                UveddiError::Io(e) => *e = std::io::Error::new(e.kind(), format!("{}: {}", context_str, e)),
                UveddiError::Analysis(s) => *s = format!("{}: {}", context_str, s),
                UveddiError::Plugin(s) => *s = format!("{}: {}", context_str, s),
                UveddiError::Configuration(s) => *s = format!("{}: {}", context_str, s),
                UveddiError::PathNotFound(s) => *s = format!("{}: {}", context_str, s),
                UveddiError::UnsupportedOutputFormat(s) => *s = format!("{}: {}", context_str, s),
                // Add more variants as needed
                _ => {} // For other variants, we don't modify the error message
            }
            err
        })
    }
}
