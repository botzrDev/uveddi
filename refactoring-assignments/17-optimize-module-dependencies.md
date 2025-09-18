# Assignment 17: Optimize Module Dependencies

## Priority: MEDIUM
## Estimated Time: 3-4 hours
## Dependencies: Assignments 13-16 (Dependency management)

## Objective
Optimize module dependency structure for faster compilation, better caching, and cleaner architecture.

## Current Problem
- Broad dependency chains slow compilation
- Poor module boundaries affect incremental compilation
- Unclear dependency relationships
- Over-dependency on large external crates

## Tasks

### 1. Analyze Current Dependency Structure

#### A. Generate Dependency Reports:
```bash
# Analyze internal module dependencies
cargo tree --depth=3 > internal_dependencies.txt

# Find heavy dependencies
cargo tree --duplicates > duplicate_dependencies.txt

# Analyze compilation units
cargo build --timings
# This creates cargo-timing.html for analysis

# Find unused dependencies
cargo +nightly udeps > unused_dependencies.txt

# Create custom dependency analysis
cat > scripts/analyze-dependencies.py << 'EOF'
#!/usr/bin/env python3
import subprocess
import json
import re

def analyze_crate_dependencies():
    result = subprocess.run(['cargo', 'metadata', '--format-version=1'],
                          capture_output=True, text=True)
    metadata = json.loads(result.stdout)

    print("Heavy Dependencies (>1MB):")
    for package in metadata['packages']:
        # This is a simplified analysis
        if 'size' in package:  # Would need actual size calculation
            print(f"  {package['name']}: {package['version']}")

    print("\nModule Dependency Chains:")
    # Analyze internal dependencies

if __name__ == "__main__":
    analyze_crate_dependencies()
EOF

python3 scripts/analyze-dependencies.py
```

#### B. Create Dependency Matrix:
```bash
# Create module dependency matrix
find src -name "mod.rs" -o -name "lib.rs" | while read modfile; do
    module_path=$(dirname "$modfile")
    module_name=$(basename "$module_path")

    echo "=== Module: $module_name ==="
    echo "Internal dependencies:"
    grep -r "use crate::" "$module_path"/ | cut -d':' -f3 | sort | uniq

    echo "External dependencies:"
    grep -r "^use [^c]" "$module_path"/ | cut -d':' -f3 | cut -d';' -f1 | sort | uniq
    echo
done > module_dependency_matrix.txt
```

### 2. Identify Optimization Opportunities

#### A. Compilation Bottlenecks:
```rust
// Analyze compilation times by module
// Create build.rs script to track compilation
use std::time::Instant;

fn main() {
    let start = Instant::now();

    // Track which modules take longest to compile
    println!("cargo:rerun-if-changed=src/");

    // Could implement actual timing here
    let duration = start.elapsed();
    println!("cargo:warning=Build script took: {:?}", duration);
}
```

#### B. Identify Module Categories:
```
Core Modules (should be fast):
- types/
- error/
- interfaces/

Heavy Modules (acceptable to be slower):
- analysis/
- report/
- plugins/

Leaf Modules (minimal dependencies):
- utils/
- config/
- cli/
```

### 3. Implement Layered Dependencies

#### A. Define Dependency Layers:
```rust
// src/layers/mod.rs
//! Enforced dependency layers for clean architecture
//!
//! Dependencies must flow in one direction only:
//! presentation -> application -> domain -> infrastructure -> types

pub mod types {
    //! Pure data types with minimal dependencies
    //! Dependencies: std, serde, basic traits only
}

pub mod infrastructure {
    //! External system integrations
    //! Dependencies: types + external crates (database, HTTP, etc.)
    pub mod database;
    pub mod filesystem;
    pub mod network;
}

pub mod domain {
    //! Business logic and rules
    //! Dependencies: types + minimal external crates
    pub mod analysis;
    pub mod validation;
    pub mod rules;
}

pub mod application {
    //! Application services and orchestration
    //! Dependencies: domain + infrastructure
    pub mod services;
    pub mod workflows;
}

pub mod presentation {
    //! User interfaces
    //! Dependencies: application layer
    pub mod cli;
    pub mod api;
}
```

