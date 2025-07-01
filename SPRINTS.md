# Uveddi Sprint Planning - Vertical Slices

**🚨 CRITICAL STATUS [Update July 1, 2025]: CODEBASE HAS 19+ COMPILATION ERRORS PREVENTING CLI EXECUTION 🚨**

**[Update July 1, 2025: Current state assessment shows foundational architecture is in place but many Sprint features are incomplete or non-functional. Sprint 6/7 operational features are planned but not implemented. Sprint 5 AI features require substantial completion work.]**

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
- [~] **2.3.2** Implement simple OpenAI API client (exists but AiAnalysisEngine is mostly empty stubs)
- [~] **2.3.3** Generate AI explanations for detected issues (framework exists but core implementation incomplete)
- [~] **2.3.4** Create basic prompt templates for issue description (partial implementation)

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
- [~] Detects cyclic dependencies and god objects (has compilation errors)
- [~] Generates AI explanations for issues (framework exists but core engine incomplete)
- [~] Outputs both JSON and Markdown formats (has compilation errors)
- [~] Handles missing API keys gracefully (partial implementation)

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
- [~] **3.4.2** Implement `PluginManager` for discovery and execution (has compilation errors)
- [x] **3.4.3** Create initial data models for plugin interaction (`DependencyGraph`, `ArchitecturalIssue`)

### Sprint 3 Deliverable
ERD-compliant architectural analysis tool with local AI support and critical anti-pattern coverage.

### Sprint 3 Definition of Done
- [~] Generates Mermaid.js diagrams in reports (framework exists but has compilation errors)
- [~] Detects all core anti-patterns (Cycles, God Objects, Unstable Interfaces) (detectors exist but have compilation errors)
- [~] Includes AI hallucination mitigation (framework exists but core engine incomplete)
- [~] Works offline with local AI models (Ollama integration exists but has compilation errors)
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
- [~] **4.1.1** Implement WASM sandboxing for plugins (foundation exists but has multiple compilation errors)
- [~] **4.1.2** Develop data exchange mechanisms (basic structs defined)
- [~] **4.1.3** Create example plugins (`GodObjectDetector`, `CyclomaticComplexityDetector`) (stubs exist but have compilation errors)
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

### Sprint 4 Definition of Done - **BLOCKED BY COMPILATION ERRORS**
- [ ] **BLOCKED**: WASM plugin sandboxing implemented (has compilation errors)
- [ ] **BLOCKED**: Example plugin works end-to-end (has compilation errors)
- [ ] **BLOCKED**: 2+ advanced anti-pattern detectors (basic detectors have compilation errors)
- [ ] **BLOCKED**: Configuration system handles all edge cases (has compilation errors)
- [ ] **BLOCKED**: Testing framework covers core workflows (tests may not pass due to compilation errors)

---

## Sprint 4: AST Disk Cache Optimization (June 2025)
**Goal:** Eliminate AST re-parsing bottlenecks and improve performance for large codebases.

### User Story
"As a developer, I want Uveddi to analyze large projects quickly by caching parsed ASTs on disk, so repeated analyses are much faster."

### Sprint 4 Tasks
- [x] **4.1.1** Research tree-sitter serialization and caching strategies
- [ ] **4.1.2** Implement Rust-native, serializable AST structure (CustomAst)
- [~] **4.1.3** Refactor AST cache to use Bincode serialization (currently serializes `tree_sitter::Tree`)
- [ ] **4.1.4** Update God Object detector and analysis engine to use CustomAst
- [ ] **4.1.5** Ensure all tests pass and performance is improved
- [ ] **4.1.6** Update documentation and sprint planning

### Sprint 4 Deliverable
A high-performance AST disk cache using a Rust-native AST, with all core analysis and anti-pattern detection working from the cache.

### Sprint 4 Definition of Done - **PARTIALLY COMPLETE BUT BLOCKED**
- [~] No re-parsing of source files on cache hit (cache exists but has compilation errors)
- [ ] **BLOCKED**: All core detectors (God Object, Cycles, etc.) work from the cached AST (has compilation errors)
- [ ] **BLOCKED**: All tests pass (compilation errors prevent test execution)
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
- [x] **5.3.1** Design verification pipeline (file created)
- [ ] **5.3.2** Implement cross-validation between static analysis and AI (stub exists)
- [ ] **5.3.3** Add verification results to reports
- [ ] **5.3.4** Update prompt templates with verification steps

#### Comprehensive AST (Days 11-14)
- [ ] **5.4.1** Expand CustomAst to match research specifications
- [ ] **5.4.2** Update all analysis to use expanded AST
- [ ] **5.4.3** Ensure cache compatibility
- [ ] **5.4.4** Update documentation

### Sprint 5 Deliverable
A DSL-configurable analysis engine with confidence scoring and hybrid AI verification.

### Sprint 5 Definition of Done - **SIGNIFICANTLY INCOMPLETE**
- [ ] **BLOCKED**: All anti-pattern detectors use hybrid verification (basic detectors have compilation errors)
- [ ] **BLOCKED**: All issues include confidence scores from AI and static analysis (AI engine incomplete)
- [ ] **BLOCKED**: AI explanations are cross-validated with static analysis (AI engine incomplete)
- [~] Gold-standard RAG pipeline, multi-layered hallucination defense, and advanced semantic search are partially implemented (framework exists but core AI engine incomplete)
- [~] All new modules have comprehensive unit and integration tests (tests exist but may not pass due to compilation errors)
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
- [ ] **PLANNED** Add AI provider benchmarks (OpenAI, Ollama) with automatic rate limiting (1-second intervals)
- [ ] **PLANNED** Create database benchmark suite with concurrent operations (3-5 concurrent tokio::spawn operations)
- [ ] **PLANNED** Integrate continuous memory monitoring using `sysinfo` crate

