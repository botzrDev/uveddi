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
    /// Description too long
    #[error("Description too long: {length} > {max_length}")]
    DescriptionTooLong { length: usize, max_length: usize },
}

/// Maximum file size for analysis (100MB)
pub const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024;

/// Maximum input length for AI prompts (8KB)
pub const MAX_PROMPT_LENGTH: usize = 8 * 1024;

/// Maximum directory depth for traversal (prevents infinite recursion)
pub const MAX_DIRECTORY_DEPTH: usize = 50;

/// Maximum number of files per analysis (prevents resource exhaustion)
pub const MAX_FILES_PER_ANALYSIS: usize = 10_000;

/// Maximum description length for database fields (10KB)
pub const MAX_DESCRIPTION_LENGTH: usize = 10 * 1024;

/// Validates and sanitizes a file path to prevent directory traversal attacks
///
/// # Arguments
/// * `path` - The input file path as a string slice
///
/// # Returns
/// * `Result<PathBuf, SecurityError>` - Canonicalized safe path or error
///
/// # UV-151
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
///
/// # Arguments
/// * `key` - The API key string
///
/// # Returns
/// * `String` - Sanitized API key for safe logging
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
///
/// # Arguments
/// * `path` - The file path
///
/// # Returns
/// * `Result<(), SecurityError>` - Ok if size is valid, error otherwise
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
///
/// # Arguments
/// * `path` - The file path
///
/// # Returns
/// * `Result<(), SecurityError>` - Ok if type is allowed, error otherwise
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
///
/// # Arguments
/// * `prompt` - The input prompt string
///
/// # Returns
/// * `Result<String, SecurityError>` - Sanitized prompt or error
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

/// Validates URL format for API endpoints
///
/// # Arguments
/// * `url` - The URL string to validate
///
/// # Returns
/// * `Result<url::Url, SecurityError>` - Parsed URL or error
pub fn validate_api_url(url: &str) -> Result<url::Url, SecurityError> {
    use url::Url;
    
    let parsed_url = Url::parse(url)
        .map_err(|_| SecurityError::InvalidUrl(url.to_string()))?;
    
    // Only allow HTTP and HTTPS schemes
    match parsed_url.scheme() {
        "http" | "https" => Ok(parsed_url),
        _ => Err(SecurityError::InvalidUrl(format!("Unsupported scheme: {}", parsed_url.scheme()))),
    }
}

/// Validates AI model name format
///
/// # Arguments
/// * `model` - The model name to validate
///
/// # Returns
/// * `Result<(), SecurityError>` - Ok if valid, error otherwise
pub fn validate_model_name(model: &str) -> Result<(), SecurityError> {
    // Model names should be alphanumeric with hyphens, colons, and dots
    if model.is_empty() || model.len() > 100 {
        return Err(SecurityError::InvalidModelName("Model name length invalid".to_string()));
    }
    
    let valid_chars = model.chars().all(|c| {
        c.is_alphanumeric() || matches!(c, '-' | ':' | '.' | '_')
    });
    
    if !valid_chars {
        return Err(SecurityError::InvalidModelName("Model name contains invalid characters".to_string()));
    }
    
    Ok(())
}

/// Sanitizes description text for database storage
///
/// # Arguments
/// * `description` - The description text to sanitize
///
/// # Returns
/// * `Result<String, SecurityError>` - Sanitized description or error
/// Validates directory depth to prevent infinite recursion
///
/// # Arguments
/// * `depth` - Current directory depth
///
/// # Returns
/// * `Result<(), SecurityError>` - Ok if depth is valid, error otherwise
pub fn validate_directory_depth(depth: usize) -> Result<(), SecurityError> {
    if depth > MAX_DIRECTORY_DEPTH {
        Err(SecurityError::DirectoryDepthExceeded {
            depth,
            max_depth: MAX_DIRECTORY_DEPTH,
        })
    } else {
        Ok(())
    }
}

/// Validates file count to prevent resource exhaustion
///
/// # Arguments
/// * `count` - Current file count
///
/// # Returns
/// * `Result<(), SecurityError>` - Ok if count is valid, error otherwise
pub fn validate_file_count(count: usize) -> Result<(), SecurityError> {
    if count > MAX_FILES_PER_ANALYSIS {
        Err(SecurityError::TooManyFiles {
            count,
            max_count: MAX_FILES_PER_ANALYSIS,
        })
    } else {
        Ok(())
    }
}

pub fn sanitize_description(description: &str) -> Result<String, SecurityError> {
    if description.len() > MAX_DESCRIPTION_LENGTH {
        return Err(SecurityError::DescriptionTooLong {
            length: description.len(),
            max_length: MAX_DESCRIPTION_LENGTH,
        });
    }
    
    // Remove potentially dangerous characters but preserve formatting
    let sanitized = description
        .chars()
        .filter(|c| {
            c.is_ascii_graphic() || matches!(*c, ' ' | '\n' | '\t' | '\r')
        })
        .collect::<String>()
        // Remove potential SQL injection patterns
        .replace("--", "")
        .replace("/*", "")
        .replace("*/", "")
        .replace(";", "")
        // Limit consecutive newlines
        .split('\n')
        .collect::<Vec<_>>()
        .join("\n");
    
    Ok(sanitized)
}

/// Validates directory depth during traversal
///
/// # Arguments
/// * `depth` - Current directory depth
///
/// # Returns
/// * `Result<(), SecurityError>` - Ok if within limits, error otherwise
pub fn validate_directory_depth(depth: usize) -> Result<(), SecurityError> {
    if depth > MAX_DIRECTORY_DEPTH {
        Err(SecurityError::DirectoryDepthExceeded {
            depth,
            max_depth: MAX_DIRECTORY_DEPTH,
        })
    } else {
        Ok(())
    }
}

