use crate::analysis::component_extractor::ComponentExtractionError;
use crate::analysis::detectors::dependency::ExtractionError;
use crate::analysis::mermaid_generator::MermaidGenerationError;
use crate::ast::tree_sitter_impl::AstError;
use crate::error::UveddiError;
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

    // File system and I/O errors
    #[error("File system error at '{path}': {source}")]
    FileSystemError {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("File encoding error for '{file}': {message}")]
    FileEncodingError { file: String, message: String },

    // Memory and resource errors
    #[error("Memory limit exceeded: used {used_mb}MB, limit {limit_mb}MB")]
    MemoryLimitExceeded { used_mb: usize, limit_mb: usize },

    #[error("Resource allocation failed: {resource_type}")]
    ResourceAllocationError { resource_type: String },

    // Tree-sitter and parsing specific errors
    #[error(
        "Tree-sitter parsing failed for {language} in '{file}' at line {line}: {syntax_error}"
    )]
    TreeSitterParseError {
        file: String,
        language: String,
        line: usize,
        syntax_error: String,
    },

    #[error("Tree-sitter query compilation failed for {language}: {query_error}")]
    TreeSitterQueryError {
        language: String,
        query_error: String,
    },

    // Multi-crate and workspace errors
    #[error("Workspace discovery failed at '{workspace_path}': {reason}")]
    WorkspaceDiscoveryError {
        workspace_path: String,
        reason: String,
    },

    #[error("Crate analysis failed for '{crate_name}' at '{crate_path}': {reason}")]
    CrateAnalysisError {
        crate_name: String,
        crate_path: String,
        reason: String,
    },

    #[error("Manifest parsing failed for '{manifest_path}': {error}")]
    ManifestParseError {
        manifest_path: String,
        error: String,
    },

    // Analysis pipeline errors
    #[error("Analysis pipeline failed at stage '{stage}': {reason}")]
    PipelineError { stage: String, reason: String },

    #[error("Detector '{detector_name}' failed on '{file}': {error}")]
    DetectorError {
        detector_name: String,
        file: String,
        error: String,
    },

    #[error("Parallel analysis failed: {worker_count} workers, {failed_count} failures")]
    ParallelAnalysisError {
        worker_count: usize,
        failed_count: usize,
    },

    // Engine and service errors
    #[error("Engine error: {0}")]
    Engine(String),

    // Configuration and setup errors
    #[error("Configuration error: {field} = {value}, {reason}")]
    ConfigurationError {
        field: String,
        value: String,
        reason: String,
    },

    #[error("Feature not available: {feature}. {suggestion}")]
    FeatureUnavailableError { feature: String, suggestion: String },

    // Existing error types for unwrap replacements (UV-276)
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

    #[error("Regex compilation failed: {error}")]
    RegexError { error: String },
}

impl AnalysisError {
    /// Create file system error with context
    pub fn file_system_error(path: impl Into<String>, source: std::io::Error) -> Self {
        Self::FileSystemError {
            path: path.into(),
            source,
        }
    }

    /// Create file encoding error
    pub fn file_encoding_error(file: impl Into<String>, message: impl Into<String>) -> Self {
        Self::FileEncodingError {
            file: file.into(),
            message: message.into(),
        }
    }

    /// Create memory limit exceeded error
    pub fn memory_limit_exceeded(used_mb: usize, limit_mb: usize) -> Self {
        Self::MemoryLimitExceeded { used_mb, limit_mb }
    }

    /// Create tree-sitter parse error
    pub fn tree_sitter_parse_error(
        file: impl Into<String>,
        language: impl Into<String>,
        line: usize,
        syntax_error: impl Into<String>,
    ) -> Self {
        Self::TreeSitterParseError {
            file: file.into(),
            language: language.into(),
            line,
            syntax_error: syntax_error.into(),
        }
    }

    /// Create tree-sitter query error
    pub fn tree_sitter_query_error(
        language: impl Into<String>,
        query_error: impl Into<String>,
    ) -> Self {
        Self::TreeSitterQueryError {
            language: language.into(),
            query_error: query_error.into(),
        }
    }

    /// Create workspace discovery error
    pub fn workspace_discovery_error(
        workspace_path: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self::WorkspaceDiscoveryError {
            workspace_path: workspace_path.into(),
            reason: reason.into(),
        }
    }

    /// Create crate analysis error
    pub fn crate_analysis_error(
        crate_name: impl Into<String>,
        crate_path: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self::CrateAnalysisError {
            crate_name: crate_name.into(),
            crate_path: crate_path.into(),
            reason: reason.into(),
        }
    }

    /// Create pipeline error
    pub fn pipeline_error(stage: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::PipelineError {
            stage: stage.into(),
            reason: reason.into(),
        }
    }

    /// Create detector error
    pub fn detector_error(
        detector_name: impl Into<String>,
        file: impl Into<String>,
        error: impl Into<String>,
    ) -> Self {
        Self::DetectorError {
            detector_name: detector_name.into(),
            file: file.into(),
            error: error.into(),
        }
    }

    /// Create configuration error
    pub fn configuration_error(
        field: impl Into<String>,
        value: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self::ConfigurationError {
            field: field.into(),
            value: value.into(),
            reason: reason.into(),
        }
    }

    /// Create feature unavailable error
    pub fn feature_unavailable_error(
        feature: impl Into<String>,
        suggestion: impl Into<String>,
    ) -> Self {
        Self::FeatureUnavailableError {
            feature: feature.into(),
            suggestion: suggestion.into(),
        }
    }

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

    /// Check if this error indicates a recoverable condition
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::FileEncodingError { .. }
                | Self::TreeSitterParseError { .. }
                | Self::DetectorError { .. }
                | Self::DataNotFoundError { .. }
        )
    }

    /// Get error category for logging and metrics
    pub fn category(&self) -> &'static str {
        match self {
            Self::FileSystemError { .. } | Self::FileEncodingError { .. } => "filesystem",
            Self::MemoryLimitExceeded { .. } | Self::ResourceAllocationError { .. } => "memory",
            Self::TreeSitterParseError { .. } | Self::TreeSitterQueryError { .. } => "parsing",
            Self::WorkspaceDiscoveryError { .. }
            | Self::CrateAnalysisError { .. }
            | Self::ManifestParseError { .. } => "workspace",
            Self::PipelineError { .. } | Self::ParallelAnalysisError { .. } => "pipeline",
            Self::DetectorError { .. } => "detector",
            Self::ConfigurationError { .. } | Self::FeatureUnavailableError { .. } => {
                "configuration"
            }
            _ => "general",
        }
    }
}

impl From<regex::Error> for AnalysisError {
    fn from(err: regex::Error) -> Self {
        AnalysisError::RegexError {
            error: err.to_string(),
        }
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for AnalysisError {
    fn from(e: Box<dyn std::error::Error + Send + Sync>) -> Self {
        AnalysisError::Other(e.to_string())
    }
}

impl From<UveddiError> for AnalysisError {
    fn from(e: UveddiError) -> Self {
        AnalysisError::Other(e.to_string())
    }
}
