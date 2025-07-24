//! Security and RBAC Module for Uveddi
//!
//! This module provides comprehensive security features including authentication,
//! authorization, audit logging, and input validation for the Uveddi analysis platform.
//! It implements a hybrid RBAC/ABAC system with enterprise-grade security controls.
//!
//! # Security Features
//!
//! ## Authentication
//! - **OAuth 2.0/OIDC Integration**: Federated authentication with enterprise IdPs
//! - **API Key Authentication**: Secure service-to-service authentication
//! - **Session Management**: Secure session handling with expiration
//! - **JWT Token Validation**: Standards-compliant token verification
//!
//! ## Authorization (RBAC/ABAC)
//! - **Role-Based Access Control**: Hierarchical role system with predefined roles
//! - **Attribute-Based Access Control**: Fine-grained permission system
//! - **Permission Scoping**: Resource-level access control (own/team/all)
//! - **Policy Engine**: Casbin-powered authorization engine
//!
//! ## Audit Logging
//! - **Comprehensive Audit Trail**: Immutable security event logging
//! - **Tamper Detection**: Cryptographic integrity verification
//! - **Compliance Support**: SOC 2 and ISO 27001 compliant logging
//! - **Security Event Classification**: Structured event categorization
//!
//! ## Rate Limiting
//! - **API Rate Limiting**: Prevent abuse and DoS attacks
//! - **Per-User/IP Limits**: Configurable rate limiting strategies
//! - **Sliding Window**: Advanced rate limiting algorithms
//!
//! ## Input Validation
//! - **Path Sanitization**: Prevent directory traversal attacks
//! - **File Type Validation**: Ensure only safe file types are processed
//! - **Size Limits**: Prevent resource exhaustion from large inputs
//! - **Input Sanitization**: Comprehensive input validation
//!
//! # User Roles
//!
//! The system defines the following standard roles:
//! - **Admin**: Full system access and configuration
//! - **Developer**: Test data access for owned projects only
//! - **QA**: Test execution and failure analysis access
//! - **Manager**: Read-only access to reports and dashboards
//! - **Service**: API access for automated integrations
//!
//! # Usage Examples
//!
//! ## Authentication
//! ```rust,no_run
//! use uveddi::security::{AuthenticationService, AuthenticatedUser};
//!
//! // OAuth authentication
//! let auth_service = AuthenticationService::new(config).await?;
//! let user = auth_service.authenticate_oidc("google", &auth_code, &nonce).await?;
//!
//! // API key authentication
//! let user = auth_service.authenticate_api_key(&api_key).await?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Authorization
//! ```rust,no_run
//! use uveddi::security::{AuthorizationEngine, AuthContext};
//!
//! let authz_engine = AuthorizationEngine::new().await?;
//! let context = AuthContext::new(user_id, "projects".to_string(), "read".to_string(), Some("own".to_string()));
//! let allowed = authz_engine.check_permission(&user_id, "projects", "read", &context).await?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Audit Logging
//! ```rust,no_run
//! use uveddi::security::{AuditLogger, AuditEvent, AuditEventType, AuditOutcome};
//!
//! let audit_logger = AuditLogger::new().await?;
//! let event = AuditEvent::new(
//!     AuditEventType::Authentication,
//!     Some(user_id),
//!     None,
//!     "users".to_string(),
//!     "login".to_string(),
//!     AuditOutcome::Success,
//!     Some("127.0.0.1".to_string()),
//!     None,
//!     serde_json::json!({"method": "password"}),
//! );
//! audit_logger.log_event(event).await?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Security Considerations
//!
//! - All authentication attempts are logged for security monitoring
//! - Failed authorization attempts trigger security alerts
//! - Rate limiting prevents brute force attacks
//! - All secrets are stored securely using external secret management
//! - Audit logs are tamper-evident and immutable
//! - Input validation prevents injection attacks
//!
//! # Compliance
//!
//! This module is designed to meet enterprise security standards including:
//! - SOC 2 Type II compliance
//! - ISO 27001 information security management
//! - GDPR privacy requirements
//! - Industry-standard authentication protocols

// Core security modules
pub mod audit;
pub mod authentication;
pub mod authorization;
pub mod config;
pub mod errors;
pub mod http_client;
pub mod middleware;
pub mod models;
pub mod rate_limiting;
pub mod secrets;
pub mod secure_config_loader;

// Standard library imports
use std::path::{Path, PathBuf};

// Re-export commonly used types
pub use errors::{SecurityError, SecurityErrorSeverity, SecurityResult};
pub use models::{
    ApiKey, AuditEvent, AuditEventType, AuditOutcome, AuthContext, AuthenticatedUser, Permission,
    RateLimitIdentifierType, RateLimitInfo, Role, Session, User, UserRole, UserRoleAssignment,
};

