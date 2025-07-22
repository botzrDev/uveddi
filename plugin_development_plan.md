# Uveddi Plugin Development Plan

This document outlines the development plan for three key plugins for the Uveddi architectural analysis CLI: the Custom Rule / Policy Engine Plugin, the AI-Generated Code Quality Linter Plugin, and the Advanced Code Metrics and Visualization Plugin. Each section details the problem solved, how the plugin helps, and a concrete plan for its implementation, followed by general implementation and quality considerations.

---

## 1. Custom Rule / Policy Engine Plugin

**Problem Solved:** Engineering teams often have unique architectural standards, domain-specific patterns, and coding conventions that off-the-shelf analysis tools cannot address. Manually enforcing these standards leads to inconsistency and consumes valuable developer time, resulting in architectural drift where implementation deviates from planning.

**How it Helps:** This plugin will transform Uveddi into a governance layer for internal standards. It empowers users to define their own architectural policies and coding standards, providing automated enforcement and quantifiable reports on adherence. This ensures that an organization's specific architectural principles are consistently applied, reducing technical debt and improving maintainability.

**Implementation Plan:**

1.  **Leverage Uveddi's Plugin Architecture:**
    *   Develop the custom rule engine as a WASM-based plugin, utilizing Uveddi's existing `WasmPluginDetectorAdapter` capabilities for secure, external, and performant loading.
    *   Define the plugin's interface for rule ingestion and violation reporting.

2.  **Declarative Rule Definition:**
    *   Design a Domain-Specific Language (DSL) or a TOML-based configuration format for defining custom rules. This format should be human-readable and allow for clear specification of policies.
    *   Implement a parser within the plugin to interpret the declarative rule definitions.

3.  **Unified Code Representation & Tree-sitter Query-Based Logic:**
    *   Ensure the plugin can access and operate on Uveddi's unified Code Property Graph (CPG) or Abstract Syntax Tree (AST), which is built using Tree-sitter.
    *   Implement custom rules as Tree-sitter queries. This will enable precise pattern matching within the AST to detect language-specific constructs or forbidden patterns across Rust, Python, and JavaScript/TypeScript codebases.
    *   Develop a query engine within the plugin that can execute these Tree-sitter queries against the parsed AST.

4.  **Integration with Reporting:**
    *   Design the plugin to output rule violations in a structured format compatible with Uveddi's reporting system.
    *   Ensure custom rule violations are seamlessly integrated into Uveddi's actionable Markdown reports with embedded Mermaid.js diagrams.
    *   Explore visual highlights (e.g., color-coding) to clearly show where policy violations occur in the reports.

5.  **Configuration Management:**
    *   Develop mechanisms for users to define, store, and version-control their custom policy configurations, potentially using shared configuration templates.

6.  **Validation Framework:**
    *   Establish a rigorous validation framework for custom rules, similar to existing anti-pattern detectors.
    *   Create benchmark datasets and specific test cases to ensure rules correctly identify violations without false positives.

---

## 2. AI-Generated Code Quality Linter Plugin

**Problem Solved:** The rise of AI code generators introduces challenges in ensuring the quality and security of automatically generated code, which can contribute to technical debt, hallucinations, inefficiencies, or deviations from best practices. Developers need verification and guardrails to trust AI outputs.

**How it Helps:** This plugin will act as a "trusted CLI" verification layer for AI-assisted development. It will analyze code, specifically flagging patterns common in AI-generated outputs that indicate potential issues. By leveraging Uveddi's dual-AI model (local for ground truth analysis, cloud for deep reasoning), it provides automated verification to ensure AI-generated code conforms to sound, scalable, and secure architectural principles.

**Implementation Plan:**

1.  **Hybrid AI Model Integration:**
    *   Leverage Uveddi's existing capability to support both local (Ollama) and cloud-based (OpenAI, Anthropic, Google Gemini) AI models.
    *   Define the plugin's interaction model with these AI backends for code analysis and claim extraction.

