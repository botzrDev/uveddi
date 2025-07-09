//! Provides resilience patterns for handling transient failures in distributed systems.
//!
//! This module includes implementations for common resilience strategies like
//! retry mechanisms with exponential backoff and jitter.

pub mod retry;
pub mod circuit_breaker;

pub use retry::{RetryClient, RetryConfig};
pub use circuit_breaker::CircuitBreaker;
