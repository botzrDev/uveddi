# UV-86: Enterprise-Grade Observability and Resilience Framework

## Overview

UV-86 implements a comprehensive observability and resilience framework for the Uveddi project, providing enterprise-grade monitoring, error handling, and graceful degradation capabilities. This implementation aligns with the detailed specifications in [UV-86_Research.md](06-research/Specialized/UV-86/UV-86_Research.md).

## Architecture

The UV-86 framework is built around five core pillars:

### 1. Structured Logging
- **Framework**: Built on Rust's `tracing` ecosystem
- **Output**: JSON format for production with structured fields
- **Correlation**: Universal `trace_id` for cross-system correlation
- **PII Protection**: Configurable redaction of sensitive data
- **Performance**: Asynchronous logging with minimal overhead

### 2. Metrics Collection
- **System**: Prometheus-based metrics with `/metrics` HTTP endpoint
- **Coverage**: Four Golden Signals (Latency, Traffic, Errors, Saturation)
- **Custom Metrics**: Analysis-specific metrics for code quality and complexity
- **SLO Tracking**: Service Level Objectives monitoring and alerting
- **Integration**: Security events exported as metrics

### 3. Graceful Degradation
- **Circuit Breakers**: Automatic failure detection and isolation
- **Fallback Strategies**: Cache-based and partial result fallbacks
- **User Communication**: Clear degradation state communication
- **Recovery**: Automatic service restoration detection

### 4. Error Recovery
- **Retry Logic**: Exponential backoff with jitter
- **Circuit Integration**: Layered defense with circuit breakers
- **Dead Letter Queue**: Persistent storage for failed operations
- **Classification**: Intelligent error type classification

### 5. Unified Observability
- **Correlation**: Cross-system event correlation via trace IDs
- **Integration**: Deep integration with UV-247 security audit system
- **Dashboards**: Role-specific monitoring dashboards
- **Alerting**: SLO-based alerting with error budget tracking

## Quick Start

### Basic Setup

```rust
use uveddi::observability::{ObservabilityConfig, ObservabilityService};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize observability with default configuration
    let config = ObservabilityConfig::default();
    let mut observability = ObservabilityService::new(config).await?;
    
    // Start all observability services
    observability.start().await?;
    
    // Your application code here
    tracing::info!("Application started with UV-86 observability");
    
    // Stop observability services on shutdown
    observability.stop().await?;
    Ok(())
}
```

### Configuration

```rust
use uveddi::observability::config::*;

let config = ObservabilityConfig {
    logging: LoggingConfig {
        level: "info".to_string(),
        format: LogFormat::Json,
        async_logging: true,
        pii_redaction: PiiRedactionConfig {
            enabled: true,
            strategy: RedactionStrategy::Hash,
            ..Default::default()
        },
        ..Default::default()
    },
    metrics: MetricsConfig {
        enabled: true,
        port: 9090,
        slo_config: SloConfig {
            availability_target: 99.9,
            latency_p99_target_ms: 5000,
            error_budget_window_days: 28,
        },
        ..Default::default()
    },
    resilience: ResilienceConfig {
        circuit_breaker: CircuitBreakerConfig {
            failure_rate_threshold: 0.5,
            minimum_throughput: 10,
            wait_duration: Duration::from_secs(60),
            ..Default::default()
        },
        retry: RetryConfig {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(60),
            backoff_multiplier: 2.0,
            jitter: true,
        },
        ..Default::default()
    },
    ..Default::default()
};
```

## Usage Examples

### Structured Logging with Trace Correlation

```rust
use uveddi::observability::{generate_trace_id, traced_info};

async fn analyze_file(file_path: &str) -> Result<()> {
    let trace_id = generate_trace_id();
    
    let _span = tracing::info_span!("file_analysis", 
        trace_id = %trace_id,
        file_path = file_path
    ).entered();
    
    tracing::info!(
        file_size_bytes = 1024,
        language = "rust",
        "Starting file analysis"
    );
    
    // Analysis logic here
    
    tracing::info!(
        issues_found = 3,
        duration_ms = 150,
        "Analysis completed"
    );
    
    Ok(())
}
```

