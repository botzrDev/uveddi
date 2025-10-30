use uveddi::analysis::errors::AnalysisError;
use uveddi::error::{ErrorHelpers, UveddiError};

#[test]
fn test_error_helpers_integration() {
    // Test that ErrorHelpers integrates properly with UveddiError
    let analysis_error = ErrorHelpers::query_error("test message");

    // Should be convertible to UveddiError
    let uveddi_error: UveddiError = analysis_error.into();

    // Verify error message formatting
    let error_string = format!("{}", uveddi_error);
    assert!(error_string.contains("Query error: test message"));
}

#[test]
fn test_error_helpers_consistency() {
    // Verify all helper methods produce consistent error formats
    let query_err = ErrorHelpers::query_error("test");
    let ast_err = ErrorHelpers::ast_error("test");
    let file_err = ErrorHelpers::file_processing_error("test.rs", "parse", "syntax");
    let config_err = ErrorHelpers::detector_config_error("detector", "invalid");

    // All should be DetectionError variants
    assert!(matches!(query_err, AnalysisError::DetectionError(_)));
    assert!(matches!(ast_err, AnalysisError::DetectionError(_)));
    assert!(matches!(file_err, AnalysisError::DetectionError(_)));
    assert!(matches!(config_err, AnalysisError::DetectionError(_)));
}
