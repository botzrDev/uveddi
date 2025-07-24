//! Security tests for JSON deserialization
//! 
//! Tests secure deserialization patterns implemented to fix UV-275.
//! These tests validate protection against malicious JSON payloads,
//! size limits, and proper error handling.

#[cfg(test)]
mod deserialization_security_tests {
    use uveddi::community::database::CommunityDatabase;
    use uveddi::error::{DeserializationError, UveddiError};
    use std::collections::HashMap;

    /// Test malicious JSON payloads that should be rejected
    #[test]
    fn test_malicious_json_payloads() {
        let db = CommunityDatabase::new_in_memory().unwrap();

        // Test oversized payload (simulate large JSON)
        let large_json = format!("[{}]", "\"language\",".repeat(50000));
        let result = db.deserialize_languages(&large_json);
        assert!(result.is_err());
        if let Err(UveddiError::Deserialization { source: Some(DeserializationError::PayloadTooLarge { .. }), .. }) = result {
            // Expected error type
        } else {
            panic!("Expected PayloadTooLarge error for oversized payload");
        }

        // Test malformed JSON
        let malformed = "{ invalid json }";
        let result = db.deserialize_custom_fields(malformed);
        assert!(result.is_err());
        if let Err(UveddiError::Deserialization { source: Some(DeserializationError::InvalidFormat(_)), .. }) = result {
            // Expected error type
        } else {
            panic!("Expected InvalidFormat error for malformed JSON");
        }

        // Test potential XSS/injection attempts in custom fields
        let injection_json = r#"{"<script>alert('xss')</script>": "value", "key": "<img src=x onerror=alert(1)>"}"#;
        let result = db.deserialize_custom_fields(injection_json);
        // Should succeed as parsing but values would be sanitized in actual usage
        assert!(result.is_ok());
        
        // Test SQL injection attempt in language codes
        let injection_languages = r#"["'; DROP TABLE community_members; --", "python"]"#;
        let result = db.deserialize_languages(injection_languages);
        assert!(result.is_err()); // Should fail validation due to invalid characters
    }

    /// Test size and count limits are enforced
    #[test]
    fn test_size_limits() {
        let db = CommunityDatabase::new_in_memory().unwrap();

        // Test array length limits for languages
        let too_many_languages: Vec<String> = (0..2000).map(|i| format!("lang{}", i)).collect();
        let json = serde_json::to_string(&too_many_languages).unwrap();
        let result = db.deserialize_languages(&json);
        assert!(result.is_err());
        if let Err(UveddiError::Deserialization { source: Some(DeserializationError::SecurityValidation(_)), .. }) = result {
            // Expected error type
        } else {
            panic!("Expected SecurityValidation error for too many languages");
        }

        // Test field count limits for custom fields
        let too_many_fields: HashMap<String, String> = 
            (0..100).map(|i| (format!("key{}", i), format!("value{}", i))).collect();
        let json = serde_json::to_string(&too_many_fields).unwrap();
        let result = db.deserialize_custom_fields(&json);
        assert!(result.is_err());
        if let Err(UveddiError::Deserialization { source: Some(DeserializationError::SecurityValidation(_)), .. }) = result {
            // Expected error type
        } else {
            panic!("Expected SecurityValidation error for too many fields");
        }

        // Test individual field size limits
        let oversized_field = HashMap::from([
            ("normal_key".to_string(), "x".repeat(2000)), // Value too large
        ]);
        let json = serde_json::to_string(&oversized_field).unwrap();
        let result = db.deserialize_custom_fields(&json);
        assert!(result.is_err());

        // Test key size limits
        let oversized_key = HashMap::from([
            ("x".repeat(200), "normal_value".to_string()), // Key too large
        ]);
        let json = serde_json::to_string(&oversized_key).unwrap();
        let result = db.deserialize_custom_fields(&json);
        assert!(result.is_err());
    }

    /// Test that valid data is accepted properly
    #[test]
    fn test_valid_data_acceptance() {
        let db = CommunityDatabase::new_in_memory().unwrap();

        // Test valid languages
        let valid_languages = r#"["rust", "python", "javascript", "typescript", "go"]"#;
        let result = db.deserialize_languages(valid_languages);
        assert!(result.is_ok());
        let languages = result.unwrap();
        assert_eq!(languages.len(), 5);
        assert!(languages.contains(&"rust".to_string()));
        assert!(languages.contains(&"python".to_string()));

        // Test valid custom fields
        let valid_fields = r#"{
            "company": "TechCorp", 
            "role": "developer",
            "experience": "senior",
            "location": "Remote"
        }"#;
        let result = db.deserialize_custom_fields(valid_fields);
        assert!(result.is_ok());
        let fields = result.unwrap();
        assert_eq!(fields.len(), 4);
        assert_eq!(fields.get("company"), Some(&"TechCorp".to_string()));
        assert_eq!(fields.get("role"), Some(&"developer".to_string()));

