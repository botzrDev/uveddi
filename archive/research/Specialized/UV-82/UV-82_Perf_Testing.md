
A Framework for High-Concurrency Performance Testing of the Uveddi System


Part 1: Foundational Architecture and Tooling

This section establishes the foundational architecture and technology stack for a robust, high-concurrency load testing framework tailored to the Uveddi project. The selections prioritize developer velocity, ecosystem maturity, and seamless integration with modern CI/CD and cloud-native practices, ensuring the system can be implemented within the required 2-3 week timeline while meeting the project's stringent performance and scalability goals.

1.1 Selecting the Load Generation Engine: A Pragmatic Choice for Developer Velocity

The selection of a load generation engine is the cornerstone of any performance testing framework. The ideal tool must not only be capable of generating the required load (1000+ concurrent users) but must also be resource-efficient, easy to script, and possess a mature ecosystem that accelerates, rather than hinders, integration with orchestration and automation systems.

1.1.1 Analysis of Leading Load Testing Tools

A comparative analysis of the leading tools reveals distinct trade-offs between performance, ease of use, and ecosystem support.
Rust-Native Tools (Goose, Drill): Tools like Goose, inspired by Locust, offer exceptional performance, leveraging Rust's efficiency to generate significantly more traffic per CPU core than alternatives like Locust. User behavior is defined in standard Rust code, providing maximum power and flexibility for teams proficient in the language.2 Drill is another lightweight, Rust-based tool that uses YAML for test definitions, appealing for its simplicity and minimal dependencies.4 However, while performant, these tools have a less mature ecosystem. For instance, Goose's distributed testing mode, "Gaggle," requires more manual setup, and its UI capabilities are still under development.1 For a project with a tight implementation timeline, the need to build custom integrations for orchestration and reporting can introduce significant overhead.
Industry-Standard Tools (JMeter, Gatling): Apache JMeter is a long-standing, feature-rich tool with a vast community and extensive protocol support.7 However, its Java-based architecture and GUI-centric approach can be resource-intensive, consuming significantly more memory than modern alternatives for simple scripts.9 Gatling offers excellent performance with its Scala-based DSL and non-blocking architecture, making it highly efficient for simulating thousands of users.10 Its code-based approach is well-suited for CI/CD integration.11 The primary drawback is the requirement for proficiency in Scala, which may present a learning curve for teams not already using the language.
Modern, Developer-Centric Tools (k6, Artillery): k6, written in Go and using JavaScript for test scripting, has emerged as a leader in the developer-centric testing space.12 It is designed for high performance, capable of generating substantial load from a single instance, and is demonstrably more resource-efficient than tools like JMeter and Artillery in various benchmarks.9 Its key strength lies in its "tests as code" philosophy, robust JavaScript API, and, most critically, its mature ecosystem of extensions and integrations.13 Artillery is another modern tool using JavaScript/YAML for scripting, but benchmarks suggest it can be less performant and more resource-intensive under high load compared to k6.9

1.1.2 Recommendation and Justification: Grafana k6

Grafana k6 is the recommended load generation engine for the Uveddi project.
This recommendation is based on a pragmatic evaluation that prioritizes not just raw performance but the overall "time to value." The Uveddi project's 2-3 week timeline for full capability necessitates a tool that minimizes custom engineering effort. k6 excels in this regard due to its unparalleled combination of performance, developer-friendly scripting, and, most importantly, its mature, officially supported integrations for Kubernetes and CI/CD.
The choice of a load testing tool is an exercise in risk management. While a Rust-native tool like Goose is technically appealing, its less-developed ecosystem for distributed testing and automation presents a project risk. The team would need to invest time in building the surrounding infrastructure. Conversely, k6 provides pre-built, production-ready solutions for the most complex aspects of this task:
Distributed Testing on Kubernetes: The official k6-operator provides a declarative, Kubernetes-native way to manage large-scale distributed tests.14
CI/CD Automation: The official run-k6-action for GitHub Actions simplifies the process of embedding load tests directly into the development workflow.16
By selecting k6, the Uveddi team can focus its efforts on crafting high-quality, realistic test scenarios rather than building and maintaining the underlying testing infrastructure. The familiarity of JavaScript for scripting also lowers the barrier to entry, enabling the entire development team to contribute to performance testing efforts.12

Table 1: Load Generation Tool Comparison


Criterion
Grafana k6
Goose
Gatling
Apache JMeter
Scripting Language
JavaScript (ES2015+)
Rust
Scala, Java, Kotlin
GUI (XML), Groovy
Performance
Excellent; high throughput with low resource usage.9
Exceptional; leverages Rust's performance, outperforming Locust.
Excellent; based on a non-blocking, asynchronous architecture.10
Good, but can be resource-intensive (CPU/Memory).9
Kubernetes Integration
Excellent; official k6-operator for declarative, distributed testing.14
Manual; requires orchestrating "Gaggle" workers as pods.19
Good; Kubernetes plugin available for Gatling Enterprise.10
Manual; requires managing master/slave pods and custom configurations.
CI/CD Support
Excellent; official actions for GitHub, GitLab, etc..16
Good; CLI-driven nature is CI-friendly, but requires custom scripting.21
Excellent; designed for automation and integrates well with CI pipelines.10
Good; integrates with Jenkins, Maven, etc., but can be complex.7
Reporting
Good; detailed console summary, JSON output, and seamless integration with Grafana.22
Good; detailed console and HTML reports.23
Excellent; detailed, visually rich HTML reports out of the box.10
Very Good; extensive reporting options via listeners and plugins.7
Ecosystem & Community
Strong and growing; backed by Grafana Labs, with a large extension ecosystem.13
Growing; backed by Tag1 Consulting, but smaller community.3
Strong; well-established with both open-source and enterprise offerings.10
Massive; largest and oldest community with extensive plugins.8


1.2 Orchestration for Distributed Load: Kubernetes as the De Facto Standard

Simulating 1000+ concurrent users requires a distributed architecture where multiple load generator instances work in concert. A container orchestration platform is essential for managing the lifecycle of these instances.

1.2.1 Analysis: Kubernetes vs. Docker Swarm

While both Kubernetes and Docker Swarm are capable container orchestrators, Kubernetes has become the de facto industry standard for complex, large-scale deployments, making it the superior choice for this framework.
Docker Swarm: Offers simplicity and a gentle learning curve, especially for teams already heavily invested in the Docker ecosystem. Its setup is straightforward, and it handles basic service discovery, load balancing, and scaling effectively.24 However, its feature set is comparatively limited. It lacks the advanced networking policies, sophisticated storage orchestration (Persistent Volumes/Claims), and, most importantly, the extensibility model that Kubernetes provides.26
Kubernetes: Is a more complex but vastly more powerful and flexible platform. It provides robust, battle-tested features for autoscaling, self-healing, and service discovery. Its key differentiator is its extensibility through Custom Resource Definitions (CRDs) and the Operator pattern.26 This allows the Kubernetes API to be extended to manage third-party applications and complex workflows as native resources.

1.2.2 Recommendation and Justification: Kubernetes with the k6-operator

