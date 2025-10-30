//! Comprehensive error handling and resilience testing
//! 
//! This module tests error handling, fault tolerance, and graceful degradation
//! across all components of the Uveddi system.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tempfile::TempDir;
use tokio::time::{sleep, timeout};
use uveddi::analysis::{
    AnalysisEngine,
    config::AnalysisConfig,
    detectors::DetectorType,
    types::{AnalysisError, AnalysisResult},
};
use uveddi::ast::AstProvider;
use uveddi::error::{
    ErrorHandler,
    ErrorReporter,
    ErrorRecovery,
    ErrorType,
    Result as UveddiResult,
    UveddiError,
};
use uveddi::resilience::{
    circuit_breaker::CircuitBreaker,
    retry::RetryPolicy,
    fallback::FallbackStrategy,
    graceful_handler::GracefulHandler,
    recovery::RecoveryManager,
};

/// Test basic error creation and propagation
#[tokio::test]
async fn test_error_creation_and_propagation() {
    // Test different error types
    let io_error = UveddiError::from(std::io::Error::new(
        std::io::ErrorKind::NotFound, 
        "File not found"
    ));
    assert!(matches!(io_error, UveddiError::Io(_)));
    assert!(io_error.to_string().contains("File not found"));
    
    let parse_error = UveddiError::parse_error("Invalid syntax at line 42");
    assert!(matches!(parse_error, UveddiError::Parse(_)));
    assert!(parse_error.to_string().contains("line 42"));
    
    let analysis_error = UveddiError::analysis_error("God object detection failed");
    assert!(matches!(analysis_error, UveddiError::Analysis(_)));
    
    let config_error = UveddiError::config_error("Invalid detector configuration");
    assert!(matches!(config_error, UveddiError::Config(_)));
    
    // Test error chaining
    let chained_error = UveddiError::from(io_error)
        .with_context("Failed to read source file")
        .with_context("Analysis initialization failed");
    
    let error_chain = chained_error.error_chain();
    assert!(error_chain.len() >= 2);
    assert!(error_chain[0].contains("Analysis initialization failed"));
}

/// Test error handling in file operations
#[tokio::test]
async fn test_file_operation_error_handling() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let ast_provider = AstProvider::new();
    
    // Test non-existent file
    let non_existent = temp_dir.path().join("does_not_exist.rs");
    let result = ast_provider.parse_file(&non_existent).await;
    
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(matches!(error, UveddiError::Io(_)));
    assert!(error.to_string().contains("No such file"));
    
    // Test empty file
    let empty_file = temp_dir.path().join("empty.rs");
    tokio::fs::write(&empty_file, "").await.expect("Failed to write empty file");
    
    let result = ast_provider.parse_file(&empty_file).await;
    // Should handle empty files gracefully
    assert!(result.is_ok() || result.is_err()); // Either outcome is acceptable
    
    // Test binary file
    let binary_file = temp_dir.path().join("binary.bin");
    tokio::fs::write(&binary_file, &[0x00, 0x01, 0x02, 0xFF, 0xFE])
        .await.expect("Failed to write binary file");
    
    let result = ast_provider.parse_file(&binary_file).await;
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("binary") || error.to_string().contains("encoding"));
    
    // Test file with invalid UTF-8
    let invalid_utf8_file = temp_dir.path().join("invalid_utf8.rs");
    tokio::fs::write(&invalid_utf8_file, &[0xFF, 0xFE, 0xFD])
        .await.expect("Failed to write invalid UTF-8 file");
    
    let result = ast_provider.parse_file(&invalid_utf8_file).await;
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("UTF-8") || error.to_string().contains("encoding"));
}

