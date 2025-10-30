//! Comprehensive path traversal security tests for UV-272
//! 
//! This test suite validates the sanitize_path function against various
//! path traversal attack vectors including symlinks, Unicode normalization,
//! and common bypass techniques.

use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use uveddi::security::{sanitize_path, errors::SecurityError};

#[cfg(test)]
mod path_traversal_tests {
    use super::*;

    /// Test basic path traversal prevention
    #[test]
    fn test_basic_path_traversal_prevention() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();
        
        // Create a test file within the base directory
        let safe_file = base_path.join("safe_file.txt");
        fs::write(&safe_file, "safe content").unwrap();
        
        // Test legitimate file access
        let result = sanitize_path(&safe_file, base_path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), safe_file.canonicalize().unwrap());
        
        // Test path traversal attempts
        let traversal_attempts = vec![
            "../../../etc/passwd",
            "..\\..\\..\\windows\\system32\\config\\sam",
            "./../../../etc/shadow",
            "../../../../usr/bin/bash",
            "../../../../../proc/self/environ",
        ];
        
        for attempt in traversal_attempts {
            let result = sanitize_path(attempt, base_path);
            assert!(
                matches!(result, Err(SecurityError::PathTraversalAttempt) | Err(SecurityError::InvalidPath)),
                "Path traversal attempt should be blocked: {}",
                attempt
            );
        }
    }

    /// Test symlink-based path traversal attacks
    #[test]
    fn test_symlink_path_traversal_prevention() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();
        
        // Create a symlink pointing outside the base directory
        let symlink_path = base_path.join("malicious_link");
        
        // Try to create symlink to /etc/passwd (will fail gracefully if not possible)
        if std::os::unix::fs::symlink("/etc/passwd", &symlink_path).is_ok() {
            let result = sanitize_path(&symlink_path, base_path);
            assert!(
                matches!(result, Err(SecurityError::PathTraversalAttempt)),
                "Symlink pointing outside base directory should be blocked"
            );
        }
        
        // Create a legitimate symlink within the base directory
        let safe_target = base_path.join("safe_target.txt");
        fs::write(&safe_target, "safe content").unwrap();
        
        let safe_symlink = base_path.join("safe_link");
        if std::os::unix::fs::symlink(&safe_target, &safe_symlink).is_ok() {
            let result = sanitize_path(&safe_symlink, base_path);
            assert!(result.is_ok(), "Legitimate symlink within base directory should be allowed");
        }
    }

    /// Test Unicode normalization attacks
    #[test]
    fn test_unicode_normalization_attacks() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();
        
        // Unicode normalization attack vectors
        let unicode_attacks = vec![
            // Unicode dot variations
            "file\u{002E}\u{002E}/../../etc/passwd",
            "file\u{FF0E}\u{FF0E}/../../etc/passwd",
            // Unicode slash variations
            "file\u{002F}\u{002E}\u{002E}\u{002F}../../etc/passwd",
            "file\u{FF0F}\u{002E}\u{002E}\u{FF0F}../../etc/passwd",
            // Mixed Unicode and ASCII
            "file.\u{FF0E}/\u{002E}\u{002E}/../../etc/passwd",
        ];
        
        for attack in unicode_attacks {
            let result = sanitize_path(attack, base_path);
            assert!(
                result.is_err(),
                "Unicode normalization attack should be blocked: {}",
                attack
            );
        }
    }

    /// Test null byte injection attacks
    #[test]
    fn test_null_byte_injection() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();
        
        // Null byte injection attempts
        let null_byte_attacks = vec![
            "safe_file.txt\0../../../etc/passwd",
            "safe_file.txt\0\0../../../etc/passwd",
            "../../../etc/passwd\0safe_file.txt",
        ];
        
        for attack in null_byte_attacks {
            let result = sanitize_path(attack, base_path);
            assert!(
                result.is_err(),
                "Null byte injection should be blocked: {:?}",
                attack
            );
        }
    }

    /// Test double encoding attacks
    #[test]
    fn test_double_encoding_attacks() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();
        
        // Double-encoded path traversal attempts
        let double_encoded_attacks = vec![
            "%252e%252e%252f%252e%252e%252f%252e%252e%252fetc%252fpasswd",
            "%2e%2e%2f%2e%2e%2f%2e%2e%2fetc%2fpasswd",
            "..%252f..%252f..%252fetc%252fpasswd",
        ];
        
        for attack in double_encoded_attacks {
            let result = sanitize_path(attack, base_path);
            assert!(
                result.is_err(),
                "Double encoding attack should be blocked: {}",
                attack
            );
        }
    }

    /// Test edge cases and boundary conditions
    #[test]
    fn test_edge_cases() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();
        
        // Test empty path
        let result = sanitize_path("", base_path);
        assert!(result.is_err());
        
        // Test current directory
        let result = sanitize_path(".", base_path);
        assert!(result.is_ok());
        
        // Test root path
        let result = sanitize_path("/", base_path);
        assert!(matches!(result, Err(SecurityError::PathTraversalAttempt)));
        
        // Test very long path
        let long_path = "../".repeat(1000) + "etc/passwd";
        let result = sanitize_path(&long_path, base_path);
        assert!(result.is_err());
    }

    /// Test invalid base directory handling
    #[test]
    fn test_invalid_base_directory() {
        // Test with non-existent base directory
        let result = sanitize_path("test.txt", "/non/existent/directory");
        assert!(matches!(result, Err(SecurityError::InvalidBasePath)));
        
        // Test with file as base directory (should fail)
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("not_a_directory.txt");
        fs::write(&file_path, "content").unwrap();
        
        let result = sanitize_path("test.txt", &file_path);
        assert!(result.is_err());
    }

    /// Test case sensitivity attacks (primarily for Windows compatibility)
    #[test]
    fn test_case_sensitivity_attacks() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();
        
        // Case variation attacks
        let case_attacks = vec![
            "../../../ETC/passwd",
            "../../../etc/PASSWD",
            "../../../Etc/Passwd",
            "..\\..\\..\\WINDOWS\\system32\\config\\SAM",
        ];
        
        for attack in case_attacks {
            let result = sanitize_path(attack, base_path);
            assert!(
                result.is_err(),
                "Case sensitivity attack should be blocked: {}",
                attack
            );
        }
    }

    /// Test legitimate file access patterns
    #[test]
    fn test_legitimate_access_patterns() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();
        
        // Create nested directory structure
        let nested_dir = base_path.join("subdir").join("nested");
        fs::create_dir_all(&nested_dir).unwrap();
        
        let nested_file = nested_dir.join("file.txt");
        fs::write(&nested_file, "content").unwrap();
        
        // Test legitimate access patterns
        let legitimate_paths = vec![
            "subdir/nested/file.txt",
            "./subdir/nested/file.txt",
            "subdir/../subdir/nested/file.txt", // This should resolve to legitimate path
        ];
        
        for path in legitimate_paths {
            let full_path = base_path.join(path);
            if full_path.exists() {
                let result = sanitize_path(&full_path, base_path);
                assert!(
                    result.is_ok(),
                    "Legitimate path should be allowed: {}",
                    path
                );
            }
        }
    }

    /// Test performance with large number of path checks
    #[test]
    fn test_performance_stress() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();
        
        let safe_file = base_path.join("test.txt");
        fs::write(&safe_file, "content").unwrap();
        
        // Test performance with many legitimate requests
        let start = std::time::Instant::now();
        for _ in 0..1000 {
            let result = sanitize_path(&safe_file, base_path);
            assert!(result.is_ok());
        }
        let duration = start.elapsed();
        
        // Should complete 1000 operations in reasonable time (< 1 second)
        assert!(duration.as_secs() < 1, "Performance test took too long: {:?}", duration);
        
        // Test performance with many attack attempts
        let start = std::time::Instant::now();
        for _ in 0..1000 {
            let result = sanitize_path("../../../etc/passwd", base_path);
            assert!(result.is_err());
        }
        let duration = start.elapsed();
        
        // Attack detection should also be fast
        assert!(duration.as_secs() < 1, "Attack detection took too long: {:?}", duration);
    }
}