Kubernetes is the recommended orchestration platform. The decision is driven by the availability of the k6-operator, which leverages Kubernetes' CRD and Operator patterns to provide a superior testing workflow.
Adopting the Kubernetes Operator pattern fundamentally changes how load tests are managed. Instead of relying on imperative shell scripts to start and coordinate distributed tests, the process becomes declarative. This shift offers profound benefits for automation, repeatability, and maintainability.
Declarative Test Definition: The k6-operator introduces a new Kubernetes resource type, the TestRun.14 A test is defined entirely within a YAML manifest file. This file specifies the number of parallel load generator pods, the test script to run (typically from a
ConfigMap), and any command-line arguments.
GitOps Alignment: This TestRun manifest becomes a version-controlled artifact. It can be stored in the same Git repository as the application code, reviewed in pull requests, and deployed using the same CI/CD pipelines. This aligns performance testing with modern GitOps practices, treating infrastructure and test configurations as code.
Simplified CI/CD: The role of the CI/CD pipeline is dramatically simplified. Instead of containing complex logic to manage test execution, the pipeline's primary task becomes applying the manifest to the cluster: $ kubectl apply -f k6-testrun.yaml. The operator handles all the underlying complexity of creating pods, distributing the script, running the test, and aggregating results.
This declarative, GitOps-aligned approach makes the entire performance testing process more robust, transparent, and easier to manage at scale, directly addressing the user's requirement for a system that integrates with the existing CI/CD pipeline.

1.3 The Observability Stack: A Multi-Layered Approach

Effective bottleneck analysis requires a comprehensive observability stack that can correlate performance data from multiple layers of the system. A single tool is insufficient; a holistic view is necessary to trace a symptom (e.g., high latency) back to its root cause (e.g., a blocked event loop or a slow database query).
The proposed stack combines industry-standard metrics collection with specialized, deep-dive tooling for the Rust and Node.js runtimes.
Core Metrics and Visualization:
Prometheus: The industry standard for metrics collection in cloud-native environments. It will be deployed within the Kubernetes cluster to scrape metrics from all system components.28 Its pull-based model is highly scalable and resilient.
Grafana: The premier tool for visualizing time-series data from Prometheus and other sources. It will be used to create a unified dashboard that correlates metrics from the load generator, application services, and infrastructure.29
Application Instrumentation: Each service in the Uveddi architecture must be instrumented to expose its internal state as Prometheus metrics.
Rust Backend (tokio): The metrics-exporter-prometheus crate provides a straightforward way to create a /metrics endpoint in a Rust application.31 This will be used to expose custom business metrics (e.g., analysis requests processed, analysis duration histogram). Crucially, the
tokio-metrics crate will be used to export detailed metrics directly from the tokio runtime, providing visibility into scheduler behavior, task poll counts, and queue depths.33 This data is essential for diagnosing async-specific performance issues.
Node.js Rendering Service: The prom-client library is the standard for instrumenting Node.js applications.28 It provides default metrics out of the box, including critical indicators of Node.js health like event loop lag, garbage collection statistics, and heap memory usage.35 Custom metrics will be added to track the duration and success/failure rate of Mermaid diagram rendering operations.
Deep-Dive Diagnostic Tooling: While Prometheus and Grafana provide the "what," specialized tools are needed to understand the "why."
tokio-console: This is an indispensable tool for debugging and profiling tokio-based applications. It provides a real-time, htop-like view of all running async tasks, their poll times, and their scheduled states.37 It is the only tool that can effectively diagnose issues like task starvation, where a long-running, non-yielding task blocks the executor, a problem invisible to traditional CPU profilers.39
perf and flamegraph: For offline analysis of CPU-bound bottlenecks in the Rust backend, the combination of Linux perf and flamegraph is the standard approach.40
perf samples the application's call stack at a high frequency, and flamegraph visualizes this data, making it immediately obvious which functions are consuming the most CPU time under load.
A complete performance investigation workflow involves using these tools in concert. An alert in Grafana showing high P99 latency might trigger a deeper investigation. The engineer would first check the application metrics: Is the Node.js event loop lagging? Are the tokio task queues growing? If the issue appears to be CPU-bound in the Rust service, they would then use perf and flamegraph to pinpoint the exact function. If it's an async scheduling issue, tokio-console would be used to observe task behavior in real-time. This multi-layered approach is essential for efficiently diagnosing and resolving complex performance problems.

Part 2: Infrastructure Design and Implementation for Scalable Testing

This section details the cloud infrastructure architecture required to support the high-concurrency testing framework. The design focuses on leveraging managed Kubernetes services, implementing dynamic autoscaling, and employing cost-optimization strategies to create a powerful, efficient, and affordable testing environment.

2.1 Cloud Provider Analysis: EKS vs. GKE vs. AKS

The choice of a managed Kubernetes provider is a key decision that impacts cost, operational overhead, and available features. All three major cloud providers—Amazon Web Services (AWS), Google Cloud Platform (GCP), and Microsoft Azure—offer mature, production-grade services.
Cost Structure: The primary cost components are the cluster management fee and the compute resources (worker nodes).
Management Fee: Both Amazon EKS and Google Kubernetes Engine (GKE) charge a flat fee of approximately $0.10 per hour per cluster (roughly $72/month).41 Azure Kubernetes Service (AKS) notably offers the control plane for free, which can be a significant cost saving, especially for environments with many small clusters.41 GKE also provides a free tier for one zonal cluster, making it cost-effective for smaller initial setups.41
Compute Costs: Worker node pricing varies based on instance type (VM family), size, and region. All providers offer significant discounts (up to 70-90%) for using Spot or Preemptible instances, which are ideal for fault-tolerant, transient workloads like load test generators.43
Networking and Other Costs: Often-overlooked costs include load balancers, persistent storage, and network egress (data transfer out of the cloud or between availability zones).45 These can become significant factors in a high-traffic testing scenario.
Feature Comparison:
AWS EKS: Tightly integrated with the AWS ecosystem, offering robust IAM integration and access to a wide range of AWS services.
Azure AKS: Strong integration with Azure DevOps and other Microsoft services. Its free control plane is a compelling financial advantage.42
GCP GKE: Often praised for its mature and user-friendly autoscaling capabilities and its Autopilot mode, which further abstracts away node management. Historically, its pricing for preemptible VMs has been very competitive and predictable.44

2.1.1 Recommendation and Justification

While all three providers are viable, Google Kubernetes Engine (GKE) is recommended as the starting point, assuming no strong pre-existing organizational preference. This recommendation is based on its combination of a free tier for the initial cluster, mature autoscaling, and a strong developer experience. However, the architectural principles and configurations outlined in this report are fundamentally cloud-agnostic and can be readily adapted to EKS or AKS. The most critical factor for cost optimization—the use of spot instances for load generators—is available on all platforms.

Table 2: Managed Kubernetes Service Pricing and Feature Comparison (Illustrative)


Feature
Amazon EKS (AWS)
Azure AKS (Azure)
Google Kubernetes Engine (GCP)
Management Fee
~$0.10 / hour / cluster 41
Free 42
Free (1st zonal cluster), then ~$0.10 / hour / cluster 41
Worker Node Pricing
On-demand, Reserved, and Spot Instances. Pricing is highly variable.
On-demand, Reserved, and Spot VMs. Pricing is highly variable.
On-demand, Committed Use, and Preemptible VMs. Pricing is highly variable.
Spot Instance Savings
Up to 90% 44
Up to 90%
Up to 80% 44
Autoscaling
Cluster Autoscaler
Cluster Autoscaler
Cluster Autoscaler, Node Auto-Provisioning, Autopilot
Integrated Load Testing
AWS Distributed Load Testing Solution (JMeter-based) 48
Azure Load Testing (JMeter/Locust-based) 50
No native service; recommends third-party tools like JMeter.52


