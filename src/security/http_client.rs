//! Secure HTTP Client Module
//!
//! This module provides a secure HTTP client implementation with HTTPS enforcement,
//! certificate validation, secure timeouts, and other security best practices.
//!
//! # Security Features
//!
//! - **HTTPS Enforcement**: All requests must use HTTPS
//! - **Certificate Validation**: Strict certificate validation enabled
//! - **Secure Timeouts**: Configurable timeouts to prevent hanging requests
//! - **Connection Limits**: Prevent resource exhaustion
//! - **Header Security**: Secure default headers
//!
//! # Usage
//!
//! ```rust,no_run
//! use uveddi::security::http_client::{SecureHttpClient, HttpSecurityConfig};
//!
//! let config = HttpSecurityConfig::default();
//! let client = SecureHttpClient::new(config)?;
//!
//! // Make a secure HTTPS request
//! let response = client.get("https://api.example.com/data").await?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::security::errors::{SecurityError, SecurityResult};
use reqwest::{Client, ClientBuilder, Response};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use url::Url;

/// HTTP Security Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpSecurityConfig {
    /// Enforce HTTPS for all requests
    pub enforce_https: bool,
    /// Verify SSL/TLS certificates
    pub verify_certificates: bool,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
    /// Maximum number of redirects to follow
    pub max_redirects: u32,
    /// Connection timeout in seconds
    pub connect_timeout_seconds: u64,
    /// Read timeout in seconds
    pub read_timeout_seconds: u64,
    /// Maximum number of idle connections per host
    pub max_idle_per_host: usize,
    /// TCP keepalive timeout in seconds
    pub tcp_keepalive_seconds: Option<u64>,
    /// User agent string for requests
    pub user_agent: String,
    /// Enable gzip compression
    pub enable_gzip: bool,
}

impl Default for HttpSecurityConfig {
    fn default() -> Self {
        Self {
            enforce_https: true,
            verify_certificates: true,
            timeout_seconds: 30,
            max_redirects: 3,
            connect_timeout_seconds: 10,
            read_timeout_seconds: 30,
            max_idle_per_host: 5,
            tcp_keepalive_seconds: Some(60),
            user_agent: format!("Uveddi-SecureClient/{}", env!("CARGO_PKG_VERSION")),
            enable_gzip: true,
        }
    }
}

/// Secure HTTP Client with enforced security policies
#[derive(Debug)]
pub struct SecureHttpClient {
    client: Client,
    config: HttpSecurityConfig,
}

impl SecureHttpClient {
    /// Create a new secure HTTP client with the given configuration
    pub fn new(config: HttpSecurityConfig) -> SecurityResult<Self> {
        // Validate configuration
        config.validate()?;

        let mut builder = ClientBuilder::new()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .connect_timeout(Duration::from_secs(config.connect_timeout_seconds))
            .read_timeout(Duration::from_secs(config.read_timeout_seconds))
            .redirect(reqwest::redirect::Policy::limited(
                config.max_redirects as usize,
            ))
            .user_agent(&config.user_agent)
            .pool_max_idle_per_host(config.max_idle_per_host);

        // Note: In reqwest 0.12+, gzip compression is enabled by default
        // The enable_gzip config option is kept for API compatibility but not used
        // since reqwest automatically handles compression

        // Configure HTTPS enforcement
        if config.enforce_https {
            builder = builder.https_only(true);
        }

        // Configure certificate verification
        if !config.verify_certificates {
            // Only allow disabling certificate verification in development
            #[cfg(debug_assertions)]
            {
                builder = builder.danger_accept_invalid_certs(true);
            }
            #[cfg(not(debug_assertions))]
            {
                return Err(SecurityError::ConfigurationError {
                    message: "Certificate verification cannot be disabled in production builds"
                        .to_string(),
                });
            }
        }

        // Configure TCP keepalive
        if let Some(keepalive) = config.tcp_keepalive_seconds {
            builder = builder.tcp_keepalive(Some(Duration::from_secs(keepalive)));
        }

        let client = builder
            .build()
            .map_err(|e| SecurityError::HttpClientError {
                message: format!("Failed to create HTTP client: {}", e),
            })?;

        Ok(Self { client, config })
    }

    /// Create a new secure HTTP client with default configuration
    pub fn new_default() -> SecurityResult<Self> {
        Self::new(HttpSecurityConfig::default())
    }

