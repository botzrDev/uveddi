use crate::error::rendering::RenderingServiceError;
use crate::{
    analysis::detectors::dependency::ExtractionError as DependencyExtractionError,
    analysis::errors::AnalysisError, ast::tree_sitter_impl::AstError, plugins::errors::PluginError,
    report::errors::ReportGenerationError, security::SecurityError,
};
use clap::error::Error as ClapError;
use reqwest::Error as ReqwestError;
use rusqlite::Error as RusqliteError;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ExtractionError {
    #[error("AST extraction error: {0}")]
    AstError(AstError),
    #[error("File read error: {0}")]
    FileReadError(#[from] std::io::Error),
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),
}

/// JSON deserialization errors with security context
#[derive(Debug, Error)]
pub enum DeserializationError {
    #[error("Invalid JSON format: {0}")]
    InvalidFormat(String),

    #[error("JSON payload too large: {size} bytes (max: {max_size})")]
    PayloadTooLarge { size: usize, max_size: usize },

    #[error("Invalid data structure: {0}")]
    InvalidStructure(String),

    #[error("Security validation failed: {0}")]
    SecurityValidation(String),
}

#[derive(Error, Debug)]
pub enum UveddiError {
    // === Core Analysis Errors ===
    #[error("Analysis error in {file}:{line}: {message}\n  → Context: {context}\n  → Suggestion: {suggestion}")]
    AnalysisError {
        file: String,
        line: u32,
        message: String,
        context: String,
        suggestion: String,
        #[source]
        source: Option<AnalysisError>,
    },

    #[error("File extraction failed for '{path}': {message}\n  → Cause: {cause}\n  → Recovery: {recovery_hint}")]
    ExtractionError {
        path: String,
        message: String,
        cause: String,
        recovery_hint: String,
        #[source]
        source: Option<ExtractionError>,
    },

    #[error("AST parsing failed for '{file}': {message}\n  → Language: {language}\n  → Suggestion: {suggestion}")]
    AstError {
        file: String,
        language: String,
        message: String,
        suggestion: String,
        #[source]
        source: Option<AstError>,
    },

    #[error("Dependency extraction failed: {message}\n  → Module: {module}\n  → Suggestion: {suggestion}")]
    DependencyExtractionError {
        module: String,
        message: String,
        suggestion: String,
        #[source]
        source: Option<DependencyExtractionError>,
    },

    // === Service & Infrastructure Errors ===
    #[error("Rendering service unavailable: {message}\n  → Service: {service_url}\n  → Suggestion: {suggestion}")]
    RenderingServiceError {
        service_url: String,
        message: String,
        suggestion: String,
        #[source]
        source: Option<RenderingServiceError>,
    },

    #[error("Database operation failed: {operation} on '{database}'\n  → Error: {message}\n  → Recovery: {recovery_hint}")]
    DatabaseError {
        operation: String,
        database: String,
        message: String,
        recovery_hint: String,
        #[source]
        source: Option<RusqliteError>,
    },

    #[error("Plugin system error: {plugin} - {message}\n  → Type: {plugin_type}\n  → Suggestion: {suggestion}")]
    PluginError {
        plugin: String,
        plugin_type: String,
        message: String,
        suggestion: String,
        #[source]
        source: Option<PluginError>,
    },

    #[error("Security violation: {message}\n  → Context: {context}\n  → Action Required: {action_required}")]
    SecurityError {
        message: String,
        context: String,
        action_required: String,
        #[source]
        source: Option<SecurityError>,
    },

    // === External System Errors ===
    #[error("Network request failed: {operation} to '{url}'\n  → Status: {status}\n  → Suggestion: {suggestion}")]
    NetworkError {
        operation: String,
        url: String,
        status: String,
        suggestion: String,
        #[source]
        source: Option<ReqwestError>,
    },

    #[error("Report generation failed: {report_type} for '{target}'\n  → Stage: {stage}\n  → Suggestion: {suggestion}")]
    ReportError {
        report_type: String,
        target: String,
        stage: String,
        suggestion: String,
        #[source]
        source: Option<ReportGenerationError>,
    },