#### B. Create Dependency Validation:
```rust
// src/layers/validation.rs
//! Compile-time dependency validation

#[cfg(test)]
mod dependency_tests {
    //! These tests ensure layers don't violate dependency rules

    #[test]
    fn types_layer_minimal_dependencies() {
        // Types layer should only depend on std and basic crates
        // This would be implemented with actual dependency checking

        let allowed_deps = vec!["std", "serde", "chrono"];
        // Check that types modules only use allowed dependencies
    }

    #[test]
    fn domain_layer_no_infrastructure_deps() {
        // Domain layer should not import infrastructure
        // Use static analysis to verify this
    }

    #[test]
    fn no_circular_dependencies() {
        // Verify no circular imports between layers
    }
}

// Could implement with macros to enforce at compile time
macro_rules! enforce_layer_dependency {
    ($current_layer:ident can_use [$($allowed:ident),*]) => {
        $(
            #[allow(unused_imports)]
            use crate::layers::$allowed::*;
        )*
    };
    ($current_layer:ident cannot_use [$($forbidden:ident),*]) => {
        $(
            // This would create compile errors if violated
            #[cfg(feature = "dependency-checking")]
            compile_error!(concat!(
                stringify!($current_layer),
                " layer cannot depend on ",
                stringify!($forbidden)
            ));
        )*
    };
}
```

### 4. Optimize External Dependencies

#### A. Minimize Heavy Dependencies:
```toml
# Optimize Cargo.toml dependencies
[dependencies]
# Replace heavy dependencies with lighter alternatives
serde = { version = "1.0", features = ["derive"] }
# Instead of: serde = { version = "1.0", features = ["derive", "rc"] }

tokio = { version = "1.0", features = ["rt", "fs", "net"] }
# Instead of: tokio = { version = "1.0", features = ["full"] }

# Use workspace dependencies to avoid duplication
clap = { workspace = true }
anyhow = { workspace = true }

# Optional heavy dependencies
tree-sitter = { version = "0.20", optional = true }
wasmtime = { version = "13.0", optional = true }

[features]
# Feature-gate heavy dependencies
tree-sitter-support = ["dep:tree-sitter"]
wasm-plugins = ["dep:wasmtime"]

# Optimize for different use cases
minimal = []
standard = ["tree-sitter-support"]
full = ["standard", "wasm-plugins"]
```

#### B. Create Lighter Abstraction Layer:
```rust
// src/abstractions/mod.rs
//! Lightweight abstractions over heavy dependencies

#[cfg(feature = "tree-sitter-support")]
pub mod ast {
    pub use tree_sitter::{Tree, Node, Parser};

    pub trait AstNode {
        fn kind(&self) -> &str;
        fn start_position(&self) -> (usize, usize);
        fn end_position(&self) -> (usize, usize);
    }

    impl AstNode for tree_sitter::Node<'_> {
        fn kind(&self) -> &str {
            self.kind()
        }

        fn start_position(&self) -> (usize, usize) {
            let pos = self.start_position();
            (pos.row, pos.column)
        }

        fn end_position(&self) -> (usize, usize) {
            let pos = self.end_position();
            (pos.row, pos.column)
        }
    }
}

#[cfg(not(feature = "tree-sitter-support"))]
pub mod ast {
    // Provide minimal fallback implementation
    pub trait AstNode {
        fn kind(&self) -> &str;
        fn start_position(&self) -> (usize, usize);
        fn end_position(&self) -> (usize, usize);
    }

    pub struct MockNode;
    impl AstNode for MockNode {
        fn kind(&self) -> &str { "unknown" }
        fn start_position(&self) -> (usize, usize) { (0, 0) }
        fn end_position(&self) -> (usize, usize) { (0, 0) }
    }
}
```

### 5. Implement Lazy Loading

#### A. Lazy Module Loading:
```rust
// src/lazy/mod.rs
use std::sync::OnceLock;

pub struct LazyModule<T> {
    cell: OnceLock<T>,
    initializer: fn() -> T,
}

impl<T> LazyModule<T> {
    pub const fn new(initializer: fn() -> T) -> Self {
        Self {
            cell: OnceLock::new(),
            initializer,
        }
    }

    pub fn get(&self) -> &T {
        self.cell.get_or_init(self.initializer)
    }
}

// Example usage for heavy modules
static AI_SERVICE: LazyModule<Box<dyn AiService>> = LazyModule::new(|| {
    #[cfg(feature = "ai-integration")]
    {
        Box::new(crate::ai::OllamaService::new())
    }
    #[cfg(not(feature = "ai-integration"))]
    {
        Box::new(crate::ai::MockAiService::new())
    }
});

pub fn get_ai_service() -> &'static dyn AiService {
    AI_SERVICE.get().as_ref()
}
```

