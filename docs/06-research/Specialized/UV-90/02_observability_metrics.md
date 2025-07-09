# Observability & Metrics Research - UV-90

**Research Prompt ID**: UV-90-OBS-002  
**Status**: Pending Research  
**Priority**: P1 - Critical for Production  
**Related Jira**: UV-90, UV-170, UV-173, UV-174  
**Date Created**: July 9, 2025  

## Research Objective

Design a comprehensive observability stack for the Uveddi image rendering service, including metrics collection, structured logging, distributed tracing, and health monitoring suitable for production deployment.

## Key Research Questions

### 1. Metrics Collection Architecture
- Which Rust metrics library provides the best Prometheus integration?
- How to implement custom metrics for rendering service performance?
- What are the essential SLI/SLO metrics for image rendering services?
- How to minimize performance overhead of metrics collection?

### 2. Structured Logging Strategy
- Integration of `tracing` crate with existing logging infrastructure
- Log correlation across Rust and Node.js service boundaries
- Structured log format standardization (JSON, structured text)
- Log aggregation and search requirements (ELK, Loki, etc.)

### 3. Distributed Tracing Implementation
- Request tracing across microservice boundaries
- Trace context propagation between Rust and Node.js
- Integration with existing tracing infrastructure
- Performance impact of distributed tracing

### 4. Health Monitoring & Alerting
- Health check endpoint design patterns
- Service dependency health monitoring
- Alerting thresholds and escalation policies
- Integration with monitoring platforms (Prometheus, Grafana, etc.)

## Specific Observability Requirements

### Application Metrics
```rust
// Research: Implement these metric types
struct RenderingMetrics {
    requests_total: Counter,
    request_duration: Histogram,
    active_requests: Gauge,
    error_rate: Counter,
    service_availability: Gauge,
    memory_usage: Gauge,
    render_queue_depth: Gauge,
}
```

### Business Metrics
- Diagram rendering success/failure rates
- Average rendering time by diagram type
- Service uptime and availability metrics
- Resource utilization trends
- User experience metrics (response times)

### Operational Metrics
- Service startup/shutdown events
- Configuration changes tracking
- Deployment success metrics
- Error budget consumption

## Integration Architecture

### Metrics Export
- Prometheus metrics endpoint (`/metrics`)
- Push gateway integration for batch jobs
- Custom metrics registry management
- Metrics sampling and aggregation strategies

### Log Management
- Centralized logging infrastructure
- Log rotation and retention policies
- Structured logging with correlation IDs
- Security considerations for log content

### Tracing Infrastructure
- OpenTelemetry integration patterns
- Jaeger or Zipkin trace export
- Trace sampling strategies
- Cross-service trace correlation

## Technology Stack Research

