
UV-82 Chaos Engineering Implementation Blueprint: A Framework for Resilient Rust Systems


Section 1: Strategic Foundations for Chaos Engineering in Rust

This report outlines a comprehensive strategy for implementing chaos engineering within the Uveddi project. The approach is tailored to its Rust-based microservices architecture, leveraging the language's unique features to build a robust, safe, and highly observable resilience testing framework. By moving beyond generic principles, this blueprint provides an actionable plan to proactively identify and mitigate weaknesses before they impact system stability.

1.1 The Chaos Engineering Lifecycle in a Rust Context

The successful implementation of chaos engineering follows a structured, iterative lifecycle.1 For the Uveddi project, this lifecycle is adapted to integrate with the existing codebase and infrastructure.
Define Steady State: The first step is to establish a quantitative baseline of the system's normal, healthy behavior. This involves leveraging the existing monitoring infrastructure (src/monitoring/) and the metrics crate to capture key performance indicators (KPIs) under typical workloads. Critical steady-state metrics for Uveddi include p95 and p99 latency for core operations like code analysis and diagram generation, the success rate of rendering requests to the Node.js service, and baseline CPU and memory utilization.2 This baseline is the yardstick against which all experiment outcomes are measured.4
Formulate a Hypothesis: For every potential failure, a clear, testable hypothesis must be formulated. This prediction describes how the system should behave when subjected to a specific fault, based on the resilience patterns already implemented in src/resilience/.1 An example hypothesis for Uveddi would be: "If the Node.js rendering service experiences a latency spike exceeding 500ms, the circuit breaker in the Rust backend will open within three consecutive failures, and subsequent API calls will immediately return a pre-defined fallback response without waiting for a timeout."
Introduce Real-World Events (Safely): This phase involves the controlled injection of failures that mimic real-world incidents, such as network latency, service outages, or resource exhaustion.1 The core principle is to start with the smallest possible "blast radius"—the scope of potential impact.2 Experiments should begin in isolated test environments, targeting a single instance or even a single function call, before gradually expanding in scope as confidence in the system's resilience grows.4
Verify and Learn: The final step is to observe the system's response and compare it to the hypothesis and the established steady-state baseline. Any deviation between the expected and actual outcome exposes a systemic weakness. These findings are not failures of the system but successes of the experiment, providing invaluable insights that drive improvements to error handling, fallback logic, or architectural design.1 This continuous feedback loop is the fundamental mechanism through which chaos engineering builds progressively more resilient systems.

1.2 Best Practices for Rust Microservices

The Uveddi project's use of Rust presents unique opportunities to implement chaos engineering with a high degree of safety and precision.
Leverage the Type System for Safety: Rust's Result<T, E> enum is the cornerstone of its error-handling philosophy. Unlike exceptions, which can be unhandled, Result forces developers to confront potential failures at compile time. Our chaos experiments will be designed to specifically trigger these Err variants, allowing us to validate that error-handling logic is correctly implemented throughout the entire call stack and does not lead to unexpected panic!s.
Embrace Code-Level Fault Injection: The landscape of off-the-shelf chaos engineering tools for Rust is less mature than for ecosystems like Java or Kubernetes.6 This is not a limitation but a strategic advantage. It encourages the development of a bespoke, in-code fault injection framework. Using Rust's powerful features like conditional compilation, procedural macros, and middleware libraries like
tower, we can achieve surgical precision. Instead of crudely terminating a container, we can simulate a specific rusqlite::Error during a database transaction or inject latency into a single tower::Service call. This approach provides far more valuable insights into the application's internal logic.
Prioritize Comprehensive Observability: Chaos engineering is ineffective and potentially dangerous without robust observability.1 It is impossible to validate a hypothesis or understand the impact of a fault without detailed metrics, structured logs, and distributed traces.3 The Uveddi project's existing foundation with the
tracing and metrics crates is a critical prerequisite and will be the backbone of our measurement and validation strategy.
Automate Continuously in CI/CD: A resilient system yesterday may not be resilient today after new features or architectural changes. Chaos engineering cannot be a one-time audit; it must be a continuous practice.1 By integrating automated chaos experiments into the CI/CD pipeline, we ensure that the system's resilience is constantly validated as it evolves, making it a core part of the development workflow rather than an afterthought.9

