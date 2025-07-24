# 🚀 UV-219 Test Infrastructure Monitoring Implementation Prompt

## 📋 Implementation Objective

You are tasked with implementing a **comprehensive Test Infrastructure Monitoring System** for the Uveddi project based on detailed research findings. This system must provide real-time test visibility, intelligent failure categorization, performance tracking, and automated reporting for a complex Rust-based static code analysis platform.

## 🎯 Project Context & Research Foundation

### Uveddi Project Overview
- **Technology Stack**: Rust backend (Axum), TypeScript frontend, Node.js rendering service
- **Architecture**: Microservices with resilience patterns, existing metrics/health monitoring
- **Test Infrastructure**: GitHub Actions CI/CD, cargo test + Criterion.rs benchmarks
- **Supported Languages**: Rust, Python, JavaScript/TypeScript
- **Current Gaps**: No centralized dashboard, failure categorization, or performance tracking

### Research-Based Architecture Decision
Based on comprehensive research, the implementation uses:
- **Backend**: Axum framework with QuestDB time-series database
- **Frontend**: Custom React/TypeScript dashboard with WebSocket real-time updates
- **Integration**: Extends existing ErrorMetrics and ComponentHealth systems
- **Performance**: Targets 4.3M+ metrics/second, sub-second dashboard updates
- **Classification**: Hybrid ML/rule-based failure categorization (85% accuracy)

## 🏗️ Implementation Architecture

### System Components Overview
```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Test Runners  │    │  Metrics        │    │  Classification │
│   (Rust/CI)     │──→ │  Aggregation    │──→ │  Engine         │
│                 │    │  Service        │    │                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                        │                        │
         │                        │                        │
         ▼                        ▼                        ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Time-Series   │    │   Real-time     │    │   Dashboard     │
│   Database      │    │   Streaming     │    │   & Analytics   │
│   (QuestDB)     │◄──►│   (WebSocket)   │◄──►│   (React/TS)    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## 📋 Phase 1: MVP Dashboard Implementation (Weeks 1-4)

### 1.1 Backend Infrastructure Setup

#### Create Core Monitoring Service
**File**: `src/monitoring/mod.rs`
```rust
//! Test infrastructure monitoring system (UV-219)
//! 
//! Provides real-time test execution monitoring, failure categorization,
//! and performance tracking for the Uveddi static code analysis platform.

pub mod dashboard;
pub mod metrics;
pub mod websocket;
pub mod database;
pub mod classification;

pub use dashboard::MonitoringDashboard;
pub use metrics::{TestMetrics, TestExecution, TestResult};
pub use websocket::WebSocketManager;
pub use database::MonitoringDatabase;
pub use classification::FailureClassifier;
```

#### Extend Existing Metrics System
**File**: `src/resilience/metrics.rs` (extend existing)
```rust
// Add to existing ErrorMetrics struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestMetrics {
    pub execution_id: String,
    pub test_name: String,
    pub test_suite: String,
    pub status: TestStatus,
    pub duration_ms: u64,
    pub resource_usage: ResourceUsage,
    pub failure_category: Option<FailureCategory>,
    pub timestamp: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestStatus {
    Passed,
    Failed,
    Skipped,
    Timeout,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu_percent: f32,
    pub memory_mb: u64,
    pub disk_io_mb: u64,
}

// Extend existing MetricsCollector
impl MetricsCollector {
    pub async fn collect_test_metrics(&self, test_result: TestResult) -> Result<(), MetricsError> {
        // Implementation for test-specific metrics collection
        // Integrate with existing error metrics system
    }
}
```

#### WebSocket Real-time Communication
**File**: `src/monitoring/websocket.rs`
```rust
use axum::{
    extract::{ws::WebSocket, WebSocketUpgrade},
    response::Response,
};
use tokio::sync::broadcast;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestEvent {
    pub event_type: TestEventType,
    pub test_metrics: TestMetrics,
    pub timestamp: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestEventType {
    TestStarted,
    TestCompleted,
    TestFailed,
    SuiteStarted,
    SuiteCompleted,
}

pub struct WebSocketManager {
    sender: broadcast::Sender<TestEvent>,
}

impl WebSocketManager {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1000);
        Self { sender }
    }

    pub async fn handle_websocket(
        &self,
        ws: WebSocketUpgrade,
    ) -> Response {
        ws.on_upgrade(|socket| self.websocket_handler(socket))
    }

    async fn websocket_handler(&self, socket: WebSocket) {
        // Implementation for real-time test event streaming
        // Sub-second latency for dashboard updates
    }

    pub async fn broadcast_test_event(&self, event: TestEvent) {
        let _ = self.sender.send(event);
    }
}
```

### 1.2 Database Integration

#### QuestDB Time-Series Storage
**File**: `src/monitoring/database.rs`
```rust
use sqlx::{PgPool, Row};
use chrono::{DateTime, Utc};