### Rust Observability Crates
```toml
# Research: Evaluate these dependencies
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

### Monitoring Platform Integration
- Prometheus configuration for Rust services
- Grafana dashboard templates for microservices
- AlertManager rule configuration
- Service discovery integration

## Implementation Patterns

### 1. Metrics Collection Patterns
- Middleware-based metrics collection
- Custom derive macros for instrumentation
- Async-aware metrics collection
- Resource usage monitoring

### 2. Structured Logging Patterns
- Request lifecycle logging
- Error context preservation
- Performance timing logs
- Security audit logging

### 3. Health Check Patterns
- Dependency health verification
- Graceful degradation indicators
- Load balancer health check compatibility
- Deep vs shallow health checks

## Specific Research Areas

### Performance Monitoring
- How to track rendering performance across different diagram types?
- What are appropriate SLA targets for image rendering services?
- How to detect performance degradation patterns?
- Resource usage correlation with performance metrics

### Error Rate Monitoring
- Error classification for alerting purposes
- Error rate thresholds for different error types
- Error trend analysis and prediction
- Integration with error handling patterns (from research prompt 1)

### Service Discovery & Health
- Dynamic service discovery health integration
- Load balancer health check requirements
- Container orchestration health check patterns
- Multi-region deployment health monitoring

### Security Observability
- Authentication/authorization metrics
- Security event logging requirements
- Compliance audit trail requirements
- Data privacy considerations in logging

## Production Readiness Checklist

### Metrics
- [ ] Business KPI tracking implementation
- [ ] SLI/SLO definition and monitoring
- [ ] Resource utilization alerts
- [ ] Performance regression detection
- [ ] Capacity planning metrics

### Logging
- [ ] Structured log format standardization
- [ ] Log aggregation pipeline
- [ ] Log retention and archival
- [ ] Security audit trail
- [ ] Debug logging toggle mechanisms

### Monitoring
- [ ] Real-time dashboard creation
- [ ] Alert rule configuration
- [ ] Escalation policy definition
- [ ] Incident response playbook
- [ ] Monitoring infrastructure resilience

### Tracing
- [ ] End-to-end request tracing
- [ ] Performance bottleneck identification
- [ ] Error root cause analysis
- [ ] Service dependency mapping
- [ ] Trace data retention policies

## Integration with Existing Infrastructure

### Current Uveddi Observability
- Analyze existing logging patterns in codebase
- Integration with current error handling
- Compatibility with CLI tool observability
- Database operation monitoring alignment

### Service Mesh Integration
- Istio/Linkerd observability integration
- Sidecar metrics collection patterns
- Service mesh health check integration
- Traffic management observability

## Expected Research Deliverables

### 1. Observability Architecture Design
- Complete metrics taxonomy and collection strategy
- Logging infrastructure design and implementation plan
- Distributed tracing architecture and integration guide
- Health monitoring and alerting framework

### 2. Implementation Roadmap
- Phased rollout plan for observability features
- Performance impact assessment
- Resource requirements analysis
- Integration timeline with existing systems

### 3. Operations Documentation
- Runbook for observability infrastructure
- Alert response procedures
- Troubleshooting guides
- Performance tuning recommendations

### 4. Compliance & Security
- Data privacy compliance in observability
- Security considerations for metrics and logs
- Audit trail requirements
- Retention policy recommendations

## Success Criteria

- [ ] Complete observability coverage for all service operations
- [ ] Sub-1% performance overhead for observability infrastructure
- [ ] Real-time visibility into service health and performance
- [ ] Automated alerting for all critical failure scenarios
- [ ] Integration with existing monitoring infrastructure
- [ ] Compliance with security and privacy requirements

## Timeline

- **Technology Evaluation**: 2-3 days
- **Architecture Design**: 2-3 days
- **Integration Planning**: 1-2 days
- **Documentation**: 1 day
- **Total Estimated**: 6-9 days

---

**Next Steps**: Conduct comprehensive evaluation of observability technologies and design production-ready observability architecture for the rendering service.


A Production-Ready Observability Architecture for the Uveddi Rendering Service


1. Executive Summary & Recommended Architecture

This document presents a comprehensive observability architecture for the Uveddi image rendering service (UV-90), designed to meet the critical P1 requirements for production deployment. The primary objective of this architecture is to establish a robust, low-overhead (<1% performance impact) observability framework that enables proactive monitoring, rapid incident response, and data-driven decision-making. It covers the foundational pillars of modern observability: metrics collection, structured logging, distributed tracing, and health monitoring.
The proposed architecture is built on a set of core principles designed for scalability, flexibility, and operational efficiency. It standardizes on industry-best practices and open-source technologies to provide deep, real-time visibility into the service's health, performance, and business impact while adhering to stringent security and compliance standards.

Key Recommendations

Instrumentation Strategy: Adopt a unified, facade-based instrumentation strategy within the Rust service. By using the tracing and metrics crates, the application's core logic is decoupled from the specific observability backend, allowing for future flexibility and promoting a consistent, ergonomic developer experience.1
Telemetry Standard: Standardize on OpenTelemetry (OTel) as the universal language for all telemetry data (metrics, logs, and traces). This ensures seamless interoperability and context propagation across service boundaries (Rust to Node.js), integration with service meshes, and compatibility with a wide range of backend systems.3
Backend Technology Stack: Implement the "PLG" stack, a powerful and cost-effective combination of best-in-class open-source tools:
Prometheus: For time-series metrics collection, powerful querying with PromQL, and rule-based alerting.5
Grafana Loki: For efficient, index-light log aggregation, which offers significant cost and resource savings over traditional full-text indexing systems like ELK.7
Grafana: For unified visualization, providing a single pane of glass for dashboards that correlate metrics, logs, and traces.7
Central Collection Hub: Deploy the OpenTelemetry Collector as a central agent responsible for receiving, processing, and exporting all telemetry data. This component acts as a critical control point, offloading complex tasks like trace sampling, data enrichment, and credential management from the application service, thereby minimizing performance overhead and enhancing architectural flexibility.10

Architectural Overview

The proposed architecture establishes a clear, unidirectional flow of telemetry data from the application services to the backend analysis tools, orchestrated by the OpenTelemetry Collector.
Data Flow:
Instrumentation: The Rust-based image rendering service is instrumented using the tracing and metrics facade crates. Application logic emits logs, traces, and metrics through these standardized APIs.
Telemetry Emission: An OpenTelemetry (OTLP) exporter within the Rust service sends all telemetry data to a locally running OpenTelemetry Collector agent. This communication uses the efficient OTLP protocol.
Cross-Service Tracing: When the Rust service communicates with the upstream Node.js service, the OpenTelemetry SDK automatically injects W3C TraceContext headers into the HTTP requests. The Node.js service, also instrumented with OpenTelemetry, extracts this context to continue the distributed trace seamlessly.12
Collection and Processing: The OpenTelemetry Collector agent receives telemetry from all services. Here, it performs critical processing tasks:
Trace Sampling: Tail-based sampling policies are applied to keep 100% of traces with errors while sampling a fraction of successful traces, managing costs without sacrificing visibility into failures.10
Metric Conversion: OTel metrics are converted into the Prometheus exposition format.
Data Enrichment: Metadata, such as Kubernetes pod and namespace labels, is automatically added to all telemetry signals.
Redaction: Sensitive information can be scrubbed from logs and traces before export.
Data Export and Storage: The Collector fans the processed data out to the specialized backend systems:
Metrics are exported to a Prometheus server for storage and alerting.
Logs are sent to a Grafana Loki instance for aggregation and querying.
Traces are exported to a Jaeger or Grafana Tempo backend for storage and analysis.
Visualization and Alerting:
Grafana connects to Prometheus, Loki, and Jaeger/Tempo as data sources, providing a unified interface for building dashboards that correlate all three telemetry types.
Prometheus evaluates alerting rules based on the incoming metrics. When a rule's condition is met, it fires an alert to AlertManager.
AlertManager deduplicates, groups, and routes these alerts to the appropriate notification channels, such as PagerDuty for critical issues and Slack for warnings.
This architecture provides a complete, production-grade observability solution that is scalable, maintainable, and aligned with modern cloud-native principles.

Part I: Foundational Architectural Decisions

The success of any observability strategy rests on a small number of foundational architectural decisions. These choices dictate the flexibility, scalability, and maintainability of the entire system. For the Uveddi rendering service, we will establish two core principles: first, a commitment to facade-based instrumentation to decouple the application from the observability backend; and second, the adoption of the OpenTelemetry Collector as a central hub to manage and process all telemetry data. These decisions work in concert to create a robust and future-proof architecture.

2. A Unified Instrumentation Strategy for Rust

The first and most critical architectural decision is how to instrument the application code itself. The chosen strategy must be ergonomic for developers, impose minimal performance overhead, and, most importantly, avoid tightly coupling the application to a specific vendor or backend technology. To achieve this, the recommended approach is to adopt a "facade-first" principle for all instrumentation within the Rust codebase.

The Facade-First Principle

A facade provides a simple, stable API for a larger body of code. In the context of Rust observability, facade crates like log, metrics, and tracing offer a set of macros and functions for emitting telemetry data without any knowledge of how that data will eventually be collected, processed, or stored.1 Application and library developers write their instrumentation code against these stable facade APIs. The final application binary then plugs in a specific "subscriber" or "exporter" crate that handles the work of sending the telemetry to a chosen backend like Prometheus or Jaeger.
This pattern is well-established and highly successful in the Rust ecosystem, with the log crate being the canonical example. By extending this principle to metrics and traces, we gain several significant advantages:
Future-Proofing the Architecture: The most significant benefit of using facades is backend independence. The application code is completely decoupled from the implementation details of the observability stack. If, in the future, the Uveddi platform decides to migrate from Prometheus to a commercial APM solution, or from Loki to a different log aggregator, the change can be accomplished by swapping out the exporter dependency and updating the initialization code. No changes to the core application logic or its thousands of instrumentation points would be required. This architectural agility is invaluable in a rapidly evolving technology landscape.
Improving Developer Experience: Facade APIs are designed to be simple and ergonomic. Macros like tracing::info! or metrics::increment_counter! are intuitive and easy to use, which encourages developers to instrument their code thoroughly and consistently.15 This leads to a richer and more complete telemetry dataset.
Enabling Library-Level Instrumentation: For a platform like Uveddi, which may consist of many internal libraries and crates, facades are essential. A shared library can use the metrics crate to emit performance data without forcing every consumer of that library to adopt a specific metrics backend. The final application is free to choose its own backend, and the library's metrics will be exported seamlessly.

OpenTelemetry as the Lingua Franca

While facades provide decoupling within the application, OpenTelemetry (OTel) provides the standard for communication between components of the observability stack.3 OpenTelemetry is an open-source observability framework that specifies a vendor-agnostic protocol (OTLP) and a set of APIs and SDKs for collecting and exporting telemetry data.17
By adopting OpenTelemetry as our "lingua franca," we ensure that all our telemetry signals—metrics, logs, and traces—can be understood by a vast ecosystem of tools. This is the cornerstone of our strategy for cross-service and cross-platform interoperability. Specifically, OTel provides:
A Standard Protocol (OTLP): A single, efficient protocol for sending all telemetry data from our services to a collector.
Standardized Context Propagation: A specification (W3C TraceContext) for propagating trace information across process and network boundaries, which is critical for distributed tracing between our Rust and Node.js services.12
A Rich Ecosystem: A wide array of collectors, exporters, and backend systems that are OTel-compatible, giving us maximum choice and avoiding vendor lock-in.
In summary, our unified instrumentation strategy is twofold: use Rust-native facades (metrics, tracing) for ergonomic, decoupled instrumentation within the application, and use OpenTelemetry as the standardized transport and propagation layer to ensure interoperability across the entire distributed system.

3. The OpenTelemetry Collector: The Central Telemetry Hub

The second foundational decision is to introduce the OpenTelemetry Collector as a central component in our architecture. Instead of having each service instance export its telemetry data directly to multiple backends (Prometheus, Loki, Jaeger), services will send all their data to a local or nearby OTel Collector instance. This collector then becomes responsible for processing and fanning out the data to the appropriate destinations.11
Deploying the OTel Collector as a dedicated agent (e.g., as a sidecar container or a Kubernetes DaemonSet) is a strategic choice that directly addresses several key requirements of the prompt, including performance, flexibility, and security.

Architectural Role and Key Benefits

The OTel Collector is a highly configurable and extensible binary that acts as a "Swiss Army knife" for telemetry data. Its primary role in our architecture is to be the single, stable endpoint for all application telemetry.
The key benefits of this approach are:
Decoupling Services from Backends: The rendering service only needs to know how to send OTLP data to a single, well-known collector endpoint (e.g., http://localhost:4317). The collector abstracts away all the complexity of backend configuration. It handles the specific protocols, authentication keys, endpoint addresses, batching logic, and retry policies for each destination (Prometheus, Loki, Jaeger).11 This simplifies the application's configuration and logic significantly.
Centralized, Advanced Processing: The collector is the ideal location to perform resource-intensive processing tasks that would be inefficient or impractical to run inside the application service itself. The most important of these is tail-based sampling for traces.10 Tail-based sampling requires buffering all spans for a given trace before making a decision to keep or drop it. Performing this in-process would consume significant memory and CPU in our performance-critical rendering service. By offloading this to the collector, we keep the application lean while still enabling intelligent sampling strategies (e.g., keeping 100% of traces with errors).10
Data Enrichment and Redaction: The collector can be configured with processors to automatically enrich telemetry with valuable metadata. For example, the k8sattributes processor can add Kubernetes-specific labels like pod_name, namespace, and deployment to all metrics, logs, and traces. Conversely, it can also be used to redact or mask sensitive information (like PII in log messages or trace attributes) before the data leaves the trusted cluster boundary, which is a critical feature for security and compliance.
By introducing the OTel Collector, we establish a strategic control point for our entire observability pipeline. The application's concern is simply to produce telemetry. The collector's concern is to process, secure, and route that telemetry. This separation of concerns is a hallmark of a mature, scalable architecture. It allows the operations team to modify routing rules, change sampling rates, or even add entirely new observability backends by only reconfiguring and redeploying the collector, with zero impact on the application services. This operational agility directly supports the project's goal of building a robust, production-ready system.

Part II: Metrics, Monitoring, and Service Reliability

With the foundational architecture established, this section details the specifics of implementing the metrics and monitoring pillar. This includes selecting the optimal Rust libraries for instrumentation, defining a concrete set of Service Level Indicators (SLIs) and Objectives (SLOs) to measure reliability, and designing the alerting and visualization components using Prometheus and Grafana.

4. Metrics Collection: Instrumenting for Performance

Metrics are the cornerstone of performance monitoring, providing the quantitative data needed to understand system behavior, track reliability, and trigger alerts. Our strategy focuses on choosing the right tools for Rust, implementing them idiomatically, and adhering to best practices to minimize performance overhead.

Analysis of Rust Metrics Libraries

The Rust ecosystem offers several high-quality crates for metrics instrumentation. The primary decision point is between using a facade-based approach, which abstracts the backend, or a direct implementation, which is tied to a specific backend like Prometheus. The key candidates are the metrics facade, the prometheus crate, and the newer prometheus-client crate.19

Criterion
metrics + metrics-exporter-prometheus
prometheus Crate
prometheus-client Crate
Architectural Approach
Facade-based. Decouples application code from the Prometheus backend. High architectural flexibility.1
Direct implementation. Tightly couples application code to the Prometheus data model and registry.21
Direct implementation. A modern, type-safe implementation focused on OpenMetrics compliance.22
Backend Flexibility
High. Can export to any backend by swapping the exporter crate (e.g., TCP, InfluxDB, etc.).2
Low. Designed specifically and only for Prometheus-compatible systems.
Low. Designed specifically for OpenMetrics/Prometheus.
API Ergonomics
Excellent. Simple, log-like macros (counter!, gauge!, histogram!) that are easy for developers to adopt.15
Good, but more verbose. Requires manual creation of Opts and registration with a Registry.21
Very good and type-safe. Leverages the type system to prevent common errors at compile time.22
OpenMetrics Compliance
Good. The exporter handles translation and sanitization of metric names and labels to be Prometheus-compliant.25
Good. Mature and widely used.
Excellent. A primary design goal is strict compliance with the OpenMetrics specification.22
Performance Overhead
Very Low. The facade itself is extremely lightweight, with overhead determined by the installed recorder.
Very Low. Highly optimized for the Prometheus use case.
Very Low. Performance is a key design goal.22
Ecosystem Maturity
High. The metrics facade is widely adopted and considered a standard pattern in the Rust community.23
High. The de facto standard for direct Prometheus instrumentation for many years.
Medium. Newer than the prometheus crate but gaining traction.

Recommendation: The recommended approach for the Uveddi rendering service is to use the metrics facade in combination with the metrics-exporter-prometheus crate.
Justification: This combination provides the best of both worlds. We get the superior architectural properties of a facade-based design—namely, backend independence and a simple, ergonomic API for developers—while still producing fully Prometheus-compatible metrics.1 This aligns perfectly with our foundational principle of decoupling. While the
prometheus crate is mature, it locks us into a single ecosystem.21 Using it directly would make it difficult to integrate libraries that use the
metrics facade and would complicate any future migration to a different backend. There are known "impedance mismatch" issues when trying to bridge the two ecosystems with crates like metrics-prometheus, which can lead to panics if not handled carefully.19 The
metrics-exporter-prometheus crate provides a clean, direct path from the metrics facade to a Prometheus-compatible endpoint without these complications.26

Implementation of Custom Metrics

The following is an idiomatic Rust example demonstrating how to implement the required RenderingMetrics and expose them via an HTTP endpoint.

Rust


// In Cargo.toml
// [dependencies]
// metrics = "0.21"
// metrics-exporter-prometheus = "0.12"
// tokio = { version = "1", features = ["full"] }
// lazy_static = "1.4"
// sysinfo = "0.29"

use metrics::{counter, gauge, histogram, Unit};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use std::time::Duration;
use sysinfo::{System, SystemExt};

// A struct to hold metric descriptions, making them easier to manage.
pub struct RenderingMetrics {
    pub requests_total: &'static str,
    pub request_duration_seconds: &'static str,
    pub active_requests: &'static str,
    pub errors_total: &'static str,
    pub service_availability: &'static str,
    pub memory_usage_bytes: &'static str,
    pub render_queue_depth: &'static str,
}

pub const METRICS: RenderingMetrics = RenderingMetrics {
    requests_total: "rendering_requests_total",
    request_duration_seconds: "rendering_request_duration_seconds",
    active_requests: "rendering_active_requests",
    errors_total: "rendering_errors_total",
    service_availability: "rendering_service_availability",
    memory_usage_bytes: "rendering_memory_usage_bytes",
    render_queue_depth: "rendering_render_queue_depth",
};

// Initialize the Prometheus exporter and return a handle to render the metrics.
pub fn initialize_metrics() -> PrometheusHandle {
    let builder = PrometheusBuilder::new();
    // It's good practice to spawn a background task for upkeep,
    // especially for histograms which can accumulate memory. [25]
    builder
       .install_recorder()
       .expect("Failed to install Prometheus recorder")
}

// Example of a middleware for a web framework like Axum
// This pattern automatically tracks key request metrics. [28]
pub async fn track_metrics<B>(req: http::Request<B>, next: axum::middleware::Next<B>) -> impl axum::response::IntoResponse {
    let start = std::time::Instant::now();
    let path = req.uri().path().to_string();
    let method = req.method().clone();

    gauge!(METRICS.active_requests, 1.0, "endpoint" => path.clone());
    let response = next.run(req).await;
    gauge!(METRICS.active_requests, -1.0, "endpoint" => path.clone());

    let latency = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();

    let labels = [
        ("method", method.to_string()),
        ("path", path),
        ("status", status),
    ];

    counter!(METRICS.requests_total, 1, &labels);
    histogram!(METRICS.request_duration_seconds, latency, &labels);

    response
}

// A function to spawn a background task for system resource monitoring.
pub fn spawn_system_metrics_collector() {
    tokio::spawn(async move {
        let mut sys = System::new_all();
        let mut interval = tokio::time::interval(Duration::from_secs(10));
        loop {
            interval.tick().await;
            sys.refresh_memory();
            let used_memory = sys.used_memory();
            gauge!(METRICS.memory_usage_bytes, used_memory as f64);
            // Add other system metrics like CPU usage from sysinfo here.
        }
    });
}


This example demonstrates several key patterns:
Initialization: The initialize_metrics function sets up the global recorder provided by metrics-exporter-prometheus.25
Middleware Pattern: The track_metrics function is a web framework middleware that automatically increments counters and records histograms for every request, reducing boilerplate code in individual handlers.28
Resource Monitoring: The spawn_system_metrics_collector function runs a background task to periodically collect system-level metrics like memory usage using the sysinfo crate, as direct process monitoring is a common requirement.29

Minimizing Performance Overhead

While modern metrics libraries are highly optimized, performance must be a primary consideration. The following best practices, derived from Prometheus guidelines, will be enforced to maintain the sub-1% overhead target 28:
Control Label Cardinality: The number of unique time series a metric can generate is its cardinality. High cardinality is the most common cause of performance degradation in monitoring systems. Labels with unbounded values (e.g., user_id, request_id, timestamp) must be avoided.
The diagram_type Case: The requirement to track performance by diagram_type presents a potential cardinality risk. If there are thousands of diagram types, or if they can be user-defined, this could create an unmanageable number of time series. The solution is to apply a bounded approach:
Define a fixed, allow-listed set of the most common and business-critical diagram_type values.
Instrument the code to use these values as a label: histogram!("render_duration_seconds", duration, "diagram_type" => "flowchart").
For any diagram_type not in the allow-list, aggregate it under a generic other label: histogram!("render_duration_seconds", duration, "diagram_type" => "other").
This strategy provides the necessary business insight for key diagram types without compromising the stability and performance of the Prometheus server.
Avoid Instrumentation in Hot Loops: For extremely performance-sensitive code paths that are executed hundreds of thousands of times per second, avoid placing metric increments inside the tightest loops. If necessary, aggregate counts in a local variable and increment the global metric once outside the loop.

5. Defining Reliability: SLIs, SLOs, and Error Budgets

To effectively monitor the service and align engineering efforts with business goals, we must move beyond simple metrics and define formal reliability targets. This is achieved using the Site Reliability Engineering (SRE) framework of Service Level Indicators (SLIs), Service Level Objectives (SLOs), and Error Budgets.30
Service Level Indicator (SLI): A quantitative measure of some aspect of the service's performance. An SLI is the actual measurement (e.g., the measured error rate over the last 5 minutes).30
Service Level Objective (SLO): A target value or range for an SLI over a period of time. An SLO is the goal we are trying to meet (e.g., 99.9% of requests should be successful over a 30-day window).32
Service Level Agreement (SLA): A formal contract with a customer that defines the consequences of failing to meet certain objectives, often involving financial penalties. SLOs are typically stricter than SLAs to provide an internal safety margin.31

SLIs for the Uveddi Image Rendering Service

Choosing the right SLIs is critical; they must accurately reflect the user's experience.35 For the image rendering service, we will define SLIs across three key pillars: Availability, Latency, and Quality.

Pillar
SLI Name
SLI Specification (The ratio of...)
Recommended SLO Target (over 30 days)
Justification
Availability
api_availability
(Total valid API requests - Total requests with 5xx HTTP status) / (Total valid API requests)
99.9%
Measures the basic reliability of the service's front door. Failures here mean users cannot even submit a job.


render_success_rate
(Total rendering jobs completed without application error) / (Total valid rendering jobs submitted)
99.5%
Measures the core business function's reliability. A lower target than API availability acknowledges that rendering is complex and can fail for reasons other than infrastructure (e.g., malformed input).
Latency
api_fast_requests
(Total API requests served in < 200ms) / (Total valid API requests)
99%
Ensures the user-facing API is responsive for quick interactions like submitting a job or checking status.35


p95_render_duration
95th percentile rendering time for diagram_type="flowchart" is < 5 seconds.
Met continuously
Targets the performance of the most common rendering tasks, ensuring a good user experience for the majority of use cases. This is a distribution-based SLO.30
Quality
render_output_validity
(Total renders producing a valid, non-zero-byte output file) / (Total successful renders)
99.99%
A proxy for quality. A successful render that produces a corrupt or empty file is a critical failure. This is a fundamental correctness check.


render_fidelity_rate
(Total renders where output dimensions match requested dimensions) / (Total successful renders)
99.99%
Another quality proxy. Ensures the rendered output conforms to the user's explicit request parameters.


render_corruption_free_rate
(Total renders that complete without internal engine corruption flags) / (Total successful renders)
99.99%
An internal quality metric. The rendering engine should have internal checks that can flag potential data corruption, which this SLI tracks.

A crucial aspect of this SLI list is the approach to measuring Quality. Directly measuring the aesthetic quality of a rendered image programmatically is an unsolved problem.36 The research on this topic often points to CSS properties or subjective user feedback, which are not applicable here.37 Therefore, the strategy is to define objective, automatable
proxy metrics that strongly correlate with user-perceived quality. A render that produces a zero-byte file, has the wrong dimensions, or is internally flagged as corrupt is objectively a low-quality result. By defining SLIs around output validity, fidelity, and corruption, we create a measurable and actionable framework for tracking service quality, which is a pragmatic engineering solution to an otherwise intractable problem.39

Error Budgets: The Currency of Reliability

Each SLO implicitly defines an error budget—the amount of unreliability that is permissible over the SLO window.34 For example:
A 99.9% availability SLO over a 30-day period (43,200 minutes) allows for (1 - 0.999) * 43200 = 43.2 minutes of downtime.
If the service handles 1,000,000 requests in a month, a 99.5% success rate SLO allows for (1 - 0.995) * 1000000 = 5,000 failed requests.
The error budget is a powerful, data-driven tool for the engineering team. It provides a clear signal for when to prioritize reliability work over shipping new features. If the budget is being consumed too quickly, the team must focus on stability. If the budget is consistently healthy, the team has the "budget" to take calculated risks, such as rolling out new features more aggressively.

6. Alerting and Visualization: Prometheus & Grafana

The metrics and SLOs we've defined are only useful if they are actively monitored and visualized. The combination of Prometheus for alerting and Grafana for visualization provides a powerful, open-source solution for this purpose.

Prometheus and AlertManager Configuration

The monitoring stack will be configured as follows:
Prometheus Scraping: The prometheus.yml file will be configured to automatically discover and scrape the /metrics endpoint of all Uveddi rendering service pods using Kubernetes service discovery.
YAML
# prometheus.yml snippet
scrape_configs:
  - job_name: 'uveddi-rendering-service'
    kubernetes_sd_configs:
      - role: pod
    relabel_configs:
      # Scrape only pods with the correct app label
      - source_labels: [__meta_kubernetes_pod_label_app]
        action: keep
        regex: uveddi-rendering-service


Prometheus Alerting Rules: Alerting rules will be defined in a separate alerts.yml file and loaded by Prometheus. These rules will be based directly on our SLOs, focusing on the rate of error budget consumption to avoid flapping alerts for transient issues.41
YAML
# alerts.yml example
groups:
  - name: UveddiRenderingSLOAlerts
    rules:
      - alert: HighErrorRateBudgetBurn
        # This expression checks if the 5-minute error rate is high enough
        # that it would consume the entire 30-day error budget in just 4 hours.
        expr: |
          sum(rate(rendering_requests_total{status=~"5.."}[5m]))
          /
          sum(rate(rendering_requests_total[5m]))
          > (14.4 * (1 - 0.999)) # 14.4 is the burn rate factor for 4h, 0.999 is the SLO
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "High error rate on Uveddi rendering service"
          description: "The service is burning through its 30-day error budget too quickly. Current error rate is {{ $value | humanizePercentage }}."


AlertManager Routing: The alertmanager.yml file will configure the routing logic for fired alerts. This ensures that the right people are notified through the right channels based on the alert's severity.42
YAML
# alertmanager.yml example
route:
  group_by: ['alertname', 'cluster']
  group_wait: 30s
  group_interval: 5m
  repeat_interval: 2h
  receiver: 'default-slack'
  routes:
    - receiver: 'critical-pagerduty'
      matchers:
        - severity="critical"
      continue: true

receivers:
  - name: 'default-slack'
    slack_configs:
      - api_url: 'https://hooks.slack.com/services/...' # [44, 45]
        channel: '#uveddi-alerts'
        send_resolved: true
  - name: 'critical-pagerduty'
    pagerduty_configs:
      - routing_key: '...' # [46, 47]

This configuration sends all alerts to Slack by default, but any alert with severity="critical" will also be sent to PagerDuty to wake up the on-call engineer.48

Grafana Dashboards for Microservices

Effective dashboards are crucial for quickly diagnosing issues. Instead of creating dozens of ad-hoc dashboards, we will follow design best practices to create a small number of powerful, reusable dashboard templates.50
Dashboard Design Best Practices:
Know Your Audience: A dashboard for an on-call engineer should be different from one for a business analyst. The on-call dashboard should prioritize SLOs and system health, while a business dashboard might focus on rendering throughput by diagram type.
Top-Down Information Flow: Dashboards should be organized logically. The most important, high-level information (our SLIs) should be at the very top. As you scroll down, the information should become more granular (e.g., resource utilization, individual endpoint performance).51
The RED Method: For any request-based service, dashboards should be structured around the "RED" metrics: Rate (requests per second), Errors (number of failed requests), and Duration (latency distribution of requests). This provides an at-a-glance summary of service health.51
Leverage Templates and Variables: To avoid dashboard sprawl, we will create a single "Uveddi Rendering Service" master dashboard. This dashboard will use Grafana variables to allow users to dynamically filter the view by environment (prod, staging), kubernetes_namespace, or diagram_type. This is vastly more maintainable than having separate dashboards for each combination.50
Proposed "Uveddi Rendering Service" Dashboard Template:
This dashboard will provide a comprehensive overview of the service's health and performance.53
Row 1: Service Level Objectives:
Stat panels showing the current status of each key SLO (Availability, Latency, Quality) and the remaining error budget percentage.
Row 2: API RED Metrics:
Graphs for Request Rate, Error Rate (broken down by HTTP status code), and Request Duration (p50, p90, p99 percentiles).
Row 3: Rendering Engine Performance:
Graphs for Rendering Throughput (renders per minute), Render Queue Depth, and Rendering Duration by diagram type (using the template variable).
Row 4: Resource Utilization:
Graphs for CPU Usage, Memory Usage, and Network I/O for the service pods.
Row 5: Logs and Traces:
A log panel showing recent error logs from Loki, and a service graph panel from Jaeger/Tempo showing dependencies. These panels will be linked to the dashboard's time range and variables, allowing for seamless correlation.

Part III: Deep Insights with Logging and Tracing

While metrics provide the "what" (e.g., "the error rate is high"), logs and traces provide the "why." Structured logging allows us to inspect the state of a specific failed request, while distributed tracing allows us to follow that request's entire journey through our microservices architecture. Together, they are indispensable tools for debugging and root cause analysis.

7. Structured Logging for Actionable Diagnostics

To make logs useful for automated analysis, they must be treated as structured data, not free-form text. Our strategy mandates that all log output from the Uveddi rendering service be in a machine-parsable, structured JSON format.

Standardizing on Structured JSON

Structured JSON logs are key-value pairs that provide rich, queryable context with every log message. This format is essential for modern log aggregation systems, as it allows for efficient filtering, aggregation, and correlation without relying on fragile regular expression parsing.56 Every log entry will automatically include essential metadata like a timestamp, log level, service name, and, crucially, the trace and span IDs of the request that generated the log.

Implementation with tracing-subscriber

The tracing crate ecosystem provides excellent support for structured logging via the tracing-subscriber crate. By enabling the json feature, we can configure a formatter that outputs all trace events and spans as newline-delimited JSON.56

Rust


// In Cargo.toml
// [dependencies]
// tracing = "0.1"
// tracing-subscriber = { version = "0.3", features = ["json", "registry"] }
// tracing-opentelemetry = "0.21"
// opentelemetry = "0.20"

use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, EnvFilter};

// This function sets up the global subscriber for logging and tracing.
pub fn initialize_subscriber(tracer: opentelemetry::sdk::trace::Tracer) {
    // Layer for filtering based on RUST_LOG environment variable
    let filter_layer = EnvFilter::from_default_env();

    // Layer for OpenTelemetry tracing
    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    // Layer for JSON logging to stdout
    let fmt_layer = fmt::layer()
       .json() // Enable JSON output format [57]
       .with_current_span(true) // Include current span's info in logs [57]
       .with_span_list(true); // Include the full span hierarchy [58]

    // Compose the layers and set the global subscriber
    tracing_subscriber::registry()
       .with(filter_layer)
       .with(telemetry_layer)
       .with(fmt_layer)
       .init();
}

// Example usage in application code:
// tracing::info!(request_id = "xyz-123", "Processing new render request");
// This will produce a JSON log line containing the message and the request_id field,
// as well as the trace_id and span_id from the active OTel context.


This configuration is powerful because it seamlessly integrates the three pillars. The fmt::layer is configured with .json(), ensuring all tracing::info!, warn!, etc. calls are serialized as JSON. Critically, .with_current_span(true) and .with_span_list(true) instruct the formatter to automatically capture the context from the telemetry_layer, embedding the trace_id and span_id directly into every log line. This provides automatic log correlation out of the box. For specialized use cases requiring even higher performance, the json-subscriber crate is a viable alternative that offers a similar API.59

Log Aggregation Backend: Loki vs. ELK Stack

The choice of log aggregation system has significant implications for cost, operational complexity, and query capabilities. The two leading open-source contenders are the traditional ELK Stack (Elasticsearch, Logstash, Kibana) and the newer, cloud-native Grafana Loki.

Feature
Grafana Loki
ELK Stack (Elasticsearch, Logstash, Kibana)
Indexing Strategy
Indexes only metadata (labels). Log content is compressed and stored as-is.60
Full-text indexing. Indexes the entire content of every log message.8
Storage/Cost Efficiency
Very High. Significantly lower storage and memory footprint due to minimal indexing.8
Low. High resource consumption due to the overhead of building and maintaining inverted indexes.62
Query Language
LogQL. A Prometheus-like query language for filtering based on labels and then optionally filtering content with grep-like syntax.62
KQL / Lucene Query Syntax. A powerful, full-featured search language for complex text analysis and queries.8
Resource Consumption
Low. Designed to be lightweight and horizontally scalable.60
High. Elasticsearch and Logstash are known to be resource-intensive, requiring significant memory and CPU.8
Ease of Setup
Simple. Designed for easy integration with Prometheus and Grafana, especially in Kubernetes environments.8
Complex. Requires configuration of three separate, powerful components (Elasticsearch, Logstash, Kibana).8
Kubernetes Integration
Excellent. Promtail agent is purpose-built for discovering and labeling logs from Kubernetes pods.8
Good. Multiple agents available (Filebeat, Fluentd), but generally more complex to configure than Promtail.63

Recommendation: Grafana Loki.
Justification: For the Uveddi project's requirements, Grafana Loki is the superior choice. Its design philosophy, "like Prometheus, but for logs," aligns perfectly with our existing technology choices.7 Our primary use case for logs is not complex, full-text analytics; it is debugging and diagnostics, specifically correlating logs with metrics and traces from a specific request. Loki excels at this. By attaching labels like
app="uveddi-rendering", pod="renderer-xyz", and the trace_id, we can use LogQL to instantly retrieve all logs for a specific request or a specific pod with very low query latency and minimal storage cost. The seamless integration with Grafana means we can visualize logs, metrics, and traces in a single dashboard without context switching, which is a massive productivity win for developers and on-call engineers.62 While the ELK stack is more powerful for deep log analytics, its operational overhead and cost are not justified for our needs.60

Log Correlation in Practice

With this architecture, a typical debugging workflow in Grafana becomes incredibly efficient:
An engineer sees a spike in the rendering_errors_total metric on a dashboard.
They click on a point in the graph, which is annotated with an exemplar containing a trace_id.
This trace_id is used to automatically generate a link to the Jaeger/Tempo UI, showing the full distributed trace for a sample failed request.
The same trace_id is also used to generate a link to the Loki query explorer in Grafana, pre-filled with the query: {app="uveddi-rendering"} |= "trace_id=....
The engineer can now see the exact metrics, traces, and logs associated with a single failed request, all within a few clicks, dramatically reducing Mean Time to Resolution (MTTR).

8. Distributed Tracing: Unraveling Request Lifecycles

Distributed tracing is essential for understanding the behavior of microservices-based applications. It allows us to visualize the entire lifecycle of a request as it travels through different services, making it possible to pinpoint performance bottlenecks and identify the root cause of errors in a complex system.64

Implementation with tracing-opentelemetry

As with logging, we will use the tracing facade for instrumentation, but this time we will bridge it to OpenTelemetry using the tracing_opentelemetry crate. This crate provides a Layer that can be added to a tracing_subscriber, which automatically converts tracing spans and events into the OpenTelemetry data model and sends them to a configured exporter.16
A complete initialization example, building on the logging setup, is as follows:

Rust


// In Cargo.toml
// [dependencies]
// opentelemetry = { version = "0.20", features = ["rt-tokio"] }
// opentelemetry-otlp = { version = "0.13", features = ["tonic"] }
// tracing = { version = "0.1", features = ["attributes"] }
// tracing-subscriber = { version = "0.3", features = ["json", "registry"] }
// tracing-opentelemetry = "0.21"

use opentelemetry::sdk::propagation::TraceContextPropagator;
use opentelemetry::sdk::{trace as sdktrace, Resource};
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use tracing::instrument;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, EnvFilter};

// Initializes an OTLP tracer that sends data to a collector.
fn init_tracer() -> Result<sdktrace::Tracer, opentelemetry::trace::TraceError> {
    // Initialize the OTLP exporter, which sends traces over gRPC.
    let exporter = opentelemetry_otlp::new_exporter()
       .tonic()
       .with_endpoint("http://localhost:4317"); // OTel Collector endpoint

    opentelemetry_otlp::new_pipeline()
       .tracing()
       .with_exporter(exporter)
       .with_trace_config(
            sdktrace::config().with_resource(Resource::new(vec![KeyValue::new(
                "service.name",
                "uveddi-rendering-service",
            )])),
        )
       .install_batch(opentelemetry::sdk::runtime::Tokio)
}

// Full subscriber setup for logs and traces.
pub fn initialize_observability() {
    // Set the W3C TraceContext propagator as the global standard.
    opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());

    let tracer = init_tracer().expect("Failed to initialize OTLP tracer");

    let filter_layer = EnvFilter::from_default_env();
    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);
    let fmt_layer = fmt::layer().json().with_current_span(true).with_span_list(true);

    tracing_subscriber::registry()
       .with(filter_layer)
       .with(telemetry_layer)
       .with(fmt_layer)
       .init();
}

// Example of an instrumented function.
#[instrument(
    name = "render_diagram",
    fields(
        diagram.id = %diagram_id,
        diagram.type = %diagram_type
    ),
    skip(diagram_data), // Avoid logging large data blobs
    err
)]
pub async fn render_diagram(diagram_id: &str, diagram_type: &str, diagram_data: &[u8]) -> Result<(), std::io::Error> {
    //... rendering logic...
    if let Err(e) = some_fallible_operation() {
        // The `err` field in `#[instrument]` automatically records the error,
        // but we can also record it manually for more context.
        tracing::error!(error.message = %e, "A specific step failed");
        return Err(e);
    }
    Ok(())
}


