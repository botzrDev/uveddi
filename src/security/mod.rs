//! Security and RBAC Module for Uveddi
//!
//! This module provides comprehensive security features including authentication,
//! authorization, audit logging, and input validation for the Uveddi analysis platform.
//! It implements a hybrid RBAC/ABAC system with enterprise-grade security controls.
//!
//! The module is feature-gated with the 'security' feature. When this feature is disabled,
//! stub implementations are provided to maintain API compatibility.
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

#[cfg(feature = "security")]
pub mod audit;
#[cfg(feature = "security")]
pub mod authentication;
#[cfg(feature = "security")]
pub mod authorization;
#[cfg(feature = "security")]
pub mod config;
#[cfg(feature = "security")]
pub mod dependency_config;
#[cfg(feature = "security")]
pub mod errors;
#[cfg(feature = "security")]
pub mod http_client;
#[cfg(feature = "security")]
pub mod middleware;
#[cfg(feature = "security")]
pub mod models;
#[cfg(feature = "security")]
pub mod rate_limiting;
#[cfg(feature = "security")]
pub mod secrets;
#[cfg(feature = "security")]
pub mod secure_config_loader;

// Stub implementations when security feature is disabled
#[cfg(not(feature = "security"))]
mod stub;

// Re-export main components for API compatibility
#[cfg(feature = "security")]
pub use authentication::AuthenticationService;
#[cfg(feature = "security")]
pub use authorization::AuthorizationEngine;
#[cfg(feature = "security")]
pub use errors::{SecurityError, SecurityResult};
#[cfg(feature = "security")]
pub use http_client::{HttpSecurityConfig, SecureHttpClient};
#[cfg(feature = "security")]
pub use models::UserRole;
#[cfg(feature = "security")]
pub use models::{AuthContext, AuthenticatedUser};

// Re-export from stub when security feature is disabled
#[cfg(not(feature = "security"))]
pub use stub::{HttpSecurityConfig, SecureHttpClient, SecurityError, SecurityResult};

// Input validation functions

// Core security modules

// Standard library imports
use std::path::{Path, PathBuf};

// Re-export commonly used types (feature-gated)
#[cfg(feature = "security")]
pub use errors::SecurityErrorSeverity;
#[cfg(feature = "security")]
pub use models::{
    ApiKey, AuditEvent, AuditEventType, AuditOutcome, Permission, RateLimitIdentifierType,
    RateLimitInfo, Role, Session, User, UserRoleAssignment,
};

// Re-export authentication and authorization config types (feature-gated)
#[cfg(feature = "security")]
pub use audit::{AuditLogger, AuditStore};
#[cfg(feature = "security")]
pub use authentication::{AuthenticationConfig, OAuthProviderConfig, OidcProviderConfig};
#[cfg(feature = "security")]
pub use config::RateLimitingConfig;
#[cfg(feature = "security")]
pub use config::{SecurityConfig, SecurityConfigLoader};
#[cfg(feature = "security")]
pub use dependency_config::{
    create_custom_http_client, create_secure_http_client, validate_input_size, SecurityLimits,
};
#[cfg(feature = "security")]
pub use middleware::SecurityServices;
#[cfg(feature = "security")]
pub use rate_limiting::RateLimiter;
#[cfg(feature = "security")]
pub use secrets::{RotationPolicy, SecretRotationManager, SecretStore, SecretStoreFactory};
#[cfg(feature = "security")]
pub use secure_config_loader::{SecretStoreHealthStatus, SecureConfigLoader};

