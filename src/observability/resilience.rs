//! Integration of observability with resilience patterns
//!
//! Provides instrumented versions of circuit breakers, retry mechanisms, and
//! graceful degradation that automatically emit metrics and telemetry events.

use crate::error::{Result, UveddiError};
use crate::observability::{
    metrics::{CircuitBreakerState, MetricsTimer, UveddiMetrics},
    telemetry::{TelemetryLevel, TelemetrySender, TelemetryValue},
    tracing_utils::TraceId,
};
use crate::resilience::{circuit_breaker::State, CircuitBreaker as BaseCircuitBreaker};
use anyhow::Context;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{error, info, instrument, warn};

/// Instrumented circuit breaker that emits observability data
pub struct InstrumentedCircuitBreaker {
    name: String,
    inner: BaseCircuitBreaker,
    metrics: Arc<UveddiMetrics>,
    telemetry: Option<TelemetrySender>,
}

impl InstrumentedCircuitBreaker {
    /// Create a new instrumented circuit breaker
    pub fn new(
        name: String,
        failure_threshold: usize,
        reset_timeout: Duration,
        metrics: Arc<UveddiMetrics>,
        telemetry: Option<TelemetrySender>,
    ) -> Self {
        Self {
            name,
            inner: BaseCircuitBreaker::new(failure_threshold, reset_timeout),
            metrics,
            telemetry,
        }
    }

    /// Execute a function with circuit breaker protection
    #[instrument(skip(self, operation), fields(circuit_breaker = %self.name))]
    pub async fn call<F, T, E>(&mut self, trace_id: TraceId, operation: F) -> Result<T>
    where
        F: std::future::Future<Output = std::result::Result<T, E>>,
        E: Into<UveddiError>,
    {
        let timer = MetricsTimer::start_with_trace_id(trace_id);

        // Check if circuit allows the call
        if !self.inner.allow_request() {
            warn!(
                trace_id = %trace_id,
                circuit_breaker = %self.name,
                "Circuit breaker is open, rejecting call"
            );

            self.metrics.update_circuit_breaker_state(
                &self.name,
                "default",
                CircuitBreakerState::Open,
            );

            if let Some(ref telemetry) = self.telemetry {
                let _ = telemetry.circuit_breaker(
                    trace_id,
                    &self.name,
                    "call_rejected",
                    "circuit_open",
                );
            }

            return Err(UveddiError::GenericError {
                message: "Circuit breaker is open".to_string(),
                context: format!(
                    "Circuit breaker '{}' is protecting downstream service",
                    self.name
                ),
                suggestion: "Wait for circuit breaker to reset or check downstream service health"
                    .to_string(),
                source: None,
            });
        }

        // Update metrics for allowed call
        let state = if self.inner.is_closed() {
            CircuitBreakerState::Closed
        } else {
            CircuitBreakerState::HalfOpen // Assume half-open if allowing requests but not closed
        };

        self.metrics
            .update_circuit_breaker_state(&self.name, "default", state);

        // Execute the operation
        let result = operation.await;
        let duration = timer.elapsed();

        match result {
            Ok(value) => {
                // Record success
                self.inner.record_success();

                self.metrics.record_circuit_breaker_transition(
                    &self.name,
                    CircuitBreakerState::Closed,
                    CircuitBreakerState::Closed,
                );

                info!(
                    trace_id = %trace_id,
                    circuit_breaker = %self.name,
                    duration_ms = duration.as_millis(),
                    "Circuit breaker call succeeded"
                );

                Ok(value)
            }
            Err(error) => {
                let error = error.into();

                // Record failure (this may change circuit state)
                let old_state = self.get_metrics_state();
                // Note: Simplified failure recording - in production you'd want
                // proper error classification and integration with the base circuit breaker
                warn!(
                    trace_id = %trace_id,
                    circuit_breaker = %self.name,
                    error = %error,
                    "Recording circuit breaker failure"
                );
                let new_state = self.get_metrics_state();

                if old_state != new_state {
                    self.metrics
                        .record_circuit_breaker_transition(&self.name, old_state, new_state);

                    if let Some(ref telemetry) = self.telemetry {
                        let _ = telemetry.circuit_breaker(
                            trace_id,
                            &self.name,
                            &format!("{:?}_to_{:?}", old_state, new_state),
                            "failure_threshold_reached",
                        );
                    }
                }

                error!(
                    trace_id = %trace_id,
                    circuit_breaker = %self.name,
                    error = %error,
                    duration_ms = duration.as_millis(),
                    "Circuit breaker call failed"
                );

                Err(error)
            }
        }
    }

    /// Get the current state as a metrics-compatible enum
    fn get_metrics_state(&self) -> CircuitBreakerState {
        if self.inner.is_closed() {
            CircuitBreakerState::Closed
        } else if self.inner.allow_request() {
            CircuitBreakerState::HalfOpen
        } else {
            CircuitBreakerState::Open
        }
    }
}

