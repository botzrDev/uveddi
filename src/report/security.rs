//! Security Measures for Report Generation
//!
//! This module implements comprehensive security measures for report generation,
//! including input validation, output sanitization, and defensive security practices
//! as outlined in the architectural analysis system research.

use std::path::Path;
use thiserror::Error;

/// Security errors related to report generation
#[derive(Error, Debug)]
pub enum ReportSecurityError {
    /// Malicious content detected
    #[error("Malicious content detected: {0}")]
    MaliciousContent(String),
    /// Content size limit exceeded
    #[error("Content size limit exceeded: {current} bytes > {limit} bytes")]
    SizeLimit {
        /// Current content size
        current: usize,
        /// Maximum allowed size
        limit: usize,
    },
    /// Invalid file path provided
    #[error("Invalid file path: {0}")]
    InvalidPath(String),
    /// Unsafe content detected
    #[error("Unsafe content found: {0}")]
    UnsafeContent(String),
    /// Encoding validation failed
    #[error("Encoding validation failed: {0}")]
    EncodingError(String),
}

/// Configuration for report security settings
#[derive(Debug, Clone)]
pub struct ReportSecurityConfig {
    /// Maximum report content size in bytes
    pub max_content_size: usize,
    /// Maximum number of diagrams per report
    pub max_diagrams: usize,
    /// Enable content sanitization
    pub enable_sanitization: bool,
    /// Enable path validation
    pub enable_path_validation: bool,
    /// Allowed file extensions for diagram output
    pub allowed_extensions: Vec<String>,
}

impl Default for ReportSecurityConfig {
    fn default() -> Self {
        Self {
            max_content_size: 50 * 1024 * 1024, // 50MB
            max_diagrams: 100,
            enable_sanitization: true,
            enable_path_validation: true,
            allowed_extensions: vec![
                "svg".to_string(),
                "png".to_string(),
                "jpg".to_string(),
                "html".to_string(),
                "md".to_string(),
                "json".to_string(),
            ],
        }
    }
}

/// Report security validator and sanitizer
pub struct ReportSecurityValidator {
    config: ReportSecurityConfig,
}

impl ReportSecurityValidator {
    /// Create a new security validator with configuration
    pub fn new(config: ReportSecurityConfig) -> Self {
        Self { config }
    }

    /// Create validator with default security settings
    pub fn default() -> Self {
        Self::new(ReportSecurityConfig::default())
    }

    /// Validate and sanitize report content before generation
    pub fn validate_content(&self, content: &str) -> Result<String, ReportSecurityError> {
        // Check content size limits
        if content.len() > self.config.max_content_size {
            return Err(ReportSecurityError::SizeLimit {
                current: content.len(),
                limit: self.config.max_content_size,
            });
        }

        // Detect potentially malicious content patterns
        self.detect_malicious_patterns(content)?;

        // Sanitize content if enabled
        if self.config.enable_sanitization {
            Ok(self.sanitize_content(content))
        } else {
            Ok(content.to_string())
        }
    }

    /// Validate file paths for diagram output
    pub fn validate_output_path(&self, path: &Path) -> Result<(), ReportSecurityError> {
        if !self.config.enable_path_validation {
            return Ok(());
        }

        // Convert to string for validation
        let path_str = path.to_string_lossy();

        // Check for path traversal attacks
        if path_str.contains("..") || path_str.contains("~") {
            return Err(ReportSecurityError::InvalidPath(
                "Path traversal detected".to_string(),
            ));
        }

        // Check for suspicious paths
        let suspicious_patterns = [
            "/etc/",
            "/proc/",
            "/sys/",
            "/dev/",
            "C:\\Windows\\",
            "C:\\Program Files\\",
            "/usr/bin/",
            "/bin/",
            "/sbin/",
        ];

        for pattern in &suspicious_patterns {
            if path_str.contains(pattern) {
                return Err(ReportSecurityError::InvalidPath(format!(
                    "Suspicious path pattern detected: {}",
                    pattern
                )));
            }
        }

        // Validate file extension
        if let Some(extension) = path.extension() {
            let ext_str = extension.to_string_lossy().to_lowercase();
            if !self.config.allowed_extensions.contains(&ext_str) {
                return Err(ReportSecurityError::InvalidPath(format!(
                    "Disallowed file extension: {}",
                    ext_str
                )));
            }
        }

        Ok(())
    }

