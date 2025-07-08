# Uveddi CI/CD Pipeline Guide

## Overview

Uveddi uses a comprehensive CI/CD pipeline built with GitHub Actions to ensure code quality, security, and reliability across all supported platforms. This guide covers the complete pipeline architecture, configuration, and usage.

## Pipeline Architecture

### Core Workflows

1. **Rust CI Pipeline** (`.github/workflows/rust-ci.yml`)
   - Primary quality gate for all Rust code
   - Multi-platform testing and validation
   - Security auditing and dependency checking

2. **Frontend E2E Pipeline** (`.github/workflows/frontend-e2e.yml`)
   - End-to-end testing for the React frontend
   - Cross-browser compatibility testing
   - Performance and accessibility validation

3. **Dependency Updates** (`.github/workflows/dependency-update.yml`)
   - Automated weekly dependency updates
   - Security vulnerability patching
   - Automated PR creation for review

## Rust CI Pipeline Details

### Trigger Conditions

The Rust CI pipeline runs on:
- **Push** to `main` or `develop` branches
- **Pull requests** targeting `main` or `develop`
- **Path filters**: Only when Rust-related files change
  - `src/**`, `tests/**`, `benches/**`, `plugins/**`
  - `Cargo.toml`, `Cargo.lock`
  - CI configuration files

### Pipeline Jobs

#### 1. Check & Format (`check`)
**Purpose**: Fast feedback on code quality and formatting
**Runtime**: ~2-3 minutes

```yaml
Steps:
- Code formatting check (cargo fmt --check)
- Linting with Clippy (cargo clippy -- -D warnings)
- Compilation check (cargo check --all-features)
```

**Requirements for Success**:
- All code must be formatted with `cargo fmt`
- No Clippy warnings allowed
- Code must compile without errors

#### 2. Test Suite (`test`)
**Purpose**: Comprehensive testing across platforms and Rust versions
**Runtime**: ~8-12 minutes
**Matrix Strategy**:
- **Platforms**: Ubuntu, Windows, macOS
- **Rust Versions**: Stable, Beta (Ubuntu only for Beta)

```yaml
Test Types:
- Unit tests (cargo test --lib --bins)
- Integration tests (cargo test --test '*')
- Documentation tests (cargo test --doc)
```

#### 3. Minimal Build Testing (`test-minimal`)
**Purpose**: Ensure core functionality works without optional dependencies
**Runtime**: ~3-5 minutes

```yaml
Feature Combinations Tested:
- No default features
- Core analysis only
- Minimal build configuration
```

#### 4. Security Audit (`security`)
**Purpose**: Vulnerability and dependency compliance checking
**Runtime**: ~2-4 minutes

```yaml
Security Tools:
- cargo-audit: CVE vulnerability scanning
- cargo-deny: License and dependency policy enforcement
```

#### 5. Architecture Validation (`architecture`)
**Purpose**: Enforce architectural constraints and boundaries
**Runtime**: ~1-2 minutes

```yaml
Validations:
- Layer boundary enforcement
- Module dependency rules
- Error handling patterns
- Documentation requirements
```

#### 6. Performance Benchmarks (`benchmarks`)
**Purpose**: Performance regression detection
**Runtime**: ~5-10 minutes
**Trigger**: Only on `main` branch pushes

```yaml
Benchmark Suites:
- AST parsing performance
- Analysis engine throughput
- Memory usage profiling
```

#### 7. Code Coverage (`coverage`)
**Purpose**: Track test coverage and identify gaps
**Runtime**: ~5-8 minutes

```yaml
Coverage Tools:
- cargo-llvm-cov for coverage generation
- Codecov integration for reporting
- LCOV format for detailed analysis
```

#### 8. Multi-Platform Builds (`build`)
**Purpose**: Create release-ready binaries
**Runtime**: ~10-15 minutes
**Trigger**: Only on `main` or `develop` branches

```yaml
Build Targets:
- Linux x86_64 (Ubuntu)
- Windows x86_64 (MSVC)
- macOS x86_64 (Intel)
- macOS aarch64 (Apple Silicon)
```

#### 9. AI Integration Tests (`ai-integration`)
**Purpose**: Validate AI provider integrations
**Runtime**: ~3-5 minutes
**Trigger**: Only on main/develop pushes with secrets available

```yaml
AI Providers Tested:
- OpenAI GPT integration
- Anthropic Claude integration
- Ollama local AI integration
- Graceful fallback behavior
```

## Feature Flags System

Uveddi uses Cargo feature flags for modular compilation:

### Available Features

```toml
[features]
default = ["analysis", "local-ai"]

# Core functionality
analysis = ["tree-sitter", "tree-sitter-rust", "tree-sitter-python", 
           "tree-sitter-javascript", "tree-sitter-typescript"]

# AI provider groups
ai = ["openai", "anthropic", "ollama"]
local-ai = ["ollama"]
cloud-ai = ["openai", "anthropic"]

# Individual providers
openai = ["reqwest"]
anthropic = ["reqwest"]
ollama = ["reqwest"]

# Optional components
backend = ["reqwest"]
plugins = []  # Experimental
minimal = []  # Lightweight build
```

### Feature Usage in CI

```bash
# Test different feature combinations
cargo test --no-default-features                    # Minimal
cargo test --no-default-features --features analysis # Core only
cargo test --all-features                           # Everything
cargo test --features "analysis,local-ai"           # Typical usage
```

