//! Metrics collection and exposure using Prometheus
//!
//! Implements the UV-86 metrics architecture with:
//! - Four Golden Signals monitoring
//! - Custom application metrics for analysis pipeline
//! - Integration with security events from UV-247
//! - Service Level Objectives (SLO) tracking

use crate::observability::config::{MetricsConfig, SloConfig};
use crate::observability::tracing_utils::TraceId;
use anyhow::{Context, Result};
use axum::response::IntoResponse;
use prometheus::{
    Counter, CounterVec, Gauge, GaugeVec, Histogram, HistogramOpts, HistogramVec, Opts, Registry,
};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time;

/// Metrics collector for the Uveddi analysis system
#[derive(Clone)]
pub struct UveddiMetrics {
    registry: Arc<Registry>,

    // Four Golden Signals
    /// Request rate (Traffic)
    request_total: CounterVec,
    /// Request duration distribution (Latency)
    request_duration: HistogramVec,
    /// Error rate (Errors)
    error_total: CounterVec,
    /// System resource utilization (Saturation)
    cpu_utilization: Gauge,
    memory_usage: Gauge,
    active_connections: Gauge,

    // Analysis-specific metrics
    /// Analysis request metrics
    analysis_requests_total: CounterVec,
    analysis_duration: HistogramVec,
    analysis_queue_length: Gauge,
    analysis_code_complexity: HistogramVec,
    analysis_bugs_found: CounterVec,
    analysis_duplicated_lines: Counter,

    // Rendering service metrics
    rendering_requests_total: CounterVec,
    rendering_duration: HistogramVec,

    // Security metrics (exported from UV-247)
    security_auth_failures: CounterVec,
    security_permission_denials: CounterVec,
    security_rate_limit_exceeded: CounterVec,

    // Circuit breaker and resilience metrics
    circuit_breaker_state: GaugeVec,
    circuit_breaker_transitions: CounterVec,
    retry_attempts: CounterVec,
    fallback_activations: CounterVec,

    // Service Level Objectives
    slo_availability: Gauge,
    slo_latency_p99: Gauge,
    error_budget_remaining: Gauge,

    // Dead Letter Queue metrics
    dlq_size: Gauge,
}

