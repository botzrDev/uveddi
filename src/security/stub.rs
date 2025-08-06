//! Security Module Stub
//!
//! This module provides stub implementations of security features
//! for builds where the 'security' feature is not enabled.

/// Stub for SecurityError - only basic functionality is provided
#[derive(Debug, thiserror::Error)]
pub enum SecurityError {
    #[error("Security feature disabled: {0}")]
    FeatureDisabled(String),
}

/// Common security result type
pub type SecurityResult<T> = Result<T, SecurityError>;

/// Stub implementation for input validation
pub fn validate_input(_input: &str, _max_length: usize) -> SecurityResult<()> {
    Ok(()) // No validation when security is disabled
}

/// Stub implementation for URL validation
pub fn validate_url(_url: &str) -> SecurityResult<()> {
    Ok(()) // No validation when security is disabled
}

/// Stub implementation for numeric range validation
pub fn validate_numeric_range(_value: i64, _min: i64, _max: i64) -> SecurityResult<()> {
    Ok(()) // No validation when security is disabled
}

/// Stub implementation for model name validation
pub fn validate_model_name(_model_name: &str) -> SecurityResult<()> {
    Ok(()) // No validation when security is disabled
}

/// HTTP security configuration (stub)
pub struct HttpSecurityConfig {
    pub timeout_seconds: u64,
}

impl Default for HttpSecurityConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 30,
        }
    }
}

/// Stub implementation of secure HTTP client
#[derive(Clone)]
pub struct SecureHttpClient {
    client: reqwest::Client,
}

impl SecureHttpClient {
    /// Create a new secure HTTP client with default configuration
    pub fn new() -> Result<Self, SecurityError> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| SecurityError::FeatureDisabled(format!("Failed to create HTTP client: {}", e)))?;
            
        Ok(Self { client })
    }
    
    /// Create a new secure HTTP client with custom configuration
    pub fn with_config(_config: HttpSecurityConfig) -> Result<Self, SecurityError> {
        Self::new() // Use default implementation in stub version
    }
    
    /// Get the underlying reqwest client
    pub fn client(&self) -> &reqwest::Client {
        &self.client
    }
}
