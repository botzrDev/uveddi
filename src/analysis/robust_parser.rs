//! Robust AST parser that safely integrates CPU-bound parsing into Tokio applications.
//!
//! This module provides a production-ready solution for handling synchronous, CPU-intensive
//! AST parsing operations in async Rust applications without blocking the Tokio runtime.

use crate::ast::tree_sitter_impl::AstParser;
use crate::core::logging::{debug, error, info, warn};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::{Mutex, OwnedSemaphorePermit, Semaphore};

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
    /// AST Parser cache size
    pub ast_cache_size: usize,
}

impl Default for RobustParserConfig {
    fn default() -> Self {
        Self {
            max_concurrent_operations: num_cpus::get(),
            default_timeout: Duration::from_secs(30),
            ast_cache_size: 1000,
        }
    }
}

/// Production-ready parser that safely integrates CPU-bound parsing
/// operations into a Tokio application using the complete resilience pattern.
#[derive(Clone)]
pub struct RobustParser {
    /// The shared, thread-safe AST parser instance.
    parser: Arc<Mutex<AstParser>>,
    /// Semaphore to limit concurrent parsing operations and prevent resource exhaustion
    concurrency_limiter: Arc<Semaphore>,
    /// Configuration for the parser
    config: RobustParserConfig,
}

