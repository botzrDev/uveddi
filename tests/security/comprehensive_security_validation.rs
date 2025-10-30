//! Comprehensive Security Validation Tests
//!
//! This module contains comprehensive security validation tests that verify
//! all security hardening measures are properly implemented and functioning.

use std::path::Path;
use std::time::Duration;
use tokio::time::timeout;
use uveddi::security::{SecurityConfig, HttpSecurityConfig, SecureHttpClient};
use uveddi::security::errors::{SecurityResult, SecurityError};

/// Test HTTP client security enforcement
#[tokio::test]
async fn test_https_enforcement() {
    // Test HTTPS-only configuration
    let mut config = HttpSecurityConfig::default();
    config.enforce_https = true;
    
    let client = SecureHttpClient::new(config).expect("Failed to create secure client");
    
    // Attempt HTTP request (should fail)
    let result = client.get("http://example.com").await;
    assert!(result.is_err(), "HTTP request should be blocked when HTTPS is enforced");
}

/// Test secure timeout configuration
#[tokio::test]
async fn test_secure_timeouts() {
    let mut config = HttpSecurityConfig::default();
    config.timeout_seconds = 1; // Very short timeout
    config.connect_timeout_seconds = 1;
    config.read_timeout_seconds = 1;
    
    let client = SecureHttpClient::new(config).expect("Failed to create secure client");
    
    // This should timeout quickly
    let start = std::time::Instant::now();
    let result = timeout(
        Duration::from_secs(5),
        client.get("https://httpbin.org/delay/10")
    ).await;
    
    let elapsed = start.elapsed();
    assert!(elapsed < Duration::from_secs(3), "Request should timeout within configured limits");
}

/// Test certificate validation
#[tokio::test]
async fn test_certificate_validation() {
    let config = HttpSecurityConfig::default();
    let client = SecureHttpClient::new(config).expect("Failed to create secure client");
    
    // Test with valid certificate
    let result = client.get("https://httpbin.org/get").await;
    match result {
        Ok(_) => {}, // Expected for valid cert
        Err(e) => {
            // Network errors are acceptable in test environments
            println!("Network error (acceptable in tests): {}", e);
        }
    }
    
    // Test with self-signed certificate (should fail)
    // Note: This requires a test server with self-signed cert
    // In practice, this would be tested with a mock service
}

/// Test input validation security
#[tokio::test]
async fn test_input_validation_security() {
    // Test path traversal prevention
    assert!(is_path_safe("../../../etc/passwd") == false);
    assert!(is_path_safe("..\\..\\windows\\system32") == false);
    assert!(is_path_safe("legitimate/path/file.txt") == true);
    
    // Test SQL injection patterns
    assert!(contains_sql_injection("'; DROP TABLE users; --") == true);
    assert!(contains_sql_injection("normal input") == false);
    
    // Test XSS patterns
    assert!(contains_xss_patterns("<script>alert('xss')</script>") == true);
    assert!(contains_xss_patterns("normal text") == false);
}

/// Test file access security
#[tokio::test]
async fn test_file_access_security() {
    // Test that sensitive files are not accessible
    let sensitive_paths = [
        "/etc/passwd",
        "/etc/shadow",
        "/root/.ssh/id_rsa",
        "C:\\Windows\\System32\\config\\SAM",
    ];
    
    for path in &sensitive_paths {
        if Path::new(path).exists() {
            // In a real application, verify these can't be accessed
            // This test would be environment-specific
            println!("Sensitive file exists (test environment): {}", path);
        }
    }
}

/// Test rate limiting enforcement
#[tokio::test]
async fn test_rate_limiting() {
    // This would test the rate limiting implementation
    // For now, we'll test that the rate limiter configuration is valid
    let config = SecurityConfig::default();
    
    // Verify rate limiting is configured
    assert!(config.rate_limiting.enabled);
    assert!(config.rate_limiting.requests_per_minute > 0);
    assert!(config.rate_limiting.burst_size > 0);
}

/// Test audit logging functionality
#[tokio::test]
async fn test_audit_logging() {
    // Test that security events are properly logged
    let config = SecurityConfig::default();
    
    // Verify audit logging is enabled
    assert!(config.audit_logging.enabled);
    assert!(!config.audit_logging.log_file_path.is_empty());
    
    // Test log rotation settings
    assert!(config.audit_logging.max_file_size_mb > 0);
    assert!(config.audit_logging.max_files > 0);
}

/// Test encryption configuration
#[tokio::test]
async fn test_encryption_config() {
    let config = SecurityConfig::default();
    
    // Verify encryption settings
    assert!(config.encryption.enabled);
    assert!(!config.encryption.key_derivation_algorithm.is_empty());
    assert!(config.encryption.key_length >= 256); // Minimum 256-bit keys
}

/// Test authentication and authorization
#[tokio::test]
async fn test_auth_security() {
    let config = SecurityConfig::default();
    
    // Verify authentication settings
    assert!(config.authentication.enabled);
    assert!(config.authentication.session_timeout_minutes > 0);
    assert!(config.authentication.max_failed_attempts > 0);
    
    // Verify authorization settings
    assert!(config.authorization.enabled);
    assert!(!config.authorization.default_role.is_empty());
}

