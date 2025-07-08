
Image Rendering Service Architecture: A Comparative Analysis and Implementation Roadmap


Executive Summary & Architecture Recommendation


Overview of the Challenge

The primary business objective is to develop a robust, scalable, and maintainable service for rendering text-based Mermaid diagrams into high-quality image formats. This service is a critical component for documentation platforms, where clear, up-to-date diagrams are essential for user comprehension and developer productivity. The service must deliver fast rendering times, handle concurrent user requests efficiently, and provide outputs suitable for modern web environments.

Core Findings

The analysis of potential rendering architectures reveals a fundamental trade-off between implementation control and operational simplicity. Self-hosted solutions using direct browser automation offer maximum performance and control but require more initial engineering effort. Conversely, integrating with managed third-party services simplifies operations significantly but introduces external dependencies, potential costs, and data privacy considerations. A critical finding is that the popular Mermaid CLI tool, while simple for local use, is architecturally unsuited for a high-concurrency web service due to its high per-request overhead.1

Architecture Recommendation

This report recommends a self-hosted architecture utilizing Playwright for direct browser automation, fronted by a Node.js web service that manages a persistent pool of browser workers. This approach provides the optimal balance of performance, scalability, and control necessary for a high-throughput, production-grade rendering environment.

Justification Summary

The recommendation for a direct Playwright-based service is founded on its superior performance characteristics in a web service context. By launching a browser instance once and reusing it for multiple requests, this architecture amortizes the significant startup cost associated with browser processes, leading to drastically lower per-request latency compared to a CLI-wrapping approach.1 Playwright is favored over Puppeteer for its modern, ergonomic API and robust cross-browser capabilities, providing future flexibility.3 This self-hosted model ensures complete control over dependencies, security posture, and the operational environment, which is paramount for a core infrastructure service.

Expected Business Outcomes

Adopting the recommended architecture will result in a highly performant diagram rendering service that enhances the end-user experience through fast page loads. It establishes a scalable foundation capable of handling significant growth in usage without architectural redesign. Furthermore, by building this capability in-house, the organization retains full ownership of the core infrastructure, avoids vendor lock-in, and can customize and extend the service to meet future business requirements.

Analysis of Rendering Service Architectures

An effective rendering service must balance performance, scalability, and implementation complexity. This section deconstructs four primary architectural patterns, analyzing their components, data flow, dependencies, and inherent strengths and weaknesses.

The CLI-Wrapper Pattern (@mermaid-js/mermaid-cli)


Architectural Blueprint

This pattern involves creating a web service, typically using Node.js with a framework like Express, that acts as a wrapper around the @mermaid-js/mermaid-cli command-line tool.5 The data flow is straightforward: the service receives an HTTP request containing Mermaid syntax, uses the Node.js
child_process module to spawn a new process executing the mmdc command, and pipes the resulting image output (SVG or PNG) back to the client.6

The Hidden Dependency: Puppeteer

The most significant architectural characteristic of mermaid-cli is its hard dependency on Puppeteer to perform the actual rendering.2 Mermaid.js requires a browser's layout engine to calculate the size and position of diagram elements correctly.8 Consequently, every invocation of the
mmdc command launches a full, headless Chromium browser instance in the background. This non-obvious implementation detail is the primary determinant of this pattern's performance profile.
This dependency also introduces considerable environmental complexity. A full Chromium installation is required, which can be challenging in minimal containerized environments and necessitates specific system libraries like libgbm-dev.11 While pre-built Docker images for
mermaid-cli exist to abstract this setup, they inherently bundle a large browser instance, increasing the image size and resource footprint.5

Architectural Assessment

The primary appeal of the CLI-wrapper is its perceived simplicity. An engineer can quickly implement a service by wrapping a command they are familiar with from local use. However, this simplicity is deceptive in a high-concurrency web service context. The mermaid-cli tool is optimized for discrete, one-off conversions or as a step in a static site generation build process, not for low-latency API calls.2
The performance model is fundamentally flawed for this use case. Benchmarks show that launching a new Puppeteer browser instance (puppeteer.launch()) incurs a significant startup cost, often between 100-300ms.1 Because the CLI-wrapper spawns a new process—and therefore a new browser instance—for every single request, this substantial latency is added to every API call. This "process-per-request" model leads to poor resource utilization and makes the architecture unsuitable for any application requiring responsive diagram rendering. It is only viable for very low-traffic systems or asynchronous, non-time-sensitive batch processing jobs.

