use std::path::PathBuf;
use crate::{
    ast::tree_sitter_impl::AstError,
    plugins::errors::PluginError,
    analysis::errors::AnalysisError,
    report::errors::ReportGenerationError,
    analysis::detectors::dependency::ExtractionError as DependencyExtractionError,
};
use clap::error::Error as ClapError;
use reqwest::Error as ReqwestError;
use rusqlite::Error as RusqliteError;
use thiserror::Error;
use serde::{Serialize, Deserialize};
use crate::error::rendering::RenderingServiceError;

#[derive(Error, Debug)]
pub enum ExtractionError {
    #[error("AST extraction error: {0}")]
    AstError(AstError),
    #[error("File read error: {0}")]
    FileReadError(#[from] std::io::Error),
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),
}

#[derive(Error, Debug)]
pub enum UveddiError {
    // === Core Analysis Errors ===
    #[error("Analysis error: {0}")]
    AnalysisError(#[from] AnalysisError),
    #[error("Extraction error: {0}")]
    ExtractionError(#[from] ExtractionError),
    #[error("AST error: {0}")]
    AstError(#[from] AstError),
    #[error("Dependency extraction error: {0}")]
    DependencyExtractionError(#[from] DependencyExtractionError),
    // === Service & Infrastructure Errors ===
    #[error("Rendering service error: {0}")]
    RenderingServiceError(#[from] RenderingServiceError),
    #[error("Database error: {0}")]
    DatabaseError(#[from] RusqliteError),
    #[error("Plugin error: {0}")]
    PluginError(#[from] PluginError),
    // === External System Errors ===
    #[error("Network error: {0}")]
    NetworkError(#[from] ReqwestError),
    #[error("Report generation error: {0}")]
    ReportError(#[from] ReportGenerationError),
    // === System Errors ===
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("Command line error: {0}")]
    CliError(#[from] ClapError),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Path error: {0}")]
    PathError(PathBuf),
    #[error("Generic error: {0}")]
    GenericError(#[from] anyhow::Error),
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
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            UveddiError::AnalysisError(_)
            | UveddiError::ExtractionError(_)
            | UveddiError::AstError(_)
            | UveddiError::DependencyExtractionError(_)
            | UveddiError::DatabaseError(_)
            | UveddiError::PluginError(_) => ErrorSeverity::High,
            UveddiError::RenderingServiceError(_)
            | UveddiError::ReportError(_)
            | UveddiError::NetworkError(_) => ErrorSeverity::Medium,
            _ => ErrorSeverity::Low,
        }
    }
    pub fn category(&self) -> ErrorCategory {
        match self {
            UveddiError::AnalysisError(_) => ErrorCategory::Analysis,
            UveddiError::ExtractionError(_) => ErrorCategory::Extraction,
            UveddiError::AstError(_) => ErrorCategory::Analysis,
            UveddiError::DependencyExtractionError(_) => ErrorCategory::Extraction,
            UveddiError::DatabaseError(_) => ErrorCategory::Database,
            UveddiError::ConfigError(_) => ErrorCategory::Configuration,
            UveddiError::ReportError(_) => ErrorCategory::Reporting,
            UveddiError::PluginError(_) => ErrorCategory::Plugin,
            UveddiError::NetworkError(_) => ErrorCategory::Network,
            UveddiError::CliError(_) => ErrorCategory::Cli,
            UveddiError::IoError(_) => ErrorCategory::Io,
            UveddiError::SerializationError(_) => ErrorCategory::Serialization,
            UveddiError::PathError(_) => ErrorCategory::Path,
            UveddiError::RenderingServiceError(_) => ErrorCategory::Rendering,
            UveddiError::GenericError(_) => ErrorCategory::Generic,
        }
    }
}

pub trait ErrorHandler {
    fn handle_error(&self, error: &UveddiError);
    fn log_error(&self, error: &UveddiError);
    fn can_retry(&self, error: &UveddiError) -> bool;
}
