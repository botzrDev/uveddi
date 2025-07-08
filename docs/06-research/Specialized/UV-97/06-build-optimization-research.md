# GPT Research Prompt: Build Optimization Research for UV-97

## Context
You are researching build optimization strategies for UV-97 (tree-sitter feature gating) implementation in the Uveddi Rust codebase. The current development workflow involves frequent edit-compile-test cycles during the refactoring process, and optimization of this cycle could significantly reduce total implementation time.

**Current Development Challenges:**
- Large codebase with many dependencies takes time to compile
- Feature flag changes require full recompilation  
- Multiple compilation configurations need testing (`--features tree-sitter`, `--no-default-features`)
- Cascading errors require multiple fix-compile cycles
- Integration tests may be slow to run repeatedly

**Current Project State:**
- Implementing canonical shim pattern with conditional compilation
- Multiple files being modified simultaneously
- Need to test both feature-enabled and feature-disabled builds frequently
- Want to minimize time between making changes and getting feedback

## Research Prompt

**Your task:** Research and design an optimal development workflow for feature flag implementation that minimizes edit-compile-test cycle time while ensuring correct implementation across both feature configurations.

### Step 1: Compilation Performance Analysis

Analyze current build performance characteristics:

#### A. Baseline Build Times
```bash
# Measure current compilation times for different configurations
time cargo check
time cargo check --no-default-features  
time cargo check --features tree-sitter
time cargo test --no-default-features --lib
time cargo test --features tree-sitter --lib
```

#### B. Incremental Compilation Impact
```bash
# Test incremental compilation effectiveness
# After making a small change to a gated file:
time cargo check  # Second run
time cargo check --no-default-features  # Second run

# After changing feature flags:
time cargo check --features tree-sitter  # After --no-default-features
```

#### C. Dependency Compilation Analysis
```bash
# Identify which dependencies take longest to compile
cargo build --timings
# Analyze if tree-sitter dependencies significantly impact build time
```

### Step 2: Workflow Strategy Analysis

Evaluate different development approaches:

#### A. Sequential Approach (Current)
```bash
# Current workflow:
1. Edit files
2. cargo check --no-default-features
3. Fix errors
4. cargo check --features tree-sitter  
5. Fix errors
6. Repeat
```
**Pros:** Thorough, catches all issues
**Cons:** Slow, redundant compilation

#### B. Feature-First Approach
```bash
# Alternative workflow:
1. Edit files
2. cargo check --features tree-sitter  # Full functionality first
3. Fix errors until it compiles
4. cargo check --no-default-features   # Then test stubs
5. Fix feature gating issues
```

#### C. Parallel Development Approach
```bash
# Alternative workflow:
1. Developer A: Focus on --features tree-sitter compilation
2. Developer B: Focus on --no-default-features compilation  
3. Coordinate on API compatibility
```

#### D. Hybrid Approach
```bash
# Optimized workflow:
1. Quick syntax check: cargo check --features tree-sitter
2. Batch fix obvious errors
3. Test both configurations: cargo check --no-default-features
4. Address feature-specific issues
```

### Step 3: Tool and Configuration Optimization

Research Rust tooling optimizations:

#### A. Cargo Configuration Options
```toml
# Evaluate .cargo/config.toml optimizations:
[build]
rustc-wrapper = "sccache"    # Compilation caching
incremental = true           # Incremental compilation
pipelining = true           # Pipeline compilation

[target.x86_64-unknown-linux-gnu]
linker = "lld"              # Faster linking

# Development vs release profiles
[profile.dev]
opt-level = 0               # No optimization for faster builds
debug = 1                   # Reduced debug info
incremental = true
```

#### B. Feature Flag Development Strategy
```toml
# Evaluate if development-specific feature combinations help:
[features]
default = ["tree-sitter"]
tree-sitter = ["dep:tree-sitter", "dep:tree-sitter-rust", "dep:tree-sitter-python", "dep:tree-sitter-javascript"]

# Development helper features?
dev-fast = []               # Skip slow initialization?
dev-stub-only = []          # Force stub mode for faster builds?
```

#### C. Compilation Target Optimization
```bash
# Research if limiting compilation scope helps:
cargo check --lib                    # Library only, skip binaries
cargo check --workspace --exclude example_crate  # Skip examples
cargo check --package core_package   # Single package only
```

### Step 4: Testing Strategy Optimization

Optimize the test feedback loop:

#### A. Test Execution Strategy
```bash
# Fast feedback options:
cargo test --lib --no-default-features                    # Unit tests only
cargo test --package core --features tree-sitter         # Core package only  
cargo test test_name --features tree-sitter              # Single test
cargo nextest run --features tree-sitter                 # Parallel test runner
```

#### B. Test Selection Strategy
```bash
# Targeted testing during development:
cargo test --features tree-sitter -- detector            # Only detector tests
cargo test --no-default-features -- stub                 # Only stub tests
cargo test --lib --features tree-sitter ast              # Only AST-related tests
```