The Headless Browser Pattern (Puppeteer/Playwright)


Architectural Blueprint

This architecture involves a service that directly integrates a browser automation library like Puppeteer or Playwright to manage rendering. The key distinction from the CLI-wrapper is that the service launches a single, persistent headless browser instance upon startup. For each incoming rendering request, it then creates a lightweight, isolated BrowserContext or Page within that persistent instance.

Performance Advantage: Amortizing Startup Cost

The core architectural benefit of this pattern is the amortization of the expensive browser launch. By maintaining a long-lived browser process and using a lightweight connection method for each task, the high startup cost is paid only once when the service initializes. Subsequent requests avoid this penalty, reducing per-request overhead by orders of magnitude (from over 100ms to under 1ms, excluding the actual rendering time).1 This transforms the performance model from the inefficient "process-per-request" to a highly efficient "task-per-worker" model, which is essential for a scalable service.

Sub-Analysis: Puppeteer vs. Playwright

While both tools can implement this pattern, there are important distinctions:
Origins and Philosophy: Puppeteer, developed by Google, is tightly coupled with the Chrome DevTools Protocol (CDP), offering deep, fine-grained control over Chromium-based browsers.3 Playwright, developed at Microsoft by engineers from the original Puppeteer team, prioritizes robust cross-browser compatibility (Chromium, Firefox, WebKit) and a more comprehensive, test-oriented feature set out of the box.3
Performance: While Puppeteer historically had a faster startup time, recent changes to Chromium's new headless mode have been shown to be significantly slower, potentially negating this advantage.14 Benchmarks of real-world scenarios show that Playwright's performance is often equivalent or slightly better than Puppeteer's.17
API and Features: Playwright is generally considered to have a more ergonomic API, with built-in functionalities like auto-waits that often require more manual configuration in Puppeteer.4 The existence of projects like
remark-mermaidjs, which chose Playwright for rendering, is a testament to its capabilities in this domain.19
Stability: Playwright achieves its cross-browser support by using patched versions of Firefox and WebKit, which can lead to minor divergences from public releases.3 Puppeteer works directly with browser vendors on the CDP, suggesting better long-term alignment.20 However, for a controlled server-side rendering environment, the risk associated with Playwright's approach is minimal and manageable.

Architectural Assessment

This pattern represents a mature, high-performance architecture for a dedicated rendering service. It grants maximum control over the entire rendering pipeline, including resource allocation, error handling (e.g., restarting crashed browser instances), and performance tuning. This control comes at the cost of higher initial implementation complexity, as the service is responsible for managing the browser lifecycle, process pooling, and inter-process communication. However, for building a scalable and robust system, these are necessary complexities to manage. Given its modern API and strong performance profile, Playwright emerges as a slightly superior choice for this architecture.

The WebAssembly (WASM) Pattern (mermaid-wasm)


Architectural Goal

The ideal WebAssembly (WASM) architecture for Mermaid rendering would involve a lightweight, server-side WASM runtime (such as WasmEdge 21) executing a compiled version of the
mermaid-js library. This would theoretically eliminate the dependency on a heavyweight headless browser, leading to a drastically smaller memory footprint, near-instantaneous startup times, and improved security through sandboxing.

The Core Obstacle: The Layout Engine Dependency

This architectural goal is currently unachievable. The core of Mermaid.js is not just a diagram-to-SVG parser; it is fundamentally a browser-native library. Its algorithm critically depends on the browser's Document Object Model (DOM) and CSS layout engine. It first renders SVG elements and then uses browser-specific APIs like SVGTextElement.getBBox() to measure the dimensions of those rendered elements (especially text) to correctly position subsequent nodes and edges.8
These layout engine APIs are not available in a standard server-side environment like Node.js or a pure WASM runtime. Attempts to simulate a browser environment on the server using tools like JSDOM have failed precisely because they lack a true layout engine and cannot implement functions like getBBox().8

Architectural Assessment

