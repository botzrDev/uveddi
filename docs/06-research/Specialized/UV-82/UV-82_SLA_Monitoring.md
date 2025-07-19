
Uveddi Project: A Site Reliability Engineering Blueprint for SLA Monitoring and Validation


The Uveddi SLA Framework: From Business Goals to Service Level Objectives

The foundation of a modern reliability practice is the shift from traditional, often adversarial Service Level Agreements (SLAs) to a collaborative, data-driven framework centered on Service Level Objectives (SLOs) and Error Budgets.1 This approach, pioneered by Google's Site Reliability Engineering (SRE) teams, reframes reliability as a product feature that must be aligned with user expectations. It provides a quantitative basis for making informed engineering decisions, balancing the velocity of new feature development against the critical need for a stable and performant service.4

The SRE Paradigm Shift: Why SLOs and Error Budgets Matter

The core principle of SRE is that 100% reliability is not only an impossible target but also the wrong one.6 Striving for perfect reliability inhibits innovation, slows down deployments, and leads to overly complex and expensive systems. Instead, reliability should be defined from the user's perspective and measured against carefully chosen objectives. This paradigm is built on four key concepts:
Service Level Indicators (SLIs): These are the direct, quantitative measurements of a service's performance. An SLI is a time series representing a specific aspect of the service, such as the latency of a request or the success rate of an operation.4
Service Level Objectives (SLOs): An SLO is a target value or range of values for an SLI, measured over a specific period. For example, an SLO might state that 99.9% of API requests should complete in under 100 milliseconds over a rolling 28-day window. SLOs set clear, unambiguous expectations for service performance and are the cornerstone of SRE's approach to reliability.7
Service Level Agreements (SLAs): An SLA is a formal or informal contract with users that outlines the expected level of service and often includes consequences, such as financial credits, for failing to meet the defined SLOs.2 From an SRE perspective, the primary goal is to meet the SLOs, which in turn ensures that the terms of the SLA are not breached.7
Error Budgets: The error budget is the mathematical inverse of the SLO and represents the acceptable level of unreliability for the service over the SLO's time window. For an availability SLO of 99.9%, the error budget is 0.1%. This budget is a critical management tool; when the service is operating well within its budget, the development team is empowered to release new features and take calculated risks. When the budget is being consumed too quickly, it serves as a clear, data-driven signal to halt new releases and prioritize reliability work until the service is stable again.4

Defining the Uveddi User Journey and Critical Services

To define meaningful SLIs and SLOs, it is essential to first understand what users care about and how they interact with the system.7 The Uveddi project has two primary user journeys that define its critical service paths:
The Code Analysis Journey: A developer or an automated CI system submits a codebase to Uveddi. The Rust backend microservices perform static analysis, process the results, and store them in the SQLite database. The success and speed of this journey are critical for developer workflow integration.
The Architectural Visualization Journey: A user interacts with the TypeScript frontend to request a visualization of their code's architecture. The Rust backend retrieves the necessary analysis data, which is then passed to the Node.js rendering service to generate a Mermaid diagram. The resulting diagram is then sent back to the frontend for display. The responsiveness and reliability of this journey directly impact the interactive user experience.
Based on these journeys, two critical, user-facing services are identified for SLO definition:
The Analysis Service: The collection of Rust microservices responsible for core static code analysis.
The Rendering Service: The Node.js service responsible for generating Mermaid diagrams.

Establishing Uveddi's Service Level Indicators (SLIs)

Following SRE best practices for user-facing systems, SLIs should focus on availability, latency, quality, and throughput.7
Availability SLIs: This indicator measures the proportion of valid requests that are served successfully. It is a fundamental measure of whether the service is usable. The SLI will be calculated as the ratio of successful requests (HTTP status codes other than 5xx) to the total number of valid requests. Client-side errors (4xx status codes) are excluded as they do not represent a failure of the service itself.
Latency SLIs: This indicator measures the time it takes for the service to process a request. Averages are a poor measure of latency as they can hide significant performance degradation for a subset of users (tail latency). Therefore, latency will be measured as a distribution using histograms, allowing for the calculation of percentiles (e.g., P50, P95, P99). This approach provides a much more accurate picture of the user experience across the entire user base.
Quality SLIs: This indicator is the inverse of availability, measuring the proportion of valid requests that result in a server-side error (HTTP 5xx status codes). A low error rate is a direct measure of the service's correctness and stability.
Throughput SLIs: This indicator measures the rate of requests handled by the system, typically in requests per second (RPS). While not a direct measure of user happiness, throughput is a critical indicator for capacity planning, load testing, and identifying saturation points in the system.7

The Uveddi SLOs and Error Budgets

The defined SLOs serve as the initial targets for the Uveddi project. These are not static; they should be reviewed and refined periodically as more performance data is collected and user expectations are better understood.6
A crucial observation from the project's current performance baselines is the significant disparity between the average rendering latency (4.2ms) and the P99 latency (77ms). This gap highlights a classic tail latency problem, where a small fraction of users experience performance that is an order of magnitude worse than the average user. Relying on average latency as a primary metric would mask this poor experience. User satisfaction is often more strongly correlated with the predictability and consistency of a service's performance than its raw speed under ideal conditions.7 A consistently 50ms response is preferable to a service that is typically 5ms but occasionally takes 500ms.
This reality dictates that the latency SLOs for Uveddi must be defined in terms of high-level percentiles, specifically P99 for the user-facing rendering service. This forces the engineering team to focus on and resolve the more complex, intermittent issues that contribute to poor tail latency, ultimately resulting in a more robust and equitable experience for all users. This decision has a direct technical implication: the monitoring instrumentation must use histogram-based metrics to capture the full distribution of latency values, a topic that will be detailed in Section 3.
The following table establishes the foundational SLIs and SLOs for the Uveddi project. This table is the cornerstone of the entire monitoring strategy, providing a single, unambiguous source of truth for what "good performance" means. It translates abstract business goals into concrete, measurable engineering targets, forming the basis for the error budget policy and all subsequent alerting and CI gating rules. It is the primary tool for aligning the objectives of product, development, and SRE teams.5
Table 1: Uveddi SLI/SLO Definitions
Service
SLI Type
SLI Definition (PromQL Metric)
SLO Target (28-day rolling window)
Error Budget (28 days)
Business Justification
Analysis Service
Availability
$sum(rate(uveddi_analysis_requests_total{status_code!~"5.."}[5m])) / sum(rate(uveddi_analysis_requests_total[5m]))$
99.9%
~43.8 minutes downtime
The core functionality of Uveddi. Must be highly reliable for integration into developer workflows and CI/CD pipelines, where downtime can block entire teams.


Latency (P95)
$histogram_quantile(0.95, sum(rate(uveddi_analysis_duration_seconds_bucket[5m])) by (le))$
< 500ms
N/A
Developers expect fast feedback from analysis tools. Latency above this threshold disrupts the development "flow state" and reduces productivity.


Quality (Error Rate)
$sum(rate(uveddi_analysis_requests_total{status_code=~"5.."}[5m])) / sum(rate(uveddi_analysis_requests_total[5m]))$
< 0.1%
1 in 1000 requests can fail
Errors in the analysis service erode developer trust in the tool and can lead to false negatives in CI pipelines, compromising code quality.
Rendering Service
Availability
$sum(rate(uveddi_render_requests_total{status_code!~"5.."}[5m])) / sum(rate(uveddi_render_requests_total[5m]))$
99.95%
~21.9 minutes downtime
The visualization feature is a critical, highly visible, interactive part of the user experience. Higher availability is expected to maintain user engagement and satisfaction.


Latency (P99)
$histogram_quantile(0.99, sum(rate(uveddi_render_duration_seconds_bucket[5m])) by (le))$
< 50ms
N/A
Aligned with the project's stated performance target. Tracking P99 ensures that even the worst-case user experiences remain within an acceptable, interactive threshold.


