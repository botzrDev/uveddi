//! UV-276: Comprehensive error handling tests for unwrap replacements
//!
//! This test suite validates that all unwrap() replacements handle error conditions
//! gracefully and provide meaningful error messages.

use std::sync::{Arc, Mutex, RwLock};
use std::collections::HashMap;
use std::path::PathBuf;
use uveddi::analysis::errors::AnalysisError;

#[cfg(test)]
mod unwrap_replacement_tests {
    use super::*;

    #[test]
    fn test_parse_error_creation() {
        let error = AnalysisError::parse_error("Failed to parse invalid syntax");
        assert!(matches!(error, AnalysisError::ParseError { .. }));
        assert_eq!(error.to_string(), "Parse operation failed: Failed to parse invalid syntax");
    }

    #[test]
    fn test_query_error_creation() {
        let error = AnalysisError::query_error("Missing capture group");
        assert!(matches!(error, AnalysisError::QueryError(_)));
        assert_eq!(error.to_string(), "Tree-sitter query error: Missing capture group");
    }

    #[test]
    fn test_collection_access_error_creation() {
        let error = AnalysisError::collection_access_error("Index out of bounds");
        assert!(matches!(error, AnalysisError::CollectionAccessError { .. }));
        assert_eq!(error.to_string(), "Collection access failed: Index out of bounds");
    }

    #[test]
    fn test_conversion_error_creation() {
        let error = AnalysisError::conversion_error("u64 to u32 overflow");
        assert!(matches!(error, AnalysisError::ConversionError { .. }));
        assert_eq!(error.to_string(), "Type conversion failed: u64 to u32 overflow");
    }

    #[test]
    fn test_data_not_found_error_creation() {
        let error = AnalysisError::data_not_found_error("Expected node not found");
        assert!(matches!(error, AnalysisError::DataNotFoundError { .. }));
        assert_eq!(error.to_string(), "Expected data not found: Expected node not found");
    }

    #[test]
    fn test_lock_error_creation() {
        let error = AnalysisError::lock_error("Mutex poisoned");
        assert!(matches!(error, AnalysisError::LockError { .. }));
        assert_eq!(error.to_string(), "Mutex lock failed: Mutex poisoned");
    }

    #[test]
    fn test_line_number_overflow_error() {
        let error = AnalysisError::LineNumberOverflow { value: u64::MAX };
        assert!(matches!(error, AnalysisError::LineNumberOverflow { .. }));
        assert!(error.to_string().contains("exceeds maximum"));
    }

    #[test]
    fn test_safe_collection_access_patterns() {
        let empty_vec: Vec<String> = vec![];
        
        // Test safe indexing pattern
        let result = empty_vec.get(0)
            .ok_or_else(|| AnalysisError::collection_access_error("Empty collection"));
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AnalysisError::CollectionAccessError { .. }));
    }

    #[test]
    fn test_safe_conversion_patterns() {
        let large_value = u64::MAX;
        
        // Test safe conversion pattern
        let result: Result<u32, AnalysisError> = large_value.try_into()
            .map_err(|_| AnalysisError::conversion_error(
                format!("Value {} exceeds u32 range", large_value)
            ));
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AnalysisError::ConversionError { .. }));
    }

    #[test]
    fn test_error_chain_compatibility() {
        let parse_error = AnalysisError::parse_error("Test parse error");
        let boxed_error: Box<dyn std::error::Error> = Box::new(parse_error);
        
        // Verify error can be converted and chained
        let analysis_error = AnalysisError::from(boxed_error);
        assert!(matches!(analysis_error, AnalysisError::Other(_)));
    }

    #[test]
    fn test_poisoned_mutex_simulation() {
        // Create a mutex that we'll poison
        let mutex = Arc::new(Mutex::new(42));
        let mutex_clone = Arc::clone(&mutex);
        
        // Simulate poison by panicking while holding the lock
        let handle = std::thread::spawn(move || {
            let _guard = mutex_clone.lock().unwrap();
            panic!("Simulated panic to poison mutex");
        });
        
        // Wait for thread to panic
        let _ = handle.join();
        
        // Now the mutex should be poisoned
        let result = mutex.lock().map_err(|_| AnalysisError::lock_error("Mutex poisoned"));
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AnalysisError::LockError { .. }));
    }

    #[test]
    fn test_poisoned_rwlock_simulation() {
        // Create an RwLock that we'll poison
        let rwlock = Arc::new(RwLock::new(HashMap::<String, String>::new()));
        let rwlock_clone = Arc::clone(&rwlock);
        
        // Simulate poison by panicking while holding the write lock
        let handle = std::thread::spawn(move || {
            let _guard = rwlock_clone.write().unwrap();
            panic!("Simulated panic to poison RwLock");
        });
        
        // Wait for thread to panic
        let _ = handle.join();
        
        // Now the RwLock should be poisoned
        let result = rwlock.read().map_err(|_| AnalysisError::lock_error("RwLock poisoned"));
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AnalysisError::LockError { .. }));
    }

    #[test]
    fn test_error_message_quality() {
        // Test that error messages are descriptive and actionable
        let errors = vec![
            AnalysisError::parse_error("Failed to parse source code at line 42"),
            AnalysisError::query_error("Tree-sitter query missing 'function' capture group"),
            AnalysisError::collection_access_error("Attempted to access element 5 in collection of size 3"),
            AnalysisError::conversion_error("Line number 4294967296 exceeds u32::MAX"),
            AnalysisError::data_not_found_error("AST node for function declaration not found"),
            AnalysisError::lock_error("Cache mutex poisoned due to panic in concurrent operation"),
        ];
        
        for error in errors {
            let message = error.to_string();
            // Verify messages are descriptive (more than just error type)
            assert!(message.len() > 20, "Error message too short: {}", message);
            // Verify messages contain context
            assert!(message.contains("failed") || message.contains("not found") || 
                   message.contains("exceeds") || message.contains("poisoned"),
                   "Error message lacks context: {}", message);
        }
    }

    #[test]
    fn test_error_debug_formatting() {
        let error = AnalysisError::parse_error("Test error");
        let debug_str = format!("{:?}", error);
        
        // Verify debug formatting includes the variant name and message
        assert!(debug_str.contains("ParseError"));
        assert!(debug_str.contains("Test error"));
    }
}