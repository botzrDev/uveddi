# CodeAtlas Sprint Planning - Vertical Slices

This document outlines the first three sprints for CodeAtlas development, focusing on vertical slices that deliver end-to-end functionality rather than just building infrastructure.

## Sprint 1: Basic CLI with Simple Analysis (2 weeks)
**Goal:** Deliver a working CLI that can analyze a simple codebase and output basic findings

### User Story
"As a developer, I can run `codeatlas analyze ./my-project` and get a basic text report showing cyclic dependencies in my codebase."

### Sprint 1 Tasks

#### Core Infrastructure (Days 1-2)
- [x] **1.1.1** Set up basic Rust project structure with `clap` for CLI parsing
- [x] **1.1.2** Implement basic `codeatlas analyze <path>` command that accepts a directory
- [x] **1.1.3** Add basic error handling and logging with `env_logger`

#### File Processing Pipeline (Days 3-4)
- [x] **1.2.1** Implement recursive file scanning for a single language (Rust)
- [x] **1.2.2** Create basic file filtering (ignore common non-source files)
- [x] **1.2.3** Implement simple dependency extraction from Rust `use` statements

#### Simple Analysis Engine (Days 5-7)
- [x] **1.3.1** Build in-memory dependency graph from extracted imports
- [x] **1.3.2** Implement basic cycle detection algorithm
- [x] **1.3.3** Create simple data structures for analysis results

#### Basic Reporting (Days 8-10)
- [x] **1.4.1** Generate plain text output showing found cyclic dependencies
- [x] **1.4.2** Include file paths and line numbers in output
- [x] **1.4.3** Add basic CLI help and usage information

### Sprint 1 Deliverable
A CLI tool that can analyze Rust projects and detect/report cyclic dependencies in plain text format.

### Sprint 1 Definition of Done
- [ ] CLI runs without crashes on sample Rust project
- [ ] Detects at least one type of architectural issue (cycles)
- [ ] Outputs human-readable results
- [ ] Has basic error handling for invalid paths
- [ ] Includes unit tests for core analysis logic

---

## Sprint 2: Multi-language Support + AST Analysis (2 weeks)
**Goal:** Extend analysis to multiple languages using proper AST parsing and add one AI-powered feature

### User Story
"As a developer, I can analyze Python and JavaScript projects, get more detailed analysis using ASTs, and see AI-generated explanations for detected issues."

### Sprint 2 Tasks

#### AST Integration (Days 1-4)
- [ ] **2.1.1** Integrate tree-sitter for AST parsing
- [ ] **2.1.2** Add support for Python and JavaScript/TypeScript parsers
- [ ] **2.1.3** Refactor dependency extraction to use AST instead of regex
- [ ] **2.1.4** Implement basic AST caching for performance

#### Enhanced Analysis (Days 5-7)
- [ ] **2.2.1** Add "God Object" detection using AST node count heuristics
- [ ] **2.2.2** Extract code snippets for detected issues
- [ ] **2.2.3** Implement severity scoring for issues

#### AI Integration - Phase 1 (Days 8-10)
- [ ] **2.3.1** Add basic configuration system for API keys
- [ ] **2.3.2** Implement simple OpenAI API client
- [ ] **2.3.3** Generate AI explanations for detected issues
- [ ] **2.3.4** Create basic prompt templates for issue description

#### Enhanced Reporting (Days 11-14)
- [ ] **2.4.1** Upgrade to structured JSON output option
- [ ] **2.4.2** Add markdown report generation
- [ ] **2.4.3** Include AI-generated explanations in reports
- [ ] **2.4.4** Add summary statistics (total issues, severity breakdown)

### Sprint 2 Deliverable
A multi-language analyzer with AST-based detection and AI-powered explanations.

### Sprint 2 Definition of Done
- [ ] Supports Rust, Python, and JavaScript analysis
- [ ] Uses AST parsing for accurate analysis
- [ ] Detects cyclic dependencies and god objects
- [ ] Generates AI explanations for issues (when API key provided)
- [ ] Outputs both JSON and Markdown formats
- [ ] Handles missing API keys gracefully

---

## Sprint 3: Local AI + Plugin Foundation (2 weeks)
**Goal:** Add local AI support and create a basic plugin system foundation

### User Story
"As a developer, I can use CodeAtlas without external API dependencies by running local AI models, and I can extend functionality with simple plugins."

### Sprint 3 Tasks

#### Local AI Integration (Days 1-5)
- [ ] **3.1.1** Implement `codeatlas init-local-ai` command
- [ ] **3.1.2** Add Ollama integration and API client
- [ ] **3.1.3** Create AI provider abstraction (local vs. API)
- [ ] **3.1.4** Implement fallback logic (API -> Local -> None)
- [ ] **3.1.5** Add model configuration options

#### Plugin System Foundation (Days 6-9)
- [ ] **3.2.1** Define basic plugin trait interface
- [ ] **3.2.2** Implement plugin discovery from `plugins/` directory
- [ ] **3.2.3** Create example plugin: custom file extensions detector
- [ ] **3.2.4** Add plugin registration and execution pipeline

#### Enhanced Analysis Pipeline (Days 10-12)
- [ ] **3.3.1** Refactor analysis engine to support plugin hooks
- [ ] **3.3.2** Add configuration file support (.codeatlas.toml)
- [ ] **3.3.3** Implement basic `.archlintignore` file parsing
- [ ] **3.3.4** Add analysis scope configuration

#### Quality & Documentation (Days 13-14)
- [ ] **3.4.1** Add comprehensive integration tests
- [ ] **3.4.2** Create plugin development documentation
- [ ] **3.4.3** Add CLI help improvements and examples
- [ ] **3.4.4** Performance optimization for large codebases

### Sprint 3 Deliverable
A fully functional architectural analysis tool with local AI and extensibility.

### Sprint 3 Definition of Done
- [ ] Works entirely offline with local AI models
- [ ] Has basic plugin system with working example
- [ ] Supports configuration files and ignore patterns
- [ ] Handles large codebases efficiently
- [ ] Includes comprehensive documentation
- [ ] Has integration tests covering main workflows

---

## Sprint Success Metrics

### Sprint 1 Success Criteria
- Can analyze a 100-file Rust project in under 30 seconds
- Detects cyclic dependencies with 95% accuracy on test projects
- Zero crashes on malformed input files

### Sprint 2 Success Criteria
- Supports 3 languages with consistent API
- AI explanations are relevant and helpful (manual review)
- JSON output validates against defined schema

### Sprint 3 Success Criteria
- Local AI setup completes in under 10 minutes
- Plugin system supports custom detectors
- Configuration system handles edge cases gracefully

## Post-Sprint 3 Roadmap
After these three sprints, the next priorities would be:
1. **Sprint 4:** Advanced anti-patterns (Unstable Interface, more complex detectors)
2. **Sprint 5:** Diagram generation (Mermaid.js integration)
3. **Sprint 6:** CI/CD integration and production deployment features