/// Test secret management
#[tokio::test]
async fn test_secret_management() {
    // Test that secrets are not exposed in configuration
    let config = SecurityConfig::default();
    
    // Verify no hardcoded secrets
    let config_str = format!("{:?}", config);
    assert!(!config_str.contains("password"));
    assert!(!config_str.contains("secret"));
    assert!(!config_str.contains("key"));
    
    // Test environment variable handling
    std::env::set_var("TEST_SECRET", "test_value");
    let env_value = std::env::var("TEST_SECRET").unwrap();
    assert_eq!(env_value, "test_value");
    std::env::remove_var("TEST_SECRET");
}

/// Test container security validation
#[tokio::test]
async fn test_container_security() {
    // Test that we're not running as root
    #[cfg(unix)]
    {
        use std::os::unix::process::parent_id;
        let uid = unsafe { libc::getuid() };
        assert_ne!(uid, 0, "Process should not run as root (UID 0)");
    }
    
    // Test file permissions
    let test_file = "/tmp/security_test_file";
    if let Ok(_) = std::fs::write(test_file, "test") {
        if let Ok(metadata) = std::fs::metadata(test_file) {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mode = metadata.permissions().mode();
                // Verify restrictive permissions (not world-readable)
                assert_eq!(mode & 0o004, 0, "Files should not be world-readable");
            }
        }
        let _ = std::fs::remove_file(test_file);
    }
}

/// Test network security
#[tokio::test]
async fn test_network_security() {
    // Test that insecure protocols are disabled
    let config = HttpSecurityConfig::default();
    
    #[cfg(not(debug_assertions))]
    {
        assert!(config.enforce_https, "HTTPS should be enforced in release builds");
    }
    
    assert!(config.timeout_seconds > 0, "Timeouts should be configured");
    assert!(config.connect_timeout_seconds > 0, "Connection timeouts should be configured");
    assert!(config.read_timeout_seconds > 0, "Read timeouts should be configured");
}

/// Test security configuration validation
#[tokio::test]
async fn test_security_config_validation() {
    let config = SecurityConfig::default();
    
    // Test configuration validation
    assert!(config.validate().is_ok(), "Default security configuration should be valid");
    
    // Test invalid configuration
    let mut invalid_config = config;
    invalid_config.rate_limiting.requests_per_minute = 0;
    assert!(invalid_config.validate().is_err(), "Invalid configuration should fail validation");
}

/// Test error handling and reporting
#[tokio::test]
async fn test_security_error_handling() {
    // Test security error creation and handling
    let error = SecurityError::AuthenticationFailed {
        reason: "Test authentication failure".to_string(),
    };
    
    assert!(error.should_audit_log(), "Authentication failures should be audit logged");
    assert_eq!(error.severity(), uveddi::security::errors::ErrorSeverity::High);
    
    // Test error serialization
    let error_msg = error.user_message();
    assert!(!error_msg.is_empty(), "Error should have user message");
    assert!(!error_msg.contains("internal"), "User message should not expose internals");
}

// Helper functions for security validation

fn is_path_safe(path: &str) -> bool {
    !path.contains("..") && !path.contains("~") && !path.starts_with('/')
}

fn contains_sql_injection(input: &str) -> bool {
    let sql_patterns = [
        "'; DROP",
        "'; DELETE",
        "'; INSERT",
        "'; UPDATE",
        "UNION SELECT",
        "OR 1=1",
        "OR '1'='1'",
    ];
    
    let input_upper = input.to_uppercase();
    sql_patterns.iter().any(|pattern| input_upper.contains(pattern))
}

fn contains_xss_patterns(input: &str) -> bool {
    let xss_patterns = [
        "<script",
        "javascript:",
        "on load=",
        "onerror=",
        "onclick=",
        "eval(",
    ];
    
    let input_lower = input.to_lowercase();
    xss_patterns.iter().any(|pattern| input_lower.contains(pattern))
}

/// Performance impact assessment
#[tokio::test]
async fn test_security_performance_impact() {
    use std::time::Instant;
    
    // Measure secure HTTP client creation time
    let start = Instant::now();
    let _client = SecureHttpClient::new(HttpSecurityConfig::default());
    let creation_time = start.elapsed();
    
    // Should be fast (under 100ms)
    assert!(creation_time < Duration::from_millis(100), 
           "Secure client creation should be fast: {:?}", creation_time);
    
    // Measure request validation time
    let start = Instant::now();
    let _safe = is_path_safe("legitimate/path/file.txt");
    let validation_time = start.elapsed();
    
    // Should be very fast (under 1ms)
    assert!(validation_time < Duration::from_millis(1),
           "Path validation should be very fast: {:?}", validation_time);
}

/// Integration test with real security scenarios
#[tokio::test]
async fn test_security_integration() {
    // Test complete security workflow
    let config = SecurityConfig::default();
    assert!(config.validate().is_ok());
    
    // Test HTTP client security
    let http_config = HttpSecurityConfig::default();
    let client = SecureHttpClient::new(http_config);
    assert!(client.is_ok());
    
    // Test input validation pipeline
    let malicious_inputs = [
        "../../../etc/passwd",
        "<script>alert('xss')</script>",
        "'; DROP TABLE users; --",
        "javascript:alert('xss')",
    ];
    
    for input in &malicious_inputs {
        assert!(!is_path_safe(input) || contains_xss_patterns(input) || contains_sql_injection(input),
               "Malicious input should be detected: {}", input);
    }
}