#### C. CI/Local Development Split
```yaml
# What should run locally vs CI?
Local (fast feedback):
  - cargo check both feature configurations
  - Unit tests for modified modules only
  
CI (comprehensive):
  - Full test suite both configurations
  - Integration tests
  - Documentation tests
  - Benchmarks
```

### Step 5: Error Handling Strategy Optimization

Optimize the fix-error cycle:

#### A. Error Prioritization
```bash
# Research: Should we fix errors in a specific order?
# Option 1: Fix all --features tree-sitter errors first (easier debugging)
# Option 2: Fix blocking errors first (faster progress) 
# Option 3: Fix errors by file (better organization)
```

#### B. Compilation Error Batching
```bash
# Research: How to get maximum error information per compilation:
cargo check --message-format=json 2>&1 | jq '.message.message' # Parse errors
RUSTFLAGS="--cap-lints=warn" cargo check                        # Continue past errors
```

#### C. IDE Integration Optimization
```bash
# Research IDE-specific optimizations:
# rust-analyzer configuration for feature flags
# VS Code settings for faster feedback
# CLI tools for rapid iteration
```

## Expected Output Format

### Workflow Optimization Strategy

```markdown
## Recommended Development Workflow for UV-97

### Optimized Edit-Compile-Test Cycle

#### Phase 1: Initial Setup (One Time)
```bash
# Configure development environment for fast builds
cargo install sccache        # Compilation caching
cargo install cargo-nextest  # Faster test runner

# Configure .cargo/config.toml:
[build]
rustc-wrapper = "sccache"
incremental = true
```

#### Phase 2: Development Loop (Repeated)
```bash
# Optimized cycle (estimated times):
1. Edit files                                    # 0 seconds
2. cargo check --features tree-sitter          # 15 seconds (incremental)
3. Fix compilation errors                        # Variable
4. cargo check --no-default-features           # 10 seconds (incremental)  
5. Fix feature gating issues                     # Variable
6. cargo test --lib --features tree-sitter     # 30 seconds (targeted)
```

**Total cycle time:** ~1 minute vs current ~3-5 minutes

### Build Performance Optimizations
| Optimization | Time Saved | Implementation Effort |
|--------------|------------|----------------------|
| sccache compilation caching | 40-60% | 5 minutes setup |
| Incremental compilation | 30-50% | Built-in |
| Feature-first workflow | 20-30% | Change habits |
| Targeted test execution | 50-70% | Learn new commands |
| Parallel development | 50%+ | Team coordination |
```

### Tool Configuration Recommendations

```markdown
## Recommended .cargo/config.toml
```toml
[build]
rustc-wrapper = "sccache"
incremental = true
pipelining = true

[profile.dev]
opt-level = 0
debug = 1
incremental = true
split-debuginfo = "unpacked"  # Faster debug builds on macOS/Windows

# Platform-specific optimizations
[target.x86_64-unknown-linux-gnu]
linker = "lld"

[target.x86_64-pc-windows-msvc]  
linker = "lld-link"

[target.x86_64-apple-darwin]
rustflags = ["-C", "link-arg=-fuse-ld=lld"]
```

## Recommended Development Commands
```bash
# Fast syntax/type checking
alias check-full="cargo check --features tree-sitter"
alias check-stub="cargo check --no-default-features"
alias check-both="check-full && check-stub"

# Fast testing  
alias test-fast="cargo nextest run --lib --features tree-sitter"
alias test-stub="cargo nextest run --lib --no-default-features"

# Error analysis
alias errors="cargo check --message-format=json 2>&1 | jq -r '.message.message' | grep -v null"
```
```

### Workflow Decision Matrix

```markdown
## Development Approach Selection

### For Solo Development:
**Recommended:** Feature-First Approach
1. Implement full tree-sitter functionality until it compiles
2. Then add feature gating for stub compatibility
3. **Rationale:** Easier to debug full functionality than stubs

### For Team Development:
**Recommended:** Parallel Development Approach  
1. Developer A: Focus on tree-sitter implementation completion
2. Developer B: Focus on stub implementation and feature gating
3. **Coordination:** Daily sync on API compatibility requirements

### For Large Refactors:
**Recommended:** Hybrid Approach
1. Use fastest possible compilation check for syntax issues
2. Batch similar fixes together (all import gating, all API fixes)
3. Test both configurations only after batches complete
```

### Success Metrics

```markdown
## Build Optimization Success Criteria

### Timing Targets:
- [ ] Edit-to-feedback cycle: <60 seconds (vs current 3-5 minutes)
- [ ] Full clean build: <2 minutes (with sccache populated)
- [ ] Incremental build after small change: <15 seconds
- [ ] Test feedback: <30 seconds for targeted tests

### Developer Experience:
- [ ] No more than 3 compilation cycles per logical change
- [ ] Clear error messages help identify next steps
- [ ] Can test both feature configurations efficiently
- [ ] Fast enough to encourage frequent testing

### Quality Maintenance:
- [ ] Both feature configurations tested before commits
- [ ] No reduction in test coverage
- [ ] No compromise on error detection quality
```

This research will optimize the development process to minimize the total time needed to complete UV-97 while maintaining code quality.