Quality (Error Rate)
$sum(rate(uveddi_render_requests_total{status_code=~"5.."}[5m])) / sum(rate(uveddi_render_requests_total[5m]))$
< 0.05%
1 in 2000 requests can fail
A broken or failed visualization is a highly visible failure that directly impacts the user's perception of the tool's quality and reliability.


Architectural Blueprint for Real-Time Monitoring

This section outlines the technical architecture for the real-time monitoring and alerting system. The design prioritizes industry-standard, open-source tools that integrate seamlessly with the existing Uveddi technology stack and provide a scalable foundation for future growth.

System Overview and Tooling Justification

The proposed monitoring architecture is composed of a tightly integrated stack of best-in-class open-source tools. A high-level diagram would show the Uveddi services exposing metrics, a Prometheus server scraping and storing these metrics, an Alertmanager instance processing and routing alerts, and a Grafana instance providing visualization and dashboarding capabilities.
The selection of each component is justified as follows:
Prometheus: As a graduated project of the Cloud Native Computing Foundation, Prometheus has become the de-facto standard for monitoring in modern, cloud-native environments.11 Its powerful PromQL query language enables sophisticated analysis and alerting, and its pull-based scraping model is exceptionally well-suited for dynamic environments with ephemeral services.12 Crucially, it has excellent support within the Rust ecosystem through mature client libraries and exporters, making it a natural fit for Uveddi.13
Grafana: Grafana is the premier open-source platform for visualizing and analyzing time-series data.14 Its ability to connect to Prometheus as a data source and create rich, interactive, and shareable dashboards is unparalleled. Grafana will serve as the primary interface for observing Uveddi's SLO compliance, analyzing performance trends, and conducting initial incident triage.15
Alertmanager: Alertmanager is a core component of the Prometheus ecosystem, designed to handle alerts fired by the Prometheus server. Its key responsibilities are to deduplicate, group, silence, inhibit, and route alerts to the correct notification channels.16 This functionality is absolutely critical for implementing a sane alerting strategy and preventing the "alert fatigue" that plagues many operations teams.17

Data Flow and Integration Points

The flow of telemetry data through the system follows a clear, logical path from instrumentation to action:
Instrumentation: The Uveddi Rust microservices and the Node.js rendering service will be instrumented using their respective client libraries. This instrumentation will expose a standard /metrics HTTP endpoint on each service instance, presenting real-time performance data in the OpenMetrics format.
Scraping: The central Prometheus server is configured with a list of "scrape targets," which are the /metrics endpoints of the Uveddi services. Prometheus will periodically send an HTTP GET request to these endpoints to "pull" the latest metric values.
Storage: Upon successful scraping, Prometheus ingests the data into its highly efficient local time-series database (TSDB). The data is stored with labels that provide dimensional context (e.g., service name, instance, HTTP status code).
Querying & Visualization: A Grafana instance is configured with Prometheus as a data source. Users and SREs interact with Grafana dashboards, which execute PromQL queries against the Prometheus TSDB in real-time to generate graphs, gauges, and tables visualizing the health and SLO compliance of the Uveddi services.
Alerting: The Prometheus server continuously evaluates a set of configured alerting rules (written in PromQL) against the data in its TSDB. When an expression in an alerting rule evaluates to true for a certain duration, Prometheus enters a "firing" state for that alert and sends it to the configured Alertmanager instance.
Notification: Alertmanager receives the alert from Prometheus. It applies its own set of routing, grouping, and inhibition rules to the incoming alert. Based on these rules, it may group the alert with others, suppress it if a higher-level alert is already firing, or route it to a specific notification receiver, such as a Slack channel or a PagerDuty service.

Scalability and Data Retention Strategy

While a single Prometheus instance is sufficient for the initial implementation, a forward-looking architecture must account for future growth. A static code analysis tool like Uveddi has the potential to generate high-cardinality metrics, especially if metrics are labeled by dimensions such as repository, file path, function name, or user ID. High cardinality can strain the resources of a single Prometheus node, leading to increased disk usage, slower query performance, and reduced ingestion capacity.
The open-source landscape offers solutions designed to address these challenges. VictoriaMetrics, for example, is a time-series database known for its high performance, resource efficiency, and superior scalability in high-cardinality environments.12 It is designed as a drop-in replacement for Prometheus in many contexts and is fully compatible with the Prometheus data model and query language.20
This compatibility is a key architectural advantage. By instrumenting the Uveddi services with standard Prometheus client libraries and exposing data in the OpenMetrics format, the system is not tightly coupled to a specific TSDB implementation. This creates a "pluggable" architecture where the storage backend can be migrated from a single-node Prometheus instance to a horizontally scalable, clustered VictoriaMetrics setup in the future. Such a migration would require minimal to no changes in the application code or Grafana dashboards, providing a clear and low-risk path to scaling the monitoring infrastructure as the Uveddi project grows.
Recommendation:
The implementation will begin with a single Prometheus node. A data retention policy of 30 to 90 days should be configured, providing a reasonable balance between the need for historical trend analysis and the management of disk space. The architectural design will explicitly document the potential for future migration to a scalable TSDB like VictoriaMetrics or a managed service (e.g., Grafana Cloud, Amazon Managed Service for Prometheus) should metric volume and cardinality exceed the capacity of a single node.

Implementation Guide: Instrumenting Uveddi for Deep Observability

This section provides specific, actionable code and configuration for instrumenting the Uveddi services. The implementation will build upon the existing src/monitoring/metrics.rs file, transforming it into a comprehensive telemetry source that directly supports the SLOs defined in Section 1.

Instrumenting the Rust Backend with metrics and tracing

The Rust backend will be instrumented using the metrics crate as a lightweight facade, which abstracts the specifics of the metrics backend.21 The
metrics-exporter-prometheus crate will be used to provide a concrete implementation that exposes a Prometheus-compatible /metrics scrape endpoint.23 This combination represents the standard, community-supported approach for Prometheus instrumentation in Rust applications.26
Code Example: Setting up the Prometheus Exporter in main.rs
The Prometheus exporter should be initialized once at application startup. This will spawn a background task that listens on a dedicated port for scrape requests from the Prometheus server.

Rust


// In src/main.rs
use metrics_exporter_prometheus::PrometheusBuilder;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize your logger/tracing subscriber here
    //...

    // Register the custom metrics defined in the monitoring module
    uveddi::monitoring::metrics::register_uveddi_metrics();

    // Configure the Prometheus exporter to listen on port 9090
    let builder = PrometheusBuilder::new();
    let addr: SocketAddr = (, 9090).into();
    builder
       .with_http_listener(addr)
       .install()
       .expect("Failed to install Prometheus exporter.");

    println!("Prometheus exporter listening on http://{}", addr);

    // Your existing application startup logic...
    // For example, starting the Axum or Actix-Web server
    //...
    
    Ok(())
}


Code Example: Defining and Using Custom Metrics
The metrics that directly map to the defined SLIs will be registered and described. This example demonstrates the creation of the core metrics for the Analysis Service. Using the describe_*! macros is a best practice that provides context and help text for the metrics when they are scraped by Prometheus.21

Rust


// In src/monitoring/metrics.rs
use metrics::{counter, gauge, histogram, describe_counter, describe_histogram, describe_gauge, Unit};