Section 2: A Multi-Layered Chaos Engineering Architecture for Uveddi

To effectively test the Uveddi system at all levels, a multi-layered chaos engineering architecture is proposed. This design provides a "resilience testing pyramid," enabling a range of experiments from fast, granular unit tests to comprehensive, system-wide simulations. This tiered approach optimizes for both test coverage and execution cost, allowing for rapid feedback during development and deep, exploratory analysis in staging environments.

2.1 Overall Architecture

The framework is composed of three tiers of fault injection: failpoints for unit-level tests, a custom middleware for inter-service communication tests, and a deterministic simulation framework for end-to-end system tests. A central configuration module, src/chaos/config.rs, will provide a single point of control for all experiments, using feature flags to ensure that no chaos-related code is present in production builds.

2.2 Tier 1: Unit-Level Fault Injection with fail-rs

The foundation of the resilience pyramid consists of fast, precise, unit-level tests designed to validate the error-handling logic of individual functions.
Purpose: To test how specific components react to predictable, internal failures, such as a database error or a file system permission issue. This tier is ideal for validating the correctness of Result::Err handling paths.
Implementation: The fail crate, a mature and well-regarded failpoint implementation for Rust, will be used.10
Mechanism: fail_point! macros will be placed at critical junctures in the code where fallible operations occur. These macros are inert by default and are compiled out of release builds, imposing zero performance overhead.
Rust
// Location: src/database/queries.rs
use fail;
use sqlx::{Pool, Sqlite};
use uuid::Uuid;
use crate::models::Analysis;

pub async fn get_analysis_result(db: &Pool<Sqlite>, id: Uuid) -> Result<Analysis, sqlx::Error> {
    // This failpoint allows us to inject an error before the query executes.
    fail::fail_point!("db_get_result_error", |msg| {
        // The closure is executed only when the failpoint is active.
        // It simulates a database I/O error, returning it to the caller.
        let err_msg = msg.unwrap_or_else(|| "injected chaos error".to_string());
        Err(sqlx::Error::Io(std::io::Error::new(
            std::io::ErrorKind::ConnectionAborted,
            err_msg,
        )))
    });

    // Original database query logic follows.
    sqlx::query_as!(/*...*/)
       .fetch_one(db)
       .await
}


Activation: In a test environment, failpoints are enabled by compiling with the fail/failpoints feature. They can then be triggered programmatically within a test function using fail::cfg() or via an environment variable (FAILPOINTS="db_get_result_error=return") for broader tests.10 This provides a highly controlled and reproducible way to test error paths.

2.3 Tier 2: Middleware-Based Fault Injection with a Custom tower::Layer

The middle tier of the pyramid focuses on the interactions between Uveddi's microservices, specifically testing the resilience of the Rust backend against failures in its dependencies, like the Node.js rendering service.
Purpose: To simulate network-level faults such as latency, packet loss, and service unavailability, directly testing the effectiveness of resilience patterns like circuit breakers and retries within the tower stack.
Implementation: A custom tower::Layer named ChaosLayer will be created. While crates like tower-fault-injector exist, their documentation is sparse, and a custom implementation offers greater control and can be tailored precisely to Uveddi's needs.13 The design will follow established
tower middleware patterns.15
Mechanism: The ChaosLayer will wrap the tower::Service responsible for making HTTP requests. Within its call method, it will consult the central ChaosConfig to determine if a fault should be injected. Based on this configuration, it can:
Inject Latency: Delay the request by wrapping the inner service's future in a tokio::time::sleep.
Inject Errors: Immediately return a Result::Err, simulating an HTTP 503 or a connection timeout, without ever calling the inner service.
Drop Requests: Return a future that never resolves, forcing upstream timeout mechanisms to engage.
Configuration: The layer will be dynamically configurable via the ChaosConfig module, allowing tests to specify the fault type, probability (e.g., affect 10% of requests), and magnitude (e.g., inject 300-500ms of latency).

2.4 Tier 3: System-Level Deterministic Simulation Testing (DST) with turmoil