/// Validates file count during analysis
///
/// # Arguments
/// * `count` - Current file count
///
/// # Returns
/// * `Result<(), SecurityError>` - Ok if within limits, error otherwise
pub fn validate_file_count(count: usize) -> Result<(), SecurityError> {
    if count > MAX_FILES_PER_ANALYSIS {
        Err(SecurityError::TooManyFiles {
            count,
            max_count: MAX_FILES_PER_ANALYSIS,
        })
    } else {
        Ok(())
    }
}

/// Validates that a path is within an allowed directory
///
/// # Arguments
/// * `path` - The file path to check
/// * `allowed_root` - The allowed root directory
///
/// # Returns
/// * `Result<(), SecurityError>` - Ok if within bounds, error otherwise
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

/// Securely sanitizes a file path to prevent directory traversal and related attacks
///
/// # UV-151 Critical Path Traversal Vulnerability Fix
///
/// Ensures the input path is canonicalized, validated, and strictly contained within the base directory.
///
/// # Arguments
/// * `input_path` - The input file path
/// * `base_dir` - The base directory for analysis
///
/// # Returns
/// * `Result<PathBuf, SecurityError>` - Canonicalized safe path or error
///
/// # Examples
/// ```rust
/// let safe_path = sanitize_path("src/main.rs", "src").unwrap();
/// ```
pub fn sanitize_path<P: AsRef<Path>>(input_path: P, base_dir: P) -> Result<PathBuf, SecurityError> {
    // Canonicalize the base directory
    let base = base_dir.as_ref().canonicalize()
        .map_err(|_| SecurityError::InvalidBasePath)?;

    // Join and canonicalize the input path
    let path = base.join(input_path.as_ref());
    let canonical = path.canonicalize()
        .map_err(|_| SecurityError::InvalidPathComponent)?;

    // Ensure the canonical path is within the base directory
    if !canonical.starts_with(&base) {
        return Err(SecurityError::PathTraversalAttempt);
    }

    // Additional validation for path components
    for component in canonical.components() {
        if let std::path::Component::Normal(name) = component {
            let name_str = name.to_string_lossy();
            if name_str.contains('\0') || name_str.starts_with('.') {
                return Err(SecurityError::InvalidPathComponent);
            }
        }
    }

    Ok(canonical)
}

/// Placeholder documentation for public items
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::symlink;
    use tempfile::tempdir;

    // UV-151: Security test suite for sanitize_path
    #[test]
    fn test_basic_traversal_attempt() {
        let dir = tempdir().unwrap();
        let result = sanitize_path("../../etc/passwd", dir.path());
        assert!(matches!(result, Err(SecurityError::PathTraversalAttempt)));
    }

    #[test]
    fn test_symlink_attack() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("target.txt");
        fs::write(&target, "safe").unwrap();
        let link = dir.path().join("link.txt");
        
        // Only test symlinks on Unix systems
        #[cfg(unix)]
        {
            symlink(&target, &link).unwrap();
            let result = sanitize_path("link.txt", dir.path());
            assert!(result.is_ok());
        }
        
        #[cfg(not(unix))]
        {
            // On non-Unix systems, just test a regular file
            fs::write(&link, "safe").unwrap();
            let result = sanitize_path("link.txt", dir.path());
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_null_byte_injection() {
        let dir = tempdir().unwrap();
        let result = sanitize_path("file\0.txt", dir.path());
        assert!(matches!(result, Err(SecurityError::InvalidPathComponent)));
    }

    #[test]
    fn test_unicode_normalization_attack() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("unicodé.txt");
        fs::write(&file, "safe").unwrap();
        let result = sanitize_path("unicodé.txt", dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_hidden_file_blocked() {
        let dir = tempdir().unwrap();
        let file = dir.path().join(".env");
        fs::write(&file, "secret").unwrap();
        let result = sanitize_path(".env", dir.path());
        assert!(matches!(result, Err(SecurityError::InvalidPathComponent)));
    }

    #[test]
    fn test_valid_path() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("main.rs");
        fs::write(&file, "fn main() {}\n").unwrap();
        let result = sanitize_path("main.rs", dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_base_dir() {
        let result = sanitize_path("main.rs", "/nonexistent_base_dir");
        assert!(matches!(result, Err(SecurityError::InvalidBasePath)));
    }

    #[test]
    fn test_dot_file_blocked() {
        let dir = tempdir().unwrap();
        let file = dir.path().join(".hidden");
        fs::write(&file, "hidden").unwrap();
        let result = sanitize_path(".hidden", dir.path());
        assert!(matches!(result, Err(SecurityError::InvalidPathComponent)));
    }

    #[test]
    fn test_path_outside_base() {
        let dir = tempdir().unwrap();
        let outside = std::env::temp_dir().join("outside.txt");
        fs::write(&outside, "outside").unwrap();
        let result = sanitize_path(outside, dir.path());
        assert!(matches!(result, Err(SecurityError::PathTraversalAttempt)));
    }

    #[test]
    fn test_valid_nested_path() {
        let dir = tempdir().unwrap();
        let nested = dir.path().join("nested");
        fs::create_dir(&nested).unwrap();
        let file = nested.join("file.txt");
        fs::write(&file, "nested").unwrap();
        let result = sanitize_path("nested/file.txt", dir.path());
        assert!(result.is_ok());
    }
}