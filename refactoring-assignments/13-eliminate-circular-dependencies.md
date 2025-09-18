# Assignment 13: Eliminate Circular Dependencies

## Priority: CRITICAL
## Estimated Time: 4-5 hours
## Dependencies: Phase 1 (God Object refactoring)

## Objective
Identify and eliminate circular dependencies between modules to improve maintainability and compilation performance.

## Current Problem
- Potential circular dependencies between analysis ↔ report, application ↔ database
- Core module using `pub use *;` creates unclear dependency chains
- Tight coupling makes testing and maintenance difficult

## Tasks

### 1. Map Current Dependencies

#### A. Generate Dependency Graph
```bash
# Install cargo-deps for dependency visualization
cargo install cargo-deps

# Generate full dependency graph
cargo deps --all-deps --include-orphans | dot -Tsvg > current_dependencies.svg

# Create text-based dependency report
find src -name "*.rs" -exec grep -l "use crate::" {} \; | while read file; do
    echo "=== $file ==="
    grep "use crate::" "$file" | sort | uniq
    echo
done > dependency_report.txt
```

#### B. Identify Module Relationships
Create module dependency matrix:
```
Module A -> Module B means "A depends on B"

analysis -> report (for output formatting)
report -> analysis (for analysis types)
application -> database (for persistence)
database -> application (for callbacks?)
core -> everything (via pub use *)
```

### 2. Analyze Circular Dependency Patterns

#### A. Direct Circular Dependencies
Look for direct A -> B -> A patterns:
```bash
# Script to detect direct circular imports
for module in $(find src -name "mod.rs" -o -name "lib.rs" | xargs dirname | sort | uniq); do
    module_name=$(basename "$module")
    echo "Checking module: $module_name"

    # Find what this module imports
    deps_out=$(grep -r "use crate::$module_name" src/ || true)

    # Find what imports this module
    deps_in=$(grep -r "use crate::" "$module"/ | grep -v "use crate::$module_name" || true)

    echo "Dependencies out: $deps_out"
    echo "Dependencies in: $deps_in"
    echo "---"
done > circular_dependency_analysis.txt
```

#### B. Indirect Circular Dependencies
Look for A -> B -> C -> A patterns:
- Create dependency tree for each module
- Use tools like `madge` or custom scripts
- Document dependency chains longer than 3 modules

### 3. Break Analysis ↔ Report Circular Dependency

#### Current Problem:
```rust
// In analysis module:
use crate::report::ReportGenerator;

// In report module:
use crate::analysis::AnalysisResult;
```

#### Solution: Extract Shared Types
```
src/
├── types/
│   ├── mod.rs
│   ├── analysis_result.rs
│   ├── report_format.rs
│   └── shared.rs
├── analysis/
│   ├── mod.rs (only analysis logic)
│   └── engine.rs
└── report/
    ├── mod.rs (only report generation)
    └── generators/
```

#### Implementation:
```rust
// src/types/analysis_result.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub findings: Vec<Finding>,
    pub metrics: Metrics,
    pub metadata: AnalysisMetadata,
}

// src/analysis/mod.rs
use crate::types::AnalysisResult;
// No imports from report module

// src/report/mod.rs
use crate::types::AnalysisResult;
// No imports from analysis module
```

### 4. Break Application ↔ Database Circular Dependency

#### Current Problem:
```rust
// In application module:
use crate::database::DatabaseService;

// In database module:
use crate::application::AppConfig;  // Circular!
```

#### Solution: Dependency Inversion
```rust
// src/interfaces/database.rs
pub trait DatabaseService {
    async fn save_analysis(&self, result: &AnalysisResult) -> Result<()>;
    async fn load_analysis(&self, id: &str) -> Result<AnalysisResult>;
}

// src/database/mod.rs
use crate::interfaces::DatabaseService;
use crate::types::AnalysisResult;  // Only types, not application

// src/application/mod.rs
use crate::interfaces::DatabaseService;
// Inject database service via dependency injection
```

### 5. Refactor Core Module

#### Current Problem:
```rust
// src/core/mod.rs
pub use features::*;
pub use interfaces::*;
pub use logging::*;
pub use mocks::*;
// Creates unclear dependencies
```

#### Solution: Explicit Exports
```rust
// src/core/mod.rs
pub mod features;
pub mod interfaces;
pub mod logging;

// Re-export only specific types that should be public
pub use features::FeatureConfig;
pub use interfaces::{DatabaseService, AnalysisEngine};
pub use logging::Logger;

// Don't export everything with *
```

### 6. Create Layered Architecture

#### Define Clear Layers:
```
Presentation Layer (CLI, Web API)
    ↓
Application Layer (Orchestration, Workflows)
    ↓
Domain Layer (Business Logic, Types)
    ↓
Infrastructure Layer (Database, File System, External APIs)
```