2.2 Kubernetes Cluster Configuration for Load Generation

The Kubernetes cluster will be designed with a clear separation of concerns to ensure both stability and cost-efficiency. This involves creating distinct node pools for different types of workloads.
Architectural Design:
A single Virtual Private Cloud (VPC) will house the entire testing infrastructure.
The cluster will be configured with two distinct node pools:
control-plane-pool: A small, fixed-size pool (e.g., 2-3 nodes) using reliable on-demand instances. This pool will run critical cluster components like CoreDNS, the Kubernetes API server components, the Prometheus/Grafana stack, and the k6-operator controller itself. Its stability is paramount.
load-generator-pool: A dynamically scaling node pool configured to use Spot/Preemptible Instances. This pool will be exclusively used for running the k6 test runner pods. It will be configured with a minimum size of 0 and a large maximum size (e.g., 20+ nodes). Taints and tolerations will be used to ensure that only k6 pods are scheduled onto these nodes.
An Application Load Balancer (ALB) or cloud-native Ingress controller will be configured to expose the Uveddi application under test to the internet so the load generators can target it.
This two-pool design isolates the volatile, short-lived load generator pods from the essential, long-running management components, preventing test workloads from impacting the stability of the monitoring and orchestration systems.

2.3 Implementing Distributed Testing with the k6 Operator

With the cluster in place, the k6-operator can be installed to manage the execution of distributed tests.
Installation: The operator is installed via a single command using its official manifest bundle or Helm chart. This creates the necessary CRDs (TestRun, PrivateLoadZone) and deploys the operator's controller manager into the cluster.14
Bash
# Install k6-operator using the official bundle
kubectl apply -f https://raw.githubusercontent.com/grafana/k6-operator/main/bundle.yaml


Test Script Management: The k6 test script (e.g., test.js) and any small associated data files are packaged into a Kubernetes ConfigMap. This makes the script available within the cluster to be mounted by the test runner pods.15
Bash
# Create a ConfigMap from the test script file
kubectl create configmap k6-test-script --from-file=test.js


Defining the TestRun: A test is defined using a TestRun custom resource. This YAML file is the declarative heart of the test execution.
YAML
# k6-testrun.yaml
apiVersion: k6.io/v1alpha1
kind: TestRun
metadata:
  name: uveddi-high-concurrency-test
spec:
  # Number of distributed k6 pods to run in parallel
  parallelism: 10
  script:
    configMap:
      # Name of the ConfigMap containing the script
      name: k6-test-script
      # Filename within the ConfigMap
      file: test.js
  # Optional: Define environment variables or CLI arguments
  arguments: "--vus 1000 --duration 10m"
  runner:
    # Ensure k6 pods are scheduled onto the spot instance pool
    tolerations:
    - key: "instance_type"
      operator: "Equal"
      value: "spot"
      effect: "NoSchedule"


Execution: Triggering a large-scale, distributed test across 10 pods is reduced to a single, simple command:
Bash
kubectl apply -f k6-testrun.yaml

The operator will observe the creation of this resource and automatically orchestrate the deployment of 10 k6 pods, each running the specified script.

2.4 Dynamic Resource Scaling and Cost Optimization

The combination of the k6-operator and the Kubernetes Cluster Autoscaler creates a highly efficient, "just-in-time" testing infrastructure that directly addresses the user's requirement for a cost-effective solution.
The process is fully automated:
Test Trigger: A developer or CI/CD pipeline executes $ kubectl apply -f k6-testrun.yaml.
Pod Scheduling: The Kubernetes scheduler sees 10 new k6 pods defined by the TestRun resource. It attempts to schedule them.
Resource Demand: The load-generator-pool is initially at or near 0 nodes. The scheduler finds there is insufficient capacity to run the 10 pods and marks them as Pending.
Autoscaler Activation: The Cluster Autoscaler detects the Pending pods. It calculates the required resources and automatically provisions new Spot/Preemptible nodes in the load-generator-pool to meet the demand.54
Test Execution: As the new nodes join the cluster, the k6 pods are scheduled onto them and begin executing the load test.
Test Completion: The TestRun completes, and the k6 pods are terminated.
Scale-Down: The Cluster Autoscaler now observes that the nodes in the load-generator-pool are underutilized. After a configurable grace period, it terminates the nodes, scaling the pool back down to zero.
This dynamic lifecycle ensures that the expensive compute resources for load generation are only provisioned for the exact duration of a test run. When idle, the testing infrastructure's cost is minimal, consisting only of the small, on-demand control-plane-pool. This approach, especially when combined with the deep discounts of spot instances, makes even very large-scale testing financially viable.43

Part 3: Crafting Realistic and High-Impact Test Scenarios

The value of a load test is directly proportional to its realism. An unrealistic test that simply bombards an endpoint with stateless requests may identify some bottlenecks but will fail to uncover issues related to user sessions, data contention, and complex workflows. This section details the design and implementation of k6 test scenarios that accurately model the behavior of Uveddi users at scale.

3.1 Modeling the Uveddi User Journey

A realistic test must simulate a complete user journey, not just isolated API calls. This involves managing state, handling authentication, and sequencing actions in a logical flow. The proposed Uveddi user journey will be modeled as a multi-step k6 scenario, drawing inspiration from the structured scenario design practices used in tools like Gatling.56
Core User Journey:
Authentication (setup function): Before the main test execution begins, a setup function will run once. This function will perform a login against an authentication endpoint to acquire a pool of JWTs or API tokens. This pre-authentication step ensures that the main test VUs are not burdened with login logic and can focus on simulating application behavior.
Stateful VU Scenario (default function): Each Virtual User (VU) will execute the default function in a loop, simulating a stateful workflow:
File Upload: The VU selects a source code file (from a predefined set) and uploads it to the analysis endpoint.
Analysis Polling: The VU makes periodic requests to a status endpoint to check if the analysis is complete, simulating a user waiting for a background job to finish.
Result Fetch: Once the analysis is complete, the VU requests the structured analysis results.
Diagram Rendering: The VU then requests a Mermaid diagram rendering of the project's architecture from the Node.js service.
Modeling Mixed Workloads with k6 Scenarios: Real-world traffic is rarely uniform. The k6 scenarios feature allows for the modeling of complex, mixed workloads where different user types perform different actions concurrently.13
JavaScript
// k6 script options
export const options = {
  scenarios: {
    // 80% of VUs will perform the full analysis workflow
    full_analysis_workflow: {
      executor: 'ramping-vus',
      exec: 'analysisJourney', // A dedicated function for this journey
      stages:,
    },
    // 20% of VUs will only re-render existing diagrams
    diagram_rerender_workflow: {
      executor: 'ramping-vus',
      exec: 'renderOnlyJourney',
      startTime: '1m', // Start this scenario 1 minute into the test
      stages: [
        { duration: '4m', target: 200 },
        { duration: '10m', target: 200 },
        { duration: '2m', target: 0 },
      ],
    },
  },
};