### Metrics Collection

```rust
use std::time::Duration;

async fn record_analysis_metrics(observability: &ObservabilityService) {
    let metrics = observability.metrics();
    
    // Record analysis request
    metrics.record_analysis_request(
        "success",
        None,
        "syntax_parsing",
        Duration::from_millis(150),
        "rust"
    );
    
    // Record code complexity
    metrics.record_code_complexity("cyclomatic", "rust", 12.5);
    
    // Record bugs found
    metrics.record_bugs_found("medium", "code_smell", "rust", 2);
    
    // Update system metrics
    metrics.update_system_metrics(45.0, 1024.0 * 1024.0 * 512.0, 15.0);
}
```

### Circuit Breaker Pattern

```rust
use uveddi::observability::{InstrumentedCircuitBreaker, generate_trace_id};

async fn call_external_service(observability: &ObservabilityService) -> Result<String> {
    let trace_id = generate_trace_id();
    let metrics = observability.metrics();
    let telemetry = observability.telemetry();
    
    let mut circuit_breaker = InstrumentedCircuitBreaker::new(
        "external_service".to_string(),
        3, // failure threshold
        Duration::from_secs(60), // reset timeout
        metrics.clone(),
        telemetry.cloned(),
    );
    
    circuit_breaker.call(trace_id, async {
        // Your external service call here
        external_service_api_call().await
    }).await
}
```

### Retry Mechanisms

```rust
use uveddi::observability::{InstrumentedRetry, generate_trace_id};

async fn fetch_with_retry(observability: &ObservabilityService) -> Result<String> {
    let trace_id = generate_trace_id();
    let metrics = observability.metrics();
    
    let retry = InstrumentedRetry::new(
        3, // max attempts
        Duration::from_millis(100), // initial delay
        Duration::from_secs(2), // max delay
        2.0, // backoff multiplier
        true, // jitter
        metrics.clone(),
        None, // telemetry
    );
    
    retry.call(
        trace_id,
        "data_service",
        "fetch_data",
        || async {
            // Your operation that might fail
            fetch_data_from_service().await
        }
    ).await
}
```

### Graceful Degradation

```rust
use uveddi::observability::{InstrumentedFallback, generate_trace_id};

async fn render_with_fallback(observability: &ObservabilityService) -> Result<String> {
    let trace_id = generate_trace_id();
    let metrics = observability.metrics();
    
    let fallback = InstrumentedFallback::new(
        "rendering_service".to_string(),
        metrics.clone(),
        None, // telemetry
    );
    
    fallback.call_with_fallback(
        trace_id,
        "cache_fallback",
        || async {
            // Primary rendering service
            primary_rendering_service().await
        },
        || async {
            // Fallback to cached or simplified rendering
            cached_rendering_fallback().await
        }
    ).await
}
```

## Metrics Reference

### Four Golden Signals

| Metric | Type | Description | Labels |
|--------|------|-------------|--------|
| `uveddi_requests_total` | Counter | Total HTTP requests | `method`, `endpoint`, `status` |
| `uveddi_request_duration_seconds` | Histogram | Request duration | `method`, `endpoint` |
| `uveddi_errors_total` | Counter | Total errors | `type`, `severity`, `component` |
| `uveddi_cpu_utilization_percent` | Gauge | CPU utilization | - |

### Analysis-Specific Metrics

| Metric | Type | Description | Labels |
|--------|------|-------------|--------|
| `uveddi_analysis_requests_total` | Counter | Analysis requests | `status`, `error_type`, `pipeline_stage` |
| `uveddi_analysis_duration_seconds` | Histogram | Analysis duration | `pipeline_stage`, `language` |
| `uveddi_analysis_code_complexity_score` | Histogram | Code complexity distribution | `complexity_type`, `language` |
| `uveddi_analysis_bugs_found_total` | Counter | Bugs found | `severity`, `category`, `language` |

