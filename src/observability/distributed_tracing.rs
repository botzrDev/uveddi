//! Distributed tracing implementation for Uveddi
//!
//! Provides distributed tracing capabilities using OpenTelemetry for:
//! - Analysis workflow tracing
//! - Cross-service correlation
//! - Performance monitoring
//! - Debugging complex operations

use crate::observability::tracing_utils::{generate_trace_id, TraceId};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Span context for distributed tracing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanContext {
    pub trace_id: TraceId,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub baggage: HashMap<String, String>,
    pub trace_flags: u8,
}

impl SpanContext {
    pub fn new(trace_id: TraceId) -> Self {
        Self {
            trace_id,
            span_id: Uuid::new_v4().to_string(),
            parent_span_id: None,
            baggage: HashMap::new(),
            trace_flags: 0,
        }
    }

    pub fn child(&self) -> Self {
        Self {
            trace_id: self.trace_id,
            span_id: Uuid::new_v4().to_string(),
            parent_span_id: Some(self.span_id.clone()),
            baggage: self.baggage.clone(),
            trace_flags: self.trace_flags,
        }
    }

    pub fn add_baggage(&mut self, key: String, value: String) {
        self.baggage.insert(key, value);
    }
}

/// Span represents a unit of work in a trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Span {
    pub context: SpanContext,
    pub operation_name: String,
    pub start_time: SystemTime,
    pub end_time: Option<SystemTime>,
    pub duration: Option<Duration>,
    pub tags: HashMap<String, String>,
    pub logs: Vec<SpanLog>,
    pub status: SpanStatus,
    pub service_name: String,
}

/// Log entry within a span
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanLog {
    pub timestamp: SystemTime,
    pub level: LogLevel,
    pub message: String,
    pub fields: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
}

/// Status of a span
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SpanStatus {
    Ok,
    Error(String),
    Cancelled,
}

impl Span {
    pub fn new(context: SpanContext, operation_name: String, service_name: String) -> Self {
        Self {
            context,
            operation_name,
            start_time: SystemTime::now(),
            end_time: None,
            duration: None,
            tags: HashMap::new(),
            logs: Vec::new(),
            status: SpanStatus::Ok,
            service_name,
        }
    }

    pub fn add_tag(&mut self, key: String, value: String) {
        self.tags.insert(key, value);
    }

    pub fn add_log(&mut self, level: LogLevel, message: String, fields: HashMap<String, serde_json::Value>) {
        self.logs.push(SpanLog {
            timestamp: SystemTime::now(),
            level,
            message,
            fields,
        });
    }

    pub fn set_error(&mut self, error: String) {
        self.status = SpanStatus::Error(error.clone());
        self.add_tag("error".to_string(), "true".to_string());
        self.add_log(
            LogLevel::Error, 
            error,
            HashMap::new()
        );
    }

    pub fn finish(&mut self) {
        let end_time = SystemTime::now();
        self.end_time = Some(end_time);
        self.duration = Some(
            end_time.duration_since(self.start_time)
                .unwrap_or(Duration::ZERO)
        );
    }

    pub fn is_finished(&self) -> bool {
        self.end_time.is_some()
    }
}

/// Active span tracker
pub struct SpanTracker {
    spans: Arc<RwLock<HashMap<String, Span>>>,
    service_name: String,
}

impl SpanTracker {
    pub fn new(service_name: String) -> Self {
        Self {
            spans: Arc::new(RwLock::new(HashMap::new())),
            service_name,
        }
    }

    /// Start a new span
    pub async fn start_span(&self, operation_name: String, parent_context: Option<SpanContext>) -> SpanContext {
        let context = match parent_context {
            Some(parent) => parent.child(),
            None => SpanContext::new(generate_trace_id()),
        };

        let span = Span::new(context.clone(), operation_name, self.service_name.clone());
        
        {
            let mut spans = self.spans.write().await;
            spans.insert(context.span_id.clone(), span);
        }

        tracing::info!(
            trace_id = %context.trace_id,
            span_id = %context.span_id,
            parent_span_id = ?context.parent_span_id,
            "Span started"
        );

        context
    }

    /// Get an active span
    pub async fn get_span(&self, span_id: &str) -> Option<Span> {
        let spans = self.spans.read().await;
        spans.get(span_id).cloned()
    }

    /// Add tag to active span
    pub async fn add_tag(&self, span_id: &str, key: String, value: String) {
        let mut spans = self.spans.write().await;
        if let Some(span) = spans.get_mut(span_id) {
            span.add_tag(key, value);
        }
    }

