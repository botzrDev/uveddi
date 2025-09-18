//! Comprehensive Input Validation Security Tests
//!
//! This test suite validates the security enhancements added to harden input validation
//! across the Uveddi application. It tests validation functions, CLI argument validation,
//! API request validation, configuration file validation, and edge cases.

use std::fs;
use std::path::Path;
use tempfile::TempDir;
use uveddi::security::{
    sanitize_for_web_display, validate_api_request, validate_cli_argument,
    validate_config_file_path, validate_db_parameter, validate_external_api_response,
    validate_input, validate_json_input, validate_memory_limit, validate_url, CliArgumentType,
    DbParameterType, SecurityError,
};

#[cfg(test)]
mod security_validation_tests {
    use super::*;

    #[test]
    fn test_basic_input_validation() {
        // Valid inputs should pass
        assert!(validate_input("normal text", "test_field").is_ok());
        assert!(validate_input("file_name.rs", "filename").is_ok());
        assert!(validate_input("user123", "username").is_ok());

        // SQL injection patterns should be rejected
        assert!(validate_input("'; DROP TABLE users; --", "malicious").is_err());
        assert!(validate_input("UNION SELECT * FROM passwords", "injection").is_err());
        assert!(validate_input("/**/", "comment_injection").is_err());
        assert!(validate_input("xp_cmdshell", "stored_proc").is_err());
        assert!(validate_input("javascript:alert(1)", "xss_attempt").is_err());

        // Oversized input should be rejected
        let large_input = "x".repeat(20000);
        assert!(validate_input(&large_input, "oversized").is_err());

        // Empty input should be valid
        assert!(validate_input("", "empty").is_ok());
    }

    #[test]
    fn test_url_validation() {
        // Valid URLs should pass
        assert!(validate_url("http://localhost:11434").is_ok());
        assert!(validate_url("https://api.example.com/v1").is_ok());
        assert!(validate_url("http://127.0.0.1:8080/health").is_ok());

        // Invalid URLs should be rejected
        assert!(validate_url("ftp://example.com").is_err());
        assert!(validate_url("javascript:alert(1)").is_err());
        assert!(validate_url("file:///etc/passwd").is_err());
        assert!(validate_url("").is_err());
        assert!(validate_url("not-a-url").is_err());

        // URLs with dangerous characters should be rejected
        assert!(validate_url("http://example.com/../etc/passwd").is_err());
        assert!(validate_url("http://example.com\\malicious").is_err());
        assert!(validate_url("http://example.com\0").is_err());
    }

    #[test]
    fn test_cli_argument_validation() {
        // File path validation
        assert!(
            validate_cli_argument("/valid/path/file.rs", "path", CliArgumentType::FilePath).is_ok()
        );
        assert!(validate_cli_argument("$(malicious)", "path", CliArgumentType::FilePath).is_err());
        assert!(validate_cli_argument("`command`", "path", CliArgumentType::FilePath).is_err());

        // Port validation
        assert!(validate_cli_argument("8080", "port", CliArgumentType::Port).is_ok());
        assert!(validate_cli_argument("65535", "port", CliArgumentType::Port).is_ok());
        assert!(validate_cli_argument("1023", "port", CliArgumentType::Port).is_err()); // System port
        assert!(validate_cli_argument("abc", "port", CliArgumentType::Port).is_err());
        assert!(validate_cli_argument("99999", "port", CliArgumentType::Port).is_err());

        // Percentage validation
        assert!(validate_cli_argument("50.5", "percentage", CliArgumentType::Percentage).is_ok());
        assert!(validate_cli_argument("0.0", "percentage", CliArgumentType::Percentage).is_ok());
        assert!(validate_cli_argument("100.0", "percentage", CliArgumentType::Percentage).is_ok());
        assert!(validate_cli_argument("-1.0", "percentage", CliArgumentType::Percentage).is_err());
        assert!(validate_cli_argument("101.0", "percentage", CliArgumentType::Percentage).is_err());

        // Count validation
        assert!(validate_cli_argument("100", "count", CliArgumentType::Count).is_ok());
        assert!(validate_cli_argument("0", "count", CliArgumentType::Count).is_ok());
        assert!(validate_cli_argument("1000001", "count", CliArgumentType::Count).is_err()); // Too large
        assert!(validate_cli_argument("-1", "count", CliArgumentType::Count).is_err());

        // Generic validation
        assert!(validate_cli_argument("normal-text", "generic", CliArgumentType::Generic).is_ok());
        assert!(
            validate_cli_argument("'; DROP TABLE", "generic", CliArgumentType::Generic).is_err()
        );
    }