The research clearly indicates that a "pure" WASM-based rendering solution for Mermaid is not a viable option for immediate implementation. The Mermaid community is actively exploring true Server-Side Rendering (SSR) to improve performance and prevent client-side layout shifts, but this remains a complex, long-term goal.9 While WASM is revolutionizing server-side computing in many areas 21, it does not yet solve this specific problem.
Therefore, this pattern should be discounted as a practical choice for the current project. It serves as a valuable point of reference that explains why headless browsers are a necessary dependency today and points toward a potential future evolution of rendering technology, perhaps contingent on the maturation of a WASM-based layout engine like Servo.

The External Service Integration Pattern (Kroki/Mermaid Live)


Architectural Blueprint

This pattern delegates the entire rendering lifecycle to a third-party service. The primary application makes an HTTP API call, sending the Mermaid source code and receiving a rendered image in return. This abstracts away all the complexity of the rendering process.

Option A: Kroki.io

Kroki is a powerful open-source project that provides a unified API for a vast ecosystem of diagramming libraries, including Mermaid.23 It can be consumed as a public free service or self-hosted for greater control and privacy.
The self-hosted Kroki architecture is a sophisticated, modular system. A central gateway server, written in Java and powered by Vert.x, routes requests to dedicated "companion containers" for specific diagram types.25 To support Mermaid, an organization must run both the main
yuzutech/kroki container and the yuzutech/kroki-mermaid companion container.25 This companion container, under the hood, uses a headless browser to perform the rendering. Therefore, self-hosting Kroki is akin to deploying a pre-packaged, multi-container implementation of the headless browser architecture. While documentation on specific resource requirements is sparse 27, community discussions suggest that any non-trivial self-hosted service requires adequate resources (e.g., a minimum of 4 GB of RAM for the service itself).28

Option B: Mermaid Live / Mermaid Chart

The official Mermaid ecosystem offers two potential endpoints. The public Mermaid Live Editor uses a rendering backend at https://mermaid.ink.30 However, this is an undocumented, unsupported endpoint with no service-level agreement (SLA), making it far too risky for a production application.
The viable commercial option is Mermaid Chart, the official Software-as-a-Service (SaaS) offering from the creators of Mermaid.js.31 This is a classic "buy versus build" decision. Mermaid Chart provides a managed rendering API along with value-added features like AI diagramming, cloud storage, and team collaboration tools.31 It offers the lowest possible implementation and operational overhead but comes with a subscription cost, data privacy considerations, and a hard dependency on an external provider.33

Architectural Assessment

The external service pattern offers a spectrum of choices. Self-hosting Kroki represents an excellent middle ground: it is less complex to implement from a software development perspective than building a custom headless browser service, but it still requires DevOps expertise to deploy and manage a Docker Compose or Kubernetes-based application. It is an ideal choice if the organization anticipates needing to support other diagram types in the future, as Kroki's unified API makes this trivial.23 Using a SaaS provider like Mermaid Chart is a strategic business decision based on an analysis of cost, risk tolerance, and the need for its specific proprietary features.

Comparative Analysis: Performance, Scalability, and Complexity

This section synthesizes the architectural analysis into direct comparisons, providing the data necessary to make an informed decision based on implementation effort, performance characteristics, and scalability potential.

Implementation and Maintenance Complexity

