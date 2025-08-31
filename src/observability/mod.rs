//! UV-86: Enterprise-Grade Observability and Resilience Framework
//!
//! This module implements the comprehensive observability architecture defined in UV-86,
//! providing structured logging, metrics collection, graceful degradation, and error recovery.
//!
//! ## Architecture Overview
//!
//! The observability framework is built around five core pillars:
//!
//! ### 1. Structured Logging
//! - Built on the `tracing` ecosystem for spans and events
//! - JSON output for production with PII redaction
//! - Correlation via trace_id across all operations
//!
//! ### 2. Metrics Collection
//! - Prometheus-based metrics for Four Golden Signals
//! - Custom application metrics for analysis pipeline
//! - Integration with security events from UV-247
//!
//! ### 3. Graceful Degradation
//! - Circuit breaker patterns for external dependencies
//! - Fallback strategies for partial service availability
//! - Clear communication of degraded states to clients
//!
//! ### 4. Error Recovery
//! - Intelligent retry mechanisms with exponential backoff
//! - Dead-letter queue for permanent failures
//! - Integration with circuit breakers
//!
//! ### 5. Unified Observability
//! - Cross-system correlation via trace_id
//! - Integration with UV-247 security audit system
//! - Role-specific dashboards and alerting
//!
//! ## Usage
//!
//! Initialize the observability system early in your application:
//!
//! ```rust
//! use uveddi::observability::{ObservabilityService, ObservabilityConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize observability
//!     let config = ObservabilityConfig::default();
//!     let observability = ObservabilityService::new(config).await?;
//!     observability.start().await?;
//!
//!     // Your application code here
//!     Ok(())
//! }
//! ```

pub mod config;
pub mod dlq;
pub mod logging;
#[cfg(feature = "prometheus")]
pub mod metrics;
pub mod resilience;
pub mod service;
pub mod telemetry;
pub mod tracing_utils;

pub use config::ObservabilityConfig;
pub use dlq::{DeadLetterQueue, DlqRecord, DlqStatistics};
pub use resilience::{InstrumentedCircuitBreaker, InstrumentedFallback, InstrumentedRetry};
pub use service::ObservabilityService;
pub use telemetry::{TelemetryCollector, TelemetryEvent};
pub use tracing_utils::{generate_trace_id, with_trace_id, TraceId};
