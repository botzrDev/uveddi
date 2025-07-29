use crate::analysis::component_extractor::ComponentExtractionError;
use crate::analysis::detectors::dependency::ExtractionError;
use crate::analysis::mermaid_generator::MermaidGenerationError;
use crate::ast::tree_sitter_impl::AstError;
use crate::plugins::errors::PluginError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AnalysisError {
    #[error("AST error: {0}")]
    AstError(#[from] AstError),
    #[error("Dependency extraction error: {0}")]
    DependencyExtractionError(#[from] ExtractionError),
    #[error("Metric calculation error: {0}")]
    MetricCalculationError(String),
    #[error("Anti-pattern detection error: {0}")]
    AntiPatternDetectionError(String),
    #[error("Detection error: {0}")]
    DetectionError(String),
    #[error("Component extraction error: {0}")]
    ComponentExtractionError(#[from] ComponentExtractionError),
    #[error("Mermaid generation error: {0}")]
    MermaidGenerationError(#[from] MermaidGenerationError),
    #[error("Symbol resolution error: {0}")]
    SymbolResolutionError(String),
    #[error("Graph analysis error: {0}")]
    GraphAnalysisError(String),
    #[error("Tree-sitter query error: {0}")]
    QueryError(String),
    #[error("Plugin error: {0}")]
    PluginError(#[from] PluginError),
    #[error("Other analysis error: {0}")]
    Other(String),
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),

    // New error types for unwrap replacements (UV-276)
    #[error("Parse operation failed: {message}")]
    ParseError { message: String },
    #[error("Collection access failed: {message}")]
    CollectionAccessError { message: String },
    #[error("Type conversion failed: {message}")]
    ConversionError { message: String },
    #[error("Expected data not found: {message}")]
    DataNotFoundError { message: String },
    #[error("Line number overflow: value {value} exceeds maximum")]
    LineNumberOverflow { value: u64 },
    #[error("Mutex lock failed: {message}")]
    LockError { message: String },
}

impl AnalysisError {
    /// Create parse error with context
    pub fn parse_error(message: impl Into<String>) -> Self {
        Self::ParseError {
            message: message.into(),
        }
    }

    /// Create query error with context
    pub fn query_error(message: impl Into<String>) -> Self {
        Self::QueryError(message.into())
    }

    /// Create collection access error
    pub fn collection_access_error(message: impl Into<String>) -> Self {
        Self::CollectionAccessError {
            message: message.into(),
        }
    }

    /// Create conversion error
    pub fn conversion_error(message: impl Into<String>) -> Self {
        Self::ConversionError {
            message: message.into(),
        }
    }

    /// Create data not found error
    pub fn data_not_found_error(message: impl Into<String>) -> Self {
        Self::DataNotFoundError {
            message: message.into(),
        }
    }

    /// Create lock error
    pub fn lock_error(message: impl Into<String>) -> Self {
        Self::LockError {
            message: message.into(),
        }
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for AnalysisError {
    fn from(e: Box<dyn std::error::Error + Send + Sync>) -> Self {
        AnalysisError::Other(e.to_string())
    }
}