/// Integration tests with the broader security system
#[cfg(test)]
mod integration_tests {
    use super::*;
    use uveddi::security::errors::SecurityError;

    #[test]
    fn test_error_categorization() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path();
        
        // Test that path traversal errors are properly categorized for audit logging
        let result = sanitize_path("../../../etc/passwd", base_path);
        
        match result {
            Err(SecurityError::PathTraversalAttempt) => {
                // Verify this error should be audit logged
                let error = SecurityError::PathTraversalAttempt;
                assert!(error.should_audit_log(), "Path traversal attempts should be audit logged");
            }
            Err(SecurityError::InvalidPath) => {
                // This is also acceptable for non-existent paths
                let error = SecurityError::InvalidPath;
                assert!(error.should_audit_log(), "Invalid path errors should be audit logged");
            }
            _ => panic!("Expected path security error"),
        }
    }

    #[test]
    fn test_error_severity() {
        // Path traversal attempts should be high severity
        let error = SecurityError::PathTraversalAttempt;
        assert_eq!(error.severity(), uveddi::security::errors::SecurityErrorSeverity::High);
        
        let error = SecurityError::InvalidPath;
        assert_eq!(error.severity(), uveddi::security::errors::SecurityErrorSeverity::Medium);
        
        let error = SecurityError::InvalidBasePath;
        assert_eq!(error.severity(), uveddi::security::errors::SecurityErrorSeverity::Medium);
    }
}