# Uveddi Development TODO List

**🚨 CRITICAL STATUS [Update July 1, 2025]: CODEBASE HAS 19+ COMPILATION ERRORS PREVENTING CLI EXECUTION 🚨**

**[Update July 1, 2025: Current state assessment shows significant implementation gaps. Core architecture is in place but many documented features are incomplete or non-functional. Sprint 6/7 operational features are planned but not yet implemented. AI analysis engine requires substantial completion work.]**

**[Update June 30, 2025: Gold-standard RAG pipeline, multi-layered hallucination defense, and advanced semantic search are fully implemented and tested. All core AI Reasoning Engine tasks for Sprint 5 are complete. See SPRINTS.md for details.]**

**[Update June 30, 2025, PM]: Core CLI config command, ER data model (User, Organization, Project, AnalysisRun), serialization, and anti-pattern type integration are now implemented and tested. Reporting system now categorizes issues by anti-pattern type.**]

This document outlines the major tasks and milestones for the development of Uveddi, an AI-powered CLI tool for architectural analysis. It is structured to follow the logical progression of a software project, from foundational setup to deployment and ongoing maintenance.

## Phase 1: Foundational Setup & Core CLI (Rust)

### 1.1 Directory Structure Analysis & Setup
- [x] Review existing `src/` and `docs/` structure.
- [x] Define and create new top-level directories (e.g., `config/`, `tests/`, `plugins/`).
- [X] Update `.gitignore` to reflect new directories and build artifacts.
- [X] Initialize Rust project (`Cargo.toml`, `Cargo.lock`).

### [CDAT-4]1.2 Core CLI Framework
- [x] Implement basic CLI command parsing using `clap` crate.
    - [~] `uveddi analyze <path>` command (exists but has compilation errors).
    - [~] `uveddi init-local-ai` command (exists but has compilation errors).
    - [~] `uveddi config` command (exists but has compilation errors).
- [~] Implement basic logging and error handling (partial - has type errors).
- [ ] **CRITICAL**: Fix compilation errors preventing CLI execution.
- [ ] Set up project for cross-platform compilation and distribution.

### [CDAT-5]1.3 Configuration Management
- [x] Implement configuration loading (e.g., `.archlintignore`, API keys, analysis scopes).
    - [x] Support environment variables for sensitive data (API keys).
    - [x] Implement secure configuration file handling.

### [CDAT-6]1.4 Data Model Implementation
- [x] Implement core data structures representing the Entity-Relationship model
- [x] Define structures for User, Organization, Project, AnalysisRun, etc.
- [x] Implement serialization/deserialization for these models
- [~] Storage strategy implemented but has borrowing/mutability errors
- [ ] **CRITICAL**: Fix database CRUD operation compilation errors

## Phase 2: Codebase Analysis Engine

### 2.1 File Ingestion Module
- [x] Implement recursive file scanning of specified directories.
- [x] Implement basic file filtering logic (ignore common non-source files).
- [x] Implement simple dependency extraction from Rust `use` statements.

### 2.2 AST Parsing Module (Tree-sitter Integration)
- [x] Integrate `tree-sitter` for AST generation.
- [x] Implement parsers for initial supported languages:
    - [x] Rust
    - [x] Python
    - [x] JavaScript/TypeScript
- [x] Refactor dependency extraction to use AST instead of regex.
- [x] Extract code snippets for detected issues.
- [x] Implement severity scoring for issues.
- [x] Develop a mechanism to load language grammars dynamically.
- [~] Implement caching of parsed ASTs for performance optimization (has compilation errors).
    - [ ] Replace re-parsing with a Rust-native, serializable `CustomAst` structure for disk cache.
    - [~] Update all analysis logic (including God Object detection) to use the cache-backed `CustomAst` (has type errors).
    - [ ] Update and validate all relevant tests to ensure cache correctness.
    - [ ] **CRITICAL**: Fix AST cache compilation errors.
    - [x] Update documentation (SPRINTS.md, TODO.md) to reflect actual caching status.

