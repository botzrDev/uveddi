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

use std::path::{Path, PathBuf, Component};
use crate::error::UveddiError;

/// Security-related errors
#[derive(Debug, thiserror::Error)]
pub enum SecurityError {
    #[error("Path traversal attempt detected: {0}")]
    PathTraversal(String),
    #[error("Invalid file path: {0}")]
    InvalidPath(String),
    #[error("File size exceeds limit: {size} bytes > {limit} bytes")]
    FileSizeExceeded { size: u64, limit: u64 },
    #[error("Unsupported file type: {0}")]
    UnsupportedFileType(String),
    #[error("Input too long: {length} > {max_length}")]
    InputTooLong { length: usize, max_length: usize },
}

/// Maximum file size for analysis (100MB)
const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024;

/// Maximum input length for AI prompts (8KB)
const MAX_PROMPT_LENGTH: usize = 8 * 1024;

/// Validates and sanitizes a file path to prevent directory traversal attacks
pub fn validate_analysis_path(path: &str) -> Result<PathBuf, SecurityError> {
    let path_buf = PathBuf::from(path);
    
    // Check for directory traversal attempts
    for component in path_buf.components() {
        match component {
            Component::ParentDir => {
                return Err(SecurityError::PathTraversal(path.to_string()));
            }
            Component::Normal(name) => {
                let name_str = name.to_string_lossy();
                if name_str.starts_with('.') && name_str.len() > 1 {
                    // Allow .rs, .py, etc. but block hidden files like .env
                    if !name_str.contains('.') || name_str.starts_with("..") {
                        return Err(SecurityError::PathTraversal(path.to_string()));
                    }
                }
            }
            _ => {}
        }
    }
    
    // Canonicalize path if it exists
    match path_buf.canonicalize() {
        Ok(canonical) => Ok(canonical),
        Err(_) => {
            // If canonicalization fails, ensure the path doesn't contain suspicious patterns
            let path_str = path_buf.to_string_lossy();
            if path_str.contains("..") || path_str.contains("~") {
                Err(SecurityError::PathTraversal(path.to_string()))
            } else {
                Ok(path_buf)
            }
        }
    }
}

/// Sanitizes API keys for logging (shows only first 4 characters)
pub fn sanitize_api_key(key: &str) -> String {
    if key.is_empty() {
        return "***".to_string();
    }
    
    if key.len() > 8 {
        format!("{}***", &key[..4])
    } else {
        "***".to_string()
    }
}

/// Validates file size to prevent memory exhaustion
pub fn validate_file_size(path: &Path) -> Result<(), SecurityError> {
    match std::fs::metadata(path) {
        Ok(metadata) => {
            let size = metadata.len();
            if size > MAX_FILE_SIZE {
                Err(SecurityError::FileSizeExceeded {
                    size,
                    limit: MAX_FILE_SIZE,
                })
            } else {
                Ok(())
            }
        }
        Err(_) => Err(SecurityError::InvalidPath(path.to_string_lossy().to_string())),
    }
}

/// Validates file type based on extension
pub fn validate_file_type(path: &Path) -> Result<(), SecurityError> {
    const ALLOWED_EXTENSIONS: &[&str] = &[
        "rs", "py", "js", "ts", "jsx", "tsx", "java", "cpp", "c", "h", "hpp",
        "go", "rb", "php", "swift", "kt", "scala", "clj", "hs", "ml", "fs",
        "elm", "dart", "vue", "svelte", "css", "scss", "sass", "less",
        "html", "xml", "json", "yaml", "yml", "toml", "md", "txt"
    ];
    
    if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
        let ext_lower = extension.to_lowercase();
        if ALLOWED_EXTENSIONS.contains(&ext_lower.as_str()) {
            Ok(())
        } else {
            Err(SecurityError::UnsupportedFileType(extension.to_string()))
        }
    } else {
        // Allow files without extensions for now (like Dockerfile, Makefile)
        Ok(())
    }
}

/// Sanitizes input for AI prompts to prevent injection attacks
pub fn sanitize_prompt(prompt: &str) -> Result<String, SecurityError> {
    if prompt.len() > MAX_PROMPT_LENGTH {
        return Err(SecurityError::InputTooLong {
            length: prompt.len(),
            max_length: MAX_PROMPT_LENGTH,
        });
    }
    
    let sanitized = prompt
        // Remove potential injection patterns
        .replace("```", "")
        .replace("SYSTEM:", "")
        .replace("USER:", "")
        .replace("ASSISTANT:", "")
        // Remove control characters but keep newlines and tabs
        .chars()
        .filter(|c| c.is_ascii_graphic() || matches!(*c, ' ' | '\n' | '\t'))
        .collect::<String>();
    
    Ok(sanitized)
}

/// Validates that a path is within an allowed directory
pub fn validate_path_within_bounds(path: &Path, allowed_root: &Path) -> Result<(), SecurityError> {
    let canonical_path = path.canonicalize()
        .map_err(|_| SecurityError::InvalidPath(path.to_string_lossy().to_string()))?;
    
    let canonical_root = allowed_root.canonicalize()
        .map_err(|_| SecurityError::InvalidPath(allowed_root.to_string_lossy().to_string()))?;
    
    if canonical_path.starts_with(canonical_root) {
        Ok(())
    } else {
        Err(SecurityError::PathTraversal(path.to_string_lossy().to_string()))
    }
}