//! Configuration validation utilities and helpers

use crate::error::UveddiError;
use std::path::Path;

/// Configuration validator providing common validation logic
pub struct ConfigValidator;

impl ConfigValidator {
    /// Validate that a path exists and is accessible
    pub fn validate_path_exists(path: &Path, context: &str) -> Result<(), UveddiError> {
        if !path.exists() {
            return Err(UveddiError::PathError {
                path: path.display().to_string(),
                reason: format!("{} path does not exist", context),
                suggestion: "Verify the path exists and is accessible".to_string(),
            });
        }
        Ok(())
    }

    /// Validate that a directory exists and is writable
    pub fn validate_writable_directory(path: &Path, context: &str) -> Result<(), UveddiError> {
        Self::validate_path_exists(path, context)?;

        if !path.is_dir() {
            return Err(UveddiError::PathError {
                path: path.display().to_string(),
                reason: format!("{} path is not a directory", context),
                suggestion: "Provide a valid directory path".to_string(),
            });
        }

        // Check if directory is writable by attempting to create a temporary file
        let test_file = path.join(".uveddi_write_test");
        match std::fs::write(&test_file, "test") {
            Ok(()) => {
                // Clean up test file
                let _ = std::fs::remove_file(&test_file);
                Ok(())
            }
            Err(_) => Err(UveddiError::PathError {
                path: path.display().to_string(),
                reason: format!("{} directory is not writable", context),
                suggestion: "Check directory permissions".to_string(),
            }),
        }
    }

