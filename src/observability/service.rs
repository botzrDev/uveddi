//! Main observability service orchestrating all UV-86 components
//!
//! Provides a unified interface for initializing and managing all observability
//! components including structured logging, metrics, telemetry, and resilience patterns.

use crate::observability::{
    config::ObservabilityConfig,
    logging,
    metrics::{MetricsServer, UveddiMetrics},
    telemetry::{TelemetryCollector, TelemetryConfig, TelemetrySender},
    tracing_utils::{generate_trace_id, TraceId},
};
use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

/// Main observability service that coordinates all UV-86 components
pub struct ObservabilityService {
    config: ObservabilityConfig,
    metrics: Arc<UveddiMetrics>,
    telemetry_sender: Option<TelemetrySender>,
    metrics_server_handle: Option<JoinHandle<Result<()>>>,
    telemetry_handle: Option<JoinHandle<Result<(), Box<dyn std::error::Error + Send + Sync>>>>,
    started: Arc<RwLock<bool>>,
}

impl ObservabilityService {
    /// Create a new observability service with the given configuration
    pub async fn new(config: ObservabilityConfig) -> Result<Self> {
        // Initialize structured logging first
        logging::init_structured_logging(&config.logging)
            .context("Failed to initialize structured logging")?;

        tracing::info!(
            message = "Initializing UV-86 observability service",
            version = env!("CARGO_PKG_VERSION"),
            "Starting enterprise-grade observability and resilience framework"
        );

        // Initialize metrics system
        let metrics = Arc::new(
            UveddiMetrics::new(&config.metrics).context("Failed to initialize metrics system")?,
        );

        tracing::info!(
            metrics_enabled = config.metrics.enabled,
            metrics_port = config.metrics.port,
            "Metrics system initialized"
        );

        Ok(Self {
            config,
            metrics,
            telemetry_sender: None,
            metrics_server_handle: None,
            telemetry_handle: None,
            started: Arc::new(RwLock::new(false)),
        })
    }

    /// Start all observability services
    pub async fn start(&mut self) -> Result<()> {
        let mut started = self.started.write().await;
        if *started {
            tracing::warn!("Observability service already started");
            return Ok(());
        }

        let trace_id = generate_trace_id();
        tracing::info!(
            trace_id = %trace_id,
            "Starting UV-86 observability service components"
        );

        // Start telemetry collector
        if self.config.logging.format != crate::observability::config::LogFormat::Human {
            let telemetry_config = TelemetryConfig {
                enable_correlation: true,
                enable_metrics_export: self.config.metrics.enabled,
                enable_audit_integration: self.config.security.enable_audit_integration,
                ..Default::default()
            };

            let telemetry_collector = TelemetryCollector::new(telemetry_config);
            self.telemetry_sender = Some(telemetry_collector.sender());

            self.telemetry_handle = Some(tokio::spawn(
                async move { telemetry_collector.start().await },
            ));

            tracing::info!(
                trace_id = %trace_id,
                "Telemetry collector started"
            );
        }

        // Start metrics server
        if self.config.metrics.enabled {
            let metrics_server =
                MetricsServer::new((*self.metrics).clone(), self.config.metrics.clone());

            self.metrics_server_handle =
                Some(tokio::spawn(async move { metrics_server.start().await }));

            tracing::info!(
                trace_id = %trace_id,
                bind_address = %self.config.metrics.bind_address,
                port = self.config.metrics.port,
                endpoint = %self.config.metrics.endpoint_path,
                "Metrics server started"
            );
        }

        // Log successful startup with SLO targets
        tracing::info!(
            trace_id = %trace_id,
            availability_target = %self.config.metrics.slo_config.availability_target,
            latency_p99_target_ms = self.config.metrics.slo_config.latency_p99_target_ms,
            error_budget_window_days = self.config.metrics.slo_config.error_budget_window_days,
            "UV-86 observability service started successfully with SLO targets configured"
        );

        // Record startup metrics
        self.metrics.record_request(
            "INTERNAL",
            "/start",
            "success",
            std::time::Duration::from_millis(0),
        );

        *started = true;
        Ok(())
    }

    /// Stop all observability services
    pub async fn stop(&mut self) -> Result<()> {
        let mut started = self.started.write().await;
        if !*started {
            tracing::warn!("Observability service not started");
            return Ok(());
        }

        let trace_id = generate_trace_id();
        tracing::info!(
            trace_id = %trace_id,
            "Stopping UV-86 observability service"
        );

        // Stop metrics server
        if let Some(handle) = self.metrics_server_handle.take() {
            handle.abort();
            tracing::info!(trace_id = %trace_id, "Metrics server stopped");
        }

        // Stop telemetry collector
        if let Some(handle) = self.telemetry_handle.take() {
            handle.abort();
            tracing::info!(trace_id = %trace_id, "Telemetry collector stopped");
        }

        self.telemetry_sender = None;

        tracing::info!(
            trace_id = %trace_id,
            "UV-86 observability service stopped successfully"
        );

        *started = false;
        Ok(())
    }

    /// Get the metrics instance for recording metrics
    pub fn metrics(&self) -> Arc<UveddiMetrics> {
        self.metrics.clone()
    }

    /// Get the telemetry sender for sending telemetry events
    pub fn telemetry(&self) -> Option<&TelemetrySender> {
        self.telemetry_sender.as_ref()
    }

