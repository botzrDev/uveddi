//! Security-specific error types for Uveddi RBAC system
//!
//! This module provides comprehensive error handling for security operations
//! including authentication, authorization, and audit logging.

use std::fmt;

/// Comprehensive security error types for the RBAC system
#[derive(Debug, thiserror::Error)]
pub enum SecurityError {
    // Authentication Errors
    #[error("Authentication failed: {reason}")]
    AuthenticationFailed { reason: String },

    #[error("Invalid credentials provided")]
    InvalidCredentials,

    #[error("Token has expired")]
    TokenExpired,

    #[error("Invalid token format or signature")]
    InvalidToken,

    #[error("OIDC provider error: {provider} - {error}")]
    OidcProviderError { provider: String, error: String },

    #[error("OAuth2 flow error: {error}")]
    OAuth2Error { error: String },

    #[error("OAuth provider error: {provider} - {error}")]
    OAuthProviderError { provider: String, error: String },

    // Authorization Errors
    #[error("Authorization denied: insufficient permissions")]
    AuthorizationDenied,

    #[error("Permission denied for resource '{resource}' action '{action}'")]
    PermissionDenied { resource: String, action: String },

    #[error("Role '{role}' not found")]
    RoleNotFound { role: String },

    #[error("User '{user_id}' not found")]
    UserNotFound { user_id: String },

    #[error("Permission '{permission}' not found")]
    PermissionNotFound { permission: String },

    #[error("Invalid role assignment: {reason}")]
    InvalidRoleAssignment { reason: String },

    // Rate Limiting Errors
    #[error("Rate limit exceeded for {identifier}: {current}/{limit} requests")]
    RateLimitExceeded {
        identifier: String,
        current: u32,
        limit: u32,
    },

    #[error("Rate limiting service unavailable")]
    RateLimitingUnavailable,

    // Configuration Errors
    #[error("Security configuration error: {message}")]
    ConfigurationError { message: String },

    #[error("Missing required configuration: {key}")]
    MissingConfiguration { key: String },

    #[error("Invalid configuration value for {key}: {value}")]
    InvalidConfiguration { key: String, value: String },

    // Audit Logging Errors
    #[error("Audit logging failed: {reason}")]
    AuditLogError { reason: String },

    #[error("Audit log integrity check failed")]
    AuditIntegrityError,

    #[error("Audit log storage error: {error}")]
    AuditStorageError { error: String },

    // Database Errors
    #[error("Database operation failed: {operation} - {error}")]
    DatabaseError { operation: String, error: String },

    #[error("Database connection failed: {error}")]
    DatabaseConnectionError { error: String },

    #[error("Database migration failed: {error}")]
    DatabaseMigrationError { error: String },

    // Cryptographic Errors
    #[error("Cryptographic operation failed: {operation} - {error}")]
    CryptographicError { operation: String, error: String },

    #[error("Hash verification failed")]
    HashVerificationError,

    #[error("Key generation failed: {error}")]
    KeyGenerationError { error: String },

    // Session Management Errors
    #[error("Session not found: {session_id}")]
    SessionNotFound { session_id: String },

    #[error("Session expired: {session_id}")]
    SessionExpired { session_id: String },

    #[error("Session creation failed: {error}")]
    SessionCreationError { error: String },

    // API Key Errors
    #[error("API key not found: {key_prefix}")]
    ApiKeyNotFound { key_prefix: String },

    #[error("API key expired: {key_prefix}")]
    ApiKeyExpired { key_prefix: String },

    #[error("API key creation failed: {error}")]
    ApiKeyCreationError { error: String },

    #[error("Invalid API key format")]
    InvalidApiKeyFormat,

    // Secret Management Errors
    #[error("Secret not found: {key}")]
    SecretNotFound { key: String },

    #[error("Secret store unavailable: {store_type}")]
    SecretStoreUnavailable { store_type: String },

    #[error("Secret operation failed: {operation} - {error}")]
    SecretOperationError { operation: String, error: String },

    // Input Validation Errors
    #[error("Invalid input: {field} - {reason}")]
    InvalidInput { field: String, reason: String },

    #[error("Input validation failed: {errors:?}")]
    ValidationError { errors: Vec<String> },

    #[error("Input length exceeds maximum allowed")]
    InputTooLong,

    #[error("SQL injection attempt detected")]
    SqlInjectionAttempt,

    // Path Security Errors
    #[error("Path traversal attempt detected")]
    PathTraversalAttempt,

    #[error("Invalid path provided")]
    InvalidPath,

    #[error("Invalid base path provided")]
    InvalidBasePath,

    // Authorization Engine Errors
    #[error("Authorization engine error: {error}")]
    AuthorizationEngineError { error: String },

    #[error("Policy evaluation failed: {policy} - {error}")]
    PolicyEvaluationError { policy: String, error: String },

