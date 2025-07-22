//! UV-86 Observability Framework Demo
//!
//! This example demonstrates the comprehensive observability and resilience capabilities
//! implemented in UV-86, including structured logging, metrics collection, circuit breakers,
//! retry mechanisms, and graceful degradation patterns.

use anyhow::Result;
use std::time::Duration;
use tokio::time::sleep;

use uveddi::error::UveddiError;
use uveddi::observability::{
    generate_trace_id, InstrumentedCircuitBreaker, InstrumentedFallback, InstrumentedRetry,
    ObservabilityConfig, ObservabilityService, TraceId,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the UV-86 observability system
    let config = ObservabilityConfig::default();
    let mut observability = ObservabilityService::new(config).await?;

    // Start all observability services
    observability.start().await?;

    tracing::info!("UV-86 Observability Demo Started");

    // Generate a trace ID for this demo session
    let demo_trace_id = generate_trace_id();

    // Demo 1: Structured Logging with Trace Correlation
    demo_structured_logging(demo_trace_id).await?;

    // Demo 2: Metrics Collection
    demo_metrics_collection(&observability, demo_trace_id).await?;

    // Demo 3: Circuit Breaker Pattern
    demo_circuit_breaker(&observability, demo_trace_id).await?;

    // Demo 4: Retry Mechanisms
    demo_retry_mechanisms(&observability, demo_trace_id).await?;

    // Demo 5: Graceful Degradation
    demo_graceful_degradation(&observability, demo_trace_id).await?;

    // Demo 6: Health Monitoring
    demo_health_monitoring(&observability, demo_trace_id).await?;

    tracing::info!(
        trace_id = %demo_trace_id,
        "UV-86 Observability Demo Completed Successfully"
    );

    // Keep the metrics server running for a bit to allow inspection
    tracing::info!("Metrics server running at http://localhost:9090/metrics");
    tracing::info!("Press Ctrl+C to exit");

    sleep(Duration::from_secs(60)).await;

    // Gracefully stop the observability service
    observability.stop().await?;

    Ok(())
}

/// Demo 1: Structured Logging with Trace Correlation
async fn demo_structured_logging(trace_id: TraceId) -> Result<()> {
    tracing::info!(
        trace_id = %trace_id,
        demo = "structured_logging",
        "Starting structured logging demonstration"
    );

    // Simulate an analysis operation with structured logging
    let _span = tracing::info_span!("analysis_operation",
        trace_id = %trace_id,
        operation = "file_analysis",
        file_path = "/example/src/main.rs"
    )
    .entered();

    tracing::info!(
        file_size_bytes = 1024,
        language = "rust",
        "File loaded for analysis"
    );

    tracing::warn!(
        complexity_score = 15.7,
        threshold = 10.0,
        "High complexity detected in function"
    );

    tracing::info!(
        issues_found = 3,
        analysis_duration_ms = 150,
        "Analysis completed successfully"
    );

    Ok(())
}

/// Demo 2: Metrics Collection
async fn demo_metrics_collection(
    observability: &ObservabilityService,
    trace_id: TraceId,
) -> Result<()> {
    tracing::info!(
        trace_id = %trace_id,
        demo = "metrics_collection",
        "Starting metrics collection demonstration"
    );

    let metrics = observability.metrics();

    // Simulate analysis requests with different outcomes
    for i in 0..10 {
        let success = i % 3 != 0; // 2/3 success rate
        let duration = Duration::from_millis(50 + (i * 10));

        if success {
            metrics.record_analysis_request("success", None, "syntax_parsing", duration, "rust");

            metrics.record_code_complexity("cyclomatic", "rust", (i as f64) * 2.5);

            if i % 5 == 0 {
                metrics.record_bugs_found("medium", "code_smell", "rust", 1);
            }
        } else {
            metrics.record_analysis_request(
                "failure",
                Some("parse_error"),
                "syntax_parsing",
                duration,
                "rust",
            );

            metrics.record_error("analysis_error", "medium", "parser");
        }

        metrics.record_request(
            "POST",
            "/api/analyze",
            if success { "200" } else { "400" },
            duration,
        );
    }

    // Update system metrics
    metrics.update_system_metrics(45.0, 1024.0 * 1024.0 * 512.0, 15.0);

    // Update SLO metrics
    metrics.update_slo_metrics(99.5, 2.1, 85.0);

    tracing::info!(
        trace_id = %trace_id,
        metrics_recorded = 10,
        "Metrics collection demonstration completed"
    );

    Ok(())
}

