//! Rendering Service Error Types
//!
//! This module defines comprehensive error types for the Uveddi image rendering service,
//! providing structured error handling for all failure scenarios including service
//! communication, resource exhaustion, data validation, and security issues.
//!
//! These error types serve as the foundation for:
//! - Retry logic decision making (UV-168)
//! - Structured logging categorization (UV-170)
//! - Circuit breaker state management (UV-172)
//! - Fallback strategy triggers (UV-171, UV-177)
//! - Metrics collection and alerting (UV-173, UV-174)

use std::time::Duration;
use thiserror::Error;

/// Comprehensive error types for the rendering service with detailed context
#[derive(Error, Debug, Clone, PartialEq)]
pub enum RenderingServiceError {
    // === Service Communication Errors ===
    /// Rendering service is completely unavailable
    #[error("Rendering service unavailable")]
    ServiceUnavailable,

    /// Connection timeout when establishing connection to service
    #[error("Connection timeout after {timeout}ms")]
    ConnectionTimeout { timeout: u64 },

    /// Request timeout waiting for service response
    #[error("Request timeout after {timeout}ms")]
    RequestTimeout { timeout: u64 },

    /// Authentication failed when connecting to service
    #[error("Authentication failed: {reason}")]
    AuthenticationFailure { reason: String },

    /// Rate limit exceeded by the service
    #[error("Rate limit exceeded{}", match .retry_after_seconds { Some(s) => format!(" - retry after {}s", s), None => String::new() })]
    RateLimitExceeded { retry_after_seconds: Option<u64> },

    /// Invalid or malformed response from service
    #[error("Invalid response from service: {details}")]
    InvalidResponse { details: String },

    /// Network connectivity issues
    #[error("Network error: {message}")]
    NetworkError { message: String },

    // === Resource Exhaustion Errors ===
    /// Memory limit exceeded during rendering
    #[error("Memory limit exceeded: {used_mb}MB used, {limit_mb}MB limit")]
    MemoryExhaustion { used_mb: u64, limit_mb: u64 },

    /// CPU timeout exceeded during rendering
    #[error("CPU timeout exceeded: {elapsed_ms}ms elapsed, {limit_ms}ms limit")]
    CpuTimeout { elapsed_ms: u64, limit_ms: u64 },

    /// Disk space insufficient for temporary files
    #[error("Disk space insufficient: {available_mb}MB available, {required_mb}MB required")]
    DiskSpaceExhaustion { available_mb: u64, required_mb: u64 },

    /// Network bandwidth limit exceeded
    #[error(
        "Network bandwidth limit exceeded: {current_mbps}Mbps current, {limit_mbps}Mbps limit"
    )]
    NetworkBandwidthLimit { current_mbps: f64, limit_mbps: f64 },

    // === Data Validation Errors ===
    /// Invalid Mermaid syntax in diagram
    #[error("Invalid Mermaid syntax{}: {details}", match .line { Some(l) => format!(" at line {}", l), None => String::new() })]
    InvalidMermaidSyntax { line: Option<u32>, details: String },

    /// Diagram complexity exceeds service limits
    #[error("Diagram too complex: {reason} (limit: {limit}, actual: {actual})")]
    DiagramComplexityExceeded {
        reason: String,
        limit: u64,
        actual: u64,
    },

    /// Input size exceeds maximum allowed
    #[error("Input size limit exceeded: {size_bytes} bytes (limit: {limit_bytes} bytes)")]
    InputTooLarge {
        size_bytes: usize,
        limit_bytes: usize,
    },

    /// Unsupported diagram type or format
    #[error("Unsupported diagram type: {diagram_type}")]
    UnsupportedDiagramType { diagram_type: String },

    // === Security Validation Errors ===
    /// Security validation failed
    #[error("Security validation failed: {reason}")]
    SecurityValidationFailed { reason: String },

    /// Malicious content detected in diagram
    #[error("Malicious content detected: {threat_type}")]
    MaliciousContentDetected { threat_type: String },

    // === Service-Specific Errors ===
    /// Puppeteer service specific error
    #[error("Puppeteer error: {message}")]
    PuppeteerError { message: String },

    /// Image format conversion error
    #[error("Image format conversion failed: from {from_format} to {to_format} - {reason}")]
    ImageConversionError {
        from_format: String,
        to_format: String,
        reason: String,
    },

    /// Rendering queue is full
    #[error("Rendering queue full: {queue_size} items (limit: {max_queue_size})")]
    QueueFull {
        queue_size: u32,
        max_queue_size: u32,
    },
}

