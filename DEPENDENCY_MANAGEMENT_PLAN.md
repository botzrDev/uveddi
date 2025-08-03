# Dependency Management and Cleanup Plan (UV-107)

## Current Dependency Issues Analysis
**Problems**: 
- 184 external dependencies with complex feature interactions
- Multiple memory allocators creating conflicts  
- Complex WASM runtime with limited adoption
- Competing serialization frameworks
- 191 wildcard imports reducing code clarity

## Dependency Optimization Strategy

### Phase 5.1: Dependency Audit and Analysis (Week 4)

#### 1. Comprehensive Dependency Review
**Current Major Dependencies** (from cargo tree analysis):
```
▶ Heavy Dependencies (>50 sub-dependencies):
  - arrow: 52 dependencies (data processing)
  - wasmtime: 47 dependencies (WASM runtime)
  - tokio: 23 dependencies (async runtime)
  - sqlx: 19 dependencies (database)

▶ Conflicting Dependencies:
  - Multiple allocators: jemalloc, mimalloc, system
  - Serialization: serde + bincode + rmp + arrow-json
  - HTTP clients: reqwest + hyper + curl
  - Async runtimes: tokio + async-std (indirect)
```

#### 2. Feature Flag Rationalization
**Current Feature Complexity**: 
- 27 feature flags with interdependencies
- Conditional compilation creating test complexity
- Inconsistent feature combinations

**Target**: Simplify to 8 core features:
```toml
[features]
default = ["analysis", "security", "database"]

# Core functionality
analysis = ["ast-parsing", "detectors"]
security = ["authentication", "audit"]
database = ["sqlx", "migrations"]

# Optional enhancements
ai = ["llm-integration", "knowledge-base"]
tui = ["ratatui", "interactive"]
plugins = ["wasmtime", "wasm-runtime"]
web-ui = ["axum", "frontend-assets"]
performance = ["profiling", "benchmarks"]
```

### Phase 5.2: Dependency Consolidation (Week 4-5)

#### 1. Serialization Framework Unification
**Current State**: Multiple competing serialization frameworks
**Target**: Standardize on serde ecosystem

```toml
# Remove conflicting serialization crates
# rmp = "0.8"           # Remove MessagePack 
# bincode = "1.0"       # Remove binary serialization
# arrow-json = "52.2"   # Keep for Arrow compatibility only

# Standardize on serde + JSON for configuration
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"           # For configuration files
```

#### 2. Memory Allocator Simplification  
**Current State**: Multiple allocators causing conflicts
**Target**: Single allocator strategy

```toml
# Production: Use jemalloc for better performance
[target.'cfg(not(target_env = "msvc"))'.dependencies]  
jemallocator = { version = "0.5", optional = true }

[features]
jemalloc = ["jemallocator"]

# Default to system allocator, enable jemalloc for production
default = []
production = ["jemalloc", "optimizations"]
```

#### 3. HTTP Client Consolidation
**Current State**: Multiple HTTP implementations
**Target**: Single HTTP client (reqwest)

```toml
# Remove redundant HTTP clients
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
# Remove: hyper (indirect via reqwest)
# Remove: curl (unused direct dependency)
```

#### 4. WASM Runtime Optimization
**Current State**: Heavy wasmtime dependency (47 sub-dependencies)
**Target**: Make WASM truly optional with lighter alternatives

```toml
# Make WASM plugin system optional
wasmtime = { version = "34.0", optional = true }
wasmtime-wasi = { version = "34.0", optional = true }

# Alternative: Consider wasmer for lighter footprint
# wasmer = { version = "4.0", optional = true }

[features]
wasm-plugins = ["wasmtime", "wasmtime-wasi"]
# wasm-plugins-alt = ["wasmer"]  # Future alternative
```

### Phase 5.3: Wildcard Import Elimination (Week 5)

#### 1. Automated Import Analysis
**Tool**: Create script to identify all wildcard imports

```bash
# Find all wildcard imports
grep -r "use.*::.*\*" src/ --include="*.rs" > wildcard_imports.txt

# Result shows 191 wildcard imports like:
# src/analysis/engine.rs:use crate::analysis::detectors::*;
# src/ast/mod.rs:use tree_sitter::*;
```

#### 2. Import Refactoring Strategy
**Approach**: Replace wildcards with explicit imports

```rust
// Before (wildcard - unclear dependencies)
use crate::analysis::detectors::*;
use tree_sitter::*;

// After (explicit - clear dependencies)  
use crate::analysis::detectors::{
    GodObjectDetector, CycleDetector, DeadCodeDetector,
    MagicValuesDetector, AntiPatternDetector
};
use tree_sitter::{Language, Node, Parser, Query, Tree, TreeCursor};
```

#### 3. Automated Refactoring Tools
**Implementation**: Scripts to automate the conversion

```bash
#!/bin/bash
# scripts/fix-wildcard-imports.sh

# Use rust-analyzer to expand wildcard imports
for file in $(find src -name "*.rs"); do
    # Replace common wildcards with explicit imports
    sed -i 's/use std::\*;/use std::{io, fs, path, collections};/g' "$file"
    sed -i 's/use serde::\*;/use serde::{Serialize, Deserialize};/g' "$file"
done

# Run cargo check to verify compilation
cargo check
```

