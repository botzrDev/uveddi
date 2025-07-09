//! Provides resilience patterns for handling transient failures in distributed systems.
//!
//! This module includes implementations for common resilience strategies like
//! retry mechanisms with exponential backoff and jitter.

pub mod retry;
pub mod circuit_breaker;
pub mod fallback;
pub mod metrics;

pub use retry::{RetryClient, RetryConfig};
pub use circuit_breaker::CircuitBreaker;
pub use fallback::{FallbackManager, FallbackConfig, FallbackMode};
pub use metrics::{MetricsCollector, MetricsConfig, ErrorMetrics, MetricsFormat};