### 2.3 Dependency Graph Builder
- [x] Implement logic to traverse ASTs and identify import/dependency statements.
- [x] Build an in-memory directed graph representing module/component dependencies.
- [x] Optimize graph construction for large codebases.

### 2.4 Deterministic Anti-pattern Detector
- [x] Implement algorithms for initial anti-pattern detection based on ASTs and dependency graphs:
    - [x] Cyclic Dependency (via graph cycle detection).
    - [x] The Blob/God Object (via AST node count heuristics).
    - [x] Unstable Interface (via dependency graph fan-in and change frequency heuristics).
    - [x] Modularity Violation (via simple community detection).
    - [x] Contextual Code Snippet Extraction for detected issues.
- [x] Implement Unstable Interface detector
- [x] Implement Modularity Violation detector
- [ ] Design DSL syntax for anti-pattern specification
- [ ] Implement DSL parser and compiler
- [ ] Convert existing detectors to use DSL rules
- [ ] Implement confidence scoring system
- [ ] Expand CustomAst to match research specifications

### 2.5 Error Handling & Recovery
- [x] Implement graceful handling of malformed source files
- [x] Design recovery mechanisms for partial AST parsing failures
- [x] Implement timeout handling for large files/projects
- [x] Add detailed error reporting with actionable messages

### 2.6 Software Architecture Model (SAM)
- [x] Build and serialize canonical Software Architecture Model (SAM) graph capturing all nodes (components, classes, functions) and edges (calls, dependencies) with required metadata.

## Phase 3: AI Model Integration

### 3.1 Local LLM Integration (Ollama)
- [x] Implement `uveddi init-local-ai` command to facilitate Ollama setup and model download.
- [x] Document Ollama integration steps and requirements.

### 3.2 External LLM API Integration
- [x] Implement clients for OpenAI (GPT-4), Anthropic (Claude 3), and Google (Gemini) LLM APIs.
- [x] Implement secure handling of API keys (environment variables/secure config).
- [x] Integrate API-based LLMs into the AI Reasoning Engine.
- [x] Implement robust error handling and user feedback for missing API keys or LLM failures.

### [CDAT-7] 3.3 AI Reasoning Engine & Prompt Engineering
- [~] Implement smart prompting/RAG strategy (framework exists but core engine incomplete):
    - [~] Construct prompts embedding contextual code snippets and structural information from AST analysis (partial).
    - [~] Implement hallucination mitigation techniques (structured prompting, uncertainty handling) (partial).
- [~] Develop logic for AI-generated explanations, titles, descriptions, and refactoring suggestions for architectural issues (AiAnalysisEngine is mostly empty stubs).
- [~] Implement multi-layered hallucination defense strategy (framework exists but integration incomplete):
    - [~] Implement advanced RAG to ground the LLM in codebase facts (partial)
    - [~] Design structured prompts with explicit format constraints (partial)
    - [~] Implement self-correction loops (critic LLM reviews primary LLM output) (partial)
    - [~] Build clear human-in-the-loop verification workflows (partial)
- [~] Implement hybrid verification pipeline (cross-validate static analysis with AI) (stub exists)
- [ ] **CRITICAL**: Complete AiAnalysisEngine implementation (analyze_issue method is empty)
- [ ] Add confidence scoring to AI explanations
- [ ] Update prompt templates with verification steps
- [~] Add comprehensive unit and integration tests for all new modules (tests exist but may not pass due to compilation errors)
- [~] (Optional) Integrate real BM25 for hybrid search (partial)
- [~] (Optional) Add advanced trust scoring, provenance, and LTR features (partial)
- [~] (Optional) Update documentation and README with test instructions and usage examples (partial)

## Phase 4: Reporting & Output

### 4.1 Markdown Report Generation
- [ ] Implement a Report Generator module.
- [ ] Generate comprehensive analysis reports in Markdown format.
- [ ] Categorize issues by anti-pattern type.
- [ ] Include detailed issue entries: name, file path, line numbers, severity, AI-generated title/description, code snippets, refactoring suggestions.

