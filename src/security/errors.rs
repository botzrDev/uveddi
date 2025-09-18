//! Security-specific error types for Uveddi RBAC system
//!
//! This module provides comprehensive error handling for security operations
//! including authentication, authorization, and audit logging.

use std::fmt;

/// Comprehensive security error types for the RBAC system
#[derive(Debug, thiserror::Error)]
pub enum SecurityError {
    // Authentication Errors
    /// Authentication failed with specific reason
    #[error("Authentication failed: {reason}")]
    AuthenticationFailed {
        /// Reason for authentication failure
        reason: String,
    },

    /// Invalid credentials were provided
    #[error("Invalid credentials provided")]
    InvalidCredentials,

    /// Authentication token has expired
    #[error("Token has expired")]
    TokenExpired,

    /// Token format or signature is invalid
    #[error("Invalid token format or signature")]
    InvalidToken,

    /// OIDC provider error
    #[error("OIDC provider error: {provider} - {error}")]
    OidcProviderError {
        /// Name of the OIDC provider
        provider: String,
        /// Error details
        error: String,
    },

    /// OAuth2 flow error
    #[error("OAuth2 flow error: {error}")]
    OAuth2Error {
        /// OAuth2 error details
        error: String,
    },

    #[error("OAuth provider error: {provider} - {error}")]
    /// OAuth provider authentication error with provider name and error details
    OAuthProviderError {
        /// Name of the OAuth provider that failed
        provider: String,
        /// Detailed error message from the provider
        error: String,
    },

    // Authorization Errors
    #[error("Authorization denied: insufficient permissions")]
    /// Access denied due to insufficient permissions
    AuthorizationDenied,

    #[error("Permission denied for resource '{resource}' action '{action}'")]
    /// Permission denied for a specific resource and action
    PermissionDenied {
        /// The resource that was accessed
        resource: String,
        /// The action that was attempted
        action: String,
    },

    #[error("Role '{role}' not found")]
    /// Role not found in the system
    RoleNotFound {
        /// Name or ID of the missing role
        role: String,
    },

    #[error("User '{user_id}' not found")]
    /// User not found in the system
    UserNotFound {
        /// ID of the missing user
        user_id: String,
    },

    #[error("Permission '{permission}' not found")]
    /// Permission not found in the system
    PermissionNotFound {
        /// Name or ID of the missing permission
        permission: String,
    },

    #[error("Invalid role assignment: {reason}")]
    /// Invalid role assignment with reason
    InvalidRoleAssignment {
        /// Reason why the role assignment is invalid
        reason: String,
    },

    // Rate Limiting Errors
    #[error("Rate limit exceeded for {identifier}: {current}/{limit} requests")]
    /// Rate limit exceeded for a client or resource
    RateLimitExceeded {
        /// Identifier for the rate-limited entity
        identifier: String,
        /// Current number of requests
        current: u32,
        /// Maximum allowed requests
        limit: u32,
    },

    #[error("Rate limiting service unavailable")]
    /// Rate limiting service is unavailable
    RateLimitingUnavailable,

    // Configuration Errors
    #[error("Security configuration error: {message}")]
    /// Security configuration error
    ConfigurationError {
        /// Configuration error message
        message: String,
    },

    #[error("Missing required configuration: {key}")]
    /// Missing required configuration key
    MissingConfiguration {
        /// The configuration key that is missing
        key: String,
    },

    #[error("Invalid configuration value for {key}: {value}")]
    /// Invalid configuration value for a key
    InvalidConfiguration {
        /// The configuration key with invalid value
        key: String,
        /// The invalid value that was provided
        value: String,
    },

    // Audit Logging Errors
    #[error("Audit logging failed: {reason}")]
    /// Audit logging operation failed
    AuditLogError {
        /// Reason for audit logging failure
        reason: String,
    },

    #[error("Audit log integrity check failed")]
    /// Audit log integrity verification failed
    AuditIntegrityError,

    #[error("Audit log storage error: {error}")]
    /// Audit log storage operation failed
    AuditStorageError {
        /// Storage error details
        error: String,
    },

    // Database Errors
    #[error("Database operation failed: {operation} - {error}")]
    /// Database operation failed
    DatabaseError {
        /// The database operation that failed
        operation: String,
        /// Error details from the database
        error: String,
    },

    #[error("Database connection failed: {error}")]
    /// Database connection failed
    DatabaseConnectionError {
        /// Connection error details
        error: String,
    },

    #[error("Database migration failed: {error}")]
    /// Database migration failed
    DatabaseMigrationError {
        /// Migration error details
        error: String,
    },

    // Cryptographic Errors
    #[error("Cryptographic operation failed: {operation} - {error}")]
    /// Cryptographic operation failed
    CryptographicError {
        /// The cryptographic operation that failed
        operation: String,
        /// Error details from the cryptographic library
        error: String,
    },

    #[error("Hash verification failed")]
    /// Hash verification failed during authentication or integrity check
    HashVerificationError,

    #[error("Key generation failed: {error}")]
    /// Cryptographic key generation failed
    KeyGenerationError {
        /// Key generation error details
        error: String,
    },

    // Session Management Errors
    #[error("Session not found: {session_id}")]
    /// Session not found in the session store
    SessionNotFound {
        /// ID of the missing session
        session_id: String,
    },

    #[error("Session expired: {session_id}")]
    /// Session has expired and is no longer valid
    SessionExpired {
        /// ID of the expired session
        session_id: String,
    },