The choice of architecture has significant implications for both the initial development effort and the long-term total cost of ownership (TCO). A solution that is quick to prototype may become a liability in production if its maintenance overhead is high.
CLI-Wrapper: This approach has a low initial implementation cost, as it only requires wrapping a command-line call. However, its maintenance overhead is high and complex. Production deployments are plagued by the need to manage the fragile process of spawning and the hidden complexities of the Puppeteer dependency, often leading to environment-specific issues and dependency conflicts in containers.12
Headless Browser (Direct): This pattern demands a high initial implementation effort. It requires building a robust service with custom logic for browser process management, worker pooling, and inter-process communication.1 Once built, however, maintenance is more straightforward because the control mechanisms are explicit and owned by the development team.
Kroki (Self-Hosted): The implementation effort here is medium and shifts from software development to DevOps. The primary tasks involve setting up a multi-container environment using Docker Compose or Kubernetes and configuring the necessary environment variables.25 Maintenance consists of updating the official Kroki containers and monitoring the health of the system.
Cloud Service (SaaS): This option has a very low implementation and maintenance burden, limited to API key management and integration. The trade-off is a recurring operational expense and a complete loss of control over the underlying infrastructure.
The following table summarizes these trade-offs.
Table 1: Architecture Options & Implementation Complexity
Architecture
Key Dependencies
Pros
Cons
Implementation Complexity (Initial)
Maintenance Overhead
CLI-Wrapper
Node.js, @mermaid-js/mermaid-cli, Puppeteer/Chromium
Simple to prototype; familiar CLI interface.
High per-request latency; poor concurrency; complex environment setup; inefficient resource use.
Low
High
Headless Browser (Direct)
Node.js, Playwright/Puppeteer, Chromium
Highest performance; best scalability; full control over environment and logic.
High initial development effort; requires complex pooling and process management.
High
Medium
Kroki (Self-Hosted)
Docker/Kubernetes, Java, Node.js, multiple containers
Good performance; supports many diagram types; abstracts away rendering logic.
Significant resource footprint; DevOps complexity; "black box" architecture.
Medium
Medium
Cloud Service (SaaS)
HTTP Client
Zero operational overhead; managed and supported; value-added features.
Recurring cost; vendor lock-in; data privacy concerns; no control over performance/uptime.
Very Low
Very Low
WASM (Future)
WASM Runtime
Potentially fastest and most lightweight.
Not currently feasible due to Mermaid's dependency on a browser layout engine.
N/A
N/A


Performance and Scalability Matrix

Performance is not a single metric but a combination of latency, throughput, and resource utilization. The chosen architecture directly dictates these characteristics.
Startup Latency: The CLI-wrapper suffers from the worst latency, as it incurs the full cost of spawning a process and launching a browser on every request.1 The direct headless browser and Kroki approaches have a high one-time startup cost but near-zero per-request startup latency, as they reuse persistent processes.
Concurrency: The direct headless browser model offers the most granular control over concurrency and is therefore the most scalable. A fine-tuned pool of browser pages can efficiently serve a high volume of requests. The CLI-wrapper is the least scalable; its concurrency is limited by the number of heavyweight browser processes the host machine can run simultaneously before becoming memory- or CPU-bound.
Memory Usage: The CLI-wrapper is highly inefficient, allocating memory for a full browser instance for each concurrent request. The direct headless browser model has a high baseline memory usage for the persistent browser but adds minimal memory overhead per request. Kroki's memory footprint will be significant due to its Java-based gateway and multiple companion containers.
The architectural choice has a profound ripple effect on scalability. The 1 process : 1 request model of the CLI-wrapper scales poorly because each new server node can only handle a handful of concurrent requests before being overwhelmed. In contrast, the N workers : M requests model of the direct headless browser architecture allows each server node to handle a much larger number of concurrent requests. This makes the system far more efficient and cost-effective to scale horizontally.
Table 2: Performance & Scalability Comparison Matrix
Metric
CLI-Wrapper
Headless Browser (Direct)
Kroki (Self-Hosted)
Avg. End-to-End Time (Simple Diagram)
High (~200-500ms)
Low (~50-100ms)
Low (~50-150ms)
Avg. End-to-End Time (Complex Diagram)
Very High (500ms+)
Medium (~100-300ms)
Medium (~100-400ms)
Memory Footprint (Baseline / Per-Request)
Low / Very High
High / Low
Very High / Low
Concurrency Model
Process-per-request
Worker Pool
Internal (Worker Pool)
Startup Latency (Cold Start / Per-Request)
High / High
High / Very Low
High / Very Low
Caching Effectiveness (Result Caching)
High
High
High


Strategic Considerations for Output Formats & Caching

Beyond the core service architecture, two strategic decisions heavily influence the final product's quality, performance, and utility: the choice of output image format and the implementation of a robust caching layer.

Output Format Deep Dive (SVG vs. PNG)