    /// Check if the observability service is running
    pub async fn is_running(&self) -> bool {
        *self.started.read().await
    }

    /// Get the current configuration
    pub fn config(&self) -> &ObservabilityConfig {
        &self.config
    }

    /// Create a new trace ID for operation correlation
    pub fn new_trace_id(&self) -> TraceId {
        generate_trace_id()
    }

    /// Record a service health check
    pub async fn health_check(&self) -> HealthStatus {
        let mut status = HealthStatus::new();

        // Check if service is running
        status.running = self.is_running().await;

        // Check metrics server health
        if self.config.metrics.enabled {
            status.metrics_server = self
                .metrics_server_handle
                .as_ref()
                .map(|h| !h.is_finished())
                .unwrap_or(false);
        } else {
            status.metrics_server = true; // Not enabled, so considered healthy
        }

        // Check telemetry collector health
        status.telemetry_collector = self
            .telemetry_handle
            .as_ref()
            .map(|h| !h.is_finished())
            .unwrap_or(true); // May not be enabled

        // Overall health
        status.healthy = status.running && status.metrics_server && status.telemetry_collector;

        // Record health check metrics
        if status.healthy {
            self.metrics.record_request(
                "INTERNAL",
                "/health",
                "success",
                std::time::Duration::from_millis(0),
            );
        } else {
            self.metrics
                .record_error("health_check", "medium", "observability_service");
        }

        status
    }

    /// Get observability statistics
    pub async fn get_stats(&self) -> ObservabilityStats {
        ObservabilityStats {
            running: self.is_running().await,
            metrics_enabled: self.config.metrics.enabled,
            telemetry_enabled: self.telemetry_sender.is_some(),
            audit_integration_enabled: self.config.security.enable_audit_integration,
            pii_redaction_enabled: self.config.logging.pii_redaction.enabled,
            async_logging_enabled: self.config.logging.async_logging,
            slo_availability_target: self.config.metrics.slo_config.availability_target,
            slo_latency_p99_target_ms: self.config.metrics.slo_config.latency_p99_target_ms,
        }
    }
}

/// Health status of the observability service
#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub healthy: bool,
    pub running: bool,
    pub metrics_server: bool,
    pub telemetry_collector: bool,
}

impl HealthStatus {
    fn new() -> Self {
        Self {
            healthy: false,
            running: false,
            metrics_server: false,
            telemetry_collector: false,
        }
    }
}

/// Statistics about the observability service
#[derive(Debug, Clone)]
pub struct ObservabilityStats {
    pub running: bool,
    pub metrics_enabled: bool,
    pub telemetry_enabled: bool,
    pub audit_integration_enabled: bool,
    pub pii_redaction_enabled: bool,
    pub async_logging_enabled: bool,
    pub slo_availability_target: f64,
    pub slo_latency_p99_target_ms: u64,
}

// Graceful shutdown handling
impl Drop for ObservabilityService {
    fn drop(&mut self) {
        // Abort any running tasks on drop
        if let Some(handle) = self.metrics_server_handle.take() {
            handle.abort();
        }

        if let Some(handle) = self.telemetry_handle.take() {
            handle.abort();
        }

        tracing::info!("ObservabilityService dropped, all tasks aborted");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observability::config::{LogFormat, LoggingConfig, ObservabilityConfig};

    #[tokio::test]
    async fn test_observability_service_creation() {
        let mut config = ObservabilityConfig::default();
        // Use human format for tests to avoid complex JSON parsing
        config.logging.format = LogFormat::Human;
        config.metrics.enabled = false; // Disable to avoid port conflicts in tests

        let service = ObservabilityService::new(config).await;
        assert!(service.is_ok());
    }

    #[tokio::test]
    async fn test_observability_service_lifecycle() {
        let mut config = ObservabilityConfig::default();
        config.logging.format = LogFormat::Human;
        config.metrics.enabled = false; // Disable to avoid port conflicts

        let mut service = ObservabilityService::new(config).await.unwrap();

        // Test initial state
        assert!(!service.is_running().await);

        // Test start
        let result = service.start().await;
        assert!(result.is_ok());
        assert!(service.is_running().await);

        // Test stop
        let result = service.stop().await;
        assert!(result.is_ok());
        assert!(!service.is_running().await);
    }

    #[tokio::test]
    async fn test_health_check() {
        let mut config = ObservabilityConfig::default();
        config.logging.format = LogFormat::Human;
        config.metrics.enabled = false;

        let mut service = ObservabilityService::new(config).await.unwrap();

        // Health check before start
        let health = service.health_check().await;
        assert!(!health.healthy);
        assert!(!health.running);

        // Start service
        service.start().await.unwrap();

        // Health check after start
        let health = service.health_check().await;
        assert!(health.running);
        // Note: healthy status depends on all components being enabled and working
    }

    #[tokio::test]
    async fn test_trace_id_generation() {
        let mut config = ObservabilityConfig::default();
        config.logging.format = LogFormat::Human;
        config.metrics.enabled = false;

        let service = ObservabilityService::new(config).await.unwrap();

        let trace_id1 = service.new_trace_id();
        let trace_id2 = service.new_trace_id();

        assert_ne!(trace_id1, trace_id2);
    }
}