// A central function to register and describe all application metrics.
// This should be called once at application startup.
pub fn register_uveddi_metrics() {
    // --- Analysis Service Metrics ---
    describe_counter!("uveddi_analysis_requests_total", "Total number of analysis requests processed, partitioned by status code and endpoint.");
    describe_histogram!(
        "uveddi_analysis_duration_seconds",
        Unit::Seconds,
        "The analysis request latency in seconds."
    );

    // --- Rendering Service Metrics (if Rust backend proxies or tracks them) ---
    describe_counter!("uveddi_render_requests_total", "Total number of rendering requests processed, partitioned by status code.");
    describe_histogram!(
        "uveddi_render_duration_seconds",
        Unit::Seconds,
        "The diagram rendering request latency in seconds."
    );

    // --- Resource Metrics ---
    describe_gauge!("uveddi_process_cpu_seconds_total", Unit::Seconds, "Total user and system CPU time spent in seconds.");
    describe_gauge!("uveddi_process_resident_memory_bytes", Unit::Bytes, "Resident memory size in bytes.");
}

// Example of instrumenting a web handler (e.g., in an Axum or Actix-Web service)
// This pattern can be encapsulated in middleware for cleaner code.
pub async fn handle_analysis_request(req: Request) -> Response {
    let start = std::time::Instant::now();
    
    // The actual business logic of the request handler
    let (status_code, response) = process_analysis_request(req).await;
    
    let duration = start.elapsed().as_secs_f64();
    let status_code_str = status_code.to_string();

    // Record the latency in the histogram
    histogram!("uveddi_analysis_duration_seconds").record(duration);
    
    // Increment the request counter with the status_code as a label
    counter!("uveddi_analysis_requests_total", "status_code" => status_code_str).increment(1);
    
    response
}


Integrating tracing for Correlated Observability
While the primary focus is on SLA metrics, integrating the tracing crate provides invaluable context for debugging performance issues. By using tracing-opentelemetry, spans can be exported to a distributed tracing system, and crucially, trace_ids can be injected into logs.27 This allows engineers to correlate a specific slow request (seen in metrics) with the exact sequence of log events and operations that contributed to its latency. The
metrics-tracing-context crate can further enrich this by automatically adding span fields as labels to metrics recorded within that span's context.23 This provides a powerful link between the "what" (metrics) and the "why" (logs and traces).

Monitoring the Node.js Rendering Service

The Node.js rendering service must also be instrumented to provide its own performance metrics. The prom-client library is the de-facto standard for this purpose in the Node.js ecosystem.
Code Example: Exposing Metrics from the Node.js Service
This example sets up an Express.js server, defines the necessary metrics for the Rendering Service SLOs, instruments the rendering endpoint, and exposes the /metrics endpoint for Prometheus to scrape.

JavaScript


// In the main file of the Node.js rendering service (e.g., server.js)
const client = require('prom-client');
const express = require('express');

const server = express();
const register = new client.Registry();

// Enable default metrics like process and event loop stats
client.collectDefaultMetrics({ register });

// Define custom metrics for the rendering service
const renderRequestsTotal = new client.Counter({
  name: 'uveddi_render_requests_total',
  help: 'Total number of rendering requests processed.',
  labelNames: ['status_code'],
  registers: [register],
});

const renderDurationSeconds = new client.Histogram({
  name: 'uveddi_render_duration_seconds',
  help: 'The Mermaid diagram rendering latency in seconds.',
  // Buckets in seconds, tailored for fast rendering times
  buckets: [0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5],
  registers: [register],
});

// Middleware to parse JSON bodies
server.use(express.json());

// The core rendering endpoint
server.post('/render', async (req, res) => {
  const end = renderDurationSeconds.startTimer();
  let statusCode = 200;

  try {
    //... your Mermaid diagram rendering logic here...
    const svgOutput = await renderMermaid(req.body.diagram);
    res.type('svg').send(svgOutput);
  } catch (error) {
    statusCode = 500;
    console.error('Rendering failed:', error);
    res.status(500).send('Internal Server Error');
  } finally {
    end();
    renderRequestsTotal.inc({ status_code: statusCode });
  }
});

// Expose the /metrics endpoint for Prometheus scraping
server.get('/metrics', async (req, res) => {
  try {
    res.set('Content-Type', register.contentType);
    res.end(await register.metrics());
  } catch (ex) {
    res.status(500).end(ex);
  }
});

const port = 8081;
server.listen(port, () => {
  console.log(`Node.js rendering service listening on port ${port}`);
  console.log(`Metrics available at http://localhost:${port}/metrics`);
});



Prometheus Configuration (prometheus.yml)

The Prometheus server needs to be configured to discover and scrape the metrics endpoints exposed by the Uveddi services. For this architecture, a simple static configuration is sufficient.
Code Example: prometheus.yml
This configuration file tells Prometheus to scrape two jobs: the Rust backend and the Node.js renderer, every 15 seconds.

YAML


# prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'uveddi-rust-backend'
    static_configs:
      - targets: ['host.docker.internal:9090'] # Use host.docker.internal if Prometheus is in Docker
        labels:
          service: 'analysis-service'

  - job_name: 'uveddi-node-renderer'
    static_configs:
      - targets: ['host.docker.internal:8081']
        labels:
          service: 'rendering-service'



Automated Performance Gating in CI/CD

Integrating performance validation directly into the Continuous Integration/Continuous Deployment (CI/CD) pipeline is a cornerstone of the "shift left" philosophy. This practice transforms the CI pipeline from a simple build-and-test tool into a proactive guardian of production performance. By catching performance regressions before code is merged into the main branch, it provides developers with immediate, actionable feedback and prevents the gradual erosion of user experience.30

The "Shift Left" Philosophy of Performance Gating

The goal of a performance gate is to automate the process of asking, "Does this change make the service unacceptably slower or less reliable?" for every single pull request. This automated check serves as a critical quality gate, ensuring that performance is treated as a first-class feature of the Uveddi project, on par with correctness and security.

GitHub Actions Workflow for Performance Testing

A GitHub Actions workflow will be created to orchestrate this process. The workflow will be triggered on every pull request that targets the main branch. It will build the application, run it in a controlled environment, execute a standardized load test against it, and then analyze the results for regressions.
For the load testing step, a tool like Artillery is highly recommended due to its powerful scripting capabilities, clear reporting, and seamless integration with CI/CD environments.32 The user's existing performance testing framework can be adapted to fit this workflow.
Code Example: performance-gate.yml GitHub Actions Workflow
This workflow defines the steps for building the Uveddi application from the pull request, running a performance test, and then executing a script to check for regressions.

YAML


#.github/workflows/performance-gate.yml
name: Performance Regression Gate

on:
  pull_request:
    branches: [ main ]

jobs:
  performance-test:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout PR code
        uses: actions/checkout@v4

      - name: Set up Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Cache dependencies
        uses: Swatinem/rust-cache@v2

      - name: Build application
        run: cargo build --release

      - name: Run Uveddi application in background
        run:./target/release/uveddi &

      - name: Wait for application to start
        run: sleep 10 # Adjust as needed

      - name: Setup Node.js for Artillery
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Install Artillery
        run: npm install -g artillery

      - name: Run Artillery performance test
        run: artillery run./tests/performance/pr-check.yml --output report.json

      - name: Upload performance report as artifact
        uses: actions/upload-artifact@v4
        with:
          name: performance-report-pr-${{ github.event.pull_request.number }}
          path: report.json

      - name: Download baseline performance report
        uses: dawidd6/action-download-artifact@v3
        with:
          name: baseline-performance-report-main
          path:./baseline
          if_no_artifact_found: 'ignore' # Allow first run on main to pass

      - name: Performance Regression Check
        id: regression-check
        run: |
          # This script compares the new report.json with the baseline report.
          # It will exit with a non-zero status code if a regression is detected.
         ./scripts/compare_performance.sh./report.json./baseline/report.json



Baseline Management and Regression Detection