impl UveddiMetrics {
    /// Create a new metrics collector with the given configuration
    pub fn new(config: &MetricsConfig) -> Result<Self> {
        let registry = Arc::new(Registry::new());

        // Four Golden Signals
        let request_total = CounterVec::new(
            Opts::new("uveddi_requests_total", "Total number of requests"),
            &["method", "endpoint", "status"],
        )?;

        let request_duration = HistogramVec::new(
            HistogramOpts::new(
                "uveddi_request_duration_seconds",
                "Duration of requests in seconds",
            )
            .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0]),
            &["method", "endpoint"],
        )?;

        let error_total = CounterVec::new(
            Opts::new("uveddi_errors_total", "Total number of errors"),
            &["type", "severity", "component"],
        )?;

        let cpu_utilization = Gauge::new(
            "uveddi_cpu_utilization_percent",
            "CPU utilization percentage",
        )?;
        let memory_usage = Gauge::new("uveddi_memory_usage_bytes", "Memory usage in bytes")?;
        let active_connections =
            Gauge::new("uveddi_active_connections", "Number of active connections")?;

        // Analysis-specific metrics
        let analysis_requests_total = CounterVec::new(
            Opts::new("uveddi_analysis_requests_total", "Total analysis requests"),
            &["status", "error_type", "pipeline_stage"],
        )?;

        let analysis_duration = HistogramVec::new(
            HistogramOpts::new(
                "uveddi_analysis_duration_seconds",
                "Analysis duration in seconds",
            )
            .buckets(vec![0.1, 0.5, 1.0, 5.0, 10.0, 30.0, 60.0, 300.0]),
            &["pipeline_stage", "language"],
        )?;

        let analysis_queue_length = Gauge::new(
            "uveddi_analysis_queue_length",
            "Number of analysis requests in queue",
        )?;

        let analysis_code_complexity = HistogramVec::new(
            HistogramOpts::new(
                "uveddi_analysis_code_complexity_score",
                "Distribution of code complexity scores",
            )
            .buckets(vec![1.0, 5.0, 10.0, 20.0, 50.0, 100.0, 200.0]),
            &["complexity_type", "language"],
        )?;

        let analysis_bugs_found = CounterVec::new(
            Opts::new("uveddi_analysis_bugs_found_total", "Number of bugs found"),
            &["severity", "category", "language"],
        )?;

        let analysis_duplicated_lines = Counter::new(
            "uveddi_analysis_duplicated_lines_total",
            "Total lines of duplicated code detected",
        )?;

        // Rendering service metrics
        let rendering_requests_total = CounterVec::new(
            Opts::new(
                "uveddi_rendering_calls_total",
                "Total rendering service calls",
            ),
            &["status", "diagram_type"],
        )?;

        let rendering_duration = HistogramVec::new(
            HistogramOpts::new(
                "uveddi_rendering_duration_seconds",
                "Rendering service call duration",
            )
            .buckets(vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0]),
            &["diagram_type"],
        )?;

        // Security metrics
        let security_auth_failures = CounterVec::new(
            Opts::new(
                "uveddi_security_auth_failures_total",
                "Authentication failures",
            ),
            &["reason", "source"],
        )?;

        let security_permission_denials = CounterVec::new(
            Opts::new(
                "uveddi_security_permission_denials_total",
                "Permission denials",
            ),
            &["resource_type", "permission_level"],
        )?;

        let security_rate_limit_exceeded = CounterVec::new(
            Opts::new(
                "uveddi_security_rate_limit_exceeded_total",
                "Rate limit exceeded events",
            ),
            &["endpoint", "user_group"],
        )?;

        // Circuit breaker and resilience metrics
        let circuit_breaker_state = GaugeVec::new(
            Opts::new(
                "uveddi_circuit_breaker_state",
                "Circuit breaker state (0=closed, 1=open, 2=half-open)",
            ),
            &["service", "endpoint"],
        )?;

        let circuit_breaker_transitions = CounterVec::new(
            Opts::new(
                "uveddi_circuit_breaker_transitions_total",
                "Circuit breaker state transitions",
            ),
            &["service", "from_state", "to_state"],
        )?;

        let retry_attempts = CounterVec::new(
            Opts::new("uveddi_retry_attempts_total", "Retry attempts"),
            &["service", "operation", "final_outcome"],
        )?;

        let fallback_activations = CounterVec::new(
            Opts::new(
                "uveddi_fallback_activations_total",
                "Fallback mechanism activations",
            ),
            &["service", "fallback_type", "reason"],
        )?;

        // Service Level Objectives
        let slo_availability = Gauge::new(
            "uveddi_slo_availability_percent",
            "Current availability percentage",
        )?;

        let slo_latency_p99 = Gauge::new(
            "uveddi_slo_latency_p99_seconds",
            "Current P99 latency in seconds",
        )?;

        let error_budget_remaining = Gauge::new(
            "uveddi_error_budget_remaining_percent",
            "Remaining error budget as percentage",
        )?;

        // Dead Letter Queue metrics
        let dlq_size = Gauge::new(
            "uveddi_dlq_size",
            "Current number of records in Dead Letter Queue",
        )?;

        // Register all metrics
        registry.register(Box::new(request_total.clone()))?;
        registry.register(Box::new(request_duration.clone()))?;
        registry.register(Box::new(error_total.clone()))?;
        registry.register(Box::new(cpu_utilization.clone()))?;
        registry.register(Box::new(memory_usage.clone()))?;
        registry.register(Box::new(active_connections.clone()))?;
        registry.register(Box::new(analysis_requests_total.clone()))?;
        registry.register(Box::new(analysis_duration.clone()))?;
        registry.register(Box::new(analysis_queue_length.clone()))?;
        registry.register(Box::new(analysis_code_complexity.clone()))?;
        registry.register(Box::new(analysis_bugs_found.clone()))?;
        registry.register(Box::new(analysis_duplicated_lines.clone()))?;
        registry.register(Box::new(rendering_requests_total.clone()))?;
        registry.register(Box::new(rendering_duration.clone()))?;
        registry.register(Box::new(security_auth_failures.clone()))?;
        registry.register(Box::new(security_permission_denials.clone()))?;
        registry.register(Box::new(security_rate_limit_exceeded.clone()))?;
        registry.register(Box::new(circuit_breaker_state.clone()))?;
        registry.register(Box::new(circuit_breaker_transitions.clone()))?;
        registry.register(Box::new(retry_attempts.clone()))?;
        registry.register(Box::new(fallback_activations.clone()))?;
        registry.register(Box::new(slo_availability.clone()))?;
        registry.register(Box::new(slo_latency_p99.clone()))?;
        registry.register(Box::new(error_budget_remaining.clone()))?;
        registry.register(Box::new(dlq_size.clone()))?;

        Ok(Self {
            registry,
            request_total,
            request_duration,
            error_total,
            cpu_utilization,
            memory_usage,
            active_connections,
            analysis_requests_total,
            analysis_duration,
            analysis_queue_length,
            analysis_code_complexity,
            analysis_bugs_found,
            analysis_duplicated_lines,
            rendering_requests_total,
            rendering_duration,
            security_auth_failures,
            security_permission_denials,
            security_rate_limit_exceeded,
            circuit_breaker_state,
            circuit_breaker_transitions,
            retry_attempts,
            fallback_activations,
            slo_availability,
            slo_latency_p99,
            error_budget_remaining,
            dlq_size,
        })
    }

    /// Get the Prometheus registry for exposing metrics
    pub fn registry(&self) -> Arc<Registry> {
        self.registry.clone()
    }

    /// Record an HTTP request
    pub fn record_request(&self, method: &str, endpoint: &str, status: &str, duration: Duration) {
        self.request_total
            .with_label_values(&[method, endpoint, status])
            .inc();

        self.request_duration
            .with_label_values(&[method, endpoint])
            .observe(duration.as_secs_f64());
    }

    /// Record an error
    pub fn record_error(&self, error_type: &str, severity: &str, component: &str) {
        self.error_total
            .with_label_values(&[error_type, severity, component])
            .inc();
    }

    /// Update system resource metrics
    pub fn update_system_metrics(&self, cpu_percent: f64, memory_bytes: f64, connections: f64) {
        self.cpu_utilization.set(cpu_percent);
        self.memory_usage.set(memory_bytes);
        self.active_connections.set(connections);
    }

    /// Record an analysis request
    pub fn record_analysis_request(
        &self,
        status: &str,
        error_type: Option<&str>,
        pipeline_stage: &str,
        duration: Duration,
        language: &str,
    ) {
        self.analysis_requests_total
            .with_label_values(&[status, error_type.unwrap_or("none"), pipeline_stage])
            .inc();

        self.analysis_duration
            .with_label_values(&[pipeline_stage, language])
            .observe(duration.as_secs_f64());
    }

    /// Update analysis queue length
    pub fn set_analysis_queue_length(&self, length: usize) {
        self.analysis_queue_length.set(length as f64);
    }

    /// Record code complexity metric
    pub fn record_code_complexity(&self, complexity_type: &str, language: &str, score: f64) {
        self.analysis_code_complexity
            .with_label_values(&[complexity_type, language])
            .observe(score);
    }

    /// Record bugs found
    pub fn record_bugs_found(&self, severity: &str, category: &str, language: &str, count: u64) {
        self.analysis_bugs_found
            .with_label_values(&[severity, category, language])
            .inc_by(count as f64);
    }

    /// Record duplicated lines
    pub fn record_duplicated_lines(&self, lines: u64) {
        self.analysis_duplicated_lines.inc_by(lines as f64);
    }

    /// Record rendering service call
    pub fn record_rendering_call(&self, status: &str, diagram_type: &str, duration: Duration) {
        self.rendering_requests_total
            .with_label_values(&[status, diagram_type])
            .inc();

        self.rendering_duration
            .with_label_values(&[diagram_type])
            .observe(duration.as_secs_f64());
    }

    /// Record security authentication failure
    pub fn record_auth_failure(&self, reason: &str, source: &str) {
        self.security_auth_failures
            .with_label_values(&[reason, source])
            .inc();
    }

    /// Record security permission denial
    pub fn record_permission_denial(&self, resource_type: &str, permission_level: &str) {
        self.security_permission_denials
            .with_label_values(&[resource_type, permission_level])
            .inc();
    }

    /// Record rate limit exceeded event
    pub fn record_rate_limit_exceeded(&self, endpoint: &str, user_group: &str) {
        self.security_rate_limit_exceeded
            .with_label_values(&[endpoint, user_group])
            .inc();
    }

    /// Update circuit breaker state
    pub fn update_circuit_breaker_state(
        &self,
        service: &str,
        endpoint: &str,
        state: CircuitBreakerState,
    ) {
        let state_value = match state {
            CircuitBreakerState::Closed => 0.0,
            CircuitBreakerState::Open => 1.0,
            CircuitBreakerState::HalfOpen => 2.0,
        };

        self.circuit_breaker_state
            .with_label_values(&[service, endpoint])
            .set(state_value);
    }

    /// Record circuit breaker transition
    pub fn record_circuit_breaker_transition(
        &self,
        service: &str,
        from_state: CircuitBreakerState,
        to_state: CircuitBreakerState,
    ) {
        self.circuit_breaker_transitions
            .with_label_values(&[
                service,
                &format!("{:?}", from_state),
                &format!("{:?}", to_state),
            ])
            .inc();
    }

    /// Record retry attempt
    pub fn record_retry_attempt(&self, service: &str, operation: &str, final_outcome: &str) {
        self.retry_attempts
            .with_label_values(&[service, operation, final_outcome])
            .inc();
    }

    /// Record fallback activation
    pub fn record_fallback_activation(&self, service: &str, fallback_type: &str, reason: &str) {
        self.fallback_activations
            .with_label_values(&[service, fallback_type, reason])
            .inc();
    }

    /// Update SLO metrics
    pub fn update_slo_metrics(
        &self,
        availability: f64,
        latency_p99: f64,
        error_budget_remaining: f64,
    ) {
        self.slo_availability.set(availability);
        self.slo_latency_p99.set(latency_p99);
        self.error_budget_remaining.set(error_budget_remaining);
    }

    /// Get Dead Letter Queue size gauge for direct manipulation
    pub fn dlq_size(&self) -> &Gauge {
        &self.dlq_size
    }
}

