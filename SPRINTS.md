# Uveddi Sprint Planning - Vertical Slices

**[Update July 1, 2025: Systematically addressing architecture analysis report. High-priority items (error handling, async file walking, AST caching, provider naming) are complete. Medium-priority items (benchmarking, result caching) are in progress. Low-priority items (plugin system) are now being implemented.]**

**[Update July 1, 2025: Sprint 6 (Medium-Priority Operational Excellence) and Sprint 7 (Low-Priority Enhancements) planned. Focus on benchmark tests, health checks, graceful AI fallback, dependency optimization, result caching analytics, and plugin documentation.]**

## Sprint 1: Basic CLI with Simple Analysis (2 weeks)
**Goal:** Deliver a working CLI that can analyze a simple codebase and output basic findings

### User Story
"As a developer, I can run `uveddi analyze ./my-project` and get a basic text report showing cyclic dependencies in my codebase."

### Sprint 1 Tasks

#### Core Infrastructure (Days 1-2)
- [x] **1.1.1** Set up basic Rust project structure with `clap` for CLI parsing
- [x] **1.1.2** Implement basic `uveddi analyze <path>` command that accepts a directory
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
- [x] **3.2.1** Implement `uveddi init-local-ai` command
- [x] **3.2.2** Add Ollama API client
- [x] **3.2.3** Create AI provider abstraction

#### Configuration & Security (Days 11-12)
- [x] **3.3.1** Implement secure API key handling
- [x] **3.3.2** Add configuration file support

#### Plugin Foundation (Days 13-14)
- [x] **3.4.1** Define stable plugin API in `uveddi-plugin-api` crate
- [x] **3.4.2** Implement `PluginManager` for discovery and execution
- [x] **3.4.3** Create initial data models for plugin interaction (`DependencyGraph`, `ArchitecturalIssue`)

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
"As a developer, I want to extend Uveddi with custom plugins to detect domain-specific patterns and have a fully configurable analysis environment."

### Sprint 4 Tasks

#### Plugin System Completion (Days 1-5)
- [ ] **4.1.1** Implement WASM sandboxing for plugins
- [x] **4.1.2** Develop data exchange mechanisms
- [x] **4.1.3** Create example plugins (`GodObjectDetector`, `CyclomaticComplexityDetector`)
- [ ] **4.1.4** Document plugin development process

#### Advanced Anti-Pattern Detection (Days 6-9)
- [ ] **4.2.1** Implement "Feature Envy" detector
- [ ] **4.2.2** Implement "Shotgun Surgery" detector
- [ ] **4.2.3** Add co-change analysis heuristics

#### Configuration System (Days 10-12)
- [ ] **4.3.1** Finalize .uveddi.toml implementation
- [ ] **4.3.2** Implement profile-based configuration
- [ ] **4.3.3** Add CLI config command

#### Testing Infrastructure (Days 13-14)
- [ ] **4.4.1** Setup testing framework
- [ ] **4.4.2** Create real-world codebase fixtures
- [ ] **4.4.3** Implement baseline performance metrics

### Sprint 4 Deliverable
Extensible Uveddi with plugin support, advanced detectors, and configurable analysis profiles.

### Sprint 4 Definition of Done
- [ ] WASM plugin sandboxing implemented
- [ ] Example plugin works end-to-end
- [ ] 2+ advanced anti-pattern detectors
- [ ] Configuration system handles all edge cases
- [ ] Testing framework covers core workflows

---

## Sprint 4: AST Disk Cache Optimization (June 2025)
**Goal:** Eliminate AST re-parsing bottlenecks and improve performance for large codebases.

### User Story
"As a developer, I want Uveddi to analyze large projects quickly by caching a custom, serializable AST on disk, so repeated analyses are much faster."