    /// Perform a GET request with security validation
    pub async fn get(&self, url: &str) -> SecurityResult<Response> {
        self.validate_url(url)?;

        let response =
            self.client
                .get(url)
                .send()
                .await
                .map_err(|e| SecurityError::HttpRequestError {
                    message: format!("GET request failed: {}", e),
                })?;

        self.validate_response(&response)?;
        Ok(response)
    }

    /// Perform a POST request with security validation
    pub async fn post(
        &self,
        url: &str,
        body: impl Into<reqwest::Body>,
    ) -> SecurityResult<Response> {
        self.validate_url(url)?;

        let response = self
            .client
            .post(url)
            .body(body)
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| SecurityError::HttpRequestError {
                message: format!("POST request failed: {}", e),
            })?;

        self.validate_response(&response)?;
        Ok(response)
    }

    /// Perform a PUT request with security validation
    pub async fn put(&self, url: &str, body: impl Into<reqwest::Body>) -> SecurityResult<Response> {
        self.validate_url(url)?;

        let response = self
            .client
            .put(url)
            .body(body)
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| SecurityError::HttpRequestError {
                message: format!("PUT request failed: {}", e),
            })?;

        self.validate_response(&response)?;
        Ok(response)
    }

    /// Perform a DELETE request with security validation
    pub async fn delete(&self, url: &str) -> SecurityResult<Response> {
        self.validate_url(url)?;

        let response =
            self.client
                .delete(url)
                .send()
                .await
                .map_err(|e| SecurityError::HttpRequestError {
                    message: format!("DELETE request failed: {}", e),
                })?;

        self.validate_response(&response)?;
        Ok(response)
    }

    /// Get the underlying reqwest client (for advanced usage)
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Get the HTTP security configuration
    pub fn config(&self) -> &HttpSecurityConfig {
        &self.config
    }

    /// Validate URL for security compliance
    fn validate_url(&self, url: &str) -> SecurityResult<()> {
        let parsed_url = Url::parse(url).map_err(|e| SecurityError::InvalidInput {
            field: "url".to_string(),
            reason: format!("Invalid URL format: {}", e),
        })?;

        // Enforce HTTPS if configured
        if self.config.enforce_https && parsed_url.scheme() != "https" {
            return Err(SecurityError::HttpsRequired {
                url: url.to_string(),
            });
        }

        // Block potentially dangerous hosts
        if let Some(host) = parsed_url.host_str() {
            // Block localhost/loopback addresses in production
            #[cfg(not(debug_assertions))]
            {
                if host == "localhost" || host == "127.0.0.1" || host == "::1" {
                    return Err(SecurityError::InvalidInput {
                        field: "url".to_string(),
                        reason: "Localhost URLs are not allowed in production".to_string(),
                    });
                }
            }

            // Block private IP ranges in production
            #[cfg(not(debug_assertions))]
            {
                if host.starts_with("10.")
                    || host.starts_with("192.168.")
                    || (host.starts_with("172.")
                        && host
                            .split('.')
                            .nth(1)
                            .and_then(|s| s.parse::<u8>().ok())
                            .map_or(false, |n| n >= 16 && n <= 31))
                {
                    return Err(SecurityError::InvalidInput {
                        field: "url".to_string(),
                        reason: "Private IP addresses are not allowed in production".to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    /// Validate HTTP response for security issues
    fn validate_response(&self, response: &Response) -> SecurityResult<()> {
        // Log response status for monitoring
        log::debug!("HTTP response: {} {}", response.status(), response.url());

        // Check for suspicious response headers
        if let Some(server) = response.headers().get("server") {
            if let Ok(server_str) = server.to_str() {
                // Log server header for security monitoring
                log::debug!("Server header: {}", server_str);
            }
        }

        // Validate content length to prevent memory exhaustion
        if let Some(content_length) = response.content_length() {
            const MAX_CONTENT_LENGTH: u64 = 100 * 1024 * 1024; // 100MB
            if content_length > MAX_CONTENT_LENGTH {
                return Err(SecurityError::HttpRequestError {
                    message: format!(
                        "Response content length {} exceeds maximum allowed {}",
                        content_length, MAX_CONTENT_LENGTH
                    ),
                });
            }
        }

        Ok(())
    }
}

impl HttpSecurityConfig {
    /// Validate the HTTP security configuration
    pub fn validate(&self) -> SecurityResult<()> {
        if self.timeout_seconds == 0 {
            return Err(SecurityError::ConfigurationError {
                message: "Timeout seconds must be greater than 0".to_string(),
            });
        }

        if self.connect_timeout_seconds == 0 {
            return Err(SecurityError::ConfigurationError {
                message: "Connect timeout seconds must be greater than 0".to_string(),
            });
        }

        if self.read_timeout_seconds == 0 {
            return Err(SecurityError::ConfigurationError {
                message: "Read timeout seconds must be greater than 0".to_string(),
            });
        }

        if self.max_redirects > 10 {
            return Err(SecurityError::ConfigurationError {
                message: "Maximum redirects should not exceed 10 for security".to_string(),
            });
        }

        if self.max_idle_per_host == 0 {
            return Err(SecurityError::ConfigurationError {
                message: "Max idle per host must be greater than 0".to_string(),
            });
        }

        if self.user_agent.is_empty() {
            return Err(SecurityError::ConfigurationError {
                message: "User agent cannot be empty".to_string(),
            });
        }

        // Validate production security requirements
        #[cfg(not(debug_assertions))]
        {
            if !self.enforce_https {
                return Err(SecurityError::ConfigurationError {
                    message: "HTTPS enforcement cannot be disabled in production".to_string(),
                });
            }

            if !self.verify_certificates {
                return Err(SecurityError::ConfigurationError {
                    message: "Certificate verification cannot be disabled in production"
                        .to_string(),
                });
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[test]
    fn test_default_config_validation() {
        let config = HttpSecurityConfig::default();
        assert!(config.validate().is_ok());
        assert!(config.enforce_https);
        assert!(config.verify_certificates);
        assert_eq!(config.timeout_seconds, 30);
    }

    #[test]
    fn test_config_validation_errors() {
        let mut config = HttpSecurityConfig::default();

        // Test zero timeout
        config.timeout_seconds = 0;
        assert!(config.validate().is_err());

        // Reset and test empty user agent
        config = HttpSecurityConfig::default();
        config.user_agent = String::new();
        assert!(config.validate().is_err());

        // Reset and test excessive redirects
        config = HttpSecurityConfig::default();
        config.max_redirects = 20;
        assert!(config.validate().is_err());
    }

    #[tokio::test]
    async fn test_secure_client_creation() {
        let config = HttpSecurityConfig::default();
        let client = SecureHttpClient::new(config);
        assert!(client.is_ok());
    }

    #[test]
    fn test_url_validation() {
        let config = HttpSecurityConfig::default();
        let client = SecureHttpClient::new(config).unwrap();

        // Valid HTTPS URL should pass
        assert!(client.validate_url("https://api.example.com/data").is_ok());

        // HTTP URL should fail when HTTPS is enforced
        assert!(client.validate_url("http://api.example.com/data").is_err());

        // Invalid URL should fail
        assert!(client.validate_url("not-a-url").is_err());
    }

    #[test]
    fn test_development_vs_production_config() {
        let mut config = HttpSecurityConfig::default();

        #[cfg(debug_assertions)]
        {
            // In debug mode, we can disable certificate verification
            config.verify_certificates = false;
            assert!(config.validate().is_ok());
        }

        #[cfg(not(debug_assertions))]
        {
            // In production, disabling HTTPS should fail
            config.enforce_https = false;
            assert!(config.validate().is_err());

            // Reset and test certificate verification
            config = HttpSecurityConfig::default();
            config.verify_certificates = false;
            assert!(config.validate().is_err());
        }
    }

    #[test]
    fn test_http_client_creation_with_fixed_types() {
        // Test that our type fixes work correctly
        let config = HttpSecurityConfig {
            enforce_https: true,
            verify_certificates: true,
            timeout_seconds: 30,
            connect_timeout_seconds: 10,
            read_timeout_seconds: 30,
            max_redirects: 5u32, // This should work with our u32 -> usize conversion
            max_idle_per_host: 10,
            tcp_keepalive_seconds: Some(60),
            user_agent: "TestAgent/1.0".to_string(),
            enable_gzip: true, // This should be handled gracefully (ignored in reqwest 0.12+)
        };

        // This should not panic or fail to compile
        let result = SecureHttpClient::new(config);
        assert!(
            result.is_ok(),
            "HTTP client creation should succeed with fixed types"
        );
    }
}