## Security Configuration

### Dependency Policy (`deny.toml`)

The pipeline enforces strict dependency policies:

```toml
[advisories]
vulnerability = "deny"    # Block known CVEs
unmaintained = "warn"     # Warn about unmaintained crates
yanked = "warn"          # Warn about yanked versions

[licenses]
allow = ["MIT", "Apache-2.0", "BSD-2-Clause", "BSD-3-Clause", "ISC"]
deny = ["GPL-2.0", "GPL-3.0", "AGPL-1.0", "AGPL-3.0"]

[bans]
multiple-versions = "warn"  # Warn about duplicate dependencies
```

### Security Scanning

- **Vulnerability Database**: Updated from RustSec Advisory DB
- **License Compliance**: Automated license compatibility checking
- **Dependency Auditing**: Transitive dependency analysis

## Quality Gates

### Branch Protection Requirements

For production readiness, configure these branch protection rules:

```yaml
Required Status Checks:
- Check & Format
- Test Suite (all matrix combinations)
- Security Audit
- Architecture Validation

Additional Requirements:
- Require code review approval
- Dismiss stale reviews when new commits are pushed
- Require up-to-date branches before merging
```

### Code Quality Standards

```yaml
Formatting:
- Must pass `cargo fmt --check`
- Consistent with Rust standard formatting

Linting:
- Zero Clippy warnings allowed
- Custom lint configuration in Cargo.toml

Testing:
- All tests must pass on all platforms
- Minimum 15% code coverage (target: 25%+)
- Integration tests for major features

Documentation:
- Public APIs must have doc comments
- Architecture decisions documented
- README kept up-to-date
```

## Performance Monitoring

### Benchmark Integration

```yaml
Benchmark Suites:
- File scanning performance (benches/analysis_engine.rs)
- AST parsing throughput
- Memory usage profiling
- AI provider response times

Regression Detection:
- Baseline performance metrics
- Automated performance alerts
- Historical trend analysis
```

### Performance Targets

```yaml
Analysis Performance:
- 100-file Rust project: <30 seconds
- 1000-file project: <5 minutes
- Memory usage: <2GB for large projects

AI Integration:
- Local AI response: <10 seconds
- Cloud AI response: <5 seconds
- Fallback activation: <1 second
```

## Monitoring and Alerting

### Success Indicators

```yaml
Green Build Indicators:
✅ All tests pass across platforms
✅ Zero security vulnerabilities
✅ Code coverage maintained/improved
✅ Performance within acceptable bounds
✅ Architecture constraints satisfied
```

### Failure Notifications

```yaml
Slack Integration:
- Channel: #uveddi-alerts
- Triggers: Failures on main/develop
- Information: Failed jobs, commit details, action links

GitHub Integration:
- PR status checks
- Detailed job summaries
- Artifact uploads for debugging
```

## Troubleshooting Common Issues

### Build Failures

**Compilation Errors**:
```bash
# Local debugging
cargo check --all-features
cargo clippy --all-features -- -D warnings
```

**Test Failures**:
```bash
# Run specific test suite
cargo test --test integration_tests
cargo test --lib -- --nocapture
```

**Platform-Specific Issues**:
```bash
# Test on specific platform
cargo test --target x86_64-pc-windows-msvc
cargo test --target x86_64-apple-darwin
```

### Security Audit Failures

**Vulnerability Found**:
```bash
# Check specific advisory
cargo audit --db ~/.cargo/advisory-db
# Update dependencies
cargo update
```

**License Issues**:
```bash
# Check license compliance
cargo deny check licenses
# Review deny.toml configuration
```

### Performance Regressions

**Benchmark Failures**:
```bash
# Run benchmarks locally
cargo bench
# Compare with baseline
cargo bench -- --save-baseline main
```

## Local Development Integration

### Pre-commit Hooks

Install and configure pre-commit hooks:

```bash
# Install pre-commit
pip install pre-commit
pre-commit install

# Manual run
pre-commit run --all-files
```

### Development Environment Setup

Use the automated setup script:

```bash
# One-command environment setup
./scripts/setup-dev-environment.sh
```

This script installs:
- Required Rust components
- Cargo tools (audit, deny, llvm-cov, etc.)
- Pre-commit hooks
- Development configuration

## Continuous Improvement

### Pipeline Optimization

```yaml
Current Optimizations:
- Aggressive caching of Cargo dependencies
- Parallel job execution
- Conditional job triggers
- Artifact reuse between jobs

Future Improvements:
- Build time optimization
- Test parallelization
- Cache hit rate improvement
- Resource usage optimization
```

### Metrics and Analytics

```yaml
Tracked Metrics:
- Build success rate
- Average build time
- Test coverage trends
- Security vulnerability count
- Performance benchmark trends

Review Schedule:
- Weekly: Build performance review
- Monthly: Security audit review
- Quarterly: Pipeline architecture review
```

## Integration with Development Workflow

### Pull Request Workflow

1. **Developer creates PR**
2. **CI pipeline triggers automatically**
3. **All quality gates must pass**
4. **Code review required**
5. **Merge after approval + CI success**

### Release Workflow

1. **Merge to main triggers full pipeline**
2. **Release binaries built for all platforms**
3. **Performance benchmarks updated**
4. **Security audit completed**
5. **Artifacts available for distribution**

---

*For questions about the CI/CD pipeline, see the troubleshooting section or contact the development team.*