pub struct MonitoringDatabase {
    pool: PgPool,
}

impl MonitoringDatabase {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPool::connect(database_url).await?;
        Ok(Self { pool })
    }

    pub async fn store_test_metrics(&self, metrics: &TestMetrics) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO test_executions (
                execution_id, test_name, test_suite, status, 
                duration_ms, cpu_percent, memory_mb, timestamp
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            metrics.execution_id,
            metrics.test_name,
            metrics.test_suite,
            metrics.status as TestStatus,
            metrics.duration_ms as i64,
            metrics.resource_usage.cpu_percent,
            metrics.resource_usage.memory_mb as i64,
            metrics.timestamp
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }

    pub async fn get_test_trends(&self, hours: i32) -> Result<Vec<TestTrend>, sqlx::Error> {
        // Implementation for historical trend analysis
        // Optimized queries for dashboard performance
    }
}
```

#### Database Schema (Migration)
**File**: `migrations/V2__test_monitoring_schema.sql`
```sql
-- Test execution tracking
CREATE TABLE test_executions (
    id SERIAL PRIMARY KEY,
    execution_id VARCHAR(255) NOT NULL,
    test_name VARCHAR(500) NOT NULL,
    test_suite VARCHAR(255) NOT NULL,
    status VARCHAR(50) NOT NULL,
    duration_ms BIGINT NOT NULL,
    cpu_percent REAL,
    memory_mb BIGINT,
    disk_io_mb BIGINT,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Test failure details
CREATE TABLE test_failures (
    id SERIAL PRIMARY KEY,
    execution_id VARCHAR(255) REFERENCES test_executions(execution_id),
    failure_category VARCHAR(100),
    error_message TEXT,
    stack_trace TEXT,
    environment_context JSONB,
    classification_confidence REAL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Performance metrics
CREATE TABLE test_performance (
    id SERIAL PRIMARY KEY,
    execution_id VARCHAR(255) REFERENCES test_executions(execution_id),
    metric_name VARCHAR(100) NOT NULL,
    metric_value REAL NOT NULL,
    metric_unit VARCHAR(50),
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL
);

-- Indexes for performance
CREATE INDEX idx_test_executions_timestamp ON test_executions(timestamp);
CREATE INDEX idx_test_executions_suite_status ON test_executions(test_suite, status);
CREATE INDEX idx_test_failures_category ON test_failures(failure_category);
CREATE INDEX idx_test_performance_metric ON test_performance(metric_name, timestamp);
```

### 1.3 GitHub Actions Integration

#### Webhook Endpoint for CI/CD
**File**: `src/monitoring/github_integration.rs`
```rust
use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct GitHubWebhookPayload {
    pub action: String,
    pub workflow_run: Option<WorkflowRun>,
    pub check_suite: Option<CheckSuite>,
}

#[derive(Debug, Deserialize)]
pub struct WorkflowRun {
    pub id: u64,
    pub name: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub html_url: String,
}

pub async fn handle_github_webhook(
    State(monitoring): State<MonitoringDashboard>,
    Json(payload): Json<GitHubWebhookPayload>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    // Process GitHub Actions webhook
    // Parse JUnit XML results
    // Extract test metrics
    // Store in database
    // Broadcast real-time updates
    
    Ok(Json(serde_json::json!({"status": "processed"})))
}

pub async fn parse_junit_xml(xml_content: &str) -> Result<Vec<TestMetrics>, ParseError> {
    // Implementation for JUnit XML parsing
    // Extract test results from cargo test and Criterion.rs
    // Convert to TestMetrics format
}
```

### 1.4 Basic Dashboard Frontend

#### React Dashboard Component
**File**: `frontend/src/components/monitoring/TestDashboard.tsx`
```typescript
import React, { useState, useEffect } from 'react';
import { WebSocketManager } from '../../lib/websocket';
import { TestMetrics, TestEvent } from '../../types/monitoring';

interface TestDashboardProps {
  websocketUrl: string;
}

export const TestDashboard: React.FC<TestDashboardProps> = ({ websocketUrl }) => {
  const [testMetrics, setTestMetrics] = useState<TestMetrics[]>([]);
  const [connectionStatus, setConnectionStatus] = useState<'connected' | 'disconnected'>('disconnected');
  const [wsManager] = useState(() => new WebSocketManager(websocketUrl));

  useEffect(() => {
    const handleTestEvent = (event: TestEvent) => {
      setTestMetrics(prev => [event.test_metrics, ...prev.slice(0, 99)]);
    };

    const handleConnectionChange = (status: 'connected' | 'disconnected') => {
      setConnectionStatus(status);
    };

    wsManager.on('test_event', handleTestEvent);
    wsManager.on('connection_change', handleConnectionChange);
    wsManager.connect();

    return () => {
      wsManager.disconnect();
    };
  }, [wsManager]);

  return (
    <div className="test-dashboard">
      <div className="dashboard-header">
        <h1>Test Infrastructure Monitoring</h1>
        <div className={`connection-status ${connectionStatus}`}>
          {connectionStatus === 'connected' ? '🟢 Live' : '🔴 Disconnected'}
        </div>
      </div>
      
      <div className="dashboard-grid">
        <TestExecutionSummary metrics={testMetrics} />
        <RealTimeTestFeed metrics={testMetrics} />
        <PerformanceTrends />
        <FailureAnalysis />
      </div>
    </div>
  );
};
```

#### WebSocket Client Manager
**File**: `frontend/src/lib/websocket.ts`
```typescript
export class WebSocketManager extends EventTarget {
  private ws: WebSocket | null = null;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  private reconnectDelay = 1000;

  constructor(private url: string) {
    super();
  }

  connect(): void {
    try {
      this.ws = new WebSocket(this.url);
      
      this.ws.onopen = () => {
        this.reconnectAttempts = 0;
        this.dispatchEvent(new CustomEvent('connection_change', { 
          detail: 'connected' 
        }));
      };

      this.ws.onmessage = (event) => {
        const testEvent = JSON.parse(event.data);
        this.dispatchEvent(new CustomEvent('test_event', { 
          detail: testEvent 
        }));
      };

      this.ws.onclose = () => {
        this.handleReconnect();
      };

      this.ws.onerror = (error) => {
        console.error('WebSocket error:', error);
      };
    } catch (error) {
      console.error('Failed to connect WebSocket:', error);
      this.handleReconnect();
    }
  }

  private handleReconnect(): void {
    if (this.reconnectAttempts < this.maxReconnectAttempts) {
      setTimeout(() => {
        this.reconnectAttempts++;
        this.connect();
      }, this.reconnectDelay * Math.pow(2, this.reconnectAttempts));
    } else {
      this.dispatchEvent(new CustomEvent('connection_change', { 
        detail: 'disconnected' 
      }));
    }
  }
}
```

## 📋 Phase 2: Failure Categorization (Weeks 5-8)

### 2.1 Intelligent Failure Classification

#### Rule-Based Classification Engine
**File**: `src/monitoring/classification.rs`
```rust
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FailureCategory {
    Infrastructure,
    CodeLogic,
    TestAutomation,
    Performance,
    Flaky,
    Environment,
}

#[derive(Debug, Clone)]
pub struct FailurePattern {
    pub category: FailureCategory,
    pub pattern: Regex,
    pub confidence: f32,
    pub description: String,
}

pub struct FailureClassifier {
    patterns: Vec<FailurePattern>,
    ml_model: Option<MLClassificationModel>,
}

impl FailureClassifier {
    pub fn new() -> Self {
        let patterns = vec![
            FailurePattern {
                category: FailureCategory::Infrastructure,
                pattern: Regex::new(r"(?i)(connection.*timeout|network.*error|dns.*resolution)").unwrap(),
                confidence: 0.95,
                description: "Network connectivity issues".to_string(),
            },
            FailurePattern {
                category: FailureCategory::CodeLogic,
                pattern: Regex::new(r"(?i)(assertion.*failed|panic.*at|index.*out.*of.*bounds)").unwrap(),
                confidence: 0.90,
                description: "Code logic errors".to_string(),
            },
            FailurePattern {
                category: FailureCategory::Performance,
                pattern: Regex::new(r"(?i)(timeout.*exceeded|execution.*too.*slow|memory.*limit)").unwrap(),
                confidence: 0.85,
                description: "Performance-related failures".to_string(),
            },
            // Add more patterns based on Uveddi-specific error patterns
        ];

        Self {
            patterns,
            ml_model: None,
        }
    }

    pub async fn classify_failure(&self, 
        error_message: &str, 
        stack_trace: &str,
        context: &TestExecutionContext
    ) -> ClassificationResult {
        // Rule-based classification first (high confidence)
        for pattern in &self.patterns {
            if pattern.pattern.is_match(error_message) || pattern.pattern.is_match(stack_trace) {
                return ClassificationResult {
                    category: pattern.category.clone(),
                    confidence: pattern.confidence,
                    method: ClassificationMethod::RuleBased,
                    reasoning: pattern.description.clone(),
                };
            }
        }

        // ML-based classification for complex cases
        if let Some(model) = &self.ml_model {
            return model.classify(error_message, stack_trace, context).await;
        }

        // Default classification
        ClassificationResult {
            category: FailureCategory::CodeLogic,
            confidence: 0.5,
            method: ClassificationMethod::Default,
            reasoning: "Unable to classify automatically".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationResult {
    pub category: FailureCategory,
    pub confidence: f32,
    pub method: ClassificationMethod,
    pub reasoning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClassificationMethod {
    RuleBased,
    MachineLearning,
    Default,
}
```

### 2.2 Context Capture Framework

#### Comprehensive Test Context
**File**: `src/monitoring/context.rs`
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestExecutionContext {
    pub environment: EnvironmentContext,
    pub execution: ExecutionContext,
    pub system: SystemContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentContext {
    pub rust_version: String,
    pub cargo_version: String,
    pub os_info: String,
    pub dependencies: Vec<DependencyInfo>,
    pub environment_variables: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub test_command: String,
    pub working_directory: String,
    pub execution_time: Duration,
    pub parallel_execution: bool,
    pub test_features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemContext {
    pub cpu_cores: u32,
    pub total_memory: u64,
    pub available_memory: u64,
    pub disk_space: u64,
    pub load_average: f32,
}

impl TestExecutionContext {
    pub async fn capture() -> Result<Self, ContextError> {
        // Implementation for comprehensive context capture
        // System information, environment variables, dependency versions
        // Resource utilization at time of test execution
    }
}
```

## 📋 Phase 3: Performance Tracking (Weeks 9-12)

### 3.1 Advanced Metrics Collection

#### Performance Monitoring Integration
**File**: `src/monitoring/performance.rs`
```rust
use criterion::Criterion;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub test_id: String,
    pub execution_time: Duration,
    pub memory_usage: MemoryMetrics,
    pub cpu_usage: CpuMetrics,
    pub io_metrics: IoMetrics,
    pub benchmark_results: Option<BenchmarkResults>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetrics {
    pub peak_usage_mb: u64,
    pub average_usage_mb: u64,
    pub allocations: u64,
    pub deallocations: u64,
}

pub struct PerformanceTracker {
    start_time: Instant,
    memory_tracker: MemoryTracker,
    cpu_tracker: CpuTracker,
}

impl PerformanceTracker {
    pub fn start_tracking(test_name: &str) -> Self {
        // Implementation for comprehensive performance tracking
        // Integration with existing Criterion.rs benchmarks
        // Resource utilization monitoring
    }

    pub async fn finish_tracking(self) -> PerformanceMetrics {
        // Calculate final metrics
        // Generate performance report
        // Detect performance regressions
    }
}

// Integration with Criterion.rs
pub fn setup_criterion_integration() -> Criterion {
    Criterion::default()
        .with_measurement(|measurement| {
            // Custom measurement integration
            // Send metrics to monitoring system
        })
        .with_profiler(|profiler| {
            // Custom profiler for detailed analysis
        })
}
```

### 3.2 Bottleneck Detection

#### Genetic Algorithm-Based Analysis
**File**: `src/monitoring/bottleneck_detection.rs`
```rust
#[derive(Debug, Clone)]
pub struct BottleneckDetector {
    genetic_algorithm: GeneticOptimizer,
    statistical_analyzer: StatisticalAnalyzer,
}

impl BottleneckDetector {
    pub async fn analyze_performance_patterns(
        &self, 
        metrics: &[PerformanceMetrics]
    ) -> Vec<BottleneckReport> {
        // Genetic algorithm-based pattern recognition
        // Statistical regression detection
        // Resource correlation analysis
    }

    pub async fn detect_regressions(
        &self,
        current_metrics: &PerformanceMetrics,
        historical_data: &[PerformanceMetrics]
    ) -> Option<RegressionAlert> {
        // Mann-Kendall trend tests
        // Change point detection
        // 95% confidence interval analysis
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BottleneckReport {
    pub bottleneck_type: BottleneckType,
    pub severity: Severity,
    pub affected_tests: Vec<String>,
    pub optimization_suggestions: Vec<OptimizationSuggestion>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BottleneckType {
    CpuBound,
    MemoryBound,
    IoBound,
    NetworkBound,
    ConcurrencyLimited,
}
```

## 📋 Phase 4: Advanced Analytics (Weeks 13-16)

### 4.1 Automated Reporting System

#### Comprehensive Report Generation
**File**: `src/monitoring/reporting.rs`
```rust
use tera::{Tera, Context};
use chrono::{DateTime, Utc, Duration as ChronoDuration};

pub struct ReportGenerator {
    template_engine: Tera,
    database: MonitoringDatabase,
}

impl ReportGenerator {
    pub async fn generate_daily_health_report(&self, date: DateTime<Utc>) -> Result<HealthReport, ReportError> {
        let metrics = self.database.get_daily_metrics(date).await?;
        let failures = self.database.get_daily_failures(date).await?;
        let performance = self.database.get_daily_performance(date).await?;

        let report = HealthReport {
            date,
            summary: self.generate_summary(&metrics, &failures, &performance),
            test_execution_stats: self.calculate_execution_stats(&metrics),
            failure_analysis: self.analyze_failures(&failures),
            performance_trends: self.analyze_performance_trends(&performance),
            recommendations: self.generate_recommendations(&metrics, &failures, &performance),
        };

        Ok(report)
    }

    pub async fn generate_weekly_trend_report(&self, week_start: DateTime<Utc>) -> Result<TrendReport, ReportError> {
        // Implementation for comprehensive weekly analysis
        // Trend identification, pattern mining, comparative analytics
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub date: DateTime<Utc>,
    pub summary: ReportSummary,
    pub test_execution_stats: ExecutionStats,
    pub failure_analysis: FailureAnalysis,
    pub performance_trends: PerformanceTrends,
    pub recommendations: Vec<Recommendation>,
}
```

### 4.2 Alert System Integration

#### Enhanced Alerting with Existing Systems
**File**: `src/monitoring/alerting.rs`
```rust
use crate::resilience::health::{HealthMonitor, AlertSeverity};

pub struct TestMonitoringAlerts {
    health_monitor: HealthMonitor,
    alert_thresholds: AlertThresholds,
}

impl TestMonitoringAlerts {
    pub async fn evaluate_test_metrics(&self, metrics: &TestMetrics) -> Vec<Alert> {
        let mut alerts = Vec::new();

        // Critical failure rate threshold
        if self.calculate_failure_rate().await > self.alert_thresholds.critical_failure_rate {
            alerts.push(Alert {
                severity: AlertSeverity::Critical,
                component: "test_execution".to_string(),
                message: "Critical test failure rate exceeded".to_string(),
                metrics: Some(metrics.clone()),
            });
        }

        // Performance regression detection
        if let Some(regression) = self.detect_performance_regression(metrics).await {
            alerts.push(Alert {
                severity: AlertSeverity::Warning,
                component: "test_performance".to_string(),
                message: format!("Performance regression detected: {}", regression.description),
                metrics: Some(metrics.clone()),
            });
        }

        alerts
    }

    pub async fn send_alerts(&self, alerts: Vec<Alert>) -> Result<(), AlertError> {
        for alert in alerts {
            // Integration with existing health monitor
            self.health_monitor.add_alert(
                &alert.component,
                &alert.message,
                alert.severity
            ).await;

            // Additional notification channels (Slack, email, etc.)
            self.send_external_notification(&alert).await?;
        }
        Ok(())
    }
}
```

## 🔧 Integration Requirements

### 1. Cargo.toml Dependencies
```toml
[dependencies]
# Existing dependencies...

# Monitoring system additions
axum = { version = "0.7", features = ["ws", "macros"] }
tokio-tungstenite = "0.20"
questdb = "0.3"  # QuestDB client
tera = "1.19"    # Template engine for reports
regex = "1.10"   # Pattern matching for failure classification
criterion = { version = "0.5", features = ["html_reports"] }
sysinfo = "0.30" # System information collection
```

### 2. Configuration Integration
**File**: `src/config/monitoring.rs`
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub database_url: String,
    pub websocket_port: u16,
    pub github_webhook_secret: String,
    pub alert_thresholds: AlertThresholds,
    pub performance_tracking: PerformanceConfig,
    pub classification: ClassificationConfig,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            database_url: "postgresql://localhost:5432/uveddi_monitoring".to_string(),
            websocket_port: 8081,
            github_webhook_secret: std::env::var("GITHUB_WEBHOOK_SECRET").unwrap_or_default(),
            alert_thresholds: AlertThresholds::default(),
            performance_tracking: PerformanceConfig::default(),
            classification: ClassificationConfig::default(),
        }
    }
}
```

### 3. Main Application Integration
**File**: `src/main.rs` (extend existing)
```rust
use crate::monitoring::MonitoringDashboard;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Existing initialization...

    // Initialize monitoring system
    let monitoring_config = MonitoringConfig::from_env()?;
    let monitoring_dashboard = MonitoringDashboard::new(monitoring_config).await?;
    
    // Start monitoring services
    monitoring_dashboard.start().await?;

    // Existing application logic...
    Ok(())
}
```

## 📊 Success Criteria & Validation

### Performance Targets
- **Response Time**: Sub-second dashboard updates, 100ms API responses
- **Throughput**: 4.3M+ metrics/second ingestion, 50k concurrent WebSocket connections
- **Classification**: 85%+ automated failure classification accuracy
- **Reliability**: 99.9% uptime, sub-minute recovery from failures

### User Experience Goals
- **Automation**: 80% reduction in manual test analysis
- **MTTR**: 50% improvement in mean time to resolution
- **Visibility**: Real-time test execution monitoring across all languages
- **Insights**: Predictive performance regression detection

### Integration Validation
- **CI/CD**: Seamless GitHub Actions integration without workflow changes
- **Existing Systems**: Full integration with ErrorMetrics and ComponentHealth
- **Scalability**: Support for growing test suites and user base
- **Maintenance**: Automated backup, monitoring, and upgrade procedures

## 🚀 Implementation Instructions

### Phase 1 Deliverables (Weeks 1-4)
1. **Backend Infrastructure**: Axum services, WebSocket communication, database integration
2. **GitHub Integration**: Webhook endpoints, JUnit XML parsing, automated metrics collection
3. **Basic Dashboard**: React components, real-time updates, test execution visibility
4. **Alert Integration**: Basic threshold-based alerts with existing health monitoring

### Phase 2 Deliverables (Weeks 5-8)
1. **Failure Classification**: Rule-based patterns, ML model training, context capture
2. **Enhanced Analytics**: Failure trend analysis, categorization reporting
3. **Improved Dashboard**: Failure analysis panels, categorization visualization
4. **Advanced Alerts**: Category-specific alerts, intelligent threshold adjustment

### Phase 3 Deliverables (Weeks 9-12)
1. **Performance Tracking**: Comprehensive metrics, bottleneck detection, regression analysis
2. **Optimization Engine**: Genetic algorithm analysis, performance recommendations
3. **Advanced Dashboard**: Performance trends, optimization suggestions, predictive analytics
4. **Capacity Planning**: Resource utilization analysis, scaling recommendations

### Phase 4 Deliverables (Weeks 13-16)
1. **Automated Reporting**: Daily health reports, weekly trend analysis, stakeholder-specific views
2. **Historical Analytics**: Long-term trend analysis, pattern mining, comparative analytics
3. **Complete Integration**: Full integration with all Uveddi systems and workflows
4. **Production Readiness**: Comprehensive testing, documentation, deployment procedures

---

## 🎯 Implementation Priority

**Start with Phase 1 MVP to deliver immediate value, then iterate through subsequent phases based on user feedback and operational needs. Focus on seamless integration with existing Uveddi infrastructure while building toward comprehensive monitoring capabilities.**

**The system should enhance rather than replace existing monitoring, providing specialized test infrastructure visibility that complements current error handling and health monitoring systems.**