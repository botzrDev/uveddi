# UV-90: Error Handling & Observability - Complete Implementation Guide

**Issue ID**: UV-90  
**Epic**: UV-1 (Image Rendering Service)  
**Date Created**: July 9, 2025  
**Status**: Implementation Ready  
**Priority**: P1 - Critical for Production  

## Table of Contents
1. [Issue Overview](#issue-overview)
2. [Research Phase Summary](#research-phase-summary)
3. [Research Findings](#research-findings)
4. [Implementation Plan](#implementation-plan)
5. [Technical Architecture](#technical-architecture)
6. [Security Requirements](#security-requirements)
7. [Testing Strategy](#testing-strategy)
8. [Timeline & Milestones](#timeline--milestones)

---

## Issue Overview

### Original Jira Issue Description

**🔧 PRODUCTION-GRADE: Error Handling & Observability**

Priority: P2 - Production Readiness  
Status: Basic error types defined

Description: Comprehensive error handling and fallback mechanisms for the rendering service

**CONSOLIDATED REQUIREMENTS** (from UV-9, UV-15, UV-90):

#### Error Handling:
- [ ] Graceful handling of rendering failures
- [ ] Fallback to Mermaid-only mode when rendering service unavailable
- [ ] Proper error messages for different failure scenarios
- [ ] Retry logic with exponential backoff
- [ ] Structured logging for all operations

#### Observability & Monitoring:
- [ ] Service health monitoring and alerts
- [ ] Metrics collection for performance monitoring
- [ ] Graceful degradation strategies
- [ ] Error recovery and retry mechanisms

#### Error Scenarios to Handle:
- Puppeteer service unavailable
- Rendering timeout
- Memory/resource exhaustion
- Invalid diagram data
- Network connectivity issues

#### Fallback Strategy:
- Detect service availability
- Gracefully degrade to text/Mermaid output
- User notification of degraded functionality
- Automatic recovery when service restored

#### Acceptance Criteria:
- [ ] All errors are properly categorized and logged
- [ ] Metrics are exposed for monitoring
- [ ] System gracefully handles partial failures
- [ ] Recovery mechanisms restore service automatically
- [ ] Fallback mechanisms work seamlessly

### Subtasks Overview

| Key | Summary | Type | Status |
|-----|---------|------|--------|
| UV-168 | Implement Retry Logic with Exponential Backoff | Subtask | To Do |
| UV-169 | Create Proper Error Messages for Failure Scenarios | Subtask | To Do |
| UV-170 | Set Up Structured Logging for All Operations | Subtask | To Do |
| UV-171 | Develop Fallback to Mermaid-Only Mode | Subtask | To Do |
| UV-172 | Implement Graceful Handling of Rendering Failures | Subtask | To Do |
| UV-173 | Establish Service Health Monitoring and Alerts | Subtask | To Do |
| UV-174 | Implement Metrics Collection for Performance | Subtask | To Do |
| UV-175 | Design Graceful Degradation Strategies | Subtask | To Do |
| UV-176 | Develop Error Recovery and Retry Mechanisms | Subtask | To Do |
| UV-177 | Implement Fallback Strategy for Service Unavailability | Subtask | To Do |

---

## Research Phase Summary

### Research Approach

To properly implement UV-90, comprehensive research was conducted in four critical areas:

1. **Error Handling Patterns** - Rust-specific error handling for microservice communication
2. **Observability & Metrics** - Modern observability stack integration
3. **Resilience Patterns** - Circuit breakers, bulkheads, and graceful degradation
4. **Service Integration** - Best practices for Rust ↔ Node.js service communication

### Research Documentation Location

All research documents are stored in: `docs/06-research/Specialized/UV-90/`

- `01_error_handling_patterns.md` - Custom error types and propagation patterns
- `02_observability_metrics.md` - Comprehensive observability architecture
- `03_resilience_patterns.md` - Multi-layered resilience framework
- `04_service_integration.md` - Production-ready service communication

### Research Status: COMPLETE ✅

All four research areas have been thoroughly investigated with detailed findings and specific technology recommendations.

---

## Research Findings

### 1. Error Handling Patterns Research Results

#### Key Findings:
- **Architecture**: Facade-first instrumentation with custom error type hierarchy
- **Technology Stack**: `thiserror` for custom errors, `anyhow` for error chains
- **Security Mandate**: Strict separation of public/private error details
- **Integration**: Extends existing `UveddiError` with rendering-specific variants

#### Recommended Error Type Structure:
```rust
#[derive(thiserror::Error, Debug)]
pub enum RenderingServiceError {
    #[error("Rendering service unavailable")]
    ServiceUnavailable,
    #[error("Connection timeout after {timeout}ms")]
    ConnectionTimeout { timeout: u64 },
    #[error("Request timeout after {timeout}ms")]
    RequestTimeout { timeout: u64 },
    #[error("Authentication failed")]
    AuthenticationFailure,
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Invalid response from service")]
    InvalidResponse,
    #[error("Memory limit exceeded")]
    MemoryExhaustion,
    #[error("CPU timeout exceeded")]
    CpuTimeout,
    #[error("Disk space insufficient")]
    DiskSpaceExhaustion,
    #[error("Network bandwidth limit exceeded")]
    NetworkBandwidthLimit,
    #[error("Invalid Mermaid syntax: {details}")]
    InvalidMermaidSyntax { details: String },
    #[error("Diagram too complex: {reason}")]
    DiagramComplexityExceeded { reason: String },
    #[error("Input size limit exceeded: {size} bytes")]
    InputTooLarge { size: usize },
    #[error("Security validation failed")]
    SecurityValidationFailed,
}
```

#### Security Mandates:
- **MANDATE**: `ProblemDetails` object in HTTP responses must never contain information from error's `source()` chain
- **MANDATE**: Full `Debug` representation should be logged internally but never sent in API responses
- **MANDATE**: Strict size limits on all incoming payloads with `UveddiError::InputTooLarge`
- **MANDATE**: Structural and semantic validation of all inputs with specific `UserInputError` variants

### 2. Observability & Metrics Research Results

#### Key Findings:
- **Architecture**: "PLG Stack" (Prometheus + Loki + Grafana) with OpenTelemetry
- **Technology Stack**: `tracing` + `metrics` crates with OTLP export
- **Central Hub**: OpenTelemetry Collector as central telemetry processing
- **Performance Target**: <1% overhead with efficient sampling strategies

#### Recommended Technology Stack:
```toml
[dependencies]
# Metrics
prometheus = "0.13"
metrics = "0.21"
metrics-exporter-prometheus = "0.12"

# Logging & Tracing
tracing = "0.1"
tracing-subscriber = "0.3"
tracing-opentelemetry = "0.21"
opentelemetry = "0.20"
opentelemetry-jaeger = "0.19"

# Health Checks
tokio-metrics = "0.3"
sysinfo = "0.29"
```

#### Core Metrics Structure:
```rust
struct RenderingMetrics {
    requests_total: Counter,
    request_duration: Histogram,
    active_requests: Gauge,
    error_rate: Counter,
    service_availability: Gauge,
    memory_usage: Gauge,
    render_queue_depth: Gauge,
    circuit_breaker_state: Gauge,
}
```

#### Observability Architecture:
1. **Instrumentation**: Rust service uses `tracing` and `metrics` facade crates
2. **Telemetry Emission**: OTLP exporter sends data to OpenTelemetry Collector
3. **Cross-Service Tracing**: W3C TraceContext headers for Rust ↔ Node.js tracing
4. **Collection & Processing**: OpenTelemetry Collector performs sampling, conversion, enrichment
5. **Storage & Visualization**: Prometheus (metrics), Loki (logs), Jaeger/Tempo (traces), Grafana (dashboards)

### 3. Resilience Patterns Research Results

#### Key Findings:
- **Architecture**: Multi-layered defense with Circuit Breakers + Bulkheads + Graceful Degradation
- **Technology Stack**: `tower` middleware with custom circuit breaker implementation
- **Fallback Strategy**: 4-tier degradation (Full → Mermaid → Templates → Text)
- **Failure Mapping**: Direct pattern mapping to specific `ServiceFailureMode` scenarios

#### Service Failure Modes:
```rust
enum ServiceFailureMode {
    CompleteOutage,           // Service completely unavailable
    PartialDegradation,       // Slow responses, timeouts
    ResourceExhaustion,       // Memory/CPU limits exceeded
    CascadingFailure,         // Dependency chain failures
    NetworkPartition,         // Connectivity issues
    DataCorruption,           // Invalid responses
}
```

#### Circuit Breaker Implementation:
```rust
pub struct CircuitBreaker {
    state: CircuitState,
    failure_threshold: u32,
    recovery_timeout: Duration,
    success_threshold: u32,
    failure_count: AtomicU32,
    success_count: AtomicU32,
    last_failure_time: AtomicU64,
}

pub enum CircuitState {
    Closed,    // Normal operation
    Open,      // Failures detected, reject requests
    HalfOpen,  // Testing recovery
}
```

#### Graceful Degradation Hierarchy:
1. **Primary**: Full rendering service with image generation
2. **Secondary**: Mermaid-only mode without image rendering
3. **Tertiary**: Static diagram templates
4. **Emergency**: Plain text output with structure information

#### Recommended Technology Stack:
```toml
[dependencies]
# Circuit Breakers
failsafe = "1.0"
tower = "0.4"

# Retry Logic
backoff = "0.4"
tokio-retry = "0.3"

# Rate Limiting
governor = "0.5"

# Timeout Management
tokio = { version = "1.0", features = ["time"] }

# Bulkhead Isolation
rayon = "1.7"
tokio-util = "0.7"
```

### 4. Service Integration Research Results

#### Key Findings:
- **Architecture**: Hybrid communication (gRPC for sync + Message Queues for async)
- **Technology Stack**: Protocol Buffers + Consul service discovery + mTLS security
- **Performance Target**: <99ms P95 latency with request-level load balancing
- **Security**: Zero-trust with mTLS + JWT authorization

#### Communication Protocol Recommendations:
- **Primary (Synchronous)**: gRPC with Protocol Buffers for immediate rendering and streaming
- **Secondary (Asynchronous)**: RabbitMQ for job submission, batch processing, and decoupled operations
- **Service Discovery**: Consul for dynamic Node.js service endpoint discovery
- **Load Balancing**: Request-level (L7) via service mesh (Linkerd) or client-side (`tower`)

#### Security Framework:
- **Transport Security**: Mutual TLS (mTLS) for encrypted communication and service authentication
- **Request Authorization**: JSON Web Tokens (JWT) for user context and permissions propagation
- **Zero-Trust Model**: All service-to-service communication requires authentication and authorization

#### Recommended Technology Stack:
```toml
[dependencies]
# HTTP Clients
reqwest = { version = "0.11", features = ["json", "stream"] }
hyper = { version = "0.14", features = ["full"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
prost = "0.11"     # Protocol Buffers

# Service Discovery
consul = "0.3"

# Load Balancing
tower = { version = "0.4", features = ["load-shed", "limit"] }
tower-balance = "0.3"
```

---

## Implementation Plan

### Implementation Strategy

#### Core Principles:
1. **Foundation First**: Implement core error handling and logging infrastructure before advanced patterns
2. **Incremental Integration**: Each phase builds on the previous while remaining independently deployable
3. **Validation Driven**: Each component includes comprehensive testing and monitoring
4. **Production Ready**: Every implementation must meet security, performance, and reliability mandates

### Phase 1: Foundation Infrastructure (Week 1-2)

#### Goals:
- Establish core error handling architecture
- Implement structured logging and basic metrics
- Create retry logic with exponential backoff
- Set up initial monitoring capabilities

#### Subtasks Implementation Order:

##### **UV-168: Implement Retry Logic with Exponential Backoff** (Day 1-2)
```rust
pub struct RetryConfig {
    max_attempts: u32,
    base_delay: Duration,
    max_delay: Duration,
    backoff_multiplier: f64,
    jitter_factor: f64,
    retry_on: Vec<ErrorType>,
}
```

**Deliverables:**
- [ ] `src/resilience/retry.rs` - Core retry logic implementation
- [ ] Integration with existing HTTP client patterns
- [ ] Exponential backoff with jitter to prevent thundering herd
- [ ] Configurable retry policies for different error types
- [ ] Unit tests with failure scenario simulation

##### **UV-169: Create Proper Error Messages for Failure Scenarios** (Day 3-4)
```rust
#[derive(thiserror::Error, Debug)]
pub enum RenderingServiceError {
    #[error("Rendering service unavailable")]
    ServiceUnavailable,
    #[error("Request timeout after {timeout}ms")]
    Timeout { timeout: u64 },
    // ...additional variants
}
```

**Deliverables:**
- [ ] `src/error/rendering.rs` - Extended error types for rendering service
- [ ] Integration with existing `UveddiError` in `src/error.rs`
- [ ] User-friendly error messages with technical details separated
- [ ] Error serialization for cross-service communication
- [ ] Security audit for information disclosure prevention

##### **UV-170: Set Up Structured Logging for All Operations** (Day 5-6)
```rust
use tracing::{info, error, instrument};

#[instrument(skip(request), fields(request_id = %request.id))]
async fn render_diagram(request: RenderRequest) -> Result<RenderResponse> {
    info!("Starting diagram rendering");
    // ...implementation
}
```

**Deliverables:**
- [ ] `src/observability/logging.rs` - Structured logging configuration
- [ ] `tracing` integration with correlation IDs
- [ ] JSON log format for aggregation
- [ ] Log level configuration and filtering
- [ ] Security redaction for sensitive data

#### Phase 1 Success Criteria:
- [ ] All rendering service errors are properly categorized and logged
- [ ] Retry logic handles transient failures automatically
- [ ] Structured logs provide clear operational visibility
- [ ] Error messages are user-friendly while preserving technical context
- [ ] Performance overhead <2% for logging and error handling

### Phase 2: Resilience Patterns (Week 3-4)

#### Goals:
- Implement circuit breaker for service communication
- Add graceful degradation with fallback modes
- Establish error recovery mechanisms
- Create comprehensive error handling for rendering failures

#### Subtasks Implementation Order:

##### **UV-171: Develop Fallback to Mermaid-Only Mode** (Day 7-8)
```rust
pub enum RenderingMode {
    Full,        // Image + Mermaid
    MermaidOnly, // Mermaid syntax only
    Template,    // Static templates
    PlainText,   // Structured text
}
```

**Deliverables:**
- [ ] `src/rendering/fallback.rs` - Fallback mode implementation
- [ ] 4-tier degradation hierarchy (Full → Mermaid → Template → Text)
- [ ] Automatic mode detection based on service health
- [ ] User notification of degraded functionality
- [ ] Configuration for fallback thresholds

##### **UV-172: Implement Graceful Handling of Rendering Failures** (Day 9-10)
```rust
pub struct CircuitBreaker {
    state: CircuitState,
    failure_threshold: u32,
    recovery_timeout: Duration,
    metrics: CircuitBreakerMetrics,
}
```

**Deliverables:**
- [ ] `src/resilience/circuit_breaker.rs` - Circuit breaker implementation
- [ ] Integration with HTTP client for Node.js service calls
- [ ] Configurable thresholds for failure detection
- [ ] Half-open state for recovery testing
- [ ] Metrics integration for monitoring

##### **UV-176: Develop Error Recovery and Retry Mechanisms** (Day 11-12)
```rust
pub struct ErrorRecovery {
    recovery_strategies: HashMap<ErrorType, RecoveryAction>,
    state_persistence: StateStore,
    health_checker: HealthChecker,
}
```

**Deliverables:**
- [ ] `src/resilience/recovery.rs` - Error recovery orchestration
- [ ] Integration between retry logic and circuit breaker
- [ ] State persistence for recovery across restarts
- [ ] Health check validation during recovery
- [ ] Automated recovery workflows

#### Phase 2 Success Criteria:
- [ ] Circuit breaker prevents cascading failures
- [ ] Graceful degradation maintains user experience during outages
- [ ] Error recovery restores service automatically when possible
- [ ] Fallback modes provide meaningful alternatives to full functionality
- [ ] Resilience patterns integrate seamlessly with error handling

### Phase 3: Advanced Observability (Week 5-6)

#### Goals:
- Implement comprehensive metrics collection
- Establish service health monitoring
- Create production-ready alerting
- Complete graceful degradation strategies

#### Subtasks Implementation Order:

##### **UV-173: Establish Service Health Monitoring and Alerts** (Day 13-14)
```rust
pub struct HealthMonitor {
    checks: Vec<HealthCheck>,
    alerting: AlertManager,
    dashboard: MetricsDashboard,
}
```

**Deliverables:**
- [ ] `src/observability/health.rs` - Health monitoring system
- [ ] Deep and shallow health check endpoints
- [ ] Integration with Prometheus for metrics collection
- [ ] AlertManager configuration for critical alerts
- [ ] Grafana dashboard for service health visualization

##### **UV-174: Implement Metrics Collection for Performance** (Day 15-16)
```rust
pub struct RenderingMetrics {
    requests_total: Counter,
    request_duration: Histogram,
    error_rate: Counter,
    circuit_breaker_state: Gauge,
    active_requests: Gauge,
}
```

**Deliverables:**
- [ ] `src/observability/metrics.rs` - Comprehensive metrics collection
- [ ] Prometheus metrics endpoint (`/metrics`)
- [ ] Business KPIs and operational metrics
- [ ] Performance tracking by diagram type
- [ ] SLI/SLO monitoring integration

##### **UV-175: Design Graceful Degradation Strategies** (Day 17-18)
```rust
pub struct DegradationController {
    mode: RenderingMode,
    triggers: Vec<DegradationTrigger>,
    recovery_conditions: Vec<RecoveryCondition>,
}
```

**Deliverables:**
- [ ] `src/resilience/degradation.rs` - Degradation strategy controller
- [ ] Feature flag integration for gradual degradation
- [ ] Load-based degradation triggers
- [ ] User segment-based degradation policies
- [ ] Recovery automation when conditions improve

##### **UV-177: Implement Fallback Strategy for Service Unavailability** (Day 19-20)
```rust
pub struct FallbackOrchestrator {
    service_discovery: ServiceRegistry,
    fallback_chain: Vec<FallbackProvider>,
    state_manager: FallbackStateManager,
}
```

**Deliverables:**
- [ ] `src/resilience/fallback_orchestrator.rs` - Complete fallback coordination
- [ ] Integration with service discovery for dynamic endpoints
- [ ] Coordinated fallback across multiple service instances
- [ ] State synchronization for consistent user experience
- [ ] End-to-end fallback testing framework

#### Phase 3 Success Criteria:
- [ ] Real-time visibility into all service operations
- [ ] Automated alerting for critical failure scenarios
- [ ] Graceful degradation responds to various failure conditions
- [ ] Fallback strategies provide seamless user experience
- [ ] Production monitoring meets <99ms P95 latency targets

---

## Technical Architecture

### Error Handling Architecture

#### Core Error Type Hierarchy:
```rust
// src/error/rendering.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RenderingServiceError {
    #[error("Rendering service unavailable")]
    ServiceUnavailable,
    
    #[error("Connection timeout after {timeout}ms")]
    ConnectionTimeout { timeout: u64 },
    
    #[error("Request timeout after {timeout}ms")]
    RequestTimeout { timeout: u64 },
    
    #[error("Authentication failed")]
    AuthenticationFailure,
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Invalid response from service")]
    InvalidResponse,
    
    // Resource exhaustion scenarios
    #[error("Memory limit exceeded")]
    MemoryExhaustion,
    
    #[error("CPU timeout exceeded")]
    CpuTimeout,
    
    #[error("Disk space insufficient")]
    DiskSpaceExhaustion,
    
    #[error("Network bandwidth limit exceeded")]
    NetworkBandwidthLimit,
    
    // Data validation failures
    #[error("Invalid Mermaid syntax: {details}")]
    InvalidMermaidSyntax { details: String },
    
    #[error("Diagram too complex: {reason}")]
    DiagramComplexityExceeded { reason: String },
    
    #[error("Input size limit exceeded: {size} bytes")]
    InputTooLarge { size: usize },
    
    #[error("Security validation failed")]
    SecurityValidationFailed,
}

impl From<RenderingServiceError> for UveddiError {
    fn from(err: RenderingServiceError) -> Self {
        UveddiError::RenderingService(err)
    }
}
```

#### Error Context Propagation:
```rust
// src/error/context.rs
use tracing::Span;

#[derive(Debug)]
pub struct ErrorContext {
    pub request_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub span_trace: Option<SpanTrace>,
    pub service_endpoint: Option<String>,
    pub retry_attempt: Option<u32>,
}

impl ErrorContext {
    pub fn from_current_span() -> Self {
        let span = Span::current();
        Self {
            request_id: span.field("request_id").unwrap_or_default(),
            timestamp: chrono::Utc::now(),
            span_trace: SpanTrace::capture(),
            service_endpoint: None,
            retry_attempt: None,
        }
    }
}
```

### Observability Architecture

#### Metrics Collection Structure:
```rust
// src/observability/metrics.rs
use metrics::{Counter, Histogram, Gauge};
use prometheus::{register_counter, register_histogram, register_gauge};

pub struct RenderingMetrics {
    // Request metrics
    pub requests_total: Counter,
    pub request_duration: Histogram,
    pub active_requests: Gauge,
    
    // Error metrics
    pub error_rate: Counter,
    pub error_rate_by_type: HashMap<String, Counter>,
    
    // Service health metrics
    pub service_availability: Gauge,
    pub circuit_breaker_state: Gauge,
    
    // Resource metrics
    pub memory_usage: Gauge,
    pub cpu_usage: Gauge,
    pub render_queue_depth: Gauge,
    
    // Business metrics
    pub diagrams_rendered_by_type: HashMap<String, Counter>,
    pub rendering_success_rate: Gauge,
}

impl RenderingMetrics {
    pub fn new() -> Self {
        Self {
            requests_total: register_counter!("rendering_requests_total", "Total rendering requests"),
            request_duration: register_histogram!("rendering_request_duration_seconds", "Request duration"),
            active_requests: register_gauge!("rendering_active_requests", "Active rendering requests"),
            error_rate: register_counter!("rendering_errors_total", "Total rendering errors"),
            // ... additional metrics
        }
    }
    
    pub fn record_request_start(&self) {
        self.requests_total.increment(1);
        self.active_requests.increment(1.0);
    }
    
    pub fn record_request_complete(&self, duration: Duration, success: bool) {
        self.active_requests.decrement(1.0);
        self.request_duration.record(duration.as_secs_f64());
        
        if !success {
            self.error_rate.increment(1);
        }
    }
}
```

#### Structured Logging Configuration:
```rust
// src/observability/logging.rs
use tracing::{Level, Subscriber};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use tracing_opentelemetry::OpenTelemetryLayer;

pub fn init_logging() -> Result<(), Box<dyn std::error::Error>> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let formatting_layer = fmt::layer()
        .json()
        .with_current_span(false)
        .with_span_list(true);

    let opentelemetry_layer = OpenTelemetryLayer::new(
        opentelemetry_jaeger::new_pipeline()
            .with_service_name("uveddi-rendering-service")
            .install_batch(opentelemetry::runtime::Tokio)?
    );

    tracing_subscriber::registry()
        .with(env_filter)
        .with(formatting_layer)
        .with(opentelemetry_layer)
        .init();

    Ok(())
}
```

### Resilience Architecture

#### Circuit Breaker Implementation:
```rust
// src/resilience/circuit_breaker.rs
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    inner: Arc<CircuitBreakerInner>,
}

struct CircuitBreakerInner {
    state: AtomicU32, // 0 = Closed, 1 = Open, 2 = HalfOpen
    failure_count: AtomicU32,
    success_count: AtomicU32,
    last_failure_time: AtomicU64,
    config: CircuitBreakerConfig,
    metrics: CircuitBreakerMetrics,
}

#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub success_threshold: u32,
    pub recovery_timeout: Duration,
    pub half_open_max_calls: u32,
}

#[derive(Debug)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            inner: Arc::new(CircuitBreakerInner {
                state: AtomicU32::new(0), // Closed
                failure_count: AtomicU32::new(0),
                success_count: AtomicU32::new(0),
                last_failure_time: AtomicU64::new(0),
                config,
                metrics: CircuitBreakerMetrics::new(),
            }),
        }
    }
    
    pub async fn call<F, T, E>(&self, f: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: Future<Output = Result<T, E>>,
    {
        match self.state() {
            CircuitState::Open => {
                if self.should_attempt_reset() {
                    self.transition_to_half_open();
                } else {
                    return Err(CircuitBreakerError::CircuitOpen);
                }
            }
            CircuitState::HalfOpen => {
                if self.inner.success_count.load(Ordering::Relaxed) >= self.inner.config.half_open_max_calls {
                    return Err(CircuitBreakerError::CircuitOpen);
                }
            }
            CircuitState::Closed => {}
        }
        
        match f.await {
            Ok(result) => {
                self.record_success();
                Ok(result)
            }
            Err(e) => {
                self.record_failure();
                Err(CircuitBreakerError::CallFailed(e))
            }
        }
    }
    
    fn state(&self) -> CircuitState {
        match self.inner.state.load(Ordering::Relaxed) {
            0 => CircuitState::Closed,
            1 => CircuitState::Open,
            2 => CircuitState::HalfOpen,
            _ => unreachable!(),
        }
    }
    
    fn record_success(&self) {
        self.inner.success_count.fetch_add(1, Ordering::Relaxed);
        
        match self.state() {
            CircuitState::HalfOpen => {
                if self.inner.success_count.load(Ordering::Relaxed) >= self.inner.config.success_threshold {
                    self.transition_to_closed();
                }
            }
            _ => {
                self.inner.failure_count.store(0, Ordering::Relaxed);
            }
        }
        
        self.inner.metrics.success_count.increment(1);
    }
    
    fn record_failure(&self) {
        let failure_count = self.inner.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
        self.inner.last_failure_time.store(
            Instant::now().elapsed().as_secs(),
            Ordering::Relaxed,
        );
        
        if failure_count >= self.inner.config.failure_threshold {
            self.transition_to_open();
        }
        
        self.inner.metrics.failure_count.increment(1);
    }
    
    fn transition_to_open(&self) {
        self.inner.state.store(1, Ordering::Relaxed);
        self.inner.metrics.state_changes.increment(1);
        tracing::warn!("Circuit breaker opened due to failure threshold");
    }
    
    fn transition_to_half_open(&self) {
        self.inner.state.store(2, Ordering::Relaxed);
        self.inner.success_count.store(0, Ordering::Relaxed);
        self.inner.metrics.state_changes.increment(1);
        tracing::info!("Circuit breaker half-open, testing recovery");
    }
    
    fn transition_to_closed(&self) {
        self.inner.state.store(0, Ordering::Relaxed);
        self.inner.failure_count.store(0, Ordering::Relaxed);
        self.inner.success_count.store(0, Ordering::Relaxed);
        self.inner.metrics.state_changes.increment(1);
        tracing::info!("Circuit breaker closed, service recovered");
    }
    
    fn should_attempt_reset(&self) -> bool {
        let last_failure = self.inner.last_failure_time.load(Ordering::Relaxed);
        let now = Instant::now().elapsed().as_secs();
        now - last_failure >= self.inner.config.recovery_timeout.as_secs()
    }
}

#[derive(Debug)]
pub enum CircuitBreakerError<E> {
    CircuitOpen,
    CallFailed(E),
}
```

#### Retry Logic with Exponential Backoff:
```rust
// src/resilience/retry.rs
use backoff::{ExponentialBackoff, ExponentialBackoffBuilder};
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
    pub jitter_factor: f64,
    pub retry_on: Vec<ErrorType>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
            retry_on: vec![
                ErrorType::ConnectionTimeout,
                ErrorType::RequestTimeout,
                ErrorType::ServiceUnavailable,
                ErrorType::NetworkPartition,
            ],
        }
    }
}

pub struct RetryClient {
    config: RetryConfig,
    metrics: RetryMetrics,
}

impl RetryClient {
    pub fn new(config: RetryConfig) -> Self {
        Self {
            config,
            metrics: RetryMetrics::new(),
        }
    }
    
    pub async fn execute_with_retry<F, T, E>(&self, mut operation: F) -> Result<T, E>
    where
        F: FnMut() -> Pin<Box<dyn Future<Output = Result<T, E>> + Send>>,
        E: std::error::Error + 'static,
    {
        let mut backoff = ExponentialBackoffBuilder::new()
            .with_initial_interval(self.config.base_delay)
            .with_max_interval(self.config.max_delay)
            .with_multiplier(self.config.backoff_multiplier)
            .with_max_elapsed_time(None)
            .build();
        
        let mut attempt = 1;
        
        loop {
            let start_time = Instant::now();
            let result = operation().await;
            let duration = start_time.elapsed();
            
            match result {
                Ok(value) => {
                    if attempt > 1 {
                        self.metrics.retry_success.increment(1);
                        tracing::info!(
                            attempt = attempt,
                            duration_ms = duration.as_millis(),
                            "Operation succeeded after retry"
                        );
                    }
                    return Ok(value);
                }
                Err(err) => {
                    if attempt >= self.config.max_attempts || !self.should_retry(&err) {
                        self.metrics.retry_exhausted.increment(1);
                        tracing::error!(
                            attempt = attempt,
                            error = %err,
                            "Operation failed after all retry attempts"
                        );
                        return Err(err);
                    }
                    
                    if let Some(delay) = backoff.next_backoff() {
                        let jittered_delay = self.add_jitter(delay);
                        self.metrics.retry_attempts.increment(1);
                        
                        tracing::warn!(
                            attempt = attempt,
                            delay_ms = jittered_delay.as_millis(),
                            error = %err,
                            "Operation failed, retrying"
                        );
                        
                        sleep(jittered_delay).await;
                        attempt += 1;
                    } else {
                        return Err(err);
                    }
                }
            }
        }
    }
    
    fn should_retry<E: std::error::Error>(&self, error: &E) -> bool {
        // Implement error type checking based on config.retry_on
        // This would need to be customized based on your error types
        true // Simplified for example
    }
    
    fn add_jitter(&self, delay: Duration) -> Duration {
        let jitter_ms = (delay.as_millis() as f64 * self.config.jitter_factor) as u64;
        let jitter = Duration::from_millis(jitter_ms);
        
        // Add random jitter between -jitter and +jitter
        let random_jitter = Duration::from_millis(
            (jitter.as_millis() as f64 * (rand::random::<f64>() * 2.0 - 1.0)) as u64
        );
        
        delay + random_jitter
    }
}
```

### Service Integration Architecture

#### HTTP Client with Resilience Patterns:
```rust
// src/service/client.rs
use reqwest::{Client, ClientBuilder};
use std::time::Duration;
use tower::{ServiceBuilder, ServiceExt};
use tower_http::{timeout::TimeoutLayer, retry::RetryLayer};

pub struct RenderingServiceClient {
    client: Client,
    circuit_breaker: CircuitBreaker,
    retry_client: RetryClient,
    metrics: ServiceClientMetrics,
    config: ServiceClientConfig,
}

#[derive(Debug, Clone)]
pub struct ServiceClientConfig {
    pub base_url: String,
    pub timeout: Duration,
    pub connection_timeout: Duration,
    pub pool_max_idle_per_host: usize,
    pub pool_timeout: Duration,
}

impl RenderingServiceClient {
    pub fn new(config: ServiceClientConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let client = ClientBuilder::new()
            .timeout(config.timeout)
            .connect_timeout(config.connection_timeout)
            .pool_max_idle_per_host(config.pool_max_idle_per_host)
            .pool_timeout(config.pool_timeout)
            .build()?;
        
        let circuit_breaker = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 5,
            success_threshold: 3,
            recovery_timeout: Duration::from_secs(60),
            half_open_max_calls: 3,
        });
        
        let retry_client = RetryClient::new(RetryConfig::default());
        
        Ok(Self {
            client,
            circuit_breaker,
            retry_client,
            metrics: ServiceClientMetrics::new(),
            config,
        })
    }
    
    #[tracing::instrument(skip(self, request))]
    pub async fn render_diagram(&self, request: RenderRequest) -> Result<RenderResponse, RenderingServiceError> {
        let request_id = uuid::Uuid::new_v4().to_string();
        
        self.retry_client.execute_with_retry(|| {
            Box::pin(self.circuit_breaker.call(async {
                self.execute_render_request(request.clone(), &request_id).await
            }))
        }).await
        .map_err(|e| match e {
            CircuitBreakerError::CircuitOpen => RenderingServiceError::ServiceUnavailable,
            CircuitBreakerError::CallFailed(err) => err,
        })
    }
    
    async fn execute_render_request(
        &self,
        request: RenderRequest,
        request_id: &str,
    ) -> Result<RenderResponse, RenderingServiceError> {
        let start_time = Instant::now();
        
        let response = self
            .client
            .post(&format!("{}/render", self.config.base_url))
            .header("X-Request-ID", request_id)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    RenderingServiceError::RequestTimeout {
                        timeout: self.config.timeout.as_millis() as u64,
                    }
                } else if e.is_connect() {
                    RenderingServiceError::ConnectionTimeout {
                        timeout: self.config.connection_timeout.as_millis() as u64,
                    }
                } else {
                    RenderingServiceError::ServiceUnavailable
                }
            })?;
        
        let duration = start_time.elapsed();
        self.metrics.request_duration.record(duration.as_secs_f64());
        
        match response.status() {
            reqwest::StatusCode::OK => {
                let render_response: RenderResponse = response
                    .json()
                    .await
                    .map_err(|_| RenderingServiceError::InvalidResponse)?;
                
                self.metrics.success_count.increment(1);
                Ok(render_response)
            }
            reqwest::StatusCode::BAD_REQUEST => {
                self.metrics.client_error_count.increment(1);
                Err(RenderingServiceError::InvalidMermaidSyntax {
                    details: "Invalid request format".to_string(),
                })
            }
            reqwest::StatusCode::TOO_MANY_REQUESTS => {
                self.metrics.rate_limit_count.increment(1);
                Err(RenderingServiceError::RateLimitExceeded)
            }
            reqwest::StatusCode::INTERNAL_SERVER_ERROR => {
                self.metrics.server_error_count.increment(1);
                Err(RenderingServiceError::ServiceUnavailable)
            }
            status => {
                self.metrics.unknown_error_count.increment(1);
                tracing::error!(status = %status, "Unexpected response status");
                Err(RenderingServiceError::ServiceUnavailable)
            }
        }
    }
}
```

---

## Security Requirements

### Transport Security
- **MANDATE**: Mutual TLS (mTLS) for all service-to-service communication
- **MANDATE**: Certificate rotation and management automation
- **MANDATE**: TLS 1.3 minimum version with approved cipher suites

### Authentication & Authorization
- **MANDATE**: JSON Web Token (JWT) validation for all requests
- **MANDATE**: Service account authentication with RBAC
- **MANDATE**: API key rotation every 90 days

### Data Protection
- **MANDATE**: Input validation and sanitization for all external data
- **MANDATE**: Size limits on all payloads (max 10MB for diagram data)
- **MANDATE**: No sensitive data in logs or error messages
- **MANDATE**: Audit logging for all security-relevant events

### Error Handling Security
- **MANDATE**: Error responses must not leak internal system information
- **MANDATE**: Stack traces and debug information only in internal logs
- **MANDATE**: Rate limiting on error endpoints to prevent enumeration attacks

---

## Testing Strategy

### Unit Testing
- [ ] Error type conversion and propagation
- [ ] Circuit breaker state transitions
- [ ] Retry logic with various failure scenarios
- [ ] Metrics collection accuracy
- [ ] Logging format validation

### Integration Testing
- [ ] Service-to-service communication patterns
- [ ] Error handling across service boundaries
- [ ] Circuit breaker integration with HTTP client
- [ ] Fallback mode activation and recovery
- [ ] Health check endpoint validation

### Chaos Engineering
- [ ] Service unavailability simulation
- [ ] Network partition testing
- [ ] Resource exhaustion scenarios
- [ ] Cascading failure prevention
- [ ] Recovery time validation

### Performance Testing
- [ ] Latency impact of resilience patterns (<50ms overhead)
- [ ] Observability overhead measurement (<1% target)
- [ ] Concurrent request handling
- [ ] Memory usage under load
- [ ] Circuit breaker performance impact

### Security Testing
- [ ] Error message information disclosure testing
- [ ] Input validation bypass attempts
- [ ] Authentication and authorization testing
- [ ] TLS configuration validation
- [ ] Audit log completeness verification

---

## Timeline & Milestones

### Phase 1: Foundation Infrastructure (Week 1-2)
**Target Completion**: July 23, 2025

#### Milestones:
- **Day 2**: UV-168 (Retry Logic) implementation complete
- **Day 4**: UV-169 (Error Messages) implementation complete
- **Day 6**: UV-170 (Structured Logging) implementation complete
- **Week 2 End**: Phase 1 integration testing and validation

#### Success Gates:
- [ ] All error types properly categorized and logged
- [ ] Retry logic handles transient failures automatically
- [ ] Structured logs provide operational visibility
- [ ] Performance overhead <2%

### Phase 2: Resilience Patterns (Week 3-4)
**Target Completion**: August 6, 2025

#### Milestones:
- **Day 8**: UV-171 (Fallback Mode) implementation complete
- **Day 10**: UV-172 (Circuit Breaker) implementation complete
- **Day 12**: UV-176 (Error Recovery) implementation complete
- **Week 4 End**: Resilience pattern integration and chaos testing

#### Success Gates:
- [ ] Circuit breaker prevents cascading failures
- [ ] Graceful degradation maintains user experience
- [ ] Error recovery restores service automatically
- [ ] Fallback modes provide meaningful alternatives

### Phase 3: Advanced Observability (Week 5-6)
**Target Completion**: August 20, 2025

#### Milestones:
- **Day 14**: UV-173 (Health Monitoring) implementation complete
- **Day 16**: UV-174 (Metrics Collection) implementation complete
- **Day 18**: UV-175 (Degradation Strategies) implementation complete
- **Day 20**: UV-177 (Fallback Orchestration) implementation complete
- **Week 6 End**: Production readiness assessment

#### Success Gates:
- [ ] Real-time visibility into all service operations
- [ ] Automated alerting for critical failure scenarios
- [ ] Production monitoring meets <99ms P95 latency targets
- [ ] Complete observability coverage

### Production Readiness Checklist
- [ ] All subtasks (UV-168 through UV-177) implemented and tested
- [ ] Security audit completed with no critical findings
- [ ] Performance testing validates <1% observability overhead
- [ ] Chaos engineering validates resilience patterns
- [ ] Documentation complete for operations team
- [ ] Monitoring dashboards and alerts configured
- [ ] Incident response procedures documented
- [ ] Production deployment plan approved

---

## Next Steps

### Immediate Actions (This Week)
1. **Begin Phase 1 Implementation**: Start with UV-168 (Retry Logic)
2. **Set up Development Environment**: Configure testing infrastructure
3. **Create Feature Branch**: `feature/UV-90-error-handling-observability`
4. **Initialize Project Structure**: Create directory structure for new components

### Week 1 Goals
- Complete UV-168, UV-169, UV-170 implementations
- Establish core testing framework
- Set up initial CI/CD pipeline integration
- Begin security audit planning

### Week 2 Goals
- Complete Phase 1 integration testing
- Begin Phase 2 implementation (UV-171, UV-172, UV-176)
- Establish chaos engineering test suite
- Security review of error handling implementation

This comprehensive implementation plan provides a structured, risk-managed approach to delivering production-grade error handling and observability for the Uveddi image rendering service while ensuring each component meets security, performance, and reliability requirements.