/// Demo 3: Circuit Breaker Pattern
async fn demo_circuit_breaker(
    observability: &ObservabilityService,
    trace_id: TraceId,
) -> Result<()> {
    tracing::info!(
        trace_id = %trace_id,
        demo = "circuit_breaker",
        "Starting circuit breaker demonstration"
    );

    let metrics = observability.metrics();
    let telemetry = observability.telemetry();

    let mut circuit_breaker = InstrumentedCircuitBreaker::new(
        "external_service".to_string(),
        3,                      // failure threshold
        Duration::from_secs(5), // reset timeout
        metrics.clone(),
        telemetry.cloned(),
    );

    // Simulate requests that will trigger circuit breaker
    for i in 0..10 {
        let request_trace_id = generate_trace_id();

        let result = circuit_breaker
            .call(request_trace_id, simulate_external_service_call(i))
            .await;

        match result {
            Ok(response) => {
                tracing::info!(
                    trace_id = %request_trace_id,
                    request_id = i,
                    response = %response,
                    "External service call succeeded"
                );
            }
            Err(error) => {
                tracing::warn!(
                    trace_id = %request_trace_id,
                    request_id = i,
                    error = %error,
                    "External service call failed"
                );
            }
        }

        sleep(Duration::from_millis(100)).await;
    }

    Ok(())
}

/// Demo 4: Retry Mechanisms
async fn demo_retry_mechanisms(
    observability: &ObservabilityService,
    trace_id: TraceId,
) -> Result<()> {
    tracing::info!(
        trace_id = %trace_id,
        demo = "retry_mechanisms",
        "Starting retry mechanisms demonstration"
    );

    let metrics = observability.metrics();
    let telemetry = observability.telemetry();

    let retry = InstrumentedRetry::new(
        3,                          // max attempts
        Duration::from_millis(100), // initial delay
        Duration::from_secs(2),     // max delay
        2.0,                        // backoff multiplier
        true,                       // jitter
        metrics.clone(),
        telemetry.cloned(),
    );

    // Simulate operations that succeed after retries
    for i in 0..3 {
        let operation_trace_id = generate_trace_id();

        let result = retry
            .call(
                operation_trace_id,
                "data_service",
                "fetch_analysis_data",
                || simulate_flaky_operation(i),
            )
            .await;

        match result {
            Ok(data) => {
                tracing::info!(
                    trace_id = %operation_trace_id,
                    operation_id = i,
                    data = %data,
                    "Flaky operation succeeded"
                );
            }
            Err(error) => {
                tracing::error!(
                    trace_id = %operation_trace_id,
                    operation_id = i,
                    error = %error,
                    "Flaky operation failed after all retries"
                );
            }
        }
    }

    Ok(())
}

