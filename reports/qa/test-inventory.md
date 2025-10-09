# Uveddi 1.0.0 Test Inventory
**Date:** 2025-11-22
**Branch:** release/1.0.0
**QA Lead:** Solo Dev

## Test Suite Summary

### Automated Test Statistics
- **Total Test Files:** 173
- **Explicit Test Targets:** 9
- **Test Categories:** 8
- **Test Scripts:** 60+

## Test Categories

### 1. Unit Tests (`--lib`)
**Location:** `src/**/*.rs` (doctests and inline tests)
**Invocation:** `cargo test --lib --features cli-standard`
**Coverage:** Core functionality, utility functions, data structures

### 2. Integration Tests (`tests/`)
**Location:** `tests/integration/`
**Key Files:**
- `database_operations.rs` - Database CRUD operations
- `websocket_reliability.rs` - WebSocket functionality
**Invocation:** `cargo test --test integration`

### 3. CLI Integration Tests
**Location:** `tests/cli/`
**Key Files:**
- `cli_integration.rs` - General CLI workflows
- `migrate_command.rs` - Database migration CLI
- `init_local_ai_command.rs` - AI initialization
**Invocation:** `cargo test --test cli_`

### 4. Analysis Engine Tests
**Location:** `tests/analysis/`
**Subdirectories:**
- `universal/` - Language-agnostic detectors
- `advanced/` - Advanced detection algorithms
- `framework/` - Testing framework utilities
**Key Detectors Tested:**
- God Object Detection
- Cyclic Dependencies
- Long Methods
- Magic Values
- Dead Code
- Code Duplication
- Tight Coupling
- Semantic Clones
- Large Classes
- Leaky Abstractions
**Invocation:** `cargo test --test analysis`

### 5. E2E Tests
**Location:** `tests/e2e/`
**Key Files:**
- `complete_analysis_workflow.rs` - Full analysis pipeline
**Invocation:** `cargo test --test e2e`

### 6. Performance Tests
**Location:** `tests/performance/`
**Key Files:**
- `load_testing.rs` - Load and stress tests
- `statistical_analysis.rs` - Performance metrics
- `trend_detection.rs` - Performance regression detection
- `integration.rs` - Performance integration tests
**Invocation:** `cargo test --test performance`

### 7. Security Tests
**Location:** `tests/security/`
**Key Files:**
- `vulnerability_testing.rs` - Security vulnerability checks
- `comprehensive_security.rs` - Security validation suite
- `secret_management_tests.rs` - Secret handling
- `path_traversal_tests.rs` - Path security
- `deserialization_security_tests.rs` - Safe deserialization
- `fuzz_input_validation.rs` - Fuzzing tests
- `compliance_validation.rs` - Compliance checks
- `cryptographic_operations_test.rs` - Crypto ops
- `authentication_integration_test.rs` - Auth testing
**Invocation:** `cargo test --test security`

### 8. Coverage Tests
**Location:** `tests/coverage/`
**Key Files:**
- `comprehensive_coverage.rs` - Code coverage suite
- `edge_case_testing.rs` - Edge case scenarios
- `regression_prevention.rs` - Regression tests
**Invocation:** `cargo test --test coverage`

## Test Execution Scripts

### Primary QA Scripts
1. **`scripts/run_qa_tests.sh`**
   - Purpose: Comprehensive detector QA validation
   - Outputs: `target/qa-reports/`
   - Generates: Accuracy, regression, and performance reports

2. **`scripts/pre_release_test.sh`**
   - Purpose: Pre-release validation checklist
   - Checks: Format, clippy, unit tests, integration tests, benchmarks
   - Tests: Release binary with real analysis

3. **`scripts/test-v1-release.sh`**
   - Purpose: v1.0 release validation
   - Components: Library, binaries, frontend, API server
   - Outputs: Multiple format reports (JSON, HTML, Markdown)

### Supporting Scripts
- `scripts/coverage-report.sh` - Generate coverage reports
- `scripts/test-all-cli-commands.sh` - CLI command validation
- `scripts/verify_detector_coverage.sh` - Detector coverage check
- `scripts/test_real_codebases.sh` - Real-world testing
- `scripts/validate-security-fixes.sh` - Security validation
- `scripts/test-resource-management.sh` - Resource usage tests

