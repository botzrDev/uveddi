# Task Assignment: Production Monitoring and Observability

## Priority: 🟡 HIGH PRIORITY - Operations Requirement

## Problem Statement
Current system lacks comprehensive monitoring and observability required for production deployment. Without proper metrics, logs, and alerting, production issues will be difficult to detect, diagnose, and resolve quickly.

## Objective
Implement comprehensive monitoring, observability, and alerting infrastructure to ensure reliable production operations and rapid issue resolution.

## Scope of Work

### Current Observability Gaps:
- No centralized logging with structured data
- Missing application performance monitoring (APM)
- No business metrics tracking
- Insufficient error monitoring and alerting
- No distributed tracing for analysis workflows
- Limited health checking and service discovery

### Observability Components:

#### 1. Structured Logging Framework
```rust
use tracing::{info, warn, error, debug, span, instrument};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use serde_json::json;

pub struct ObservabilityManager {
    logger: Logger,
    metrics: MetricsCollector,
    tracer: TracingManager,
}

impl ObservabilityManager {
    pub fn init() -> Result<Self> {
        // Initialize structured logging
        let logger = Logger::new()
            .with_json_format()
            .with_correlation_ids()
            .with_context_enrichment();
        
        // Initialize metrics collection
        let metrics = MetricsCollector::new()
            .with_prometheus_exporter()
            .with_custom_metrics();
        
        // Initialize distributed tracing
        let tracer = TracingManager::new()
            .with_jaeger_exporter()
            .with_sampling_strategy();
        
        Ok(Self { logger, metrics, tracer })
    }
}

// Structured logging with context
#[instrument(skip(self), fields(project_id = %project.id, analysis_type = %analysis_type))]
pub async fn analyze_project(&self, project: &Project, analysis_type: AnalysisType) -> Result<AnalysisResult> {
    let span = span!(tracing::Level::INFO, "project_analysis");
    let _enter = span.enter();
    
    info!(
        project_id = %project.id,
        file_count = project.files.len(),
        estimated_duration = ?self.estimate_duration(project),
        "Starting project analysis"
    );
    
    match self.perform_analysis(project, analysis_type).await {
        Ok(result) => {
            info!(
                project_id = %project.id,
                issues_found = result.issues.len(),
                analysis_duration = ?result.duration,
                "Analysis completed successfully"
            );
            Ok(result)
        },
        Err(e) => {
            error!(
                project_id = %project.id,
                error = %e,
                error_type = std::any::type_name::<typeof(e)>(),
                "Analysis failed"
            );
            Err(e)
        }
    }
}
```

#### 2. Metrics Collection and Exposition
```rust
use prometheus::{Encoder, TextEncoder, Counter, Histogram, Gauge, Registry};
use std::collections::HashMap;

pub struct MetricsCollector {
    registry: Registry,
    
    // Business metrics
    analyses_total: Counter,
    analysis_duration: Histogram,
    issues_detected_total: Counter,
    
    // System metrics
    memory_usage: Gauge,
    active_connections: Gauge,
    cpu_usage: Gauge,
    
    // Error metrics
    errors_total: Counter,
    error_by_type: HashMap<String, Counter>,
}

impl MetricsCollector {
    pub fn new() -> Result<Self> {
        let registry = Registry::new();
        
        let analyses_total = Counter::new("uveddi_analyses_total", 
            "Total number of analyses performed")?;
        registry.register(Box::new(analyses_total.clone()))?;
        
        let analysis_duration = Histogram::with_opts(
            prometheus::HistogramOpts::new("uveddi_analysis_duration_seconds", 
                "Time spent analyzing projects")
                .buckets(vec![1.0, 5.0, 10.0, 30.0, 60.0, 300.0, 1800.0])
        )?;
        registry.register(Box::new(analysis_duration.clone()))?;
        
        let issues_detected_total = Counter::new("uveddi_issues_detected_total", 
            "Total number of issues detected")?;
        registry.register(Box::new(issues_detected_total.clone()))?;
        
        let memory_usage = Gauge::new("uveddi_memory_usage_bytes", 
            "Current memory usage in bytes")?;
        registry.register(Box::new(memory_usage.clone()))?;
        
        let active_connections = Gauge::new("uveddi_active_connections", 
            "Number of active database connections")?;
        registry.register(Box::new(active_connections.clone()))?;
        
        let errors_total = Counter::new("uveddi_errors_total", 
            "Total number of errors")?;
        registry.register(Box::new(errors_total.clone()))?;
        
        Ok(Self {
            registry,
            analyses_total,
            analysis_duration,
            issues_detected_total,
            memory_usage,
            active_connections,
            cpu_usage: Gauge::new("uveddi_cpu_usage_percent", "CPU usage percentage")?,
            errors_total,
            error_by_type: HashMap::new(),
        })
    }
    
    pub fn record_analysis(&self, duration: Duration, issues_count: usize) {
        self.analyses_total.inc();
        self.analysis_duration.observe(duration.as_secs_f64());
        self.issues_detected_total.inc_by(issues_count as f64);
    }
    
    pub fn record_error(&self, error_type: &str) {
        self.errors_total.inc();
        
        if let Some(counter) = self.error_by_type.get(error_type) {
            counter.inc();
        }
    }
    
    pub fn export_metrics(&self) -> Result<String> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer)?;
        Ok(String::from_utf8(buffer)?)
    }
}
```