/// Demo 5: Graceful Degradation
async fn demo_graceful_degradation(
    observability: &ObservabilityService,
    trace_id: TraceId,
) -> Result<()> {
    tracing::info!(
        trace_id = %trace_id,
        demo = "graceful_degradation",
        "Starting graceful degradation demonstration"
    );

    let metrics = observability.metrics();
    let telemetry = observability.telemetry();

    let fallback = InstrumentedFallback::new(
        "rendering_service".to_string(),
        metrics.clone(),
        telemetry.cloned(),
    );

    // Demonstrate fallback from primary to secondary service
    for i in 0..3 {
        let request_trace_id = generate_trace_id();

        let result = fallback
            .call_with_fallback(
                request_trace_id,
                "cache_fallback",
                || simulate_primary_rendering_service(i),
                || simulate_fallback_rendering_service(),
            )
            .await;

        match result {
            Ok(diagram) => {
                tracing::info!(
                    trace_id = %request_trace_id,
                    request_id = i,
                    diagram_type = %diagram,
                    "Diagram rendering completed"
                );
            }
            Err(error) => {
                tracing::error!(
                    trace_id = %request_trace_id,
                    request_id = i,
                    error = %error,
                    "Both primary and fallback rendering failed"
                );
            }
        }
    }

    Ok(())
}

/// Demo 6: Health Monitoring
async fn demo_health_monitoring(
    observability: &ObservabilityService,
    trace_id: TraceId,
) -> Result<()> {
    tracing::info!(
        trace_id = %trace_id,
        demo = "health_monitoring",
        "Starting health monitoring demonstration"
    );

    // Check health status
    let health = observability.health_check().await;
    tracing::info!(
        trace_id = %trace_id,
        healthy = health.healthy,
        running = health.running,
        metrics_server = health.metrics_server,
        telemetry_collector = health.telemetry_collector,
        "Health check completed"
    );

    // Get observability statistics
    let stats = observability.get_stats().await;
    tracing::info!(
        trace_id = %trace_id,
        metrics_enabled = stats.metrics_enabled,
        telemetry_enabled = stats.telemetry_enabled,
        audit_integration = stats.audit_integration_enabled,
        pii_redaction = stats.pii_redaction_enabled,
        slo_availability_target = stats.slo_availability_target,
        slo_latency_target = stats.slo_latency_p99_target_ms,
        "Observability statistics retrieved"
    );

    Ok(())
}

// Simulation functions

async fn simulate_external_service_call(request_id: usize) -> Result<String, UveddiError> {
    sleep(Duration::from_millis(50)).await;

    if request_id >= 2 && request_id <= 5 {
        // Simulate failures that will trigger circuit breaker
        Err(UveddiError::NetworkError {
            operation: "external_call".to_string(),
            url: "https://external-service.example.com".to_string(),
            status: "503".to_string(),
            suggestion: "Service temporarily unavailable".to_string(),
            source: None,
        })
    } else {
        Ok(format!(
            "Response from external service for request {}",
            request_id
        ))
    }
}

async fn simulate_flaky_operation(operation_id: usize) -> Result<String, UveddiError> {
    sleep(Duration::from_millis(30)).await;

    // Succeeds on the 2nd or 3rd attempt
    static mut CALL_COUNT: usize = 0;
    unsafe {
        CALL_COUNT += 1;
        if CALL_COUNT % 3 == 0 || CALL_COUNT % 3 == 2 {
            Ok(format!(
                "Data retrieved successfully for operation {}",
                operation_id
            ))
        } else {
            Err(UveddiError::GenericError {
                message: "Transient network error".to_string(),
                context: "flaky_operation".to_string(),
                suggestion: "Retry the operation".to_string(),
                source: None,
            })
        }
    }
}

async fn simulate_primary_rendering_service(request_id: usize) -> Result<String, UveddiError> {
    sleep(Duration::from_millis(100)).await;

    if request_id == 1 {
        // Simulate primary service failure
        Err(UveddiError::RenderingServiceError {
            service_url: "http://rendering-service:8080".to_string(),
            message: "Primary rendering service overloaded".to_string(),
            suggestion: "Try fallback service".to_string(),
            source: None,
        })
    } else {
        Ok("high_quality_diagram".to_string())
    }
}

async fn simulate_fallback_rendering_service() -> Result<String, UveddiError> {
    sleep(Duration::from_millis(50)).await;
    Ok("cached_diagram".to_string())
}