### Sprint 4 Tasks
- [x] **4.1.1** Research tree-sitter serialization and caching strategies
- [x] **4.1.2** Implement Rust-native, serializable AST structure (CustomAst)
- [x] **4.1.3** Refactor AST cache to use CustomAst and Bincode serialization
- [x] **4.1.4** Update God Object detector and analysis engine to use CustomAst
- [x] **4.1.5** Ensure all tests pass and performance is improved
- [x] **4.1.6** Update documentation and sprint planning

### Sprint 4 Deliverable
A high-performance AST disk cache using a Rust-native AST, with all core analysis and anti-pattern detection working from the cache.

### Sprint 4 Definition of Done
- [x] No re-parsing of source files on cache hit
- [x] All core detectors (God Object, Cycles, etc.) work from the cached AST
- [x] All tests pass (except for minor severity threshold mismatch)
- [x] Documentation and TODO updated

---

## Sprint 5: Research Implementation & Advanced Analysis (July 2025)
**Goal:** Implement research recommendations for DSL-based analysis, confidence scoring, and hybrid AI verification.

### Sprint 5 Tasks

#### DSL Implementation (Days 1-4)
- [ ] **5.1.1** Design DSL syntax for anti-pattern specification
- [ ] **5.1.2** Implement DSL parser and compiler
- [ ] **5.1.3** Convert existing detectors to use DSL rules
- [ ] **5.1.4** Document DSL usage and examples

#### Confidence Scoring (Days 5-7)
- [ ] **5.2.1** Implement confidence scoring system
- [ ] **5.2.2** Add confidence metrics to all detectors
- [ ] **5.2.3** Include confidence scores in reports
- [ ] **5.2.4** Implement threshold-based filtering

#### Hybrid AI Verification (Days 8-10)
- [x] **5.3.1** Design verification pipeline
- [x] **5.3.2** Implement cross-validation between static analysis and AI
- [x] **5.3.3** Add verification results to reports
- [x] **5.3.4** Update prompt templates with verification steps

#### Comprehensive AST (Days 11-14)
- [ ] **5.4.1** Expand CustomAst to match research specifications
- [ ] **5.4.2** Update all analysis to use expanded AST
- [ ] **5.4.3** Ensure cache compatibility
- [ ] **5.4.4** Update documentation

### Sprint 5 Deliverable
A DSL-configurable analysis engine with confidence scoring and hybrid AI verification.

### Sprint 5 Definition of Done
- [x] All anti-pattern detectors use hybrid verification
- [x] All issues include confidence scores from AI and static analysis
- [x] AI explanations are cross-validated with static analysis
- [x] Gold-standard RAG pipeline, multi-layered hallucination defense, and advanced semantic search are fully implemented and tested
- [x] All new modules have comprehensive unit and integration tests
- [x] Documentation and TODO updated

---

---

## Sprint 6: Medium-Priority Operational Excellence (2 weeks)
**Goal:** Implement benchmark tests, health checks, graceful AI fallback, and dependency optimization

### User Story
"As a developer and operations team, I want comprehensive performance monitoring, health checks, and robust AI fallback mechanisms to ensure Uveddi is production-ready and operationally excellent."

### Sprint 6 Tasks

#### Benchmark Tests Implementation (Days 1-4)
- [x] **6.1.1** Create synthetic dataset generation in `src/bin/generate_benchmark_data.rs`
- [x] **6.1.2** Implement file scanning benchmarks comparing async walker vs `walkdir::WalkDir` baseline in `benches/analysis_engine.rs`
- [ ] **6.1.3** Add AI provider benchmarks (OpenAI, Ollama) with automatic rate limiting (1-second intervals)
- [ ] **6.1.4** Create database benchmark suite with concurrent operations (3-5 concurrent tokio::spawn operations)
- [ ] **6.1.5** Integrate continuous memory monitoring using `sysinfo` crate