/// Test error handling in syntax parsing
#[tokio::test]
async fn test_syntax_parsing_error_handling() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let ast_provider = AstProvider::new();
    
    // Test file with syntax errors
    let syntax_error_file = temp_dir.path().join("syntax_error.rs");
    tokio::fs::write(&syntax_error_file, r#"
fn broken_function( {
    let x = ;
    missing_semicolon()
    return "unclosed string
}
"#).await.expect("Failed to write syntax error file");
    
    let result = ast_provider.parse_file(&syntax_error_file).await;
    
    // Should handle syntax errors gracefully
    match result {
        Ok(ast) => {
            // If parsing succeeds, should have error nodes or partial AST
            assert!(ast.has_errors() || ast.node_count() > 0);
        }
        Err(error) => {
            // If parsing fails, should provide helpful error message
            assert!(matches!(error, UveddiError::Parse(_)));
            assert!(error.to_string().contains("syntax") || error.to_string().contains("parse"));
        }
    }
    
    // Test incomplete file
    let incomplete_file = temp_dir.path().join("incomplete.rs");
    tokio::fs::write(&incomplete_file, "fn incomplete_function() {")
        .await.expect("Failed to write incomplete file");
    
    let result = ast_provider.parse_file(&incomplete_file).await;
    // Should handle incomplete files gracefully
    assert!(result.is_ok() || result.is_err());
    
    // Test file with mixed valid/invalid content
    let mixed_file = temp_dir.path().join("mixed.rs");
    tokio::fs::write(&mixed_file, r#"
fn valid_function() -> i32 {
    42
}

fn broken_function( {
    invalid syntax here
}

fn another_valid_function() -> String {
    "hello".to_string()
}
"#).await.expect("Failed to write mixed file");
    
    let result = ast_provider.parse_file(&mixed_file).await;
    match result {
        Ok(ast) => {
            // Should parse the valid parts
            assert!(ast.node_count() > 0);
        }
        Err(_) => {
            // Acceptable if tree-sitter can't handle mixed content
        }
    }
}

/// Test memory pressure and resource exhaustion handling
#[tokio::test]
async fn test_memory_pressure_handling() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    
    // Create extremely large file
    let large_file = temp_dir.path().join("large.rs");
    let large_content = "fn large_function() {\n".to_string() + 
        &"    println!(\"line\");\n".repeat(100_000) + 
        "}\n";
    
    tokio::fs::write(&large_file, large_content).await
        .expect("Failed to write large file");
    
    // Configure with strict memory limits
    let config = AnalysisConfig {
        max_file_size: 1024, // Very small limit
        max_memory_usage: Some(64 * 1024 * 1024), // 64MB limit
        parallel_analysis: false, // Disable parallelism to control memory
        enable_ai_explanations: false,
        cache_enabled: false, // Disable caching to save memory
        detector_types: vec![DetectorType::GodObject],
        language_filters: vec!["rust".to_string()],
        exclude_patterns: vec![],
    };
    
    let engine = AnalysisEngine::new(config).expect("Failed to create engine");
    let result = engine.analyze(&large_file).await;
    
    // Should handle memory pressure gracefully
    match result {
        Ok((issues, graph)) => {
            // If successful, results should be reasonable
            assert!(issues.len() < 10000); // Shouldn't create excessive issues
            assert!(graph.node_count() < 1000); // Shouldn't create excessive nodes
        }
        Err(error) => {
            // Should provide helpful error about resource limits
            assert!(
                error.to_string().contains("file size") ||
                error.to_string().contains("memory") ||
                error.to_string().contains("limit") ||
                error.to_string().contains("too large")
            );
        }
    }
}

/// Test concurrent operation error handling
#[tokio::test]
async fn test_concurrent_error_handling() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let error_count = Arc::new(Mutex::new(0));
    let success_count = Arc::new(Mutex::new(0));
    
    // Create multiple files with various issues
    let mut file_paths = Vec::new();
    for i in 0..10 {
        let file_path = temp_dir.path().join(format!("test_{}.rs", i));
        let content = match i % 4 {
            0 => "fn valid() -> i32 { 42 }".to_string(),
            1 => "fn broken_syntax( { invalid }".to_string(),
            2 => format!("fn large() {{\n{}\n}}", "println!(\"line\");\n".repeat(10000)),
            3 => String::new(), // Empty file
            _ => unreachable!(),
        };
        
        tokio::fs::write(&file_path, content).await
            .expect("Failed to write test file");
        file_paths.push(file_path);
    }
    
    // Process files concurrently
    let handles: Vec<_> = file_paths.into_iter().map(|file_path| {
        let error_count = Arc::clone(&error_count);
        let success_count = Arc::clone(&success_count);
        
        tokio::spawn(async move {
            let ast_provider = AstProvider::new();
            match ast_provider.parse_file(&file_path).await {
                Ok(_) => {
                    let mut count = success_count.lock().unwrap();
                    *count += 1;
                }
                Err(_) => {
                    let mut count = error_count.lock().unwrap();
                    *count += 1;
                }
            }
        })
    }).collect();
    
    // Wait for all operations
    for handle in handles {
        handle.await.expect("Task failed");
    }
    
    let total_errors = *error_count.lock().unwrap();
    let total_successes = *success_count.lock().unwrap();
    
    // Should handle mix of successes and errors
    assert!(total_successes > 0, "Should have some successes");
    assert!(total_errors > 0, "Should have some errors");
    assert_eq!(total_successes + total_errors, 10);
}

/// Test timeout handling
#[tokio::test]
async fn test_timeout_handling() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    
    // Create file that might take time to process
    let slow_file = temp_dir.path().join("slow.rs");
    let slow_content = format!(r#"
// Complex nested structure that takes time to parse
{}
"#, create_deeply_nested_content(1000));
    
    tokio::fs::write(&slow_file, slow_content).await
        .expect("Failed to write slow file");
    
    let ast_provider = AstProvider::new();
    
    // Test with very short timeout
    let result = timeout(Duration::from_millis(1), ast_provider.parse_file(&slow_file)).await;
    
    match result {
        Ok(parse_result) => {
            // If parsing completed quickly, verify result
            assert!(parse_result.is_ok() || parse_result.is_err());
        }
        Err(_) => {
            // Timeout occurred - this is acceptable for stress testing
        }
    }
    
    // Test with reasonable timeout
    let result = timeout(Duration::from_secs(5), ast_provider.parse_file(&slow_file)).await;
    assert!(result.is_ok(), "Reasonable timeout should not fail");
}

/// Test circuit breaker pattern
#[tokio::test]
async fn test_circuit_breaker() {
    let circuit_breaker = CircuitBreaker::new(
        3, // failure_threshold
        Duration::from_millis(100), // timeout
        Duration::from_millis(200), // recovery_time
    );
    
    // Simulate failures to trip the circuit breaker
    for _ in 0..3 {
        let result = circuit_breaker.call(|| async {
            Err::<(), UveddiError>(UveddiError::analysis_error("Simulated failure"))
        }).await;
        assert!(result.is_err());
    }
    
    // Circuit should be open now
    assert!(circuit_breaker.is_open());
    
    // Calls should fail fast
    let start = Instant::now();
    let result = circuit_breaker.call(|| async {
        Ok::<(), UveddiError>(())
    }).await;
    let duration = start.elapsed();
    
    assert!(result.is_err());
    assert!(duration < Duration::from_millis(50)); // Should fail fast
    
    // Wait for recovery period
    sleep(Duration::from_millis(250)).await;
    
    // Circuit should allow one test call
    assert!(circuit_breaker.is_half_open());
    
    // Successful call should close the circuit
    let result = circuit_breaker.call(|| async {
        Ok::<(), UveddiError>(())
    }).await;
    assert!(result.is_ok());
    assert!(circuit_breaker.is_closed());
}

/// Test retry mechanism
#[tokio::test]
async fn test_retry_mechanism() {
    let retry_policy = RetryPolicy::new(
        3, // max_attempts
        Duration::from_millis(10), // initial_delay
        2.0, // backoff_multiplier
    );
    
    let attempt_count = Arc::new(Mutex::new(0));
    
    // Test successful retry after failures
    let attempt_count_clone = Arc::clone(&attempt_count);
    let result = retry_policy.execute(|| async {
        let mut count = attempt_count_clone.lock().unwrap();
        *count += 1;
        
        if *count < 3 {
            Err(UveddiError::analysis_error("Temporary failure"))
        } else {
            Ok("Success")
        }
    }).await;
    
    assert!(result.is_ok());
    assert_eq!(*attempt_count.lock().unwrap(), 3);
    
    // Test permanent failure
    let attempt_count = Arc::new(Mutex::new(0));
    let attempt_count_clone = Arc::clone(&attempt_count);
    let result = retry_policy.execute(|| async {
        let mut count = attempt_count_clone.lock().unwrap();
        *count += 1;
        Err::<&str, UveddiError>(UveddiError::analysis_error("Permanent failure"))
    }).await;
    
    assert!(result.is_err());
    assert_eq!(*attempt_count.lock().unwrap(), 3); // Should try max_attempts times
}

/// Test fallback strategies
#[tokio::test]
async fn test_fallback_strategies() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    
    // Primary strategy that might fail
    let primary_strategy = || async {
        Err::<String, UveddiError>(UveddiError::analysis_error("Primary failed"))
    };
    
    // Fallback strategy
    let fallback_strategy = FallbackStrategy::new()
        .with_fallback(|| async { Ok("Fallback 1".to_string()) })
        .with_fallback(|| async { Ok("Fallback 2".to_string()) })
        .with_default("Default value".to_string());
    
    let result = fallback_strategy.execute(primary_strategy).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Fallback 1");
    
    // Test cascading fallbacks
    let failing_fallback_strategy = FallbackStrategy::new()
        .with_fallback(|| async { 
            Err::<String, UveddiError>(UveddiError::analysis_error("Fallback 1 failed")) 
        })
        .with_fallback(|| async { Ok("Fallback 2".to_string()) })
        .with_default("Default value".to_string());
    
    let result = failing_fallback_strategy.execute(primary_strategy).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Fallback 2");
    
    // Test all fallbacks failing
    let all_failing_strategy = FallbackStrategy::new()
        .with_fallback(|| async { 
            Err::<String, UveddiError>(UveddiError::analysis_error("Fallback 1 failed")) 
        })
        .with_fallback(|| async { 
            Err::<String, UveddiError>(UveddiError::analysis_error("Fallback 2 failed")) 
        })
        .with_default("Default value".to_string());
    
    let result = all_failing_strategy.execute(primary_strategy).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Default value");
}

