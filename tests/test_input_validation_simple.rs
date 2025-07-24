//! Simple test for input validation functionality

use uveddi::security::{validate_input, validate_url, validate_model_name, validate_numeric_range};

#[test]
fn test_basic_input_validation() {
    // Test valid input
    assert!(validate_input("normal text", "test_field").is_ok());
    
    // Test SQL injection
    assert!(validate_input("'; DROP TABLE users; --", "test_field").is_err());
    
    // Test length limit
    let long_input = "a".repeat(10001);
    assert!(validate_input(&long_input, "test_field").is_err());
}

#[test]
fn test_url_validation() {
    // Valid URLs
    assert!(validate_url("http://localhost:11434").is_ok());
    assert!(validate_url("https://api.example.com").is_ok());
    
    // Invalid URLs
    assert!(validate_url("javascript:alert(1)").is_err());
    assert!(validate_url("ftp://example.com").is_err());
}

#[test]
fn test_model_name_validation() {
    // Valid model names
    assert!(validate_model_name("deepseek-coder").is_ok());
    assert!(validate_model_name("llama2-7b").is_ok());
    
    // Invalid model names
    assert!(validate_model_name("model/with/slashes").is_err());
    assert!(validate_model_name("").is_err());
}

#[test]
fn test_numeric_range_validation() {
    // Valid ranges
    assert!(validate_numeric_range(50, 0, 100, "test").is_ok());
    assert!(validate_numeric_range(0, 0, 100, "test").is_ok());
    assert!(validate_numeric_range(100, 0, 100, "test").is_ok());
    
    // Invalid ranges
    assert!(validate_numeric_range(-1, 0, 100, "test").is_err());
    assert!(validate_numeric_range(101, 0, 100, "test").is_err());
}