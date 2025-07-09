use crate::error::{ErrorCategory, ErrorSeverity, RenderingServiceError};
use std::cmp::max;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use tokio::time::sleep;

/// Configuration for the retry mechanism.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// The maximum number of attempts for an operation.
    pub max_attempts: u32,
    /// The base delay between retries.
    pub base_delay: Duration,
    /// The maximum delay between retries.
    pub max_delay: Duration,
    /// The multiplier for exponential backoff.
    pub backoff_multiplier: f64,
    /// The jitter factor to randomize delays.
    pub jitter_factor: f64,
    /// Categories of errors that should trigger a retry.
    pub retry_on_categories: Vec<ErrorCategory>,
    /// Severities of errors that should trigger a retry.
    pub retry_on_severities: Vec<ErrorSeverity>,
    /// Whether to respect rate limit headers from the server.
    pub respect_rate_limits: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
            retry_on_categories: vec![
                ErrorCategory::ServiceCommunication,
                ErrorCategory::ResourceExhaustion,
                ErrorCategory::ServiceSpecific, // NOTE: Replaced TemporaryFailure with ServiceSpecific
            ],
            retry_on_severities: vec![ErrorSeverity::Low, ErrorSeverity::Medium],
            respect_rate_limits: true,
        }
    }
}

/// A client that executes operations with a configurable retry strategy.
pub struct RetryClient {
    config: RetryConfig,
}

impl RetryClient {
    /// Creates a new `RetryClient` with the given configuration.
    pub fn new(config: RetryConfig) -> Self {
        Self { config }
    }

    /// Executes an operation with retry logic based on the client's configuration.
    ///
    /// The operation is a function that returns a `Future` which resolves to a `Result`.
    /// If the operation fails with a retryable error, it will be attempted again
    /// after a delay, until the maximum number of attempts is reached.
    pub async fn execute_with_retry<F, T>(&self, operation: F) -> Result<T, RenderingServiceError>
    where
        F: Fn() -> Pin<Box<dyn Future<Output = Result<T, RenderingServiceError>> + Send>>,
    {
        let mut attempt = 1;
        let mut delay = self.config.base_delay;

        loop {
            match operation().await {
                Ok(result) => {
                    if attempt > 1 {
                        tracing::info!(
                            attempt = attempt,
                            "Operation succeeded after {} retries",
                            attempt - 1
                        );
                    }
                    return Ok(result);
                }
                Err(error) => {
                    // REPLACE string matching with type-safe error classification
                    let should_retry = self.should_retry_error(&error);

                    if !should_retry || attempt >= self.config.max_attempts {
                        tracing::error!(
                            attempt = attempt,
                            error_category = ?error.category(),
                            error_severity = ?error.severity(),
                            error = %error,
                            "Operation failed after {} attempts",
                            attempt
                        );
                        return Err(error);
                    }

                    // Handle rate limiting specially
                    let actual_delay =
                        if let RenderingServiceError::RateLimitExceeded { retry_after_seconds } =
                            &error
                        {
                            self.calculate_rate_limit_delay(*retry_after_seconds)
                        } else {
                            self.calculate_exponential_delay(delay)
                        };

                    tracing::warn!(
                        attempt = attempt,
                        delay_ms = actual_delay.as_millis(),
                        error_category = ?error.category(),
                        error_severity = ?error.severity(),
                        error = %error,
                        "Operation failed, retrying in {}ms",
                        actual_delay.as_millis()
                    );

                    sleep(actual_delay).await;
                    delay = self.next_delay(delay);
                    attempt += 1;
                }
            }
        }
    }

    // ... existing methods ...

    fn should_retry_error(&self, error: &RenderingServiceError) -> bool {
        // Use the error's built-in retry logic
        if !error.is_retryable() { // NOTE: Corrected method name from should_retry to is_retryable
            return false;
        }

        // Check if error category is in our retry list
        if !self.config.retry_on_categories.contains(&error.category()) {
            return false;
        }

        // Check if error severity allows retry
        self.config.retry_on_severities.contains(&error.severity())
    }

    fn calculate_rate_limit_delay(&self, retry_after_seconds: Option<u64>) -> Duration {
        if self.config.respect_rate_limits {
            if let Some(seconds) = retry_after_seconds {
                // Use the server's suggested delay, but cap it at max_delay
                return Duration::from_secs(seconds).min(self.config.max_delay);
            }
        }
        // Fallback to normal exponential backoff
        self.calculate_exponential_delay(self.config.base_delay)
    }

    fn calculate_exponential_delay(&self, current_delay: Duration) -> Duration {
        // Apply jitter to the current delay
        let jitter = 1.0 + (rand::random::<f64>() - 0.5) * 2.0 * self.config.jitter_factor;
        Duration::from_millis(
            (current_delay.as_millis() as f64 * jitter)
                .min(self.config.max_delay.as_millis() as f64) as u64,
        )
    }

    fn next_delay(&self, current_delay: Duration) -> Duration {
        Duration::from_millis(
            (current_delay.as_millis() as f64 * self.config.backoff_multiplier)
                .min(self.config.max_delay.as_millis() as f64) as u64,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_retry_success_first_attempt() {
        let config = RetryConfig::default();
        let client = RetryClient::new(config);

        let result = client
            .execute_with_retry(|| Box::pin(async { Ok::<i32, RenderingServiceError>(42) }))
            .await;

        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_retry_exhaustion() {
        let config = RetryConfig {
            max_attempts: 2,
            base_delay: Duration::from_millis(1),
            retry_on_severities: vec![
                ErrorSeverity::Low,
                ErrorSeverity::Medium,
                ErrorSeverity::High,
            ],
            ..Default::default()
        };
        let client = RetryClient::new(config);

        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = client
            .execute_with_retry(move || {
                let counter = counter_clone.clone();
                Box::pin(async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Err::<i32, RenderingServiceError>(RenderingServiceError::ConnectionTimeout {
                        timeout: 5000,
                    })
                })
            })
            .await;

        assert!(result.is_err());
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_error_category_filtering() {
        let config = RetryConfig {
            max_attempts: 3,
            base_delay: Duration::from_millis(1),
            retry_on_categories: vec![ErrorCategory::ServiceCommunication],
            ..Default::default()
        };
        let client = RetryClient::new(config);

        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        // This error should NOT trigger retries (wrong category)
        let result = client
            .execute_with_retry(move || {
                let counter = counter_clone.clone();
                Box::pin(async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    Err::<i32, RenderingServiceError>(RenderingServiceError::InvalidMermaidSyntax {
                        details: "test".to_string(),
                        line: None,
                    })
                })
            })
            .await;

        assert!(result.is_err());
        assert_eq!(counter.load(Ordering::SeqCst), 1); // Only one attempt
    }

    #[tokio::test]
    async fn test_rate_limit_respect() {
        let config = RetryConfig {
            max_attempts: 3,
            base_delay: Duration::from_millis(10),
            respect_rate_limits: true,
            ..Default::default()
        };
        let client = RetryClient::new(config);

        let start_time = std::time::Instant::now();

        let result = client
            .execute_with_retry(|| {
                Box::pin(async {
                    Err::<i32, RenderingServiceError>(RenderingServiceError::RateLimitExceeded {
                        retry_after_seconds: Some(1),
                    })
                })
            })
            .await;

        let elapsed = start_time.elapsed();
        assert!(result.is_err());
        // Should have waited at least 1 second due to rate limiting
        assert!(elapsed >= Duration::from_millis(900));
    }
}