    #[error("Authorization context invalid: {reason}")]
    InvalidAuthContext { reason: String },

    // Network/External Service Errors
    #[error("External service error: {service} - {error}")]
    ExternalServiceError { service: String, error: String },

    #[error("Network connection failed: {endpoint} - {error}")]
    NetworkError { endpoint: String, error: String },

    #[error("HTTP request failed: {status} - {error}")]
    HttpError { status: u16, error: String },

    // HTTP Client Security Errors
    #[error("HTTPS required for URL: {url}")]
    HttpsRequired { url: String },

    #[error("HTTP client configuration error: {message}")]
    HttpClientError { message: String },

    #[error("HTTP request error: {message}")]
    HttpRequestError { message: String },

    #[error("HTTP response validation failed: {message}")]
    HttpResponseError { message: String },

    // System Errors
    #[error("System resource unavailable: {resource}")]
    SystemResourceUnavailable { resource: String },

    #[error("Internal system error: {error}")]
    InternalError { error: String },

    #[error("Initialization failed: {component} - {error}")]
    InitializationError { component: String, error: String },
}

impl SecurityError {
    /// Create a new authentication failed error
    pub fn authentication_failed(reason: impl Into<String>) -> Self {
        Self::AuthenticationFailed {
            reason: reason.into(),
        }
    }

    /// Create a new authorization denied error
    pub fn authorization_denied() -> Self {
        Self::AuthorizationDenied
    }

    /// Create a new permission denied error
    pub fn permission_denied(resource: impl Into<String>, action: impl Into<String>) -> Self {
        Self::PermissionDenied {
            resource: resource.into(),
            action: action.into(),
        }
    }

    /// Create a new rate limit exceeded error
    pub fn rate_limit_exceeded(identifier: impl Into<String>, current: u32, limit: u32) -> Self {
        Self::RateLimitExceeded {
            identifier: identifier.into(),
            current,
            limit,
        }
    }

    /// Create a new configuration error
    pub fn configuration_error(message: impl Into<String>) -> Self {
        Self::ConfigurationError {
            message: message.into(),
        }
    }

    /// Create a new database error
    pub fn database_error(operation: impl Into<String>, error: impl Into<String>) -> Self {
        Self::DatabaseError {
            operation: operation.into(),
            error: error.into(),
        }
    }

    /// Create a new audit log error
    pub fn audit_log_error(reason: impl Into<String>) -> Self {
        Self::AuditLogError {
            reason: reason.into(),
        }
    }

    /// Check if this error is related to authentication
    pub fn is_authentication_error(&self) -> bool {
        matches!(
            self,
            SecurityError::AuthenticationFailed { .. }
                | SecurityError::InvalidCredentials
                | SecurityError::TokenExpired
                | SecurityError::InvalidToken
                | SecurityError::OidcProviderError { .. }
                | SecurityError::OAuth2Error { .. }
        )
    }

    /// Check if this error is related to authorization
    pub fn is_authorization_error(&self) -> bool {
        matches!(
            self,
            SecurityError::AuthorizationDenied
                | SecurityError::PermissionDenied { .. }
                | SecurityError::RoleNotFound { .. }
                | SecurityError::PermissionNotFound { .. }
                | SecurityError::InvalidRoleAssignment { .. }
        )
    }

    /// Check if this error is related to rate limiting
    pub fn is_rate_limit_error(&self) -> bool {
        matches!(
            self,
            SecurityError::RateLimitExceeded { .. } | SecurityError::RateLimitingUnavailable
        )
    }

    /// Check if this error should be logged as a security event
    pub fn should_audit_log(&self) -> bool {
        matches!(
            self,
            SecurityError::AuthenticationFailed { .. }
                | SecurityError::InvalidCredentials
                | SecurityError::AuthorizationDenied
                | SecurityError::PermissionDenied { .. }
                | SecurityError::RateLimitExceeded { .. }
                | SecurityError::AuditIntegrityError
                | SecurityError::InvalidApiKeyFormat
                | SecurityError::ValidationError { .. }
                | SecurityError::PathTraversalAttempt
                | SecurityError::InvalidPath
                | SecurityError::HttpsRequired { .. }
                | SecurityError::HttpClientError { .. }
        )
    }