#### B. Dynamic Feature Loading:
```rust
// src/features/dynamic.rs
pub struct FeatureLoader {
    loaded_features: std::collections::HashSet<String>,
}

impl FeatureLoader {
    pub fn load_feature(&mut self, feature: &str) -> Result<()> {
        match feature {
            "ai-integration" => {
                #[cfg(feature = "ai-integration")]
                {
                    // Initialize AI services
                    self.loaded_features.insert(feature.to_string());
                    Ok(())
                }
                #[cfg(not(feature = "ai-integration"))]
                {
                    Err(UveddiError::FeatureNotAvailable {
                        feature: feature.to_string(),
                    })
                }
            }
            "web-dashboard" => {
                #[cfg(feature = "web-dashboard")]
                {
                    // Initialize web services
                    self.loaded_features.insert(feature.to_string());
                    Ok(())
                }
                #[cfg(not(feature = "web-dashboard"))]
                {
                    Err(UveddiError::FeatureNotAvailable {
                        feature: feature.to_string(),
                    })
                }
            }
            _ => Err(UveddiError::FeatureNotAvailable {
                feature: feature.to_string(),
            }),
        }
    }

    pub fn is_loaded(&self, feature: &str) -> bool {
        self.loaded_features.contains(feature)
    }
}
```

### 6. Create Workspace Structure

#### A. Split into Workspace Crates:
```toml
# Cargo.toml (workspace root)
[workspace]
members = [
    "crates/uveddi-core",
    "crates/uveddi-cli",
    "crates/uveddi-analysis",
    "crates/uveddi-plugins",
    "crates/uveddi-types",
]

[workspace.dependencies]
# Shared dependencies across workspace
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["rt"] }
anyhow = "1.0"
thiserror = "1.0"

# crates/uveddi-types/Cargo.toml
[package]
name = "uveddi-types"
version = "0.9.0"

[dependencies]
serde = { workspace = true }
chrono = "0.4"

# crates/uveddi-core/Cargo.toml
[package]
name = "uveddi-core"
version = "0.9.0"

[dependencies]
uveddi-types = { path = "../uveddi-types" }
serde = { workspace = true }
tokio = { workspace = true }

# crates/uveddi-analysis/Cargo.toml
[package]
name = "uveddi-analysis"
version = "0.9.0"

[dependencies]
uveddi-core = { path = "../uveddi-core" }
uveddi-types = { path = "../uveddi-types" }
tree-sitter = { version = "0.20", optional = true }

[features]
tree-sitter-support = ["dep:tree-sitter"]
```

#### B. Update Internal Dependencies:
```rust
// In uveddi-core/src/lib.rs
pub use uveddi_types::*;

// In uveddi-analysis/src/lib.rs
use uveddi_core::AnalysisEngine;
use uveddi_types::{AnalysisResult, Finding};
```

### 7. Optimize Compilation Performance

#### A. Compilation Units:
```rust
// src/compilation/mod.rs
//! Optimize compilation by reducing cross-module dependencies

// Group related functionality to reduce recompilation
pub mod analysis_core {
    //! Core analysis types and traits (changes rarely)
    pub use crate::types::*;
    pub use crate::interfaces::*;
}

pub mod analysis_impl {
    //! Analysis implementations (changes frequently)
    use super::analysis_core::*;
    // Implementation code here
}

// Use re-exports to minimize dependency chains
pub use analysis_core::*;
```

#### B. Incremental Compilation Optimization:
```rust
// build.rs
use std::env;

fn main() {
    // Optimize for incremental compilation
    if env::var("CARGO_CFG_DEBUG_ASSERTIONS").is_ok() {
        // Development build optimizations
        println!("cargo:rustc-cfg=dev_mode");
    }

    // Feature-specific build optimizations
    if env::var("CARGO_FEATURE_TREE_SITTER").is_ok() {
        println!("cargo:rustc-cfg=has_tree_sitter");
    }
}
```

### 8. Create Dependency Monitoring

#### A. Dependency Health Check:
```rust
// src/dependencies/health.rs
pub struct DependencyHealthChecker {
    checks: Vec<Box<dyn DependencyCheck>>,
}

pub trait DependencyCheck {
    fn name(&self) -> &str;
    fn check(&self) -> DependencyStatus;
}

pub enum DependencyStatus {
    Healthy,
    Warning(String),
    Error(String),
}

impl DependencyHealthChecker {
    pub fn new() -> Self {
        let mut checker = Self {
            checks: Vec::new(),
        };

        checker.add_check(Box::new(DatabaseDependencyCheck));
        checker.add_check(Box::new(AiServiceDependencyCheck));

        checker
    }

    pub fn add_check(&mut self, check: Box<dyn DependencyCheck>) {
        self.checks.push(check);
    }

    pub fn check_all(&self) -> Vec<(String, DependencyStatus)> {
        self.checks
            .iter()
            .map(|check| (check.name().to_string(), check.check()))
            .collect()
    }
}

struct DatabaseDependencyCheck;
impl DependencyCheck for DatabaseDependencyCheck {
    fn name(&self) -> &str { "database" }

    fn check(&self) -> DependencyStatus {
        // Check database connectivity
        DependencyStatus::Healthy
    }
}
```