impl From<reqwest::Error> for RenderingServiceError {
    fn from(e: reqwest::Error) -> Self {
        if e.is_timeout() {
            RenderingServiceError::request_timeout(Duration::from_secs(30))
        } else if e.is_connect() {
            RenderingServiceError::connection_timeout(Duration::from_secs(10))
        } else {
            RenderingServiceError::NetworkError {
                message: e.to_string(),
            }
        }
    }
}

impl RenderingServiceError {
    /// Determines if this error type should trigger retry logic
    pub fn is_retryable(&self) -> bool {
        match self {
            // Retryable errors - transient failures
            Self::ServiceUnavailable => true,
            Self::ConnectionTimeout { .. } => true,
            Self::RequestTimeout { .. } => true,
            Self::NetworkError { .. } => true,
            Self::MemoryExhaustion { .. } => true,
            Self::CpuTimeout { .. } => true,
            Self::DiskSpaceExhaustion { .. } => false, // Unlikely to resolve quickly
            Self::NetworkBandwidthLimit { .. } => true,
            Self::QueueFull { .. } => true,
            Self::PuppeteerError { .. } => true, // May be transient
            Self::RateLimitExceeded { .. } => true, // With backoff
            Self::InvalidResponse { .. } => false, // Likely persistent

            // Non-retryable errors - permanent failures
            Self::AuthenticationFailure { .. } => false,
            Self::InvalidMermaidSyntax { .. } => false,
            Self::DiagramComplexityExceeded { .. } => false,
            Self::InputTooLarge { .. } => false,
            Self::UnsupportedDiagramType { .. } => false,
            Self::SecurityValidationFailed { .. } => false,
            Self::MaliciousContentDetected { .. } => false,
            Self::ImageConversionError { .. } => false,
        }
    }

    /// Determines if this error should trigger circuit breaker opening
    pub fn should_open_circuit(&self) -> bool {
        match self {
            // Errors indicating service health issues
            Self::ServiceUnavailable => true,
            Self::ConnectionTimeout { .. } => true,
            Self::RequestTimeout { .. } => true,
            Self::NetworkError { .. } => true,
            Self::MemoryExhaustion { .. } => true,
            Self::CpuTimeout { .. } => true,
            Self::PuppeteerError { .. } => true,
            Self::InvalidResponse { .. } => true,

            // Errors that don't indicate service health issues
            Self::AuthenticationFailure { .. } => false,
            Self::RateLimitExceeded { .. } => false,
            Self::DiskSpaceExhaustion { .. } => false,
            Self::NetworkBandwidthLimit { .. } => false,
            Self::InvalidMermaidSyntax { .. } => false,
            Self::DiagramComplexityExceeded { .. } => false,
            Self::InputTooLarge { .. } => false,
            Self::UnsupportedDiagramType { .. } => false,
            Self::SecurityValidationFailed { .. } => false,
            Self::MaliciousContentDetected { .. } => false,
            Self::ImageConversionError { .. } => false,
            Self::QueueFull { .. } => false,
        }
    }

    /// Determines if this error should trigger fallback mode
    pub fn should_trigger_fallback(&self) -> bool {
        match self {
            // Errors that should trigger fallback to Mermaid-only mode
            Self::ServiceUnavailable => true,
            Self::ConnectionTimeout { .. } => true,
            Self::RequestTimeout { .. } => true,
            Self::NetworkError { .. } => true,
            Self::MemoryExhaustion { .. } => true,
            Self::CpuTimeout { .. } => true,
            Self::DiskSpaceExhaustion { .. } => true,
            Self::NetworkBandwidthLimit { .. } => true,
            Self::PuppeteerError { .. } => true,
            Self::QueueFull { .. } => true,
            Self::InvalidResponse { .. } => true,

            // Errors that should not trigger fallback (user/input errors)
            Self::AuthenticationFailure { .. } => false,
            Self::RateLimitExceeded { .. } => false,
            Self::InvalidMermaidSyntax { .. } => false,
            Self::DiagramComplexityExceeded { .. } => false,
            Self::InputTooLarge { .. } => false,
            Self::UnsupportedDiagramType { .. } => false,
            Self::SecurityValidationFailed { .. } => false,
            Self::MaliciousContentDetected { .. } => false,
            Self::ImageConversionError { .. } => false,
        }
    }