3.2 Simulating High-Concurrency Workloads

To accurately assess the system's breaking points, the test must simulate realistic load patterns, including gradual increases, sustained peaks, and sudden spikes.
Load Profiles with stages: The k6 options.stages array is used to define the load profile over time. A comprehensive test should include multiple stages 59:
Ramp-up: A gradual increase in VUs allows the system (and its autoscaling mechanisms) to warm up. A steep ramp-up can help identify issues with cold starts or resource initialization.
Sustained Load: A prolonged period at the target load (e.g., 1000 VUs for 10-15 minutes) is essential for identifying memory leaks, performance degradation over time, and thermal throttling.
Spike Test: A short, intense burst of traffic (e.g., ramping to 2000 VUs for 1 minute) tests the system's resilience and its ability to recover from sudden surges.
Ramp-down: A gradual decrease allows for observation of how the system scales down resources.
Simulating User "Think Time": Real users do not issue requests as fast as a machine can. They pause to read, type, or think. The sleep() function in k6 is critical for simulating this "think time".61 Adding random delays (e.g.,
sleep(Math.random() * 5 + 1);) between requests makes the load pattern less uniform and more realistic, preventing artificial bottlenecks caused by perfectly synchronized requests. Without think time, a test measures the performance of the system under a denial-of-service attack, not under a realistic user load.
Data Parameterization: To avoid unrealistic caching at the database, CDN, or application level, each VU should operate on different data. The k6 SharedArray is an efficient mechanism for loading test data (e.g., a list of user credentials or filenames from a JSON or CSV file) once in the init context and sharing it in memory across all VUs.62 This ensures that VUs can randomly select unique users and files for each iteration, creating a much more realistic and challenging load.

3.3 Code Implementation: Stateful File Uploads and Authentication in k6

The Uveddi project's requirement to test file uploads up to 100MB presents a significant technical challenge in a distributed Kubernetes environment that requires a specific architectural solution.

3.3.1 The Large File Data Challenge

The Need: To run a distributed test with 10 parallel k6 pods, each pod must have access to the large (1KB to 100MB) test files.
The Standard Approach: The common method for providing configuration and data to pods in Kubernetes is via ConfigMaps.15
The Limitation: ConfigMaps have a hard size limit of 1MB per entry.63 This makes them completely unsuitable for distributing the 100MB test files required by the Uveddi project. Attempting to use this standard method would lead to immediate failure.

3.3.2 The initContainer Solution

The correct and scalable solution is to use an initContainer in the TestRun pod specification. An initContainer is a special container that runs to completion before the main application container (in this case, k6) starts. It can be used to prepare the environment, such as by downloading necessary data.
The workflow is as follows:
Large test files are stored in a cloud object store (e.g., AWS S3, Google Cloud Storage).
The TestRun manifest is configured with an initContainer that uses a lightweight image (e.g., alpine) and a command-line tool (wget, curl, or a cloud-specific CLI).
A shared emptyDir volume is mounted to both the initContainer and the main k6 container.
When the test pod starts, the initContainer runs first. It downloads the large test files from the object store into the shared volume.
Once the initContainer completes successfully, the main k6 container starts. It now sees the large test files in the shared volume as if they were on its local filesystem and can use them in the test script.

3.3.3 Implementation Example

Below is a combined k6 script and TestRun manifest that implements the full workflow, including authentication, data parameterization, file uploads, and the initContainer pattern for large files.
k6 Script (test.js):

JavaScript


import http from 'k6/http';
import { check, sleep } from 'k6';
import { SharedArray } from 'k6/data';

// Load user credentials from a JSON file (mounted from the initContainer)
const users = new SharedArray('users', function () {
  return JSON.parse(open('/test-data/users.json')).users;
});

// Load a list of test files to upload
const testFiles =;

// Read the binary data for the large file in the init context
const largeFileBin = open(testFiles.path, 'b');

export const options = {
  scenarios: {
    //... defined in section 3.1...
  },
  thresholds: {
    'http_req_failed': ['rate<0.01'],
    'http_req_duration': ['p(95)<100'],
  },
};

// 1. Setup: Login once to get an auth token for a test run
let authToken;
export function setup() {
  const loginRes = http.post(`${__ENV.API_HOST}/login`, {
    username: 'testrunner',
    password: 'supersecretpassword',
  });
  check(loginRes, { 'login successful': (r) => r.status === 200 });
  return { token: loginRes.json('access_token') };
}

// 2. Main VU function for the analysis journey
export function analysisJourney(data) {
  const user = users[__VU % users.length]; // Pick a unique user for this VU
  const fileToUpload = testFiles; // Cycle through test files

  const headers = { Authorization: `Bearer ${data.token}` };

  // Step 1: Upload a source code file
  const uploadData = {
    file: http.file(largeFileBin, fileToUpload.name),
    projectName: `project-${__VU}-${__ITER}`,
  };

  const uploadRes = http.post(`${__ENV.API_HOST}/analyze`, uploadData, { headers });
  check(uploadRes, { 'upload successful': (r) => r.status === 202 });
  const analysisId = uploadRes.json('analysisId');

  sleep(2); // Think time

  // Step 2: Poll for analysis completion
  let status = '';
  let pollAttempts = 0;
  while (status!== 'completed' && pollAttempts < 10) {
    const statusRes = http.get(`${__ENV.API_HOST}/analyze/status/${analysisId}`, { headers });
    if (statusRes.status === 200) {
      status = statusRes.json('status');
    }
    pollAttempts++;
    sleep(1);
  }
  check(status, { 'analysis completed': (s) => s === 'completed' });

  // Step 3: Request diagram rendering
  const renderRes = http.get(`${__ENV.RENDER_HOST}/render/${analysisId}`, { headers });
  check(renderRes, { 'rendering successful': (r) => r.status === 200 });

  sleep(3); // Think time
}


TestRun Manifest with initContainer:

YAML


# k6-testrun-large-files.yaml
apiVersion: k6.io/v1alpha1
kind: TestRun
metadata:
  name: uveddi-large-file-test
spec:
  parallelism: 10
  script:
    localFile: /test-data/test.js # Script will be available in the shared volume
  runner:
    # Define the shared volume
    volumes:
      - name: test-data-volume
        emptyDir: {}
    # Mount the volume to the main k6 container
    volumeMounts:
      - name: test-data-volume
        mountPath: /test-data
    # Define the initContainer to download data
    initContainers:
      - name: data-downloader
        image: alpine/git # Using git to clone a repo with test data
        command:
          - "sh"
          - "-c"
          - "git clone https://github.com/my-org/uveddi-test-data.git /test-data"
        # Mount the volume to the initContainer
        volumeMounts:
          - name: test-data-volume
            mountPath: /test-data


This comprehensive approach solves the large file problem robustly and provides a complete, stateful, and realistic test scenario for the Uveddi application.

Part 4: Systematic Bottleneck Identification and Analysis

Identifying performance bottlenecks in a distributed microservices architecture requires a systematic approach that correlates data from across the stack. A performance issue is rarely isolated to a single component; its symptoms may appear in one place (e.g., client-side latency) while its root cause lies elsewhere (e.g., database contention or a blocked async runtime). This section outlines the methodologies for analyzing test results and pinpointing performance limitations in the Uveddi system.