This example shows:
Exporter Configuration: init_tracer configures an OTLP exporter to send traces to the OTel Collector.67
Layer Composition: The OpenTelemetryLayer is composed with the logging and filtering layers in the tracing_subscriber.69
Automatic Instrumentation: The #[instrument] macro automatically creates a new span every time render_diagram is called. It captures the function arguments as span attributes and, importantly, the err parameter ensures that if the function returns an Err, the error is automatically recorded on the span and its status is set to Error.16

Context Propagation: The Cross-Service Challenge

A trace is only useful if it can be correlated across service boundaries. The key requirement is to propagate the trace context (specifically, the trace_id and parent span_id) from the calling service to the receiving service.
The Standard: W3C TraceContext: To ensure interoperability between our Rust and Node.js services, we will standardize on the W3C TraceContext specification. This is the industry-standard, vendor-neutral protocol for context propagation and is supported by default in most OpenTelemetry SDKs. It works by passing two HTTP headers: traceparent and tracestate.12
Rust Configuration: In our Rust service, calling opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new()); during initialization, as shown in the example above, configures the SDK to automatically inject and extract these headers for all instrumented HTTP clients and servers.
Node.js Configuration: A parallel configuration is required in the upstream Node.js service. The OpenTelemetry SDK for Node.js, when used with the @opentelemetry/instrumentation-http and @opentelemetry/instrumentation-express packages, will automatically handle W3C context propagation. The configuration in the tracing.js file should explicitly set the propagator 72:
JavaScript
// In Node.js tracing.js setup
const { CompositePropagator, W3CTraceContextPropagator, W3CBaggagePropagator } = require("@opentelemetry/core");