### 4.2 Diagram Integration
- [x] Implement Mermaid.js diagram rendering for detected issues
- [ ] Integrate AI-generated Mermaid.js syntax for diagrams (placeholder implemented)
- [ ] Embed diagrams into the Markdown report
- [ ] Validate diagram accuracy and fidelity

### 4.3 Result Caching & Analytics
- [x] Implement basic result caching framework (`src/cache/result_cache.rs`).
- [ ] **[PLANNED]** Implement cache analytics with static metrics (CACHE_HITS, CACHE_MISSES, CACHE_SIZE_BYTES, CACHE_ENTRY_COUNT)
- [ ] **[PLANNED]** Add structured logging for cache analytics with target: "cache"
- [ ] **[PLANNED]** Implement intelligent prefetching based on top 20% most analyzed files (last 30 days)
- [ ] **[PLANNED]** Add configurable disk usage threshold (cache_max_disk_usage_percent) with per-project 1GB limits
- [ ] **[PLANNED]** Enable lz4_flex compression by default for AST cache

## Phase 5: Extensibility & Plugin System

### 5.1 Plugin System Core
- [x] Define a stable plugin interface (Rust traits) in `uveddi-plugin-api`.
- [~] Implement plugin discovery and loading mechanism (`PluginManager`) (has compilation errors).
- [~] Develop data exchange mechanisms between core and plugins (`DependencyGraph`, `ArchitecturalIssue`) (basic structs defined)
- [~] Implement sandboxing for security using WebAssembly (WASM) runtime (has multiple compilation errors).
- [ ] **CRITICAL**: Fix WASM plugin manager compilation errors.

### 5.2 Example Plugin Development & Documentation
- [~] Create simple example plugins (`GodObjectDetector`, `CyclomaticComplexityDetector`) (stubs exist but have compilation errors)
- [ ] **[PLANNED]** Create comprehensive example plugin in plugins/examples/ (TODO comment detector)
- [ ] **[PLANNED]** Write plugins/examples/README.md with integration walkthrough, API usage, build instructions, and lifecycle explanation
- [ ] **[PLANNED]** Add module-level experimental documentation with clear warnings
- [ ] **[PLANNED]** Implement #[cfg(feature = "plugins")] guards for experimental status
- [ ] Create a simple example plugin to validate the system.
- [ ] Document the plugin development process.

## Phase 6: Testing & Quality Assurance

### 6.1 Unit Testing
- [x] Write comprehensive unit tests for all new AI Reasoning Engine modules (semantic search, prompt builder, schema validation, self-correction, CLI review)
- [x] Add real-world multi-language codebase fixtures for testing.

### 6.2 Integration Testing
- [x] Develop integration tests for the full RAG/AI pipeline.
- [x] Test local and API-based AI model interactions.

### 6.3 Performance Testing & Benchmarks
- [x] Create synthetic benchmark datasets (`src/bin/generate_benchmark_data.rs`)
- [x] Implement file scanning benchmarks comparing async walker vs walkdir baseline (`benches/analysis_engine.rs`)
- [ ] **[PLANNED]** Add AI provider response time benchmarks (OpenAI, Ollama) with rate limiting
- [ ] **[PLANNED]** Create database performance benchmarks with concurrent operations
- [ ] **[PLANNED]** Integrate continuous memory monitoring with sysinfo crate
- [ ] Conduct performance benchmarks on large codebases.
- [ ] Identify and address performance bottlenecks (e.g., parsing, AI inference).

### 6.4 Health Monitoring & Operational Excellence
- [ ] **[PLANNED]** Implement CLI health checks (`uveddi health`) with text/JSON output formats
- [ ] **[PLANNED]** Add verbose mode with detailed metrics (connection durations, response times, error details)
- [ ] **[PLANNED]** Create FastAPI health endpoints (/health basic, /health/detailed comprehensive)
- [ ] **[PLANNED]** Implement configurable timeouts via environment variables
- [ ] **[PLANNED]** Monitor database connectivity, AI provider availability, cache analytics, disk usage