    /// Gets the error severity level for logging and alerting
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            // Critical - service unavailable
            Self::ServiceUnavailable => ErrorSeverity::Critical,
            Self::MemoryExhaustion { .. } => ErrorSeverity::Critical,
            Self::DiskSpaceExhaustion { .. } => ErrorSeverity::Critical,

            // High - significant service degradation
            Self::ConnectionTimeout { .. } => ErrorSeverity::High,
            Self::RequestTimeout { .. } => ErrorSeverity::High,
            Self::NetworkError { .. } => ErrorSeverity::High,
            Self::CpuTimeout { .. } => ErrorSeverity::High,
            Self::PuppeteerError { .. } => ErrorSeverity::High,
            Self::QueueFull { .. } => ErrorSeverity::High,
            Self::InvalidResponse { .. } => ErrorSeverity::High,

            // Medium - operational issues
            Self::AuthenticationFailure { .. } => ErrorSeverity::Medium,
            Self::RateLimitExceeded { .. } => ErrorSeverity::Medium,
            Self::NetworkBandwidthLimit { .. } => ErrorSeverity::Medium,
            Self::ImageConversionError { .. } => ErrorSeverity::Medium,

            // Low - user/input errors
            Self::InvalidMermaidSyntax { .. } => ErrorSeverity::Low,
            Self::DiagramComplexityExceeded { .. } => ErrorSeverity::Low,
            Self::InputTooLarge { .. } => ErrorSeverity::Low,
            Self::UnsupportedDiagramType { .. } => ErrorSeverity::Low,

            // Security - special handling
            Self::SecurityValidationFailed { .. } => ErrorSeverity::Security,
            Self::MaliciousContentDetected { .. } => ErrorSeverity::Security,
        }
    }

    /// Gets the error category for metrics collection
    pub fn category(&self) -> ErrorCategory {
        match self {
            Self::ServiceUnavailable
            | Self::ConnectionTimeout { .. }
            | Self::RequestTimeout { .. }
            | Self::NetworkError { .. }
            | Self::AuthenticationFailure { .. }
            | Self::RateLimitExceeded { .. }
            | Self::InvalidResponse { .. } => ErrorCategory::ServiceCommunication,

            Self::MemoryExhaustion { .. }
            | Self::CpuTimeout { .. }
            | Self::DiskSpaceExhaustion { .. }
            | Self::NetworkBandwidthLimit { .. }
            | Self::QueueFull { .. } => ErrorCategory::ResourceExhaustion,

            Self::InvalidMermaidSyntax { .. }
            | Self::DiagramComplexityExceeded { .. }
            | Self::InputTooLarge { .. }
            | Self::UnsupportedDiagramType { .. } => ErrorCategory::DataValidation,

            Self::SecurityValidationFailed { .. } | Self::MaliciousContentDetected { .. } => {
                ErrorCategory::Security
            }

            Self::PuppeteerError { .. } | Self::ImageConversionError { .. } => {
                ErrorCategory::ServiceSpecific
            }
        }
    }

    /// Gets user-friendly error message (safe for external display)
    pub fn user_message(&self) -> String {
        match self {
            Self::ServiceUnavailable => {
                "The diagram rendering service is temporarily unavailable. Please try again later."
                    .to_string()
            }
            Self::ConnectionTimeout { .. } | Self::RequestTimeout { .. } => {
                "The rendering request timed out. Please try again.".to_string()
            }
            Self::NetworkError { .. } => {
                "A network error occurred. Please check your connection and try again.".to_string()
            }
            Self::AuthenticationFailure { .. } => {
                "Authentication failed. Please check your credentials.".to_string()
            }
            Self::RateLimitExceeded {
                retry_after_seconds,
            } => match retry_after_seconds {
                Some(seconds) => format!(
                    "Rate limit exceeded. Please wait {} seconds before trying again.",
                    seconds
                ),
                None => "Rate limit exceeded. Please wait before trying again.".to_string(),
            },
            Self::InvalidMermaidSyntax { line, details } => match line {
                Some(line_num) => {
                    format!("Invalid diagram syntax at line {}: {}", line_num, details)
                }
                None => format!("Invalid diagram syntax: {}", details),
            },
            Self::DiagramComplexityExceeded { reason, .. } => {
                format!("Diagram is too complex: {}", reason)
            }
            Self::InputTooLarge {
                size_bytes,
                limit_bytes,
            } => {
                format!(
                    "Input size ({} bytes) exceeds the maximum allowed ({} bytes)",
                    size_bytes, limit_bytes
                )
            }
            Self::UnsupportedDiagramType { diagram_type } => {
                format!("Unsupported diagram type: {}", diagram_type)
            }
            Self::SecurityValidationFailed { .. } => {
                "Security validation failed. Please review your diagram content.".to_string()
            }
            Self::MaliciousContentDetected { .. } => {
                "Potentially malicious content detected. Please review your diagram.".to_string()
            }
            _ => "An error occurred while rendering the diagram. Please try again.".to_string(),
        }
    }

    /// Gets technical error details (for internal logging/debugging)
    pub fn technical_details(&self) -> String {
        format!("{:?}", self)
    }
}