const sdk = new NodeSDK({
  //... other config
  textMapPropagator: new CompositePropagator({
    propagators:,
  }),
  instrumentations: [getNodeAutoInstrumentations()],
});


With this setup, when the Node.js service calls the Rust rendering service, the trace context will be seamlessly passed, and the spans generated by the Rust service will correctly appear as children of the spans from the Node.js service in the trace view.

Trace Backend and Sampling

Backend Recommendation: For the trace storage backend, Jaeger is the recommended starting point. It is a mature, feature-rich, and widely adopted open-source distributed tracing system.64 The all-in-one Jaeger deployment is simple to set up for development and initial production use. As the system scales and trace volume grows, migrating to
Grafana Tempo should be considered. Tempo is designed for massive scale and offers native integration with Loki and Prometheus within Grafana, which aligns with our overall stack.
Sampling Strategy: Ingesting and storing 100% of traces from a high-throughput service is prohibitively expensive and often unnecessary. We must employ a sampling strategy to manage costs while retaining critical diagnostic information.
Strategy
Decision Point
Pros
Cons
Best For
Head-based (Probabilistic)
At the beginning of the trace (on the root span).
Low resource overhead, predictable cost. Simple to implement in the SDK.
"Dumb" sampling; may drop important traces (e.g., those with errors) just by chance. Misses context from the full trace.
Very high-volume services where cost control is paramount and losing some error traces is acceptable.
Tail-based
After all spans in a trace have been collected and buffered.
Intelligent sampling. Can make decisions based on the full trace, e.g., "keep all traces with errors" or "keep all slow traces."
High resource overhead (memory/CPU) to buffer traces. Introduces latency in trace visibility. Complex to implement.
Production environments where capturing 100% of interesting traces (errors, high latency) is critical for reliability.

