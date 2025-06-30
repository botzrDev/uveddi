# CodeAtlas Development TODO List

**[Update June 30, 2025: Gold-standard RAG pipeline, multi-layered hallucination defense, and advanced semantic search are fully implemented and tested. All core AI Reasoning Engine tasks for Sprint 5 are complete. See SPRINTS.md for details.]**

**[Update June 30, 2025, PM]: Core CLI config command, ER data model (User, Organization, Project, AnalysisRun), serialization, and anti-pattern type integration are now implemented and tested. Reporting system now categorizes issues by anti-pattern type.**]

This document outlines the major tasks and milestones for the development of CodeAtlas, an AI-powered CLI tool for architectural analysis. It is structured to follow the logical progression of a software project, from foundational setup to deployment and ongoing maintenance.

## Phase 1: Foundational Setup & Core CLI (Rust)

### 1.1 Directory Structure Analysis & Setup
- [x] Review existing `src/` and `docs/` structure.
- [x] Define and create new top-level directories (e.g., `config/`, `tests/`, `plugins/`).
- [X] Update `.gitignore` to reflect new directories and build artifacts.
- [X] Initialize Rust project (`Cargo.toml`, `Cargo.lock`).

### [CDAT-4]1.2 Core CLI Framework
- [x] Implement basic CLI command parsing using `clap` crate.
    - [x] `codeatlas analyze <path>` command.
    - [x] `codeatlas init-local-ai` command.
    - [x] `codeatlas config` command.
- [x] Implement basic logging and error handling.
- [ ] Set up project for cross-platform compilation and distribution.

### [CDAT-5]1.3 Configuration Management
- [x] Implement configuration loading (e.g., `.archlintignore`, API keys, analysis scopes).
    - [x] Support environment variables for sensitive data (API keys).
    - [x] Implement secure configuration file handling.

### [CDAT-6]1.4 Data Model Implementation
- [x] Implement core data structures representing the Entity-Relationship model
- [x] Define structures for User, Organization, Project, AnalysisRun, etc.
- [x] Implement serialization/deserialization for these models
- [ ] Design storage strategy (filesystem-based vs. database)

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
- [x] Implement caching of parsed ASTs for performance optimization.
    - [x] Replace re-parsing with a Rust-native, serializable `CustomAst` structure for disk cache.
    - [x] Update all analysis logic (including God Object detection) to use the cache-backed `CustomAst`.
    - [x] Update and validate all relevant tests to ensure cache correctness.
    - [x] Update documentation (SPRINTS.md, TODO.md) to reflect new caching strategy.

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
- [x] Implement `codeatlas init-local-ai` command to facilitate Ollama setup and model download.
- [x

### 3.2 External LLM API Integration
- [x] Implement clients for OpenAI (GPT-4), Anthropic (Claude 3), and Google (Gemini) LLM APIs.
- [x] Implement secure handling of API keys (environment variables/secure config).
- [x] Integrate API-based LLMs into the AI Reasoning Engine.
- [x] Implement robust error handling and user feedback for missing API keys or LLM failures.

### [CDAT-7] 3.3 AI Reasoning Engine & Prompt Engineering
- [x] Implement smart prompting/RAG strategy:
    - [x] Construct prompts embedding contextual code snippets and structural information from AST analysis.
    - [x] Implement hallucination mitigation techniques (structured prompting, uncertainty handling).
- [x] Develop logic for AI-generated explanations, titles, descriptions, and refactoring suggestions for architectural issues.
- [x] Implement multi-layered hallucination defense strategy:
    - [x] Implement advanced RAG to ground the LLM in codebase facts
    - [x] Design structured prompts with explicit format constraints
    - [x] Implement self-correction loops (critic LLM reviews primary LLM output)
    - [x] Build clear human-in-the-loop verification workflows
- [x] Implement hybrid verification pipeline (cross-validate static analysis with AI)
- [x] Add confidence scoring to AI explanations
- [x] Update prompt templates with verification steps
- [x] Add comprehensive unit and integration tests for all new modules (semantic search, prompt builder, schema validation, self-correction, CLI review)
- [x] (Optional) Integrate real BM25 for hybrid search
- [x] (Optional) Add advanced trust scoring, provenance, and LTR features
- [x] (Optional) Update documentation and README with test instructions and usage examples

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

## Phase 5: Extensibility & Plugin System

### 5.1 Plugin System Core
- [x] Define a stable plugin interface (Rust traits).
- [x] Implement plugin discovery and loading mechanism (e.g., from a `plugins/` directory).
- [ ] Implement sandboxing for security using WebAssembly (WASM) runtime.
- [ ] Develop data exchange mechanisms between core and plugins.

### 5.2 Example Plugin Development
- [ ] Create a simple example plugin to validate the system.
- [ ] Document the plugin development process.

## Phase 6: Testing & Quality Assurance

### 6.1 Unit Testing
- [x] Write comprehensive unit tests for all new AI Reasoning Engine modules (semantic search, prompt builder, schema validation, self-correction, CLI review)
- [x] Add real-world multi-language codebase fixtures for testing.

### 6.2 Integration Testing
- [x] Develop integration tests for the full RAG/AI pipeline.
- [x] Test local and API-based AI model interactions.

### 6.3 Performance Testing
- [ ] Conduct performance benchmarks on large codebases.
- [ ] Identify and address performance bottlenecks (e.g., parsing, AI inference).

### 6.4 User Acceptance Testing (UAT) / Beta Program
- [ ] Recruit beta testers (Senior Devs, Tech Leads, Architects).
- [ ] Gather feedback and iterate on features and UX.

### 6.5 Real-World Project Benchmarking
- [ ] Select 3-5 popular open-source projects for benchmark analysis
- [ ] Document architectural issues discovered in these projects
- [ ] Create case studies showcasing CodeAtlas' effectiveness
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
