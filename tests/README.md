# Uveddi Test Suite - v1.0.0 Release

This document provides comprehensive guidance for running and maintaining the Uveddi test suite. For detailed test inventory, see `reports/qa/test-inventory.md`.

## Quick Start

```bash
# Run all tests with standard features
cargo test --features cli-standard

# Run all tests with all features
cargo test --all-features

# Run tests with output
cargo test --features cli-standard -- --nocapture

# Run specific test category
cargo test --test <test_name>

# Run pre-release validation
./scripts/pre_release_test.sh

# Run comprehensive QA suite
./scripts/run_qa_tests.sh
```

## Test Categories

### 1. Unit Tests
**Location:** Inline in `src/**/*.rs` and doctests
**Command:** `cargo test --lib --features cli-standard`
**Purpose:** Test individual functions and modules in isolation

### 2. Integration Tests
**Location:** `tests/integration/`
**Command:** `cargo test --test integration`
**Coverage:**
- `database_operations.rs` - Database CRUD and migrations
- `websocket_reliability.rs` - WebSocket functionality

### 3. CLI Tests
**Location:** `tests/cli/`
**Command:** `cargo test --test cli_`
**Coverage:**
- Command-line interface workflows
- Migration commands
- AI initialization
- Output formats

### 4. Analysis Engine Tests
**Location:** `tests/analysis/`
**Command:** `cargo test --test analysis`
**Detectors Tested:**
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

### 5. End-to-End Tests
**Location:** `tests/e2e/`
**Command:** `cargo test --test e2e`
**Coverage:** Complete analysis workflows from init to report generation

### 6. Performance Tests
**Location:** `tests/performance/`
**Command:** `cargo test --test performance`
**Coverage:** Load testing, statistical analysis, trend detection

### 7. Security Tests
**Location:** `tests/security/`
**Command:** `cargo test --test security`
**Coverage:** Vulnerability testing, secret management, path security, compliance

### 8. Coverage Tests
**Location:** `tests/coverage/`
**Command:** `cargo test --test coverage`
**Coverage:** Edge cases, regression prevention, comprehensive coverage

## Test Invocation Reference

### By Scope
```bash
# All tests
cargo test --all-features

# Library only (fast)
cargo test --lib --features cli-standard

# Integration only
cargo test --tests --features cli-standard

# Documentation tests
cargo test --doc --features cli-standard

# Specific test file
cargo test --test database_crud
```

### By Feature Set
```bash
# Minimal CLI
cargo test --features cli-core

# Standard CLI (recommended for most testing)
cargo test --features cli-standard

# CLI with AI (requires Ollama)
cargo test --features cli-ai

# Full feature set
cargo test --features cli-full
```

### With Options
```bash
# Show test output
cargo test -- --nocapture

# Single-threaded (for debugging)
cargo test -- --test-threads=1

# Run specific test
cargo test test_name

# Run tests matching pattern
cargo test pattern

# Show ignored tests
cargo test -- --ignored

# Run both normal and ignored
cargo test -- --include-ignored
```

## Automated Test Scripts

### Primary Scripts

#### Pre-Release Testing
```bash
./scripts/pre_release_test.sh
```
**Checks:**
1. Code formatting (`cargo fmt -- --check`)
2. Linting (`cargo clippy`)
3. Unit tests
4. Integration tests
5. Benchmarks
6. Release build
7. End-to-end analysis with release binary

#### Comprehensive QA
```bash
./scripts/run_qa_tests.sh
```
**Features:**
- Detector accuracy validation
- Regression testing
- Performance benchmarks
- Test fixture validation
- Automated report generation
- Results archival

#### v1.0 Release Testing
```bash
./scripts/test-v1-release.sh
```
**Components:**
- Library build validation
- Binary build validation
- Multiple output format tests (JSON, HTML, Markdown)
- CLI command validation
- Frontend build (if applicable)
- API server tests (if applicable)
- Documentation validation

### Supporting Scripts

```bash
# Coverage report
./scripts/coverage-report.sh

# All CLI commands
./scripts/test-all-cli-commands.sh

# Detector coverage verification
./scripts/verify_detector_coverage.sh

# Real-world codebase testing
./scripts/test_real_codebases.sh

# Security validation
./scripts/validate-security-fixes.sh

# Resource management
./scripts/test-resource-management.sh

# Detector calibration
./scripts/calibrate_detectors.sh
```

## Feature Flags

### CLI Profiles
- `cli-core` - Minimal CLI functionality
- `cli-standard` - **Recommended** - Standard CLI with language support and security
- `cli-ai` - Standard + AI features (requires Ollama)
- `cli-plugins` - Standard + plugin support
- `cli-full` - Everything enabled

### Individual Features
- `tree-sitter` - AST parsing for all languages
- `security` - Security analysis features
- `ai` - AI integration base
- `local-ai` - Local AI with Ollama
- `wasm-plugins` - Plugin system
- `ast-cache` - AST caching
- `analysis-cache` - Analysis result caching

## External Dependencies

### Required for Full Testing
- **Rust toolchain** - Latest stable
- **cargo-clippy** - Linting
- **cargo-fmt** - Formatting

### Optional but Recommended
- **Ollama** - For AI-related tests
  - Check: `curl http://localhost:11434/api/tags`
  - Model: `ollama pull deepseek-coder:6.7b-instruct-q4_0`
- **cargo-audit** - Security auditing
  - Install: `cargo install cargo-audit`
- **cargo-tarpaulin** - Coverage generation
  - Install: `cargo install cargo-tarpaulin`

## Test Fixtures