### Phase 5.4: Build System Optimization (Week 5-6)

#### 1. Compilation Speed Improvements
**Current Issue**: Complex dependency tree slows compilation
**Target**: 50% faster incremental builds

```toml
# Cargo.toml optimizations
[profile.dev]
opt-level = 0
debug = true
split-debuginfo = "unpacked"  # Faster linking on macOS/Linux
incremental = true
codegen-units = 256          # Faster parallel compilation

[profile.dev.package."*"]   # Optimize dependencies in dev builds
opt-level = 1               # Some optimization for dependencies
```

#### 2. Feature Flag Optimization
**Strategy**: Reduce feature flag complexity

```toml
# Simplified feature dependencies
[features]
default = ["analysis", "security"]

analysis = [
    "dep:tree-sitter",
    "dep:petgraph", 
    "ast-parsing"
]

security = [
    "dep:jsonwebtoken",
    "dep:argon2",
    "authentication"
]

# Remove redundant feature combinations
# Old: ai-analysis, ai-explanations, ai-context -> New: ai
ai = ["dep:reqwest", "llm-integration"]
```

#### 3. Optional Dependency Management
**Approach**: Make heavy dependencies truly optional

```toml
# Heavy dependencies as optional
arrow = { version = "52.2", optional = true }
wasmtime = { version = "34.0", optional = true }
ratatui = { version = "0.29", optional = true }

[features]
data-processing = ["arrow"]     # Only for data analysis features
wasm-plugins = ["wasmtime"]     # Only for plugin system
tui = ["ratatui"]              # Only for terminal UI

# Users can choose minimal installation:
# cargo install uveddi --no-default-features --features analysis
```

### Phase 5.5: Dependency Security Hardening (Week 6)

#### 1. Automated Security Scanning
**Implementation**: Enhanced cargo-audit integration

```yaml
# .github/workflows/dependency-security.yml
name: Dependency Security Scan

on:
  schedule:
    - cron: '0 2 * * 1'  # Weekly Monday 2AM
  push:
    paths: ['Cargo.toml', 'Cargo.lock']

jobs:
  security-audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install cargo-audit
        run: cargo install cargo-audit
      - name: Run security audit
        run: cargo audit --deny warnings
      - name: Run dependency analysis
        run: cargo tree --duplicates
```

#### 2. Supply Chain Security
**Strategy**: Pin and verify critical dependencies

```toml
# Cargo.toml - Pin security-critical dependencies
[dependencies]
# Security-critical: pin exact versions
jsonwebtoken = "=8.3.0"       # JWT handling
argon2 = "=0.5.2"             # Password hashing  
rustls = "=0.23.28"           # TLS implementation

# Allow patch updates for non-security dependencies
serde = "1.0"                 # Serialization (stable API)
clap = "4.0"                  # CLI parsing (stable API)
```

#### 3. Dependency Vulnerability Monitoring
**Implementation**: Automated updates with testing

```bash
#!/bin/bash
# scripts/dependency-update.sh

# Update dependencies and run tests
cargo update
cargo test --all-features

# Check for new vulnerabilities
cargo audit

# If tests pass and no vulns, commit changes
if [ $? -eq 0 ]; then
    git add Cargo.lock
    git commit -m "chore: update dependencies - security scan passed"
fi
```

## Implementation Timeline

### Week 4: Dependency Audit and Analysis
- [ ] Complete dependency tree analysis
- [ ] Identify conflicting and redundant dependencies
- [ ] Document current dependency complexity
- [ ] Plan consolidation strategy

### Week 5: Consolidation and Cleanup  
- [ ] Unify serialization frameworks
- [ ] Consolidate HTTP clients
- [ ] Eliminate 191 wildcard imports
- [ ] Optimize feature flag system

### Week 6: Build Optimization and Security
- [ ] Implement build system optimizations
- [ ] Set up automated security scanning
- [ ] Pin security-critical dependencies
- [ ] Document dependency management practices

## Success Metrics

### Quantitative Targets
- [ ] Reduce total dependencies from 184 to <120
- [ ] Eliminate all 191 wildcard imports
- [ ] Reduce feature flags from 27 to 8
- [ ] Achieve 50% faster incremental build times
- [ ] Zero dependency conflicts or warnings

### Qualitative Improvements
- [ ] Clear dependency ownership and rationale
- [ ] Simplified build configuration
- [ ] Enhanced security posture
- [ ] Better dependency documentation

## Risk Mitigation

### Technical Risks
- **Breaking changes**: Incremental migration with testing
- **Performance regression**: Benchmark before/after changes
- **Feature compatibility**: Comprehensive feature matrix testing

### Process Risks
- **Build complexity**: Automated testing of all feature combinations
- **Maintenance burden**: Automated dependency monitoring
- **Security vulnerabilities**: Automated scanning and alerting

## Long-term Maintenance Strategy

### 1. Dependency Governance
- Monthly dependency review meetings
- Quarterly major dependency updates
- Immediate response to security advisories

### 2. Automated Tooling
- CI/CD integration for dependency checks
- Automated vulnerability scanning
- Performance regression detection

### 3. Documentation
- Dependency decision records (ADRs)
- Feature flag usage guidelines
- Security hardening procedures