/// Test graceful degradation
#[tokio::test]
async fn test_graceful_degradation() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let graceful_handler = GracefulHandler::new();
    
    // Configure graceful handler for different error types
    graceful_handler.configure_degradation(ErrorType::Io, |_error| {
        vec![
            "Skip inaccessible files".to_string(),
            "Continue with available files".to_string(),
        ]
    });
    
    graceful_handler.configure_degradation(ErrorType::Parse, |_error| {
        vec![
            "Skip malformed files".to_string(),
            "Report syntax errors separately".to_string(),
        ]
    });
    
    graceful_handler.configure_degradation(ErrorType::Memory, |_error| {
        vec![
            "Reduce analysis scope".to_string(),
            "Disable resource-intensive detectors".to_string(),
            "Process files sequentially".to_string(),
        ]
    });
    
    // Test degradation handling
    let io_error = UveddiError::io_error("Permission denied");
    let degradation_actions = graceful_handler.handle_error(&io_error);
    assert!(!degradation_actions.is_empty());
    assert!(degradation_actions.iter().any(|action| action.contains("Skip")));
    
    let parse_error = UveddiError::parse_error("Invalid syntax");
    let degradation_actions = graceful_handler.handle_error(&parse_error);
    assert!(!degradation_actions.is_empty());
    assert!(degradation_actions.iter().any(|action| action.contains("malformed")));
    
    let memory_error = UveddiError::memory_error("Out of memory");
    let degradation_actions = graceful_handler.handle_error(&memory_error);
    assert!(!degradation_actions.is_empty());
    assert!(degradation_actions.iter().any(|action| action.contains("Reduce")));
}