Recommendation: Hybrid Sampling via OTel Collector.
The optimal strategy is a hybrid approach, enabled by our OTel Collector architecture.
Client-side (SDK): The Rust and Node.js services will be configured to send 100% of traces to the OTel Collector.
Collector-side (Agent): The OTel Collector will be configured with the Tail Sampling Processor.10 This processor will buffer traces for a short period (e.g., 30 seconds) and apply a set of policies before exporting them to Jaeger. A recommended initial policy set would be:
Keep any trace where the root span's status is Error.
Keep any trace where the root span's duration exceeds a certain threshold (e.g., 5 seconds).
For all other traces, apply a probabilistic sampling rate of 10%.
This hybrid strategy gives us the best of both worlds: we get 100% visibility into all errors and significant performance issues, while effectively managing the cost and volume of storing routine, successful traces.

Part IV: Operational and Security Readiness

A complete observability stack goes beyond instrumentation and data collection. It must also include robust mechanisms for monitoring the health of the service itself and be designed with security and compliance as first-class citizens. This final section details the patterns for health checks, resiliency, and the security considerations for the entire observability pipeline.

9. Service Health and Resiliency Patterns

Health checks are critical for enabling automated systems like container orchestrators and load balancers to manage the lifecycle of our service instances effectively. A poorly designed health check can be worse than none at all, potentially causing cascading failures. Therefore, we must distinguish between different types of health checks for different purposes.

