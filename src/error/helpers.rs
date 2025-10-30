use crate::analysis::errors::AnalysisError;

/// Utility functions for standardized error creation across detectors
///
/// Provides consistent error message formatting and context for common
/// error scenarios in the analysis pipeline.
pub struct ErrorHelpers;

impl ErrorHelpers {
    /// Create a standardized query error for AST operations
    ///
    /// # Arguments
    /// * `message` - Specific error description
    ///
    /// # Returns
    /// Formatted AnalysisError with consistent "Query error:" prefix
    pub fn query_error(message: &str) -> AnalysisError {
        AnalysisError::DetectionError(format!("Query error: {}", message))
    }

    /// Create a standardized AST error for missing tree operations
    ///
    /// # Arguments
    /// * `operation` - The operation that failed (e.g., "parsing", "traversal")
    ///
    /// # Returns
    /// Formatted AnalysisError indicating missing AST tree
    pub fn ast_error(operation: &str) -> AnalysisError {
        AnalysisError::DetectionError(format!("AST {}: tree missing", operation))
    }

    /// Create a standardized file processing error
    ///
    /// # Arguments
    /// * `file_path` - Path to the file that failed processing
    /// * `operation` - The operation that failed
    /// * `cause` - The underlying error cause
    ///
    /// # Returns
    /// Formatted AnalysisError with file context
    pub fn file_processing_error(file_path: &str, operation: &str, cause: &str) -> AnalysisError {
        AnalysisError::DetectionError(format!(
            "File processing failed during {}: {} ({})",
            operation, file_path, cause
        ))
    }

    /// Create a standardized detector configuration error
    ///
    /// # Arguments
    /// * `detector_name` - Name of the detector with invalid config
    /// * `config_issue` - Description of the configuration problem
    ///
    /// # Returns
    /// Formatted AnalysisError for configuration issues
    pub fn detector_config_error(detector_name: &str, config_issue: &str) -> AnalysisError {
        AnalysisError::DetectionError(format!(
            "Detector '{}' configuration error: {}",
            detector_name, config_issue
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_error() {
        let error = ErrorHelpers::query_error("invalid syntax");
        match error {
            AnalysisError::DetectionError(msg) => {
                assert_eq!(msg, "Query error: invalid syntax");
            }
            other => panic!("Expected DetectionError, got: {:?}", other),
        }
    }

    #[test]
    fn test_ast_error() {
        let error = ErrorHelpers::ast_error("parsing");
        match error {
            AnalysisError::DetectionError(msg) => {
                assert_eq!(msg, "AST parsing: tree missing");
            }
            other => panic!("Expected DetectionError, got: {:?}", other),
        }
    }

    #[test]
    fn test_file_processing_error() {
        let error = ErrorHelpers::file_processing_error(
            "/path/to/file.rs",
            "analysis",
            "permission denied",
        );
        match error {
            AnalysisError::DetectionError(msg) => {
                assert!(msg.contains("File processing failed during analysis"));
                assert!(msg.contains("/path/to/file.rs"));
                assert!(msg.contains("permission denied"));
            }
            other => panic!("Expected DetectionError, got: {:?}", other),
        }
    }

    #[test]
    fn test_detector_config_error() {
        let error = ErrorHelpers::detector_config_error("god_object", "invalid threshold");
        match error {
            AnalysisError::DetectionError(msg) => {
                assert_eq!(
                    msg,
                    "Detector 'god_object' configuration error: invalid threshold"
                );
            }
            other => panic!("Expected DetectionError, got: {:?}", other),
        }
    }
}