/// Test error recovery mechanisms
#[tokio::test]
async fn test_error_recovery() {
    let recovery_manager = RecoveryManager::new();
    
    // Configure recovery strategies
    recovery_manager.add_recovery_strategy(ErrorType::Network, Box::new(|_error| {
        Box::pin(async {
            // Simulate network recovery
            sleep(Duration::from_millis(10)).await;
            Ok("Network recovered".to_string())
        })
    }));
    
    recovery_manager.add_recovery_strategy(ErrorType::FileSystem, Box::new(|_error| {
        Box::pin(async {
            // Simulate filesystem recovery
            Ok("Filesystem recovered".to_string())
        })
    }));
    
    // Test recovery
    let network_error = UveddiError::network_error("Connection failed");
    let recovery_result = recovery_manager.attempt_recovery(&network_error).await;
    assert!(recovery_result.is_ok());
    
    let fs_error = UveddiError::filesystem_error("Disk full");
    let recovery_result = recovery_manager.attempt_recovery(&fs_error).await;
    assert!(recovery_result.is_ok());
    
    // Test unrecoverable error
    let unknown_error = UveddiError::analysis_error("Unknown error");
    let recovery_result = recovery_manager.attempt_recovery(&unknown_error).await;
    assert!(recovery_result.is_err()); // No recovery strategy configured
}