    /// Add log to active span
    pub async fn add_log(&self, span_id: &str, level: LogLevel, message: String, fields: HashMap<String, serde_json::Value>) {
        let mut spans = self.spans.write().await;
        if let Some(span) = spans.get_mut(span_id) {
            span.add_log(level, message, fields);
        }
    }

    /// Set error on span
    pub async fn set_error(&self, span_id: &str, error: String) {
        let mut spans = self.spans.write().await;
        if let Some(span) = spans.get_mut(span_id) {
            span.set_error(error);
        }
    }

    /// Finish a span
    pub async fn finish_span(&self, span_id: &str) -> Option<Span> {
        let mut spans = self.spans.write().await;
        if let Some(span) = spans.get_mut(span_id) {
            span.finish();
            
            tracing::info!(
                trace_id = %span.context.trace_id,
                span_id = %span.context.span_id,
                operation = %span.operation_name,
                duration_ms = span.duration.map(|d| d.as_millis()).unwrap_or(0),
                status = ?span.status,
                "Span finished"
            );
            
            Some(span.clone())
        } else {
            None
        }
    }

    /// Get all spans for a trace
    pub async fn get_trace_spans(&self, trace_id: TraceId) -> Vec<Span> {
        let spans = self.spans.read().await;
        spans.values()
            .filter(|span| span.context.trace_id == trace_id)
            .cloned()
            .collect()
    }
}

/// Analysis-specific tracing utilities
pub struct AnalysisTracer {
    span_tracker: Arc<SpanTracker>,
}

impl AnalysisTracer {
    pub fn new(service_name: String) -> Self {
        Self {
            span_tracker: Arc::new(SpanTracker::new(service_name)),
        }
    }

    /// Trace a complete analysis workflow
    pub async fn trace_analysis_workflow<F, T>(&self, 
        project_id: &str,
        analysis_type: &str,
        workflow_fn: F
    ) -> Result<T>
    where
        F: std::future::Future<Output = Result<T>>,
    {
        let context = self.span_tracker.start_span(
            format!("analysis_workflow_{}", analysis_type),
            None
        ).await;
        
        // Add analysis context
        self.span_tracker.add_tag(&context.span_id, "project.id".to_string(), project_id.to_string()).await;
        self.span_tracker.add_tag(&context.span_id, "analysis.type".to_string(), analysis_type.to_string()).await;
        self.span_tracker.add_tag(&context.span_id, "service.name".to_string(), "uveddi".to_string()).await;
        
        let start_time = Instant::now();
        let result = workflow_fn.await;
        let duration = start_time.elapsed();
        
        match &result {
            Ok(_) => {
                self.span_tracker.add_tag(&context.span_id, "success".to_string(), "true".to_string()).await;
                self.span_tracker.add_log(
                    &context.span_id,
                    LogLevel::Info,
                    "Analysis completed successfully".to_string(),
                    {
                        let mut fields = HashMap::new();
                        fields.insert("duration_ms".to_string(), serde_json::Value::Number(
                            serde_json::Number::from(duration.as_millis() as u64)
                        ));
                        fields
                    }
                ).await;
            }
            Err(e) => {
                self.span_tracker.set_error(&context.span_id, e.to_string()).await;
            }
        }
        
        self.span_tracker.finish_span(&context.span_id).await;
        result
    }

    /// Trace file parsing operations
    pub async fn trace_file_parsing<F, T>(&self,
        parent_context: &SpanContext,
        file_path: &str,
        language: &str,
        parse_fn: F
    ) -> Result<T>
    where
        F: std::future::Future<Output = Result<T>>,
    {
        let context = self.span_tracker.start_span(
            "file_parsing".to_string(),
            Some(parent_context.clone())
        ).await;
        
        // Add file parsing context
        self.span_tracker.add_tag(&context.span_id, "file.path".to_string(), file_path.to_string()).await;
        self.span_tracker.add_tag(&context.span_id, "file.language".to_string(), language.to_string()).await;
        
        let start_time = Instant::now();
        let result = parse_fn.await;
        let duration = start_time.elapsed();
        
        match &result {
            Ok(_) => {
                self.span_tracker.add_log(
                    &context.span_id,
                    LogLevel::Info,
                    format!("File parsed successfully: {}", file_path),
                    {
                        let mut fields = HashMap::new();
                        fields.insert("parsing_duration_ms".to_string(), serde_json::Value::Number(
                            serde_json::Number::from(duration.as_millis() as u64)
                        ));
                        fields
                    }
                ).await;
            }
            Err(e) => {
                self.span_tracker.set_error(&context.span_id, format!("Parsing failed for {}: {}", file_path, e)).await;
            }
        }
        
        self.span_tracker.finish_span(&context.span_id).await;
        result
    }

