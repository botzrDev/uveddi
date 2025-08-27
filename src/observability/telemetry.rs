//! Telemetry event collection and correlation
//!
//! Provides unified telemetry event handling that correlates logs, metrics, and audit events
//! using trace IDs as defined in the UV-86 specification.

use crate::observability::tracing_utils::TraceId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::Level;

/// A telemetry event that can be correlated across systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryEvent {
    /// Unique trace ID for correlation
    pub trace_id: TraceId,
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    /// Event type
    pub event_type: TelemetryEventType,
    /// Event severity level
    pub severity: TelemetryLevel,
    /// Service or component that generated the event
    pub component: String,
    /// Event message
    pub message: String,
    /// Additional structured fields
    pub fields: HashMap<String, TelemetryValue>,
    /// Optional error information
    pub error: Option<TelemetryError>,
}

/// Types of telemetry events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TelemetryEventType {
    /// Operational log event
    Log,
    /// Metrics event
    Metric,
    /// Security audit event
    SecurityAudit,
    /// Performance event
    Performance,
    /// Error/exception event
    Error,
    /// Circuit breaker state change
    CircuitBreaker,
    /// Fallback activation
    Fallback,
    /// Service degradation
    Degradation,
}

/// Telemetry event severity levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TelemetryLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Critical,
}

impl From<TelemetryLevel> for Level {
    fn from(level: TelemetryLevel) -> Self {
        match level {
            TelemetryLevel::Trace => Level::TRACE,
            TelemetryLevel::Debug => Level::DEBUG,
            TelemetryLevel::Info => Level::INFO,
            TelemetryLevel::Warn => Level::WARN,
            TelemetryLevel::Error => Level::ERROR,
            TelemetryLevel::Critical => Level::ERROR, // Map critical to error for tracing
        }
    }
}

/// Telemetry value types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TelemetryValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Duration(std::time::Duration),
    Timestamp(DateTime<Utc>),
}

/// Error information in telemetry events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryError {
    /// Error type or category
    pub error_type: String,
    /// Error message
    pub message: String,
    /// Error code if applicable
    pub code: Option<String>,
    /// Stack trace or additional context
    pub context: Option<String>,
}

/// Telemetry collector that aggregates and correlates events
pub struct TelemetryCollector {
    /// Event receiver channel
    event_receiver: mpsc::UnboundedReceiver<TelemetryEvent>,
    /// Event sender channel
    event_sender: mpsc::UnboundedSender<TelemetryEvent>,
    /// Configuration
    config: TelemetryConfig,
}

/// Configuration for telemetry collection
#[derive(Debug, Clone)]
pub struct TelemetryConfig {
    /// Buffer size for events
    pub buffer_size: usize,
    /// Enable event correlation
    pub enable_correlation: bool,
    /// Maximum age for correlation context
    pub correlation_ttl: std::time::Duration,
    /// Enable metrics export
    pub enable_metrics_export: bool,
    /// Enable audit log integration
    pub enable_audit_integration: bool,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            buffer_size: 10000,
            enable_correlation: true,
            correlation_ttl: std::time::Duration::from_secs(3600), // 1 hour
            enable_metrics_export: true,
            enable_audit_integration: true,
        }
    }
}