The choice between SVG (Scalable Vector Graphics) and PNG (Portable Network Graphics) is not merely a technical detail but a critical product decision. While both can represent diagrams, their underlying technologies lead to vastly different capabilities. SVG is an XML-based vector format that describes images as a set of drawing instructions, whereas PNG is a pixel-based raster format.35
Scalability and Quality: SVGs are resolution-independent. They can be scaled to any size without any loss of quality, ensuring diagrams remain perfectly crisp on all displays, from small mobile screens to high-resolution monitors. This is a crucial feature for modern, responsive web design.37 PNGs, being raster-based, are fixed in resolution and will become blurry or pixelated when scaled up.37
File Size: For the geometric shapes and text that constitute Mermaid diagrams, SVGs almost always result in a significantly smaller file size compared to PNGs.36 This directly translates to faster page load times and reduced bandwidth consumption. SVG file sizes can be further reduced using optimization tools like SVGO or SVGOMG, which remove redundant code without affecting the visual output.39
Accessibility and SEO: This is a defining advantage for SVG. Because an SVG file is structured, text-based XML, its content—including diagram labels and descriptions—can be read and indexed by screen readers and search engines. This dramatically improves both accessibility for visually impaired users and Search Engine Optimization (SEO) for the documentation.36 A PNG is opaque to these systems, relying solely on a secondary
alt text attribute.
Interactivity: As a part of the DOM, SVGs can be styled with CSS and manipulated with JavaScript. This enables rich, interactive experiences such as hover effects, clickable nodes that link to other pages, or dynamic animations—all of which are impossible with a static PNG.38 Mermaid's own API supports binding events directly to the generated SVG elements.42
Ultimately, choosing PNG treats a diagram as a static, disconnected image, while choosing SVG treats it as a dynamic, accessible, and integral part of the web document itself. For a modern documentation platform, the benefits of SVG are not just desirable; they are essential features. The service should therefore default to and strongly recommend SVG output, offering PNG only as a fallback for legacy systems or formats that do not support vector graphics.
Table 3: SVG vs. PNG Trade-Offs for Documentation
Attribute
SVG (Scalable Vector Graphics)
PNG (Portable Network Graphics)
Scalability
Infinite; no quality loss.
Limited; pixelates when enlarged.
File Size (for Diagrams)
Typically much smaller.
Typically larger.
Visual Quality
Always crisp and clear.
Dependent on original resolution.
Interactivity
High (CSS/JS manipulation).
None (static image).
Accessibility & SEO
High (text is readable by machines).
Low (opaque to screen readers/crawlers).
Browser Support
Universal in modern browsers.
Universal.
Transparency
Supported.
Supported (superior for raster transparency).
Primary Use Case
Logos, icons, and diagrams for responsive web.
Detailed photographs, screenshots.


Caching Architecture

Since rendering is the most computationally expensive operation in the service, an aggressive and intelligent caching strategy is critical for achieving high performance, scalability, and cost-efficiency. The goal is to ensure that a diagram with the exact same source text and configuration is never rendered more than once.
A multi-layer caching architecture is recommended:
Cache Key Strategy: The cache key should be a cryptographic hash (e.g., SHA-256) of the canonical Mermaid source text combined with any rendering options (such as theme or background color). This deterministic approach ensures that any change to the input uniquely identifies the output, effectively creating a content-addressable storage system and simplifying cache invalidation.43
Layer 1: Hot Cache (In-Memory/Distributed): A fast, low-latency cache, such as Redis or an in-process cache like node-cache, should be used as the first line of defense. When a request arrives, the service first checks this cache using the generated key. A cache hit results in an immediate response, bypassing the rendering queue entirely.44
Layer 2: Cold Cache (CDN/Browser): On every successful response, the service must set appropriate HTTP Cache-Control headers, such as public, max-age=31536000, immutable.46 This header instructs downstream caches—including Content Delivery Networks (CDNs) and the end-user's browser—to store a copy of the rendered image for an extended period (e.g., one year). This layer offloads the vast majority of traffic for popular diagrams from the rendering service, drastically reducing infrastructure load and cost.43
This system will operate in an "on-demand" caching model. The first time a unique diagram is requested, it will trigger a rendering operation, and the result will be populated in both the L1 and L2 caches. All subsequent requests for that same diagram will be served instantly from one of these cache layers.48

Implementation Roadmap for Recommended Architecture

This section provides a phased, actionable plan for building the recommended Direct Headless Browser (Playwright) Service. This roadmap is structured into sprints to guide development from a basic functional service to a production-ready, scalable application.

Phase 1: Core Service & API Implementation (Sprint 1-2)