#### Enforce Layer Dependencies:
```rust
// src/layers/mod.rs
//! Architectural layers with enforced dependency rules
//!
//! Dependencies must flow downward only:
//! presentation -> application -> domain -> infrastructure

pub mod presentation {
    // Can use: application, domain
    // Cannot use: infrastructure (directly)
}

pub mod application {
    // Can use: domain, infrastructure
    // Cannot use: presentation
}

pub mod domain {
    // Can use: only standard library and external crates
    // Cannot use: application, presentation, infrastructure
}

pub mod infrastructure {
    // Can use: domain
    // Cannot use: application, presentation
}
```

### 7. Implement Dependency Injection

#### Create Service Container:
```rust
// src/container/mod.rs
use std::sync::Arc;

pub struct ServiceContainer {
    database: Arc<dyn DatabaseService>,
    analysis_engine: Arc<dyn AnalysisEngine>,
    report_generator: Arc<dyn ReportGenerator>,
}

impl ServiceContainer {
    pub fn new() -> Self {
        // Wire up dependencies without circular references
        let database = Arc::new(SqliteDatabase::new());
        let analysis_engine = Arc::new(RustAnalysisEngine::new());
        let report_generator = Arc::new(HtmlReportGenerator::new());

        Self {
            database,
            analysis_engine,
            report_generator,
        }
    }
}
```

#### Inject Dependencies:
```rust
// src/application/orchestrator.rs
pub struct AnalysisOrchestrator {
    database: Arc<dyn DatabaseService>,
    engine: Arc<dyn AnalysisEngine>,
    reporter: Arc<dyn ReportGenerator>,
}

impl AnalysisOrchestrator {
    pub fn new(container: &ServiceContainer) -> Self {
        Self {
            database: container.database.clone(),
            engine: container.analysis_engine.clone(),
            reporter: container.report_generator.clone(),
        }
    }
}
```

### 8. Extract Shared Interfaces

#### Create Interface Definitions:
```rust
// src/interfaces/analysis.rs
#[async_trait]
pub trait AnalysisEngine {
    async fn analyze(&self, target: &Path) -> Result<AnalysisResult>;
    fn supported_languages(&self) -> &[Language];
}

// src/interfaces/reporting.rs
#[async_trait]
pub trait ReportGenerator {
    async fn generate(&self, result: &AnalysisResult, format: ReportFormat) -> Result<String>;
    fn supported_formats(&self) -> &[ReportFormat];
}

// src/interfaces/persistence.rs
#[async_trait]
pub trait DatabaseService {
    async fn save(&self, result: &AnalysisResult) -> Result<String>;
    async fn load(&self, id: &str) -> Result<AnalysisResult>;
    async fn list(&self, filter: &Filter) -> Result<Vec<AnalysisMetadata>>;
}
```

### 9. Update Module Structure

#### New Dependency-Safe Structure:
```
src/
├── types/              # Shared types (no dependencies on other modules)
│   ├── analysis.rs
│   ├── report.rs
│   └── config.rs
├── interfaces/         # Trait definitions (depend only on types)
│   ├── analysis.rs
│   ├── reporting.rs
│   └── persistence.rs
├── domain/             # Business logic (depends on types, interfaces)
│   ├── analysis/
│   ├── validation/
│   └── rules/
├── infrastructure/     # External integrations (depends on interfaces)
│   ├── database/
│   ├── filesystem/
│   └── ai/
├── application/        # Orchestration (depends on interfaces)
│   ├── workflows/
│   └── services/
└── presentation/       # User interfaces (depends on application)
    ├── cli/
    └── api/
```

### 10. Validate Dependency Resolution

#### A. Static Analysis:
```bash
# Use cargo-deny to check for circular dependencies
echo '[bans]
deny = [
    { name = "circular-dependency-check" }
]' > deny.toml

cargo deny check bans
```

#### B. Dependency Graphing:
```bash
# Generate new dependency graph after changes
cargo deps --all-deps | dot -Tsvg > new_dependencies.svg

# Compare before and after
diff current_dependencies.svg new_dependencies.svg
```

#### C. Build Order Verification:
```bash
# Verify modules can be built in correct order
modules=("types" "interfaces" "domain" "infrastructure" "application" "presentation")

for module in "${modules[@]}"; do
    echo "Building module: $module"
    cargo check --lib --manifest-path "src/$module/Cargo.toml" || {
        echo "Dependency violation in $module"
        exit 1
    }
done
```

## Success Criteria
- [ ] No circular dependencies detected by static analysis
- [ ] Clear layered architecture with one-way dependencies
- [ ] Dependency injection system working
- [ ] All modules buildable independently
- [ ] Faster compilation due to better dependency resolution
- [ ] All tests pass

## Breaking Changes
- Module import paths may change
- Some internal APIs reorganized
- Dependency injection required for some components

## Verification Commands
```bash
# Check for circular dependencies
cargo deny check

# Generate dependency graph
cargo deps --all-deps | dot -Tsvg > dependencies_final.svg

# Test compilation order
./scripts/test-dependency-order.sh

# Run all tests
cargo test --all-features
```

## Completion Notes
_To be filled by AI developer:_
- Circular dependencies eliminated: ___
- New module structure: ___
- Dependency injection complexity: ___
- Compilation time improvement: ___
- Breaking changes: ___