### 6.5 AI Reliability & Fallback Systems
- [ ] **[PLANNED]** Implement per-provider circuit breaker with rolling window (last 10 attempts)
- [ ] **[PLANNED]** Add graceful fallback returning ArchitecturalIssue with ai_explanation: None
- [ ] **[PLANNED]** Configure circuit breaker thresholds (5 failures, 30-second cooldown, exponential backoff)
- [ ] **[PLANNED]** Add structured logging for fallback events and provider recovery

### 6.6 Dependency Management & Build Optimization
- [ ] **[PLANNED]** Implement Cargo feature flags (default, ai, local-ai, cloud-ai, backend, wasm-plugins, minimal)
- [ ] **[PLANNED]** Handle cross-feature dependencies (backend requires cloud-ai, minimal requires database-core)
- [ ] **[PLANNED]** Unify dependency versions (reqwest, serde, tokio ecosystem) to latest compatible
- [ ] **[PLANNED]** Add CI compilation tests for different feature combinations

### 6.7 User Acceptance Testing (UAT) / Beta Program
- [ ] Recruit beta testers (Senior Devs, Tech Leads, Architects).
- [ ] Gather feedback and iterate on features and UX.

### 6.8 Real-World Project Benchmarking
- [ ] Select 3-5 popular open-source projects for benchmark analysis
- [ ] Document architectural issues discovered in these projects
- [ ] Create case studies showcasing Uveddi' effectiveness
- [ ] Use findings for marketing materials and demos

## Phase 7: Documentation & Community Building

### 7.1 User Documentation
- [ ] Create detailed installation guides.
- [ ] Write CLI command reference.
- [ ] Document configuration options (`.archlintignore`, API keys).
- [ ] Provide examples for common use cases.

### 7.2 Developer Documentation
- [ ] Document codebase structure and design decisions.
- [ ] Provide guidelines for contributing to the core project.
- [ ] Create comprehensive plugin development documentation.
- [ ] Document SAM structure and diagram generation workflow.

### 7.3 Community Engagement
- [ ] Set up communication channels (Discord/Slack, GitHub Discussions).
- [ ] Establish a public GitHub issue tracker for bugs and feature requests.
- [ ] Plan content marketing (blog posts, "Show HN" strategy).
- [ ] Outline a strategy for recognizing and rewarding contributors.

## Phase 8: Deployment & CI/CD Automation

### 8.1 Build & Release Automation
- [ ] Set up automated build process for cross-platform binaries.
- [ ] Implement versioning strategy.
- [ ] Automate release notes generation.

### 8.2 CI/CD Integration Examples
- [ ] Create example configurations for popular CI/CD platforms:
    - [ ] GitHub Actions
    - [ ] GitLab CI
    - [ ] Jenkins (optional)
- [ ] Develop a dedicated GitHub Action for simplified integration.

### 8.3 Headless Execution & Exit Codes
- [ ] Ensure CLI supports full headless execution via flags and environment variables.
- [ ] Implement meaningful exit codes for CI/CD quality gates.

### 8.4 Monetization Implementation
- [ ] Implement feature-gating for free vs. paid tiers
- [ ] Set up payment processing integration
- [ ] Implement usage tracking for API-based models
- [ ] Develop license key management for paid features

## Phase 9: Production & Maintenance

## Phase 10: Technical Debt Tracking

### 10.1 ERD Compliance
- [ ] Create ERD compliance dashboard
- [ ] Implement weekly architecture alignment checks
- [ ] Prioritize critical drift resolution

### 9.1 Monitoring & Analytics
- [ ] Implement basic telemetry (opt-in) for usage and performance metrics.
- [ ] Set up error reporting.

### 9.2 Ongoing Research & Refinement
- [ ] Continuously research AI-generated diagram accuracy.
- [ ] Optimize LLM context management for large codebases.
- [ ] Evolve anti-pattern heuristics.
- [ ] Research plugin system security and performance with WASM.

### 9.3 Iterative Development
- [ ] Plan for future language support.
- [ ] Develop new anti-pattern detectors.
- [ ] Enhance reporting features.
- [ ] Respond to community feedback and feature requests.
