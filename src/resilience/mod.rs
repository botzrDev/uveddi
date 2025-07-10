//! Provides resilience patterns for handling transient failures in distributed systems.
//!
//! This module includes implementations for common resilience strategies like
//! retry mechanisms with exponential backoff and jitter.

pub mod circuit_breaker;
pub mod fallback;
pub mod health;
pub mod metrics;
pub mod retry;
pub mod graceful_handler;
pub mod degradation;
pub mod recovery;
pub mod availability;

pub use circuit_breaker::CircuitBreaker;
pub use fallback::{FallbackConfig, FallbackManager, FallbackMode};
pub use health::HealthMonitor;
pub use metrics::{ErrorMetrics, MetricsCollector, MetricsConfig, MetricsFormat};
pub use retry::{RetryClient, RetryConfig};
pub use graceful_handler::{GracefulFailureHandler, GracefulResponse, DegradationLevel};
pub use degradation::{DegradationEngine, ServiceLevel};
pub use recovery::RecoveryManager;
pub use availability::AvailabilityDetector;