2.  **Grounding and De-Hallucination:**
    *   Implement an iterative grounding loop and a "De-Hallucinator" technique.
    *   When the LLM makes a verifiable claim (e.g., "Method X calls Method Y"), programmatically confirm it by querying the code knowledge graph (CPG).
    *   Flag claims as hallucinations if they cannot be grounded in the factual code structure.
    *   Define and categorize a hallucination taxonomy relevant to code analysis (e.g., factual contradictions, intent deviations, internal inconsistencies).

3.  **Claim Extraction and Verification:**
    *   Utilize an LLM to parse AI-generated code for specific claims (e.g., "Class A depends on Class B").
    *   Verify each extracted claim against deterministic sources of truth, such as the code graph or compiler warnings.

4.  **High-Value Test Suite:**
    *   Develop a robust evaluation test suite using curated real-world examples and programmatic synthetic data generation.
    *   Generate code examples with specific anti-patterns or inject flaws into clean code to test detection capabilities.
    *   Consider using existing benchmarks like CodeXGLUE to assess underlying code understanding.

5.  **Confidence Scoring:**
    *   Integrate with Uveddi's existing confidence scoring system for AI-generated explanations.
    *   Potentially adopt a probabilistic model for detection confidence to provide users with certainty levels for findings.

6.  **Feedback Loop for Improvement:**
    *   Implement a feedback loop to collect user-provided verdicts (True Positive/False Positive) on AI-generated findings (with consent).
    *   Use this data to identify weaknesses in deterministic detectors and refine the deterministic rules over time, making the system self-improving.

7.  **Actionable Reports:**
    *   Present findings as part of Uveddi's structured reports.
    *   Include AI-generated explanations and refactoring suggestions for identified issues.

---

## 3. Advanced Code Metrics and Visualization Plugin

**Problem Solved:** Developers and architects require more granular, quantitative insights into code health than high-level anti-pattern detection alone provides. Nuanced metrics like cyclomatic complexity, deeper cohesion metrics, or historical change coupling can reveal subtle issues or inform refactoring efforts more precisely.

**How it Helps:** This plugin will implement a broader suite of industry-standard code metrics and offer specialized visualizations to provide deeper understanding. It will provide actionable data to manage and reduce technical debt and make more informed refactoring decisions.

**Implementation Plan:**

1.  **Core Metric Calculation:**
    *   **Cyclomatic Complexity:** Leverage Uveddi's Tree-sitter parsing and Control Flow Graph (CFG) construction capabilities. Extend the "Enhancing Long Methods Detector" to expose cyclomatic complexity per function.
    *   **Coupling Metrics:** Complete the implementation of the Tight Coupling detector to compute Fan-in/Fan-out, Coupling Between Objects (CBO), and Response For a Class (RFC). These metrics will rely on Uveddi's robust, multi-language dependency graph.
    *   **Cohesion Metrics:** Extend the existing God Object detector's LCOM4 (Lack of Cohesion in Methods) analysis to be a more general, exposeable metric.

2.  **Historical (Change) Coupling Analysis:**
    *   **Git Integration:** Implement integration with Git repositories to pull historical commit data.
    *   **Dynamic Knowledge Graph Construction:** Explore dynamically updating the knowledge graph by processing Git commits incrementally to enable temporal analysis.
    *   **Behavioral Analysis:** Develop capabilities to mimic CodeScene's behavioral analysis of Git history to identify architectural hotspots and change coupling patterns.

3.  **Advanced Visualization:**
    *   **Mermaid.js Templates:** Utilize Uveddi's existing Mermaid.js diagram generation capabilities.
    *   **Treemaps:** Implement Component Treemaps to clearly show the location and volume of metrics (e.g., complexity) within the project hierarchy.
    *   **Visual Encoding:** Employ severity-based visual encoding using color schemes, shape/size variations, and distinct iconography to highlight issues. Ensure accessibility with colorblind-friendly palettes.
    *   **Layout Optimization:** Implement automatic layout algorithms for readability and ensure the system can effectively handle large codebases (e.g., with zoom/pan capabilities).