#### Health Check Endpoints (Days 5-8)
- [ ] **PLANNED** Implement CLI health checks with `uveddi health` command
- [ ] **PLANNED** Add text output (default) and `--format json` flag for machine-readable output
- [ ] **PLANNED** Create `--verbose` flag with detailed metrics (connection durations, response times, error details)
- [ ] **PLANNED** Add FastAPI endpoints: `/health` (basic) and `/health/detailed` (comprehensive metrics)
- [ ] **PLANNED** Implement configurable timeouts via environment variables (UVEDDI_HEALTH_DB_TIMEOUT, UVEDDI_HEALTH_AI_TIMEOUT)

#### Graceful AI Fallback Enhancement (Days 9-11)
- [ ] **PLANNED** Implement per-provider circuit breaker with rolling window (last 10 attempts)
- [ ] **PLANNED** Add circuit breaker configuration to Config struct with HashMap<String, CircuitBreakerConfig>
- [ ] **PLANNED** Update fallback logic to return ArchitecturalIssue with ai_explanation: None when all providers fail
- [ ] **PLANNED** Add structured logging for fallback events and provider recovery

#### Dependency Tree Simplification (Days 12-14)
- [ ] **PLANNED** Implement Cargo feature flags: default, ai, local-ai, cloud-ai, backend, wasm-plugins, minimal
- [ ] **PLANNED** Add cross-feature dependency handling (backend requires cloud-ai, minimal requires database-core)
- [ ] **PLANNED** Unify dependency versions (reqwest, serde, tokio ecosystem) to latest compatible versions
- [ ] **PLANNED** Add CI compilation tests for different feature combinations

### Sprint 6 Deliverable
Production-ready Uveddi with comprehensive monitoring, robust AI fallback, and optimized dependencies.

### Sprint 6 Definition of Done - **NOT STARTED**
- [ ] **PLANNED** Benchmark suite validates AST caching performance gains
- [ ] **PLANNED** Health checks provide operational visibility for all critical components
- [ ] **PLANNED** AI provider failures gracefully fallback without system crashes
- [ ] **PLANNED** Feature flags enable minimal builds and optional AI providers
- [ ] **PLANNED** All dependency version conflicts resolved

---

## Sprint 7: Low-Priority Enhancements & Polish (1 week)
**Goal:** Complete result caching optimization and finalize plugin system documentation

### User Story
"As a developer, I want intelligent caching with analytics and clear plugin system documentation for future extensibility."

### Sprint 7 Tasks

#### Result Caching Optimization (Days 1-4)
- [x] **7.1.1** Implement foundational result caching framework in `src/cache/result_cache.rs`
- [ ] **PLANNED** Implement cache analytics with static metrics (CACHE_HITS, CACHE_MISSES, CACHE_SIZE_BYTES, CACHE_ENTRY_COUNT)
- [ ] **PLANNED** Add structured logging for cache analytics: `log::info!(target: "cache", "cache_hit_ratio={}, size_mb={}", ratio, size)`
- [ ] **PLANNED** Implement intelligent prefetching based on top 20% most analyzed files (last 30 days from analysis_runs table)
- [ ] **PLANNED** Add configurable disk usage threshold (cache_max_disk_usage_percent) with per-project 1GB prefetch limits
- [ ] **PLANNED** Enable lz4_flex compression by default for AST cache

#### Plugin System Documentation & Examples (Days 5-7)
- [ ] **PLANNED** Create comprehensive example plugin in `plugins/examples/` (TODO comment detector)
- [ ] **PLANNED** Write `plugins/examples/README.md` with integration walkthrough, API usage, build instructions, and lifecycle explanation
- [ ] **PLANNED** Add module-level experimental documentation with clear warnings
- [ ] **PLANNED** Implement `#[cfg(feature = "plugins")]` guards for experimental status

### Sprint 7 Deliverable
Optimized caching system with analytics and comprehensive plugin documentation.

### Sprint 7 Definition of Done - **NOT STARTED**
- [ ] **PLANNED** Cache hit/miss analytics available via health endpoints
- [ ] **PLANNED** Intelligent prefetching reduces analysis times on frequently accessed files
- [ ] **PLANNED** Plugin system documented with working examples
- [ ] **PLANNED** All experimental features clearly marked and guarded

---

## Immediate Priority Sprint 0: Critical Bug Fixes
- [ ] **CRITICAL**: Fix 19+ compilation errors preventing CLI execution
- [ ] **CRITICAL**: Complete AiAnalysisEngine implementation (analyze_issue method)
- [ ] **CRITICAL**: Fix WASM plugin manager compilation errors
- [ ] **CRITICAL**: Fix database CRUD operation borrowing/mutability errors
- [ ] **CRITICAL**: Fix AST cache type errors and integration issues
- [ ] **HIGH**: Ensure basic CLI commands (analyze, config, init-local-ai) function properly
- [ ] **HIGH**: Validate that core anti-pattern detectors work without compilation errors

## Next Sprint: CI/CD Integration & Production Deployment
- [ ] **BLOCKED**: Begin Sprint 8: CI/CD integration and production deployment (requires Sprint 0 completion)
- [ ] **BLOCKED**: Address any remaining test failures (requires functional codebase)
