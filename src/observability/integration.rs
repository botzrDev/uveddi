//! Observability integration service for Uveddi
//!
//! Provides a unified interface for integrating all observability components
//! into the Uveddi application stack including API servers, analysis engines,
//! and background services.

use crate::observability::{
    config::ObservabilityConfig,
    distributed_tracing::{AnalysisTracer, DistributedTracingManager, SpanContext},
    health_check::{HealthCheckSystem, SystemHealthStatus},
    metrics::UveddiMetrics,
    service::ObservabilityService,
    ServiceStatus,
};
use crate::resilience::alerting::AdvancedAlertSystem;
use anyhow::{Context, Result};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Unified observability manager for the entire Uveddi system
pub struct ObservabilityManager {
    config: ObservabilityConfig,
    observability_service: ObservabilityService,
    health_check_system: Arc<HealthCheckSystem>,
    tracing_manager: Arc<DistributedTracingManager>,
    alert_system: Arc<AdvancedAlertSystem>,
    metrics: Arc<UveddiMetrics>,
    analysis_tracer: Arc<AnalysisTracer>,
}

impl ObservabilityManager {
    /// Initialize the complete observability system
    pub async fn new(config: ObservabilityConfig) -> Result<Self> {
        tracing::info!("Initializing comprehensive observability system");
        
        // Initialize core observability service
        let mut observability_service = ObservabilityService::new(config.clone()).await?;
        observability_service.start().await?;
        
        let metrics = observability_service.metrics();
        
        // Initialize health check system
        let health_check_system = Arc::new(HealthCheckSystem::new(
            metrics.clone(),
            config.clone(),
        ));
        
        // Start health monitoring if enabled
        if config.health_check.enabled && config.health_check.background_monitoring {
            health_check_system.start_monitoring(config.health_check.check_interval).await;
        }
        
        // Initialize distributed tracing
        let tracing_config = crate::observability::distributed_tracing::TracingConfig {
            service_name: config.tracing.service_name.clone(),
            enable_logging: true,
            enable_metrics: config.metrics.enabled,
            sampling_rate: config.tracing.sampling_rate,
            max_spans_per_trace: config.tracing.max_spans_per_trace,
            span_timeout_seconds: config.tracing.span_timeout_seconds,
        };
        
        let tracing_manager = Arc::new(DistributedTracingManager::new(tracing_config));
        let analysis_tracer = Arc::new(AnalysisTracer::new(config.tracing.service_name.clone()));
        
        // Initialize alert system (using health monitor from resilience module)
        let health_monitor = Arc::new(crate::resilience::health::HealthMonitor::new());
        let alert_system = Arc::new(AdvancedAlertSystem::new(health_monitor));
        
        tracing::info!(
            health_checks_enabled = config.health_check.enabled,
            tracing_enabled = config.tracing.enabled,
            alerting_enabled = config.alerting.enabled,
            metrics_enabled = config.metrics.enabled,
            "Observability system initialization complete"
        );
        
        Ok(Self {
            config,
            observability_service,
            health_check_system,
            tracing_manager,
            alert_system,
            metrics,
            analysis_tracer,
        })
    }
    
    /// Get the metrics system
    pub fn metrics(&self) -> Arc<UveddiMetrics> {
        self.metrics.clone()
    }
    
    /// Get the health check system
    pub fn health_checker(&self) -> Arc<HealthCheckSystem> {
        self.health_check_system.clone()
    }
    
    /// Get the analysis tracer
    pub fn analysis_tracer(&self) -> Arc<AnalysisTracer> {
        self.analysis_tracer.clone()
    }
    
    /// Get the tracing manager
    pub fn tracing_manager(&self) -> Arc<DistributedTracingManager> {
        self.tracing_manager.clone()
    }
    
    /// Get the alert system
    pub fn alert_system(&self) -> Arc<AdvancedAlertSystem> {
        self.alert_system.clone()
    }
    
    /// Record analysis operation
    pub async fn record_analysis(&self, 
        operation_name: &str,
        project_id: &str,
        language: &str,
        duration: Duration,
        success: bool,
        issues_found: usize,
    ) {
        // Record metrics
        self.metrics.record_analysis_request(
            if success { "success" } else { "failure" },
            if success { None } else { Some("analysis_error") },
            "analysis",
            duration,
            language,
        );
        
        if success {
            self.metrics.record_bugs_found("various", "anti_pattern", language, issues_found as u64);
        } else {
            self.metrics.record_error("analysis_failure", "high", "analysis_engine");
        }
        
        tracing::info!(
            operation = operation_name,
            project_id = project_id,
            language = language,
            duration_ms = duration.as_millis(),
            success = success,
            issues_found = issues_found,
            "Analysis operation recorded"
        );
    }
    
    /// Trace analysis workflow with comprehensive monitoring
    pub async fn trace_analysis_workflow<F, T>(&self,
        project_id: &str,
        analysis_type: &str,
        workflow_fn: F,
    ) -> Result<T>
    where
        F: std::future::Future<Output = Result<T>>,
    {
        if !self.config.tracing.enabled || !self.tracing_manager.should_sample() {
            // Execute without tracing if disabled or not sampled
            return workflow_fn.await;
        }
        
        let start_time = std::time::Instant::now();
        
        let result = self.analysis_tracer.trace_analysis_workflow(
            project_id,
            analysis_type,
            workflow_fn
        ).await;
        
        let duration = start_time.elapsed();
        
        // Record analysis metrics
        self.record_analysis(
            analysis_type,
            project_id,
            "mixed", // Default language when not specified
            duration,
            result.is_ok(),
            0, // Default issues count
        ).await;
        
        result
    }
    
