//! Provides resilience patterns for handling transient failures in distributed systems.
//!
//! This module includes implementations for common resilience strategies like
//! retry mechanisms with exponential backoff and jitter.

pub mod availability;
pub mod circuit_breaker;
pub mod degradation;
pub mod fallback;
pub mod graceful_handler;
pub mod health;
pub mod metrics;
pub mod recovery;
pub mod retry;

pub use availability::AvailabilityDetector;
pub use circuit_breaker::CircuitBreaker;
pub use degradation::{DegradationEngine, ServiceLevel};
pub use fallback::{FallbackConfig, FallbackManager, FallbackMode};
pub use graceful_handler::{DegradationLevel, GracefulFailureHandler, GracefulResponse};
pub use health::HealthMonitor;
pub use metrics::{ErrorMetrics, MetricsCollector, MetricsConfig, MetricsFormat};
pub use recovery::RecoveryManager;
pub use retry::{RetryClient, RetryConfig};