Kubernetes Probes: Liveness vs. Readiness

Kubernetes defines three types of probes to manage container health, but the two most important for our service are liveness and readiness probes. It is crucial to understand their distinct roles and implement them correctly.76
Liveness Probe (/livez): The purpose of the liveness probe is to answer the question: "Is this container running correctly?" If the liveness probe fails, Kubernetes's response is drastic: it kills the container and attempts to restart it according to its restart policy.78 Because the consequence is so severe, a liveness probe should be a
shallow check. It should verify that the application's main loop is running and the HTTP server is responsive, but it must not check the health of its dependencies. If a liveness probe checks a downstream database, and that database has a transient failure, Kubernetes could restart all service pods, turning a temporary database issue into a full-scale service outage.
Readiness Probe (/readyz): The purpose of the readiness probe is to answer the question: "Is this container ready to accept production traffic?" If the readiness probe fails, Kubernetes's response is more graceful: it removes the pod's IP address from the corresponding Service's endpoints, effectively taking it out of the load balancer's rotation until it becomes ready again.76 A readiness probe should be a
deep check. It should verify not only its own internal state but also its ability to connect to critical downstream dependencies like databases or message queues. This ensures that traffic is only sent to pods that are fully capable of handling it.80

Health Check Implementation in Rust

We will implement two distinct HTTP endpoints in the Uveddi rendering service to serve these probes.

Rust


// Example using the Axum web framework

use axum::{http::StatusCode, response::IntoResponse, routing::get, Router};
use std::sync::Arc;

// A shared state to hold dependency health status
struct AppState {
    // In a real app, this would be updated by background checks
    database_healthy: std::sync::atomic::AtomicBool,
}

// Liveness probe: A simple, shallow check.
// This should almost never fail unless the process is deadlocked or crashed.
async fn liveness_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

// Readiness probe: A deep check that includes dependencies.
async fn readiness_check(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
) -> impl IntoResponse {
    // Check internal state and critical dependencies.
    if state.database_healthy.load(std::sync::atomic::Ordering::Relaxed) {
        (StatusCode::OK, "Ready")
    } else {
        // Return a 503 Service Unavailable if a dependency is down.
        (StatusCode::SERVICE_UNAVAILABLE, "Not Ready: Database connection failed")
    }
}

pub fn health_check_routes() -> Router {
    let state = Arc::new(AppState {
        database_healthy: std::sync::atomic::AtomicBool::new(true), // Assume healthy initially
    });

    Router::new()
       .route("/livez", get(liveness_check))
       .route("/readyz", get(readiness_check))
       .with_state(state)
}


This implementation provides a /livez endpoint that will always return 200 OK as long as the web server is running, and a /readyz endpoint that performs a deeper check on the status of its dependencies.

Service Mesh Integration

If the Uveddi platform uses a service mesh like Istio or Linkerd, the mesh provides its own layer of observability and health checking.81 It is important to understand how these layers complement each other:
Mesh Metrics vs. Application Metrics: The service mesh (via its sidecar proxy, e.g., Envoy) automatically generates metrics for all network traffic entering and leaving the pod (e.g., request volume, TCP connection stats, TLS handshake failures).83 These are invaluable for diagnosing network-level issues. Application metrics, which we instrumented earlier, provide the "ground truth" from inside the application (e.g., render queue depth, specific application error counts). A discrepancy between the two can be highly informative; for example, if the mesh reports a high rate of successful requests but the application reports a high error rate, it points to a bug in the application logic rather than a network problem.
Mesh Health Checks: The service mesh can perform its own health checks (outlier detection) and automatically eject unhealthy pods from the load balancing pool. This works in concert with Kubernetes readiness probes. The mesh provides a fast, network-level check, while the application's readiness probe provides a slower, more comprehensive check of internal state.

10. Security and Compliance for Observability Data

Observability data is powerful, but it can also contain sensitive information. A production-ready architecture must include robust controls for security and compliance.
Data Redaction and Masking: It is a strict requirement to prevent Personally Identifiable Information (PII), credentials, or other sensitive data from being exported. This will be enforced at two levels:
Application Level: Developers must be trained to never include sensitive data directly in log messages or as trace attributes. The #[instrument(skip(...))] attribute should be used to prevent sensitive function arguments from being captured.16
Collector Level: The OpenTelemetry Collector can be configured with processors, such as the transform processor, to use regular expressions or other rules to mask or redact specific patterns from logs and trace attributes before they are exported to the backend. This provides a centralized defense-in-depth security control.
Endpoint Security: The observability-related HTTP endpoints (/metrics, /livez, /readyz) should not be exposed to the public internet. They should be bound to a separate port that is only accessible from within the Kubernetes cluster network, controlled by network policies. The main application port serves user traffic, while the internal admin/observability port serves the cluster's operational systems.
Data Retention and Archival Policies: To manage costs and comply with data privacy regulations (like GDPR's right to be forgotten), we will establish clear retention policies for each data type:
Metrics (Prometheus): Retain high-resolution data for 30 days for operational use. For long-term trend analysis, downsampled metrics can be archived to a long-term storage solution like Thanos or VictoriaMetrics.
Logs (Loki): Retain logs in Loki's fast, queryable "hot" storage for 14 days for debugging recent incidents. After 14 days, logs will be automatically flushed to cheaper, long-term object storage (e.g., AWS S3, Google Cloud Storage) where they can be retained for the duration required by compliance (e.g., 1 year) and re-queried if necessary.
Traces (Jaeger/Tempo): Traces are primarily used for debugging active or recent issues and are high-volume. They will be retained in the backend for 7 days.
Audit Trail: All changes to the observability infrastructure's configuration—including Prometheus alerting rules, AlertManager routing, and Grafana dashboards—must be managed via GitOps (i.e., stored in a Git repository and applied via a CI/CD pipeline). This provides a full audit trail of who changed what and when. Furthermore, deployment events will be recorded as annotations in Grafana, allowing the team to easily correlate a change in system performance with a specific new release.

Conclusion and Recommendations

The architecture detailed in this report provides a comprehensive, scalable, and production-ready observability solution for the Uveddi image rendering service. By adopting a unified instrumentation strategy based on the tracing and metrics facades and standardizing on OpenTelemetry, we create a flexible and future-proof system that is decoupled from any single backend vendor. The strategic use of the OpenTelemetry Collector as a central processing hub minimizes application overhead while enabling powerful capabilities like tail-based sampling and centralized data enrichment.
The recommended "PLG" stack—Prometheus, Loki, and Grafana—offers a best-in-class, cost-effective, and tightly integrated solution for metrics, logs, and visualization. This is complemented by Jaeger for deep distributed tracing analysis.
To ensure success, the following actions are recommended:
Adopt the Proposed Architecture: Formally adopt the recommended architecture, including the facade-first instrumentation strategy, the use of the OpenTelemetry Collector, and the PLG backend stack.
Implement in Phases: Roll out the observability features according to the phased roadmap provided in the appendix. Begin with foundational elements like structured logging and health checks, followed by metrics and dashboards, and finally, distributed tracing and advanced alerting.
Define and Monitor SLOs: Immediately ratify and implement the proposed SLIs and SLOs for availability, latency, and quality. These reliability targets should be visualized in a prominent Grafana dashboard and used to drive data-informed engineering priorities via the error budget policy.
Invest in Training: Ensure the development team is trained on the best practices outlined in this document, particularly regarding structured logging, controlled-cardinality metrics, and the use of the #[instrument] macro for effective tracing.
Establish Operational Procedures: Implement the initial alert response playbooks and integrate the management of all observability configurations (Prometheus rules, Grafana dashboards) into the team's existing GitOps workflow to ensure a full audit trail.
By implementing this design, the Uveddi team will gain the deep, real-time visibility necessary to operate the image rendering service with confidence, ensuring high levels of performance, reliability, and security for its users.

Appendix


A. Phased Implementation Roadmap

A phased rollout is recommended to manage complexity and deliver value incrementally.
Phase 1: Foundational Visibility (Sprint 1-2)
Goal: Establish basic logging and health monitoring.
Tasks:
Integrate tracing and tracing-subscriber with JSON output.
Ensure all logs are structured and sent to standard output.
Set up a Loki and Promtail instance to collect logs.
Implement the /livez and /readyz health check endpoints.
Configure Kubernetes liveness and readiness probes.
Outcome: All service logs are centralized and searchable. The service is correctly managed by Kubernetes.
Phase 2: Metrics and Proactive Monitoring (Sprint 3-4)
Goal: Implement comprehensive metrics collection and dashboarding.
Tasks:
Integrate the metrics crate and metrics-exporter-prometheus.
Instrument the application to emit all defined application, business, and operational metrics.
Set up Prometheus to scrape the service.
Define SLIs and SLOs and build the primary Grafana dashboard.
Visualize RED metrics, resource utilization, and key business metrics.
Outcome: Real-time visibility into service performance and health. SLOs are tracked.
Phase 3: Distributed Insights and Alerting (Sprint 5-6)
Goal: Enable end-to-end tracing and automated alerting.
Tasks:
Integrate tracing-opentelemetry and configure the OTel SDK.
Set up the OpenTelemetry Collector with tail-based sampling policies.
Configure Jaeger/Tempo as the trace backend.
Instrument cross-service calls to ensure W3C context propagation to the Node.js service.
Configure Prometheus alerting rules based on SLO burn rates.
Set up AlertManager with routing to Slack and PagerDuty.
Create initial alert response playbooks.
Outcome: Ability to trace requests end-to-end, diagnose complex issues, and receive automated alerts for critical problems.

B. Initial Alert Response Playbook

Alert Name: HighErrorRateBudgetBurn
Severity: Critical
Description: The service is experiencing a high rate of errors, consuming its 30-day error budget at a rate that will lead to SLO violation if not addressed.
Step 1: Triage (Acknowledge and Assess)
Acknowledge the alert in PagerDuty to notify the team you are investigating.
Open the link from the alert notification to the "Uveddi Rendering Service" Grafana dashboard.
Step 2: Investigate (Identify Scope and Cause)
Check the SLO Panel: Confirm which availability SLO is being violated (api_availability or render_success_rate).
Examine RED Metrics: Look at the "API RED Metrics" row.
Is the Rate of requests unusually high (potential overload)?
Which HTTP status codes are causing the Errors? (e.g., 500, 503). Note the distribution.
Correlate with Changes: Check the Grafana annotations for any recent deployments that correlate with the start of the alert.
Dive into Logs: Use the dashboard's time range to jump to Loki. Filter for logs with level="error" during the incident window. Look for common error messages or stack traces.
Examine Traces: If logs are inconclusive, use an exemplar trace_id from a failed request (found in Prometheus or logs) to view the full trace in Jaeger. Identify which service or specific span is failing and what the associated error message is.
Step 3: Remediate (Mitigate and Resolve)
If caused by a recent deployment: Initiate a rollback to the previous stable version.
If caused by a downstream dependency failure: Check the status of the dependency. If possible, engage the team responsible for that service.
If caused by overload: Consider scaling up the number of service replicas if autoscaling has not already triggered.
If a specific endpoint is failing: Consider using a feature flag or configuration to temporarily disable the problematic feature if it is non-critical.
Step 4: Follow-up
Once the incident is resolved, create a post-mortem ticket.
Document the root cause, the actions taken, and any follow-up work needed to prevent recurrence (e.g., adding a new metric, improving a health check, fixing a bug).

C. Consolidated Cargo.toml and Configuration Files


Cargo.toml Dependencies


Ini, TOML


[dependencies]
# Core Async Runtime
tokio = { version = "1", features = ["full"] }

# Web Framework (example: Axum)
axum = "0.7"
http = "1.0"

# Metrics Facade and Prometheus Exporter
metrics = "0.21"
metrics-exporter-prometheus = "0.12"

# Logging and Tracing Facade
tracing = { version = "0.1", features = ["attributes"] }
tracing-subscriber = { version = "0.3", features = ["json", "registry", "env-filter"] }

# OpenTelemetry Integration
opentelemetry = { version = "0.20", features = ["rt-tokio"] }
opentelemetry-otlp = { version = "0.13", features = ["tonic"] }
opentelemetry_sdk = { version = "0.20", features = ["rt-tokio"] }
tracing-opentelemetry = "0.21"

# System Info for Resource Metrics
sysinfo = "0.29"



OpenTelemetry Collector Configuration (otel-collector-config.yml)


YAML


receivers:
  otlp:
    protocols:
      grpc:
      http:

processors:
  batch:
  # Tail-based sampling configuration [10]
  tail_sampling:
    decision_wait: 30s
    num_traces: 50000
    policies:
      }
        },
        # Policy 2: Sample 10% of all other traces
        {
          name: "probabilistic-policy",
          type: "probabilistic",
          probabilistic: { sampling_percentage: 10 }
        }
      ]