Task 1: Project Setup: Initialize a new Node.js project with TypeScript for type safety and maintainability.
Task 2: Web Framework and API Design: Implement a basic web server using a performant framework like Fastify or Express. Define a single RESTful endpoint, POST /render, that accepts a JSON body with the following structure: { "source": "graph TD...", "format": "svg", "theme": "dark" }. This adheres to best practices for API design.49
Task 3: Initial Rendering Logic: Install Playwright and its browser dependencies (npx playwright install --with-deps chromium).19 Implement the core rendering logic. In this initial phase, it is acceptable to launch a new browser instance on each request (
playwright.chromium.launch()) to validate the end-to-end rendering pipeline.
Task 4: Basic Error Handling: Implement try...catch blocks around the rendering logic to gracefully handle Mermaid syntax errors or other rendering failures, returning an appropriate HTTP error code (e.g., 400 Bad Request) with a descriptive message.

Phase 2: Scalability & Performance Enhancements (Sprint 3-4)

Task 1: Implement Browser Pooling: This is the most critical step for production performance. Refactor the service to manage a persistent browser pool.
On service startup, launch a single, long-lived Playwright browser instance.
Create a pool of reusable BrowserContext or Page objects. This concept is analogous to database connection pooling, where expensive resources are created once and reused.34
For each incoming request, acquire a worker (page/context) from the pool. If the pool is empty, the request should be queued until a worker becomes available.
After rendering is complete, the worker must be cleaned and released back to the pool.
Task 2: Ensure Asynchronous Flow: Verify that the entire request-to-response lifecycle is fully asynchronous and non-blocking, making efficient use of the Node.js event loop to handle high concurrency.52

Phase 3: Caching & Output Optimization (Sprint 5)

Task 1: Implement Layer 1 Cache: Integrate a Redis client into the service. Before attempting to acquire a worker from the pool, the service must first check Redis for a cached result using a SHA-256 hash of the request payload as the key. On a cache miss, the service should render the image and then store the result in Redis with a long Time-To-Live (TTL).
Task 2: Implement Layer 2 Cache Headers: Configure the API endpoint to set Cache-Control and ETag headers on all successful image responses. This will enable downstream CDN and browser caching, significantly reducing traffic to the service.46
Task 3: SVG Optimization: After generating an SVG string, pass it through an optimization library like svgo. This will minify the XML, remove unnecessary metadata, and reduce the final file size before it is sent to the client or stored in the cache.39

Phase 4: Observability, Security & Deployment (Sprint 6+)