#### 3. Health Check System
```rust
use tokio::time::{Duration, timeout};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthStatus {
    pub overall_status: ServiceStatus,
    pub components: HashMap<String, ComponentHealth>,
    pub timestamp: DateTime<Utc>,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ServiceStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub status: ServiceStatus,
    pub last_check: DateTime<Utc>,
    pub response_time_ms: u64,
    pub error_message: Option<String>,
    pub details: serde_json::Value,
}

pub struct HealthChecker {
    database_health: DatabaseHealthChecker,
    plugin_system_health: PluginSystemHealthChecker,
    memory_health: MemoryHealthChecker,
    external_services_health: ExternalServicesHealthChecker,
}

impl HealthChecker {
    pub async fn check_health(&self) -> HealthStatus {
        let mut components = HashMap::new();
        
        // Check database health
        let db_health = timeout(Duration::from_secs(5), self.database_health.check()).await
            .unwrap_or_else(|_| ComponentHealth {
                status: ServiceStatus::Unhealthy,
                last_check: Utc::now(),
                response_time_ms: 5000,
                error_message: Some("Database health check timeout".to_string()),
                details: json!({}),
            });
        components.insert("database".to_string(), db_health);
        
        // Check plugin system health
        let plugin_health = timeout(Duration::from_secs(3), self.plugin_system_health.check()).await
            .unwrap_or_else(|_| ComponentHealth {
                status: ServiceStatus::Unhealthy,
                last_check: Utc::now(),
                response_time_ms: 3000,
                error_message: Some("Plugin system health check timeout".to_string()),
                details: json!({}),
            });
        components.insert("plugin_system".to_string(), plugin_health);
        
        // Check memory health
        let memory_health = self.memory_health.check().await;
        components.insert("memory".to_string(), memory_health);
        
        // Check external services
        let external_health = self.external_services_health.check().await;
        components.insert("external_services".to_string(), external_health);
        
        let overall_status = self.calculate_overall_status(&components);
        
        HealthStatus {
            overall_status,
            components,
            timestamp: Utc::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
    
    fn calculate_overall_status(&self, components: &HashMap<String, ComponentHealth>) -> ServiceStatus {
        let unhealthy_count = components.values()
            .filter(|c| matches!(c.status, ServiceStatus::Unhealthy))
            .count();
        let degraded_count = components.values()
            .filter(|c| matches!(c.status, ServiceStatus::Degraded))
            .count();
        
        if unhealthy_count > 0 {
            ServiceStatus::Unhealthy
        } else if degraded_count > 0 {
            ServiceStatus::Degraded
        } else {
            ServiceStatus::Healthy
        }
    }
}
```

#### 4. Alerting System
```rust
use tokio::sync::mpsc;
use reqwest::Client;

pub struct AlertManager {
    alert_rules: Vec<AlertRule>,
    notification_channels: Vec<NotificationChannel>,
    alert_sender: mpsc::UnboundedSender<Alert>,
}

#[derive(Debug, Clone)]
pub struct Alert {
    pub level: AlertLevel,
    pub component: String,
    pub message: String,
    pub details: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub correlation_id: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AlertLevel {
    Info,
    Warning,
    Critical,
    Emergency,
}

pub struct AlertRule {
    pub name: String,
    pub condition: Box<dyn AlertCondition + Send + Sync>,
    pub level: AlertLevel,
    pub cooldown: Duration,
    pub last_fired: Option<DateTime<Utc>>,
}

impl AlertManager {
    pub fn new() -> (Self, mpsc::UnboundedReceiver<Alert>) {
        let (alert_sender, alert_receiver) = mpsc::unbounded_channel();
        
        let mut manager = Self {
            alert_rules: Vec::new(),
            notification_channels: Vec::new(),
            alert_sender,
        };
        
        // Add default alert rules
        manager.add_default_rules();
        
        (manager, alert_receiver)
    }
    
    fn add_default_rules(&mut self) {
        // Memory usage alert
        self.alert_rules.push(AlertRule {
            name: "High Memory Usage".to_string(),
            condition: Box::new(MemoryUsageCondition { threshold: 0.8 }),
            level: AlertLevel::Warning,
            cooldown: Duration::from_secs(300),
            last_fired: None,
        });
        
        // Analysis failure rate alert
        self.alert_rules.push(AlertRule {
            name: "High Analysis Failure Rate".to_string(),
            condition: Box::new(FailureRateCondition { threshold: 0.1, window: Duration::from_secs(300) }),
            level: AlertLevel::Critical,
            cooldown: Duration::from_secs(180),
            last_fired: None,
        });
        
        // Database connection alert
        self.alert_rules.push(AlertRule {
            name: "Database Connection Issues".to_string(),
            condition: Box::new(DatabaseConnectionCondition {}),
            level: AlertLevel::Emergency,
            cooldown: Duration::from_secs(60),
            last_fired: None,
        });
    }
    
    pub async fn check_alerts(&mut self, metrics: &MetricsSnapshot) {
        for rule in &mut self.alert_rules {
            if let Some(last_fired) = rule.last_fired {
                if Utc::now().signed_duration_since(last_fired) < rule.cooldown {
                    continue; // Still in cooldown
                }
            }
            
            if rule.condition.evaluate(metrics) {
                let alert = Alert {
                    level: rule.level.clone(),
                    component: "analysis_system".to_string(),
                    message: format!("Alert triggered: {}", rule.name),
                    details: rule.condition.details(metrics),
                    timestamp: Utc::now(),
                    correlation_id: uuid::Uuid::new_v4().to_string(),
                };
                
                let _ = self.alert_sender.send(alert);
                rule.last_fired = Some(Utc::now());
            }
        }
    }
}
```

