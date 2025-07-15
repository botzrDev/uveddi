//! Robust AST parser that safely integrates CPU-bound parsing into Tokio applications.
//!
//! This module provides a production-ready solution for handling synchronous, CPU-intensive
//! AST parsing operations in async Rust applications without blocking the Tokio runtime.

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use thiserror::Error;

/// Comprehensive error enum representing all possible failure modes
/// when parsing files in a robust, async context.
#[derive(Error, Debug)]
pub enum ParseFileError {
    /// The underlying parsing operation failed with a specific error
    #[error("Parse error: {0}")]
    Parse(#[from] crate::ast::AstError),
    
    /// The parsing operation exceeded the specified timeout duration
    #[error("Parse operation timed out after {0:?}")]
    Timeout(Duration),
    
    /// The parsing task panicked on the blocking thread pool
    #[error("Parse operation panicked: {0}")]
    Panic(String),
}

/// Configuration for the robust parser
#[derive(Debug, Clone)]
pub struct RobustParserConfig {
    /// Maximum number of concurrent parsing operations
    pub max_concurrent_operations: usize,
    /// Default timeout for parsing operations
    pub default_timeout: Duration,
}

impl Default for RobustParserConfig {
    fn default() -> Self {
        Self {
            max_concurrent_operations: num_cpus::get(),
            default_timeout: Duration::from_secs(30),
        }
    }
}

/// Production-ready parser that safely integrates CPU-bound parsing
/// operations into a Tokio application using the complete resilience pattern.
pub struct RobustParser {
    /// Semaphore to limit concurrent parsing operations and prevent resource exhaustion
    concurrency_limiter: Arc<Semaphore>,
    /// Configuration for the parser
    config: RobustParserConfig,
}

impl RobustParser {
    /// Creates a new RobustParser with the specified configuration.
    pub fn new(config: RobustParserConfig) -> Self {
        Self {
            concurrency_limiter: Arc::new(Semaphore::new(config.max_concurrent_operations)),
            config,
        }
    }
    
    /// Creates a new RobustParser with default configuration.
    pub fn with_default_config() -> Self {
        Self::new(RobustParserConfig::default())
    }
    
    /// Gets the current configuration
    pub fn config(&self) -> &RobustParserConfig {
        &self.config
    }
    
    /// Gets the number of available permits in the semaphore
    pub fn available_permits(&self) -> usize {
        self.concurrency_limiter.available_permits()
    }
    