    /// Sanitize Mermaid diagram code to prevent injection attacks
    pub fn sanitize_mermaid_code(&self, mermaid_code: &str) -> Result<String, ReportSecurityError> {
        // Check for script injection patterns
        let dangerous_patterns = [
            "<script",
            "</script>",
            "javascript:",
            "vbscript:",
            "onload=",
            "onerror=",
            "onclick=",
            "onmouseover=",
            "eval(",
            "Function(",
            "setTimeout(",
            "setInterval(",
            "document.cookie",
            "window.location",
            "alert(",
        ];

        for pattern in &dangerous_patterns {
            if mermaid_code.to_lowercase().contains(pattern) {
                return Err(ReportSecurityError::UnsafeContent(format!(
                    "Potentially dangerous pattern found: {}",
                    pattern
                )));
            }
        }

        // Sanitize and return safe version
        Ok(self.sanitize_content(mermaid_code))
    }

    /// Validate JSON report data structure for security issues
    pub fn validate_json_structure(
        &self,
        json_data: &serde_json::Value,
    ) -> Result<(), ReportSecurityError> {
        // Check for excessively deep nesting (potential DoS)
        if self.get_json_depth(json_data) > 50 {
            return Err(ReportSecurityError::UnsafeContent(
                "JSON structure too deeply nested".to_string(),
            ));
        }

        // Check for excessively large arrays (potential DoS)
        self.validate_json_arrays(json_data)?;

        // Validate string content within JSON
        self.validate_json_strings(json_data)?;

        Ok(())
    }

    /// Content Security Policy (CSP) directives for HTML reports
    pub fn get_content_security_policy(&self) -> String {
        vec![
            "default-src 'self'",
            "script-src 'self' 'unsafe-inline' https://cdn.jsdelivr.net",
            "style-src 'self' 'unsafe-inline' https://fonts.googleapis.com",
            "font-src 'self' https://fonts.gstatic.com",
            "img-src 'self' data:",
            "connect-src 'self'",
            "frame-ancestors 'none'",
            "object-src 'none'",
            "base-uri 'self'",
        ]
        .join("; ")
    }

    /// Generate secure HTTP headers for report serving
    pub fn get_security_headers(&self) -> Vec<(String, String)> {
        vec![
            (
                "Content-Security-Policy".to_string(),
                self.get_content_security_policy(),
            ),
            ("X-Content-Type-Options".to_string(), "nosniff".to_string()),
            ("X-Frame-Options".to_string(), "DENY".to_string()),
            (
                "Referrer-Policy".to_string(),
                "strict-origin-when-cross-origin".to_string(),
            ),
            (
                "Permissions-Policy".to_string(),
                "geolocation=(), microphone=(), camera=()".to_string(),
            ),
        ]
    }

    /// Detect potentially malicious patterns in content
    fn detect_malicious_patterns(&self, content: &str) -> Result<(), ReportSecurityError> {
        let malicious_patterns = [
            // Script injection
            "<script",
            "javascript:",
            "vbscript:",
            // Command injection
            "; rm -rf",
            "| rm -rf",
            "&& rm -rf",
            "; del",
            "| del",
            "&& del",
            // Path traversal
            "../../../",
            "..\\..\\..\\",
            // SQL injection patterns
            "'; DROP TABLE",
            "' OR '1'='1",
            // XSS patterns
            "document.cookie",
            "window.location",
            "eval(",
            "Function(",
        ];

        let content_lower = content.to_lowercase();
        for pattern in &malicious_patterns {
            if content_lower.contains(pattern) {
                return Err(ReportSecurityError::MaliciousContent(format!(
                    "Detected pattern: {}",
                    pattern
                )));
            }
        }

        Ok(())
    }