        // Test valid metadata (reuses custom fields validation)
        let result = db.deserialize_metadata(valid_fields);
        assert!(result.is_ok());
        let metadata = result.unwrap();
        assert_eq!(metadata.len(), 4);
    }

    /// Test language code validation rules
    #[test]
    fn test_language_validation() {
        let db = CommunityDatabase::new_in_memory().unwrap();

        // Test valid language codes
        let valid_languages = r#"["rust", "python3", "c-sharp", "objective_c"]"#;
        let result = db.deserialize_languages(valid_languages);
        assert!(result.is_ok());

        // Test invalid language codes with special characters
        let invalid_languages = r#"["rust", "python/3", "c#", "objective-c++"]"#;
        let result = db.deserialize_languages(invalid_languages);
        assert!(result.is_err()); // Should fail due to invalid characters

        // Test language codes that are too long
        let long_language = "x".repeat(100);
        let long_languages = format!(r#"["rust", "{}"]"#, long_language);
        let result = db.deserialize_languages(&long_languages);
        assert!(result.is_err()); // Should fail due to length limit

        // Test empty language codes
        let empty_languages = r#"["rust", "", "python"]"#;
        let result = db.deserialize_languages(empty_languages);
        assert!(result.is_ok()); // Empty strings are allowed but not ideal
    }

    /// Test edge cases and boundary conditions
    #[test]
    fn test_edge_cases() {
        let db = CommunityDatabase::new_in_memory().unwrap();

        // Test empty arrays and objects
        let empty_languages = r#"[]"#;
        let result = db.deserialize_languages(empty_languages);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);

        let empty_fields = r#"{}"#;
        let result = db.deserialize_custom_fields(empty_fields);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);

        // Test exactly at the limits
        let at_limit_languages: Vec<String> = (0..1000).map(|i| format!("l{}", i)).collect();
        let json = serde_json::to_string(&at_limit_languages).unwrap();
        let result = db.deserialize_languages(&json);
        assert!(result.is_ok()); // Should succeed at exactly the limit

        let at_limit_fields: HashMap<String, String> = 
            (0..50).map(|i| (format!("k{}", i), format!("v{}", i))).collect();
        let json = serde_json::to_string(&at_limit_fields).unwrap();
        let result = db.deserialize_custom_fields(&json);
        assert!(result.is_ok()); // Should succeed at exactly the limit

        // Test Unicode and special characters in valid contexts
        let unicode_fields = r#"{"café": "résumé", "名前": "値"}"#;
        let result = db.deserialize_custom_fields(unicode_fields);
        assert!(result.is_ok()); // Unicode should be accepted
    }

    /// Test error message quality and logging
    #[test] 
    fn test_error_messages() {
        let db = CommunityDatabase::new_in_memory().unwrap();

        // Test that error messages are informative
        let malformed = "{ invalid";
        let result = db.deserialize_languages(malformed);
        assert!(result.is_err());
        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("JSON"));
        assert!(error_msg.contains("format"));

        // Test that security validation errors are clear
        let too_many: Vec<String> = (0..2000).map(|i| format!("l{}", i)).collect();
        let json = serde_json::to_string(&too_many).unwrap();
        let result = db.deserialize_languages(&json);
        assert!(result.is_err());
        let error_msg = format!("{}", result.unwrap_err());
        assert!(error_msg.contains("too large") || error_msg.contains("many"));
    }

    /// Test performance with large valid payloads
    #[test]
    fn test_performance_regression() {
        let db = CommunityDatabase::new_in_memory().unwrap();

        // Test that reasonable-sized valid payloads perform acceptably
        let reasonable_languages: Vec<String> = (0..100).map(|i| format!("lang{}", i)).collect();
        let json = serde_json::to_string(&reasonable_languages).unwrap();
        
        let start = std::time::Instant::now();
        let result = db.deserialize_languages(&json);
        let duration = start.elapsed();
        
        assert!(result.is_ok());
        assert!(duration.as_millis() < 100); // Should complete quickly

        let reasonable_fields: HashMap<String, String> = 
            (0..20).map(|i| (format!("key{}", i), format!("value{}", i))).collect();
        let json = serde_json::to_string(&reasonable_fields).unwrap();
        
        let start = std::time::Instant::now();
        let result = db.deserialize_custom_fields(&json);
        let duration = start.elapsed();
        
        assert!(result.is_ok());
        assert!(duration.as_millis() < 50); // Should complete very quickly
    }
}