    #[test]
    fn test_api_request_validation() {
        // Valid requests should pass
        assert!(
            validate_api_request(Some("application/json"), Some(1024), Some("curl/7.68.0")).is_ok()
        );

        assert!(validate_api_request(None, None, None).is_ok());

        // Invalid content types should be rejected
        assert!(validate_api_request(Some("text/html"), None, None).is_err());

        assert!(validate_api_request(Some("application/x-evil"), None, None).is_err());

        // Oversized requests should be rejected
        assert!(validate_api_request(
            Some("application/json"),
            Some(20 * 1024 * 1024), // 20MB, over 10MB limit
            None
        )
        .is_err());

        // Content type with parameters should work
        assert!(
            validate_api_request(Some("application/json; charset=utf-8"), Some(1024), None).is_ok()
        );

        // Suspicious user agents should be logged but not blocked
        assert!(validate_api_request(None, None, Some("malicious-bot/1.0")).is_ok());
    }

    #[test]
    fn test_config_file_path_validation() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let config_path = temp_dir.path().join("test.toml");

        // Create a test config file
        fs::write(&config_path, "test = true")?;

        // Valid config file should pass
        assert!(validate_config_file_path(
            &config_path.to_string_lossy(),
            Some(&[temp_dir.path().to_str().unwrap()])
        )
        .is_ok());

        // Invalid extensions should be rejected
        let bad_path = temp_dir.path().join("test.exe");
        fs::write(&bad_path, "test")?;
        assert!(validate_config_file_path(
            &bad_path.to_string_lossy(),
            Some(&[temp_dir.path().to_str().unwrap()])
        )
        .is_err());

        // Files outside allowed directories should be rejected
        let outside_path = "/tmp/malicious.toml";
        assert!(validate_config_file_path(
            outside_path,
            Some(&[temp_dir.path().to_str().unwrap()])
        )
        .is_err());

        // Very large config files should be rejected
        let large_config = temp_dir.path().join("large.toml");
        let large_content = "x".repeat(2 * 1024 * 1024); // 2MB, over 1MB limit
        fs::write(&large_config, large_content)?;

        assert!(validate_config_file_path(
            &large_config.to_string_lossy(),
            Some(&[temp_dir.path().to_str().unwrap()])
        )
        .is_err());