A static performance threshold (e.g., "P99 latency must always be less than 50ms") is a brittle and ineffective way to gate CI. Such a gate fails to recognize legitimate performance improvements and can unnecessarily block changes that involve acceptable trade-offs. A far more robust approach is dynamic baselining, where the performance of a pull request is compared directly against the current performance of the target branch (main).33
This creates a self-adapting performance target. If the team ships an optimization that improves the P99 latency on main from 40ms to 30ms, the baseline is automatically updated. A subsequent pull request introducing a 35ms P99 latency would correctly be flagged as a regression, even though it is still well below the original 50ms SLO. This process turns the CI pipeline into a continuous conversation about the relative performance impact of every change, which is a more nuanced and powerful method for managing performance over the long term.
Implementation Strategy:
Baseline Generation: A separate GitHub Actions workflow will be created that runs on every push to the main branch. This workflow executes the exact same performance test script (pr-check.yml) and uploads the resulting report.json as an artifact named baseline-performance-report-main. This ensures the baseline is always up-to-date with the latest version of the production code.
Baseline Retrieval: The performance-gate.yml workflow for pull requests will include a step to download the baseline-performance-report-main artifact.
Comparison Logic: A shell script (scripts/compare_performance.sh) will be responsible for the comparison. This script will parse the key metrics (e.g., P95/P99 latency, requests per second) from both the PR's report.json and the downloaded baseline report.json. It will then calculate the percentage change for each metric and compare it against predefined thresholds. If any metric has regressed beyond its threshold, the script will exit with a non-zero status code, causing the GitHub Actions job to fail and block the merge.
The following table makes the pass/fail logic of the performance gate explicit and transparent to all developers. It defines the contract for what constitutes an acceptable performance profile for new code, preventing ambiguity and subjective debates during code review.
Table 2: CI/CD Performance Gate Criteria
Metric
Baseline Source
Comparison Method
Failure Threshold
Action on Failure
P99 Latency
main branch baseline artifact
Percentage Increase
> 10%
Fail workflow, block merge, post comment to PR.
P95 Latency
main branch baseline artifact
Percentage Increase
> 5%
Fail workflow, block merge, post comment to PR.
Throughput (RPS)
main branch baseline artifact
Percentage Decrease
> 5%
Fail workflow, block merge, post comment to PR.
Error Rate
Static Threshold
Absolute Value
> 0.1%
Fail workflow, block merge, post comment to PR.


Proactive Alerting and Incident Response

A robust monitoring system is incomplete without an intelligent alerting strategy. The goal is not simply to be notified when something breaks, but to be alerted proactively to conditions that threaten service reliability, and to do so in a way that respects the on-call engineer's time and attention. This requires moving beyond simple static thresholds and adopting a strategy based on the consumption of the error budget.

A Multi-Tier Alerting Strategy for SLOs

Instead of alerting when a metric crosses a static line, a more sophisticated approach is to alert based on the rate at which the service's error budget is being consumed.4 This method provides alerts that are directly tied to the user-facing SLOs and can be tiered based on urgency.
Warning (Severity: warning): This tier of alert signifies that the error budget is being consumed at a rate that, if sustained, will lead to its complete exhaustion before the end of the 28-day SLO window. For example, an alert could fire if the service has consumed 5% of its monthly budget in just 24 hours. This is a non-urgent alert that does not require immediate action. It should be routed to a team's chat channel (e.g., Slack) to signal that reliability work should be prioritized in the near future to avoid an SLO breach.
Critical (Severity: critical): This tier signifies an acute and immediate threat to the SLO. It fires when the error budget is being consumed at an extremely high rate (e.g., a rate that would exhaust the entire monthly budget in less than 48 hours) or when a critical SLI has fallen below a hard-coded "cliff" (e.g., availability drops below 99%). These alerts indicate an active or imminent user-impacting outage and must page the on-call engineer via a service like PagerDuty for immediate investigation and remediation.35

Prometheus Alerting Rules (rules.yml)

The multi-tier, SLO-based alerting strategy is implemented using PromQL expressions in a Prometheus rules file. These rules codify the conditions for both warning and critical alerts.
Code Example: SLO Burn Rate Alerts
This example demonstrates how to create alerts based on the burn rate of the error budget for both the Rendering and Analysis services. The queries calculate the error rate over a short window and compare it to a multiple of the total allowed error rate.

YAML


# rules/uveddi_alerts.yml
groups:
- name: uveddi.slo.alerts
  rules:
  # --- CRITICAL ALERTS (Page PagerDuty) ---
  - alert: HighErrorBudgetBurn_RenderingService_Availability
    # Fires if the error rate over the last hour is 14x the 28-day SLO target.
    # This means 2% of the monthly budget has been burned in 1 hour (1/(28*24) * 14 ~= 0.02).
    # At this rate, the budget would be exhausted in ~50 hours.
    expr: |
      sum(rate(uveddi_render_requests_total{status_code=~"5.."}[1h])) by (service)
      / 
      sum(rate(uveddi_render_requests_total[1h])) by (service)
      > (14 * (1 - 0.9995))
    for: 5m
    labels:
      severity: critical
      service: rendering-service
    annotations:
      summary: "High error budget burn on Rendering Service (Availability)"
      description: "The Rendering Service error rate is {{ $value | humanizePercentage }}. More than 2% of the 28-day error budget has been consumed in the last hour."
      runbook_url: "https://internal.wiki/uveddi/runbooks/rendering-service-availability"

  # --- WARNING ALERTS (Notify Slack) ---
  - alert: SlowErrorBudgetBurn_AnalysisService_Availability
    # Fires if the error rate over the last 6 hours is 2x the 28-day SLO target.
    # This means ~0.5% of the monthly budget has been burned in 6 hours (6/(28*24) * 2 ~= 0.005).
    # At this rate, the budget would be exhausted in ~2 weeks.
    expr: |
      sum(rate(uveddi_analysis_requests_total{status_code=~"5.."}[6h])) by (service)
      /
      sum(rate(uveddi_analysis_requests_total[6h])) by (service)
      > (2 * (1 - 0.999))
    for: 30m
    labels:
      severity: warning
      service: analysis-service
    annotations:
      summary: "Slow error budget burn on Analysis Service (Availability)"
      description: "The Analysis Service is experiencing an elevated error rate of {{ $value | humanizePercentage }}. If this continues, the 28-day error budget will be consumed in approximately 14 days."
      runbook_url: "https://internal.wiki/uveddi/runbooks/analysis-service-availability"



Alertmanager Configuration (alertmanager.yml) for Intelligent Routing

The design of the alertmanager.yml file is one of the most critical components of a successful SRE practice. It is not merely a technical configuration; it is a system for managing the cognitive load on the on-call engineering team. An unmanaged stream of alerts leads to "alert fatigue," where important signals are lost in the noise and engineers become desensitized to pages.17
A well-configured Alertmanager protects the team's most valuable resource: their focused attention. It achieves this through several key features:
Routing: Directing alerts to different notification channels based on their labels (e.g., severity). This ensures that only truly critical, actionable alerts will interrupt an engineer's sleep or weekend, while lower-severity warnings can be handled during business hours.35
Grouping: Combining multiple alerts with similar labels into a single, consolidated notification. For example, if a network issue causes 50 service instances to become unreachable, grouping by alertname and cluster will result in one notification ("50 instances are down in cluster-prod") instead of 50 separate pages, providing immediate context rather than chaos.17
Inhibition: Suppressing a set of lower-level alerts when a specific higher-level alert is already firing. A classic example is having a ClusterUnreachable alert silence all InstanceDown alerts originating from that cluster. This prevents a storm of cascading, redundant notifications and allows the on-call engineer to focus on the root cause.37
Code Example: alertmanager.yml
This configuration implements the multi-tier strategy by routing alerts with severity: critical to PagerDuty, while alerts with severity: warning go to a specific Slack channel. A default route catches any other alerts and sends them to a general-purpose info channel.