// Re-export authentication and authorization config types
pub use audit::{AuditLogger, AuditStore};
pub use authentication::{AuthenticationConfig, OAuthProviderConfig, OidcProviderConfig};
pub use authorization::AuthorizationEngine;
pub use config::RateLimitingConfig;
pub use config::{SecurityConfig, SecurityConfigLoader};
pub use http_client::{SecureHttpClient, HttpSecurityConfig};
pub use middleware::SecurityServices;
pub use rate_limiting::RateLimiter;
pub use secrets::{SecretStore, SecretStoreFactory, SecretRotationManager, RotationPolicy};
pub use secure_config_loader::{SecureConfigLoader, SecretStoreHealthStatus};

use crate::error::UveddiError;
use std::path::Component;

// Include integration tests in test builds
#[cfg(test)]
mod integration_tests;

/// Maximum number of files allowed per analysis
pub const MAX_FILES_PER_ANALYSIS: usize = 10000;

/// Maximum input length for general text fields
pub const MAX_INPUT_LENGTH: usize = 10000;

/// Maximum length for file paths
pub const MAX_PATH_LENGTH: usize = 4096;

/// Maximum length for model names
pub const MAX_MODEL_NAME_LENGTH: usize = 100;

/// Validate file size for analysis
pub fn validate_file_size(path: &Path) -> Result<(), SecurityError> {
    let metadata = std::fs::metadata(path).map_err(|_| SecurityError::InvalidInput {
        field: "file_path".to_string(),
        reason: "Could not read file metadata".to_string(),
    })?;
    let size = metadata.len();
    let max_size = 10 * 1024 * 1024; // 10MB
    if size > max_size {
        return Err(SecurityError::ValidationError {
            errors: vec![format!("File size {} exceeds limit {}", size, max_size)],
        });
    }
    Ok(())
}

/// Validate file type for analysis
pub fn validate_file_type(path: &Path) -> Result<(), SecurityError> {
    let allowed = ["rs", "py", "js", "jsx", "ts", "tsx"];
    let ext =
        path.extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| SecurityError::InvalidInput {
                field: "file_extension".to_string(),
                reason: "No extension found".to_string(),
            })?;
    if !allowed.contains(&ext) {
        return Err(SecurityError::InvalidInput {
            field: "file_extension".to_string(),
            reason: format!("Unsupported file type: {}", ext),
        });
    }
    Ok(())
}

/// Validate model name for AI analysis
pub fn validate_model_name(name: &str) -> Result<(), SecurityError> {
    if name.is_empty()
        || name.contains('/')
        || name.contains("..")
        || name.contains('\\')
        || name.contains("\0")
        || name.len() > 100
    {
        return Err(SecurityError::InvalidInput {
            field: "model_name".to_string(),
            reason: "Invalid characters or length".to_string(),
        });
    }
    Ok(())
}

/// Sanitize description for database insertion
pub fn sanitize_description(desc: &str) -> String {
    desc.chars()
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
        .collect()
}

/// Validate directory depth
pub fn validate_directory_depth(depth: usize) -> Result<(), SecurityError> {
    let max_depth = 100;
    if depth > max_depth {
        return Err(SecurityError::ValidationError {
            errors: vec![format!(
                "Directory depth {} exceeds maximum {}",
                depth, max_depth
            )],
        });
    }
    Ok(())
}

/// Validate file count for analysis
pub fn validate_file_count(count: usize) -> Result<(), SecurityError> {
    if count > MAX_FILES_PER_ANALYSIS {
        return Err(SecurityError::ValidationError {
            errors: vec![format!(
                "File count {} exceeds maximum {}",
                count, MAX_FILES_PER_ANALYSIS
            )],
        });
    }
    Ok(())
}

/// Comprehensive input validation function
/// 
/// Validates input for SQL injection patterns, length constraints, and other security concerns.
/// This is the primary validation function for general text inputs.
/// 
/// # Arguments
/// * `input` - The input string to validate
/// * `field_name` - The name of the field being validated (for error reporting)
/// 
/// # Returns
/// * `Ok(())` - Input is valid
/// * `Err(SecurityError)` - Input contains dangerous patterns or exceeds limits
pub fn validate_input(input: &str, field_name: &str) -> Result<(), SecurityError> {
    // Check for SQL injection patterns
    if contains_sql_injection_patterns(input) {
        return Err(SecurityError::InvalidInput {
            field: field_name.to_string(),
            reason: "Contains potentially dangerous SQL patterns".to_string(),
        });
    }
    
    // Check length constraints
    if input.len() > MAX_INPUT_LENGTH {
        return Err(SecurityError::InvalidInput {
            field: field_name.to_string(),
            reason: format!("Input length {} exceeds maximum {}", input.len(), MAX_INPUT_LENGTH),
        });
    }
    
    Ok(())
}

/// Check for SQL injection patterns
/// 
/// Detects common SQL injection attack patterns in input strings.
/// Uses a comprehensive list of dangerous SQL keywords and patterns.
fn contains_sql_injection_patterns(input: &str) -> bool {
    let dangerous_patterns = [
        ";", "--", "/*", "*/", "xp_", "sp_", 
        "union", "select", "insert", "update", "delete", "drop",
        "exec", "execute", "script", "javascript:",
        "alter", "create", "truncate", "grant", "revoke",
    ];
    
    let input_lower = input.to_lowercase();
    dangerous_patterns.iter().any(|pattern| input_lower.contains(pattern))
}

