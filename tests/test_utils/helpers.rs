//! Test helper functions for setup, teardown, and common operations
//!
//! This module provides utility functions that are commonly used across
//! multiple test files to reduce code duplication and improve maintainability.

use std::path::Path;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::fs;
use tokio::time::{timeout, Duration};

use crate::analysis::config::AnalysisConfig;
use crate::analysis::errors::AnalysisError;
use crate::analysis::traits::{AstParserTrait, DependencyExtractorTrait, ResultCacheTrait};

/// Returns a temporary directory for test isolation
pub fn temp_test_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp test dir")
}

/// Creates a sample Rust file for testing
pub async fn create_sample_rust_file(dir: &TempDir, filename: &str, content: &str) -> Result<std::path::PathBuf, std::io::Error> {
    let file_path = dir.path().join(filename);
    fs::write(&file_path, content).await?;
    Ok(file_path)
}

/// Creates a sample Python file for testing
pub async fn create_sample_python_file(dir: &TempDir, filename: &str, content: &str) -> Result<std::path::PathBuf, std::io::Error> {
    let file_path = dir.path().join(filename);
    fs::write(&file_path, content).await?;
    Ok(file_path)
}

/// Creates a sample JavaScript file for testing
pub async fn create_sample_js_file(dir: &TempDir, filename: &str, content: &str) -> Result<std::path::PathBuf, std::io::Error> {
    let file_path = dir.path().join(filename);
    fs::write(&file_path, content).await?;
    Ok(file_path)
}

/// Creates a basic test configuration for analysis
pub fn create_test_config() -> AnalysisConfig {
    AnalysisConfig {
        project_root: "/tmp/test_project".to_string(),
        excluded_paths: vec![".git".to_string(), "target".to_string()],
        language_configs: std::collections::HashMap::new(),
        performance_config: None,
        plugin_configs: std::collections::HashMap::new(),
        cache_config: None,
        ai_config: None,
    }
}

/// Async test timeout wrapper
pub async fn with_timeout<F, R>(duration: Duration, future: F) -> Result<R, tokio::time::error::Elapsed>
where
    F: std::future::Future<Output = R>,
{
    timeout(duration, future).await
}

/// Setup function for async tests that need temporary directories
pub async fn setup_async_test() -> (TempDir, AnalysisConfig) {
    let temp_dir = temp_test_dir();
    let config = create_test_config();
    (temp_dir, config)
}

/// Cleanup function for async tests
pub async fn cleanup_async_test(_temp_dir: TempDir) {
    // TempDir automatically cleans up when dropped
    // This function is for future cleanup needs
}

/// Asserts that a path exists
pub fn assert_path_exists(path: &Path) {
    assert!(path.exists(), "Path should exist: {}", path.display());
}

/// Asserts that a path does not exist
pub fn assert_path_not_exists(path: &Path) {
    assert!(!path.exists(), "Path should not exist: {}", path.display());
}

/// Wait for a condition to be true (useful for async tests)
pub async fn wait_for_condition<F>(
    condition: F,
    timeout_duration: Duration,
    poll_interval: Duration,
) -> Result<(), String>
where
    F: Fn() -> bool,
{
    let start = std::time::Instant::now();
    
    while start.elapsed() < timeout_duration {
        if condition() {
            return Ok(());
        }
        tokio::time::sleep(poll_interval).await;
    }
    
    Err("Condition was not met within timeout".to_string())
}

/// Creates a mock error for testing error handling
pub fn create_mock_error(message: &str) -> AnalysisError {
    AnalysisError::ParseError {
        message: message.to_string(),
    }
}

/// Test result type for convenience
pub type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

/// Macro for creating async test functions with common setup
#[macro_export]
macro_rules! async_test {
    ($name:ident, $body:expr) => {
        #[tokio::test]
        async fn $name() -> TestResult {
            let (temp_dir, config) = setup_async_test().await;
            
            let result = $body(temp_dir, config).await;
            
            // Cleanup happens automatically when temp_dir is dropped
            result
        }
    };
}

/// Macro for creating async test functions with timeout
#[macro_export]
macro_rules! async_test_with_timeout {
    ($name:ident, $timeout:expr, $body:expr) => {
        #[tokio::test]
        async fn $name() -> TestResult {
            let (temp_dir, config) = setup_async_test().await;
            
            let result = with_timeout($timeout, $body(temp_dir, config)).await
                .map_err(|_| "Test timed out".to_string())?;
            
            result
        }
    };
}
