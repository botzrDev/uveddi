# 🤖 **GPT Prompt for UV-168: Type-Safe Retry Logic Implementation**

```markdown
# UV-168: Implement Type-Safe Retry Logic with RenderingServiceError Integration

You are a senior Rust developer working on the Uveddi project, a sophisticated static code analysis tool. Your task is to enhance the existing retry mechanism to use type-safe error handling instead of string-based error matching.

## 🎯 **Objective**
Refactor `src/resilience/retry.rs` to integrate with the RenderingServiceError system from UV-169, replacing string-based error detection with intelligent, type-safe error classification.

## 📋 **Current State Analysis**
The retry system is already implemented with:
- ✅ Exponential backoff with jitter
- ✅ Comprehensive test suite  
- ✅ Basic error handling
- ❌ String-based error matching (needs upgrade)
- ❌ No integration with RenderingServiceError types

## 🔧 **Required Changes**

### **1. Update Imports and Dependencies**
Add to the top of `src/resilience/retry.rs`:
```rust
use crate::error::{RenderingServiceError, ErrorCategory, ErrorSeverity};
use std::cmp::max;
```

### **2. Enhance RetryConfig Structure**
Replace the current `retry_on_errors: Vec<String>` with:
```rust
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
    pub jitter_factor: f64,
    // REPLACE: retry_on_errors: Vec<String> with:
    pub retry_on_categories: Vec<ErrorCategory>,
    pub retry_on_severities: Vec<ErrorSeverity>,
    pub respect_rate_limits: bool,
}
```

### **3. Update Default Implementation**
Replace the default retry_on_errors with:
```rust
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
                ErrorCategory::TemporaryFailure,
            ],
            retry_on_severities: vec![
                ErrorSeverity::Low,
                ErrorSeverity::Medium,
            ],
            respect_rate_limits: true,
        }
    }
}
```

### **4. Refactor execute_with_retry Method**
Update the method signature and implementation:

```rust
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
                let actual_delay = if let RenderingServiceError::RateLimitExceeded { retry_after_seconds } = &error {
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
```

### **5. Add Helper Methods**
Implement these new methods in the `RetryClient` impl block:

```rust
impl RetryClient {
    // ... existing methods ...

    fn should_retry_error(&self, error: &RenderingServiceError) -> bool {
        // Use the error's built-in retry logic
        if !error.should_retry() {
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
                .min(self.config.max_delay.as_millis() as f64) as u64
        )
    }

    fn next_delay(&self, current_delay: Duration) -> Duration {
        Duration::from_millis(
            (current_delay.as_millis() as f64 * self.config.backoff_multiplier)
                .min(self.config.max_delay.as_millis() as f64) as u64
        )
    }
}
```

### **6. Update Test Suite**
Modify the existing tests to use RenderingServiceError instead of String errors:

```rust
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
            Box::pin(async { Ok::<i32, RenderingServiceError>(42) })
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
                Err::<i32, RenderingServiceError>(RenderingServiceError::ConnectionTimeout { timeout: 5000 })
            })
        }).await;

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
        let result = client.execute_with_retry(move || {
            let counter = counter_clone.clone();
            Box::pin(async move {
                counter.fetch_add(1, Ordering::SeqCst);
                Err::<i32, RenderingServiceError>(RenderingServiceError::InvalidMermaidSyntax { 
                    details: "test".to_string(), 
                    line: None 
                })
            })
        }).await;

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
        
        let result = client.execute_with_retry(|| {
            Box::pin(async { 
                Err::<i32, RenderingServiceError>(RenderingServiceError::RateLimitExceeded { 
                    retry_after_seconds: Some(1) 
                })
            })
        }).await;

        let elapsed = start_time.elapsed();
        assert!(result.is_err());
        // Should have waited at least 1 second due to rate limiting
        assert!(elapsed >= Duration::from_millis(900));
    }
}
```

## ✅ **Acceptance Criteria**

Before submitting your implementation, ensure:

1. **Compilation**: `cargo check --all-targets` passes without warnings
2. **Tests**: `cargo test resilience::retry` passes all tests
3. **Type Safety**: All error handling uses RenderingServiceError types
4. **Rate Limiting**: Rate limit errors are handled with appropriate delays
5. **Logging**: Error categories and severities are logged properly
6. **Backward Compatibility**: Existing functionality is preserved

## 🚨 **Important Notes**

- **DO NOT** change the public API of RetryClient beyond the error type constraint
- **PRESERVE** all existing exponential backoff and jitter logic
- **ENSURE** all tests pass after your changes
- **USE** the error classification methods from RenderingServiceError
- **MAINTAIN** the same logging patterns but enhance with error metadata

## 🔍 **Validation Commands**

Run these commands to validate your implementation:
```bash
cargo check --all-targets
cargo test resilience::retry
cargo clippy -- -D warnings
cargo fmt --check
```

## 📁 **Files to Modify**

- `src/resilience/retry.rs` (primary implementation)
- Update imports and dependencies as needed

Your implementation should be production-ready and integrate seamlessly with the existing Uveddi architecture. Focus on type safety, error handling best practices, and maintaining the high code quality standards of the project.
```

---

## 📋 **Prompt Usage Instructions**

### **For Project Managers:**
1. Copy the entire prompt content between the markdown code blocks
2. Paste into your AI coding assistant (ChatGPT, Claude, etc.)
3. Provide access to the current codebase files mentioned
4. Review the implementation before merging

### **For Developers:**
This prompt provides:
- ✅ Clear objective and context
- ✅ Detailed implementation steps
- ✅ Complete code examples
- ✅ Updated test cases
- ✅ Validation criteria
- ✅ Important constraints and notes

### **Integration Points:**
This implementation enables:
- **UV-170**: Structured logging with error categories
- **UV-172**: Circuit breaker integration via error classification
- **UV-171**: Fallback triggering using should_trigger_fallback()
- **UV-174**: Error metrics collection by category

### **Dependencies:**
- **Requires**: UV-169 (RenderingServiceError system) - ✅ Complete
- **Enables**: UV-170, UV-172, UV-171, UV-174

**Ready to assign to your AI coding assistant!** 🚀