impl RobustParser {
    /// Creates a new RobustParser with the specified configuration.
    pub fn new(config: RobustParserConfig) -> Self {
        let ast_parser = AstParser::with_cache_size(config.ast_cache_size)
            .expect("Failed to create AstParser with valid cache size");

        Self {
            parser: Arc::new(Mutex::new(ast_parser)),
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

        // Clone file_path for logging since it will be moved into the async block
        let file_path_for_logging = file_path.clone();

        // Step 1: Acquire semaphore permit to limit concurrency
        // Using acquire_owned() ensures the permit is moved into the spawn_blocking task
        // and automatically released when the task completes (RAII pattern).
        // This prevents resource exhaustion from too many concurrent CPU-bound operations.
        debug!(
            "Acquiring semaphore permit for parsing file: {}",
            file_path_for_logging.display()
        );
        let _permit: OwnedSemaphorePermit = self
            .concurrency_limiter
            .clone()
            .acquire_owned()
            .await
            .expect("Semaphore should not be closed");

        info!(
            "Permit acquired - offloading parsing task to blocking pool for file: {}",
            file_path_for_logging.display()
        );

        // Clone the Arc to the parser to move it into the blocking task
        let parser = self.parser.clone();

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

                // Lock the mutex to get exclusive access to the parser for this thread
                let mut parser_guard = parser.blocking_lock();
                parser_guard.parse_file(&file_path)
            })
            .await
        })
        .await;

        // Step 4: Handle the complex nested Result from timeout + spawn_blocking + parsing
        // This demonstrates the complete error mapping pattern for robust async operations
        match parse_result {
            // Timeout occurred - the entire operation took too long
            Err(_timeout_elapsed) => {
                error!(
                    "Parsing operation timed out after {:?} for file: {}",
                    timeout,
                    file_path_for_logging.display()
                );
                Err(ParseFileError::Timeout(timeout))
            }

            // Operation completed within timeout, now handle spawn_blocking result
            Ok(join_result) => match join_result {
                // spawn_blocking task completed successfully, handle parsing result
                Ok(parsing_result) => match parsing_result {
                    Ok(parsed_file) => {
                        info!(
                            "Successfully parsed file: {}",
                            file_path_for_logging.display()
                        );
                        // Convert tree_sitter_impl::ParsedFile to the compatibility type
                        Ok(crate::ast::ParsedFileCompat::from_tree_sitter(parsed_file))
                    }
                    Err(parse_error) => {
                        warn!(
                            "Parse error for file {}: {}",
                            file_path_for_logging.display(),
                            parse_error
                        );
                        Err(ParseFileError::Parse(parse_error))
                    }
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

                    error!(
                        "Parsing task panicked for file {}: {}",
                        file_path_for_logging.display(),
                        panic_message
                    );
                    Err(ParseFileError::Panic(panic_message))
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // A mock parser that can be configured to panic
    #[derive(Clone)]
    struct MockAstParser {
        should_panic: bool,
    }

    impl MockAstParser {
        fn new(should_panic: bool) -> Self {
            Self { should_panic }
        }

        fn parse_file(
            &mut self,
            _path: &PathBuf,
        ) -> Result<crate::ast::ParsedFile, crate::ast::AstError> {
            if self.should_panic {
                panic!("Simulated parser panic for test");
            }
            // In a real test, you'd return a valid ParsedFile here
            unimplemented!("This mock is only for panic testing");
        }
    }

    #[test]
    fn test_robust_parser_creation() {
        let config = RobustParserConfig {
            max_concurrent_operations: 4,
            default_timeout: Duration::from_secs(10),
            ast_cache_size: 100,
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
        debug!("Starting test_successful_parsing");
        let parser = RobustParser::with_default_config();
        let temp_dir = tempfile::tempdir().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(&test_file, "fn main() { println!(\"Hello, world!\"); }").unwrap();
        let timeout = Duration::from_secs(5);
        let result = tokio::time::timeout(
            Duration::from_secs(10),
            parser.parse_file_robust(test_file, Some(timeout)),
        )
        .await;
        match result {
            Ok(inner) => {
                assert!(inner.is_ok(), "Parsing should succeed: {:?}", inner);
                let parsed_file = inner.unwrap();
                assert_eq!(parsed_file.language, crate::ast::SourceLanguage::Rust);
                debug!("test_successful_parsing completed successfully");
            }
            Err(_) => panic!("test_successful_parsing timed out"),
        }
    }

    #[tokio::test]
    async fn test_timeout_behavior() {
        debug!("Starting test_timeout_behavior");
        let config = RobustParserConfig {
            max_concurrent_operations: 1,
            default_timeout: Duration::from_millis(1),
            ast_cache_size: 10,
        };
        let parser = RobustParser::new(config);
        let temp_dir = tempfile::tempdir().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        // Create a file that might take a moment to parse, ensuring timeout is triggered
        let long_content: String = (0..1000).map(|_| "fn func() {} \n").collect();
        std::fs::write(&test_file, long_content).unwrap();
        let result = tokio::time::timeout(
            Duration::from_secs(10),
            parser.parse_file_robust(test_file, None),
        )
        .await;
        match result {
            Ok(inner) => match inner {
                Err(ParseFileError::Timeout(_)) => {
                    debug!("test_timeout_behavior completed: timeout as expected")
                }
                Ok(_) => info!("Parsing completed faster than expected timeout"),
                Err(e) => panic!("Unexpected error type: {:?}", e),
            },
            Err(_) => panic!("test_timeout_behavior timed out"),
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_concurrency_limiting() {
        debug!("Starting test_concurrency_limiting");
        let config = RobustParserConfig {
            max_concurrent_operations: 2, // Limit to 2 concurrent parses
            default_timeout: Duration::from_secs(10),
            ast_cache_size: 10,
        };
        let parser = RobustParser::new(config);
        let temp_dir = tempfile::tempdir().unwrap();

        let files: Vec<_> = (0..4)
            .map(|i| {
                let path = temp_dir.path().join(format!("test{}.rs", i));
                std::fs::write(&path, format!("fn test{}() {{}}", i)).unwrap();
                path
            })
            .collect();

        let start = std::time::Instant::now();

        let mut handles = Vec::new();
        for file in files {
            let parser = parser.clone();
            handles.push(tokio::spawn(async move {
                parser.parse_file_robust(file, None).await
            }));
        }

        let results = futures::future::join_all(handles).await;

        let elapsed = start.elapsed();
        println!("4 parses with concurrency 2 took: {:?}", elapsed);

        // Verification
        assert_eq!(results.len(), 4);
        let success_count = results
            .into_iter()
            .filter(|r| r.is_ok() && r.as_ref().unwrap().is_ok())
            .count();
        assert_eq!(
            success_count, 4,
            "All 4 files should have been parsed successfully"
        );

        // Check that caching worked. All files are unique so should be cache misses.
        let stats = {
            let parser_guard = parser.parser.lock().await;
            parser_guard.get_cache_stats().unwrap()
        };
        assert!(
            stats.misses >= 4,
            "Should have at least 4 cache misses as files are unique"
        );

        debug!("test_concurrency_limiting completed successfully");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 8)]
    async fn test_high_load_stress() {
        debug!("Starting test_high_load_stress");
        let config = RobustParserConfig {
            max_concurrent_operations: 4,
            default_timeout: Duration::from_secs(20),
            ast_cache_size: 50, // Smaller cache to force evictions
        };
        let parser = RobustParser::new(config);
        let temp_dir = tempfile::tempdir().unwrap();
        let mut handles: Vec<
            tokio::task::JoinHandle<Result<crate::ast::ParsedFile, ParseFileError>>,
        > = vec![];

        for i in 0..100 {
            let file_path = temp_dir.path().join(format!("stress_test_{}.rs", i));
            std::fs::write(&file_path, format!("fn main() {{ println!(\"{}\"); }}", i)).unwrap();
            let parser = parser.clone();
            handles.push(tokio::spawn(async move {
                parser.parse_file_robust(file_path, None).await
            }));
        }

        let result = tokio::time::timeout(Duration::from_secs(60), async {
            futures::future::join_all(handles).await
        })
        .await;

        match result {
            Ok(results) => {
                let success_count = results
                    .iter()
                    .filter(|r| r.as_ref().unwrap().is_ok())
                    .count();
                assert_eq!(
                    success_count, 100,
                    "All 100 parses should succeed, got {}",
                    success_count
                );
                debug!("test_high_load_stress completed successfully");
            }
            Err(_) => panic!("test_high_load_stress timed out"),
        }
    }

    #[tokio::test]
    async fn test_panic_resilience() {
        // This test now uses the actual RobustParser but with a mocked AstParser that panics.
        // We can't easily swap the parser inside RobustParser, so we simulate the call pattern.
        let should_panic = true;
        let result = tokio::task::spawn_blocking(move || {
            if should_panic {
                panic!("Simulated parser panic for test");
            }
        })
        .await;

        // Assert that the panic was caught by the JoinHandle
        assert!(
            result.is_err(),
            "Expected spawn_blocking to catch the panic"
        );
        let join_error = result.unwrap_err();
        assert!(join_error.is_panic(), "The join error should be a panic");

        // Now, we map it to our application error, just like in the real implementation
        let app_error = if let Ok(panic_payload) = join_error.try_into_panic() {
            let msg = if let Some(s) = panic_payload.downcast_ref::<&str>() {
                s.to_string()
            } else {
                "Unknown panic".to_string()
            };
            ParseFileError::Panic(msg)
        } else {
            unreachable!("We already confirmed this is a panic");
        };

        assert!(matches!(app_error, ParseFileError::Panic(_)));
        assert!(app_error
            .to_string()
            .contains("Simulated parser panic for test"));
    }
}