4.1 The Uveddi System Under Load: A Multi-Layered Monitoring Approach

The core strategy for bottleneck analysis is the creation of a unified Grafana dashboard that visualizes time-series data from all critical system components simultaneously. This allows an engineer to observe how a spike in load generator traffic impacts application services and underlying infrastructure in real-time.
Correlated Visualization: The Grafana dashboard will be designed with rows corresponding to different layers of the system:
Load Generator (k6): Key client-side metrics like requests per second (http_reqs), P95/P99 response times (http_req_duration), and error rate (http_req_failed). This is the primary view of user-perceived performance.
Rust Backend (Prometheus): Application-specific metrics (e.g., uveddi_analysis_requests_total, uveddi_analysis_duration_seconds_bucket) and crucial tokio runtime metrics (e.g., tokio_workers_busy_duration_seconds, tokio_local_queue_depth).33
Node.js Rendering Service (Prometheus): Core Node.js health metrics (nodejs_eventloop_lag_seconds, nodejs_heap_space_size_used_bytes) and custom metrics for rendering performance.35
Infrastructure (Kubernetes/Prometheus): CPU and memory utilization of the application pods, and I/O wait times on the nodes.
By aligning these graphs on a common time axis, it becomes possible to identify causal relationships. For example, a sharp increase in http_req_duration in the k6 panel might correlate directly with a spike in nodejs_eventloop_lag_seconds in the Node.js panel, immediately pointing to a blocking operation in the rendering service as the root cause.

Table 3: Key Performance Indicators (KPIs) and Monitoring Sources


KPI / Metric
System Component
Collection Tool
Metric Name (Example)
Purpose
P95/P99 Latency
End-User Experience
k6
http_req_duration
Measures user-perceived response time. Primary success criterion.
Throughput
End-User Experience
k6
http_reqs
Measures the number of requests per second the system can handle.
Error Rate
End-User Experience
k6
http_req_failed
Tracks the percentage of failed requests; indicates reliability issues.
Tokio Task Poll Count
Rust Backend
Prometheus (tokio-metrics)
tokio_tasks_polled_count
High poll counts for a task can indicate inefficient await patterns.
Tokio Worker Busy Time
Rust Backend
Prometheus (tokio-metrics)
tokio_workers_busy_duration_seconds
High busy time indicates CPU-bound work; should be low for I/O-bound apps.
Tokio Queue Depth
Rust Backend
Prometheus (tokio-metrics)
tokio_local_queue_depth
A consistently growing queue indicates the runtime cannot process tasks fast enough.
Event Loop Lag
Node.js Service
Prometheus (prom-client)
nodejs_eventloop_lag_seconds
The most critical Node.js metric; high lag indicates synchronous, blocking code is stalling the application.35
GC Duration
Node.js Service
Prometheus (prom-client)
nodejs_gc_duration_seconds
Frequent or long garbage collection pauses can introduce latency.
Pod CPU/Memory Usage
Infrastructure
Prometheus (cAdvisor)
container_cpu_usage_seconds_total
Tracks resource consumption; essential for right-sizing and identifying leaks.
Disk I/O Wait
Infrastructure
Prometheus (node-exporter)
node_disk_io_time_seconds_total
High I/O wait on the Rust pod's node can indicate a database bottleneck (e.g., SQLite).


4.2 Advanced Rust Performance Profiling with perf and flamegraph

When Grafana metrics indicate that the Rust backend is CPU-bound (i.e., high tokio_workers_busy_duration_seconds and high pod CPU utilization), a deeper analysis is required to identify the specific code paths responsible. The standard toolkit for this on Linux is perf and flamegraph.
Methodology for Profiling Under Load:
Enable Debug Symbols: Modify the project's Cargo.toml to include debug information in release builds. This is essential for mapping performance samples back to source code lines.
Ini, TOML
[profile.release]
debug = true # Or "line-tables-only" for smaller binaries


Initiate Load Test: Start the high-concurrency test using the Kubernetes framework to put the application under realistic stress.
Access the Running Pod: Use kubectl exec to get a shell inside the running Rust application pod.
Bash
kubectl exec -it <uveddi-rust-pod-name> -- /bin/bash


Record Performance Data: Use perf to record performance samples. The -p flag targets the process ID of the Rust application, --call-graph dwarf ensures accurate stack traces, and -F 99 samples at 99 Hz.
Bash
# Inside the pod
perf record -p $(pidof uveddi-backend) --call-graph dwarf -F 99 -o /tmp/perf.data -- sleep 60


Extract and Visualize: Copy the perf.data file from the pod to a local machine and use the flamegraph cargo subcommand to generate an interactive SVG flame graph.40
Bash
# On local machine
kubectl cp <uveddi-rust-pod-name>:/tmp/perf.data./perf.data
flamegraph -o flamegraph.svg --perf./perf.data


The resulting flame graph provides a visual representation of the application's call stack, where the width of each function block is proportional to the CPU time it consumed. This allows for immediate identification of "hot" functions that are prime candidates for optimization.64

4.3 Analyzing and Mitigating the SQLite Concurrency Bottleneck

The current architecture's use of SQLite presents the most significant and predictable bottleneck for the Uveddi project under high concurrency.
Problem Analysis: SQLite is a file-based, embedded database. Its concurrency model is simple and robust for many use cases but has a critical limitation for server-side applications: while it allows an unlimited number of concurrent read transactions, it enforces a single, exclusive writer at any given moment.66 When multiple threads or processes attempt to write to the database simultaneously, they are serialized. One writer acquires a database-level lock, and all other potential writers must wait. Under a load of 1000+ concurrent users, many of whom will be initiating write operations (e.g., saving analysis results), this will lead to severe lock contention. The result will be a cascade of
sqlite3.OperationalError: database is locked errors and dramatically increased request latency, effectively creating a hard ceiling on the application's throughput. While configurations like Write-Ahead Logging (WAL) mode can improve read/write concurrency, they do not solve the fundamental single-writer limitation.66
Architectural Solution: Implement a Redis Caching Layer: The only viable long-term solution is to architecturally shield the SQLite database from concurrent writes and repetitive reads. A Redis caching layer, implemented within the Rust backend using the cache-aside pattern, is the recommended solution.68
Implementation Strategy (Cache-Aside Pattern):
Integrate Redis: Add the redis-rs crate to the Rust project's dependencies to provide a client for connecting to a Redis instance.70
Modify Read Path: For any operation that fetches data (e.g., getting analysis results), the application logic must be updated:
First, attempt to fetch the data from Redis using a deterministic key (e.g., analysis:result:<analysis_id>).
If the data exists in Redis (a cache hit), return it immediately to the client.
If the data does not exist (a cache miss), query the SQLite database to retrieve the data.
Before returning the data to the client, store it in Redis with a reasonable Time-To-Live (TTL), such as 5-10 minutes. Subsequent requests for the same data will now result in a cache hit.69
Modify Write Path: For any operation that creates or modifies data (e.g., saving new analysis results):
Write the data to the primary data store (SQLite).
Immediately after the successful write, invalidate the corresponding entry in the Redis cache by deleting the key (e.g., DEL analysis:result:<analysis_id>). This ensures that the next read request will miss the cache, fetch the fresh data from SQLite, and repopulate the cache.
This pattern dramatically reduces the load on SQLite. The vast majority of read requests will be served directly from Redis's in-memory store at sub-millisecond speeds, and the write operations to SQLite, while still serialized, will be far less frequent. For future scalability, if write contention on SQLite remains an issue even with caching, the team should consider migrating to a true client/server database like PostgreSQL, which is designed for high concurrent write throughput.71