### Location
- `tests/fixtures/` - Test code samples
- Supported languages: Rust (.rs), Python (.py), JavaScript (.js), TypeScript (.ts)

### Adding Fixtures
1. Create file in `tests/fixtures/`
2. Include realistic code patterns
3. Document expected detection results
4. Reference in relevant test files

## Benchmarks

```bash
# Run all benchmarks
cargo bench

# Specific benchmark suite
cargo bench --bench <name>

# With baseline comparison
cargo bench -- --baseline <name>
```

## Coverage Generation

### Using cargo-tarpaulin (Recommended)
```bash
# Install
cargo install cargo-tarpaulin

# Generate HTML coverage
cargo tarpaulin --out Html --output-dir reports/qa/coverage

# Generate lcov format
cargo tarpaulin --out Lcov --output-dir reports/qa/coverage

# With all features
cargo tarpaulin --all-features --out Html --output-dir reports/qa/coverage
```

### Using Script
```bash
./scripts/coverage-report.sh
```

## Continuous Integration

### GitHub Actions Workflow
Tests run automatically on:
- Pull requests
- Pushes to main/master
- Release branches

### CI Test Sequence
1. Format check
2. Clippy linting
3. Unit tests
4. Integration tests
5. Security audit
6. Build release artifacts

## Troubleshooting

### Common Issues

**Tests timeout**
```bash
# Increase timeout (default: 60s)
RUST_TEST_TIME_OUT=300 cargo test
```

**Out of memory**
```bash
# Run single-threaded
cargo test -- --test-threads=1
```

**Ollama tests fail**
```bash
# Start Ollama
ollama serve

# Pull required model
ollama pull deepseek-coder:6.7b-instruct-q4_0

# Skip AI tests
cargo test --features cli-standard  # without cli-ai
```

**Test file not found**
```bash
# List all test targets
grep "\[\[test\]\]" Cargo.toml -A 2

# Run specific test
cargo test --test <exact_name>
```

### Debug Failed Tests

```bash
# Show all output
cargo test -- --nocapture

# Run specific failing test
cargo test failing_test_name -- --nocapture --exact

# Single-threaded for debugging
cargo test failing_test_name -- --test-threads=1 --nocapture
```

## Test Development Guidelines

### General Principles
- Follow Rust best practices and idioms
- Use descriptive test names (e.g., `test_god_object_detector_identifies_large_classes`)
- Clear AAA pattern: Arrange, Act, Assert
- Test both happy path and error cases
- Use fixtures for complex test data
- Avoid hardcoded paths, use `tempfile` for file operations

### Example Test Structure
```rust
#[test]
fn test_feature_handles_edge_case() {
    // Arrange: Set up test data
    let input = create_test_fixture();

    // Act: Execute the code under test
    let result = feature_function(input);

    // Assert: Verify expected behavior
    assert!(result.is_ok());
    assert_eq!(result.unwrap().count, 5);
}
```

### Integration Test Pattern
```rust
use assert_cmd::Command;

#[test]
fn test_cli_command() {
    let mut cmd = Command::cargo_bin("uveddi").unwrap();
    cmd.arg("analyze")
       .arg("./test-data")
       .assert()
       .success();
}
```

## Test Artifacts

### Output Locations
- **Logs:** `reports/qa/logs/`
- **Coverage:** `reports/qa/coverage/`
- **QA Reports:** `target/qa-reports/`
- **Benchmark Results:** `target/criterion/`

### Viewing Results
```bash
# Open HTML coverage
xdg-open reports/qa/coverage/index.html

# Open QA dashboard
xdg-open target/qa-reports/qa_dashboard.html

# View benchmark report
xdg-open target/criterion/report/index.html
```

## Best Practices

1. **Run tests frequently** - Before committing changes
2. **Use appropriate features** - Don't test with `--all-features` if not needed
3. **Check coverage** - Aim for >80% code coverage on new code
4. **Update fixtures** - Keep test data realistic and comprehensive
5. **Document edge cases** - Comment why specific tests exist
6. **Clean up resources** - Use `tempfile` and proper cleanup
7. **Parallelize when safe** - Use `--test-threads=1` only when necessary

## Contributing Tests

When adding new features:
1. Write tests BEFORE implementation (TDD)
2. Add integration tests for user-facing features
3. Update this README if adding new test categories
4. Ensure CI passes before merging
5. Update test inventory in `reports/qa/test-inventory.md`

## Performance Testing

### Benchmark Suites
Located in `benches/` directory

### Running Benchmarks
```bash
# All benchmarks
cargo bench

# Specific benchmark
cargo bench <benchmark_name>

# Save baseline
cargo bench -- --save-baseline <name>

# Compare to baseline
cargo bench -- --baseline <name>
```

## Release Testing Checklist

Before releasing a new version:

- [ ] `cargo fmt -- --check` passes
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo test --all-features` passes
- [ ] `./scripts/pre_release_test.sh` passes
- [ ] `./scripts/run_qa_tests.sh` passes
- [ ] Coverage >80% on critical paths
- [ ] All benchmarks complete successfully
- [ ] Security audit clean (`cargo audit`)
- [ ] Integration tests with real codebases pass
- [ ] Documentation builds without warnings

## Additional Resources

- **Test Inventory:** `reports/qa/test-inventory.md`
- **QA Reports:** `reports/qa/`
- **Benchmark Data:** `target/criterion/`
- **CI Configuration:** `.github/workflows/`

---

**Last Updated:** 2025-11-22 (v1.0.0 Release)
**QA Lead:** Solo Dev
**Status:** Ready for Release Testing