### Resilience Metrics

| Metric | Type | Description | Labels |
|--------|------|-------------|--------|
| `uveddi_circuit_breaker_state` | Gauge | Circuit breaker state (0=closed, 1=open, 2=half-open) | `service`, `endpoint` |
| `uveddi_circuit_breaker_transitions_total` | Counter | State transitions | `service`, `from_state`, `to_state` |
| `uveddi_retry_attempts_total` | Counter | Retry attempts | `service`, `operation`, `final_outcome` |
| `uveddi_fallback_activations_total` | Counter | Fallback activations | `service`, `fallback_type`, `reason` |

### Service Level Objectives

| Metric | Type | Description |
|--------|------|-------------|
| `uveddi_slo_availability_percent` | Gauge | Current availability percentage |
| `uveddi_slo_latency_p99_seconds` | Gauge | Current P99 latency |
| `uveddi_error_budget_remaining_percent` | Gauge | Remaining error budget |

## Configuration Reference

### Logging Configuration

```toml
[observability.logging]
level = "info"
format = "Json"  # or "Human" for development
async_logging = true

[observability.logging.pii_redaction]
enabled = true
strategy = "Hash"  # "Mask", "Hash", "Remove", "Partial"
field_patterns = [
    ".*password.*",
    ".*token.*",
    ".*secret.*",
    ".*email.*"
]

[observability.logging.rotation]
max_file_size = 104857600  # 100MB
max_files = 10
compress = true
```

### Metrics Configuration

```toml
[observability.metrics]
enabled = true
bind_address = "0.0.0.0"
port = 9090
endpoint_path = "/metrics"
collection_interval = "30s"

[observability.metrics.slo_config]
availability_target = 99.9
latency_p99_target_ms = 5000
error_budget_window_days = 28
```

### Resilience Configuration

```toml
[observability.resilience.circuit_breaker]
failure_rate_threshold = 0.5
minimum_throughput = 10
wait_duration = "60s"
permitted_calls_in_half_open_state = 5
sliding_window_size = 100

[observability.resilience.retry]
max_attempts = 3
initial_delay = "100ms"
max_delay = "60s"
backoff_multiplier = 2.0
jitter = true

[observability.resilience.fallback]
enable_cache_fallback = true
cache_ttl = "300s"
enable_partial_results = true
```

## Integration with UV-247 Security System

The UV-86 framework seamlessly integrates with the UV-247 security audit system through:

### Trace ID Correlation
- Universal `trace_id` included in all operational logs
- Same `trace_id` included in UV-247 audit events
- Enables cross-system investigation and correlation

### Security Event Metrics
- Security events exported as Prometheus metrics
- Real-time correlation between security and performance
- Dashboards showing security posture alongside operational health

### Audit Integration Configuration

```rust
let config = ObservabilityConfig {
    security: SecurityConfig {
        enable_audit_integration: true,
        audit_service_url: Some("http://uv247-audit:8080".to_string()),
        export_security_metrics: true,
        event_sampling_rate: 1.0, // 100% sampling
    },
    ..Default::default()
};
```

## Deployment Considerations

### Docker Environment

```dockerfile
# Expose metrics port
EXPOSE 9090

# Set logging configuration
ENV RUST_LOG=info
ENV UVEDDI_METRICS_PORT=9090
ENV UVEDDI_LOG_FORMAT=json
```

### Kubernetes

```yaml
apiVersion: v1
kind: Service
metadata:
  name: uveddi-metrics
  labels:
    app: uveddi
spec:
  ports:
  - port: 9090
    name: metrics
  selector:
    app: uveddi
---
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: uveddi
spec:
  selector:
    matchLabels:
      app: uveddi
  endpoints:
  - port: metrics
    interval: 30s
    path: /metrics
```