4.4 Real-time Analysis with tokio-console

For a high-performance tokio-based application, understanding the behavior of the async runtime itself is as important as profiling CPU usage. tokio-console is the essential tool for this task.
Setup: Instrumenting the application requires enabling Tokio's tracing feature and adding the console-subscriber crate. The application must be compiled and run with the RUSTFLAGS="--cfg tokio_unstable" flag.37
Usage and Insights: During a load test, an engineer can run $ tokio-console and connect it to the running Uveddi backend. The console provides a live, continuously updated view of:
All Active Tasks: A list of every tokio task, its name, and its current state.
Poll Durations: How long each task's poll method takes to execute. A task with a very long poll time is likely performing blocking or CPU-intensive work and should be optimized or moved to a blocking thread pool.
Scheduled Time: The amount of time a task spends in the run queue, ready to execute but waiting for a worker thread to become available. High scheduled times under load are a primary indicator of task starvation or an overloaded scheduler.39 This is a subtle but critical performance issue in async systems that
tokio-console is uniquely designed to expose.
By observing these metrics in real-time under load, the team can diagnose complex async-specific issues that would be nearly impossible to find with traditional profiling tools alone.

Part 5: Automation, Integration, and Continuous Performance Validation

The final and most crucial step in establishing a mature performance testing capability is to embed it into the daily development workflow. The goal is to transform performance testing from an occasional, manual exercise into an automated, continuous process that proactively prevents performance regressions. This is achieved through deep integration with the CI/CD pipeline and the implementation of automated quality gates.

5.1 Integrating Load Tests into the CI/CD Pipeline (GitHub Actions)

The declarative nature of the k6-operator makes CI/CD integration remarkably clean and straightforward. A GitHub Actions workflow can be created to automatically trigger a high-concurrency load test on every pull request or push to the main branch.
A production-ready GitHub Actions workflow file is provided below. This workflow automates the entire process:

YAML


#.github/workflows/k6-load-test.yml
name: Uveddi High-Concurrency Load Test

on:
  # Trigger on pull requests targeting the main branch
  pull_request:
    branches: [ main ]
  # Allow manual triggering from the Actions tab
  workflow_dispatch:

jobs:
  k6-test:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Setup kubectl
        uses: azure/setup-kubectl@v3
        with:
          version: 'v1.28.2'

      - name: Authenticate to Google Cloud
        uses: 'google-github-actions/auth@v2'
        with:
          credentials_json: '${{ secrets.GCP_SA_KEY }}'

      - name: Configure kubectl for GKE
        run: |
          gcloud container clusters get-credentials <your-gke-cluster-name> --zone <your-gke-zone> --project <your-gcp-project-id>

      - name: Deploy Test Script as ConfigMap
        run: |
          kubectl create configmap k6-test-script --from-file=./tests/performance/test.js --dry-run=client -o yaml | kubectl apply -f -

      - name: Run k6 Distributed Test
        uses: grafana/k6-action@v0.3.1
        with:
          # Path to the TestRun manifest file in the repository
          filename:./tests/performance/k6-testrun.yaml

      - name: Generate and Upload HTML Report
        if: always() # Run this step even if the test fails
        run: |
          # (Assuming the test is configured to output results to a file)
          # This step would use a tool like k6-reporter to generate an HTML report
          # from the k6 JSON output.
          # Example: k6-reporter -i k6-results.json -o report.html
          echo "HTML report generation step"

      - name: Upload Report Artifact
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: k6-load-test-report
          path: report.html # Path to the generated report


This workflow uses the official grafana/k6-action, which simplifies the process by handling the application of the TestRun manifest and waiting for its completion.16

5.2 Establishing Performance Quality Gates

A quality gate is an automated checkpoint in the CI/CD pipeline that enforces specific quality criteria. If the criteria are not met, the pipeline fails, preventing the problematic code from being merged or deployed.73 For performance testing, this is the mechanism that prevents regressions.
k6 provides a first-class feature for implementing performance quality gates: Thresholds. Thresholds are pass/fail criteria defined directly within the test script's options object.17
The power of this feature lies in its direct integration with CI/CD systems. When a k6 test runs, it evaluates the thresholds at the end of the execution. If any threshold condition is not met, k6 will exit with a non-zero status code. This non-zero exit code is the universal signal for failure in CI/CD environments. The GitHub Actions runner will interpret this as a failed step, automatically halting the workflow and marking the pull request check as failed.75
This creates a powerful, automated feedback loop:
A developer submits a pull request with a code change.
The GitHub Action triggers, running the high-concurrency k6 test against a staging environment.
The k6 script evaluates the performance against the predefined thresholds.
If the change introduced a performance regression (e.g., P95 latency increased beyond the threshold), the k6 process exits with an error code.
The CI/CD pipeline fails, blocking the merge and immediately notifying the developer that their change negatively impacted performance.
This "shift-left" approach catches performance issues early in the development cycle, when they are cheapest and easiest to fix, rather than late in the release cycle or, worse, in production.60
Code Example (k6 script with thresholds):
JavaScript
// In test.js
export const options = {
  //... other options like scenarios and stages
  thresholds: {
    // 99% of requests must finish within 150ms
    'http_req_duration': ['p(99)<150'],

    // P95 latency for the analysis scenario must be below 100ms
    'http_req_duration{scenario:full_analysis_workflow}': ['p(95)<100'],

    // Error rate must be less than 1%
    'http_req_failed': ['rate<0.01'],

    // A custom check for response correctness must pass for 99.9% of iterations
    'checks{my_custom_check:true}': ['rate>0.999'],
  },
};



5.3 Automated Reporting and Alerting