## Feature-Specific Test Suites

### Database Tests
- `database_crud.rs` - CRUD operations
- `sqlite_db.rs` - SQLite integration
- `integration/database_operations.rs` - DB integration

### AI/ML Tests
- `ai/provider_abstraction.rs` - Provider abstraction
- `ai/ollama_provider.rs` - Ollama integration
- `ai/ollama_integration.rs` - Ollama tests
- `ai/hallucination_mitigation.rs` - AI safety
- `ai/full_pipeline_integration.rs` - End-to-end AI
- `ai_explanations.rs` - AI-generated explanations
- `report_ai_explanation.rs` - AI report generation

### Configuration Tests
- `config/config_file.rs` - Configuration handling
- `config/secure_api_keys.rs` - API key security

### Memory Optimization Tests
- `memory_optimization_phase1.rs`
- `memory_optimization_phase2.rs`
- `memory_optimization_phase3.rs`
- `memory/arena_allocation_stress_test.rs`

### Plugin System Tests
- `plugin_system_comprehensive.rs` - Plugin infrastructure
- `tests/plugins/` - Plugin examples

## Test Invocation Reference

### Run All Tests
```bash
cargo test --all-features
```

### Run by Category
```bash
# Unit tests only
cargo test --lib --features cli-standard

# Integration tests
cargo test --tests --features cli-standard

# Specific test file
cargo test --test database_crud

# With output
cargo test -- --nocapture

# Single-threaded (for debugging)
cargo test -- --test-threads=1
```

### Run QA Suites
```bash
# Full QA validation
./scripts/run_qa_tests.sh

# Pre-release checklist
./scripts/pre_release_test.sh

# v1.0 release tests
./scripts/test-v1-release.sh
```

### Generate Coverage
```bash
# Coverage report
./scripts/coverage-report.sh

# With tarpaulin (if installed)
cargo tarpaulin --out Html --output-dir reports/qa/coverage
```

### Run Benchmarks
```bash
# All benchmarks
cargo bench

# Specific benchmark
cargo bench --bench <benchmark_name>
```

## Test Data & Fixtures

### Fixture Locations
- `tests/fixtures/` - Test code samples
- `tests/fixtures/*.rs` - Rust fixtures
- `tests/fixtures/*.py` - Python fixtures
- `tests/fixtures/*.js` - JavaScript fixtures
- `tests/fixtures/*.ts` - TypeScript fixtures

### Test Data
- `target/benchmark-data/` - Benchmark test projects
- `tests/test_utils/fixtures.rs` - Fixture utilities
- `tests/test_utils/validation_test.rs` - Validation helpers

## Test Requirements

### Required Features for Full Testing
- `cli-standard` - Standard CLI features (RECOMMENDED)
- `cli-full` - All CLI features including AI and plugins
- `tree-sitter` - Language parsing
- `security` - Security features
- `ai` - AI integration (requires Ollama)

### External Dependencies
- **Ollama** (optional): For AI-related tests
  - Check: `curl http://localhost:11434/api/tags`
  - Model: `deepseek-coder:6.7b-instruct-q4_0`
- **cargo-audit** (optional): For security audits
- **cargo-tarpaulin** (optional): For coverage generation

## Test Execution Environment

### Build Profiles
- `dev` - Development (fastest compile)
- `dev-fast` - Fast iteration
- `release` - Production optimization
- `test-fast` - Optimized for CI/CD

### Recommended Test Sequence
1. Format check: `cargo fmt -- --check`
2. Lint: `cargo clippy -- -D warnings`
3. Unit tests: `cargo test --lib`
4. Integration tests: `cargo test --tests`
5. E2E tests: `cargo test --test e2e`
6. Benchmarks: `cargo bench`
7. Coverage: `./scripts/coverage-report.sh`
8. Release validation: `./scripts/test-v1-release.sh`

## Known Test Limitations
- AI tests require Ollama to be running
- Some integration tests may be skipped if dependencies unavailable
- Performance tests may vary based on hardware
- Coverage tools (tarpaulin) must be installed separately

## Test Artifacts & Outputs
- **Logs:** `reports/qa/logs/`
- **Coverage:** `reports/qa/coverage/`
- **QA Reports:** `target/qa-reports/`
- **Test Results:** Console output and XML/JSON (when configured)
