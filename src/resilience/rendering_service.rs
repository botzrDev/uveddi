use thiserror::Error;

#[derive(Error, Debug)]
pub enum RenderingServiceError {
    #[error("Service unavailable: {0}")]
    Unavailable(String),
    #[error("Timeout: {0}")]
    Timeout(String),
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    // Add other rendering service errors as needed
}