    // === System Errors ===
    #[error(
        "Configuration error: {message}\n  → Location: {location}\n  → Suggestion: {suggestion}"
    )]
    ConfigError {
        message: String,
        location: String,
        suggestion: String,
    },

    #[error("Command line error: {message}\n  → Command: {command}\n  → Suggestion: {suggestion}")]
    CliError {
        command: String,
        message: String,
        suggestion: String,
        #[source]
        source: Option<ClapError>,
    },

    #[error("File system error: {operation} failed for '{path}'\n  → Error: {message}\n  → Suggestion: {suggestion}")]
    IoError {
        operation: String,
        path: String,
        message: String,
        suggestion: String,
        #[source]
        source: Option<std::io::Error>,
    },

    #[error("Serialization error: {operation} failed for '{data_type}'\n  → Error: {message}\n  → Suggestion: {suggestion}")]
    SerializationError {
        operation: String,
        data_type: String,
        message: String,
        suggestion: String,
        #[source]
        source: Option<serde_json::Error>,
    },

    #[error("Deserialization security error: {message}\n  → Context: {context}\n  → Suggestion: {suggestion}")]
    Deserialization {
        message: String,
        context: String,
        suggestion: String,
        #[source]
        source: Option<DeserializationError>,
    },

    #[error("Invalid path: '{path}'\n  → Reason: {reason}\n  → Suggestion: {suggestion}")]
    PathError {
        path: String,
        reason: String,
        suggestion: String,
    },

    #[error("Unexpected error: {message}\n  → Context: {context}\n  → Suggestion: {suggestion}")]
    GenericError {
        message: String,
        context: String,
        suggestion: String,
        #[source]
        source: Option<anyhow::Error>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCategory {
    Analysis,
    Extraction,
    Rendering,
    Database,
    Plugin,
    Network,
    Reporting,
    Configuration,
    Cli,
    Io,
    Serialization,
    Path,
    ServiceCommunication,
    ResourceExhaustion,
    ServiceSpecific,
    Generic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl UveddiError {
    /// Creates a helpful configuration error with suggestions
    pub fn config_error(message: &str, location: &str) -> Self {
        let suggestion = Self::generate_config_suggestion(message);
        Self::ConfigError {
            message: message.to_string(),
            location: location.to_string(),
            suggestion,
        }
    }

    /// Creates a helpful IO error with context and suggestions
    pub fn io_error(operation: &str, path: &str, source: std::io::Error) -> Self {
        let suggestion = Self::generate_io_suggestion(&source, operation);
        Self::IoError {
            operation: operation.to_string(),
            path: path.to_string(),
            message: source.to_string(),
            suggestion,
            source: Some(source),
        }
    }

    /// Creates a helpful network error with actionable suggestions
    pub fn network_error(operation: &str, url: &str, source: ReqwestError) -> Self {
        let (status, suggestion) = Self::generate_network_suggestion(&source, url);
        Self::NetworkError {
            operation: operation.to_string(),
            url: url.to_string(),
            status,
            suggestion,
            source: Some(source),
        }
    }

    /// Creates a helpful database error with recovery hints
    pub fn database_error(operation: &str, database: &str, source: RusqliteError) -> Self {
        let recovery_hint = Self::generate_database_recovery_hint(&source, operation);
        Self::DatabaseError {
            operation: operation.to_string(),
            database: database.to_string(),
            message: source.to_string(),
            recovery_hint,
            source: Some(source),
        }
    }

    /// Creates a database error from just a message
    pub fn database_error_msg(message: &str) -> Self {
        Self::DatabaseError {
            operation: "database operation".to_string(),
            database: "sqlite".to_string(),
            message: message.to_string(),
            recovery_hint: "Check database integrity and schema compatibility".to_string(),
            source: None,
        }
    }

    /// Creates a helpful analysis error with context
    pub fn analysis_error(file: &str, line: u32, message: &str, context: &str) -> Self {
        let suggestion = Self::generate_analysis_suggestion(message, context);
        Self::AnalysisError {
            file: file.to_string(),
            line,
            message: message.to_string(),
            context: context.to_string(),
            suggestion,
            source: None,
        }
    }
    
    /// Alias for config_error to maintain backwards compatibility
    pub fn configuration_error(message: &str) -> Self {
        Self::config_error(message, "configuration")
    }
    
    /// Creates a validation error
    pub fn validation_error(message: &str) -> Self {
        Self::ConfigError {
            message: message.to_string(),
            location: "validation".to_string(),
            suggestion: "Check input validation rules and format requirements".to_string(),
        }
    }
    
    /// Creates an initialization error
    pub fn initialization_error(message: &str) -> Self {
        Self::ConfigError {
            message: message.to_string(),
            location: "initialization".to_string(),
            suggestion: "Ensure all required components are properly initialized".to_string(),
        }
    }

    /// Generates helpful suggestions for configuration errors
    fn generate_config_suggestion(message: &str) -> String {
        match message.to_lowercase() {
            msg if msg.contains("ollama") => {
                "Ensure Ollama is running on localhost:11434 or update the endpoint in config"
            }
            msg if msg.contains("feature") => {
                "Check that required features are enabled in Cargo.toml"
            }
            msg if msg.contains("path") || msg.contains("file") => {
                "Verify the file path exists and is readable"
            }
            msg if msg.contains("permission") => {
                "Check file permissions and run with appropriate privileges"
            }
            msg if msg.contains("format") => "Validate configuration file format (TOML syntax)",
            _ => "Check the configuration documentation for valid options",
        }
        .to_string()
    }

    /// Generates helpful suggestions for IO errors
    fn generate_io_suggestion(error: &std::io::Error, operation: &str) -> String {
        use std::io::ErrorKind;
        match error.kind() {
            ErrorKind::NotFound => format!(
                "Create the required file or directory for {} operation",
                operation
            ),
            ErrorKind::PermissionDenied => {
                "Check file permissions or run with elevated privileges".to_string()
            }
            ErrorKind::AlreadyExists => {
                "Choose a different name or remove the existing file".to_string()
            }
            ErrorKind::InvalidInput => "Verify the path format and special characters".to_string(),
            ErrorKind::TimedOut => "Check disk space and system load, then retry".to_string(),
            _ => format!(
                "Retry the {} operation or check system resources",
                operation
            ),
        }
    }

    /// Generates helpful suggestions for network errors
    fn generate_network_suggestion(error: &ReqwestError, url: &str) -> (String, String) {
        let status = if let Some(status) = error.status() {
            format!("HTTP {}", status.as_u16())
        } else if error.is_timeout() {
            "Timeout".to_string()
        } else if error.is_connect() {
            "Connection Failed".to_string()
        } else {
            "Network Error".to_string()
        };

        let suggestion = if error.is_timeout() {
            "Increase timeout settings or check network speed"
        } else if error.is_connect() {
            if url.contains("localhost") || url.contains("127.0.0.1") {
                "Start the local service or check the port number"
            } else {
                "Check internet connection and DNS settings"
            }
        } else if let Some(status) = error.status() {
            match status.as_u16() {
                401 => "Check authentication credentials",
                403 => "Verify API permissions and access rights",
                404 => "Confirm the endpoint URL and API version",
                429 => "Reduce request rate or implement backoff",
                500..=599 => "Service temporarily unavailable, retry later",
                _ => "Check API documentation for error details",
            }
        } else {
            "Check network connectivity and firewall settings"
        };

        (status, suggestion.to_string())
    }

    /// Generates helpful recovery hints for database errors
    fn generate_database_recovery_hint(error: &RusqliteError, operation: &str) -> String {
        match error {
            RusqliteError::SqliteFailure(err, msg) => match err.code {
                rusqlite::ErrorCode::DatabaseLocked => {
                    "Close other database connections or wait for lock release".to_string()
                }
                rusqlite::ErrorCode::DatabaseCorrupt => {
                    "Restore from backup or recreate the database".to_string()
                }
                rusqlite::ErrorCode::CannotOpen => {
                    "Check database file permissions and path".to_string()
                }
                rusqlite::ErrorCode::ReadOnly => {
                    "Use a writable database or change to read-only operations".to_string()
                }
                _ => {
                    if let Some(msg) = msg {
                        if msg.contains("syntax") {
                            "Check SQL query syntax and table schema".to_string()
                        } else if msg.contains("constraint") {
                            "Verify data satisfies table constraints".to_string()
                        } else {
                            format!("Review {} operation and database schema", operation)
                        }
                    } else {
                        format!("Check database state and retry {} operation", operation)
                    }
                }
            },
            _ => format!(
                "Verify database connection and retry {} operation",
                operation
            ),
        }
    }

    /// Generates helpful suggestions for analysis errors
    fn generate_analysis_suggestion(message: &str, context: &str) -> String {
        match message.to_lowercase() {
            msg if msg.contains("parse") || msg.contains("syntax") => {
                "Check file syntax and language detection"
            }
            msg if msg.contains("memory") => "Reduce analysis scope or increase memory limits",
            msg if msg.contains("timeout") => "Increase analysis timeout or reduce file complexity",
            msg if msg.contains("unsupported") => {
                "Enable required features or use supported file types"
            }
            _ => {
                if context.contains("large") {
                    "Consider excluding large files or increasing limits"
                } else if context.contains("complex") {
                    "Simplify code structure or adjust detector thresholds"
                } else {
                    "Check file encoding and language support"
                }
            }
        }
        .to_string()
    }

    pub fn severity(&self) -> ErrorSeverity {
        match self {
            UveddiError::AnalysisError { .. }
            | UveddiError::ExtractionError { .. }
            | UveddiError::AstError { .. }
            | UveddiError::DependencyExtractionError { .. }
            | UveddiError::DatabaseError { .. }
            | UveddiError::PluginError { .. }
            | UveddiError::SecurityError { .. }
            | UveddiError::Deserialization { .. } => ErrorSeverity::High,
            UveddiError::RenderingServiceError { .. }
            | UveddiError::ReportError { .. }
            | UveddiError::NetworkError { .. } => ErrorSeverity::Medium,
            _ => ErrorSeverity::Low,
        }
    }
    pub fn category(&self) -> ErrorCategory {
        match self {
            UveddiError::AnalysisError { .. } => ErrorCategory::Analysis,
            UveddiError::ExtractionError { .. } => ErrorCategory::Extraction,
            UveddiError::AstError { .. } => ErrorCategory::Analysis,
            UveddiError::DependencyExtractionError { .. } => ErrorCategory::Extraction,
            UveddiError::DatabaseError { .. } => ErrorCategory::Database,
            UveddiError::ConfigError { .. } => ErrorCategory::Configuration,
            UveddiError::ReportError { .. } => ErrorCategory::Reporting,
            UveddiError::PluginError { .. } => ErrorCategory::Plugin,
            UveddiError::SecurityError { .. } => ErrorCategory::ServiceSpecific,
            UveddiError::NetworkError { .. } => ErrorCategory::Network,
            UveddiError::CliError { .. } => ErrorCategory::Cli,
            UveddiError::IoError { .. } => ErrorCategory::Io,
            UveddiError::SerializationError { .. } => ErrorCategory::Serialization,
            UveddiError::Deserialization { .. } => ErrorCategory::Serialization,
            UveddiError::PathError { .. } => ErrorCategory::Path,
            UveddiError::RenderingServiceError { .. } => ErrorCategory::Rendering,
            UveddiError::GenericError { .. } => ErrorCategory::Generic,
        }
    }
}

/// Implementations to convert from standard error types to UveddiError
impl From<std::io::Error> for UveddiError {
    fn from(error: std::io::Error) -> Self {
        Self::io_error("file operation", "unknown path", error)
    }
}

impl From<RusqliteError> for UveddiError {
    fn from(error: RusqliteError) -> Self {
        Self::database_error("database operation", "unknown database", error)
    }
}

impl From<ReqwestError> for UveddiError {
    fn from(error: ReqwestError) -> Self {
        Self::network_error("network request", "unknown URL", error)
    }
}

impl From<serde_json::Error> for UveddiError {
    fn from(error: serde_json::Error) -> Self {
        Self::SerializationError {
            operation: "JSON operation".to_string(),
            data_type: "unknown".to_string(),
            message: error.to_string(),
            suggestion: "Check JSON syntax and data types".to_string(),
            source: Some(error),
        }
    }
}

impl From<AstError> for UveddiError {
    fn from(error: AstError) -> Self {
        Self::AstError {
            file: "unknown file".to_string(),
            language: "unknown".to_string(),
            message: error.to_string(),
            suggestion: "Check file syntax and language support".to_string(),
            source: Some(error),
        }
    }
}

impl From<AnalysisError> for UveddiError {
    fn from(error: AnalysisError) -> Self {
        Self::analysis_error("unknown file", 0, &error.to_string(), "analysis operation")
    }
}

impl From<ExtractionError> for UveddiError {
    fn from(error: ExtractionError) -> Self {
        Self::ExtractionError {
            path: "unknown path".to_string(),
            message: error.to_string(),
            cause: "extraction operation".to_string(),
            recovery_hint: "Check file format and language support".to_string(),
            source: Some(error),
        }
    }
}

impl From<DependencyExtractionError> for UveddiError {
    fn from(error: DependencyExtractionError) -> Self {
        Self::DependencyExtractionError {
            module: "unknown module".to_string(),
            message: error.to_string(),
            suggestion: "Check module dependencies and imports".to_string(),
            source: Some(error),
        }
    }
}

impl From<PluginError> for UveddiError {
    fn from(error: PluginError) -> Self {
        Self::PluginError {
            plugin: "unknown plugin".to_string(),
            plugin_type: "WASM".to_string(),
            message: error.to_string(),
            suggestion: "Check plugin configuration and WASM module".to_string(),
            source: Some(error),
        }
    }
}

impl From<SecurityError> for UveddiError {
    fn from(error: SecurityError) -> Self {
        Self::SecurityError {
            message: error.to_string(),
            context: "security check".to_string(),
            action_required: "Review security policies and access controls".to_string(),
            source: Some(error),
        }
    }
}

impl From<RenderingServiceError> for UveddiError {
    fn from(error: RenderingServiceError) -> Self {
        Self::RenderingServiceError {
            service_url: "unknown service".to_string(),
            message: error.to_string(),
            suggestion: "Check service availability and network connectivity".to_string(),
            source: Some(error),
        }
    }
}

impl From<ReportGenerationError> for UveddiError {
    fn from(error: ReportGenerationError) -> Self {
        Self::ReportError {
            report_type: "unknown report".to_string(),
            target: "unknown target".to_string(),
            stage: "generation".to_string(),
            suggestion: "Check report template and data availability".to_string(),
            source: Some(error),
        }
    }
}

impl From<ClapError> for UveddiError {
    fn from(error: ClapError) -> Self {
        Self::CliError {
            command: "unknown command".to_string(),
            message: error.to_string(),
            suggestion: "Check command syntax and available options".to_string(),
            source: Some(error),
        }
    }
}

impl From<anyhow::Error> for UveddiError {
    fn from(error: anyhow::Error) -> Self {
        // Try to provide more specific context based on the error message
        let error_msg = error.to_string();
        let (context, suggestion) = if error_msg.contains("database") {
            (
                "database operation",
                "Check database connection and schema integrity",
            )
        } else if error_msg.contains("parse") || error_msg.contains("syntax") {
            ("code parsing", "Verify file syntax and encoding")
        } else if error_msg.contains("memory") || error_msg.contains("allocation") {
            (
                "memory management",
                "Increase available memory or reduce analysis scope",
            )
        } else if error_msg.contains("timeout") {
            (
                "operation timeout",
                "Increase timeout values or analyze smaller codebases",
            )
        } else if error_msg.contains("permission") || error_msg.contains("access") {
            ("file access", "Check file permissions and access rights")
        } else if error_msg.contains("network") || error_msg.contains("connection") {
            (
                "network operation",
                "Check network connectivity and firewall settings",
            )
        } else {
            (
                "system operation",
                "Check logs for detailed error information",
            )
        };

        Self::GenericError {
            message: error.to_string(),
            context: context.to_string(),
            suggestion: suggestion.to_string(),
            source: Some(error),
        }
    }
}

pub trait ErrorHandler {
    fn handle_error(&self, error: &UveddiError);
    fn log_error(&self, error: &UveddiError);
    fn can_retry(&self, error: &UveddiError) -> bool;
}
