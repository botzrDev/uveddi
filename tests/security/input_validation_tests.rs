//! Comprehensive input validation tests for UV-273
//! 
//! This test suite validates all input validation utilities to prevent injection attacks
//! and ensure comprehensive security coverage across all external input points.

use uveddi::security::{
    validate_input, validate_url, validate_model_name, validate_numeric_range,
    validate_character_set, SecurityError
};
use uveddi::database::Database;
use tempfile::TempDir;

#[cfg(test)]
mod input_validation_tests {
    use super::*;

    #[test]
    fn test_sql_injection_prevention() {
        let malicious_inputs = vec![
            "'; DROP TABLE users; --",
            "1; DELETE FROM analysis_runs; --",
            "test' UNION SELECT * FROM users --",
            "admin'/**/OR/**/1=1/**/--",
            "'; EXEC xp_cmdshell('dir'); --",
            "test' AND (SELECT COUNT(*) FROM users) > 0 --",
            "' OR 1=1 --",
            "1' OR '1'='1",
            "'; TRUNCATE TABLE analysis_runs; --",
            "test'; INSERT INTO users VALUES ('hacker', 'password'); --",
        ];

        for input in malicious_inputs {
            let result = validate_input(input, "test_field");
            assert!(result.is_err(), "Should reject SQL injection: {}", input);
            
            if let Err(SecurityError::InvalidInput { reason, .. }) = result {
                assert!(reason.contains("dangerous SQL patterns"));
            } else {
                panic!("Expected InvalidInput error for SQL injection: {}", input);
            }
        }
    }

    #[test]
    fn test_safe_sql_patterns_allowed() {
        let safe_inputs = vec![
            "normal file path.txt",
            "project-name",
            "valid_identifier",
            "user@example.com",
            "description with spaces",
            "file.rs",
            "my-project/src/main.rs",
        ];

        for input in safe_inputs {
            let result = validate_input(input, "test_field");
            assert!(result.is_ok(), "Should accept safe input: {}", input);
        }
    }

    #[test]
    fn test_length_validation() {
        // Test normal length
        let normal_input = "a".repeat(1000);
        assert!(validate_input(&normal_input, "test").is_ok());

        // Test at the limit
        let at_limit = "a".repeat(10000);
        assert!(validate_input(&at_limit, "test").is_ok());

        // Test over the limit
        let over_limit = "a".repeat(10001);
        let result = validate_input(&over_limit, "test");
        assert!(result.is_err());
        
        if let Err(SecurityError::InvalidInput { reason, .. }) = result {
            assert!(reason.contains("exceeds maximum"));
        }
    }

    #[test]
    fn test_url_validation() {
        // Valid URLs
        let valid_urls = vec![
            "http://localhost:11434",
            "https://api.example.com",
            "http://127.0.0.1:8080/api",
            "https://ollama.example.com:443/v1",
            "http://192.168.1.100:11434",
        ];

        for url in valid_urls {
            assert!(validate_url(url).is_ok(), "Should accept valid URL: {}", url);
        }

        // Invalid URLs
        let invalid_urls = vec![
            "ftp://example.com",
            "javascript:alert(1)",
            "http://example.com/../../../etc/passwd",
            "https://example.com\0",
            "",
            "not-a-url",
            "http://",
            "https://",
            "http://example.com\\malicious",
        ];

        for url in invalid_urls {
            assert!(validate_url(url).is_err(), "Should reject invalid URL: {}", url);
        }
    }

    #[test]
    fn test_model_name_validation() {
        // Valid model names
        let valid_names = vec![
            "deepseek-coder",
            "llama2-7b",
            "codellama",
            "mistral-7b-instruct",
            "gpt-3.5-turbo",
            "claude-3-sonnet",
        ];

        for name in valid_names {
            assert!(validate_model_name(name).is_ok(), "Should accept valid model: {}", name);
        }

        // Invalid model names
        let invalid_names = vec![
            "",
            "model/with/slashes",
            "model..with..dots",
            "model\\with\\backslashes",
            "model\0with\0nulls",
            &"a".repeat(200), // Too long
            "model with spaces",
            "../malicious-model",
            "model;DROP TABLE",
        ];

        for name in invalid_names {
            assert!(validate_model_name(name).is_err(), "Should reject invalid model: {}", name);
        }
    }