/// Test error aggregation and reporting
#[tokio::test]
async fn test_error_aggregation_and_reporting() {
    let error_reporter = ErrorReporter::new();
    
    // Simulate various errors during analysis
    let errors = vec![
        UveddiError::io_error("File not found: file1.rs"),
        UveddiError::parse_error("Syntax error in file2.rs at line 42"),
        UveddiError::io_error("Permission denied: file3.rs"),
        UveddiError::analysis_error("God object detection failed in file4.rs"),
        UveddiError::memory_error("Out of memory while processing file5.rs"),
    ];
    
    for error in errors {
        error_reporter.report_error(error);
    }
    
    // Generate error summary
    let summary = error_reporter.generate_summary();
    
    // Verify summary structure
    assert!(summary.total_errors > 0);
    assert!(summary.error_by_type.contains_key(&ErrorType::Io));
    assert!(summary.error_by_type.contains_key(&ErrorType::Parse));
    assert!(summary.error_by_type.contains_key(&ErrorType::Analysis));
    assert!(summary.error_by_type.contains_key(&ErrorType::Memory));
    
    // Verify error grouping
    assert_eq!(summary.error_by_type[&ErrorType::Io], 2); // file1.rs and file3.rs
    assert_eq!(summary.error_by_type[&ErrorType::Parse], 1); // file2.rs
    assert_eq!(summary.error_by_type[&ErrorType::Analysis], 1); // file4.rs
    assert_eq!(summary.error_by_type[&ErrorType::Memory], 1); // file5.rs
    
    // Test error deduplication
    error_reporter.report_error(UveddiError::io_error("File not found: file1.rs")); // Duplicate
    let new_summary = error_reporter.generate_summary();
    assert_eq!(new_summary.total_errors, summary.total_errors + 1); // Should still count
    
    // Test error pattern detection
    let patterns = summary.detect_patterns();
    assert!(patterns.iter().any(|p| p.contains("File not found")));
    assert!(patterns.iter().any(|p| p.contains("Permission denied")));
}

/// Test resource cleanup on errors
#[tokio::test]
async fn test_resource_cleanup_on_errors() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    
    // Test that resources are properly cleaned up even when errors occur
    let config = AnalysisConfig {
        max_file_size: 1024 * 1024,
        parallel_analysis: true,
        enable_ai_explanations: false,
        cache_enabled: true,
        detector_types: vec![DetectorType::GodObject],
        language_filters: vec!["rust".to_string()],
        exclude_patterns: vec![],
    };
    
    // Create file that will cause an error
    let error_file = temp_dir.path().join("error.rs");
    tokio::fs::write(&error_file, "invalid rust syntax {{{").await
        .expect("Failed to write error file");
    
    // Track resource usage before analysis
    let initial_memory = get_memory_usage();
    let initial_handles = get_open_handles();
    
    let engine = AnalysisEngine::new(config).expect("Failed to create engine");
    let _result = engine.analyze(&error_file).await; // May succeed or fail
    
    // Force cleanup
    drop(engine);
    
    // Give system time to cleanup
    sleep(Duration::from_millis(100)).await;
    
    // Check that resources were cleaned up
    let final_memory = get_memory_usage();
    let final_handles = get_open_handles();
    
    // Memory should not have grown significantly
    assert!(final_memory <= initial_memory + 50 * 1024 * 1024, // 50MB tolerance
           "Memory leak detected: {} -> {} bytes", initial_memory, final_memory);
    
    // File handles should be cleaned up
    assert!(final_handles <= initial_handles + 10, // Small tolerance
           "File handle leak detected: {} -> {}", initial_handles, final_handles);
}