/// Error severity levels for logging and alerting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
    Security,
}

impl ErrorSeverity {
    /// Gets the string representation for logging
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
            Self::Security => "security",
        }
    }
}

/// Error categories for metrics collection and analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    ServiceCommunication,
    ResourceExhaustion,
    DataValidation,
    Security,
    ServiceSpecific,
}

impl ErrorCategory {
    /// Gets the string representation for metrics
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ServiceCommunication => "service_communication",
            Self::ResourceExhaustion => "resource_exhaustion",
            Self::DataValidation => "data_validation",
            Self::Security => "security",
            Self::ServiceSpecific => "service_specific",
        }
    }
}

/// Helper functions for creating common error instances
impl RenderingServiceError {
    /// Creates a connection timeout error
    pub fn connection_timeout(timeout: Duration) -> Self {
        Self::ConnectionTimeout {
            timeout: timeout.as_millis() as u64,
        }
    }

    /// Creates a request timeout error
    pub fn request_timeout(timeout: Duration) -> Self {
        Self::RequestTimeout {
            timeout: timeout.as_millis() as u64,
        }
    }

    /// Creates a memory exhaustion error
    pub fn memory_exhaustion(used_mb: u64, limit_mb: u64) -> Self {
        Self::MemoryExhaustion { used_mb, limit_mb }
    }

    /// Creates a CPU timeout error
    pub fn cpu_timeout(elapsed: Duration, limit: Duration) -> Self {
        Self::CpuTimeout {
            elapsed_ms: elapsed.as_millis() as u64,
            limit_ms: limit.as_millis() as u64,
        }
    }

    /// Creates an invalid Mermaid syntax error
    pub fn invalid_mermaid(details: impl Into<String>, line: Option<u32>) -> Self {
        Self::InvalidMermaidSyntax {
            line,
            details: details.into(),
        }
    }