### 9. Create Build Optimization Scripts

#### A. Build Performance Analysis:
```bash
#!/bin/bash
# scripts/analyze-build-performance.sh

echo "Analyzing build performance..."

# Clean build timing
cargo clean
cargo build --timings 2>&1 | tee build-timing.log

# Incremental build timing
touch src/main.rs
time cargo build 2>&1 | tee incremental-timing.log

# Feature-specific build times
for feature in minimal standard full; do
    echo "Testing feature: $feature"
    cargo clean
    time cargo build --features $feature 2>&1 | tee "build-$feature.log"
done

# Dependency analysis
cargo tree --depth 1 > dependency-tree-shallow.txt
cargo tree --depth 3 > dependency-tree-deep.txt

# Generate report
python3 scripts/generate-build-report.py
```

#### B. Dependency Optimization Script:
```bash
#!/bin/bash
# scripts/optimize-dependencies.sh

echo "Optimizing dependencies..."

# Find unused dependencies
cargo +nightly udeps

# Check for duplicate dependencies
cargo tree --duplicates

# Analyze dependency sizes
for dep in $(cargo tree --depth 1 | grep -v "^[[:space:]]*$" | cut -d' ' -f1); do
    echo "Analyzing dependency: $dep"
    # Could add actual size analysis here
done

# Suggest optimizations
echo "Optimization suggestions:"
echo "1. Remove unused dependencies"
echo "2. Use lighter alternatives for heavy crates"
echo "3. Feature-gate optional dependencies"
```

### 10. Validate Optimizations

#### A. Performance Benchmarks:
```rust
// benches/dependency_benchmarks.rs
use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark_module_loading(c: &mut Criterion) {
    c.bench_function("load_core_modules", |b| {
        b.iter(|| {
            // Benchmark core module loading time
            let _core = uveddi_core::init();
        })
    });

    c.bench_function("load_analysis_modules", |b| {
        b.iter(|| {
            let _analysis = uveddi_analysis::init();
        })
    });
}

fn benchmark_compilation_units(c: &mut Criterion) {
    c.bench_function("compile_types_only", |b| {
        b.iter(|| {
            // This would need special test setup
            // to benchmark compilation units
        })
    });
}

criterion_group!(benches, benchmark_module_loading, benchmark_compilation_units);
criterion_main!(benches);
```

#### B. Dependency Validation Tests:
```rust
// tests/dependency_validation.rs
#[test]
fn validate_layer_dependencies() {
    // Test that dependency layers are respected
    // This would use static analysis or metadata
}

#[test]
fn validate_feature_dependencies() {
    // Test that features properly gate their dependencies
    #[cfg(not(feature = "tree-sitter"))]
    {
        // Ensure tree-sitter types are not available
        // when feature is disabled
    }
}

#[test]
fn validate_compilation_performance() {
    // Test that compilation times are within acceptable ranges
    // This would need integration with build timing data
}
```

## Success Criteria
- [ ] Reduced compilation time by at least 25%
- [ ] Clear dependency layering enforced
- [ ] Workspace structure implemented if beneficial
- [ ] Heavy dependencies properly feature-gated
- [ ] No circular dependencies
- [ ] Improved incremental compilation performance

## Performance Targets
- **Clean build**: <120s (down from current)
- **Incremental build**: <10s for small changes
- **Feature builds**:
  - minimal: <30s
  - standard: <60s
  - full: <120s

## Verification Commands
```bash
# Test build performance
./scripts/analyze-build-performance.sh

# Validate dependencies
cargo deny check
cargo +nightly udeps

# Test workspace structure
cargo check --workspace

# Benchmark dependency loading
cargo bench dependency_benchmarks
```

## Completion Notes
_To be filled by AI developer:_
- Compilation time improvement: ___
- Workspace structure: ___
- Dependencies optimized: ___
- Layer validation: ___
- Performance benchmarks: ___