# Rust Project Analysis Guide

## Overview

This guide covers Uveddi analysis for Rust projects, including Cargo-based projects, libraries, and applications.

## Key Analysis Areas

### 1. Dead Code Detection

Rust's compiler is excellent at dead code detection, but Uveddi provides additional insights:

- **Exported but unused public functions** - Library APIs that may be deprecated
- **Conditional compilation issues** - Code that's dead under certain feature flags
- **Test-only code** - Functions only used in tests but marked as public

**Configuration Tips:**
```toml
[dead_code]
confidence_threshold = 0.9  # Higher for Rust due to strong typing
library_mode = true         # Enable for library crates
ignore_patterns = [
    "src/bin/**",           # Binary entry points
    "**/examples/**",       # Example code
    "benches/**"            # Benchmark code
]
```

### 2. Large Classes Detection

In Rust, this translates to:
- **Large structs** with too many fields
- **Large impl blocks** with too many methods
- **God objects** that do too many things

**Rust-Specific Thresholds:**
- Max logical LOC: 200 (Rust encourages smaller modules)
- Max methods: 20 (per impl block)
- Max fields: 15 (consider using composition)

### 3. Architectural Anti-Patterns

**Common Rust Anti-Patterns:**
- **Circular dependencies** between modules
- **God modules** (lib.rs or main.rs with everything)
- **Tight coupling** between unrelated functionality
- **Magic values** in configuration or business logic

### 4. Rust-Specific Analysis

**Ownership and Borrowing:**
- Excessive cloning (performance issue)
- Unnecessary reference counting (Arc/Rc overuse)
- Mutex contention in concurrent code

**Error Handling:**
- Inconsistent error types
- Panic-prone code paths
- Missing error propagation

## Best Practices

### Code Organization
```rust
// Good: Modular structure
mod config;
mod handlers;
mod models;
mod utils;

// Bad: Everything in lib.rs
pub fn handler1() { ... }
pub fn handler2() { ... }
pub struct Config { ... }
pub struct Model { ... }
```

### Dependency Management
```toml
# Use specific versions
serde = "1.0.152"

# Group related features
[features]
default = ["json-support"]
json-support = ["serde_json"]
xml-support = ["serde_xml_rs"]
```

## Common Issues and Solutions

### Issue: Large main.rs or lib.rs
**Problem:** All code in the root file
**Solution:** Extract modules and use proper visibility

```rust
// Before
pub fn handle_request() { /* 100 lines */ }
pub fn process_data() { /* 150 lines */ }
pub fn validate_input() { /* 75 lines */ }

// After
mod handlers;
mod processors;
mod validators;

pub use handlers::handle_request;
pub use processors::process_data;
pub use validators::validate_input;
```

### Issue: Circular Dependencies
**Problem:** Module A depends on B, B depends on A
**Solution:** Extract shared types or use dependency injection

```rust
// Before: Circular dependency
// user.rs imports order.rs
// order.rs imports user.rs

// After: Extract shared types
// shared/types.rs contains UserId, OrderId
// user.rs and order.rs both import shared/types.rs
```

### Issue: God Struct
**Problem:** Struct with too many responsibilities
**Solution:** Composition and single responsibility

```rust
// Before: God struct
pub struct AppState {
    pub config: Config,
    pub db: Database,
    pub cache: Cache,
    pub auth: Auth,
    pub logger: Logger,
    pub metrics: Metrics,
    // ... 20 more fields
}

// After: Composition
pub struct AppServices {
    pub data: DataServices,
    pub auth: AuthServices,
    pub observability: ObservabilityServices,
}

pub struct DataServices {
    pub db: Database,
    pub cache: Cache,
}
```

## Analysis Commands

```bash
# Basic Rust analysis
uveddi analyze src/

# Focus on library exports
uveddi analyze src/ --library-mode

# Include benchmark and example code
uveddi analyze . --include-patterns "benches/**,examples/**"

# Generate detailed architectural report
uveddi analyze src/ --output-format html --enable-ai

# Check for performance anti-patterns
uveddi analyze src/ --detectors performance,memory

# Validate before publishing to crates.io
uveddi analyze src/ --strict --no-warnings
```

## Integration with Cargo

Add to your `Cargo.toml`:
```toml
[package.metadata.uveddi]
# Uveddi-specific configuration
strict_mode = true
check_examples = true
check_benchmarks = false
```

## CI/CD Integration

```yaml
# .github/workflows/analysis.yml
name: Code Analysis

on: [push, pull_request]

jobs:
  analyze:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install Uveddi
        run: cargo install uveddi
      - name: Run Analysis
        run: uveddi analyze src/ --output-format json --output analysis.json
      - name: Upload Results
        uses: actions/upload-artifact@v3
        with:
          name: analysis-results
          path: analysis.json
```

## Performance Considerations

- **Large workspaces:** Use `--parallel` flag for faster analysis
- **Memory usage:** Configure `--memory-limit` for large codebases
- **Incremental analysis:** Use `--changed-files` for Git-based analysis
- **Feature flags:** Analyze with different feature combinations

## Troubleshooting

### Common Compilation Issues
```bash
# Check if Rust toolchain is properly configured
cargo check

# Verify Uveddi can parse your code
uveddi doctor --parsers

# Run analysis with verbose output
uveddi analyze src/ --verbose
```

### Performance Issues
```bash
# Check analysis performance
uveddi analyze src/ --benchmark

# Reduce analysis scope
uveddi analyze src/lib.rs --depth 3

# Use memory-optimized settings
uveddi analyze src/ --memory-optimization
```

For more Rust-specific tips, see the [Rust Performance Book](https://nnethercote.github.io/perf-book/) and [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/).