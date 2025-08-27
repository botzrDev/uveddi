use thiserror::Error;

/// Errors that can occur during report generation
#[derive(Error, Debug)]
pub enum ReportGenerationError {
    /// Template rendering failed
    #[error("Template rendering error: {0}")]
    TemplateError(String),
    /// Data extraction failed
    #[error("Data extraction error: {0}")]
    DataExtractionError(String),
    /// File write operation failed
    #[error("File write error: {0}")]
    FileWriteError(String),
    /// Unknown error occurred
    #[error("Unknown report error: {0}")]
    Unknown(String),
}