    /// Validate that a file has the expected extension
    pub fn validate_file_extension(
        path: &Path,
        expected_extensions: &[&str],
        context: &str,
    ) -> Result<(), UveddiError> {
        if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
            if expected_extensions.contains(&extension) {
                Ok(())
            } else {
                Err(UveddiError::config_error(
                    &format!(
                        "{} file has invalid extension '{}', expected one of: {}",
                        context,
                        extension,
                        expected_extensions.join(", ")
                    ),
                    "file validation",
                ))
            }
        } else {
            Err(UveddiError::config_error(
                &format!("{} file has no extension", context),
                "file validation",
            ))
        }
    }

    /// Validate a URL format
    pub fn validate_url(url: &str, context: &str) -> Result<(), UveddiError> {
        if url.is_empty() {
            return Err(UveddiError::config_error(
                &format!("{} URL cannot be empty", context),
                "URL validation",
            ));
        }

        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(UveddiError::config_error(
                &format!("{} URL must start with http:// or https://", context),
                "URL validation",
            ));
        }

        // Basic URL validation without external dependency
        if !url.contains("://") || url.len() < 10 {
            return Err(UveddiError::config_error(
                &format!("{} URL format is invalid", context),
                "URL validation",
            ));
        }

        Ok(())
    }

    /// Validate a numeric range
    pub fn validate_range<T>(
        value: T,
        min: T,
        max: T,
        field_name: &str,
        context: &str,
    ) -> Result<(), UveddiError>
    where
        T: PartialOrd + std::fmt::Display + Copy,
    {
        if value < min || value > max {
            return Err(UveddiError::config_error(
                &format!(
                    "{} {} must be between {} and {}, got {}",
                    context, field_name, min, max, value
                ),
                "range validation",
            ));
        }
        Ok(())
    }

    /// Validate that a string is not empty
    pub fn validate_non_empty_string(value: &str, field_name: &str, context: &str) -> Result<(), UveddiError> {
        if value.trim().is_empty() {
            return Err(UveddiError::config_error(
                &format!("{} {} cannot be empty", context, field_name),
                "string validation",
            ));
        }
        Ok(())
    }

    /// Validate that a value is one of the allowed options
    pub fn validate_enum_value<T>(
        value: &T,
        allowed_values: &[T],
        field_name: &str,
        context: &str,
    ) -> Result<(), UveddiError>
    where
        T: PartialEq + std::fmt::Display,
    {
        if !allowed_values.contains(value) {
            let allowed_str = allowed_values
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            return Err(UveddiError::config_error(
                &format!(
                    "{} {} must be one of: {}, got {}",
                    context, field_name, allowed_str, value
                ),
                "enum validation",
            ));
        }
        Ok(())
    }

    /// Validate memory size in bytes
    pub fn validate_memory_size(size_bytes: usize, min_mb: usize, max_gb: usize, context: &str) -> Result<(), UveddiError> {
        let min_bytes = min_mb * 1024 * 1024;
        let max_bytes = max_gb * 1024 * 1024 * 1024;

        if size_bytes < min_bytes {
            return Err(UveddiError::config_error(
                &format!(
                    "{} memory size {}MB is below minimum {}MB",
                    context,
                    size_bytes / (1024 * 1024),
                    min_mb
                ),
                "memory validation",
            ));
        }

        if size_bytes > max_bytes {
            return Err(UveddiError::config_error(
                &format!(
                    "{} memory size {}GB is above maximum {}GB",
                    context,
                    size_bytes / (1024 * 1024 * 1024),
                    max_gb
                ),
                "memory validation",
            ));
        }

        Ok(())
    }

    /// Validate a timeout value in seconds
    pub fn validate_timeout(timeout_seconds: u64, min_seconds: u64, max_seconds: u64, context: &str) -> Result<(), UveddiError> {
        if timeout_seconds > 0 && timeout_seconds < min_seconds {
            return Err(UveddiError::config_error(
                &format!(
                    "{} timeout {}s is below minimum {}s",
                    context, timeout_seconds, min_seconds
                ),
                "timeout validation",
            ));
        }

        if timeout_seconds > max_seconds {
            return Err(UveddiError::config_error(
                &format!(
                    "{} timeout {}s is above maximum {}s",
                    context, timeout_seconds, max_seconds
                ),
                "timeout validation",
            ));
        }

        Ok(())
    }

    /// Validate a percentage value (0.0 to 1.0)
    pub fn validate_percentage(value: f64, field_name: &str, context: &str) -> Result<(), UveddiError> {
        Self::validate_range(value, 0.0, 1.0, field_name, context)
    }

    /// Validate that parent directory exists for a file path
    pub fn validate_parent_directory_exists(path: &Path, context: &str) -> Result<(), UveddiError> {
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                return Err(UveddiError::PathError {
                    path: parent.display().to_string(),
                    reason: format!("{} parent directory does not exist", context),
                    suggestion: "Create the parent directory first".to_string(),
                });
            }
        }
        Ok(())
    }

    /// Validate configuration compatibility between modules
    pub fn validate_compatibility(
        analysis_enabled: bool,
        output_format: &str,
        ai_enabled: bool,
    ) -> Result<(), UveddiError> {
        // Check if AI is enabled but analysis is disabled
        if ai_enabled && !analysis_enabled {
            return Err(UveddiError::config_error(
                "AI analysis cannot be enabled without main analysis",
                "compatibility validation",
            ));
        }

        // Check if HTML output is requested with incompatible settings
        if output_format == "html" && !analysis_enabled {
            return Err(UveddiError::config_error(
                "HTML output format requires analysis to be enabled",
                "compatibility validation",
            ));
        }

        Ok(())
    }
}

/// Validation result aggregator for collecting multiple validation errors
pub struct ValidationResult {
    errors: Vec<UveddiError>,
}

impl ValidationResult {
    /// Create a new validation result
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
        }
    }

    /// Add a validation error
    pub fn add_error(&mut self, error: UveddiError) {
        self.errors.push(error);
    }

    /// Add a validation result from a function
    pub fn add_result(&mut self, result: Result<(), UveddiError>) {
        if let Err(error) = result {
            self.errors.push(error);
        }
    }

    /// Check if there are any validation errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Get the first error if any exist
    pub fn into_result(self) -> Result<(), UveddiError> {
        if let Some(first_error) = self.errors.into_iter().next() {
            Err(first_error)
        } else {
            Ok(())
        }
    }

    /// Get all validation errors
    pub fn into_errors(self) -> Vec<UveddiError> {
        self.errors
    }

    /// Combine multiple validation errors into a single error
    pub fn into_combined_result(self) -> Result<(), UveddiError> {
        if self.errors.is_empty() {
            Ok(())
        } else if self.errors.len() == 1 {
            Err(self.errors.into_iter().next().unwrap())
        } else {
            let error_messages: Vec<String> = self.errors
                .into_iter()
                .map(|e| e.to_string())
                .collect();
            Err(UveddiError::config_error(
                &format!("Multiple configuration errors: {}", error_messages.join("; ")),
                "combined validation",
            ))
        }
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}