4.  **WASM Plugin Data Ingestion:**
    *   The plugin will be responsible for calculating these metrics and then feeding them into Uveddi's visualization pipeline, leveraging the `WasmPluginDetectorAdapter` instances.

5.  **Actionable Reports:**
    *   Deliver these insights through structured, portable Markdown reports with integrated diagrams and visualizations.

---

## General Implementation & Quality Considerations for All Plugins:

*   **Phased Rollout:** Implement all plugins in controlled phases, allowing for iterative refinement and early value delivery.
*   **Rigorous Testing:** Conduct comprehensive unit and integration testing, covering multi-language scenarios, configurable thresholds, and performance aspects. Isolate deterministic components for unit testing and use seeded random number generators for reproducible integration tests.
*   **Robust Error Handling:** Employ Uveddi's two-tiered error handling strategy, utilizing `thiserror` for structured, recoverable errors within library code and `anyhow` for application-level error propagation and rich diagnostics.
*   **Comprehensive Documentation:** Adhere to Uveddi's standardized documentation structure for each plugin, including clear explanations of detection strategies, confidence scoring, language-specific behaviors, configuration options, and usage examples.
*   **CI/CD Integration:** Design plugins to support headless execution and produce exit codes suitable for integration into CI/CD pipelines as architectural quality gates.
*   **Performance and Scalability:** Prioritize performance by leveraging Rust's concurrency features (e.g., `rayon` for parallelizing AST parsing) and ensuring efficient memory usage, especially for large codebases.

---

## Plugin Publishing for the Community

To enable community access and distribution of Uveddi plugins, the following considerations and steps are proposed:

1.  **WASM Module Distribution:**
    *   Plugins, being WASM modules, can be distributed as standalone `.wasm` files.
    *   A dedicated registry or a well-defined community repository (e.g., a GitHub organization or a specific `uveddi-plugins` repository) could host these compiled WASM modules.

2.  **Plugin Discovery and Installation:**
    *   **CLI Integration:** Enhance the Uveddi CLI with commands for `uveddi plugin install <plugin-name>` and `uveddi plugin list`.
    *   **Configuration:** Allow users to specify plugin sources (e.g., local paths, remote URLs, or registry names) in their Uveddi configuration.
    *   **Dependency Management:** If plugins have external dependencies (e.g., specific Tree-sitter grammars), the installation process should handle these.

3.  **Versioning and Compatibility:**
    *   Implement a clear versioning strategy for plugins (e.g., Semantic Versioning).
    *   Ensure backward compatibility where possible, or provide clear guidance on breaking changes.
    *   The Uveddi core should be able to identify and load plugins compatible with its current version.

4.  **Security and Trust:**
    *   Given WASM's sandboxed nature, plugins offer inherent security. However, a mechanism for verifying plugin integrity (e.g., checksums, digital signatures) could be considered for enhanced trust.
    *   Community guidelines for plugin submission and review would be essential to maintain quality and security.

5.  **Documentation and Examples:**
    *   Provide comprehensive documentation for plugin developers on how to build, test, and publish their plugins.
    *   Offer example plugins (e.g., a simple "Hello World" plugin or a basic custom rule) to serve as templates.

6.  **Community Contribution Workflow:**
    *   Establish a clear contribution workflow for community-developed plugins, including pull request templates, code review processes, and testing requirements.
    *   Leverage existing Uveddi community guidelines for consistency.

7.  **Monetization/Premium Plugins (Future Consideration):**
    *   While initially focusing on open-source community plugins, the architecture could potentially support a marketplace for premium or enterprise-specific plugins in the future, if desired. This would require a more robust licensing and distribution mechanism.