YAML


# alertmanager.yml
global:
  resolve_timeout: 5m
  # Secrets should be managed via environment variables or a secrets management system
  slack_api_url: 'YOUR_SLACK_WEBHOOK_URL'

route:
  # Default receiver for any alert that doesn't match a more specific route.
  receiver: 'slack-info'
  # Group alerts by their name and the service they belong to.
  group_by: ['alertname', 'service']
  # Wait 30 seconds to buffer alerts for the same group before sending an initial notification.
  group_wait: 30s
  # Wait 5 minutes before sending a notification for a new alert in a group that's already firing.
  group_interval: 5m
  # If an alert is still firing, resend the notification every 4 hours.
  repeat_interval: 4h
  
  # Define specific routing rules. Alertmanager checks these in order.
  routes:
    - receiver: 'pagerduty-critical'
      matchers:
        - severity="critical"
      # continue: true # Uncomment this if you want critical alerts to also go to the default Slack channel.

    - receiver: 'slack-warnings'
      matchers:
        - severity="warning"

receivers:
  - name: 'slack-info'
    slack_configs:
      - channel: '#uveddi-alerts-info'
        send_resolved: true
        title: '{{.CommonAnnotations.summary }}'
        text: '{{.CommonAnnotations.description }}\nRunbook: {{.CommonAnnotations.runbook_url }}'

  - name: 'slack-warnings'
    slack_configs:
      - channel: '#uveddi-alerts-warnings'
        send_resolved: true
        title: ' {{.CommonAnnotations.summary }}'
        text: '{{.CommonAnnotations.description }}\nRunbook: {{.CommonAnnotations.runbook_url }}'

  - name: 'pagerduty-critical'
    pagerduty_configs:
      - routing_key: 'YOUR_PAGERDUTY_ROUTING_KEY' # Secret
        send_resolved: true
        # PagerDuty uses a different event structure
        summary: '{{.CommonAnnotations.summary }}'
        details:
          description: '{{.CommonAnnotations.description }}'
          runbook: '{{.CommonAnnotations.runbook_url }}'
          firing_alerts: '{{.Alerts.Firing | len }}'



Introduction to Runbook Automation

To further reduce Mean Time to Resolution (MTTR), the incident response process can be enhanced with runbook automation. This involves creating automated workflows that are triggered by alerts to perform diagnostic or simple remediation tasks.38
Tools like Rundeck (now part of PagerDuty as Process Automation) can be configured to receive webhooks from Alertmanager or PagerDuty.40 When a specific alert fires, it can trigger a pre-defined Rundeck job.
Example Use Case: A HighCPUUsage alert for the Analysis Service could trigger a Rundeck job that:
Securely SSHes into the affected instance.
Runs diagnostic commands like top -b -n 1, ps aux, and collects the last 100 lines of the service log.
Appends this diagnostic information as a note to the corresponding PagerDuty incident.
This automation provides the on-call engineer with immediate, rich context the moment they receive the page, significantly accelerating the triage and investigation process.

Advanced Analysis: Predictive Monitoring and Anomaly Detection

Beyond real-time alerting on SLO violations, the collected time-series data can be used for more advanced, proactive analysis. By applying statistical methods, the system can identify subtle performance trends, detect anomalous behavior that might not breach a hard threshold, and forecast future resource needs.

Trend Analysis and Capacity Planning with predict_linear

Prometheus's query language, PromQL, includes functions for basic predictive analysis. The predict_linear() function uses simple linear regression on a range vector to predict what the value of a time series will be at a specified time in the future.42 This is an incredibly powerful tool for proactive capacity planning and avoiding resource exhaustion.
Code Example: Predicting Disk Space Exhaustion
This PromQL query looks at the rate of change of free disk space over the last six hours and predicts its value 24 hours from now. An alert can be configured to fire if the predicted value is less than zero, giving operators a full day's notice to provision more storage or clean up disk space before it becomes a critical issue.

Code snippet


# Alert if any filesystem is predicted to run out of free space in the next 24 hours.
# predict_linear(v range-vector, t scalar)
# v = node_filesystem_free_bytes[6h] -> the time series data over the last 6 hours
# t = 24 * 3600 -> the number of seconds in the future to predict
predict_linear(node_filesystem_free_bytes{fstype!~"tmpfs"}[6h], 24 * 3600) < 0



Statistical Anomaly Detection in PromQL

Static thresholds are often insufficient for detecting meaningful deviations in highly dynamic systems. A service's request rate, for example, may have natural daily or weekly seasonal patterns. A fixed threshold that is high enough to avoid false positives during peak traffic will be completely insensitive to anomalies during off-peak hours.
A more robust approach is to use statistical methods to detect when a metric deviates significantly from its own recent, normal behavior. The Z-score is a simple yet effective statistical measure for this purpose. It quantifies how many standard deviations a data point is from the mean of its distribution.43 A Z-score greater than a certain threshold (typically 2 or 3) is considered a statistical anomaly.
The Z-score can be calculated directly in PromQL using the following pattern:
$Z = (current\_value - moving\_average) / moving\_standard\_deviation$
$( (metric - avg_over_time(metric[window])) / stddev_over_time(metric[window]) )$

Practical PromQL Recipes for Anomaly Detection

By applying the Z-score pattern to the Uveddi service metrics, we can create powerful anomaly detection alerts that adapt to changing traffic patterns.
Detecting Latency Regressions:
This query calculates the Z-score for the P99 rendering latency. It compares the current 5-minute P99 latency to the moving average and standard deviation over the last hour. An alert would fire if the latency suddenly becomes significantly higher than its recent norm, even if it hasn't yet breached the hard 50ms SLO. This can provide an early warning of a developing performance problem.

Code snippet


# Alert if the P99 rendering latency is more than 3 standard deviations 
# above its 1-hour moving average.
(
  histogram_quantile(0.99, sum(rate(uveddi_render_duration_seconds_bucket{service="rendering-service"}[5m])) by (le))
  -
  avg_over_time(histogram_quantile(0.99, sum(rate(uveddi_render_duration_seconds_bucket{service="rendering-service"}[5m])) by (le))[1h])
) / stddev_over_time(histogram_quantile(0.99, sum(rate(uveddi_render_duration_seconds_bucket{service="rendering-service"}[5m])) by (le))[1h]) > 3


Detecting Sudden Spikes in Error Rate:
This query detects an anomalous spike in the error rate for the Analysis Service. It is far more sensitive than a simple threshold because it compares the current error rate to its own recent history. A jump from 0.01% to 0.1% might be a significant anomaly during a quiet period, even though it is still within the 0.1% SLO.

Code snippet


# Alert if the 5-minute error rate for the analysis service is anomalously high 
# compared to its behavior over the last hour.
(
  sum(rate(uveddi_analysis_requests_total{status_code=~"5.."}[5m])) by (service)
  -
  avg_over_time(sum(rate(uveddi_analysis_requests_total{status_code=~"5.."}[5m])) by (service)[1h])
) / stddev_over_time(sum(rate(uveddi_analysis_requests_total{status_code=~"5.."}[5m])) by (service)[1h]) > 3


These advanced queries transform the monitoring system from a reactive tool that reports on failures to a proactive one that identifies risks and deviations before they escalate into user-facing incidents.

Implementation Roadmap and Strategic Recommendations

This final section provides a phased, actionable plan for implementing the monitoring, alerting, and validation strategy outlined in this document. It also addresses potential risks and offers strategic recommendations for fostering a durable culture of reliability.

Phased Implementation Plan