    /// Sanitize content by removing/escaping dangerous characters
    fn sanitize_content(&self, content: &str) -> String {
        content
            // HTML entities
            .replace("&", "&amp;")
            .replace("<", "&lt;")
            .replace(">", "&gt;")
            .replace("\"", "&quot;")
            .replace("'", "&#x27;")
            // Remove null bytes and control characters
            .chars()
            .filter(|&c| c != '\0' && (c.is_ascii_graphic() || c.is_ascii_whitespace()))
            .collect()
    }

    /// Calculate JSON nesting depth
    fn get_json_depth(&self, value: &serde_json::Value) -> usize {
        match value {
            serde_json::Value::Object(obj) => {
                1 + obj
                    .values()
                    .map(|v| self.get_json_depth(v))
                    .max()
                    .unwrap_or(0)
            }
            serde_json::Value::Array(arr) => {
                1 + arr
                    .iter()
                    .map(|v| self.get_json_depth(v))
                    .max()
                    .unwrap_or(0)
            }
            _ => 0,
        }
    }

    /// Validate JSON arrays for reasonable sizes
    fn validate_json_arrays(&self, value: &serde_json::Value) -> Result<(), ReportSecurityError> {
        match value {
            serde_json::Value::Array(arr) => {
                if arr.len() > 10000 {
                    return Err(ReportSecurityError::UnsafeContent(format!(
                        "Array too large: {} elements",
                        arr.len()
                    )));
                }
                // Recursively validate nested arrays
                for item in arr {
                    self.validate_json_arrays(item)?;
                }
            }
            serde_json::Value::Object(obj) => {
                for value in obj.values() {
                    self.validate_json_arrays(value)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Validate string content within JSON structures
    fn validate_json_strings(&self, value: &serde_json::Value) -> Result<(), ReportSecurityError> {
        match value {
            serde_json::Value::String(s) => {
                // Check string length
                if s.len() > 1_000_000 {
                    // 1MB limit per string
                    return Err(ReportSecurityError::UnsafeContent(
                        "String too large in JSON".to_string(),
                    ));
                }
                // Check for malicious patterns
                self.detect_malicious_patterns(s)?;
            }
            serde_json::Value::Array(arr) => {
                for item in arr {
                    self.validate_json_strings(item)?;
                }
            }
            serde_json::Value::Object(obj) => {
                for value in obj.values() {
                    self.validate_json_strings(value)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_path_traversal_detection() {
        let validator = ReportSecurityValidator::default();

        // Should reject path traversal
        assert!(validator
            .validate_output_path(&PathBuf::from("../../../etc/passwd"))
            .is_err());
        assert!(validator
            .validate_output_path(&PathBuf::from("..\\..\\..\\windows\\system32"))
            .is_err());

        // Should accept safe paths
        assert!(validator
            .validate_output_path(&PathBuf::from("output/diagrams/graph.svg"))
            .is_ok());
    }

    #[test]
    fn test_malicious_content_detection() {
        let validator = ReportSecurityValidator::default();

        // Should detect script injection
        assert!(validator
            .validate_content("<script>alert('xss')</script>")
            .is_err());
        assert!(validator
            .validate_content("javascript:alert('xss')")
            .is_err());

        // Should accept safe content
        assert!(validator.validate_content("This is safe content.").is_ok());
    }

    #[test]
    fn test_mermaid_sanitization() {
        let validator = ReportSecurityValidator::default();

        // Should detect dangerous patterns
        assert!(validator
            .sanitize_mermaid_code("graph TD\n  A[Start] --> B<script>alert('xss')</script>")
            .is_err());

        // Should accept safe Mermaid
        assert!(validator
            .sanitize_mermaid_code("graph TD\n  A[Start] --> B[End]")
            .is_ok());
    }

    #[test]
    fn test_content_sanitization() {
        let validator = ReportSecurityValidator::default();

        let dirty_content = "<script>alert('xss')</script>";
        let clean_content = validator.sanitize_content(dirty_content);

        assert_eq!(
            clean_content,
            "&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;"
        );
    }
}
