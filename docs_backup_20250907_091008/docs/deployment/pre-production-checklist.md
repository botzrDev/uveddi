# Uveddi Pre-Production Testing Checklist
## Version: v1.0-alpha → v1.0-beta

### 🔴 Critical Requirements

#### Core Functionality
- [ ] **Anti-pattern detection works with tree-sitter features**
  - Test: God Objects, Dead Code, Magic Values detection
  - Command: `cargo run --features=production -- analyze ./test_files`
  - Success: Detects >0 issues in test files with known anti-patterns
  
- [ ] **All language parsers functional**
  - [ ] Rust parsing (`--features=rust-lang`)
  - [ ] Python parsing (`--features=python-lang`)
  - [ ] JavaScript parsing (`--features=javascript-lang`)
  - [ ] TypeScript parsing (`--features=typescript-lang`)

- [ ] **Build configurations compile without errors**
  - [ ] `cargo build --features=dev-minimal`
  - [ ] `cargo build --features=dev-core`
  - [ ] `cargo build --features=production`
  - [ ] `cargo build --features=alpha`

#### Performance Benchmarks
- [ ] **Build times acceptable**
  - dev-core: <20s
  - production: <5min
  - Documented workarounds for WSL users

- [ ] **Analysis performance**
  - [ ] Small codebase (<1000 files): <30s
  - [ ] Medium codebase (<10000 files): <5min
  - [ ] Large codebase (>10000 files): <30min with memory optimization

### 🟡 Testing Requirements

#### Test Coverage
- [ ] **Core tests passing (>95%)**
  ```bash
  cargo test --features=production --lib
  ```
  
- [ ] **Integration tests passing (100%)**
  ```bash
  cargo test --features=production --test integration
  ```

- [ ] **Known failing tests documented**
  - Current: 18/679 tests failing
  - Document workarounds in known-issues.md

#### Functional Testing
- [ ] **CLI commands working**
  ```bash
  uveddi --help
  uveddi analyze ./src --output-format json
  uveddi analyze ./src --output-format html
  uveddi serve --port 8888
  ```

- [ ] **Report generation**
  - [ ] JSON reports generate correctly
  - [ ] HTML reports render properly
  - [ ] Markdown reports format correctly

- [ ] **Web services**
  - [ ] API server starts on specified port
  - [ ] Health endpoint responds
  - [ ] Rendering service initializes
  - [ ] Dashboard loads (if frontend enabled)

### 🟢 Documentation Requirements

#### User Documentation
- [ ] **README.md streamlined** (<100 lines)
- [ ] **Getting started guide** works for new users
- [ ] **Installation instructions** accurate for all platforms
- [ ] **CLI reference** complete and accurate

#### Developer Documentation
- [ ] **Building instructions** work
- [ ] **Feature flags** documented
- [ ] **Testing guide** accurate
- [ ] **Contributing guide** clear

#### Production Documentation
- [ ] **This checklist** complete
- [ ] **Production deployment guide** exists
- [ ] **Known issues** documented
- [ ] **Security considerations** documented

### 🔵 WASM Plugin System

#### Plugin Functionality
- [ ] **Plugin CLI commands available**
  ```bash
  uveddi plugin list
  uveddi plugin install test.wasm
  uveddi plugin info <plugin-id>
  uveddi plugin remove <plugin-id>
  ```

- [ ] **Plugin runtime working**
  - [ ] Plugins load successfully
  - [ ] Host functions accessible
  - [ ] Security sandboxing enforced
  - [ ] Resource limits applied

### 🟣 Security & Compliance

#### Security Testing
- [ ] **No hardcoded secrets** in codebase
- [ ] **Dependencies audited**
  ```bash
  cargo audit
  ```
- [ ] **Input validation** on all user inputs
- [ ] **Path traversal** protection verified

#### Licensing
- [ ] **License file** present
- [ ] **Dependencies licenses** compatible
- [ ] **Attribution** requirements met

### 📊 Metrics & Monitoring

#### Observability
- [ ] **Logging** works at all levels
- [ ] **Error reporting** includes context
- [ ] **Performance metrics** collected
- [ ] **Memory usage** within limits

### 🚀 Release Preparation

#### Version Management
- [ ] **Version number** updated in Cargo.toml
- [ ] **CHANGELOG.md** updated with all changes
- [ ] **Git tag** created for release

#### Distribution
- [ ] **Binary builds** for target platforms
  - [ ] Linux x86_64
  - [ ] macOS ARM64
  - [ ] macOS x86_64
  - [ ] Windows x86_64
  
- [ ] **Docker image** builds and runs
- [ ] **Installation script** works

### ✅ Sign-off Criteria

#### Minimum Viable Product (MVP)
- [ ] Core analysis works for at least one language
- [ ] Reports generate in at least one format
- [ ] Documentation sufficient for basic usage
- [ ] No critical security issues

#### Beta Release Criteria
- [ ] All languages supported
- [ ] Web services functional
- [ ] Plugin system operational
- [ ] Documentation complete
- [ ] Performance acceptable
- [ ] <5% test failures

#### Production Release Criteria
- [ ] All tests passing (>99%)
- [ ] Performance optimized
- [ ] Security hardened
- [ ] Documentation comprehensive
- [ ] Support channels established

### 📝 Testing Commands

```bash
# Full pre-production test suite
./scripts/pre-production-test.sh

# Quick validation
cargo build --features=production --release
cargo test --features=production
./target/release/uveddi analyze ./test_files --output-format json

# Performance validation
./scripts/performance-validation.sh

# Security audit
cargo audit
./scripts/security-scan.sh
```

### 🐛 Issue Tracking

Track all issues discovered during pre-production testing:
1. Issue description
2. Severity (Critical/High/Medium/Low)
3. Workaround (if any)
4. Fix ETA
5. Owner

### 📅 Timeline

- **Week 1**: Core functionality testing
- **Week 2**: Performance and security testing
- **Week 3**: Documentation and polish
- **Week 4**: Final validation and release preparation

### 👥 Responsibilities

- **Core Team**: Functionality, performance, security
- **QA Team**: Testing, validation, issue tracking
- **DevOps**: Build, deployment, monitoring
- **Documentation**: User guides, API docs, examples

---

**Last Updated**: September 2024
**Target Release**: v1.0-beta
**Status**: Alpha → Beta transition