    /// Get the error severity level
    pub fn severity(&self) -> SecurityErrorSeverity {
        match self {
            SecurityError::AuthenticationFailed { .. }
            | SecurityError::InvalidCredentials
            | SecurityError::AuthorizationDenied
            | SecurityError::PermissionDenied { .. }
            | SecurityError::AuditIntegrityError
            | SecurityError::PathTraversalAttempt
            | SecurityError::HttpsRequired { .. } => SecurityErrorSeverity::High,

            SecurityError::TokenExpired
            | SecurityError::InvalidToken
            | SecurityError::RateLimitExceeded { .. }
            | SecurityError::SessionExpired { .. }
            | SecurityError::ApiKeyExpired { .. }
            | SecurityError::InvalidPath
            | SecurityError::InvalidBasePath
            | SecurityError::HttpClientError { .. }
            | SecurityError::HttpRequestError { .. } => SecurityErrorSeverity::Medium,

            SecurityError::ConfigurationError { .. }
            | SecurityError::DatabaseError { .. }
            | SecurityError::NetworkError { .. }
            | SecurityError::HttpResponseError { .. } => SecurityErrorSeverity::Low,

            _ => SecurityErrorSeverity::Low,
        }
    }
}

/// Security error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityErrorSeverity {
    High,
    Medium,
    Low,
}

impl fmt::Display for SecurityErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SecurityErrorSeverity::High => write!(f, "HIGH"),
            SecurityErrorSeverity::Medium => write!(f, "MEDIUM"),
            SecurityErrorSeverity::Low => write!(f, "LOW"),
        }
    }
}

/// Result type for security operations
pub type SecurityResult<T> = Result<T, SecurityError>;

/// Convert from various error types to SecurityError
#[cfg(feature = "sqlx")]
impl From<sqlx::Error> for SecurityError {
    fn from(error: sqlx::Error) -> Self {
        SecurityError::DatabaseError {
            operation: "database_operation".to_string(),
            error: error.to_string(),
        }
    }
}

impl From<rusqlite::Error> for SecurityError {
    fn from(error: rusqlite::Error) -> Self {
        SecurityError::DatabaseError {
            operation: "sqlite_operation".to_string(),
            error: error.to_string(),
        }
    }
}

impl
    From<
        oauth2::RequestTokenError<
            reqwest::Error,
            oauth2::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>,
        >,
    > for SecurityError
{
    fn from(
        error: oauth2::RequestTokenError<
            reqwest::Error,
            oauth2::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>,
        >,
    ) -> Self {
        SecurityError::OAuth2Error {
            error: error.to_string(),
        }
    }
}

impl From<jsonwebtoken::errors::Error> for SecurityError {
    fn from(error: jsonwebtoken::errors::Error) -> Self {
        match error.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => SecurityError::TokenExpired,
            jsonwebtoken::errors::ErrorKind::InvalidToken => SecurityError::InvalidToken,
            jsonwebtoken::errors::ErrorKind::InvalidSignature => SecurityError::InvalidToken,
            _ => SecurityError::InvalidToken,
        }
    }
}

impl From<argon2::Error> for SecurityError {
    fn from(error: argon2::Error) -> Self {
        SecurityError::CryptographicError {
            operation: "argon2_hash".to_string(),
            error: error.to_string(),
        }
    }
}

impl From<config::ConfigError> for SecurityError {
    fn from(error: config::ConfigError) -> Self {
        SecurityError::ConfigurationError {
            message: error.to_string(),
        }
    }
}

impl From<reqwest::Error> for SecurityError {
    fn from(error: reqwest::Error) -> Self {
        SecurityError::NetworkError {
            endpoint: error
                .url()
                .map(|u| u.to_string())
                .unwrap_or_else(|| "unknown".to_string()),
            error: error.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_categorization() {
        let auth_error = SecurityError::authentication_failed("invalid password");
        assert!(auth_error.is_authentication_error());
        assert!(!auth_error.is_authorization_error());
        assert!(!auth_error.is_rate_limit_error());

        let authz_error = SecurityError::authorization_denied();
        assert!(!authz_error.is_authentication_error());
        assert!(authz_error.is_authorization_error());
        assert!(!authz_error.is_rate_limit_error());

        let rate_limit_error = SecurityError::rate_limit_exceeded("user123", 100, 50);
        assert!(!rate_limit_error.is_authentication_error());
        assert!(!rate_limit_error.is_authorization_error());
        assert!(rate_limit_error.is_rate_limit_error());
    }

    #[test]
    fn test_error_severity() {
        let high_error = SecurityError::authentication_failed("test");
        assert_eq!(high_error.severity(), SecurityErrorSeverity::High);

        let medium_error = SecurityError::TokenExpired;
        assert_eq!(medium_error.severity(), SecurityErrorSeverity::Medium);

        let low_error = SecurityError::configuration_error("test");
        assert_eq!(low_error.severity(), SecurityErrorSeverity::Low);
    }

    #[test]
    fn test_should_audit_log() {
        let auth_error = SecurityError::authentication_failed("test");
        assert!(auth_error.should_audit_log());

        let config_error = SecurityError::configuration_error("test");
        assert!(!config_error.should_audit_log());

        let permission_error = SecurityError::permission_denied("projects", "read");
        assert!(permission_error.should_audit_log());
    }
}