    #[test]
    fn test_numeric_range_validation() {
        // Valid ranges
        assert!(validate_numeric_range(50, 0, 100, "confidence").is_ok());
        assert!(validate_numeric_range(0, 0, 100, "confidence").is_ok());
        assert!(validate_numeric_range(100, 0, 100, "confidence").is_ok());
        assert!(validate_numeric_range(1, 1, 1000, "loc").is_ok());
        assert!(validate_numeric_range(1000, 1, 1000, "loc").is_ok());

        // Invalid ranges
        assert!(validate_numeric_range(-1, 0, 100, "confidence").is_err());
        assert!(validate_numeric_range(101, 0, 100, "confidence").is_err());
        assert!(validate_numeric_range(0, 1, 1000, "loc").is_err());
        assert!(validate_numeric_range(1001, 1, 1000, "loc").is_err());
    }

    #[test]
    fn test_character_set_validation() {
        let alphanumeric = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        let alphanumeric_with_dash = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-";
        
        // Valid input
        assert!(validate_character_set("abc123", "test", alphanumeric).is_ok());
        assert!(validate_character_set("test-input", "test", alphanumeric_with_dash).is_ok());
        assert!(validate_character_set("", "test", alphanumeric).is_ok()); // Empty string should be valid
        
        // Invalid input
        assert!(validate_character_set("abc@123", "test", alphanumeric).is_err());
        assert!(validate_character_set("test-input", "test", alphanumeric).is_err());
        assert!(validate_character_set("test input", "test", alphanumeric).is_err()); // Space not allowed
    }

    #[test]
    fn test_edge_cases() {
        // Empty strings
        assert!(validate_input("", "test").is_ok());
        
        // Unicode characters
        assert!(validate_input("测试", "test").is_ok());
        assert!(validate_input("файл.txt", "test").is_ok());
        assert!(validate_input("🔒secure", "test").is_ok());
        
        // Special characters that should be allowed
        assert!(validate_input("file_name.txt", "test").is_ok());
        assert!(validate_input("path/to/file", "test").is_ok());
        assert!(validate_input("config.json", "test").is_ok());
        assert!(validate_input("my-project", "test").is_ok());
    }

    #[test]
    fn test_performance_with_large_inputs() {
        use std::time::Instant;
        
        let large_input = "a".repeat(9999); // Just under the limit
        let start = Instant::now();
        let result = validate_input(&large_input, "test");
        let duration = start.elapsed();
        
        assert!(result.is_ok());
        assert!(duration.as_millis() < 100, "Validation should be fast even for large inputs");
        
        // Test with many patterns
        let pattern_input = "select union insert update delete drop exec execute script".repeat(100);
        let start = Instant::now();
        let result = validate_input(&pattern_input, "test");
        let duration = start.elapsed();
        
        assert!(result.is_err()); // Should detect SQL patterns
        assert!(duration.as_millis() < 100, "Pattern detection should be fast");
    }

    #[test]
    fn test_case_insensitive_sql_detection() {
        let case_variants = vec![
            "SELECT * FROM users",
            "select * from users",
            "SeLeCt * FrOm UsErS",
            "UNION ALL SELECT",
            "union all select",
            "UnIoN aLl SeLeCt",
            "DROP TABLE",
            "drop table",
            "DrOp TaBlE",
        ];

        for input in case_variants {
            let result = validate_input(input, "test");
            assert!(result.is_err(), "Should detect SQL injection regardless of case: {}", input);
        }
    }

    #[test]
    fn test_boundary_conditions() {
        // Test exactly at the limit
        let at_limit = "a".repeat(10000);
        assert!(validate_input(&at_limit, "test").is_ok());
        
        // Test one over the limit
        let over_limit = "a".repeat(10001);
        assert!(validate_input(&over_limit, "test").is_err());
        
        // Test numeric boundaries
        assert!(validate_numeric_range(i32::MIN, i32::MIN, i32::MAX, "test").is_ok());
        assert!(validate_numeric_range(i32::MAX, i32::MIN, i32::MAX, "test").is_ok());
        assert!(validate_numeric_range(0, -100, 100, "test").is_ok());
    }

