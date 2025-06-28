# CodeAtlas Development TODO List

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
    - [ ] `codeatlas init-local-ai` command.
    - [ ] `codeatlas config` command.
- [x] Implement basic logging and error handling.
- [ ] Set up project for cross-platform compilation and distribution.

### [CDAT-5]1.3 Configuration Management
- [ ] Implement configuration loading (e.g., `.archlintignore`, API keys, analysis scopes).
    - [ ] Support environment variables for sensitive data (API keys).
    - [ ] Implement secure configuration file handling.

### [CDAT-6]1.4 Data Model Implementation
- [ ] Implement core data structures representing the Entity-Relationship model
- [ ] Define structures for User, Organization, Project, AnalysisRun, etc.
- [ ] Implement serialization/deserialization for these models
- [ ] Design storage strategy (filesystem-based vs. database)

## Phase 2: Codebase Analysis Engine

### 2.1 File Ingestion Module
- [ ] Implement recursive file scanning of specified directories.
- [ ] Implement `.archlintignore` parsing and file filtering logic.
- [ ] Handle various file types and encodings.

### 2.2 AST Parsing Module (Tree-sitter Integration)
- [ ] Integrate `tree-sitter` for AST generation.
- [ ] Implement parsers for initial supported languages:
    - [ ] Rust
    - [ ] Python
    - [ ] JavaScript/TypeScript
- [ ] Develop a mechanism to load language grammars dynamically.
- [ ] Implement caching of parsed ASTs for performance optimization.

### 2.3 Dependency Graph Builder
- [ ] Implement logic to traverse ASTs and identify import/dependency statements.
- [ ] Build an in-memory directed graph representing module/component dependencies.
- [ ] Optimize graph construction for large codebases.

### 2.4 Deterministic Anti-pattern Detector
- [ ] Implement algorithms for initial anti-pattern detection based on ASTs and dependency graphs:
    - [ ] Cyclic Dependency (via graph cycle detection).
    - [ ] The Blob/God Object (via AST node count heuristics).
    - [ ] Unstable Interface (via dependency graph fan-in and change frequency heuristics).
    - [ ] Contextual Code Snippet Extraction for detected issues.

### 2.5 Error Handling & Recovery
- [ ] Implement graceful handling of malformed source files
- [ ] Design recovery mechanisms for partial AST parsing failures
- [ ] Implement timeout handling for large files/projects
- [ ] Add detailed error reporting with actionable messages

## Phase 3: AI Model Integration

### 3.1 Local LLM Integration (Ollama)
- [ ] Implement `codeatlas init-local-ai` command to facilitate Ollama setup and model download.
- [ ] Develop Rust bindings/integration for Ollama API.
- [ ] Implement logic to utilize a specified local model (e.g., `mistral:7b-instruct-v0.2-q4_K_M`).
- [ ] Integrate local LLM into the AI Reasoning Engine.

### 3.2 External LLM API Integration
- [ ] Implement clients for OpenAI (GPT-4), Anthropic (Claude 3), and Google (Gemini) LLM APIs.
- [ ] Implement secure handling of API keys (environment variables/secure config).
- [ ] Integrate API-based LLMs into the AI Reasoning Engine.

### [CDAT-7] 3.3 AI Reasoning Engine & Prompt Engineering
- [ ] Implement smart prompting/RAG strategy:
    - [ ] Construct prompts embedding contextual code snippets and structural information from AST analysis.
    - [ ] Implement hallucination mitigation techniques (structured prompting, uncertainty handling).
- [ ] Develop logic for AI-generated explanations, titles, descriptions, and refactoring suggestions for architectural issues.
- [ ] Implement multi-layered hallucination defense strategy:
    - [ ] Implement advanced RAG to ground the LLM in codebase facts
    - [ ] Design structured prompts with explicit format constraints
    - [ ] Implement self-correction loops (critic LLM reviews primary LLM output)
    - [ ] Build clear human-in-the-loop verification workflows

## Phase 4: Reporting & Output

### 4.1 Markdown Report Generation
- [ ] Implement a Report Generator module.
- [ ] Generate comprehensive analysis reports in Markdown format.
- [ ] Categorize issues by anti-pattern type.
- [ ] Include detailed issue entries: name, file path, line numbers, severity, AI-generated title/description, code snippets, refactoring suggestions.

### 4.2 Diagram Integration
- [ ] Implement logic for AI to generate valid Mermaid.js syntax for diagrams (e.g., cyclic dependencies, class relationships).
- [ ] Embed Mermaid.js syntax directly into the Markdown report.

## Phase 5: Extensibility & Plugin System

### 5.1 Plugin System Core
- [ ] Define a stable plugin interface (Rust traits).
- [ ] Implement plugin discovery and loading mechanism (e.g., from a `plugins/` directory).
- [ ] Implement sandboxing for security using WebAssembly (WASM) runtime.
- [ ] Develop data exchange mechanisms between core and plugins.

### 5.2 Example Plugin Development
- [ ] Create a simple example plugin to validate the system.
- [ ] Document the plugin development process.

## Phase 6: Testing & Quality Assurance

### 6.1 Unit Testing
- [ ] Write comprehensive unit tests for all modules (CLI, file ingestion, AST parsing, graph builder, anti-pattern detection, AI integration, reporting).

### 6.2 Integration Testing
- [ ] Develop integration tests for the full analysis pipeline.
- [ ] Test local and API-based AI model interactions.

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