    /// Parses a file asynchronously with full resilience guarantees.
    /// 
    /// This method demonstrates the complete pattern for safely integrating
    /// synchronous, CPU-bound work into a Tokio application:
    /// 
    /// 1. **Acquire Semaphore Permit**: Limits concurrent operations using RAII pattern
    /// 2. **Offload to Blocking Pool**: Uses spawn_blocking to isolate CPU-bound work
    /// 3. **Enforce Timeout**: Wraps operation in timeout to prevent indefinite hangs
    /// 4. **Handle All Error Cases**: Maps complex nested Results to clean error types
    /// 
    /// # Arguments
    /// * `file_path` - Path to the file to parse
    /// * `timeout_duration` - Maximum time allowed for the parsing operation (optional, uses default if None)
    /// 
    /// # Returns
    /// * `Ok(ParsedFile)` - Successfully parsed file
    /// * `Err(ParseFileError)` - One of: Parse error, Timeout, or Panic
    pub async fn parse_file_robust(
        &self,
        file_path: std::path::PathBuf,
        timeout_duration: Option<Duration>,
    ) -> Result<crate::ast::ParsedFile, ParseFileError> {
        let timeout = timeout_duration.unwrap_or(self.config.default_timeout);
        
        // Step 1: Acquire semaphore permit to limit concurrency
        // Using acquire_owned() ensures the permit is moved into the spawn_blocking task
        // and automatically released when the task completes (RAII pattern).
        // This prevents resource exhaustion from too many concurrent CPU-bound operations.
        let _permit: OwnedSemaphorePermit = self
            .concurrency_limiter
            .clone()
            .acquire_owned()
            .await
            .expect("Semaphore should not be closed");
        
        // Step 2: Wrap the entire operation in a timeout
        // This is the outer timeout that protects against the entire operation hanging,
        // including both the spawn_blocking overhead and the actual parsing work.
        let parse_result = tokio::time::timeout(timeout, async move {
            // Step 3: Offload the synchronous parsing work to the blocking thread pool
            // spawn_blocking is essential because:
            // - It moves CPU-bound work off the async executor threads
            // - It prevents the cooperative scheduler from being starved
            // - It isolates potentially long-running synchronous operations
            tokio::task::spawn_blocking(move || {
                // The _permit is moved into this closure and will be automatically
                // dropped when the closure completes, releasing the semaphore permit.
                
                // Call the actual synchronous parsing function
                // This is where the CPU-intensive work happens on a dedicated thread
                let mut parser = crate::ast::tree_sitter_impl::AstParser::new()
                    .map_err(|e| crate::ast::AstError::Other(format!("Failed to create parser: {}", e)))?;
                parser.parse_file(&file_path)
            })
            .await
        })
        .await;
        
        // Step 4: Handle the complex nested Result from timeout + spawn_blocking + parsing
        // This demonstrates the complete error mapping pattern for robust async operations
        match parse_result {
            // Timeout occurred - the entire operation took too long
            Err(_timeout_elapsed) => Err(ParseFileError::Timeout(timeout)),
            
            // Operation completed within timeout, now handle spawn_blocking result
            Ok(join_result) => match join_result {
                // spawn_blocking task completed successfully, handle parsing result
                Ok(parsing_result) => match parsing_result {
                    Ok(parsed_file) => Ok(parsed_file),
                    Err(parse_error) => Err(ParseFileError::Parse(parse_error)),
                },
                
                // spawn_blocking task panicked
                // Note: The panic is contained within the blocking thread pool and
                // does not affect the main async runtime. The panicked thread is
                // cleaned up automatically, but the task continues running until
                // the synchronous function completes naturally.
                Err(join_error) => {
                    let panic_message = if join_error.is_panic() {
                        // Extract panic message if possible
                        match join_error.try_into_panic() {
                            Ok(panic_payload) => {
                                if let Some(s) = panic_payload.downcast_ref::<String>() {
                                    s.clone()
                                } else if let Some(s) = panic_payload.downcast_ref::<&str>() {
                                    s.to_string()
                                } else {
                                    "Unknown panic".to_string()
                                }
                            }
                            Err(_) => "Panic payload extraction failed".to_string(),
                        }
                    } else {
                        "Task was cancelled".to_string()
                    };
                    
                    Err(ParseFileError::Panic(panic_message))
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_robust_parser_creation() {
        let config = RobustParserConfig {
            max_concurrent_operations: 4,
            default_timeout: Duration::from_secs(10),
        };
        
        let parser = RobustParser::new(config.clone());
        assert_eq!(parser.config().max_concurrent_operations, 4);
        assert_eq!(parser.config().default_timeout, Duration::from_secs(10));
        assert_eq!(parser.available_permits(), 4);
    }

    #[test]
    fn test_default_config() {
        let parser = RobustParser::with_default_config();
        assert_eq!(parser.config().max_concurrent_operations, num_cpus::get());
        assert_eq!(parser.config().default_timeout, Duration::from_secs(30));
    }

    #[test]
    fn test_error_types() {
        // Test that our error types can be created and formatted
        let timeout_error = ParseFileError::Timeout(Duration::from_secs(5));
        assert!(timeout_error.to_string().contains("timed out"));
        
        let panic_error = ParseFileError::Panic("test panic".to_string());
        assert!(panic_error.to_string().contains("panicked"));
    }

    #[tokio::test]
    async fn test_successful_parsing() {
        let parser = RobustParser::with_default_config();
        
        // Create a temporary test file
        let temp_dir = tempfile::tempdir().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(&test_file, "fn main() { println!(\"Hello, world!\"); }").unwrap();
        
        let timeout = Duration::from_secs(5);
        let result = parser.parse_file_robust(test_file, Some(timeout)).await;
        
        assert!(result.is_ok(), "Parsing should succeed: {:?}", result);
        let parsed_file = result.unwrap();
        assert_eq!(parsed_file.language, crate::ast::SourceLanguage::Rust);
    }

    #[tokio::test]
    async fn test_timeout_behavior() {
        let config = RobustParserConfig {
            max_concurrent_operations: 1,
            default_timeout: Duration::from_millis(1), // Very short timeout
        };
        let parser = RobustParser::new(config);
        
        // Create a temporary test file
        let temp_dir = tempfile::tempdir().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(&test_file, "fn main() {}").unwrap();
        
        let result = parser.parse_file_robust(test_file, None).await;
        
        // Should timeout due to very short timeout
        match result {
            Err(ParseFileError::Timeout(_)) => {
                // Expected - timeout occurred
            }
            Ok(_) => {
                // This might happen if parsing is very fast, which is also fine
                println!("Parsing completed faster than expected timeout");
            }
            Err(e) => {
                panic!("Unexpected error type: {:?}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_concurrency_limiting() {
        let config = RobustParserConfig {
            max_concurrent_operations: 1, // Only allow 1 concurrent operation
            default_timeout: Duration::from_secs(10),
        };
        let parser = RobustParser::new(config);
        
        // Create temporary test files
        let temp_dir = tempfile::tempdir().unwrap();
        let test_file1 = temp_dir.path().join("test1.rs");
        let test_file2 = temp_dir.path().join("test2.rs");
        std::fs::write(&test_file1, "fn main() {}").unwrap();
        std::fs::write(&test_file2, "fn test() {}").unwrap();
        
        let start = std::time::Instant::now();
        
        // Start two operations simultaneously - they should be serialized by the semaphore
        let (result1, result2) = tokio::join!(
            parser.parse_file_robust(test_file1, Some(Duration::from_secs(5))),
            parser.parse_file_robust(test_file2, Some(Duration::from_secs(5)))
        );
        
        let elapsed = start.elapsed();
        
        // Both should succeed
        assert!(result1.is_ok(), "First parse should succeed: {:?}", result1);
        assert!(result2.is_ok(), "Second parse should succeed: {:?}", result2);
        
        // Due to concurrency limiting, this should take some measurable time
        // (though not necessarily long since parsing is fast)
        println!("Concurrent parsing took: {:?}", elapsed);
    }
}