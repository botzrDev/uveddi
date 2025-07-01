use thiserror::Error;

/// Unified error type for all Uveddi operations
#[derive(Debug, Error)]
pub enum UveddiError {
    // Path and file system errors
    #[error("Path not found: {0}")]
    PathNotFound(String),
    #[error("Invalid input path: {path}")]
    InvalidInputPath { path: std::path::PathBuf },
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    // Database errors
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    // AST parsing errors
    #[error("AST parsing error: {0}")]
    AstParsing(String),
    #[error("Language not supported: {0}")]
    UnsupportedLanguage(String),

    // Analysis errors
    #[error("Analysis error: {0}")]
    Analysis(String),
    #[error("Dependency extraction error: {0}")]
    DependencyExtraction(String),
    #[error("Anti-pattern detection error: {0}")]
    AntiPatternDetection(String),

    // AI provider errors
    #[error("AI API error: {provider}: {message}")]
    AiApi { provider: String, message: String },
    #[error("AI context building error: {0}")]
    AiContext(String),
    #[error("AI response parsing error: {0}")]
    AiResponseParsing(String),
    #[error("No AI providers available")]
    NoAiProviders,

    // Report generation errors
    #[error("Report generation error: {0}")]
    ReportGeneration(String),
    #[error("Unsupported output format: {0}")]
    UnsupportedOutputFormat(String),
    #[error("JSON serialization error: {0}")]
    JsonSerialization(#[from] serde_json::Error),

    // Configuration errors
    #[error("Configuration error: {0}")]
    Configuration(String),
    #[error("Missing required configuration: {0}")]
    MissingConfiguration(String),

    // Plugin errors
    #[error("Plugin error: {0}")]
    Plugin(String),
    #[error("Plugin discovery error: {0}")]
    PluginDiscovery(String),
    #[error("Plugin error: {0}")]
    PluginError(String),
    #[error("WASM runtime error: {0}")]
    WasmRuntimeError(#[from] wasmtime::Error),
    #[error("Plugin verification failed: {0}")]
    PluginVerificationError(String),
    #[error("Resource limit exceeded: {0}")]
    ResourceLimitError(String),

    // Cache errors
    #[error("Cache error: {0}")]
    Cache(String),
    #[error("Cache corruption detected: {0}")]
    CacheCorruption(String),

    // Network and external service errors
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("External service error: {service}: {message}")]
    ExternalService { service: String, message: String },

    // Validation errors
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Invalid format: expected {expected}, got {actual}")]
    InvalidFormat { expected: String, actual: String },
}

// From implementations for backward compatibility during migration
impl From<crate::ast::tree_sitter::AstError> for UveddiError {
    fn from(err: crate::ast::tree_sitter::AstError) -> Self {
        match err {
            crate::ast::tree_sitter::AstError::Io(io_err) => UveddiError::Io(io_err),
            crate::ast::tree_sitter::AstError::UnsupportedLanguage(lang) => UveddiError::UnsupportedLanguage(lang),
            _ => UveddiError::AstParsing(err.to_string()),
        }
    }
}

impl From<crate::analysis::dependency_extractor::ExtractionError> for UveddiError {
    fn from(err: crate::analysis::dependency_extractor::ExtractionError) -> Self {
        match err {
            crate::analysis::dependency_extractor::ExtractionError::AstError(ast_err) => ast_err.into(),
            crate::analysis::dependency_extractor::ExtractionError::IoError(_path, io_err) => {
                UveddiError::Io(io_err)
            },
            _ => UveddiError::DependencyExtraction(err.to_string()),
        }
    }
}