    /// Trace detector execution
    pub async fn trace_detector_execution<F, T>(&self,
        parent_context: &SpanContext,
        detector_name: &str,
        execution_fn: F
    ) -> Result<T>
    where
        F: std::future::Future<Output = Result<T>>,
    {
        let context = self.span_tracker.start_span(
            format!("detector_{}", detector_name),
            Some(parent_context.clone())
        ).await;
        
        self.span_tracker.add_tag(&context.span_id, "detector.name".to_string(), detector_name.to_string()).await;
        
        let start_time = Instant::now();
        let result = execution_fn.await;
        let duration = start_time.elapsed();
        
        match &result {
            Ok(_) => {
                self.span_tracker.add_log(
                    &context.span_id,
                    LogLevel::Info,
                    format!("Detector {} executed successfully", detector_name),
                    {
                        let mut fields = HashMap::new();
                        fields.insert("execution_duration_ms".to_string(), serde_json::Value::Number(
                            serde_json::Number::from(duration.as_millis() as u64)
                        ));
                        fields
                    }
                ).await;
            }
            Err(e) => {
                self.span_tracker.set_error(&context.span_id, format!("Detector {} failed: {}", detector_name, e)).await;
            }
        }
        
        self.span_tracker.finish_span(&context.span_id).await;
        result
    }

    /// Trace AI analysis operations
    pub async fn trace_ai_analysis<F, T>(&self,
        parent_context: &SpanContext,
        model_name: &str,
        prompt_type: &str,
        ai_fn: F
    ) -> Result<T>
    where
        F: std::future::Future<Output = Result<T>>,
    {
        let context = self.span_tracker.start_span(
            "ai_analysis".to_string(),
            Some(parent_context.clone())
        ).await;
        
        self.span_tracker.add_tag(&context.span_id, "ai.model".to_string(), model_name.to_string()).await;
        self.span_tracker.add_tag(&context.span_id, "ai.prompt_type".to_string(), prompt_type.to_string()).await;
        
        let start_time = Instant::now();
        let result = ai_fn.await;
        let duration = start_time.elapsed();
        
        match &result {
            Ok(_) => {
                self.span_tracker.add_log(
                    &context.span_id,
                    LogLevel::Info,
                    format!("AI analysis completed with model {}", model_name),
                    {
                        let mut fields = HashMap::new();
                        fields.insert("inference_duration_ms".to_string(), serde_json::Value::Number(
                            serde_json::Number::from(duration.as_millis() as u64)
                        ));
                        fields.insert("model".to_string(), serde_json::Value::String(model_name.to_string()));
                        fields
                    }
                ).await;
            }
            Err(e) => {
                self.span_tracker.set_error(&context.span_id, format!("AI analysis failed: {}", e)).await;
            }
        }
        
        self.span_tracker.finish_span(&context.span_id).await;
        result
    }

    /// Get the span tracker for manual operations
    pub fn span_tracker(&self) -> Arc<SpanTracker> {
        self.span_tracker.clone()
    }

    /// Extract trace ID from current context
    pub async fn current_trace_id(&self, span_id: &str) -> Option<TraceId> {
        self.span_tracker.get_span(span_id).await.map(|span| span.context.trace_id)
    }
}

/// Tracing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {
    pub service_name: String,
    pub enable_logging: bool,
    pub enable_metrics: bool,
    pub sampling_rate: f64, // 0.0 to 1.0
    pub max_spans_per_trace: usize,
    pub span_timeout_seconds: u64,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            service_name: "uveddi".to_string(),
            enable_logging: true,
            enable_metrics: true,
            sampling_rate: 1.0, // Trace everything in development
            max_spans_per_trace: 1000,
            span_timeout_seconds: 300, // 5 minutes
        }
    }
}

/// Distributed tracing manager
pub struct DistributedTracingManager {
    tracer: AnalysisTracer,
    config: TracingConfig,
}

impl DistributedTracingManager {
    pub fn new(config: TracingConfig) -> Self {
        let tracer = AnalysisTracer::new(config.service_name.clone());
        
        tracing::info!(
            service_name = %config.service_name,
            sampling_rate = config.sampling_rate,
            "Distributed tracing initialized"
        );
        
        Self { tracer, config }
    }

    pub fn tracer(&self) -> &AnalysisTracer {
        &self.tracer
    }

    pub fn config(&self) -> &TracingConfig {
        &self.config
    }

    /// Check if a trace should be sampled
    pub fn should_sample(&self) -> bool {
        use rand::Rng;
        let mut rng = rand::rng();
        rng.random::<f64>() < self.config.sampling_rate
    }

