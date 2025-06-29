# CodeAtlas Sprint Planning - Vertical Slices

**[Update June 28, 2025: Sprint 2 is 100% complete and tested. All tasks and deliverables are done. See TODO.md for next steps.]**

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

---

## Sprint 2: Multi-language Support + AST Analysis (2 weeks)
**Goal:** Extend analysis to multiple languages using proper AST parsing and add one AI-powered feature

### User Story
"As a developer, I can analyze Python and JavaScript projects, get more detailed analysis using ASTs, and see AI-generated explanations for detected issues."

### Sprint 2 Tasks

#### AST Integration (Days 1-4)
- [x] **2.1.1** Integrate tree-sitter for AST parsing
- [x] **2.1.2** Add support for Python and JavaScript/TypeScript parsers
- [x] **2.1.3** Refactor dependency extraction to use AST instead of regex
- [x] **2.1.4** Implement basic AST caching for performance

#### Enhanced Analysis (Days 5-7)
- [x] **2.2.1** Add "God Object" detection using AST node count heuristics
- [x] **2.2.2** Extract code snippets for detected issues
- [x] **2.2.3** Implement severity scoring for issues

#### AI Integration - Phase 1 (Days 8-10)
- [x] **2.3.1** Add basic configuration system for API keys
- [x] **2.3.2** Implement simple OpenAI API client
- [x] **2.3.3** Generate AI explanations for detected issues
- [x] **2.3.4** Create basic prompt templates for issue description

#### Enhanced Reporting (Days 11-14)
- [x] **2.4.1** Upgrade to structured JSON output option
- [x] **2.4.2** Add markdown report generation
- [x] **2.4.3** Include AI-generated explanations in reports
- [x] **2.4.4** Add summary statistics (total issues, severity breakdown)

### Sprint 2 Deliverable
A multi-language analyzer with AST-based detection and AI-powered explanations.

### Sprint 2 Definition of Done
- [x] Supports Rust, Python, and JavaScript analysis
- [x] Uses AST parsing for accurate analysis
- [x] Detects cyclic dependencies and god objects
- [x] Generates AI explanations for issues (when API key provided)
- [x] Outputs both JSON and Markdown formats
- [x] Handles missing API keys gracefully

---

## Sprint 3: ERD Compliance + Local AI (2 weeks)
**Goal:** Address critical architectural drift and add local AI support

### User Story
"As a developer, I can get complete architectural analysis with diagrams, all core anti-pattern detection, and safe AI explanations - either through cloud APIs or local models."

### Sprint 3 Tasks

#### Critical ERD Compliance (Days 1-7)
- [x] **3.1.1** Implement Mermaid.js diagram rendering
- [x] **3.1.2** Complete Unstable Interface detector
- [x] **3.1.3** Complete Modularity Violation detector
- [x] **3.1.4** Implement hallucination mitigation prompts

#### Local AI Integration (Days 8-10)
- [x] **3.2.1** Implement `codeatlas init-local-ai` command
- [x] **3.2.2** Add Ollama API client
- [x] **3.2.3** Create AI provider abstraction

#### Configuration & Security (Days 11-12)
- [x] **3.3.1** Implement secure API key handling
- [x] **3.3.2** Add configuration file support

#### Plugin Foundation (Days 13-14)
- [x] **3.4.1** Define basic plugin trait interface
- [x] **3.4.2** Implement plugin discovery

### Sprint 3 Deliverable
ERD-compliant architectural analysis tool with local AI support and critical anti-pattern coverage.

### Sprint 3 Definition of Done
- [x] Generates Mermaid.js diagrams in reports
- [x] Detects all core anti-patterns (Cycles, God Objects, Unstable Interfaces)
- [x] Includes AI hallucination mitigation
- [x] Works offline with local AI models
- [x] Secure API key configuration

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
- 100% of reports include architecture diagrams
- All core anti-patterns detected in test projects
- <5% hallucination rate in AI explanations
- Local AI setup completes in under 5 minutes

### Sprint 4 Success Criteria
- Plugin system handles WASM modules securely
- Example plugin detects issues in test projects
- Advanced detectors achieve 90%+ accuracy
- Configuration changes apply without restart

## Sprint 4: Plugin Ecosystem & Advanced Anti-Patterns (2 weeks)
**Goal:** Complete plugin system and implement advanced architectural analysis capabilities

### User Story
"As a developer, I want to extend CodeAtlas with custom plugins to detect domain-specific patterns and have a fully configurable analysis environment."

### Sprint 4 Tasks

#### Plugin System Completion (Days 1-5)
- [ ] **4.1.1** Implement WASM sandboxing for plugins
- [ ] **4.1.2** Develop data exchange mechanisms
- [ ] **4.1.3** Create example plugin: custom file extensions detector
- [ ] **4.1.4** Document plugin development process

#### Advanced Anti-Pattern Detection (Days 6-9)
- [ ] **4.2.1** Implement "Feature Envy" detector
- [ ] **4.2.2** Implement "Shotgun Surgery" detector
- [ ] **4.2.3** Add co-change analysis heuristics

#### Configuration System (Days 10-12)
- [ ] **4.3.1** Finalize .codeatlas.toml implementation
- [ ] **4.3.2** Implement profile-based configuration
- [ ] **4.3.3** Add CLI config command

#### Testing Infrastructure (Days 13-14)
- [ ] **4.4.1** Setup testing framework
- [ ] **4.4.2** Create real-world codebase fixtures
- [ ] **4.4.3** Implement baseline performance metrics

### Sprint 4 Deliverable
Extensible CodeAtlas with plugin support, advanced detectors, and configurable analysis profiles.

### Sprint 4 Definition of Done
- [ ] WASM plugin sandboxing implemented
- [ ] Example plugin works end-to-end
- [ ] 2+ advanced anti-pattern detectors
- [ ] Configuration system handles all edge cases
- [ ] Testing framework covers core workflows

---

## Post-Sprint 4 Roadmap
After these four sprints, the next priorities would be:
1. **Sprint 5:** CI/CD integration and production deployment
2. **Sprint 6:** ERD compliance audit and optimization
3. **Sprint 7:** Monetization implementation