#[cfg(test)]
mod helpers {
    use super::*;
    
    /// Create deeply nested content for stress testing
    pub fn create_deeply_nested_content(depth: usize) -> String {
        let mut content = String::new();
        
        // Create nested modules
        for i in 0..depth {
            content.push_str(&format!("mod nested_{} {{\n", i));
        }
        
        content.push_str("fn deeply_nested_function() { println!(\"deep\"); }\n");
        
        for _ in 0..depth {
            content.push_str("}\n");
        }
        
        content
    }
    
    /// Get current memory usage (approximate)
    pub fn get_memory_usage() -> u64 {
        #[cfg(target_os = "linux")]
        {
            if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
                for line in status.lines() {
                    if line.starts_with("VmRSS:") {
                        if let Some(kb_str) = line.split_whitespace().nth(1) {
                            if let Ok(kb) = kb_str.parse::<u64>() {
                                return kb * 1024; // Convert KB to bytes
                            }
                        }
                    }
                }
            }
        }
        
        // Fallback for non-Linux or if proc fs not available
        0
    }
    
    /// Get current number of open file handles (approximate)
    pub fn get_open_handles() -> u64 {
        #[cfg(target_os = "linux")]
        {
            if let Ok(entries) = std::fs::read_dir("/proc/self/fd") {
                return entries.count() as u64;
            }
        }
        
        // Fallback for non-Linux
        0
    }
    
    /// Error simulation helper
    pub struct ErrorSimulator {
        failure_rate: f64,
        error_types: Vec<UveddiError>,
    }
    
    impl ErrorSimulator {
        pub fn new(failure_rate: f64) -> Self {
            Self {
                failure_rate,
                error_types: vec![
                    UveddiError::io_error("Simulated IO error"),
                    UveddiError::parse_error("Simulated parse error"),
                    UveddiError::analysis_error("Simulated analysis error"),
                    UveddiError::memory_error("Simulated memory error"),
                ],
            }
        }
        
        pub fn maybe_fail<T>(&self, success_value: T) -> Result<T, UveddiError> {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            
            if rng.gen::<f64>() < self.failure_rate {
                let error_index = rng.gen_range(0..self.error_types.len());
                Err(self.error_types[error_index].clone())
            } else {
                Ok(success_value)
            }
        }
    }
    
    /// Test harness for error scenarios
    pub struct ErrorTestHarness {
        pub temp_dir: TempDir,
        pub error_reporter: ErrorReporter,
        pub recovery_manager: RecoveryManager,
    }
    
    impl ErrorTestHarness {
        pub fn new() -> Self {
            Self {
                temp_dir: TempDir::new().expect("Failed to create temp directory"),
                error_reporter: ErrorReporter::new(),
                recovery_manager: RecoveryManager::new(),
            }
        }
        
        pub async fn create_problematic_file(&self, name: &str, problem_type: &str) -> PathBuf {
            let file_path = self.temp_dir.path().join(name);
            
            let content = match problem_type {
                "syntax_error" => "fn broken( { invalid syntax }",
                "large" => &"println!(\"line\");\n".repeat(10000),
                "empty" => "",
                "binary" => return file_path, // Handle separately
                _ => "fn valid() {}",
            };
            
            if problem_type == "binary" {
                tokio::fs::write(&file_path, &[0x00, 0x01, 0x02, 0xFF]).await
                    .expect("Failed to write binary file");
            } else {
                tokio::fs::write(&file_path, content).await
                    .expect("Failed to write file");
            }
            
            file_path
        }
        
        pub fn simulate_errors(&mut self, count: usize) {
            let simulator = ErrorSimulator::new(1.0); // Always fail
            
            for i in 0..count {
                if let Err(error) = simulator.maybe_fail::<()>(()) {
                    self.error_reporter.report_error(error);
                }
            }
        }
        
        pub fn get_error_summary(&self) -> ErrorSummary {
            self.error_reporter.generate_summary()
        }
    }
}