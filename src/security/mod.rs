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
pub mod middleware;
pub mod models;
pub mod rate_limiting;
pub mod secrets;
pub mod secure_config_loader;

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
pub use middleware::SecurityServices;
pub use rate_limiting::RateLimiter;
pub use secrets::{SecretStore, SecretStoreFactory, SecretRotationManager, RotationPolicy};
pub use secure_config_loader::{SecureConfigLoader, SecretStoreHealthStatus};

use crate::error::UveddiError;
use std::path::{Component, Path, PathBuf};

// Include integration tests in test builds
#[cfg(test)]
mod integration_tests;

/// Maximum number of files allowed per analysis
pub const MAX_FILES_PER_ANALYSIS: usize = 10000;

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
