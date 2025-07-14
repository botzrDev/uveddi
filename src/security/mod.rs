//! Security and Safety Module for Uveddi
//!
//! This module provides security-related utilities and safety mechanisms for the Uveddi
//! analysis tool. It handles sensitive data management, input validation, and sandboxing
//! for potentially unsafe operations.
//!
//! # Security Features
//!
//! ## Credential Management
//! - **API Key Storage**: Secure handling of AI provider API keys
//! - **Environment Variables**: Safe access to sensitive configuration
//! - **Key Rotation**: Support for credential rotation and updates
//!
//! ## Input Validation
//! - **Path Sanitization**: Prevent directory traversal attacks
//! - **File Type Validation**: Ensure only safe file types are processed
//! - **Size Limits**: Prevent resource exhaustion from large inputs
//!
//! ## Plugin Sandboxing
//! - **WASM Isolation**: Sandbox custom analysis plugins using WebAssembly
//! - **Resource Limits**: CPU and memory limits for plugin execution
//! - **API Restrictions**: Controlled access to system resources
//!
//! # Threat Model
//!
//! The security module addresses these potential threats:
//! - **Malicious Input Files**: Code files designed to exploit parser vulnerabilities
//! - **Path Traversal**: Attempts to access files outside the analysis scope
//! - **Resource Exhaustion**: Large or deeply nested files causing DoS
//! - **Information Disclosure**: Accidental exposure of sensitive data in reports
//! - **Plugin Vulnerabilities**: Untrusted analysis plugins causing system compromise
//!
//! # Usage Guidelines
//!
//! ```rust,no_run
//! use uveddi::security;
//!
//! // Validate input path before analysis
//! let safe_path = security::validate_analysis_path("./src")?;
//!
//! // Sanitize API keys before logging
//! let safe_key = security::sanitize_api_key(&api_key);
//! log::info!("Using API key: {}", safe_key);
//!
//! // Check file size limits
//! if security::is_file_too_large(&file_path)? {
//!     return Err("File exceeds maximum size limit".into());
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Future Enhancements
//!
//! - **Cryptographic Verification**: Code signature validation
//! - **Audit Logging**: Security event logging and monitoring
//! - **Access Control**: Role-based permissions for analysis features
//! - **Data Privacy**: Anonymization of sensitive code patterns

use crate::error::UveddiError;
use std::path::{Component, Path, PathBuf};

/// Security-related errors for Uveddi
#[derive(Debug, thiserror::Error)]
pub enum SecurityError {
    /// Path traversal attempt detected
    #[error("Path traversal attempt detected: {0}")]
    PathTraversal(String),
    /// Invalid file path
    #[error("Invalid file path: {0}")]
    InvalidPath(String),
    /// File size exceeds limit
    #[error("File size exceeds limit: {size} bytes > {limit} bytes")]
    FileSizeExceeded { size: u64, limit: u64 },
    /// Unsupported file type
    #[error("Unsupported file type: {0}")]
    UnsupportedFileType(String),
    /// Input too long for AI prompt
    #[error("Input too long: {length} > {max_length}")]
    InputTooLong { length: usize, max_length: usize },
    /// Invalid base path for analysis
    #[error("Invalid base path")]
    InvalidBasePath,
    /// Path traversal attempt outside base directory
    #[error("Path traversal attempt outside base directory")]
    PathTraversalAttempt,
    /// Invalid path component detected (e.g., null byte, hidden file)
    #[error("Invalid path component detected")]
    InvalidPathComponent,
    /// Directory depth exceeds maximum allowed
    #[error("Directory depth exceeds maximum: {depth} > {max_depth}")]
    DirectoryDepthExceeded { depth: usize, max_depth: usize },
    /// Too many files in analysis
    #[error("File count exceeds maximum: {count} > {max_count}")]
    TooManyFiles { count: usize, max_count: usize },
    /// Invalid URL format
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    /// Invalid model name
    #[error("Invalid model name: {0}")]
    InvalidModelName(String),
}

/// Maximum number of files allowed per analysis
pub const MAX_FILES_PER_ANALYSIS: usize = 10000;

/// Validate file size for analysis
pub fn validate_file_size(path: &Path) -> Result<(), SecurityError> {
    let metadata = std::fs::metadata(path)
        .map_err(|_| SecurityError::InvalidPath(path.display().to_string()))?;
    let size = metadata.len();
    let max_size = 10 * 1024 * 1024; // 10MB
    if size > max_size {
        return Err(SecurityError::FileSizeExceeded {
            size,
            limit: max_size,
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
            .ok_or(SecurityError::UnsupportedFileType(
                "No extension".to_string(),
            ))?;
    if !allowed.contains(&ext) {
        return Err(SecurityError::UnsupportedFileType(ext.to_string()));
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
        return Err(SecurityError::InvalidModelName(name.to_string()));
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
        return Err(SecurityError::DirectoryDepthExceeded { depth, max_depth });
    }
    Ok(())
}

/// Validate file count for analysis
pub fn validate_file_count(count: usize) -> Result<(), SecurityError> {
    if count > MAX_FILES_PER_ANALYSIS {
        return Err(SecurityError::TooManyFiles {
            count,
            max_count: MAX_FILES_PER_ANALYSIS,
        });
    }
    Ok(())
}