    /// Creates a diagram complexity error
    pub fn diagram_too_complex(reason: impl Into<String>, limit: u64, actual: u64) -> Self {
        Self::DiagramComplexityExceeded {
            reason: reason.into(),
            limit,
            actual,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_retryability() {
        // Retryable errors
        assert!(RenderingServiceError::ServiceUnavailable.is_retryable());
        assert!(RenderingServiceError::connection_timeout(Duration::from_secs(30)).is_retryable());
        assert!(RenderingServiceError::request_timeout(Duration::from_secs(60)).is_retryable());

        // Non-retryable errors
        assert!(!RenderingServiceError::AuthenticationFailure {
            reason: "invalid token".to_string()
        }
        .is_retryable());
        assert!(!RenderingServiceError::invalid_mermaid("syntax error", Some(5)).is_retryable());
        assert!(!RenderingServiceError::SecurityValidationFailed {
            reason: "malicious".to_string()
        }
        .is_retryable());
    }

    #[test]
    fn test_circuit_breaker_triggers() {
        // Should open circuit
        assert!(RenderingServiceError::ServiceUnavailable.should_open_circuit());
        assert!(
            RenderingServiceError::connection_timeout(Duration::from_secs(30))
                .should_open_circuit()
        );
        assert!(RenderingServiceError::memory_exhaustion(1024, 512).should_open_circuit());

        // Should not open circuit
        assert!(!RenderingServiceError::AuthenticationFailure {
            reason: "invalid".to_string()
        }
        .should_open_circuit());
        assert!(
            !RenderingServiceError::invalid_mermaid("syntax error", None).should_open_circuit()
        );
        assert!(!RenderingServiceError::RateLimitExceeded {
            retry_after_seconds: Some(60)
        }
        .should_open_circuit());
    }

    #[test]
    fn test_fallback_triggers() {
        // Should trigger fallback
        assert!(RenderingServiceError::ServiceUnavailable.should_trigger_fallback());
        assert!(
            RenderingServiceError::connection_timeout(Duration::from_secs(30))
                .should_trigger_fallback()
        );
        assert!(RenderingServiceError::memory_exhaustion(1024, 512).should_trigger_fallback());

        // Should not trigger fallback
        assert!(
            !RenderingServiceError::invalid_mermaid("syntax error", None).should_trigger_fallback()
        );
        assert!(!RenderingServiceError::SecurityValidationFailed {
            reason: "malicious".to_string()
        }
        .should_trigger_fallback());
    }

    #[test]
    fn test_error_severity() {
        assert_eq!(
            RenderingServiceError::ServiceUnavailable.severity(),
            ErrorSeverity::Critical
        );
        assert_eq!(
            RenderingServiceError::connection_timeout(Duration::from_secs(30)).severity(),
            ErrorSeverity::High
        );
        assert_eq!(
            RenderingServiceError::invalid_mermaid("syntax error", None).severity(),
            ErrorSeverity::Low
        );
        assert_eq!(
            RenderingServiceError::SecurityValidationFailed {
                reason: "test".to_string()
            }
            .severity(),
            ErrorSeverity::Security
        );
    }

    #[test]
    fn test_error_categories() {
        assert_eq!(
            RenderingServiceError::ServiceUnavailable.category(),
            ErrorCategory::ServiceCommunication
        );
        assert_eq!(
            RenderingServiceError::memory_exhaustion(1024, 512).category(),
            ErrorCategory::ResourceExhaustion
        );
        assert_eq!(
            RenderingServiceError::invalid_mermaid("test", None).category(),
            ErrorCategory::DataValidation
        );
        assert_eq!(
            RenderingServiceError::SecurityValidationFailed {
                reason: "test".to_string()
            }
            .category(),
            ErrorCategory::Security
        );
    }

    #[test]
    fn test_user_messages() {
        let error = RenderingServiceError::ServiceUnavailable;
        let user_msg = error.user_message();
        assert!(user_msg.contains("temporarily unavailable"));
        assert!(!user_msg.contains("debug") && !user_msg.contains("internal"));

        let syntax_error = RenderingServiceError::invalid_mermaid("missing semicolon", Some(10));
        let syntax_msg = syntax_error.user_message();
        assert!(syntax_msg.contains("line 10"));
        assert!(syntax_msg.contains("missing semicolon"));
    }

    #[test]
    fn test_helper_constructors() {
        let timeout_error = RenderingServiceError::connection_timeout(Duration::from_millis(5000));
        match timeout_error {
            RenderingServiceError::ConnectionTimeout { timeout } => assert_eq!(timeout, 5000),
            other => panic!("Expected ConnectionTimeout, got: {:?}", other),
        }

        let memory_error = RenderingServiceError::memory_exhaustion(1024, 512);
        match memory_error {
            RenderingServiceError::MemoryExhaustion { used_mb, limit_mb } => {
                assert_eq!(used_mb, 1024);
                assert_eq!(limit_mb, 512);
            }
            other => panic!("Expected MemoryExhaustion, got: {:?}", other),
        }
    }
}