The top of the pyramid is reserved for the most powerful form of chaos testing: deterministic simulation. This approach is designed to uncover complex, emergent bugs that only arise from the intricate interactions of multiple components over time.
Purpose: To test the entire distributed system under a wide variety of chaotic conditions—including network partitions, message reordering, and clock skew—in a perfectly reproducible environment.
Implementation: The turmoil framework will be integrated into the project.18 This requires abstracting I/O and time dependencies behind traits that can have different implementations for production and testing.
Mechanism: turmoil executes multiple simulated "hosts" (representing Uveddi's microservices) within a single, deterministic, single-threaded environment. It provides its own simulated network (turmoil::net) and a simulated clock (turmoil::time). All sources of randomness, including task scheduling, are derived from a single seed value provided at the start of the test.
Adaptation: Using conditional compilation (#[cfg(feature = "turmoil_test")]), the application will be compiled to use turmoil's primitives instead of tokio's. This allows the exact same application logic to be run in both a real and a simulated environment. The key benefit is reproducibility: if a test run with seed 12345 uncovers a race condition, that exact failure can be reproduced on demand by running the test again with the same seed, dramatically simplifying debugging.18

2.5 Centralized Experiment Configuration and Control

A single module, src/chaos/config.rs, will serve as the command center for all chaos experiments. It will be responsible for parsing experiment configurations from a file (e.g., chaos.yaml) or environment variables. This module will be guarded by a chaos feature flag, ensuring it and all related logic are excluded from production builds. This provides a global "kill switch" and a unified interface for managing the blast radius, probability, and intensity of all injected faults.

Section 3: Tooling and Framework Analysis

The selection of appropriate tooling is critical for the success of a chaos engineering practice. The proposed architecture deliberately favors a bespoke, code-native approach over adopting a single, generic framework. This decision is based on the specific technical requirements of the Uveddi project and the unique characteristics of the Rust ecosystem.

3.1 Primary Tooling Recommendations

The recommended toolset is a composition of specialized crates, each serving a distinct purpose within the multi-layered architecture:
fail-rs: Chosen for its ability to inject granular, function-level faults with zero production overhead. It is ideal for unit-testing specific error-handling code paths.10
Custom tower::Layer: Developed in-house to provide precise control over network-level fault injection (latency, errors) directly within the tokio and tower stack that powers Uveddi's service communication.15
turmoil: Selected for its powerful deterministic simulation capabilities, enabling the discovery and reproducible debugging of complex, emergent bugs in distributed systems.18

3.2 Comparative Analysis

The following table compares the recommended tools against common alternatives, justifying the selection based on criteria essential to the Uveddi project.
Tool
Primary Use Case
Integration with Rust/Tokio
Injection Precision
Safety Mechanism
Implementation Effort
fail-rs
Unit-level error path validation
Native Rust crate
Function/Expression
Compile-time feature flags
Low
Custom tower::Layer
Inter-service network fault simulation
Native tower middleware
Service/Request
Runtime configuration, feature flags
Medium
turmoil
System-wide deterministic simulation
Native tokio replacement
System-wide (network, time)
Single-threaded, seeded RNG
High
kaos
Test harness for availability
Rust crate, orchestrates binaries
Process-level (crashes)
Test environment isolation
Medium
Generic K8s Tools (e.g., Chaos Mesh)
Infrastructure-level chaos
External to application
Pod/Node/Network
Blast radius configuration (YAML)
Low (for basic use)

This comparison highlights a crucial distinction. While infrastructure-level tools like Chaos Mesh are powerful for testing how a system responds to events like a pod deletion, they treat the application as an opaque black box.7 They cannot verify if a specific
sqlx::Error is handled gracefully or if a retry policy is implemented correctly. The chosen code-native tools (fail-rs, tower::Layer, turmoil) provide the necessary introspection to test the application's internal resilience logic, which is the primary objective.

3.3 Justification and Ecosystem Context

Why Not kaos? The kaos crate is positioned as a "chaotic testing harness".23 Its primary function appears to be orchestrating repeated runs of a service and asserting its availability, assuming that failures (like panics) are introduced by other means, such as integrated failpoints.23 While it could be a useful component in a larger testing strategy, it does not provide the direct fault injection mechanisms required by our architecture. Our proposed framework offers more direct control and observability over the fault injection process itself.
Why Not chaos-monkey-rs? A search of the Rust ecosystem reveals no prominent, maintained crate named chaos-monkey-rs that provides functionality equivalent to the original Netflix tool.25 Existing repositories with similar names are typically for unrelated projects, such as interpreters for the Monkey programming language.28 This lack of a ready-made, comprehensive solution reinforces the decision to build a tailored framework from more fundamental, composable libraries.
Complementing, Not Replacing, Infrastructure Tools: The decision to focus on code-native tools does not preclude the future use of infrastructure-level tools like Chaos Mesh or Litmus.29 These tools are excellent for answering questions about infrastructure resilience: "Does our Kubernetes deployment correctly restart a failed service?" or "Can the system handle a node failure?" Our proposed architecture answers questions about
application resilience: "Does our service handle a database timeout gracefully?" The two approaches are complementary. Once application resilience is validated, infrastructure-level chaos experiments can be layered on top to test the system's robustness as a whole.

Section 4: The Uveddi Chaos Experiment Catalog

This catalog serves as the central playbook for all chaos engineering activities within the Uveddi project. It provides a structured, actionable list of experiments designed to systematically probe the system's resilience. Each entry defines a specific hypothesis, the method of fault injection, and clear success criteria, transforming chaos engineering from an ad-hoc practice into a scientific discipline.

4.1 Experiment Structure

Every experiment in the catalog will be documented with the following attributes to ensure clarity, consistency, and reproducibility:
ID: A unique identifier for tracking (e.g., NET-01, DB-01).
Description: A concise summary of the failure scenario being simulated.
Hypothesis: A clear statement of the expected system behavior under duress, based on implemented resilience patterns.
Injection Method: The specific tool and technique from the proposed architecture (e.g., fail::fail_point!, ChaosLayer, turmoil simulation).
Blast Radius: The defined scope of the experiment's impact (e.g., "Single unit test," "10% of requests in staging environment").
Key Metrics: The primary metrics from the observability platform used to measure the impact and validate the outcome.
Success Criteria: A measurable, pass/fail condition for the experiment.
Stop/Rollback Procedure: The automated or manual steps to halt the experiment if it causes excessive deviation from the steady state.

4.2 Experiment Catalog Table

The following table presents a starter set of experiments tailored to the Uveddi architecture's most critical failure points. This catalog should be expanded over time as the system evolves.

ID
Description
Hypothesis
Injection Method
Blast Radius
Key Metrics
Success Criteria
NET-01
Rendering service is unavailable.
The backend's circuit breaker to the rendering service will open. API requests for diagrams will fail fast and return a fallback response. The system remains stable.
ChaosLayer configured to return an HTTP 503 error for 100% of requests.
Single API instance in test environment.
rendering_service.client.error_rate, circuit_breaker.state, api.endpoint.latency
Circuit breaker opens within 3 failed requests. Subsequent requests fail in <50ms. No panics in the backend.
NET-02
High latency from rendering service.
The backend's timeout logic will trigger. The circuit breaker will eventually open due to repeated timeouts. API requests will fail gracefully.
ChaosLayer configured to inject 2000ms of latency into 100% of requests.
Single API instance in test environment.
rendering_service.client.p99_latency, api.endpoint.error_rate (timeouts)
API returns a timeout error within the configured limit (e.g., 1000ms). No resource exhaustion (e.g., thread starvation) in the backend.
DB-01
SQLite connection is dropped during a transaction commit.
The analysis job will fail. The error is logged with context. The transaction is rolled back, ensuring no partial or corrupt data is written to the database.
fail::fail_point!("db_commit_error") inside sqlx transaction commit logic to return an I/O error.
Unit test scope.
db.transaction.error_count, log_output (for specific error message)
The test returns an Err. A post-test query confirms the transaction was rolled back and no data was committed.
DB-02
SQLite database file is corrupted.
The application fails to start or returns a clear error message on the first query attempt. It does not hang or enter an undefined state.
In a test environment, use PRAGMA writable_schema=1; to intentionally corrupt the schema of a test database file.31
Isolated test environment with a dedicated, corrupted DB file.
Application startup logs, health check endpoint status.
Application health check fails with a specific database-related error code.
RES-01
Memory pressure (OOM) during large file analysis.
The analysis process fails gracefully without crashing the entire service. The failure is logged, and the service remains available for other requests.
Run the Uveddi service in a container with a low memory limit (e.g., docker run --memory=128m) and provide a large input repository.32
Isolated container in CI.
Container exit code, host OOM killer logs, service availability metrics.
The container is killed by the OOM killer. The orchestrator (e.g., Docker Compose, Kubernetes) restarts it successfully. Other services are unaffected.
RES-02
CPU exhaustion during concurrent analysis.
The system's throughput may degrade, and latency may increase, but it remains available and does not crash. Requests are processed, albeit more slowly.
Run a load test with a high number of concurrent analysis jobs. A helper process can be used to generate synthetic CPU load on the host.35
Staging environment with load testing tools.
system.cpu.utilization, api.endpoint.p99_latency, api.request_queue.depth
System remains responsive (no dropped requests). Latency increases but stays within a defined SLO (e.g., < 5s). CPU utilization approaches 100% without causing a crash.
FS-01
Disk I/O error during code scanning.
The file scanning operation for a specific repository fails with a clear I/O error. The error is logged, and the overall service remains operational.
fail::fail_point!("fs_read_error") at a low-level file read operation to return a std::io::Error.36
Unit test scope.
log_output, analysis_job.status
The function attempting the read returns the correct io::Error. The calling code handles the Err variant gracefully.


Section 5: Observability, Measurement, and Validation

The ability to precisely measure the impact of chaos is what separates chaos engineering from simply breaking things. This section details the strategy for integrating experiments with Uveddi's existing tracing and metrics infrastructure to validate hypotheses and quantify system resilience.

5.1 Deep Integration with tracing and metrics

The cornerstone of our observability strategy is the tight integration of metrics and traces. By using the metrics-tracing-context crate, every metric emitted during an experiment will be automatically enriched with contextual labels from the active tracing::Span.37 This provides an exceptionally powerful way to analyze experiment results.
Implementation:
Setup: The application's main entry point will configure the tracing subscriber with a MetricsLayer and the metrics recorder with a TracingContextLayer. This one-time setup enables the automatic context propagation.37
Instrumentation: Each chaos experiment will be executed within a dedicated tracing::Span. This span will contain metadata about the experiment, such as its name, the type of fault, and its parameters.
Rust
// Example from a chaos test runner function
use tracing;
use metrics;

async fn run_latency_experiment() {
    // Create a span that describes the experiment.
    let experiment_span = tracing::info_span!(
        "chaos_experiment",
        name = "NET-02-latency",
        target_service = "rendering_service",
        latency_ms = 2000,
        probability = 1.0
    );
    // Enter the span. All code within this scope is part of the experiment.
    let _enter = experiment_span.enter();

    // Trigger an API call that will be subjected to the chaos middleware.
    let start_time = std::time::Instant::now();
    let result = api_client::request_diagram().await;
    let duration = start_time.elapsed();

    // This metric will now be automatically labeled with all fields from the span:
    // {..., name="NET-02-latency", target_service="rendering_service",...}
    metrics::histogram!("api.endpoint.diagram_request.latency", duration.as_millis() as f64);

    if result.is_err() {
        metrics::counter!("api.endpoint.diagram_request.errors", 1);
    }
}


Benefit: This approach allows for precise filtering and aggregation in monitoring dashboards. An engineer can select an experiment_name from a dropdown and immediately see the impact on latency, error rates, and resource usage for that specific experiment, isolating the signal from the noise of the overall system.3

5.2 Measuring System Recovery and User Impact

We will track a set of key metrics to quantify both the system's automated recovery capabilities and the impact on the end-user experience.3
Recovery Time Metrics:
Time to Detect (TTD): The duration from the moment a fault is injected to the moment the first corresponding alert is triggered by the monitoring system.
Time to Mitigate (TTM): The duration from detection until an automated resilience mechanism (e.g., circuit breaker opening, fallback activation) successfully engages. This is measured by observing state changes in the relevant metrics (e.g., a metric circuit_breaker_state changing from 0 to 1).
User Experience Metrics:
Latency: The p50, p95, and p99 response times for critical endpoints during an experiment.
Error Rate: The percentage of user-facing requests that result in an error (e.g., HTTP 5xx).
Throughput: The number of requests per second the system can handle, to detect degradation in capacity.

5.3 Data Integrity Validation

For any chaos experiment that involves the SQLite database, it is paramount to verify that no data is lost or corrupted.
Mechanism: A multi-step validation process will be implemented within the test harness.
Strategy:
Pre-computation: Before injecting a fault, the test will establish a known good state. This can involve calculating a checksum or hash of the relevant data set within the database.
Fault Injection: The experiment proceeds, injecting a fault like a simulated crash during a write operation using a fail failpoint.
Post-validation: After the system has recovered, a validation function runs. It re-computes the checksum and compares it to the pre-experiment value. Any mismatch indicates data corruption. For transactional tests, the validation function will query the database to ensure that a failed transaction was fully rolled back and left no partial data artifacts. For more advanced corruption scenarios, we can use PRAGMA writable_schema = 1; in a tightly controlled test environment to directly manipulate the database file and test the application's ability to handle malformed data.31

5.4 Monitoring Dashboards and Alerting

Dashboard Design: A dedicated "Chaos Engineering" dashboard will be created in the project's monitoring tool (e.g., Grafana). The dashboard will be templated with a variable for experiment_name, populated by the labels from our tracing spans. This will allow any team member to select a running or past experiment and view a curated set of relevant system health metrics.
Safety-Net Alerting: Specific alerting rules will be configured to act as a safety mechanism for the experiments themselves. For example, an alert will be triggered if a chaos experiment causes the system's error rate to exceed its SLO for more than 60 seconds. This "meta-alert" signals that the experiment has uncovered a critical vulnerability and should be automatically halted.

Section 6: CI/CD Integration and Automation Strategy

To maximize the value of chaos engineering, it must be transformed from a periodic, manual exercise into an automated, continuous quality gate integrated directly into the development workflow. This section details the plan for embedding chaos experiments into the Uveddi project's GitHub Actions pipeline.
This approach ensures that resilience is treated as a first-class feature, validated on every change. When a pull request is opened, it is tested not only for correctness ("does it work?") but also for robustness ("does it break gracefully?"). This automated feedback loop is the most effective way to build a culture of resilience and ensure the Uveddi project becomes robust by design, not by accident.1

6.1 GitHub Actions Workflow Design

A new workflow file, .github/workflows/chaos-ci.yml, will be created to orchestrate the chaos tests. This workflow will build upon standard Rust CI practices and integrate dedicated steps for chaos experiments.30
Workflow Triggers:
On pull_request events targeting the main branch.
On a schedule (e.g., nightly) for more comprehensive, longer-running tests.
Key Jobs:
build_and_unit_test: A standard job that runs cargo build and cargo test to ensure basic correctness.
chaos_unit_tests (on pull_request): This job focuses on fast, low-risk, unit-level chaos tests.
It will run a specific test binary dedicated to failpoint tests (e.g., tests/chaos_failpoints.rs).
It will enable the necessary feature flag: cargo test --test chaos_failpoints --features "fail/failpoints".
The tests within this suite will programmatically enable and disable specific failpoints using fail::cfg() to validate individual error-handling paths.
chaos_integration_tests (on schedule): This job orchestrates system-level experiments against a full environment.
It uses Docker Compose to spin up the entire Uveddi stack (Rust backend, Node.js renderer, SQLite volume).
It runs a dedicated test orchestrator binary that makes API calls to the running services.
It enables chaos modes (e.g., latency injection via the ChaosLayer) by setting environment variables that are read by the ChaosConfig module.
It executes a suite of experiments from the catalog, collects metrics, and validates outcomes.
Finally, it publishes a detailed report as a GitHub Actions artifact.
A simplified workflow might look like this:

YAML


#.github/workflows/chaos-ci.yml
name: Chaos Engineering CI

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]
  schedule:
    - cron: '0 2 * * *' # Run nightly at 2 AM UTC

jobs:
  # Standard build and test job
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build
        run: cargo build --verbose
      - name: Run tests
        run: cargo test --verbose

  # Fast, unit-level chaos tests for every PR
  chaos_unit_tests:
    runs-on: ubuntu-latest
    if: github.event_name == 'pull_request'
    steps:
      - uses: actions/checkout@v4
      - name: Run failpoint tests
        # This test binary contains tests that use fail::cfg()
        run: cargo test --test chaos_failpoints --features "fail/failpoints"

  # Slower, integration-level chaos tests run on a schedule
  chaos_integration_tests:
    runs-on: ubuntu-latest
    if: github.event_name == 'schedule'
    steps:
      - uses: actions/checkout@v4
      - name: Set up Docker Compose
        #... setup steps...
      - name: Run integration chaos suite
        env:
          # Enable chaos experiments via environment variables
          UVEDDI_CHAOS_MODE: "LATENCY"
          UVEDDI_CHAOS_LATENCY_MS: "500"
          UVEDDI_CHAOS_PROBABILITY: "0.1"
        run: cargo run --bin chaos_test_runner
      - name: Publish report
        uses: actions/upload-artifact@v4
        with:
          name: chaos-report
          path: target/chaos_report.json



6.2 Strategy for Gradual Rollout

The introduction of chaos testing will follow a phased approach to minimize disruption and build confidence:
Pull Requests: Initially, only the fast and deterministic chaos_unit_tests job will run on PRs. This provides immediate feedback on resilience regressions in core logic without significantly increasing CI time.
Nightly Staging Runs: The chaos_integration_tests job will run nightly against a dedicated staging environment. This is where more disruptive experiments, such as latency injection and service unavailability, will be performed.
Manual "Game Days": Before major releases, the team will conduct "Game Days." These are planned sessions where engineers manually trigger more extreme or exploratory chaos experiments in the staging environment. This not only tests the system but also allows the team to practice their incident response procedures in a safe setting.

6.3 Reporting and Actionable Insights

The outcome of chaos testing must be visible and actionable.
CI Status: A failed chaos test will fail the overall CI check, blocking the pull request from being merged. This treats a resilience failure with the same severity as a compilation error or a failed unit test.
Reports: The JSON reports generated by the integration test suite will be uploaded as artifacts, providing a detailed record of which experiments were run, what the observed impact on key metrics was, and whether the outcome matched the hypothesis.
Trend Analysis: Over time, the results from these automated runs will create a historical record of the system's resilience. This data can be used to track improvements and demonstrate the value of the chaos engineering practice.
Works cited
Chaos Engineering: Testing for Resilience in Distributed Systems | by Lokesh Prasad, accessed July 19, 2025, https://medium.com/@lokeshprasad125/chaos-engineering-testing-for-resilience-in-distributed-systems-17aad9393b39
Chaos Engineering: An Approach to Resilience in the System | by Sandeep Kaushik, accessed July 19, 2025, https://medium.com/@shyamsandeep28/chaos-engineering-an-approach-to-resilience-in-the-system-826aeda5255d
How to Build Observability into Chaos Engineering - Last9, accessed July 19, 2025, https://last9.io/blog/how-to-build-observability-into-chaos-engineering/
Chaos engineering - O'Reilly Media, accessed July 19, 2025, https://www.oreilly.com/content/chaos-engineering/
Chaos Engineering Upgraded. Chaos Kong is the most destructive… | by Netflix Technology Blog | Netflix TechBlog, accessed July 19, 2025, https://netflixtechblog.com/chaos-engineering-upgraded-878d341f15fa
Chaos Engineering for Microservices - DZone, accessed July 19, 2025, https://dzone.com/articles/chaos-engineering-for-microservices
A curated list of awesome Chaos Engineering resources - GitHub, accessed July 19, 2025, https://github.com/dastergon/awesome-chaos-engineering
Chaos Engineering - Gremlin, accessed July 19, 2025, https://www.gremlin.com/chaos-engineering
Chaos Conf Q&A: The Benefits, Challenges and Practices of Chaos Engineering - InfoQ, accessed July 19, 2025, https://www.infoq.com/articles/chaos-engineering-conf/
fail - Rust - tikv, accessed July 19, 2025, https://tikv.github.io/doc/fail/index.html
tikv/fail-rs: Fail points for rust - GitHub, accessed July 19, 2025, https://github.com/tikv/fail-rs
failpoints - Rust - Docs.rs, accessed July 19, 2025, https://docs.rs/failpoints
tower-fault-injector - crates.io: Rust Package Registry, accessed July 19, 2025, https://crates.io/crates/tower-fault-injector
accessed December 31, 1969, https://github.com/nmoutschen/tower-fault-injector
axum::middleware - Rust - Docs.rs, accessed July 19, 2025, https://docs.rs/axum/latest/axum/middleware/index.html
Creating a Rate Limiter Middleware using Tower for Axum (Rust) | by Khalid Ali - Medium, accessed July 19, 2025, https://medium.com/@khalludi123/creating-a-rate-limiter-middleware-using-tower-for-axum-rust-be1d65fbeca
Layer in tower - Rust - Docs.rs, accessed July 19, 2025, https://docs.rs/tower/latest/tower/trait.Layer.html
Deterministic simulation testing for async Rust - S2.dev, accessed July 19, 2025, https://s2.dev/blog/dst
Deterministic Simulation Testing on every commit - Astradot Blog, accessed July 19, 2025, https://blog.astradot.com/deterministic-simulation-testing-on-every-commit-2/
Deterministic simulation testing for async Rust - Reddit, accessed July 19, 2025, https://www.reddit.com/r/rust/comments/1jr8ogo/deterministic_simulation_testing_for_async_rust/
tower - Rust - Docs.rs, accessed July 19, 2025, https://docs.rs/tower
Integrate Chaos Mesh to GitHub Actions, accessed July 19, 2025, https://chaos-mesh.org/docs/next/integrate-chaos-mesh-into-github-actions/
kaos - Rust - Docs.rs, accessed July 19, 2025, https://docs.rs/kaos
vertexclique/kaos: Chaotic Testing Harness - GitHub, accessed July 19, 2025, https://github.com/vertexclique/kaos
Home - Chaos Monkey, accessed July 19, 2025, https://netflix.github.io/chaosmonkey/
CHAOS MONKEY: A BEGINNER'S GUIDE - BMC Blogs, accessed July 19, 2025, https://blogs.bmc.com/chaos-monkey/?print-posts=pdf
Netflix/chaosmonkey: Chaos Monkey is a resiliency tool that helps applications tolerate random instance failures. - GitHub, accessed July 19, 2025, https://github.com/Netflix/chaosmonkey
hatashiro/monkey-rs: An interpreter for Monkey with parser combinator written in Rust, accessed July 19, 2025, https://github.com/hatashiro/monkey-rs
litmuschaos/github-chaos-actions: Github actions to trigger chaos on your review apps, accessed July 19, 2025, https://github.com/litmuschaos/github-chaos-actions
Integrate Chaos Mesh to GitHub Actions, accessed July 19, 2025, https://chaos-mesh.org/docs/integrate-chaos-mesh-into-github-actions/
How to simulate a corrupt SQLite database - Stack Overflow, accessed July 19, 2025, https://stackoverflow.com/questions/31924510/how-to-simulate-a-corrupt-sqlite-database
How to simulate OutOfMemory exception - Stack Overflow, accessed July 19, 2025, https://stackoverflow.com/questions/2782328/how-to-simulate-outofmemory-exception
How do I debug a memory issue in Rust? - Stack Overflow, accessed July 19, 2025, https://stackoverflow.com/questions/38254937/how-do-i-debug-a-memory-issue-in-rust
Fixing Memory Leaks in Rust - OneSignal, accessed July 19, 2025, https://onesignal.com/blog/solving-memory-leaks-in-rust/
Best way to peg the CPU at 100%? : r/rust - Reddit, accessed July 19, 2025, https://www.reddit.com/r/rust/comments/1fxpoq0/best_way_to_peg_the_cpu_at_100/
Error in std::io - Rust, accessed July 19, 2025, https://doc.rust-lang.org/std/io/struct.Error.html
metrics_tracing_context - Rust - Docs.rs, accessed July 19, 2025, https://docs.rs/metrics-tracing-context
metrics_tracing_context - Rust - Docs.rs, accessed July 19, 2025, https://docs.rs/metrics-tracing-context/latest/metrics_tracing_context/
Building and testing Rust - GitHub Docs, accessed July 19, 2025, https://docs.github.com/en/actions/how-tos/use-cases-and-examples/building-and-testing/building-and-testing-rust
GitHub Chaos Actions in Your CI/CD workflow [Part-1] - DEV Community, accessed July 19, 2025, https://dev.to/litmus-chaos/github-chaos-actions-in-your-ci-cd-workflow-mke