Task 1: Logging and Metrics: Integrate a structured logger (e.g., Pino) to record request details, render times, cache status, and errors. Expose key operational metrics (e.g., render time percentiles, cache hit/miss ratio, worker pool utilization) in a Prometheus-compatible format for monitoring and alerting.
Task 2: Security Hardening:
Resource Limiting: Implement a strict timeout for all rendering operations using Playwright's built-in timeout functionality. This prevents malformed or malicious diagram definitions from causing runaway processes that consume excessive CPU or memory.
Input Validation: Sanitize and validate all user-provided input to prevent potential injection attacks, even though the rendering itself occurs within a sandboxed browser environment.
Dependency Scanning: Integrate a security scanner like Snyk or npm audit into the Continuous Integration (CI) pipeline to continuously monitor for vulnerabilities in all third-party dependencies.
Task 3: Containerization and Deployment: Create a comprehensive Dockerfile for the service. This file must include the command to correctly install Playwright and its necessary browser dependencies (npx playwright install --with-deps chromium).19 Deploy the final containerized application to a scalable platform such as Kubernetes, AWS ECS, or Google Cloud Run, configured behind a load balancer and CDN.
Works cited
What's the performance difference of puppeteer.launch() versus puppeteer.connect()?, accessed July 7, 2025, https://stackoverflow.com/questions/52431775/whats-the-performance-difference-of-puppeteer-launch-versus-puppeteer-connect
improve performance of markdown file processing · Issue #694 · mermaid-js/mermaid-cli, accessed July 7, 2025, https://github.com/mermaid-js/mermaid-cli/issues/694
Puppeteer vs. Playwright: Automated testing tools compared - Contentful, accessed July 7, 2025, https://www.contentful.com/blog/puppeteer-vs-playwright/
Playwright vs Puppeteer - BugBug.io, accessed July 7, 2025, https://bugbug.io/blog/testing-frameworks/playwright-vs-puppeteer/
mermaid-js/mermaid-cli: Command line tool for the ... - GitHub, accessed July 7, 2025, https://github.com/mermaid-js/mermaid-cli
child_process - Node documentation - Deno Docs, accessed July 7, 2025, https://docs.deno.com/api/node/child_process/
Child process | Node.js v24.3.0 Documentation, accessed July 7, 2025, https://nodejs.org/api/child_process.html
Server-side mermaid - Saltcorn wiki, accessed July 7, 2025, https://wiki.saltcorn.com/view/ShowPage/server-side-mermaid
Server Side Support · Issue #3650 · mermaid-js/mermaid - GitHub, accessed July 7, 2025, https://github.com/mermaid-js/mermaid/issues/3650
What makes mermaid.js cannot work on server-side, while other visualization libs can? · lumeland lume · Discussion #641 - GitHub, accessed July 7, 2025, https://github.com/lumeland/lume/discussions/641
Mermaid Diagrams in A Static Site Using MDX and Contentlayer, accessed July 7, 2025, https://respawn.io/posts/contentlayer-mermaid-diagrams
Could not find Chromium (rev. 1108766). This can occur if either - Stack Overflow, accessed July 7, 2025, https://stackoverflow.com/questions/78862090/could-not-find-chromium-rev-1108766-this-can-occur-if-either
matthewfeickert/mermaid-cli - Docker Image, accessed July 7, 2025, https://hub.docker.com/r/matthewfeickert/mermaid-cli
"new"}) is significantly slower than launch({headless: "old"}), especially for Page.pdf · Issue #10071 · puppeteer/puppeteer - GitHub, accessed July 7, 2025, https://github.com/puppeteer/puppeteer/issues/10071
Puppeteer vs. Playwright — Which One is Better? - Medium, accessed July 7, 2025, https://medium.com/@datajournal/puppeteer-vs-playwright-26236d90f37b
Playwright vs Puppeteer: Choosing the Right Browser Automation Tool in 2024 | by Shanika Wickramasinghe | Frontend Weekly | Medium, accessed July 7, 2025, https://medium.com/front-end-weekly/playwright-vs-puppeteer-choosing-the-right-browser-automation-tool-in-2024-d46d2cbadf71
Puppeteer vs Selenium vs Playwright, a speed comparison - Checkly, accessed July 7, 2025, https://www.checklyhq.com/blog/puppeteer-vs-selenium-vs-playwright-speed-comparison/
Playwright vs Puppeteer: What's the Difference? - Autify, accessed July 7, 2025, https://autify.com/blog/playwright-vs-puppeteer
remcohaszing/remark-mermaidjs: A remark plugin to render mermaid diagrams using playwright - GitHub, accessed July 7, 2025, https://github.com/remcohaszing/remark-mermaidjs
Playwright vs Puppeteer: Which to choose in 2024? | BrowserStack, accessed July 7, 2025, https://www.browserstack.com/guide/playwright-vs-puppeteer
Server-side rendering (SSR) using WebAssembly (Wasm) | by Timothy McCallum - Medium, accessed July 7, 2025, https://medium.com/wasm/server-side-rendering-ssr-using-webassembly-wasm-752841a13439
Server-side rendering with Rust and WebAssembly - Reddit, accessed July 7, 2025, https://www.reddit.com/r/rust/comments/onw13w/serverside_rendering_with_rust_and_webassembly/
Kroki!, accessed July 7, 2025, https://kroki.io/
Managed Kroki Service | Elest.io, accessed July 7, 2025, https://elest.io/open-source/kroki
yuzutech/kroki: Creates diagrams from textual descriptions! - GitHub, accessed July 7, 2025, https://github.com/yuzutech/kroki
Architecture - Kroki Documentation, accessed July 7, 2025, https://docs.kroki.io/kroki/architecture/
Install Kroki :: Kroki Documentation, accessed July 7, 2025, https://docs.kroki.io/kroki/setup/install/
Self hosted : slow performances - Technical Help - Baserow, accessed July 7, 2025, https://community.baserow.io/t/self-hosted-slow-performances/2213
Hardware requirements? : r/selfhosted - Reddit, accessed July 7, 2025, https://www.reddit.com/r/selfhosted/comments/1asrm27/hardware_requirements/
mermaid-js/mermaid-live-editor: Edit, preview and share ... - GitHub, accessed July 7, 2025, https://github.com/mermaid-js/mermaid-live-editor
Getting Started - Mermaid Chart, accessed July 7, 2025, https://docs.mermaidchart.com/mermaid-oss/intro/getting-started.html
Mermaid Chart: Home, accessed July 7, 2025, https://www.mermaidchart.com/
Mermaid Chart, a Markdown-like tool for creating diagrams, raises $7.5M | Hacker News, accessed July 7, 2025, https://news.ycombinator.com/item?id=39772520
How to use Request js (Node js Module) pools - Stack Overflow, accessed July 7, 2025, https://stackoverflow.com/questions/19043355/how-to-use-request-js-node-js-module-pools
Is it better to use SVG or JPG/PNG images for page performance? [closed] - Stack Overflow, accessed July 7, 2025, https://stackoverflow.com/questions/64583456/is-it-better-to-use-svg-or-jpg-png-images-for-page-performance
PNG vs. SVG: What are the differences? | Adobe, accessed July 7, 2025, https://www.adobe.com/creativecloud/file-types/image/comparison/png-vs-svg.html
www.adobe.com, accessed July 7, 2025, https://www.adobe.com/creativecloud/file-types/image/comparison/png-vs-svg.html#:~:text=While%20PNGs%20are%20capable%20of,size%20without%20losing%20their%20resolution.
SVG vs PNG: 4 Key Differences and How to Choose | Cloudinary, accessed July 7, 2025, https://cloudinary.com/guides/image-formats/svg-vs-png-4-key-differences-and-how-to-choose
A Developer's Guide to SVG Optimization - Cloudinary, accessed July 7, 2025, https://cloudinary.com/guides/image-formats/a-developers-guide-to-svg-optimization
SVGOMG - Optimize and minify SVG images, accessed July 7, 2025, https://svgomg.net/
Drawing diagrams in Sanity with Mermaid.js - Raymond Julin, accessed July 7, 2025, https://www.raymondjulin.com/blog/drawing-diagrams-in-sanity-with-mermaid-js
Usage - Mermaid, accessed July 7, 2025, https://mermaid.js.org/config/usage.html
What is Cached Images and Files? - Cloudinary, accessed July 7, 2025, https://cloudinary.com/guides/web-performance/what-is-cached-images-and-files
Cost-Effective Image Management: Maximizing Efficiency Through Network Image Caching in Mobile Apps - DEV Community, accessed July 7, 2025, https://dev.to/tentanganak/cost-effective-image-management-maximizing-efficiency-through-network-image-caching-in-mobile-apps-1hhn
Simple Image Caching Techniques - Medium, accessed July 7, 2025, https://medium.com/@satindersingh71/simple-image-caching-techniques-ab58a06b7fb8
How to Optimize Image Caching in Next.js for Blazing Fast Loading Times - Medium, accessed July 7, 2025, https://medium.com/@melvinmps11301/how-to-optimize-image-caching-in-next-js-for-blazing-fast-loading-times-1275721dbe58
Boost Your Workflow with Effective Image Rendering Methods - Cloudinary, accessed July 7, 2025, https://cloudinary.com/guides/web-performance/rendering-images
What is image service caching?—ArcGIS Server, accessed July 7, 2025, https://enterprise.arcgis.com/en/server/11.4/publish-services/linux/what-is-image-service-caching-.htm
REST Architecture - Part 1: Building The API - Client-Server Systems - Auth0, accessed July 7, 2025, https://auth0.com/blog/rest-architecture-part-1-building-api/
Design patterns for modern web APIs | by David Luecke | The Feathers Flightpath, accessed July 7, 2025, https://blog.feathersjs.com/design-patterns-for-modern-web-apis-1f046635215
Connection pooling in Node.js - Medium, accessed July 7, 2025, https://medium.com/@amr258144/connection-pooling-in-node-js-ea4421c72dc
The Node.js Event Loop, accessed July 7, 2025, https://nodejs.org/en/learn/asynchronous-work/event-loop-timers-and-nexttick
