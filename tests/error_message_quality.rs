use uveddi::error::{ErrorCategory, ErrorSeverity, UveddiError};

/// Tests that verify error message quality and actionability
#[cfg(test)]
mod error_message_quality_tests {
    use super::*;

    #[test]
    fn test_config_error_formatting() {
        let error =
            UveddiError::config_error("Invalid Ollama endpoint configuration", "config.toml:15");

        let message = format!("{}", error);

        // Verify error message contains helpful information
        assert!(message.contains("Configuration error"));
        assert!(message.contains("Suggestion:"));
        assert!(message.contains("Ollama is running"));
        assert!(message.contains("Location: config.toml:15"));

        // Verify categorization
        assert_eq!(error.category(), ErrorCategory::Configuration);
        assert_eq!(error.severity(), ErrorSeverity::Low);
    }

    #[test]
    fn test_io_error_context() {
        let io_error =
            std::io::Error::new(std::io::ErrorKind::NotFound, "No such file or directory");

        let error = UveddiError::io_error(
            "reading configuration",
            "/nonexistent/config.toml",
            io_error,
        );

        let message = format!("{}", error);

        assert!(message.contains("File system error"));
        assert!(message.contains("reading configuration"));
        assert!(message.contains("/nonexistent/config.toml"));
        assert!(message.contains("Suggestion:"));
        assert!(message.contains("Create the required file"));
    }

    #[tokio::test]
    async fn test_network_error_suggestions() {
        // Mock reqwest error for testing
        let mock_url = "http://localhost:11434/api/generate";

        // Test timeout scenario
        let timeout_error = reqwest::ClientBuilder::new()
            .timeout(std::time::Duration::from_millis(1))
            .build()
            .unwrap()
            .get("http://httpbin.org/delay/10")
            .send()
            .await
            .unwrap_err();

        let error = UveddiError::network_error("AI inference request", mock_url, timeout_error);

        let message = format!("{}", error);
        assert!(message.contains("Network request failed"));
        assert!(message.contains("AI inference request"));
        assert!(message.contains("localhost:11434"));
        assert!(message.contains("Suggestion:"));
    }

    #[test]
    fn test_database_error_recovery_hints() {
        let sqlite_error = rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(rusqlite::ErrorCode::DatabaseLocked as i32),
            Some("database is locked".to_string()),
        );

        let error = UveddiError::database_error(
            "inserting analysis results",
            "uveddi_cache.db",
            sqlite_error,
        );

        let message = format!("{}", error);

