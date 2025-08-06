//! Secure dependency configuration and validation
//! 
//! This module provides secure configurations for external dependencies
//! and implements security best practices for HTTP clients, XML parsing, etc.

use reqwest::ClientBuilder;
use std::time::Duration;
use thiserror::Error;

/// Create secure HTTP client with proper TLS configuration
pub fn create_secure_http_client() -> Result<reqwest::Client, SecurityError> {
    ClientBuilder::new()
        .timeout(Duration::from_secs(30))
        .use_rustls_tls()  // Use rustls instead of native-tls for better security
        .https_only(true)  // Reject HTTP connections
        .build()
        .map_err(|e| SecurityError::HttpClientCreation(e.to_string()))
}

/// Security configuration for handling external data with size limits
pub struct SecurityLimits {
    pub max_xml_size: usize,
    pub max_json_size: usize,
    pub max_request_size: usize,
}

impl Default for SecurityLimits {
    fn default() -> Self {
        Self {
            max_xml_size: 1_000_000,    // 1MB limit for XML
            max_json_size: 10_000_000,  // 10MB limit for JSON
            max_request_size: 50_000_000, // 50MB limit for requests
        }
    }
}

/// Validate input size against security limits
pub fn validate_input_size(input: &str, limit: usize, data_type: &str) -> Result<(), SecurityError> {
    if input.len() > limit {
        return Err(SecurityError::InputTooLarge {
            data_type: data_type.to_string(),
            size: input.len(),
            limit,
        });
    }
    Ok(())
}

/// Create secure HTTP client with custom configuration
pub fn create_custom_http_client(
    timeout_secs: u64,
    user_agent: &str,
) -> Result<reqwest::Client, SecurityError> {
    ClientBuilder::new()
        .timeout(Duration::from_secs(timeout_secs))
        .user_agent(user_agent)
        .use_rustls_tls()
        .https_only(true)
        .build()
        .map_err(|e| SecurityError::HttpClientCreation(e.to_string()))
}

#[derive(Debug, Error)]
pub enum SecurityError {
    #[error("HTTP client creation failed: {0}")]
    HttpClientCreation(String),
    
    #[error("{data_type} document too large: {size} bytes (limit: {limit})")]
    InputTooLarge {
        data_type: String,
        size: usize,
        limit: usize,
    },
    
    #[error("Security validation failed: {0}")]
    ValidationFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_secure_http_client_creation() {
        let client = create_secure_http_client();
        assert!(client.is_ok());
    }

    #[test]
    fn test_input_size_validation() {
        let small_input = "small";
        let large_input = "x".repeat(2_000_000);
        
        assert!(validate_input_size(small_input, 1_000_000, "test").is_ok());
        assert!(validate_input_size(&large_input, 1_000_000, "test").is_err());
    }

    #[tokio::test]
    async fn test_custom_http_client() {
        let client = create_custom_http_client(10, "TestAgent/1.0");
        assert!(client.is_ok());
    }
}