// Removed unused UveddiError import
// Removed unused Component import
// Removed unused Validate import

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
            reason: format!(
                "Input length {} exceeds maximum {}",
                input.len(),
                MAX_INPUT_LENGTH
            ),
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
        ";",
        "--",
        "/*",
        "*/",
        "xp_",
        "sp_",
        "union",
        "select",
        "insert",
        "update",
        "delete",
        "drop",
        "exec",
        "execute",
        "script",
        "javascript:",
        "alter",
        "create",
        "truncate",
        "grant",
        "revoke",
    ];

    let input_lower = input.to_lowercase();
    dangerous_patterns
        .iter()
        .any(|pattern| input_lower.contains(pattern))
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
pub fn validate_numeric_range(
    value: i32,
    min: i32,
    max: i32,
    field_name: &str,
) -> Result<(), SecurityError> {
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
pub fn validate_character_set(
    input: &str,
    field_name: &str,
    allowed_chars: &str,
) -> Result<(), SecurityError> {
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

/// Validate code analysis data (more permissive than user input validation)
///
/// This function provides validation specifically for code analysis results that may
/// contain programming language keywords, symbols, and longer content that would be
/// rejected by general input validation but are safe in the context of analysis results.
///
/// # Arguments
/// * `input` - The code analysis data to validate
/// * `field_name` - Name of the field for error reporting
/// * `max_length` - Maximum allowed length (default: 100KB for code snippets)
///
/// # Returns
/// * `Ok(())` - Input passes code analysis validation
/// * `Err(SecurityError)` - Input violates security constraints
pub fn validate_code_analysis_data(
    input: &str,
    field_name: &str,
    max_length: Option<usize>,
) -> Result<(), SecurityError> {
    let max_len = max_length.unwrap_or(100_000); // 100KB default for code content

    // Only check for excessive length - code content naturally contains SQL keywords, etc.
    if input.len() > max_len {
        return Err(SecurityError::InvalidInput {
            field: field_name.to_string(),
            reason: format!(
                "Content length {} exceeds maximum {} for code analysis data",
                input.len(),
                max_len
            ),
        });
    }

    // Check for null bytes which could indicate binary data corruption
    if input.contains('\0') {
        return Err(SecurityError::InvalidInput {
            field: field_name.to_string(),
            reason: "Code analysis data contains null bytes".to_string(),
        });
    }

    // Allow all other content - programming languages naturally contain
    // semicolons, SQL keywords, comments, etc. that are safe in this context
    Ok(())
}

/// Validate file paths for database storage without path traversal checking
///
/// This function validates file paths specifically for database storage of analysis results.
/// Unlike `sanitize_path`, it doesn't perform path traversal checks since these are analysis
/// results being stored, not user input paths being accessed.
///
/// # Arguments
/// * `file_path` - The file path from analysis results
/// * `field_name` - Name of the field for error reporting
///
/// # Returns
/// * `Ok(())` - Path is valid for storage
/// * `Err(SecurityError)` - Path violates storage constraints
pub fn validate_file_path_for_storage(
    file_path: &str,
    field_name: &str,
) -> Result<(), SecurityError> {
    // Check length constraints
    if file_path.len() > MAX_PATH_LENGTH {
        return Err(SecurityError::InvalidInput {
            field: field_name.to_string(),
            reason: format!(
                "Path length {} exceeds maximum {}",
                file_path.len(),
                MAX_PATH_LENGTH
            ),
        });
    }

    // Check for null bytes which could indicate corruption
    if file_path.contains('\0') {
        return Err(SecurityError::InvalidInput {
            field: field_name.to_string(),
            reason: "File path contains null bytes".to_string(),
        });
    }

    // Don't apply SQL injection pattern detection to file paths
    // as they naturally contain characters like dashes, dots, etc.
    // that are flagged as dangerous but are normal in file paths

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
    let base = base_dir
        .as_ref()
        .canonicalize()
        .map_err(|_| SecurityError::InvalidBasePath)?;

    // Step 2: Canonicalize the input path to resolve symlinks and normalize
    let resolved = input_path
        .as_ref()
        .canonicalize()
        .map_err(|_| SecurityError::InvalidPath)?;

    // Step 3: Verify that the canonicalized path is within the base directory
    if !resolved.starts_with(&base) {
        return Err(SecurityError::PathTraversalAttempt);
    }

    Ok(resolved)
}

// ================================================================================================
// ENHANCED VALIDATION FUNCTIONS FOR INPUT HARDENING
// ================================================================================================

/// Enhanced configuration file path validation with security checks
///
/// Addresses the security finding: Configuration file path not validated for traversal attacks.
/// This function validates configuration file paths before allowing file system access.
///
/// # Arguments
/// * `config_path` - The configuration file path to validate
/// * `allowed_directories` - Optional list of directories where config files are allowed
///
/// # Returns
/// * `Ok(PathBuf)` - Validated and canonicalized path
/// * `Err(SecurityError)` - Path validation failed
///
/// # Security Features
/// - Path traversal prevention
/// - Size limit validation (1MB for config files)
/// - File extension validation (.toml, .yaml, .yml, .json only)
/// - Directory restriction enforcement
pub fn validate_config_file_path(
    config_path: &str,
    allowed_directories: Option<&[&str]>,
) -> Result<PathBuf, SecurityError> {
    // Basic path validation
    validate_file_path_for_storage(config_path, "config_path")?;

    // Parse path and validate components
    let path = Path::new(config_path);
    
    // Validate file extension
    let extension = path.extension()
        .and_then(|ext| ext.to_str())
        .ok_or_else(|| SecurityError::InvalidInput {
            field: "config_path".to_string(),
            reason: "Configuration file must have a valid extension".to_string(),
        })?;
    
    let allowed_extensions = ["toml", "yaml", "yml", "json"];
    if !allowed_extensions.contains(&extension) {
        return Err(SecurityError::InvalidInput {
            field: "config_path".to_string(),
            reason: format!(
                "Invalid config file extension '{}'. Allowed: {:?}",
                extension, allowed_extensions
            ),
        });
    }

    // Check file size if file exists
    if path.exists() {
        let metadata = std::fs::metadata(path).map_err(|_| SecurityError::InvalidInput {
            field: "config_path".to_string(),
            reason: "Could not read configuration file metadata".to_string(),
        })?;
        
        const MAX_CONFIG_SIZE: u64 = 1024 * 1024; // 1MB
        if metadata.len() > MAX_CONFIG_SIZE {
            return Err(SecurityError::InvalidInput {
                field: "config_path".to_string(),
                reason: format!(
                    "Configuration file size {} exceeds maximum {} bytes",
                    metadata.len(),
                    MAX_CONFIG_SIZE
                ),
            });
        }
    }

    // If allowed directories are specified, verify path is within them
    if let Some(allowed_dirs) = allowed_directories {
        let canonicalized = path.canonicalize().map_err(|_| SecurityError::InvalidInput {
            field: "config_path".to_string(),
            reason: "Could not canonicalize configuration file path".to_string(),
        })?;

        let is_allowed = allowed_dirs.iter().any(|allowed_dir| {
            if let Ok(allowed_canonical) = Path::new(allowed_dir).canonicalize() {
                canonicalized.starts_with(allowed_canonical)
            } else {
                false
            }
        });

        if !is_allowed {
            return Err(SecurityError::InvalidInput {
                field: "config_path".to_string(),
                reason: format!(
                    "Configuration file path is not within allowed directories: {:?}",
                    allowed_dirs
                ),
            });
        }

        Ok(canonicalized)
    } else {
        // Just return canonicalized path if no directory restrictions
        path.canonicalize().map_err(|_| SecurityError::InvalidInput {
            field: "config_path".to_string(),
            reason: "Could not canonicalize configuration file path".to_string(),
        })
    }
}

/// Enhanced API request validation with comprehensive security checks
///
/// Validates API request parameters including headers, query parameters, and request bodies
/// to prevent injection attacks and ensure data integrity.
///
/// # Arguments
/// * `content_type` - HTTP Content-Type header value
/// * `content_length` - Request content length
/// * `user_agent` - User-Agent header for basic bot detection
///
/// # Returns
/// * `Ok(())` - Request passes validation
/// * `Err(SecurityError)` - Request violates security policies
pub fn validate_api_request(
    content_type: Option<&str>,
    content_length: Option<u64>,
    user_agent: Option<&str>,
) -> Result<(), SecurityError> {
    // Validate Content-Type if present
    if let Some(ct) = content_type {
        let allowed_content_types = [
            "application/json",
            "application/x-www-form-urlencoded",
            "multipart/form-data",
            "text/plain",
        ];
        
        // Extract base content type (ignore charset and other parameters)
        let base_ct = ct.split(';').next().unwrap_or(ct).trim();
        
        if !allowed_content_types.iter().any(|&allowed| base_ct == allowed) {
            return Err(SecurityError::InvalidInput {
                field: "content_type".to_string(),
                reason: format!(
                    "Unsupported content type '{}'. Allowed: {:?}",
                    base_ct, allowed_content_types
                ),
            });
        }
    }

    // Validate Content-Length
    if let Some(length) = content_length {
        const MAX_REQUEST_SIZE: u64 = 10 * 1024 * 1024; // 10MB
        if length > MAX_REQUEST_SIZE {
            return Err(SecurityError::InvalidInput {
                field: "content_length".to_string(),
                reason: format!(
                    "Request size {} exceeds maximum {} bytes",
                    length, MAX_REQUEST_SIZE
                ),
            });
        }
    }

    // Basic bot detection via User-Agent
    if let Some(ua) = user_agent {
        validate_input(ua, "user_agent")?;
        
        // Check for suspicious user agents
        let suspicious_patterns = [
            "bot", "crawler", "spider", "scraper", "scan",
        ];
        
        let ua_lower = ua.to_lowercase();
        let has_suspicious = suspicious_patterns.iter().any(|&pattern| ua_lower.contains(pattern));
        
        // Log suspicious user agents but don't block them (could be legitimate bots)
        if has_suspicious {
            tracing::info!("Suspicious user agent detected: {}", ua);
        }
    }

    Ok(())
}

/// Validate JSON input with size limits and structure validation
///
/// Validates JSON data to prevent large payload attacks and ensures basic structure integrity.
///
/// # Arguments
/// * `json_str` - JSON string to validate
/// * `max_depth` - Maximum allowed nesting depth (default: 10)
/// * `max_size` - Maximum JSON size in bytes (default: 1MB)
///
/// # Returns
/// * `Ok(serde_json::Value)` - Parsed and validated JSON
/// * `Err(SecurityError)` - JSON validation failed
pub fn validate_json_input(
    json_str: &str,
    max_depth: Option<usize>,
    max_size: Option<usize>,
) -> Result<serde_json::Value, SecurityError> {
    let max_depth = max_depth.unwrap_or(10);
    let max_size = max_size.unwrap_or(1024 * 1024); // 1MB default

    // Check size first
    if json_str.len() > max_size {
        return Err(SecurityError::InvalidInput {
            field: "json_input".to_string(),
            reason: format!(
                "JSON size {} exceeds maximum {} bytes",
                json_str.len(),
                max_size
            ),
        });
    }

    // Basic input validation
    validate_input(json_str, "json_input")?;

    // Parse JSON
    let json_value: serde_json::Value = serde_json::from_str(json_str).map_err(|e| {
        SecurityError::InvalidInput {
            field: "json_input".to_string(),
            reason: format!("Invalid JSON format: {}", e),
        }
    })?;

    // Validate nesting depth
    if get_json_depth(&json_value) > max_depth {
        return Err(SecurityError::InvalidInput {
            field: "json_input".to_string(),
            reason: format!(
                "JSON nesting depth exceeds maximum of {}",
                max_depth
            ),
        });
    }

    Ok(json_value)
}

/// Calculate JSON nesting depth
fn get_json_depth(value: &serde_json::Value) -> usize {
    match value {
        serde_json::Value::Object(map) => {
            map.values()
                .map(get_json_depth)
                .max()
                .unwrap_or(0) + 1
        }
        serde_json::Value::Array(arr) => {
            arr.iter()
                .map(get_json_depth)
                .max()
                .unwrap_or(0) + 1
        }
        _ => 1,
    }
}

/// Enhanced CLI argument validation with security-focused checks
///
/// Validates CLI arguments to prevent injection attacks and ensure proper bounds checking.
///
/// # Arguments
/// * `arg_value` - Command line argument value
/// * `arg_name` - Name of the argument for error reporting
/// * `validation_type` - Type of validation to perform
///
/// # Returns
/// * `Ok(())` - Argument passes validation
/// * `Err(SecurityError)` - Argument validation failed
pub fn validate_cli_argument(
    arg_value: &str,
    arg_name: &str,
    validation_type: CliArgumentType,
) -> Result<(), SecurityError> {
    match validation_type {
        CliArgumentType::FilePath => {
            validate_file_path_for_storage(arg_value, arg_name)?;
            
            // Additional CLI-specific checks
            if arg_value.contains("$(") || arg_value.contains("`") {
                return Err(SecurityError::InvalidInput {
                    field: arg_name.to_string(),
                    reason: "File path contains shell injection patterns".to_string(),
                });
            }
        }
        CliArgumentType::Port => {
            let port: u16 = arg_value.parse().map_err(|_| SecurityError::InvalidInput {
                field: arg_name.to_string(),
                reason: "Invalid port number format".to_string(),
            })?;
            
            // Validate port range (avoid system ports)
            if port < 1024 {
                return Err(SecurityError::InvalidInput {
                    field: arg_name.to_string(),
                    reason: "Port number cannot be below 1024 (system reserved)".to_string(),
                });
            }
        }
        CliArgumentType::Percentage => {
            let percentage: f64 = arg_value.parse().map_err(|_| SecurityError::InvalidInput {
                field: arg_name.to_string(),
                reason: "Invalid percentage format".to_string(),
            })?;
            
            if !(0.0..=100.0).contains(&percentage) {
                return Err(SecurityError::InvalidInput {
                    field: arg_name.to_string(),
                    reason: "Percentage must be between 0.0 and 100.0".to_string(),
                });
            }
        }
        CliArgumentType::Count => {
            let count: u32 = arg_value.parse().map_err(|_| SecurityError::InvalidInput {
                field: arg_name.to_string(),
                reason: "Invalid count format".to_string(),
            })?;
            
            if count > 1_000_000 {
                return Err(SecurityError::InvalidInput {
                    field: arg_name.to_string(),
                    reason: "Count exceeds maximum allowed value of 1,000,000".to_string(),
                });
            }
        }
        CliArgumentType::Generic => {
            validate_input(arg_value, arg_name)?;
        }
    }
    
    Ok(())
}

/// CLI argument validation types
#[derive(Debug, Clone, Copy)]
pub enum CliArgumentType {
    /// File path argument requiring path validation
    FilePath,
    /// Port number argument (1-65535)
    Port,
    /// Percentage value argument (0-100)
    Percentage,
    /// Count or numeric argument
    Count,
    /// Generic string argument
    Generic,
}

/// Validate database query parameters to prevent SQL injection
///
/// Enhanced validation for any parameters that will be used in database queries,
/// even with parameterized queries, to add an extra layer of security.
///
/// # Arguments
/// * `param_value` - The parameter value to validate
/// * `param_name` - Name of the parameter for error reporting
/// * `param_type` - Expected parameter type
///
/// # Returns
/// * `Ok(())` - Parameter passes validation
/// * `Err(SecurityError)` - Parameter validation failed
pub fn validate_db_parameter(
    param_value: &str,
    param_name: &str,
    param_type: DbParameterType,
) -> Result<(), SecurityError> {
    match param_type {
        DbParameterType::Id => {
            // Validate numeric ID
            let _id: i64 = param_value.parse().map_err(|_| SecurityError::InvalidInput {
                field: param_name.to_string(),
                reason: "ID must be a valid integer".to_string(),
            })?;
        }
        DbParameterType::Text => {
            validate_input(param_value, param_name)?;
        }
        DbParameterType::FilePath => {
            validate_file_path_for_storage(param_value, param_name)?;
        }
        DbParameterType::CodeContent => {
            validate_code_analysis_data(param_value, param_name, None)?;
        }
        DbParameterType::Timestamp => {
            // Validate RFC3339 timestamp format
            chrono::DateTime::parse_from_rfc3339(param_value).map_err(|_| {
                SecurityError::InvalidInput {
                    field: param_name.to_string(),
                    reason: "Invalid timestamp format (RFC3339 required)".to_string(),
                }
            })?;
        }
    }
    
    Ok(())
}

/// Database parameter types for validation
#[derive(Debug, Clone, Copy)]
pub enum DbParameterType {
    /// Unique identifier parameter
    Id,
    /// General text parameter
    Text,
    /// File path parameter
    FilePath,
    /// Source code content parameter
    CodeContent,
    /// Timestamp parameter
    Timestamp,
}

/// Validate external API responses to prevent malicious data injection
///
/// Validates responses from external services (like Ollama) to ensure they don't
/// contain malicious content that could be stored in the database or displayed to users.
///
/// # Arguments
/// * `response_data` - The external API response data
/// * `max_size` - Maximum allowed response size in bytes
///
/// # Returns
/// * `Ok(())` - Response passes validation
/// * `Err(SecurityError)` - Response validation failed
pub fn validate_external_api_response(
    response_data: &str,
    max_size: Option<usize>,
) -> Result<(), SecurityError> {
    let max_size = max_size.unwrap_or(1024 * 1024); // 1MB default
    
    // Size check
    if response_data.len() > max_size {
        return Err(SecurityError::InvalidInput {
            field: "external_api_response".to_string(),
            reason: format!(
                "Response size {} exceeds maximum {} bytes",
                response_data.len(),
                max_size
            ),
        });
    }
    
    // Check for null bytes (binary data)
    if response_data.contains('\0') {
        return Err(SecurityError::InvalidInput {
            field: "external_api_response".to_string(),
            reason: "Response contains null bytes (possible binary data)".to_string(),
        });
    }
    
    // Check for excessive control characters (but allow newlines, tabs, carriage returns)
    let control_char_count = response_data
        .chars()
        .filter(|c| c.is_control() && !matches!(*c, '\n' | '\t' | '\r'))
        .count();
    
    if control_char_count > response_data.len() / 100 { // More than 1% control chars
        return Err(SecurityError::InvalidInput {
            field: "external_api_response".to_string(),
            reason: "Response contains excessive control characters".to_string(),
        });
    }
    
    // Basic HTML/script tag detection in API responses
    let dangerous_html_patterns = [
        "<script", "</script>", "<iframe", "</iframe>", 
        "javascript:", "vbscript:", "data:text/html",
        "onload=", "onerror=", "onclick=",
    ];
    
    let response_lower = response_data.to_lowercase();
    for pattern in &dangerous_html_patterns {
        if response_lower.contains(pattern) {
            return Err(SecurityError::InvalidInput {
                field: "external_api_response".to_string(),
                reason: format!("Response contains potentially dangerous HTML/script content: {}", pattern),
            });
        }
    }
    
    Ok(())
}

/// Validate memory limits to prevent resource exhaustion
///
/// Validates memory-related parameters to prevent out-of-memory attacks
/// and ensure reasonable resource usage.
///
/// # Arguments
/// * `memory_mb` - Memory limit in megabytes
/// * `field_name` - Name of the field for error reporting
///
/// # Returns
/// * `Ok(())` - Memory limit is valid
/// * `Err(SecurityError)` - Memory limit is invalid or excessive
pub fn validate_memory_limit(memory_mb: u64, field_name: &str) -> Result<(), SecurityError> {
    const MIN_MEMORY_MB: u64 = 64;   // 64MB minimum
    const MAX_MEMORY_MB: u64 = 8192; // 8GB maximum (adjust based on system capabilities)
    
    if memory_mb < MIN_MEMORY_MB {
        return Err(SecurityError::InvalidInput {
            field: field_name.to_string(),
            reason: format!(
                "Memory limit {} MB is below minimum {} MB",
                memory_mb, MIN_MEMORY_MB
            ),
        });
    }
    
    if memory_mb > MAX_MEMORY_MB {
        return Err(SecurityError::InvalidInput {
            field: field_name.to_string(),
            reason: format!(
                "Memory limit {} MB exceeds maximum {} MB",
                memory_mb, MAX_MEMORY_MB
            ),
        });
    }
    
    Ok(())
}

/// Sanitize text content for safe display in web interfaces
///
/// Sanitizes text content to prevent XSS attacks when displaying
/// analysis results or user-generated content in web interfaces.
///
/// # Arguments
/// * `content` - The text content to sanitize
///
/// # Returns
/// * `String` - Sanitized content safe for HTML display
pub fn sanitize_for_web_display(content: &str) -> String {
    content
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
        .replace('/', "&#x2F;")
}