        assert!(message.contains("Database operation failed"));
        assert!(message.contains("inserting analysis results"));
        assert!(message.contains("uveddi_cache.db"));
        assert!(message.contains("Recovery:"));
        assert!(message.contains("Close other database connections"));
    }

    #[test]
    fn test_analysis_error_suggestions() {
        let error = UveddiError::analysis_error(
            "src/large_file.rs",
            42,
            "Memory allocation failed during parsing",
            "File size exceeds memory limits",
        );

        let message = format!("{}", error);

        assert!(message.contains("Analysis error in src/large_file.rs:42"));
        assert!(message.contains("Memory allocation failed"));
        assert!(message.contains("Context: File size exceeds memory limits"));
        assert!(message.contains("Suggestion:"));
        assert!(message.contains("memory limits"));
    }

    #[test]
    fn test_error_message_actionability() {
        // Test that all error message helper functions provide actionable guidance
        let errors = vec![
            UveddiError::config_error("Unknown feature flag", "Cargo.toml:25"),
            UveddiError::config_error("Ollama connection failed", "runtime"),
            UveddiError::analysis_error("test.rs", 1, "Parse error", "complex syntax"),
        ];

        for error in errors {
            let message = format!("{}", error);

            // Every error should have a suggestion
            assert!(message.contains("Suggestion:"));

            // Suggestions should be actionable (contain verbs)
            assert!(
                message.contains("Check")
                    || message.contains("Ensure")
                    || message.contains("Verify")
                    || message.contains("Install")
                    || message.contains("Update")
                    || message.contains("Enable")
                    || message.contains("Create")
                    || message.contains("Reduce")
                    || message.contains("Increase")
                    || message.contains("Simplify")
            );
        }
    }

    #[test]
    fn test_error_message_consistency() {
        // Test that similar errors have consistent formatting
        let config_errors = vec![
            UveddiError::config_error("Missing field", "config.toml:10"),
            UveddiError::config_error("Invalid value", "config.toml:20"),
        ];

        for error in config_errors {
            let message = format!("{}", error);
            assert!(message.starts_with("Configuration error:"));
            assert!(message.contains("Location:"));
            assert!(message.contains("Suggestion:"));
        }
    }

    #[test]
    fn test_error_severity_classification() {
        // Test that errors are properly classified by severity
        let high_severity_errors = vec![
            UveddiError::analysis_error("test.rs", 1, "Critical failure", "system"),
            UveddiError::database_error(
                "query",
                "db",
                rusqlite::Error::SqliteFailure(
                    rusqlite::ffi::Error::new(rusqlite::ErrorCode::DatabaseCorrupt as i32),
                    None,
                ),
            ),
        ];

        for error in high_severity_errors {
            assert_eq!(error.severity(), ErrorSeverity::High);
        }

        let low_severity_errors = vec![UveddiError::config_error("Minor issue", "config")];

        for error in low_severity_errors {
            assert_eq!(error.severity(), ErrorSeverity::Low);
        }
    }

    #[test]
    fn test_error_categorization() {
        let test_cases = vec![
            (
                UveddiError::config_error("test", "location"),
                ErrorCategory::Configuration,
            ),
            (
                UveddiError::io_error(
                    "test",
                    "path",
                    std::io::Error::new(std::io::ErrorKind::NotFound, "test"),
                ),
                ErrorCategory::Io,
            ),
            (
                UveddiError::analysis_error("file", 1, "message", "context"),
                ErrorCategory::Analysis,
            ),
        ];

        for (error, expected_category) in test_cases {
            assert_eq!(error.category(), expected_category);
        }
    }

    #[test]
    fn test_suggestion_quality() {
        // Test specific suggestion quality for different error types

        // Ollama connection error
        let ollama_error =
            UveddiError::config_error("Failed to connect to Ollama service", "runtime");
        let message = format!("{}", ollama_error);
        assert!(message.contains("localhost:11434"));

        // Feature flag error
        let feature_error =
            UveddiError::config_error("Feature 'tree-sitter' not enabled", "Cargo.toml");
        let message = format!("{}", feature_error);
        assert!(message.contains("Cargo.toml"));

        // Permission error
        let io_error =
            std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Permission denied");
        let perm_error = UveddiError::io_error("writing", "/protected/file", io_error);
        let message = format!("{}", perm_error);
        assert!(message.contains("permissions") || message.contains("privileges"));
    }

    #[test]
    fn test_error_message_length() {
        // Test that error messages are not too verbose but contain essential info
        let error = UveddiError::config_error("Test error", "test.toml:1");
        let message = format!("{}", error);

        // Should be informative but not excessively long
        assert!(message.len() > 50); // Has substance
        assert!(message.len() < 500); // Not too verbose

        // Should have proper line breaks for readability
        assert!(message.contains('\n'));
    }

    #[test]
    fn test_nested_error_information() {
        // Test that source error information is preserved and accessible
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "Original error message");

        let wrapped_error = UveddiError::io_error("test operation", "/test/path", io_error);

        // Should be able to access the source error
        let source = std::error::Error::source(&wrapped_error);
        assert!(source.is_some());

        // The formatted message should include context from both levels
        let message = format!("{}", wrapped_error);
        assert!(message.contains("test operation"));
        assert!(message.contains("/test/path"));
    }

    /// Tests error message formatting in different scenarios
    #[test]
    fn test_error_message_scenarios() {
        struct TestCase {
            name: &'static str,
            error: UveddiError,
            should_contain: Vec<&'static str>,
            should_not_contain: Vec<&'static str>,
        }

        let test_cases = vec![
            TestCase {
                name: "Local service connection failure",
                error: UveddiError::config_error(
                    "Cannot connect to Ollama service",
                    "ai_config.toml:5",
                ),
                should_contain: vec!["localhost:11434", "running", "Suggestion:"],
                should_not_contain: vec!["internet", "DNS"],
            },
            TestCase {
                name: "Missing file analysis",
                error: UveddiError::analysis_error(
                    "missing_file.rs",
                    0,
                    "File not found during analysis",
                    "Project structure scan",
                ),
                should_contain: vec!["missing_file.rs", "Context:", "Suggestion:"],
                should_not_contain: vec!["memory", "timeout"],
            },
        ];

        for test_case in test_cases {
            let message = format!("{}", test_case.error);

            for should_contain in &test_case.should_contain {
                assert!(
                    message.contains(should_contain),
                    "Test '{}': Message should contain '{}'\nActual message: {}",
                    test_case.name,
                    should_contain,
                    message
                );
            }

            for should_not_contain in &test_case.should_not_contain {
                assert!(
                    !message.contains(should_not_contain),
                    "Test '{}': Message should not contain '{}'\nActual message: {}",
                    test_case.name,
                    should_not_contain,
                    message
                );
            }
        }
    }
}

/// Integration tests for error handling in real scenarios
#[cfg(test)]
mod error_integration_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_file_permission_error_integration() {
        let temp_dir = TempDir::new().unwrap();
        let protected_file = temp_dir.path().join("protected.txt");

        // Create a file and make it read-only
        fs::write(&protected_file, "test content").unwrap();
        let mut perms = fs::metadata(&protected_file).unwrap().permissions();
        perms.set_readonly(true);
        fs::set_permissions(&protected_file, perms).unwrap();

        // Try to write to the read-only file
        let result = fs::write(&protected_file, "new content");

        if let Err(io_error) = result {
            let error = UveddiError::io_error(
                "updating analysis cache",
                &protected_file.display().to_string(),
                io_error,
            );

            let message = format!("{}", error);
            assert!(message.contains("permissions") || message.contains("privileges"));
            assert!(message.contains("updating analysis cache"));
        }
    }

    #[test]
    fn test_configuration_error_integration() {
        // Simulate a common configuration error scenario
        let error = UveddiError::config_error(
            "tree-sitter feature required but not enabled",
            "detected at runtime",
        );

        let message = format!("{}", error);

        // Should provide specific guidance for this common issue
        assert!(message.contains("Cargo.toml"));
        assert!(message.contains("features"));

        // Should indicate severity appropriately
        assert_eq!(error.severity(), ErrorSeverity::Low);
        assert_eq!(error.category(), ErrorCategory::Configuration);
    }
}