The implementation is broken down into three logical phases to allow for incremental delivery of value and to build momentum.
Phase 1: Foundational Monitoring (Days 1-3)
[ ] Task: Instrument the Rust backend and Node.js rendering service with the core SLI metrics (request counters, latency histograms) as detailed in Section 3.
[ ] Task: Deploy Prometheus and Grafana into a development or staging environment.
[ ] Task: Create the initial prometheus.yml scrape configuration to begin collecting metrics from the Uveddi services.
[ ] Task: Build the first version of the "Uveddi SLO Dashboard" in Grafana, creating panels to visualize the SLIs defined in Section 1. This provides immediate visibility into service performance.
Goal: Establish end-to-end metric collection and visualization.
Phase 2: Alerting and CI Gating (Days 4-7)
[ ] Task: Configure the Prometheus alerting rules (rules.yml) for SLO burn rates as specified in Section 5.
[ ] Task: Deploy and configure an Alertmanager instance. Integrate it with Slack for warning alerts and PagerDuty for critical alerts.
[ ] Task: Implement the performance-gate.yml GitHub Actions workflow for automated performance testing on pull requests.
[ ] Task: Implement the main-baseline.yml GitHub Actions workflow to establish and update the performance baseline on every merge to main.
Goal: Automate performance regression detection and establish a proactive alerting pipeline.
Phase 3: Refinement and Advanced Analysis (Week 2 and beyond)
[ ] Task: Implement the advanced predictive and anomaly detection alerting rules from Section 6.
[ ] Task: After collecting at least two weeks of production performance data, convene a meeting with stakeholders to review the initial SLOs and error budgets. Adjust targets based on observed performance and user feedback.
[ ] Task: Identify the most frequent and impactful critical alerts and begin developing automated diagnostic runbooks using a tool like Rundeck/PagerDuty Process Automation.
Goal: Mature the monitoring system from reactive to proactive and begin automating operational toil.

Risk Assessment and Mitigation

Risk: The performance overhead of instrumentation could negatively impact the application.
Mitigation: The metrics crate in Rust is specifically designed to be extremely lightweight and have a negligible performance footprint.22 The performance impact of the instrumentation will be measured as part of the CI performance test itself. Any significant overhead will be detected before it reaches production.
Risk: Poorly tuned alerting rules could lead to alert fatigue, causing engineers to ignore important notifications.
Mitigation: The initial deployment should use conservative alerting thresholds. The multi-tier strategy is designed to mitigate this risk by separating critical pages from informational warnings. The tuning of alert thresholds and Alertmanager's grouping/inhibition rules should be treated as a continuous, iterative process, not a one-time setup. Regular review of alert frequency and actionability is essential.
Risk: Flaky or inconsistent performance tests in the CI pipeline could block valid changes or allow regressions to pass.
Mitigation: The CI test environment must be as stable and isolated as possible. The performance test script should include warm-up periods to ensure the application reaches a steady state before measurements begin. The compare_performance.sh script should be configured with a small tolerance (e.g., 2-5%) to account for minor, acceptable variance in test runs.

Fostering an SRE Culture

The technology and processes detailed in this report are powerful enablers, but their long-term success is contingent upon a cultural shift towards the principles of Site Reliability Engineering. The tools themselves do not create reliability; the culture does. The research and best practices from leading technology companies like Google and Netflix consistently emphasize that cultural aspects such as shared ownership, blameless postmortems, and data-driven decision-making are prerequisites for SRE success.4
Without a culture of shared ownership between development, operations, and product teams, the systems described here can be perceived as "SRE policing" development. The error budget policy is the key mechanism for fostering this shared ownership. It must be embraced by leadership as the primary, objective tool for negotiating the inherent trade-off between feature velocity and service reliability. When the error budget is healthy, the product team can confidently prioritize new features. When it is depleted, all teams must agree that the priority is to stabilize the system.
Recommendation:
To embed this culture, it is recommended to establish a regular (e.g., bi-weekly or monthly) SLO Review Meeting. This meeting should include stakeholders from SRE, development, and product management. The Grafana SLO dashboard should serve as the central artifact for these discussions, providing a shared, objective view of service health. This forum should be used to:
Review SLO compliance and error budget consumption over the previous period.
Celebrate successes and periods of high reliability.
Conduct blameless postmortems for any SLO breaches or significant incidents, focusing on systemic causes and learning opportunities, not individual blame.
Make data-driven decisions about prioritizing new features versus reliability work for the upcoming period.
By adopting both the technical blueprint and the cultural mindset outlined in this document, the Uveddi project can build a robust, scalable, and user-trusted service.

Appendices


Appendix A: Grafana Dashboard JSON Model

The following is a JSON model for a foundational Uveddi SLO Dashboard. It can be imported directly into Grafana. It assumes a Prometheus data source named Prometheus. It includes panels for tracking the key SLIs for both the Analysis and Rendering services.

JSON


{
  "__inputs":,
  "__requires":,
  "annotations": {
    "list": [
      {
        "builtIn": 1,
        "datasource": {
          "type": "grafana",
          "uid": "-- Grafana --"
        },
        "enable": true,
        "hide": true,
        "iconColor": "rgba(0, 211, 255, 1)",
        "name": "Annotations & Alerts",
        "type": "dashboard"
      }
    ]
  },
  "editable": true,
  "fiscalYearStartMonth": 0,
  "graphTooltip": 0,
  "id": null,
  "links":,
  "panels":)) / sum(rate(uveddi_render_requests_total[28d])) * 100",
          "legendFormat": "Availability"
        }
      ],
      "options": { "reduceOptions": { "calcs": ["last"], "fields": "", "values": false }, "orientation": "auto", "textMode": "auto", "colorMode": "thresholds", "graphMode": "area", "justifyMode": "auto" },
      "thresholds": { "mode": "absolute", "steps": [{ "color": "red", "value": null }, { "color": "orange", "value": 99.95 }, { "color": "green", "value": 99.98 }] }
    },
    {
      "id": 2,
      "title": "Rendering Service: P99 Latency (5m)",
      "type": "timeseries",
      "datasource": "Prometheus",
      "gridPos": { "h": 8, "w": 12, "x": 0, "y": 4 },
      "targets": [
        {
          "expr": "histogram_quantile(0.99, sum(rate(uveddi_render_duration_seconds_bucket[5m])) by (le))",
          "legendFormat": "P99 Latency"
        }
      ],
      "options": { "tooltip": { "mode": "multi" }, "legend": { "displayMode": "list", "placement": "bottom" } },
      "fieldConfig": { "defaults": { "unit": "s", "custom": { "axisPlacement": "right" } } }
    },
    {
      "id": 3,
      "title": "Analysis Service: Availability (28d)",
      "type": "stat",
      "datasource": "Prometheus",
      "gridPos": { "h": 4, "w": 6, "x": 6, "y": 0 },
      "targets": [
        {
          "expr": "sum(rate(uveddi_analysis_requests_total{status_code!~\"5..\"}[28d])) / sum(rate(uveddi_analysis_requests_total[28d])) * 100",
          "legendFormat": "Availability"
        }
      ],
      "options": { "reduceOptions": { "calcs": ["last"], "fields": "", "values": false }, "orientation": "auto", "textMode": "auto", "colorMode": "thresholds", "graphMode": "area", "justifyMode": "auto" },
      "thresholds": { "mode": "absolute", "steps": [{ "color": "red", "value": null }, { "color": "orange", "value": 99.9 }, { "color": "green", "value": 99.95 }] }
    },
    {
      "id": 4,
      "title": "Analysis Service: P95 Latency (5m)",
      "type": "timeseries",
      "datasource": "Prometheus",
      "gridPos": { "h": 8, "w": 12, "x": 12, "y": 4 },
      "targets": [
        {
          "expr": "histogram_quantile(0.95, sum(rate(uveddi_analysis_duration_seconds_bucket[5m])) by (le))",
          "legendFormat": "P95 Latency"
        }
      ],
      "options": { "tooltip": { "mode": "multi" }, "legend": { "displayMode": "list", "placement": "bottom" } },
      "fieldConfig": { "defaults": { "unit": "s" } }
    }
  ],
  "schemaVersion": 37,
  "style": "dark",
  "tags": ["uveddi", "slo"],
  "templating": { "list": },
  "time": { "from": "now-6h", "to": "now" },
  "timepicker": {},
  "timezone": "browser",
  "title": "Uveddi SLO Dashboard",
  "uid": "uveddi-slo-dashboard",
  "version": 1,
  "weekStart": ""
}