        Ok(())
    }

    #[test]
    fn test_json_input_validation() {
        // Valid JSON should pass
        let simple_json = r#"{"name": "test", "value": 42}"#;
        assert!(validate_json_input(simple_json, None, None).is_ok());

        // Invalid JSON should be rejected
        let invalid_json = r#"{"name": "test", "value": 42"#; // Missing closing brace
        assert!(validate_json_input(invalid_json, None, None).is_err());

        // Oversized JSON should be rejected
        let large_json = format!(r#"{{"data": "{}"}}"#, "x".repeat(2 * 1024 * 1024));
        assert!(validate_json_input(&large_json, None, Some(1024)).is_err());

        // Deeply nested JSON should be rejected
        let deep_json = "{".repeat(20) + "\"value\": true" + &"}".repeat(20);
        assert!(validate_json_input(&deep_json, Some(10), None).is_err());

        // SQL injection in JSON values should be detected
        let malicious_json = r#"{"query": "'; DROP TABLE users; --"}"#;
        assert!(validate_json_input(malicious_json, None, None).is_err());
    }

    #[test]
    fn test_database_parameter_validation() {
        // Valid ID parameters
        assert!(validate_db_parameter("123", "id", DbParameterType::Id).is_ok());
        assert!(validate_db_parameter("-1", "id", DbParameterType::Id).is_ok());

        // Invalid ID parameters
        assert!(validate_db_parameter("abc", "id", DbParameterType::Id).is_err());
        assert!(validate_db_parameter("12.5", "id", DbParameterType::Id).is_err());

        // Text parameters
        assert!(validate_db_parameter("normal text", "text", DbParameterType::Text).is_ok());
        assert!(validate_db_parameter("'; DROP TABLE", "text", DbParameterType::Text).is_err());

        // File paths
        assert!(
            validate_db_parameter("/path/to/file.rs", "path", DbParameterType::FilePath).is_ok()
        );
        assert!(
            validate_db_parameter("../../../etc/passwd", "path", DbParameterType::FilePath).is_ok()
        ); // Allowed for storage

        // Code content (more permissive)
        assert!(validate_db_parameter(
            "fn main() { println!(\"Hello\"); }",
            "code",
            DbParameterType::CodeContent
        )
        .is_ok());
        assert!(validate_db_parameter(
            "SELECT * FROM table;",
            "code",
            DbParameterType::CodeContent
        )
        .is_ok()); // SQL allowed in code

        // Timestamps
        assert!(validate_db_parameter(
            "2023-01-01T12:00:00Z",
            "timestamp",
            DbParameterType::Timestamp
        )
        .is_ok());
        assert!(
            validate_db_parameter("invalid-date", "timestamp", DbParameterType::Timestamp).is_err()
        );
    }

    #[test]
    fn test_external_api_response_validation() {
        // Valid responses should pass
        let valid_response = "This is a normal API response with some content.";
        assert!(validate_external_api_response(valid_response, None).is_ok());

        // Responses with null bytes should be rejected
        let null_byte_response = "Response with \0 null byte";
        assert!(validate_external_api_response(null_byte_response, None).is_err());

        // Responses with excessive control characters should be rejected
        let control_char_response = "\x01\x02\x03".repeat(100) + "normal text";
        assert!(validate_external_api_response(&control_char_response, None).is_err());

        // Responses with dangerous HTML should be rejected
        let html_response = "<script>alert('xss')</script>";
        assert!(validate_external_api_response(html_response, None).is_err());

        let iframe_response = "<iframe src='javascript:alert(1)'></iframe>";
        assert!(validate_external_api_response(iframe_response, None).is_err());

        // Oversized responses should be rejected
        let large_response = "x".repeat(2 * 1024 * 1024);
        assert!(validate_external_api_response(&large_response, Some(1024)).is_err());

        // Normal code content should pass
        let code_response = "fn main() {\n    println!(\"Hello, world!\");\n}";
        assert!(validate_external_api_response(code_response, None).is_ok());
    }

    #[test]
    fn test_memory_limit_validation() {
        // Valid memory limits
        assert!(validate_memory_limit(256, "memory").is_ok()); // 256MB
        assert!(validate_memory_limit(1024, "memory").is_ok()); // 1GB
        assert!(validate_memory_limit(4096, "memory").is_ok()); // 4GB

        // Memory limits that are too small
        assert!(validate_memory_limit(32, "memory").is_err()); // 32MB, below 64MB minimum

        // Memory limits that are too large
        assert!(validate_memory_limit(16384, "memory").is_err()); // 16GB, above 8GB maximum
    }

    #[test]
    fn test_web_display_sanitization() {
        // Basic HTML escaping
        assert_eq!(
            sanitize_for_web_display("Hello <script>alert('xss')</script>"),
            "Hello &lt;script&gt;alert(&#x27;xss&#x27;)&lt;&#x2F;script&gt;"
        );

        // Ampersand escaping
        assert_eq!(sanitize_for_web_display("Tom & Jerry"), "Tom &amp; Jerry");

        // Quote escaping
        assert_eq!(
            sanitize_for_web_display("He said \"Hello\" to 'everyone'."),
            "He said &quot;Hello&quot; to &#x27;everyone&#x27;."
        );

        // Normal text should remain unchanged
        assert_eq!(
            sanitize_for_web_display("Normal text with spaces"),
            "Normal text with spaces"
        );
    }

    #[test]
    fn test_edge_cases_and_boundary_conditions() {
        // Empty strings
        assert!(validate_input("", "empty").is_ok());
        assert!(validate_api_request(None, Some(0), None).is_ok());

        // Boundary values for memory
        assert!(validate_memory_limit(64, "memory").is_ok()); // Minimum
        assert!(validate_memory_limit(8192, "memory").is_ok()); // Maximum
        assert!(validate_memory_limit(63, "memory").is_err()); // Below minimum
        assert!(validate_memory_limit(8193, "memory").is_err()); // Above maximum

        // Unicode handling
        assert!(validate_input("用户名", "unicode").is_ok());
        assert!(validate_input("🚀 rocket", "emoji").is_ok());

        // Very long but valid strings
        let long_valid = "a".repeat(9999);
        assert!(validate_input(&long_valid, "long_valid").is_ok());

        // Just over the limit
        let just_over_limit = "a".repeat(10001);
        assert!(validate_input(&just_over_limit, "over_limit").is_err());
    }

    #[test]
    fn test_cascading_validation_failures() {
        // Test that validation fails early and provides meaningful errors
        let result = validate_input("'; DROP TABLE users; --", "malicious");
        assert!(result.is_err());

        if let Err(SecurityError::InvalidInput { field, reason }) = result {
            assert_eq!(field, "malicious");
            assert!(reason.contains("dangerous SQL patterns"));
        } else {
            panic!("Expected InvalidInput error with SQL injection reason");
        }

        // Test URL validation error messages
        let result = validate_url("javascript:alert(1)");
        assert!(result.is_err());

        if let Err(SecurityError::InvalidInput { field, reason }) = result {
            assert_eq!(field, "url");
            assert!(reason.contains("http://") || reason.contains("https://"));
        } else {
            panic!("Expected InvalidInput error for invalid URL protocol");
        }
    }

    #[test]
    fn test_configuration_integration() {
        // Test that security validation integrates properly with configuration loading
        use tempfile::TempDir;
        use uveddi::config::Config;

        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test.toml");

        // Valid configuration should work
        let valid_config = r#"
ollama_model = "deepseek-coder:6.7b"

[dead_code]
confidence_threshold = 0.8
library_mode = true
ignore_patterns = ["test", "spec"]

[large_classes]
max_logical_loc = 500
max_methods = 25
"#;
        fs::write(&config_path, valid_config).unwrap();

        let result = Config::from_file(&config_path.to_string_lossy());
        assert!(result.is_ok());

        // Configuration with invalid values should be rejected
        let invalid_config = r#"
ollama_model = "'; DROP TABLE models; --"
"#;
        let invalid_path = temp_dir.path().join("invalid.toml");
        fs::write(&invalid_path, invalid_config).unwrap();

        let result = Config::from_file(&invalid_path.to_string_lossy());
        assert!(result.is_err());
    }

    #[test]
    fn test_performance_with_large_inputs() {
        use std::time::Instant;

        // Test that validation is reasonably fast even with large inputs
        let large_input = "normal_text_".repeat(1000);

        let start = Instant::now();
        let result = validate_input(&large_input, "performance_test");
        let duration = start.elapsed();

        assert!(result.is_ok());
        assert!(
            duration.as_millis() < 100,
            "Validation took too long: {:?}",
            duration
        );

        // Test with many small validations
        let start = Instant::now();
        for i in 0..1000 {
            let input = format!("test_input_{}", i);
            assert!(validate_input(&input, "bulk_test").is_ok());
        }
        let duration = start.elapsed();

        assert!(
            duration.as_millis() < 500,
            "Bulk validation took too long: {:?}",
            duration
        );
    }
}
