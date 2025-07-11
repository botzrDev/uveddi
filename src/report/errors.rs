use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReportGenerationError {
    #[error("Template rendering error: {0}")]
    TemplateRenderingError(String),
    #[error("Data serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("File I/O error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Invalid report format: {0}")]
    InvalidFormatError(String),
    // Add other report generation errors as needed
}