impl TelemetryCollector {
    /// Create a new telemetry collector
    pub fn new(config: TelemetryConfig) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        Self {
            event_receiver,
            event_sender,
            config,
        }
    }

    /// Get a sender handle for submitting telemetry events
    pub fn sender(&self) -> TelemetrySender {
        TelemetrySender {
            sender: self.event_sender.clone(),
        }
    }

    /// Start the telemetry collector event loop
    pub async fn start(mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        tracing::info!("Starting telemetry collector");

        let mut correlation_context = CorrelationContext::new(self.config.correlation_ttl);

        while let Some(event) = self.event_receiver.recv().await {
            self.process_event(&event, &mut correlation_context).await;
        }

        Ok(())
    }

    /// Process a telemetry event
    async fn process_event(
        &self,
        event: &TelemetryEvent,
        correlation_context: &mut CorrelationContext,
    ) {
        // Update correlation context
        if self.config.enable_correlation {
            correlation_context.add_event(event.clone());
        }

        // Log the event using tracing
        self.log_event(event);

        // Export to metrics if enabled
        if self.config.enable_metrics_export {
            self.export_to_metrics(event).await;
        }

        // Send to audit system if enabled
        if self.config.enable_audit_integration
            && matches!(event.event_type, TelemetryEventType::SecurityAudit)
        {
            self.send_to_audit_system(event).await;
        }
    }

    /// Log the event using the tracing system
    fn log_event(&self, event: &TelemetryEvent) {
        // Log based on severity level using static macros
        match event.severity {
            TelemetryLevel::Trace => {
                tracing::trace!(
                    target: "uveddi::telemetry",
                    trace_id = %event.trace_id,
                    component = %event.component,
                    event_type = ?event.event_type,
                    timestamp = %event.timestamp,
                    error_type = event.error.as_ref().map(|e| e.error_type.as_str()),
                    error_code = event.error.as_ref().and_then(|e| e.code.as_deref()),
                    message = %event.message,
                    "Telemetry event"
                );
            }
            TelemetryLevel::Debug => {
                tracing::debug!(
                    target: "uveddi::telemetry",
                    trace_id = %event.trace_id,
                    component = %event.component,
                    event_type = ?event.event_type,
                    timestamp = %event.timestamp,
                    error_type = event.error.as_ref().map(|e| e.error_type.as_str()),
                    error_code = event.error.as_ref().and_then(|e| e.code.as_deref()),
                    message = %event.message,
                    "Telemetry event"
                );
            }
            TelemetryLevel::Info => {
                tracing::info!(
                    target: "uveddi::telemetry",
                    trace_id = %event.trace_id,
                    component = %event.component,
                    event_type = ?event.event_type,
                    timestamp = %event.timestamp,
                    error_type = event.error.as_ref().map(|e| e.error_type.as_str()),
                    error_code = event.error.as_ref().and_then(|e| e.code.as_deref()),
                    message = %event.message,
                    "Telemetry event"
                );
            }
            TelemetryLevel::Warn => {
                tracing::warn!(
                    target: "uveddi::telemetry",
                    trace_id = %event.trace_id,
                    component = %event.component,
                    event_type = ?event.event_type,
                    timestamp = %event.timestamp,
                    error_type = event.error.as_ref().map(|e| e.error_type.as_str()),
                    error_code = event.error.as_ref().and_then(|e| e.code.as_deref()),
                    message = %event.message,
                    "Telemetry event"
                );
            }
            TelemetryLevel::Error | TelemetryLevel::Critical => {
                tracing::error!(
                    target: "uveddi::telemetry",
                    trace_id = %event.trace_id,
                    component = %event.component,
                    event_type = ?event.event_type,
                    timestamp = %event.timestamp,
                    error_type = event.error.as_ref().map(|e| e.error_type.as_str()),
                    error_code = event.error.as_ref().and_then(|e| e.code.as_deref()),
                    message = %event.message,
                    "Telemetry event"
                );
            }
        }
    }

    /// Export event data to metrics system
    async fn export_to_metrics(&self, event: &TelemetryEvent) {
        // This would integrate with the metrics system
        // For now, we'll log the export action
        tracing::debug!(
            trace_id = %event.trace_id,
            event_type = ?event.event_type,
            "Exporting event to metrics system"
        );
    }

    /// Send security audit events to the UV-247 audit system
    async fn send_to_audit_system(&self, event: &TelemetryEvent) {
        // This would integrate with the UV-247 audit system
        // For now, we'll log the audit action
        tracing::info!(
            trace_id = %event.trace_id,
            component = %event.component,
            message = %event.message,
            "Sending event to UV-247 audit system"
        );
    }
}

/// Handle for sending telemetry events
#[derive(Clone)]
pub struct TelemetrySender {
    sender: mpsc::UnboundedSender<TelemetryEvent>,
}

impl TelemetrySender {
    /// Send a telemetry event
    pub fn send(
        &self,
        event: TelemetryEvent,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.sender.send(event)?;
        Ok(())
    }