Appendix B: Complete Prometheus Configuration Files

prometheus.yml

YAML


# prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

alerting:
  alertmanagers:
  - static_configs:
    - targets:
      - 'alertmanager:9093' # Assumes Alertmanager is running in a container named 'alertmanager'

rule_files:
  - '/etc/prometheus/rules/*.yml'

scrape_configs:
  - job_name: 'prometheus'
    static_configs:
      - targets: ['localhost:9090']

  - job_name: 'uveddi-rust-backend'
    static_configs:
      - targets: ['host.docker.internal:9090'] # Adjust if not running in Docker
        labels:
          service: 'analysis-service'

  - job_name: 'uveddi-node-renderer'
    static_configs:
      - targets: ['host.docker.internal:8081'] # Adjust if not running in Docker
        labels:
          service: 'rendering-service'


rules/uveddi_alerts.yml

YAML


# /etc/prometheus/rules/uveddi_alerts.yml
groups:
- name: uveddi.slo.alerts
  rules:
  # --- CRITICAL ALERTS (Page PagerDuty) ---
  - alert: HighErrorBudgetBurn_RenderingService_Availability
    expr: |
      sum(rate(uveddi_render_requests_total{status_code=~"5.."}[1h])) by (service)
      / 
      sum(rate(uveddi_render_requests_total[1h])) by (service)
      > (14 * (1 - 0.9995))
    for: 5m
    labels:
      severity: critical
    annotations:
      summary: "High error budget burn on Rendering Service (Availability)"
      description: "The Rendering Service error rate is {{ $value | humanizePercentage }}. More than 2% of the 28-day error budget has been consumed in the last hour."
      runbook_url: "https://internal.wiki/uveddi/runbooks/rendering-service-availability"

  - alert: HighErrorBudgetBurn_RenderingService_Latency
    expr: |
      sum(rate(uveddi_render_duration_seconds_bucket{le='+Inf'}[1h])) by (service) - sum(rate(uveddi_render_duration_seconds_bucket{le='0.05'}[1h])) by (service)
      /
      sum(rate(uveddi_render_duration_seconds_bucket{le='+Inf'}[1h])) by (service)
      > (14 * (1 - 0.99))
    for: 10m
    labels:
      severity: critical
    annotations:
      summary: "High error budget burn on Rendering Service (Latency)"
      description: "More than 2% of the 28-day latency error budget for the Rendering Service has been consumed in the last hour."
      runbook_url: "https://internal.wiki/uveddi/runbooks/rendering-service-latency"

  # --- WARNING ALERTS (Notify Slack) ---
  - alert: SlowErrorBudgetBurn_AnalysisService_Availability
    expr: |
      sum(rate(uveddi_analysis_requests_total{status_code=~"5.."}[6h])) by (service)
      /
      sum(rate(uveddi_analysis_requests_total[6h])) by (service)
      > (2 * (1 - 0.999))
    for: 30m
    labels:
      severity: warning
    annotations:
      summary: "Slow error budget burn on Analysis Service (Availability)"
      description: "The Analysis Service is experiencing an elevated error rate of {{ $value | humanizePercentage }}. If this continues, the 28-day error budget will be consumed in approximately 14 days."
      runbook_url: "https://internal.wiki/uveddi/runbooks/analysis-service-availability"



Appendix C: Complete Alertmanager Configuration

alertmanager.yml

YAML


# alertmanager.yml
global:
  resolve_timeout: 5m
  # It is strongly recommended to use a secret management tool or environment variables
  # for sensitive values like webhook URLs and API keys.
  # slack_api_url: 'YOUR_SLACK_WEBHOOK_URL'

route:
  receiver: 'slack-info'
  group_by: ['alertname', 'service', 'cluster']
  group_wait: 30s
  group_interval: 5m
  repeat_interval: 4h
  
  routes:
    - receiver: 'pagerduty-critical'
      matchers:
        - severity="critical"

    - receiver: 'slack-warnings'
      matchers:
        - severity="warning"

# Inhibition rules help prevent alert storms.
# This rule silences any 'warning' severity alerts if a 'critical' alert
# with the same alertname and service is already firing.
inhibit_rules:
  - source_matchers:
      - severity="critical"
    target_matchers:
      - severity="warning"
    equal: ['alertname', 'service']

receivers:
  - name: 'slack-info'
    slack_configs:
      - api_url: 'YOUR_SLACK_WEBHOOK_URL_FOR_INFO'
        send_resolved: true
        channel: '#uveddi-alerts-info'
        title: '[INFO] {{.CommonLabels.alertname }} for {{.CommonLabels.service }}'
        text: '{{ range.Alerts }}*Summary:* {{.Annotations.summary }}\n*Description:* {{.Annotations.description }}\n*Runbook:* {{.Annotations.runbook_url }}\n{{ end }}'

  - name: 'slack-warnings'
    slack_configs:
      - api_url: 'YOUR_SLACK_WEBHOOK_URL_FOR_WARNINGS'
        send_resolved: true
        channel: '#uveddi-alerts-warnings'
        title: ':warning: {{.CommonLabels.alertname }} for {{.CommonLabels.service }}'
        text: '{{ range.Alerts }}*Summary:* {{.Annotations.summary }}\n*Description:* {{.Annotations.description }}\n*Runbook:* {{.Annotations.runbook_url }}\n{{ end }}'

  - name: 'pagerduty-critical'
    pagerduty_configs:
      - routing_key: 'YOUR_PAGERDUTY_ROUTING_KEY'
        send_resolved: true
        summary: ' {{.CommonLabels.alertname }} for {{.CommonLabels.service }}'
        severity: 'critical'
        source: '{{.CommonLabels.service }}'
        details:
          description: '{{.CommonAnnotations.description }}'
          runbook_url: '{{.CommonAnnotations.runbook_url }}'
          num_firing: '{{.Alerts.Firing | len }}'
          num_resolved: '{{.Alerts.Resolved | len }}'



Appendix D: Complete GitHub Actions Workflows

PR Performance Gate: .github/workflows/performance-gate.yml
(As provided in Section 4.2)
Main Branch Baselining: .github/workflows/main-baseline.yml

YAML


#.github/workflows/main-baseline.yml
name: Update Performance Baseline

on:
  push:
    branches: [ main ]

jobs:
  update-baseline:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout main branch
        uses: actions/checkout@v4

      - name: Set up Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Cache dependencies
        uses: Swatinem/rust-cache@v2

      - name: Build application
        run: cargo build --release

      - name: Run Uveddi application in background
        run:./target/release/uveddi &

      - name: Wait for application to start
        run: sleep 10

      - name: Setup Node.js for Artillery
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Install Artillery
        run: npm install -g artillery

      - name: Run Artillery performance test
        run: artillery run./tests/performance/pr-check.yml --output report.json

      - name: Upload baseline performance report
        uses: actions/upload-artifact@v4
        with:
          name: baseline-performance-report-main
          path: report.json
          retention-days: 90 # Keep baselines for historical analysis


