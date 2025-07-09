use std::time::Duration;
use tokio::time::sleep;
use std::future::Future;
use std::pin::Pin;

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
    pub jitter_factor: f64,
    pub retry_on_errors: Vec<String>, // Error type names that should trigger retry
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
            retry_on_errors: vec![
                "ConnectionTimeout".to_string(),
                "RequestTimeout".to_string(),
                "ServiceUnavailable".to_string(),
            ],
        }
    }
}

pub struct RetryClient {
    config: RetryConfig,
}

impl RetryClient {
    pub fn new(config: RetryConfig) -> Self {
        Self { config }
    }

    pub async fn execute_with_retry<F, T, E>(&self, operation: F) -> Result<T, E>
    where
        F: Fn() -> Pin<Box<dyn Future<Output = Result<T, E>> + Send>>,
        E: std::fmt::Display + std::fmt::Debug,
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
                    // Check if this error type should trigger a retry
                    let error_type = format!("{:?}", error);
                    let should_retry = self.config.retry_on_errors.iter()
                        .any(|retry_error| error_type.contains(retry_error));

                    if !should_retry || attempt >= self.config.max_attempts {
                        tracing::error!(
                            attempt = attempt,
                            error = %error,
                            "Operation failed after {} attempts",
                            attempt
                        );
                        return Err(error);
                    }

                    // Calculate delay with jitter
                    let jitter = 1.0 + (rand::random::<f64>() - 0.5) * 2.0 * self.config.jitter_factor;
                    let actual_delay = Duration::from_millis(
                        (delay.as_millis() as f64 * jitter).min(self.config.max_delay.as_millis() as f64) as u64
                    );

                    tracing::warn!(
                        attempt = attempt,
                        delay_ms = actual_delay.as_millis(),
                        error = %error,
                        "Operation failed, retrying in {}ms",
                        actual_delay.as_millis()
                    );

                    sleep(actual_delay).await;

                    // Exponential backoff for next iteration
                    delay = Duration::from_millis(
                        (delay.as_millis() as f64 * self.config.backoff_multiplier)
                            .min(self.config.max_delay.as_millis() as f64) as u64
                    );

                    attempt += 1;
                }
            }
        }
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

        let result = client.execute_with_retry(|| {
            Box::pin(async { Ok::<i32, String>(42) })
        }).await;

        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_retry_exhaustion() {
        let config = RetryConfig {
            max_attempts: 2,
            base_delay: Duration::from_millis(1),
            ..Default::default()
        };
        let client = RetryClient::new(config);

        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = client.execute_with_retry(move || {
            let counter = counter_clone.clone();
            Box::pin(async move {
                counter.fetch_add(1, Ordering::SeqCst);
                Err::<i32, String>("ConnectionTimeout".to_string())
            })
        }).await;

        assert!(result.is_err());
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_exponential_backoff() {
        let config = RetryConfig {
            max_attempts: 3,
            base_delay: Duration::from_millis(10),
            backoff_multiplier: 2.0,
            jitter_factor: 0.0, // No jitter for predictable testing
            ..Default::default()
        };
        let client = RetryClient::new(config);

        let start_time = std::time::Instant::now();
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let _result = client.execute_with_retry(move || {
            let counter = counter_clone.clone();
            Box::pin(async move {
                let count = counter.fetch_add(1, Ordering::SeqCst);
                if count < 2 {
                    Err::<i32, String>("ConnectionTimeout".to_string())
                } else {
                    Ok(42)
                }
            })
        }).await;

        let elapsed = start_time.elapsed();
        // Should have waited at least 10ms + 20ms = 30ms
        assert!(elapsed >= Duration::from_millis(25));
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_jitter_applied() {
        let config = RetryConfig {
            max_attempts: 2,
            base_delay: Duration::from_millis(100),
            jitter_factor: 0.5,
            ..Default::default()
        };
        let client = RetryClient::new(config);

        let start_time = std::time::Instant::now();

        let _result = client.execute_with_retry(|| {
            Box::pin(async { Err::<i32, String>("ConnectionTimeout".to_string()) })
        }).await;

        let elapsed = start_time.elapsed();
        // With 50% jitter, delay should be between 50ms and 150ms
        assert!(elapsed >= Duration::from_millis(40));
        assert!(elapsed <= Duration::from_millis(200));
    }

    #[tokio::test]
    async fn test_error_filtering() {
        let config = RetryConfig {
            max_attempts: 3,
            base_delay: Duration::from_millis(1),
            retry_on_errors: vec!["ConnectionTimeout".to_string()],
            ..Default::default()
        };
        let client = RetryClient::new(config);

        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        // This error should NOT trigger retries
        let result = client.execute_with_retry(move || {
            let counter = counter_clone.clone();
            Box::pin(async move {
                counter.fetch_add(1, Ordering::SeqCst);
                Err::<i32, String>("ValidationError".to_string())
            })
        }).await;

        assert!(result.is_err());
        assert_eq!(counter.load(Ordering::SeqCst), 1); // Only one attempt
    }
}