    #[error("Session creation failed: {error}")]
    /// Failed to create a new session
    SessionCreationError {
        /// Session creation error details
        error: String,
    },

    // API Key Errors
    #[error("API key not found: {key_prefix}")]
    /// API key not found in the key store
    ApiKeyNotFound {
        /// Prefix of the missing API key for identification
        key_prefix: String,
    },

    #[error("API key expired: {key_prefix}")]
    /// API key has expired and is no longer valid
    ApiKeyExpired {
        /// Prefix of the expired API key for identification
        key_prefix: String,
    },

    #[error("API key creation failed: {error}")]
    /// Failed to create a new API key
    ApiKeyCreationError {
        /// API key creation error details
        error: String,
    },

    #[error("Invalid API key format")]
    /// API key format is invalid or corrupted
    InvalidApiKeyFormat,

    // Secret Management Errors
    #[error("Secret not found: {key}")]
    /// Secret not found in the secret store
    SecretNotFound {
        /// Key of the missing secret
        key: String,
    },

    #[error("Secret store unavailable: {store_type}")]
    /// Secret store service is unavailable
    SecretStoreUnavailable {
        /// Type of secret store that is unavailable
        store_type: String,
    },

    #[error("Secret operation failed: {operation} - {error}")]
    /// Secret management operation failed
    SecretOperationError {
        /// The secret operation that failed
        operation: String,
        /// Error details from the secret store
        error: String,
    },

    // Input Validation Errors
    #[error("Invalid input: {field} - {reason}")]
    /// Input validation failed for a specific field
    InvalidInput {
        /// The field that failed validation
        field: String,
        /// Reason why the input is invalid
        reason: String,
    },

    #[error("Input validation failed: {errors:?}")]
    /// Multiple input validation errors occurred
    ValidationError {
        /// List of validation error messages
        errors: Vec<String>,
    },

    #[error("Input length exceeds maximum allowed")]
    /// Input data exceeds maximum allowed length
    InputTooLong,

    #[error("SQL injection attempt detected")]
    /// Potential SQL injection attack detected in input
    SqlInjectionAttempt,

    // Path Security Errors
    #[error("Path traversal attempt detected")]
    /// Potential path traversal attack detected
    PathTraversalAttempt,

    #[error("Invalid path provided")]
    /// File system path is invalid or malformed
    InvalidPath,

    #[error("Invalid base path provided")]
    /// Base path for file operations is invalid
    InvalidBasePath,

    // Authorization Engine Errors
    #[error("Authorization engine error: {error}")]
    /// Authorization engine internal error
    AuthorizationEngineError {
        /// Authorization engine error details
        error: String,
    },

    #[error("Policy evaluation failed: {policy} - {error}")]
    /// Security policy evaluation failed
    PolicyEvaluationError {
        /// Name or identifier of the policy that failed
        policy: String,
        /// Error details from policy evaluation
        error: String,
    },

    #[error("Authorization context invalid: {reason}")]
    /// Authorization context is invalid or incomplete
    InvalidAuthContext {
        /// Reason why the authorization context is invalid
        reason: String,
    },

    // Network/External Service Errors
    #[error("External service error: {service} - {error}")]
    /// External service integration error
    ExternalServiceError {
        /// Name of the external service that failed
        service: String,
        /// Error details from the external service
        error: String,
    },

    #[error("Network connection failed: {endpoint} - {error}")]
    /// Network connection or communication error
    NetworkError {
        /// Network endpoint that failed
        endpoint: String,
        /// Network error details
        error: String,
    },

    #[error("HTTP request failed: {status} - {error}")]
    /// HTTP request failed with error status
    HttpError {
        /// HTTP status code returned
        status: u16,
        /// HTTP error details
        error: String,
    },

    // HTTP Client Security Errors
    #[error("HTTPS required for URL: {url}")]
    /// HTTPS is required but HTTP was used
    HttpsRequired {
        /// URL that requires HTTPS
        url: String,
    },

    #[error("HTTP client configuration error: {message}")]
    /// HTTP client configuration or setup error
    HttpClientError {
        /// HTTP client error message
        message: String,
    },

    #[error("HTTP request error: {message}")]
    /// HTTP request construction or sending error
    HttpRequestError {
        /// HTTP request error message
        message: String,
    },

    #[error("HTTP response validation failed: {message}")]
    /// HTTP response parsing or validation error
    HttpResponseError {
        /// HTTP response error message
        message: String,
    },

    // System Errors
    #[error("System resource unavailable: {resource}")]
    /// Required system resource is unavailable
    SystemResourceUnavailable {
        /// Name of the unavailable system resource
        resource: String,
    },

    #[error("Internal system error: {error}")]
    /// Internal system error occurred
    InternalError {
        /// Internal error details
        error: String,
    },

    #[error("Initialization failed: {component} - {error}")]
    /// System component initialization failed
    InitializationError {
        /// Name of the component that failed to initialize
        component: String,
        /// Initialization error details
        error: String,
    },
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
    /// High severity - immediate security concern
    High,
    /// Medium severity - moderate security risk
    Medium,
    /// Low severity - minor security issue
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

// Convert from various error types to SecurityError
// Note: sqlx support is not currently enabled as a feature
// Uncomment and add sqlx feature to Cargo.toml if needed
// #[cfg(feature = "sqlx")]
// impl From<sqlx::Error> for SecurityError {
//     fn from(error: sqlx::Error) -> Self {
//         SecurityError::DatabaseError {
//             operation: "database_operation".to_string(),
//             error: error.to_string(),
//         }
//     }
// }

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
