use thiserror::Error;

#[derive(Error, Debug)]
pub enum ReportGenerationError {
    #[error("Template rendering error: {0}")]
    TemplateError(String),
    #[error("Data extraction error: {0}")]
    DataExtractionError(String),
    #[error("File write error: {0}")]
    FileWriteError(String),
    #[error("Unknown report error: {0}")]
    Unknown(String),
}