    /// Get comprehensive system health status
    pub async fn get_system_health(&self) -> SystemHealthStatus {
        self.health_check_system.check_health().await
    }
    
    /// Get cached system health status (fast)
    pub async fn get_cached_health(&self) -> SystemHealthStatus {
        self.health_check_system.get_cached_status().await
    }
    
    /// Create middleware for HTTP request monitoring
    pub fn create_http_middleware(&self) -> HttpObservabilityMiddleware {
        HttpObservabilityMiddleware::new(
            self.metrics.clone(),
            self.tracing_manager.clone(),
        )
    }
    
    /// Shutdown the observability system gracefully
    pub async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("Shutting down observability system");
        
        self.observability_service.stop().await?;
        
        tracing::info!("Observability system shutdown complete");
        Ok(())
    }
    
    /// Get observability system status
    pub async fn get_observability_status(&self) -> ObservabilityStatus {
        let health_status = self.get_cached_health().await;
        let service_stats = self.observability_service.get_stats().await;
        
        ObservabilityStatus {
            overall_healthy: health_status.overall_status == ServiceStatus::Healthy,
            metrics_enabled: service_stats.metrics_enabled,
            tracing_enabled: self.config.tracing.enabled,
            alerting_enabled: self.config.alerting.enabled,
            health_checks_enabled: self.config.health_check.enabled,
            components_healthy: health_status.components.len(),
            uptime_seconds: health_status.uptime_seconds,
            version: health_status.version.clone(),
        }
    }
}

/// HTTP middleware for observability
pub struct HttpObservabilityMiddleware {
    metrics: Arc<UveddiMetrics>,
    tracing_manager: Arc<DistributedTracingManager>,
}

impl HttpObservabilityMiddleware {
    pub fn new(
        metrics: Arc<UveddiMetrics>,
        tracing_manager: Arc<DistributedTracingManager>,
    ) -> Self {
        Self {
            metrics,
            tracing_manager,
        }
    }
    
    /// Record HTTP request metrics
    pub async fn record_request(&self,
        method: &str,
        path: &str,
        status_code: u16,
        duration: Duration,
        headers: Option<&std::collections::HashMap<String, String>>,
    ) {
        let status = if status_code < 400 {
            "success"
        } else if status_code < 500 {
            "client_error"  
        } else {
            "server_error"
        };
        
        self.metrics.record_request(method, path, status, duration);
        
        if status_code >= 400 {
            let error_type = if status_code < 500 {
                "client_error"
            } else {
                "server_error"
            };
            self.metrics.record_error(error_type, "medium", "http_server");
        }
        
        // Extract trace context from headers if present
        if let Some(headers) = headers {
            let _trace_context = self.tracing_manager.extract_trace_context(headers);
            // In a full implementation, would propagate trace context
        }
        
        tracing::debug!(
            method = method,
            path = path,
            status_code = status_code,
            duration_ms = duration.as_millis(),
            "HTTP request processed"
        );
    }
}

/// Observability system status
#[derive(Debug, Clone)]
pub struct ObservabilityStatus {
    pub overall_healthy: bool,
    pub metrics_enabled: bool,
    pub tracing_enabled: bool,
    pub alerting_enabled: bool,
    pub health_checks_enabled: bool,
    pub components_healthy: usize,
    pub uptime_seconds: u64,
    pub version: String,
}

/// Convenience functions for common observability operations
pub struct ObservabilityHelpers;

impl ObservabilityHelpers {
    /// Create a correlation ID for request tracking
    pub fn generate_correlation_id() -> String {
        Uuid::new_v4().to_string()
    }
    
    /// Extract user ID from request context (placeholder)
    pub fn extract_user_id(_request: &str) -> Option<String> {
        // In real implementation, would extract from JWT or session
        None
    }
    
    /// Sanitize sensitive data for logging
    pub fn sanitize_for_logging(data: &str) -> String {
        // Simple sanitization - in production would use more sophisticated PII detection
        if data.contains("password") || data.contains("secret") || data.contains("token") {
            "***REDACTED***".to_string()
        } else {
            data.to_string()
        }
    }
    
    /// Format duration for human reading
    pub fn format_duration(duration: Duration) -> String {
        let total_ms = duration.as_millis();
        if total_ms < 1000 {
            format!("{}ms", total_ms)
        } else if total_ms < 60_000 {
            format!("{:.2}s", duration.as_secs_f64())
        } else {
            let minutes = total_ms / 60_000;
            let seconds = (total_ms % 60_000) / 1000;
            format!("{}m {}s", minutes, seconds)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_observability_manager_creation() {
        let config = ObservabilityConfig {
            metrics: crate::observability::config::MetricsConfig {
                enabled: false, // Disable for test
                ..Default::default()
            },
            health_check: crate::observability::config::HealthCheckConfig {
                background_monitoring: false, // Disable for test
                ..Default::default()
            },
            ..Default::default()
        };
        
        let manager = ObservabilityManager::new(config).await;
        assert!(manager.is_ok());
        
        let manager = manager.unwrap();
        let status = manager.get_observability_status().await;
        assert!(!status.metrics_enabled); // Should be disabled as configured
    }
    
    #[test]
    fn test_observability_helpers() {
        let correlation_id = ObservabilityHelpers::generate_correlation_id();
        assert!(!correlation_id.is_empty());
        
        let sanitized = ObservabilityHelpers::sanitize_for_logging("password=secret123");
        assert_eq!(sanitized, "***REDACTED***");
        
        let duration_str = ObservabilityHelpers::format_duration(Duration::from_millis(1500));
        assert_eq!(duration_str, "1.50s");
    }
}