    /// Create a trace context from HTTP headers (for distributed tracing)
    pub fn extract_trace_context(&self, headers: &HashMap<String, String>) -> Option<SpanContext> {
        // In a real implementation, this would parse standard tracing headers like:
        // - traceparent (W3C Trace Context)
        // - tracestate
        // - b3 headers (Zipkin)
        // - jaeger headers
        
        if let Some(trace_parent) = headers.get("traceparent") {
            // Parse W3C traceparent header: version-trace_id-parent_id-trace_flags
            let parts: Vec<&str> = trace_parent.split('-').collect();
            if parts.len() == 4 {
                if let Ok(trace_id) = uuid::Uuid::parse_str(&format!("{}-{}-{}-{}-{}", 
                    &parts[1][0..8], &parts[1][8..12], &parts[1][12..16], &parts[1][16..20], &parts[1][20..32])) {
                    let mut context = SpanContext::new(TraceId::from(trace_id));
                    context.parent_span_id = Some(parts[2].to_string());
                    context.trace_flags = u8::from_str_radix(parts[3], 16).unwrap_or(0);
                    return Some(context);
                }
            }
        }
        
        None
    }

    /// Inject trace context into HTTP headers
    pub fn inject_trace_context(&self, context: &SpanContext, headers: &mut HashMap<String, String>) {
        // Inject W3C traceparent header
        let traceparent = format!(
            "00-{}-{}-{:02x}",
            context.trace_id.to_string().replace("-", ""),
            context.span_id,
            context.trace_flags
        );
        headers.insert("traceparent".to_string(), traceparent);
        
        // Inject baggage if present
        if !context.baggage.is_empty() {
            let baggage: Vec<String> = context.baggage.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            headers.insert("baggage".to_string(), baggage.join(","));
        }
    }
}

/// Convenience macro for tracing analysis operations
#[macro_export]
macro_rules! trace_analysis {
    ($tracer:expr, $operation:literal, $code:block) => {
        $tracer.trace_analysis_workflow("unknown", $operation, async move {
            $code
        }).await
    };
    
    ($tracer:expr, $parent:expr, $operation:literal, $code:block) => {
        {
            let context = $tracer.span_tracker().start_span($operation.to_string(), Some($parent)).await;
            let result = async move { $code }.await;
            $tracer.span_tracker().finish_span(&context.span_id).await;
            result
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_span_lifecycle() {
        let tracker = SpanTracker::new("test_service".to_string());
        
        // Start span
        let context = tracker.start_span("test_operation".to_string(), None).await;
        assert!(!context.span_id.is_empty());
        assert!(context.parent_span_id.is_none());
        
        // Add tag and log
        tracker.add_tag(&context.span_id, "test.key".to_string(), "test.value".to_string()).await;
        tracker.add_log(&context.span_id, LogLevel::Info, "Test message".to_string(), HashMap::new()).await;
        
        // Finish span
        let span = tracker.finish_span(&context.span_id).await;
        assert!(span.is_some());
        
        let span = span.unwrap();
        assert!(span.is_finished());
        assert_eq!(span.tags.get("test.key"), Some(&"test.value".to_string()));
        assert_eq!(span.logs.len(), 1);
    }

    #[tokio::test]
    async fn test_child_span_creation() {
        let tracker = SpanTracker::new("test_service".to_string());
        
        let parent_context = tracker.start_span("parent_operation".to_string(), None).await;
        let child_context = tracker.start_span("child_operation".to_string(), Some(parent_context.clone())).await;
        
        assert_eq!(child_context.trace_id, parent_context.trace_id);
        assert_eq!(child_context.parent_span_id, Some(parent_context.span_id.clone()));
        assert_ne!(child_context.span_id, parent_context.span_id);
    }

    #[tokio::test]
    async fn test_analysis_tracer() {
        let tracer = AnalysisTracer::new("test_service".to_string());
        
        let result = tracer.trace_analysis_workflow("project123", "security", async {
            tokio::time::sleep(Duration::from_millis(10)).await;
            Ok::<String, anyhow::Error>("Analysis complete".to_string())
        }).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Analysis complete");
    }

    #[tokio::test]
    async fn test_trace_context_extraction() {
        let manager = DistributedTracingManager::new(TracingConfig::default());
        let mut headers = HashMap::new();
        
        // Create a mock traceparent header
        headers.insert("traceparent".to_string(), "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01".to_string());
        
        let context = manager.extract_trace_context(&headers);
        assert!(context.is_some());
        
        let context = context.unwrap();
        assert_eq!(context.parent_span_id, Some("00f067aa0ba902b7".to_string()));
        assert_eq!(context.trace_flags, 1);
    }
}