exporters:
  prometheus:
    endpoint: "0.0.0.0:8889"
  loki:
    endpoint: "http://loki:3100/loki/api/v1/push"
  jaeger:
    endpoint: "jaeger-collector:14250"
    tls:
      insecure: true

service:
  pipelines:
    traces:
      receivers: [otlp]
      processors: [batch, tail_sampling]
      exporters: [jaeger]
    metrics:
      receivers: [otlp]
      processors: [batch]
      exporters: [prometheus]
    logs:
      receivers: [otlp]
      processors: [batch]
      exporters: [loki]


Works cited
metrics - crates.io: Rust Package Registry, accessed July 9, 2025, https://crates.io/crates/metrics
metrics-rs/metrics: A metrics ecosystem for Rust. - GitHub, accessed July 9, 2025, https://github.com/metrics-rs/metrics
Getting Started with OpenTelemetry in Rust - Last9, accessed July 9, 2025, https://last9.io/blog/opentelemetry-in-rust/
Rust | OpenTelemetry, accessed July 9, 2025, https://opentelemetry.io/docs/languages/rust/
What is Prometheus? | New Relic, accessed July 9, 2025, https://newrelic.com/blog/best-practices/what-is-prometheus
Overview - Prometheus, accessed July 9, 2025, https://prometheus.io/docs/introduction/overview/
Monitoring & Logging with Prometheus, Grafana, ELK, and Loki (2025 Guide for DevOps), accessed July 9, 2025, https://www.refontelearning.com/blog/monitoring-logging-prometheus-grafana-elk-stack-loki
DevOps Made Simple: A Beginner's Guide to Log Management in DevOps - ELK Stack vs Loki Stack - DEV Community, accessed July 9, 2025, https://dev.to/yash_sonawane25/devops-made-simple-a-beginners-guide-to-log-management-in-devops-elk-stack-vs-loki-stack-2f8
Monitoring Microservices using Prometheus & Grafana | Orkes Platform, accessed July 9, 2025, https://orkes.io/blog/monitoring-microservices-using-prometheus-and-grafana/
Tail Sampling with OpenTelemetry: Why it's useful, how to do it, and what to consider, accessed July 9, 2025, https://opentelemetry.io/blog/2022/tail-sampling/
Ingestion Sampling with OpenTelemetry - Datadog Docs, accessed July 9, 2025, https://docs.datadoghq.com/opentelemetry/ingestion_sampling/
Context propagation - OpenTelemetry, accessed July 9, 2025, https://opentelemetry.io/docs/concepts/context-propagation/
OpenTelemetry Context Propagation Explained | Better Stack Community, accessed July 9, 2025, https://betterstack.com/community/guides/observability/otel-context-propagation/
Sampling in OpenTelemetry: A Beginner's Guide | Better Stack Community, accessed July 9, 2025, https://betterstack.com/community/guides/observability/opentelemetry-sampling/
Crate metrics - Rust, accessed July 9, 2025, https://prisma.github.io/prisma-engines/doc/metrics/index.html
Getting Started with Tracing in Rust - shuttle.dev, accessed July 9, 2025, https://www.shuttle.dev/blog/2024/01/09/getting-started-tracing-rust
Getting Started - OpenTelemetry, accessed July 9, 2025, https://opentelemetry.io/docs/languages/rust/getting-started/
How to monitor your Rust applications with OpenTelemetry - Datadog, accessed July 9, 2025, https://www.datadoghq.com/blog/monitor-rust-otel/
metrics_prometheus - Rust - Docs.rs, accessed July 9, 2025, https://docs.rs/metrics-prometheus
instrumentisto/metrics-prometheus-rs: `prometheus` backend for `metrics` crate - GitHub, accessed July 9, 2025, https://github.com/instrumentisto/metrics-prometheus-rs
prometheus - Rust - Docs.rs, accessed July 9, 2025, https://docs.rs/prometheus
prometheus-client - crates.io: Rust Package Registry, accessed July 9, 2025, https://crates.io/crates/prometheus-client
Rust Service Logs/Metrics - Reddit, accessed July 9, 2025, https://www.reddit.com/r/rust/comments/1hgkmk6/rust_service_logsmetrics/
Using Prometheus metrics in a Rust web service - LogRocket Blog, accessed July 9, 2025, https://blog.logrocket.com/using-prometheus-metrics-in-a-rust-web-service/
metrics_exporter_prometheus - Rust - Docs.rs, accessed July 9, 2025, https://docs.rs/metrics-exporter-prometheus/
metrics-exporter-prometheus - crates.io: Rust Package Registry, accessed July 9, 2025, https://crates.io/crates/metrics-exporter-prometheus/0.16.1
metrics-exporter-prometheus - crates.io: Rust Package Registry, accessed July 9, 2025, https://crates.io/crates/metrics-exporter-prometheus
Instrumentation | Prometheus, accessed July 9, 2025, https://prometheus.io/docs/practices/instrumentation/
Let's build a custom prometheus exporter in Rust | by Tanisha Banik - Medium, accessed July 9, 2025, https://26tanishabanik.medium.com/lets-build-a-custom-prometheus-exporter-in-rust-ed7f16294278
Defining slo: service level objective meaning - Google SRE, accessed July 9, 2025, https://sre.google/sre-book/service-level-objectives/
SLA vs. SLI vs. SLO: Understanding Service Levels - Splunk, accessed July 9, 2025, https://www.splunk.com/en_us/blog/learn/sla-vs-sli-vs-slo.html
SLA vs. SLO vs. SLI: What's the Difference? | UptimeRobot Blog, accessed July 9, 2025, https://uptimerobot.com/blog/sla-slo-sli/
What's the Difference Between SLAs, SLOs and SLIs? - PagerDuty, accessed July 9, 2025, https://www.pagerduty.com/resources/digital-operations/learn/what-is-slo-sla-sli/
SRE Fundamentals: SLA vs SLO vs SLI - Chronosphere, accessed July 9, 2025, https://chronosphere.io/learn/know-the-sre-fundamentals-differences-between-sli-vs-slo-vs-sla/
Choosing Effective SLIs | Last9, accessed July 9, 2025, https://last9.io/blog/choosing-effective-slis/
Programmatically assessing image quality [closed] - Photography Stack Exchange, accessed July 9, 2025, https://photo.stackexchange.com/questions/96901/programmatically-assessing-image-quality
image-rendering - CSS - MDN Web Docs, accessed July 9, 2025, https://developer.mozilla.org/en-US/docs/Web/CSS/image-rendering
Slow rendering | App quality - Android Developers, accessed July 9, 2025, https://developer.android.com/topic/performance/vitals/render
Creating a service-level indicator | Google Cloud Observability, accessed July 9, 2025, https://cloud.google.com/stackdriver/docs/solutions/slo-monitoring/api/identifying-custom-sli
Service Level Objectives - Datadog Docs, accessed July 9, 2025, https://docs.datadoghq.com/service_management/service_level_objectives/
Alerting rules | Prometheus, accessed July 9, 2025, https://prometheus.io/docs/prometheus/latest/configuration/alerting_rules/
Alertmanager | Prometheus, accessed July 9, 2025, https://prometheus.io/docs/alerting/latest/alertmanager/
Effective Alerting with Prometheus Alertmanager | Better Stack ..., accessed July 9, 2025, https://betterstack.com/community/guides/monitoring/prometheus-alertmanager/
Prometheus Alert Manager integration with Slack - Hewlett Packard Enterprise Community, accessed July 9, 2025, https://community.hpe.com/t5/software-general/prometheus-alert-manager-integration-with-slack/td-p/7240761
Prometheus Alert Manager Setup and Alert Configurations (Slack) | by Krishabh - Medium, accessed July 9, 2025, https://medium.com/@krishabh080/prometheus-alert-manager-setup-and-alert-configurations-slack-800f6bb5111e
Integrating Prometheus AlertManager with PagerDuty in Calico - Tigera, accessed July 9, 2025, https://www.tigera.io/blog/deep-dive/integrating-prometheus-alertmanager-with-pagerduty-in-calico/
Prometheus Integration Guide - PagerDuty, accessed July 9, 2025, https://www.pagerduty.com/docs/guides/prometheus-integration-guide/
Step-by-step guide to setting up Prometheus Alertmanager with Slack, PagerDuty, and Gmail - Grafana, accessed July 9, 2025, https://grafana.com/blog/2020/02/25/step-by-step-guide-to-setting-up-prometheus-alertmanager-with-slack-pagerduty-and-gmail/
Monitoring using Prometheus, Grafana, Alertmanager and Pagerduty | by Sanjal S Eralil, accessed July 9, 2025, https://medium.com/@sanjaleralil1/monitoring-using-prometheus-grafana-alertmanager-and-pagerduty-a34b4e6d475e
Grafana dashboard best practices, accessed July 9, 2025, https://grafana.com/docs/grafana/latest/dashboards/build-dashboards/best-practices/
Getting started with Grafana: best practices to design your first dashboard, accessed July 9, 2025, https://grafana.com/blog/2024/07/03/getting-started-with-grafana-best-practices-to-design-your-first-dashboard/
Grafana dashboards — best practices and dashboards-as-code – Blog - Andreas Sommer, accessed July 9, 2025, https://andidog.de/blog/2022-04-21-grafana-dashboards-best-practices-dashboards-as-code
The Top 30 Grafana Dashboard Examples - Logit.io, accessed July 9, 2025, https://logit.io/blog/post/top-grafana-dashboards-and-visualisations/
Spinnaker Microservices | Grafana Labs, accessed July 9, 2025, https://grafana.com/grafana/dashboards/11219-spinnaker-microservices/
Rocket.Chat MicroService Metrics | Grafana Labs, accessed July 9, 2025, https://grafana.com/grafana/dashboards/23427-microservice-metrics/
tracing_subscriber - Rust - Docs.rs, accessed July 9, 2025, https://docs.rs/tracing-subscriber
Json in tracing_subscriber::fmt::format - Rust - Docs.rs, accessed July 9, 2025, https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/format/struct.Json.html
Format in tracing_subscriber::fmt::format - Rust - Docs.rs, accessed July 9, 2025, https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/format/struct.Format.html
json-subscriber - crates.io: Rust Package Registry, accessed July 9, 2025, https://crates.io/crates/json-subscriber
Grafana Loki vs ELK Logging Stacks - Wallarm, accessed July 9, 2025, https://www.wallarm.com/cloud-native-products-101/grafana-loki-vs-elk-logging-stacks
Loki vs. Elasticsearch: Choosing the Right Logging System for You - KubeBlogs, accessed July 9, 2025, https://www.kubeblogs.com/loki-vs-elasticsearch/
Loki vs Elasticsearch - Which tool to choose for Log Analytics? - SigNoz, accessed July 9, 2025, https://signoz.io/blog/loki-vs-elasticsearch/
ELK VS Loki! How to gather logs from Kubernetes cluster and effectively navigate through them in 2023 | IT Svit, accessed July 9, 2025, https://itsvit.com/blog/elk-vs-loki-how-to-gather-logs-from-kubernetes-cluster-and-effectively-navigate-through-them/
Getting Started with Jaeger for Distributed Tracing - Last9, accessed July 9, 2025, https://last9.io/blog/jaeger-for-distributed-tracing/
tracing-opentelemetry - crates.io: Rust Package Registry, accessed July 9, 2025, https://crates.io/crates/tracing-opentelemetry
tracing_opentelemetry - Rust - Docs.rs, accessed July 9, 2025, https://docs.rs/tracing-opentelemetry
opentelemetry-rust/examples/tracing-jaeger/src/main.rs at main - GitHub, accessed July 9, 2025, https://github.com/open-telemetry/opentelemetry-rust/blob/main/examples/tracing-jaeger/src/main.rs
Simple OpenTelemetry logger in Rust | by Kosta Malsev - Medium, accessed July 9, 2025, https://kostya-malsev.medium.com/simple-opentelemetry-logger-in-rust-999d75864aba
OpenTelemetry Examples with Rust - Tech with Tirslo, accessed July 9, 2025, https://tirslo.hashnode.dev/opentelemetry-examples-with-rust
Dude, where's my error? How OpenTelemetry records errors, accessed July 9, 2025, https://opentelemetry.io/blog/2024/otel-errors/
OpenTelemetry Context Propagation for Better Tracing - Last9, accessed July 9, 2025, https://last9.io/blog/opentelemetry-context-propagation/
opentelemetry/instrumentation-express - NPM, accessed July 9, 2025, https://www.npmjs.com/package/@opentelemetry/instrumentation-express
OpenTelemetry for Full-Stack JavaScript | New Relic, accessed July 9, 2025, https://newrelic.com/blog/how-to-relic/opentelemetry-full-stack-javascript
Getting Started - Jaeger, accessed July 9, 2025, https://www.jaegertracing.io/docs/1.70/getting-started/
Getting Started - Jaeger, accessed July 9, 2025, https://www.jaegertracing.io/docs/1.25/getting-started/
Liveness, Readiness, and Startup Probes - Kubernetes, accessed July 9, 2025, https://kubernetes.io/docs/concepts/configuration/liveness-readiness-startup-probes/
Configure Liveness, Readiness and Startup Probes - Kubernetes, accessed July 9, 2025, https://kubernetes.io/docs/tasks/configure-pod-container/configure-liveness-readiness-startup-probes/
Guide to Understanding Your Kubernetes Liveness Probes Best Practices - Fairwinds, accessed July 9, 2025, https://www.fairwinds.com/blog/a-guide-to-understanding-kubernetes-liveness-probes-best-practices
Kubernetes Liveness Probes: A Complete Guide - Qovery, accessed July 9, 2025, https://www.qovery.com/blog/kubernetes-liveness-probes-a-complete-guide/
Pattern: Health Check API - Microservices.io, accessed July 9, 2025, https://microservices.io/patterns/observability/health-check-api.html
Istio / Observability, accessed July 9, 2025, https://istio.io/latest/docs/tasks/observability/
Service Mesh in Kubernetes: A Comparison of Istio and Linkerd | by Yasinkartal | Medium, accessed July 9, 2025, https://medium.com/@yasinkartal2009/service-mesh-in-kubernetes-istio-and-linkerd-f4865a9bcc86
Istio vs Linkerd Service Mesh Technologies - Wallarm, accessed July 9, 2025, https://www.wallarm.com/cloud-native-products-101/istio-vs-linkerd-service-mesh-technologies
Linkerd vs. Istio: 7 Key Differences - Solo.io, accessed July 9, 2025, https://www.solo.io/topics/linkerd/linkerd-vs-istio