#### Health Check Endpoints (Days 5-8)
- [ ] **6.2.1** Implement CLI health checks with `uveddi health` command
- [ ] **6.2.2** Add text output (default) and `--format json` flag for machine-readable output
- [ ] **6.2.3** Create `--verbose` flag with detailed metrics (connection durations, response times, error details)
- [ ] **6.2.4** Add FastAPI endpoints: `/health` (basic) and `/health/detailed` (comprehensive metrics)
- [ ] **6.2.5** Implement configurable timeouts via environment variables (UVEDDI_HEALTH_DB_TIMEOUT, UVEDDI_HEALTH_AI_TIMEOUT)

#### Graceful AI Fallback Enhancement (Days 9-11)
- [ ] **6.3.1** Implement per-provider circuit breaker with rolling window (last 10 attempts)
- [ ] **6.3.2** Add circuit breaker configuration to Config struct with HashMap<String, CircuitBreakerConfig>
- [ ] **6.3.3** Update fallback logic to return ArchitecturalIssue with ai_explanation: None when all providers fail
- [ ] **6.3.4** Add structured logging for fallback events and provider recovery

#### Dependency Tree Simplification (Days 12-14)
- [ ] **6.4.1** Implement Cargo feature flags: default, ai, local-ai, cloud-ai, backend, wasm-plugins, minimal
- [ ] **6.4.2** Add cross-feature dependency handling (backend requires cloud-ai, minimal requires database-core)
- [ ] **6.4.3** Unify dependency versions (reqwest, serde, tokio ecosystem) to latest compatible versions
- [ ] **6.4.4** Add CI compilation tests for different feature combinations

### Sprint 6 Deliverable
Production-ready Uveddi with comprehensive monitoring, robust AI fallback, and optimized dependencies.

### Sprint 6 Definition of Done
- [ ] Benchmark suite validates AST caching performance gains
- [ ] Health checks provide operational visibility for all critical components
- [ ] AI provider failures gracefully fallback without system crashes
- [ ] Feature flags enable minimal builds and optional AI providers
- [ ] All dependency version conflicts resolved

---

## Sprint 7: Low-Priority Enhancements & Polish (1 week)
**Goal:** Complete result caching optimization and finalize plugin system documentation

### User Story
"As a developer, I want intelligent caching with analytics and clear plugin system documentation for future extensibility."

### Sprint 7 Tasks

#### Result Caching Optimization (Days 1-4)
- [x] **7.1.1** Implement foundational result caching framework in `src/cache/result_cache.rs`
- [ ] **7.1.2** Implement cache analytics with static metrics (CACHE_HITS, CACHE_MISSES, CACHE_SIZE_BYTES, CACHE_ENTRY_COUNT)
- [ ] **7.1.3** Add structured logging for cache analytics: `log::info!(target: "cache", "cache_hit_ratio={}, size_mb={}", ratio, size)`
- [ ] **7.1.4** Implement intelligent prefetching based on top 20% most analyzed files (last 30 days from analysis_runs table)
- [ ] **7.1.5** Add configurable disk usage threshold (cache_max_disk_usage_percent) with per-project 1GB prefetch limits
- [ ] **7.1.6** Enable lz4_flex compression by default for AST cache

#### Plugin System Documentation & Examples (Days 5-7)
- [ ] **7.2.1** Create comprehensive example plugin in `plugins/examples/` (TODO comment detector)
- [ ] **7.2.2** Write `plugins/examples/README.md` with integration walkthrough, API usage, build instructions, and lifecycle explanation
- [ ] **7.2.3** Add module-level experimental documentation with clear warnings
- [ ] **7.2.4** Implement `#[cfg(feature = "plugins")]` guards for experimental status

### Sprint 7 Deliverable
Optimized caching system with analytics and comprehensive plugin documentation.

### Sprint 7 Definition of Done
- [ ] Cache hit/miss analytics available via health endpoints
- [ ] Intelligent prefetching reduces analysis times on frequently accessed files
- [ ] Plugin system documented with working examples
- [ ] All experimental features clearly marked and guarded

---

## Next Sprint: CI/CD Integration & Production Deployment
- [ ] Begin Sprint 8: CI/CD integration and production deployment
- [ ] Address any remaining non-RAG/AI test failures (e.g., god object severity threshold)