Works cited
Service Level Agreement Best Practices: A Complete Guide for Successful SLA Management - Supportman, accessed July 19, 2025, https://supportman.io/articles/service-level-agreement-best-practices/
What is a Service Level Agreement (SLA)? - InvGate, accessed July 19, 2025, https://invgate.com/itsm/service-level-management/service-level-agreement
Service-Level Agreement (SLA): Why It's Important and How to Write One | Coursera, accessed July 19, 2025, https://www.coursera.org/articles/sla
Guide to Building an SRE Function: Principles and Best Practices - Edvantis, accessed July 19, 2025, https://www.edvantis.com/blog/sre-function/
The Comprehensive Guide on SLIs, SLOs, and Error Budgets - Blameless, accessed July 19, 2025, https://www.blameless.com/the-comprehensive-guide-on-slis-slos-and-error-budgets
Chapter 2 - Implementing SLOs - Google SRE, accessed July 19, 2025, https://sre.google/workbook/implementing-slos/
Defining slo: service level objective meaning - Google SRE, accessed July 19, 2025, https://sre.google/sre-book/service-level-objectives/
What is SLA? Best Practices for Service-level Agreements - Iterators, accessed July 19, 2025, https://www.iteratorshq.com/blog/what-is-sla-best-practices-for-service-level-agreements/
What is SLA? - Service Level Agreement Explained - AWS, accessed July 19, 2025, https://aws.amazon.com/what-is/service-level-agreement/
How to Define and Monitor API Service Level Agreements (SLAs) and Objectives (SLOs), accessed July 19, 2025, https://apitoolkit.io/blog/monitor-api-slas-and-slos/
13 Best Open Source & Free Monitoring Tools in 2025 - DevOpsCube, accessed July 19, 2025, https://devopscube.com/best-opensource-monitoring-tools/
Compare Prometheus vs VictoriaMetrics - InfluxDB, accessed July 19, 2025, https://www.influxdata.com/comparison/prometheus-vs-victoria/
Monitoring a Rust Web Application Using Prometheus and Grafana ..., accessed July 19, 2025, https://medium.com/better-programming/monitoring-a-rust-web-application-using-prometheus-and-grafana-3c75d9435dec?responsesOpen=true
Axum App Monitoring with Prometheus and Grafana - DevOps.dev, accessed July 19, 2025, https://blog.devops.dev/axum-app-monitoring-with-prometheus-and-grafana-b554692095b5
Grafana: The open and composable observability platform | Grafana Labs, accessed July 19, 2025, https://grafana.com/
Alerting rules - Prometheus, accessed July 19, 2025, https://prometheus.io/docs/prometheus/latest/configuration/alerting_rules/
Prometheus Alertmanager Best Practices - Sysdig, accessed July 19, 2025, https://sysdig.com/blog/prometheus-alertmanager/
Comparing Open Source Time Series Databases: Prometheus vs InfluxDB, accessed July 19, 2025, https://risingwave.com/blog/comparing-monitoring-and-alerting-features-of-open-source-time-series-databases-prometheus-vs-influxdb-vs-victoriametrics/
risingwave.com, accessed July 19, 2025, https://risingwave.com/blog/comparing-monitoring-and-alerting-features-of-open-source-time-series-databases-prometheus-vs-influxdb-vs-victoriametrics/#:~:text=For%20infrastructure%20monitoring%20and%20application,monitoring%20and%20large%2Dscale%20environments.
Benchmarking InfluxDB vs. VictoriaMetrics: Choosing the Right Time-Series Database, accessed July 19, 2025, https://soufianebouchaara.com/benchmarking-influxdb-vs-victoriametrics-choosing-the-right-time-series-database/
metrics - Rust - Docs.rs, accessed July 19, 2025, https://docs.rs/metrics/
Crate metrics - Rust, accessed July 19, 2025, https://prisma.github.io/prisma-engines/doc/metrics/index.html
metrics-rs/metrics: A metrics ecosystem for Rust. - GitHub, accessed July 19, 2025, https://github.com/metrics-rs/metrics
metrics_exporter_prometheus - Rust - Docs.rs, accessed July 19, 2025, https://docs.rs/metrics-exporter-prometheus/
metrics-exporter-prometheus - crates.io: Rust Package Registry, accessed July 19, 2025, https://crates.io/crates/metrics-exporter-prometheus
Rust Service Logs/Metrics - Reddit, accessed July 19, 2025, https://www.reddit.com/r/rust/comments/1hgkmk6/rust_service_logsmetrics/
Getting Started - OpenTelemetry, accessed July 19, 2025, https://opentelemetry.io/docs/languages/rust/getting-started/
tracing_opentelemetry - Rust - Docs.rs, accessed July 19, 2025, https://docs.rs/tracing-opentelemetry
open-telemetry/opentelemetry-rust - GitHub, accessed July 19, 2025, https://github.com/open-telemetry/opentelemetry-rust
Static Code Analysis Best Practices for Developers - ACCELQ, accessed July 19, 2025, https://www.accelq.com/blog/static-code-analysis-best-practices/
How to enforce code quality gates in GitHub Actions - Graphite, accessed July 19, 2025, https://graphite.dev/guides/enforce-code-quality-gates-github-actions
Running Artillery on GitHub Actions, accessed July 19, 2025, https://www.artillery.io/docs/cicd/github-actions
Run a load test from GitHub Actions | Implementing DevOps practices, accessed July 19, 2025, https://microsoft.github.io/TechExcel-Implementing-DevOps-practices-to-accelerate-developer-productivity/docs/04_implement_load_testing/0402.html
GitHub Action for continuous benchmarking of pull requests, to monitor performance regression, accessed July 19, 2025, https://github.com/openpgpjs/github-action-pull-request-benchmark
Prometheus Alerting Examples for Developers - Last9, accessed July 19, 2025, https://last9.io/blog/prometheus-alerting-examples/
Prometheus and PagerDuty Integration - GitHub, accessed July 19, 2025, https://github.com/chef/monitoring-integration-automate/blob/main/prometheus/prometheus_PagerDuty_Integration_and_Notification.md
Prometheus Alertmanager: The Basics and a Quick Tutorial - Coralogix, accessed July 19, 2025, https://coralogix.com/guides/prometheus-monitoring/rometheus-alertmanage/
What is Runbook Automation? - Rundeck, accessed July 19, 2025, https://www.rundeck.com/what-is-runbook-automation
Enhancing IT efficiency with Runbook Automation: Best Practices & Examples - Squadcast, accessed July 19, 2025, https://www.squadcast.com/sre-best-practices/runbook-automation
Rundeck Runbook Automation, accessed July 19, 2025, https://www.rundeck.com/
Configure Notifications using PagerDuty Plugin - Rundeck Docs, accessed July 19, 2025, https://docs.rundeck.com/docs/learning/howto/pagerduty-notification.html
Query functions - Prometheus, accessed July 19, 2025, https://prometheus.io/docs/prometheus/latest/querying/functions/
Anomaly Detection in Application Metrics for Dynamic Alerting - DevOps.dev, accessed July 19, 2025, https://blog.devops.dev/anomaly-detection-in-application-metrics-for-dynamic-alerting-4885ac5894c7
Practical Anomaly Detection with Prometheus and PromQL | by Grigor Khachatryan, accessed July 19, 2025, https://grigorkh.medium.com/practical-anomaly-detection-with-prometheus-and-promql-d3027b97da96
Prometheus Anomaly detection: Z-Score in PromQL · - Omar Ghader, accessed July 19, 2025, https://omarghader.github.io/prometheus-anomaly-detection-z-score-in-promql/
SRE Best Practices for Reliability & Reslilence | Blameless, accessed July 19, 2025, https://www.blameless.com/the-essential-guide-to-sre
Site Reliability Engineering Top 10 Best Practice - Enov8, accessed July 19, 2025, https://www.enov8.com/blog/site-reliability-engineering-sre-top-10-best-practice/