/// Circuit breaker states for metrics
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

/// Timer utility for measuring operation duration
pub struct MetricsTimer {
    start: Instant,
    trace_id: Option<TraceId>,
}

impl MetricsTimer {
    /// Start a new timer
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
            trace_id: None,
        }
    }

    /// Start a new timer with a trace ID
    pub fn start_with_trace_id(trace_id: TraceId) -> Self {
        Self {
            start: Instant::now(),
            trace_id: Some(trace_id),
        }
    }

    /// Get the elapsed duration
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Get the trace ID if available
    pub fn trace_id(&self) -> Option<TraceId> {
        self.trace_id
    }
}

/// Metrics server for exposing Prometheus metrics via HTTP
pub struct MetricsServer {
    metrics: UveddiMetrics,
    config: MetricsConfig,
}

impl MetricsServer {
    /// Create a new metrics server
    pub fn new(metrics: UveddiMetrics, config: MetricsConfig) -> Self {
        Self { metrics, config }
    }

    /// Start the metrics server
    pub async fn start(&self) -> Result<()> {
        use axum::{
            http::StatusCode,
            response::{IntoResponse, Response},
            routing::get,
            Router,
        };

        let registry = self.metrics.registry();

        let app = Router::new()
            .route(&self.config.endpoint_path, get(metrics_handler))
            .with_state(registry);

        let addr = format!("{}:{}", self.config.bind_address, self.config.port);
        let listener = tokio::net::TcpListener::bind(&addr)
            .await
            .with_context(|| format!("Failed to bind metrics server to {}", addr))?;

        tracing::info!(
            address = %addr,
            endpoint = %self.config.endpoint_path,
            "Metrics server started"
        );

        axum::serve(listener, app)
            .await
            .context("Metrics server error")?;

        Ok(())
    }
}

/// Handler for the metrics endpoint
async fn metrics_handler(
    axum::extract::State(registry): axum::extract::State<Arc<Registry>>,
) -> axum::response::Response {
    use prometheus::TextEncoder;

    let encoder = TextEncoder::new();
    let metric_families = registry.gather();

    match encoder.encode_to_string(&metric_families) {
        Ok(output) => (axum::http::StatusCode::OK, output).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to encode metrics");
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to encode metrics",
            )
                .into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observability::config::MetricsConfig;

    #[test]
    fn test_metrics_creation() {
        let config = MetricsConfig::default();
        let metrics = UveddiMetrics::new(&config).unwrap();

        // Test that we can record metrics without panicking
        metrics.record_request("GET", "/api/analyze", "200", Duration::from_millis(100));
        metrics.record_error("analysis_error", "high", "engine");
        metrics.update_system_metrics(50.0, 1024.0 * 1024.0 * 1024.0, 10.0);
    }

    #[test]
    fn test_metrics_timer() {
        let timer = MetricsTimer::start();
        std::thread::sleep(Duration::from_millis(10));
        let elapsed = timer.elapsed();
        assert!(elapsed >= Duration::from_millis(10));
    }
}