### Grafana Dashboard

Import the provided dashboard configuration from `monitoring/grafana/uv86-dashboard.json` for:
- Four Golden Signals monitoring
- Analysis pipeline health
- Circuit breaker status
- SLO compliance tracking
- Security event correlation

### Alerting Rules

Example Prometheus alerting rules:

```yaml
groups:
- name: uveddi.rules
  rules:
  - alert: UveddiHighErrorRate
    expr: rate(uveddi_errors_total[5m]) > 0.1
    for: 2m
    labels:
      severity: warning
    annotations:
      summary: "High error rate in Uveddi"
      
  - alert: UveddiSLOBreach
    expr: uveddi_slo_availability_percent < 99.9
    for: 5m
    labels:
      severity: critical
    annotations:
      summary: "SLO availability breach"
      
  - alert: UveddiCircuitBreakerOpen
    expr: uveddi_circuit_breaker_state > 0
    for: 1m
    labels:
      severity: warning
    annotations:
      summary: "Circuit breaker opened for {{ $labels.service }}"
```

## Performance Impact

The UV-86 implementation is designed for minimal performance impact:

- **Logging**: < 1ms overhead per operation with async logging
- **Metrics**: Negligible impact using in-memory counters/histograms
- **Circuit Breakers**: < 0.1ms overhead per protected call
- **Memory**: ~10MB baseline overhead for metrics collection

## Testing

Comprehensive test coverage includes:

```bash
# Run all observability tests
cargo test observability

# Run specific component tests
cargo test observability::metrics
cargo test observability::resilience
cargo test observability::telemetry

# Run the demo example
cargo run --example uv86_observability_demo

# Load testing with observability
cargo run --example load_test_with_observability
```

## Troubleshooting

### Common Issues

1. **Metrics server not starting**
   - Check port availability (default 9090)
   - Verify network configuration
   - Check logs for binding errors

2. **Missing trace correlation**
   - Ensure trace_id propagation across async boundaries
   - Verify UV-247 integration configuration
   - Check structured logging format

3. **Circuit breaker not triggering**
   - Verify failure threshold configuration
   - Check error classification logic
   - Review failure rate calculations

4. **High memory usage**
   - Check metrics cardinality
   - Review PII redaction patterns
   - Monitor log buffer sizes

### Debug Configuration

```rust
let config = ObservabilityConfig {
    logging: LoggingConfig {
        level: "debug".to_string(),
        format: LogFormat::Human, // For debugging
        ..Default::default()
    },
    ..Default::default()
};
```

## Future Enhancements

Planned improvements for UV-86:

1. **Advanced PII Detection**: ML-based PII detection and redaction
2. **Distributed Tracing**: OpenTelemetry integration for cross-service tracing
3. **Anomaly Detection**: ML-based anomaly detection in metrics
4. **Auto-scaling Integration**: Metrics-driven auto-scaling triggers
5. **Cost Optimization**: Intelligent sampling and retention policies

## Compliance and Security

UV-86 supports enterprise compliance requirements:

- **SOC 2**: Availability, security, and processing integrity controls
- **ISO 27001**: Logging, monitoring, and access control requirements
- **GDPR**: PII redaction and data protection capabilities
- **HIPAA**: Audit trail and access logging (with proper configuration)

## Support and Documentation

- **API Documentation**: Run `cargo doc --open` for detailed API docs
- **Example Code**: See `examples/uv86_observability_demo.rs`
- **Configuration Reference**: See `src/observability/config.rs`
- **Architecture Details**: See `docs/06-research/Specialized/UV-86/UV-86_Research.md`

---

UV-86 provides a production-ready observability and resilience framework that enables Uveddi to operate reliably at enterprise scale while maintaining comprehensive visibility into system behavior and security posture.