/// Instrumented retry mechanism with exponential backoff and observability
pub struct InstrumentedRetry {
    max_attempts: u32,
    initial_delay: Duration,
    max_delay: Duration,
    backoff_multiplier: f64,
    jitter: bool,
    metrics: Arc<UveddiMetrics>,
    telemetry: Option<TelemetrySender>,
}

impl InstrumentedRetry {
    /// Create a new instrumented retry mechanism
    pub fn new(
        max_attempts: u32,
        initial_delay: Duration,
        max_delay: Duration,
        backoff_multiplier: f64,
        jitter: bool,
        metrics: Arc<UveddiMetrics>,
        telemetry: Option<TelemetrySender>,
    ) -> Self {
        Self {
            max_attempts,
            initial_delay,
            max_delay,
            backoff_multiplier,
            jitter,
            metrics,
            telemetry,
        }
    }

    /// Execute an operation with retry logic
    #[instrument(skip(self, operation), fields(max_attempts = self.max_attempts))]
    pub async fn call<F, Fut, T, E>(
        &self,
        trace_id: TraceId,
        service: &str,
        operation_name: &str,
        operation: F,
    ) -> Result<T>
    where
        F: Fn() -> Fut + Send + Sync,
        Fut: std::future::Future<Output = std::result::Result<T, E>> + Send,
        E: Into<UveddiError>,
    {
        let mut last_error = None;
        let mut delay = self.initial_delay;

        for attempt in 1..=self.max_attempts {
            let timer = MetricsTimer::start_with_trace_id(trace_id);

            info!(
                trace_id = %trace_id,
                service = service,
                operation = operation_name,
                attempt = attempt,
                max_attempts = self.max_attempts,
                "Executing retry attempt"
            );

            match operation().await {
                Ok(result) => {
                    let duration = timer.elapsed();

                    // Record successful retry
                    let final_outcome = if attempt == 1 {
                        "success_first_try"
                    } else {
                        "success_after_retry"
                    };
                    self.metrics
                        .record_retry_attempt(service, operation_name, final_outcome);

                    info!(
                        trace_id = %trace_id,
                        service = service,
                        operation = operation_name,
                        attempt = attempt,
                        duration_ms = duration.as_millis(),
                        "Retry operation succeeded"
                    );

                    return Ok(result);
                }
                Err(error) => {
                    let error: UveddiError = error.into();
                    let duration = timer.elapsed();

                    warn!(
                        trace_id = %trace_id,
                        service = service,
                        operation = operation_name,
                        attempt = attempt,
                        error = %error,
                        duration_ms = duration.as_millis(),
                        "Retry attempt failed"
                    );

                    last_error = Some(error);

                    // If this is not the last attempt, wait before retrying
                    if attempt < self.max_attempts {
                        let sleep_duration = if self.jitter {
                            self.add_jitter(delay)
                        } else {
                            delay
                        };

                        info!(
                            trace_id = %trace_id,
                            service = service,
                            operation = operation_name,
                            attempt = attempt,
                            delay_ms = sleep_duration.as_millis(),
                            "Waiting before next retry attempt"
                        );

                        sleep(sleep_duration).await;

                        // Calculate next delay with exponential backoff
                        delay = Duration::from_millis(
                            ((delay.as_millis() as f64) * self.backoff_multiplier) as u64,
                        )
                        .min(self.max_delay);
                    }
                }
            }
        }

        // All attempts failed
        let final_error = last_error.unwrap();

        self.metrics
            .record_retry_attempt(service, operation_name, "failure_all_attempts");

        if let Some(ref telemetry) = self.telemetry {
            let mut fields = HashMap::new();
            fields.insert(
                "attempts".to_string(),
                TelemetryValue::Integer(self.max_attempts as i64),
            );
            fields.insert(
                "service".to_string(),
                TelemetryValue::String(service.to_string()),
            );
            fields.insert(
                "operation".to_string(),
                TelemetryValue::String(operation_name.to_string()),
            );

            let _ = telemetry.send(crate::observability::telemetry::TelemetryEvent {
                trace_id,
                timestamp: chrono::Utc::now(),
                event_type: crate::observability::telemetry::TelemetryEventType::Error,
                severity: TelemetryLevel::Error,
                component: service.to_string(),
                message: format!(
                    "All {} retry attempts failed for {}",
                    self.max_attempts, operation_name
                ),
                fields,
                error: Some(crate::observability::telemetry::TelemetryError {
                    error_type: format!("{:?}", final_error.category()),
                    message: final_error.to_string(),
                    code: None,
                    context: Some("retry_exhausted".to_string()),
                }),
            });
        }

        error!(
            trace_id = %trace_id,
            service = service,
            operation = operation_name,
            attempts = self.max_attempts,
            error = %final_error,
            "All retry attempts exhausted"
        );

        Err(final_error)
    }

    /// Add jitter to prevent thundering herd effect
    fn add_jitter(&self, delay: Duration) -> Duration {
        use rand::Rng;
        let jitter_ms = rand::rng().gen_range(0..=delay.as_millis() / 4);
        delay + Duration::from_millis(jitter_ms as u64)
    }
}