    /// Send a log event
    pub fn log(
        &self,
        trace_id: TraceId,
        level: TelemetryLevel,
        component: &str,
        message: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let event = TelemetryEvent {
            trace_id,
            timestamp: Utc::now(),
            event_type: TelemetryEventType::Log,
            severity: level,
            component: component.to_string(),
            message: message.to_string(),
            fields: HashMap::new(),
            error: None,
        };
        self.send(event)
    }

    /// Send an error event
    pub fn error(
        &self,
        trace_id: TraceId,
        component: &str,
        error_type: &str,
        message: &str,
        context: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let event = TelemetryEvent {
            trace_id,
            timestamp: Utc::now(),
            event_type: TelemetryEventType::Error,
            severity: TelemetryLevel::Error,
            component: component.to_string(),
            message: message.to_string(),
            fields: HashMap::new(),
            error: Some(TelemetryError {
                error_type: error_type.to_string(),
                message: message.to_string(),
                code: None,
                context: context.map(|s| s.to_string()),
            }),
        };
        self.send(event)
    }

    /// Send a security audit event
    pub fn security_audit(
        &self,
        trace_id: TraceId,
        component: &str,
        message: &str,
        fields: HashMap<String, TelemetryValue>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let event = TelemetryEvent {
            trace_id,
            timestamp: Utc::now(),
            event_type: TelemetryEventType::SecurityAudit,
            severity: TelemetryLevel::Info,
            component: component.to_string(),
            message: message.to_string(),
            fields,
            error: None,
        };
        self.send(event)
    }

    /// Send a circuit breaker event
    pub fn circuit_breaker(
        &self,
        trace_id: TraceId,
        component: &str,
        state_change: &str,
        reason: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut fields = HashMap::new();
        fields.insert(
            "state_change".to_string(),
            TelemetryValue::String(state_change.to_string()),
        );
        fields.insert(
            "reason".to_string(),
            TelemetryValue::String(reason.to_string()),
        );

        let event = TelemetryEvent {
            trace_id,
            timestamp: Utc::now(),
            event_type: TelemetryEventType::CircuitBreaker,
            severity: TelemetryLevel::Warn,
            component: component.to_string(),
            message: format!("Circuit breaker {} - {}", state_change, reason),
            fields,
            error: None,
        };
        self.send(event)
    }
}

/// Correlation context for tracking related events
struct CorrelationContext {
    events: HashMap<TraceId, Vec<TelemetryEvent>>,
    ttl: std::time::Duration,
}

impl CorrelationContext {
    fn new(ttl: std::time::Duration) -> Self {
        Self {
            events: HashMap::new(),
            ttl,
        }
    }

    fn add_event(&mut self, event: TelemetryEvent) {
        let trace_id = event.trace_id;
        self.events
            .entry(trace_id)
            .or_insert_with(Vec::new)
            .push(event);

        // Clean up old events
        self.cleanup_old_events();
    }

    fn cleanup_old_events(&mut self) {
        let cutoff = Utc::now() - chrono::Duration::from_std(self.ttl).unwrap_or_default();

        self.events.retain(|_, events| {
            events.retain(|event| event.timestamp > cutoff);
            !events.is_empty()
        });
    }

    #[allow(dead_code)]
    fn get_correlated_events(&self, trace_id: TraceId) -> Option<&Vec<TelemetryEvent>> {
        self.events.get(&trace_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_event_creation() {
        let trace_id = TraceId::new();
        let event = TelemetryEvent {
            trace_id,
            timestamp: Utc::now(),
            event_type: TelemetryEventType::Log,
            severity: TelemetryLevel::Info,
            component: "test".to_string(),
            message: "test message".to_string(),
            fields: HashMap::new(),
            error: None,
        };

        assert_eq!(event.trace_id, trace_id);
        assert_eq!(event.component, "test");
        assert_eq!(event.message, "test message");
    }

    #[tokio::test]
    async fn test_telemetry_sender() {
        let config = TelemetryConfig::default();
        let collector = TelemetryCollector::new(config);
        let sender = collector.sender();

        let trace_id = TraceId::new();
        let result = sender.log(trace_id, TelemetryLevel::Info, "test", "test message");

        assert!(result.is_ok());
    }
}