#### 5. Distributed Tracing
```rust
use opentelemetry::{global, trace::{TraceError, Tracer}, KeyValue};
use opentelemetry_jaeger::new_pipeline;
use tracing_opentelemetry::OpenTelemetryLayer;

pub struct TracingManager {
    tracer: Box<dyn Tracer + Send + Sync>,
}

impl TracingManager {
    pub fn init() -> Result<Self, TraceError> {
        let tracer = new_pipeline()
            .with_service_name("uveddi")
            .with_agent_endpoint("http://localhost:14268/api/traces")
            .install_batch(opentelemetry::runtime::Tokio)?;
            
        global::set_tracer_provider(tracer.clone());
        
        Ok(Self {
            tracer: Box::new(tracer),
        })
    }
    
    #[instrument(skip(self))]
    pub async fn trace_analysis(&self, project_id: &str, analysis_fn: impl Future<Output = Result<AnalysisResult>>) -> Result<AnalysisResult> {
        let span = self.tracer
            .span_builder("project_analysis")
            .with_attributes(vec![
                KeyValue::new("project.id", project_id.to_string()),
                KeyValue::new("service.name", "uveddi"),
                KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
            ])
            .start(&*self.tracer);
        
        let _guard = tracing_opentelemetry::OpenTelemetrySpanExt::set_parent(&span);
        
        let result = analysis_fn.await;
        
        match &result {
            Ok(analysis_result) => {
                span.set_attribute(KeyValue::new("analysis.issues_found", analysis_result.issues.len() as i64));
                span.set_attribute(KeyValue::new("analysis.success", true));
            },
            Err(e) => {
                span.set_attribute(KeyValue::new("analysis.success", false));
                span.set_attribute(KeyValue::new("analysis.error", e.to_string()));
            },
        }
        
        result
    }
}
```

## Expected Outcome
- Comprehensive observability across all system components
- Real-time monitoring of business and technical metrics
- Proactive alerting on system issues
- Detailed distributed tracing for debugging
- Production-ready logging and metrics collection
- Operational dashboards for system health monitoring

## Time Estimate: 2-3 weeks

## Dependencies:
- Prometheus/Grafana setup for metrics visualization
- Jaeger or similar for distributed tracing
- Log aggregation system (ELK stack or similar)
- Alerting infrastructure (PagerDuty, Slack, email)

## Testing Required:
```bash
# Test metrics collection
./scripts/test-metrics-collection.sh

# Test alerting system
./scripts/test-alerting-scenarios.sh  

# Test health check endpoints
./scripts/test-health-checks.sh

# Test distributed tracing
./scripts/test-tracing-integration.sh

# Load test with observability
./scripts/test-observability-under-load.sh
```

## Implementation Phases:

### Week 1: Foundation
- [ ] Implement structured logging framework
- [ ] Set up metrics collection and exposition
- [ ] Create health check system
- [ ] Add basic alerting rules

### Week 2: Advanced Features  
- [ ] Implement distributed tracing
- [ ] Create operational dashboards
- [ ] Add comprehensive alerting
- [ ] Set up log aggregation

### Week 3: Production Integration
- [ ] Integrate with external monitoring systems
- [ ] Create operational runbooks
- [ ] Set up on-call procedures
- [ ] Performance testing with full observability

## Success Metrics:
- [ ] <30 second mean time to detection (MTTD) for critical issues
- [ ] 99.9% uptime monitoring availability
- [ ] Complete trace coverage for all analysis workflows
- [ ] Zero false positive alerts in production
- [ ] <2 minute mean time to acknowledgment (MTTA) for alerts