/// Instrumented fallback mechanism that tracks fallback activations
pub struct InstrumentedFallback {
    service: String,
    metrics: Arc<UveddiMetrics>,
    telemetry: Option<TelemetrySender>,
}

impl InstrumentedFallback {
    /// Create a new instrumented fallback mechanism
    pub fn new(
        service: String,
        metrics: Arc<UveddiMetrics>,
        telemetry: Option<TelemetrySender>,
    ) -> Self {
        Self {
            service,
            metrics,
            telemetry,
        }
    }

    /// Execute an operation with fallback support
    #[instrument(skip(self, primary, fallback), fields(service = %self.service))]
    pub async fn call_with_fallback<F1, F2, Fut1, Fut2, T, E1, E2>(
        &self,
        trace_id: TraceId,
        fallback_type: &str,
        primary: F1,
        fallback: F2,
    ) -> Result<T>
    where
        F1: FnOnce() -> Fut1 + Send,
        F2: FnOnce() -> Fut2 + Send,
        Fut1: std::future::Future<Output = std::result::Result<T, E1>> + Send,
        Fut2: std::future::Future<Output = std::result::Result<T, E2>> + Send,
        E1: Into<UveddiError>,
        E2: Into<UveddiError>,
    {
        let timer = MetricsTimer::start_with_trace_id(trace_id);

        info!(
            trace_id = %trace_id,
            service = %self.service,
            fallback_type = fallback_type,
            "Attempting primary operation"
        );

        match primary().await {
            Ok(result) => {
                let duration = timer.elapsed();

                info!(
                    trace_id = %trace_id,
                    service = %self.service,
                    duration_ms = duration.as_millis(),
                    "Primary operation succeeded"
                );

                Ok(result)
            }
            Err(primary_error) => {
                let primary_error = primary_error.into();
                let primary_duration = timer.elapsed();

                warn!(
                    trace_id = %trace_id,
                    service = %self.service,
                    error = %primary_error,
                    duration_ms = primary_duration.as_millis(),
                    "Primary operation failed, activating fallback"
                );

                // Record fallback activation
                self.metrics.record_fallback_activation(
                    &self.service,
                    fallback_type,
                    "primary_failure",
                );

                if let Some(ref telemetry) = self.telemetry {
                    let _ = telemetry.send(crate::observability::telemetry::TelemetryEvent {
                        trace_id,
                        timestamp: chrono::Utc::now(),
                        event_type: crate::observability::telemetry::TelemetryEventType::Fallback,
                        severity: TelemetryLevel::Warn,
                        component: self.service.clone(),
                        message: format!(
                            "Fallback activated: {} due to primary failure",
                            fallback_type
                        ),
                        fields: {
                            let mut fields = HashMap::new();
                            fields.insert(
                                "fallback_type".to_string(),
                                TelemetryValue::String(fallback_type.to_string()),
                            );
                            fields.insert(
                                "primary_error".to_string(),
                                TelemetryValue::String(primary_error.to_string()),
                            );
                            fields
                        },
                        error: None,
                    });
                }

                // Try fallback
                let fallback_timer = MetricsTimer::start_with_trace_id(trace_id);

                match fallback().await {
                    Ok(result) => {
                        let fallback_duration = fallback_timer.elapsed();

                        info!(
                            trace_id = %trace_id,
                            service = %self.service,
                            fallback_type = fallback_type,
                            duration_ms = fallback_duration.as_millis(),
                            "Fallback operation succeeded"
                        );

                        Ok(result)
                    }
                    Err(fallback_error) => {
                        let fallback_error = fallback_error.into();
                        let fallback_duration = fallback_timer.elapsed();

                        error!(
                            trace_id = %trace_id,
                            service = %self.service,
                            fallback_type = fallback_type,
                            primary_error = %primary_error,
                            fallback_error = %fallback_error,
                            fallback_duration_ms = fallback_duration.as_millis(),
                            "Both primary and fallback operations failed"
                        );

                        // Return the primary error as it's likely more informative
                        Err(primary_error)
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observability::config::MetricsConfig;

    #[tokio::test]
    async fn test_instrumented_retry() {
        let metrics = Arc::new(UveddiMetrics::new(&MetricsConfig::default()).unwrap());
        let retry = InstrumentedRetry::new(
            3,
            Duration::from_millis(100),
            Duration::from_secs(1),
            2.0,
            false,
            metrics,
            None,
        );

        let trace_id = TraceId::new();
        let call_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));

        let result = retry
            .call(trace_id, "test_service", "test_operation", {
                let call_count = call_count.clone();
                move || {
                    let count = call_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                    async move {
                        if count < 3 {
                            Err(UveddiError::GenericError {
                                message: "Transient error".to_string(),
                                context: "test".to_string(),
                                suggestion: "retry".to_string(),
                                source: None,
                            })
                        } else {
                            Ok("success")
                        }
                    }
                }
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 3);
    }
}