    #[test]
    fn test_complex_sql_injection_patterns() {
        let advanced_injections = vec![
            "1' AND (SELECT substring(password,1,1) FROM users WHERE username='admin')='a'--",
            "'; WAITFOR DELAY '00:00:05'--",
            "1' AND (SELECT COUNT(*) FROM information_schema.tables)>0--",
            "' OR 1=1; DROP TABLE users; --",
            "admin'--",
            "admin'#",
            "admin'/*",
            "' OR 'x'='x",
            "' OR username='admin'--",
            "1'; DECLARE @S CHAR(2000) SET @S=CAST(0x44524f... AS CHAR(2000)); EXEC(@S);--",
        ];

        for injection in advanced_injections {
            let result = validate_input(injection, "test");
            assert!(result.is_err(), "Should detect advanced SQL injection: {}", injection);
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_database_validation_integration() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let mut db = Database::new(Some(&db_path)).unwrap();

        // Test valid insertion should work
        let valid_issues = vec![
            uveddi::database::models::ArchitecturalIssue {
                issue_id: None,
                analysis_run_id: 1,
                anti_pattern_type_id: 1,
                file_path: "src/main.rs".to_string(),
                start_line: Some(10),
                end_line: Some(20),
                severity: "medium".to_string(),
                description: "Valid description".to_string(),
                code_snippet: Some("fn main() { }".to_string()),
                ai_explanation: Some("This is a valid explanation".to_string()),
            }
        ];

        let result = db.store_issues(&valid_issues);
        assert!(result.is_ok(), "Valid issues should be stored successfully");

        // Test SQL injection prevention in database
        let malicious_issues = vec![
            uveddi::database::models::ArchitecturalIssue {
                issue_id: None,
                analysis_run_id: 1,
                anti_pattern_type_id: 1,
                file_path: "'; DROP TABLE users; --".to_string(),
                start_line: Some(10),
                end_line: Some(20),
                severity: "high".to_string(),
                description: "'; DELETE FROM analysis_runs; --".to_string(),
                code_snippet: Some("'; EXEC xp_cmdshell('malicious'); --".to_string()),
                ai_explanation: Some("' UNION SELECT * FROM secrets --".to_string()),
            }
        ];

        let result = db.store_issues(&malicious_issues);
        assert!(result.is_err(), "Malicious inputs should be rejected");
        
        // Verify the error is a security error
        match result.unwrap_err() {
            uveddi::error::UveddiError::SecurityError { .. } => {
                // Expected security error
            }
            _ => panic!("Expected SecurityError for malicious input"),
        }
    }

    #[test]
    fn test_cli_validation_integration() {
        use uveddi::cli::analyze_command::AnalyzeCommand;
        use std::path::PathBuf;

        // Test valid command
        let valid_command = AnalyzeCommand {
            path: PathBuf::from("./src"),
            output_format: "json".to_string(),
            output: Some(PathBuf::from("report.json")),
            enable_ai: true,
            ollama_api_url: Some("http://localhost:11434".to_string()),
            ollama_model: Some("deepseek-coder".to_string()),
            dead_code_confidence: Some(0.8),
            dead_code_library_mode: false,
            dead_code_ignore_patterns: Some(vec!["test".to_string(), "mock".to_string()]),
            dead_code_keep_alive: Some(vec!["main".to_string(), "init".to_string()]),
            large_classes_max_loc: Some(500),
            large_classes_max_methods: Some(20),
            large_classes_max_fields: Some(15),
            large_classes_max_complexity: Some(50),
            large_classes_max_lcom: Some(0.8),
            large_classes_ignore_patterns: Some(vec!["generated".to_string()]),
            large_classes_min_severity: Some(25),
            enable_memory_optimization: false,
            memory_limit_gb: Some(4.0),
            memory_profile: Some("default".to_string()),
            enable_image_rendering: false,
            mermaid_only: false,
            no_fallback: false,
            rendering_service_url: "http://localhost:3000".to_string(),
            #[cfg(feature = "image-rendering")]
            diagram_format: "png".to_string(),
        };

        let result = valid_command.validate_inputs();
        assert!(result.is_ok(), "Valid command should pass validation");

        // Test malicious command
        let malicious_command = AnalyzeCommand {
            path: PathBuf::from("'; DROP TABLE users; --"),
            output_format: "json'; DELETE FROM data; --".to_string(),
            output: Some(PathBuf::from("'; EXEC malicious; --.json")),
            enable_ai: true,
            ollama_api_url: Some("javascript:alert(1)".to_string()),
            ollama_model: Some("../../../etc/passwd".to_string()),
            dead_code_confidence: Some(150.0), // Invalid range
            dead_code_library_mode: false,
            dead_code_ignore_patterns: Some(vec!["'; DROP TABLE test; --".to_string()]),
            dead_code_keep_alive: Some(vec!["' UNION SELECT * FROM secrets --".to_string()]),
            large_classes_max_loc: Some(1_000_000), // Invalid range
            large_classes_max_methods: Some(100_000), // Invalid range
            large_classes_max_fields: Some(100_000), // Invalid range
            large_classes_max_complexity: Some(100_000), // Invalid range
            large_classes_max_lcom: Some(150.0), // Invalid range
            large_classes_ignore_patterns: Some(vec!["'; EXEC xp_cmdshell('dir'); --".to_string()]),
            large_classes_min_severity: Some(150), // Invalid range
            enable_memory_optimization: false,
            memory_limit_gb: Some(1000.0), // Invalid range
            memory_profile: Some("'; DELETE FROM config; --".to_string()),
            enable_image_rendering: false,
            mermaid_only: false,
            no_fallback: false,
            rendering_service_url: "http://localhost:3000".to_string(),
            #[cfg(feature = "image-rendering")]
            diagram_format: "png".to_string(),
        };

        let result = malicious_command.validate_inputs();
        assert!(result.is_err(), "Malicious command should be rejected");
    }

    #[test]
    fn test_validation_error_messages() {
        // Test that error messages are informative but don't leak sensitive information
        let result = validate_input("'; DROP TABLE users; --", "username");
        assert!(result.is_err());
        
        if let Err(SecurityError::InvalidInput { field, reason }) = result {
            assert_eq!(field, "username");
            assert!(reason.contains("dangerous SQL patterns"));
            assert!(!reason.contains("DROP TABLE")); // Don't echo the malicious input
        }

        // Test URL validation error
        let result = validate_url("javascript:alert(1)");
        assert!(result.is_err());
        
        if let Err(SecurityError::InvalidInput { field, reason }) = result {
            assert_eq!(field, "url");
            assert!(reason.contains("must start with http"));
        }

        // Test numeric range error
        let result = validate_numeric_range(150, 0, 100, "confidence");
        assert!(result.is_err());
        
        if let Err(SecurityError::InvalidInput { field, reason }) = result {
            assert_eq!(field, "confidence");
            assert!(reason.contains("must be between 0 and 100"));
        }
    }

    #[test]
    fn test_concurrent_validation() {
        use std::thread;
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};

        let success_count = Arc::new(AtomicUsize::new(0));
        let failure_count = Arc::new(AtomicUsize::new(0));

        let handles: Vec<_> = (0..10).map(|i| {
            let success_count = Arc::clone(&success_count);
            let failure_count = Arc::clone(&failure_count);
            
            thread::spawn(move || {
                for j in 0..100 {
                    let input = if j % 2 == 0 {
                        format!("valid_input_{}", i * 100 + j)
                    } else {
                        format!("'; DROP TABLE test_{}; --", i * 100 + j)
                    };
                    
                    match validate_input(&input, "test") {
                        Ok(_) => success_count.fetch_add(1, Ordering::SeqCst),
                        Err(_) => failure_count.fetch_add(1, Ordering::SeqCst),
                    };
                }
            })
        }).collect();

        for handle in handles {
            handle.join().unwrap();
        }

        let total_success = success_count.load(Ordering::SeqCst);
        let total_failure = failure_count.load(Ordering::SeqCst);
        
        assert_eq!(total_success + total_failure, 1000);
        assert_eq!(total_success, 500); // Half should be valid
        assert_eq!(total_failure, 500); // Half should be SQL injection attempts
    }
}