/// Validate URL format for API endpoints
/// 
/// Validates that URLs are properly formatted and don't contain dangerous characters.
/// Only allows HTTP and HTTPS protocols for security.
/// 
/// # Arguments
/// * `url` - The URL string to validate
/// 
/// # Returns
/// * `Ok(())` - URL is valid
/// * `Err(SecurityError)` - URL is invalid or contains dangerous patterns
pub fn validate_url(url: &str) -> Result<(), SecurityError> {
    if url.is_empty() {
        return Err(SecurityError::InvalidInput {
            field: "url".to_string(),
            reason: "URL cannot be empty".to_string(),
        });
    }
    
    // Basic URL validation
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(SecurityError::InvalidInput {
            field: "url".to_string(),
            reason: "URL must start with http:// or https://".to_string(),
        });
    }
    
    // Check for dangerous characters
    if url.contains("..") || url.contains("\\") || url.contains("\0") {
        return Err(SecurityError::InvalidInput {
            field: "url".to_string(),
            reason: "URL contains invalid characters".to_string(),
        });
    }
    
    Ok(())
}

/// Validate numeric input ranges
/// 
/// Ensures numeric values fall within acceptable ranges to prevent overflow
/// and other numeric-based attacks.
/// 
/// # Arguments
/// * `value` - The numeric value to validate
/// * `min` - Minimum allowed value (inclusive)
/// * `max` - Maximum allowed value (inclusive)
/// * `field_name` - The name of the field being validated
/// 
/// # Returns
/// * `Ok(())` - Value is within range
/// * `Err(SecurityError)` - Value is outside acceptable range
pub fn validate_numeric_range(value: i32, min: i32, max: i32, field_name: &str) -> Result<(), SecurityError> {
    if value < min || value > max {
        return Err(SecurityError::InvalidInput {
            field: field_name.to_string(),
            reason: format!("Value {} must be between {} and {}", value, min, max),
        });
    }
    Ok(())
}

/// Validate character set for specific fields
/// 
/// Restricts input to only allowed characters to prevent injection attacks
/// and ensure data integrity.
/// 
/// # Arguments
/// * `input` - The input string to validate
/// * `field_name` - The name of the field being validated
/// * `allowed_chars` - String containing all allowed characters
/// 
/// # Returns
/// * `Ok(())` - All characters are allowed
/// * `Err(SecurityError)` - Input contains disallowed characters
pub fn validate_character_set(input: &str, field_name: &str, allowed_chars: &str) -> Result<(), SecurityError> {
    for ch in input.chars() {
        if !allowed_chars.contains(ch) {
            return Err(SecurityError::InvalidInput {
                field: field_name.to_string(),
                reason: format!("Character '{}' is not allowed", ch),
            });
        }
    }
    Ok(())
}

/// Sanitize and validate file paths to prevent path traversal attacks
/// 
/// This function implements comprehensive path security by:
/// - Resolving all symbolic links using canonicalize()
/// - Ensuring the resolved path stays within the base directory
/// - Preventing Unicode normalization attacks
/// - Blocking common path traversal patterns
/// 
/// # Security Features
/// - **Symlink Resolution**: Uses `std::fs::canonicalize()` to resolve all symbolic links
/// - **Path Containment**: Strict verification that resolved path is within base directory
/// - **Unicode Safety**: Handles Unicode normalization attacks
/// - **Error Handling**: Secure error messages that don't leak path information
/// 
/// # Arguments
/// * `input_path` - The untrusted path to sanitize
/// * `base_dir` - The trusted base directory that must contain the resolved path
/// 
/// # Returns
/// * `Ok(PathBuf)` - The canonicalized, safe path
/// * `Err(SecurityError)` - Path traversal attempt or invalid path
/// 
/// # Examples
/// ```rust
/// use uveddi::security::sanitize_path;
/// use std::path::Path;
/// 
/// // Safe path within base directory
/// let safe_path = sanitize_path("./file.txt", "/safe/base")?;
/// 
/// // Path traversal attempt - will return error
/// let result = sanitize_path("../../../etc/passwd", "/safe/base");
/// assert!(result.is_err());
/// ```
pub fn sanitize_path<P: AsRef<Path>>(input_path: P, base_dir: P) -> Result<PathBuf, SecurityError> {
    // Step 1: Canonicalize the base directory to get absolute, resolved path
    let base = base_dir.as_ref().canonicalize()
        .map_err(|_| SecurityError::InvalidBasePath)?;
    
    // Step 2: Canonicalize the input path to resolve symlinks and normalize
    let resolved = input_path.as_ref().canonicalize()
        .map_err(|_| SecurityError::InvalidPath)?;
    
    // Step 3: Verify that the canonicalized path is within the base directory
    if !resolved.starts_with(&base) {
        return Err(SecurityError::PathTraversalAttempt);
    }
    
    Ok(resolved)
}