While quality gates provide an automated pass/fail signal, detailed reports are essential for developers to understand and debug performance issues.
CI/CD Reporting: The GitHub Actions workflow should be configured to provide comprehensive feedback:
Live Logs: The output of the k6-action will stream the k6 console summary directly into the action's logs, providing immediate, high-level results.
Artifacts: For detailed analysis, the k6 test should be configured to output its results to a JSON file (--out json=results.json). A subsequent step in the workflow can then use a tool like k6-reporter to convert this JSON file into a user-friendly HTML report.76 This report is then uploaded as a build artifact, allowing developers to download and inspect detailed graphs and metrics for failed runs.
Pull Request Comments: More advanced integrations can be configured to post a summary of the test results directly as a comment on the pull request, providing immediate visibility without needing to dig into the action logs.
Historical Trend Analysis with Grafana: The Grafana dashboard, configured in Part 4, serves as the long-term system of record for performance. By streaming k6 results to a time-series database like Prometheus or InfluxDB, the dashboard can visualize performance trends over time.22 This allows the team to answer critical questions like: "How has our P99 latency changed over the last three releases?" or "Did the recent database migration improve or degrade performance?"
Proactive Alerting: The same Prometheus metrics that power the Grafana dashboard can be used to configure alerts via Alertmanager. Rules can be set to trigger notifications (e.g., to Slack or PagerDuty) if key performance indicators in a staging or production environment degrade beyond acceptable levels. This provides a safety net that complements the CI-based quality gates, ensuring that performance issues that might slip through testing are still detected and addressed quickly.
By implementing this three-tiered approach—automated CI gates, detailed reporting artifacts, and long-term dashboarding—the Uveddi project can build a culture of performance awareness and ensure the system remains fast, reliable, and scalable as it evolves.
Works cited
The Goose Book: What Is Goose?, accessed July 19, 2025, https://book.goose.rs/
goose - Rust - Docs.rs, accessed July 19, 2025, https://docs.rs/goose
tag1consulting/goose: Load testing framework, inspired by Locust - GitHub, accessed July 19, 2025, https://github.com/tag1consulting/goose
fcsonline/drill: Drill is an HTTP load testing application written in Rust - GitHub, accessed July 19, 2025, https://github.com/fcsonline/drill
Open source load testing tool review: 2020 | Grafana Labs, accessed July 19, 2025, https://grafana.com/blog/2020/03/03/open-source-load-testing-tool-review/
Goose - The most scalable load testing framework | Tag1 Consulting, accessed July 19, 2025, https://www.tag1consulting.com/goose-podcasts-blogs-presentations-more
Apache JMeter reviews 2025 - PeerSpot, accessed July 19, 2025, https://www.peerspot.com/products/apache-jmeter-reviews
JMeter Reviews & Ratings 2025 - TrustRadius, accessed July 19, 2025, https://www.trustradius.com/products/apache-jmeter/reviews
A Journey Of Selecting Tool To build 'Load Test As A Service' | by Nayan Gaur - Medium, accessed July 19, 2025, https://medium.com/capillary-tech/a-journey-of-selecting-tool-to-build-load-test-as-a-service-322178a00005
Gatling Reviews 2025: Details, Pricing, & Features - G2, accessed July 19, 2025, https://www.g2.com/products/gatling/reviews
AWS Marketplace: Gatling Enterprise - Distributed Load Testing Reviews - Amazon.com, accessed July 19, 2025, https://aws.amazon.com/marketplace/reviews/reviews-list/B07PFQ5B72
Top 25 Load Testing Software in 2025 | BrowserStack, accessed July 19, 2025, https://www.browserstack.com/guide/load-testing-software
grafana/k6: A modern load testing tool, using Go and JavaScript - GitHub, accessed July 19, 2025, https://github.com/grafana/k6
Running distributed tests | Grafana k6 documentation, accessed July 19, 2025, https://grafana.com/docs/k6/latest/testing-guides/running-distributed-tests/
An Introduction to Distributed Load Testing with k6 on Kubernetes | by PI | Neural Engineer, accessed July 19, 2025, https://medium.com/neural-engineer/an-introduction-to-distributed-load-testing-with-k6-on-kubernetes-ba7fc87299c5
Integrating k6 Load Tests with GitHub Actions, GitLab CI and Azure DevOps - Learn with RV, accessed July 19, 2025, https://razvanvancea.ro/blog/2024/05/17/integrating-k6-load-tests-with-github-actions-gitlab-ci-and-azure-devops/
Performance testing with Grafana k6 and GitHub Actions, accessed July 19, 2025, https://grafana.com/blog/2024/07/15/performance-testing-with-grafana-k6-and-github-actions/
Loadrunner vs JMeter: Which is the Better Performance Testing Tool - BrowserStack, accessed July 19, 2025, https://www.browserstack.com/guide/loadrunner-vs-jmeter
Distributed load testing using Google Kubernetes Engine | Cloud Architecture Center, accessed July 19, 2025, https://cloud.google.com/architecture/distributed-load-testing-using-gke
Load Testing in Kubernetes: Tools and Best Practices - Testkube, accessed July 19, 2025, https://testkube.io/learn/load-testing-in-kubernetes-tools-and-best-practices
Getting Started with Goose - QAInsights, accessed July 19, 2025, https://qainsights.com/getting-started-with-goose/
Grafana dashboards | Grafana k6 documentation, accessed July 19, 2025, https://grafana.com/docs/k6/latest/results-output/grafana-dashboards/
Running A Load Test - The Goose Book, accessed July 19, 2025, https://book.goose.rs/getting-started/running.html
Kubernetes vs. Docker Swarm: Pros/Cons and 6 Key Differences - Lumigo, accessed July 19, 2025, https://lumigo.io/kubernetes-monitoring/kubernetes-vs-docker-swarm-pros-cons-and-6-key-differences/
Docker Swarm Vs. Kubernetes: Which One To Choose In 2025 ? - Thinksys Inc., accessed July 19, 2025, https://thinksys.com/devops/docker-swarm-vs-kubernetes-comparison/
Kubernetes vs Docker Swarm: Complete Comparison Guide for 2025 - ExamSnap, accessed July 19, 2025, https://www.examsnap.com/certification/kubernetes-vs-docker-swarm-complete-comparison-guide-for-2025/
grafana/k6-operator: An operator for running distributed k6 tests. - GitHub, accessed July 19, 2025, https://github.com/grafana/k6-operator
Setting Up Prometheus to Monitor a Node.js Application: A Step-by-Step Guide - Medium, accessed July 19, 2025, https://medium.com/@hemanthkumarreddy739/setting-up-prometheus-to-monitor-a-node-js-application-a-step-by-step-guide-324c382b95c7
JMeter Load Test | Grafana Labs, accessed July 19, 2025, https://grafana.com/grafana/dashboards/1152-jmeter-load-test/
Load testing | Grafana Labs, accessed July 19, 2025, https://grafana.com/load-testing/
metrics-exporter-prometheus - crates.io: Rust Package Registry, accessed July 19, 2025, https://crates.io/crates/metrics-exporter-prometheus
PrometheusBuilder in metrics_exporter_prometheus - Rust - Docs.rs, accessed July 19, 2025, https://docs.rs/metrics-exporter-prometheus/latest/metrics_exporter_prometheus/struct.PrometheusBuilder.html
Utilities for collecting metrics from a Tokio application - GitHub, accessed July 19, 2025, https://github.com/tokio-rs/tokio-metrics
Instrument Node.js code: Prometheus custom metrics - SquaredUp, accessed July 19, 2025, https://squaredup.com/blog/instrument-node-with-prometheus/
Monitoring Node.js: Key Metrics You Should Track | Last9, accessed July 19, 2025, https://last9.io/blog/node-js-key-metrics/
Node.js exporter - Prometheus OSS - Grafana, accessed July 19, 2025, https://grafana.com/oss/prometheus/exporters/nodejs-exporter/
README.md - tokio-console subscriber - GitHub, accessed July 19, 2025, https://github.com/tokio-rs/console/blob/main/console-subscriber/README.md
Next steps with Tracing | Tokio - An asynchronous Rust runtime, accessed July 19, 2025, https://tokio.rs/tokio/topics/tracing-next-steps
Thoughts about profiling Rust/Tokio applications - The Rust Programming Language Forum, accessed July 19, 2025, https://users.rust-lang.org/t/thoughts-about-profiling-rust-tokio-applications/120069
Profiling - The Rust Performance Book, accessed July 19, 2025, https://nnethercote.github.io/perf-book/profiling.html
Kubernetes Cost: EKS vs AKS vs GKE - Sedai, accessed July 19, 2025, https://www.sedai.io/blog/kubernetes-cost-eks-vs-aks-vs-gke
Managed Kubernetes Pricing Comparison: EKS vs. AKS vs. GKE | by Jay Chapel | Medium, accessed July 19, 2025, https://jaychapel.medium.com/managed-kubernetes-pricing-comparison-eks-vs-aks-vs-gke-dbf3f2c6290c
Kubernetes Cost Optimization: Best Practices, Tools, and Automation - Cloudchipr, accessed July 19, 2025, https://cloudchipr.com/blog/kubernetes-cost-optimization
Cloud Pricing Comparison: AWS vs. Azure vs. Google Cloud Platform in 2025 - Cast AI, accessed July 19, 2025, https://cast.ai/blog/cloud-pricing-comparison/
Kubernetes Cost Optimization: Strategies & Best Practices - CloudBolt, accessed July 19, 2025, https://www.cloudbolt.io/cloud-cost-management/kubernetes-cost-optimization/
The True Cost of Running Kubernetes in the Cloud - Convox, accessed July 19, 2025, https://www.convox.com/blog/cost-of-running-k8s
Is it true that GCP is cheaper than Azure/AWS? : r/googlecloud - Reddit, accessed July 19, 2025, https://www.reddit.com/r/googlecloud/comments/1d0yq13/is_it_true_that_gcp_is_cheaper_than_azureaws/
aws-solutions/distributed-load-testing-on-aws - GitHub, accessed July 19, 2025, https://github.com/aws-solutions/distributed-load-testing-on-aws
Distributed Load Testing on AWS, accessed July 19, 2025, https://aws.amazon.com/solutions/implementations/distributed-load-testing-on-aws/
What is Azure Load Testing? | Microsoft Learn, accessed July 19, 2025, https://learn.microsoft.com/en-us/azure/load-testing/overview-what-is-azure-load-testing
Azure Load Testing - Tools and Services, accessed July 19, 2025, https://azure.microsoft.com/en-us/products/load-testing
Load testing best practices | Cloud Run Documentation, accessed July 19, 2025, https://cloud.google.com/run/docs/about-load-testing
Install k6 Operator | Grafana k6 documentation, accessed July 19, 2025, https://grafana.com/docs/k6/latest/set-up/set-up-distributed-k6/install-k6-operator/
Kubernetes Cost Optimization: Strategies for Maximum Efficiency & Savings, accessed July 19, 2025, https://www.getambassador.io/blog/kubernetes-cost-optimization-strategies
The Definitive Guide to Kubernetes Cost Optimization - PerfectScale, accessed July 19, 2025, https://www.perfectscale.io/blog/kubernetes-cost-optimization
gatling.io-doc/content/tutorials/writing-realistic-tests/index.md at main · gatling/gatling.io-doc · GitHub, accessed July 19, 2025, https://github.com/gatling/gatling.io-doc/blob/main/content/tutorials/writing-realistic-tests/index.md
Write realistic advanced Gatling tests to simulate real world scenarios for your application using http endpoints groups and injection profiles - Gatling documentation, accessed July 19, 2025, https://docs.gatling.io/guides/optimize-scripts/writing-realistic-tests/
Scenarios | Grafana k6 documentation, accessed July 19, 2025, https://grafana.com/docs/k6/latest/using-k6/scenarios/
Mastering VUs and Test Duration with K6 - Parottasalna, accessed July 19, 2025, https://parottasalna.com/2025/02/16/mastering-vus-and-test-duration-with-k6/
Shift-Left Performance Testing: Integrating k6 in CI/CD Pipelines | by Sumit Soman - Medium, accessed July 19, 2025, https://medium.com/@sumit.somanchd/shift-left-performance-testing-integrating-k6-in-ci-cd-pipelines-e56355abe861
k6-learn/Modules/III-k6-Intermediate/03-Workload-modeling.md at main - GitHub, accessed July 19, 2025, https://github.com/grafana/k6-learn/blob/main/Modules/III-k6-Intermediate/03-Workload-modeling.md
k6-learn/Modules/III-k6-Intermediate/04-Adding-test-data.md at main - GitHub, accessed July 19, 2025, https://github.com/grafana/k6-learn/blob/main/Modules/III-k6-Intermediate/04-Adding-test-data.md
Scaling Distributed Load Testing with k6 on Kubernetes: Handling Large Data Files | by PI | Neural Engineer | Medium, accessed July 19, 2025, https://medium.com/neural-engineer/scaling-k6-load-testing-on-kubernetes-handling-large-data-files-dff4758b1ae5
Flame Graph Performance Truth Analysis - DEV Community, accessed July 19, 2025, https://dev.to/member_a07758c4/flame-graph-performance-truth-analysis-4ce
How to interpret a flamegraph? : r/rust - Reddit, accessed July 19, 2025, https://www.reddit.com/r/rust/comments/wf9mbn/how_to_interpret_a_flamegraph/
Abusing SQLite to Handle Concurrency - SkyPilot Blog, accessed July 19, 2025, https://blog.skypilot.co/abusing-sqlite-to-handle-concurrency/
Appropriate Uses For SQLite, accessed July 19, 2025, https://www.sqlite.org/whentouse.html
SQLite vs Redis - Key Differences - Airbyte, accessed July 19, 2025, https://airbyte.com/data-engineering-resources/sqlite-vs-redis
How to use Redis for Query Caching, accessed July 19, 2025, https://redis.io/learn/howtos/solutions/microservices/caching
redis - Rust - Docs.rs, accessed July 19, 2025, https://docs.rs/redis/latest/redis/
Top SQLite Competitors & Alternatives 2025 | Gartner Peer Insights, accessed July 19, 2025, https://www.gartner.com/reviews/market/data-and-analytics-others/vendor/sqlite/product/sqlite/alternatives
Top 10 SQLite Alternatives & Competitors in 2025 - G2, accessed July 19, 2025, https://www.g2.com/products/sqlite/competitors/alternatives
Mastering Quality Gates in SQA - Number Analytics, accessed July 19, 2025, https://www.numberanalytics.com/blog/mastering-quality-gates-in-sqa
Quality Gates And Their Importance - StaleElement, accessed July 19, 2025, https://staleelement.com/2025/03/19/quality-gates-and-their-importance/
How to Automate Load Testing with K6 for CI/CD Pipelines - Frugal Testing, accessed July 19, 2025, https://www.frugaltesting.com/blog/how-to-automate-load-testing-with-k6-for-ci-cd-pipelines
Run Performance Test with K6 in Gitlab CI | by Irwan Syarifudin - Medium, accessed July 19, 2025, https://medium.com/@irwansyarifudin16/run-performance-test-with-k6-in-gitlab-ci-548afb8d4b4f
k6 Load Testing Results | Grafana Labs, accessed July 19, 2025, https://grafana.com/grafana/dashboards/4411-k6-load-testing-results/
