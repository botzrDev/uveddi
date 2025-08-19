# Uveddi Build Optimization Plan

## Current Problems
- 80+ direct dependencies
- 600+ transitive dependencies from Cargo.lock (8,910 lines)
- Heavy default features: `tree-sitter`, `security`, `memory-optimization`
- Multiple overlapping crypto/HTTP libraries
- Build scripts slowing compilation

## Recommended Solutions

### 1. Create Minimal Development Features

Add to Cargo.toml `[features]` section:

```toml
# Minimal development build (only essential deps)
dev-minimal = ["dep:clap", "dep:serde", "dep:serde_json", "dep:tokio", "dep:anyhow"]

# Single-language development (choose one)
dev-rust-only = ["dev-minimal", "rust-lang"]
dev-python-only = ["dev-minimal", "python-lang"] 
dev-js-only = ["dev-minimal", "javascript-lang"]
dev-ts-only = ["dev-minimal", "typescript-lang"]

# Fast iteration (no heavy analysis)
dev-fast = ["dev-minimal", "dep:tracing", "dep:regex"]

# Core analysis without tree-sitter
dev-core = ["dev-minimal", "dep:petgraph", "dep:walkdir", "dep:ignore"]
```

### 2. Consolidate Overlapping Dependencies

#### Cryptographic Libraries (Currently 8 crates)
**Problem**: `rustls`, `ring`, `argon2`, `blake3`, `sha2`, `md5`, `jsonwebtoken`, `subtle`

**Solution**: 
```toml
# Consolidate crypto behind single feature
crypto-full = ["dep:rustls", "dep:ring", "dep:blake3", "dep:sha2"]
crypto-minimal = ["dep:sha2"] # Only SHA-2 for basic hashing
```

#### HTTP/Web Libraries (Currently 9 crates)
**Problem**: `reqwest`, `axum`, `tower`, `tower-http`, `tower_governor`, `tokio-tungstenite`

**Solution**:
```toml
# Web features
web-client = ["dep:reqwest"]
web-server = ["dep:axum", "dep:tower", "dep:tower-http"] 
web-full = ["web-client", "web-server", "dep:tower_governor", "dep:tokio-tungstenite"]
```

### 3. Optional Heavy Dependencies

Make expensive dependencies truly optional:

```toml
# Memory optimization (currently in default)
memory-optimization = ["mimalloc", "bumpalo", "bumpalo-herd", "memmap2", "rkyv"]

# Security (currently in default) 
security = ["crypto-full", "web-full", "dep:casbin", "dep:oauth2", "dep:vaultrs"]

# Tree-sitter (currently in default)
# Already properly configured but should not be in default for dev
```

### 4. Development Build Profiles

Add to Cargo.toml:

```toml
[profile.dev-fast]
inherits = "dev"
opt-level = 0
debug = false
debug-assertions = false
overflow-checks = false
lto = false
panic = 'unwind'
incremental = true
codegen-units = 16  # Faster parallel compilation
```

### 5. New Default Feature Set

**Current**: `default = ["tree-sitter", "security", "memory-optimization"]`

**Proposed**:
```toml
# Lightweight default for development
default = ["dev-core"] 

# Full production features
production = ["tree-sitter", "security", "memory-optimization", "web-full"]

# Keep existing feature combinations but don't make them default
community = ["tui", "tree-sitter", "local-ai", "security", "memory-optimization"]
alpha = ["community"] # Backward compatibility
```

## Implementation Strategy

### Phase 1: Quick Wins (Immediate)
1. Create `dev-minimal` feature set
2. Change default from heavy features to lightweight
3. Add `profile.dev-fast` 
4. Test build time improvements

### Phase 2: Consolidation (Week 1)
1. Group crypto dependencies behind feature flags
2. Group web dependencies behind feature flags  
3. Make tree-sitter parsing optional by language
4. Update CI to test multiple feature combinations

### Phase 3: Advanced Optimization (Week 2)
1. Evaluate removing redundant dependencies entirely
2. Replace heavy dependencies with lighter alternatives where possible
3. Consider workspace splitting for plugin system
4. Benchmark and document improvements

## Memory Optimization Principles Applied

### Build Pipeline Hierarchy (Based on Memory Hierarchy Research)
```
Compiler Cache (fastest) → Dependency Cache → Source Compilation (slowest)
```

Our optimizations respect this hierarchy:
- **Fewer dependencies** = Better compiler cache utilization
- **Optional features** = Lazy loading of expensive compilation units  
- **Consolidated libs** = Object pooling for similar functionality

### Diagnostic-First Approach
Following the research's "Diagnose First, Act Second" principle:
```bash
# Baseline measurement
time cargo build --features=default  # Current: ~5min

# After optimization  
time cargo build --features=dev-minimal    # Target: ~1min
time cargo build --features=production     # Target: ~3min
```

## Expected Improvements

### Build Time Reduction (Memory-Conscious)
- **Dev builds**: 60-80% faster (from ~5min to ~1-2min)
- **CI builds**: 40-60% faster with cached dependencies
- **Clean builds**: 50-70% faster due to fewer dependencies
- **Memory usage**: 30-50% less RAM during compilation

### Disk Usage Reduction  
- **Target directory**: 40-60% smaller
- **Dependency downloads**: 50-70% fewer crates to download

### Memory Usage
- **Compilation**: 30-50% less RAM during builds
- **Runtime**: Smaller binary size for development builds

## Migration Guide

### For Development
```bash
# Fast development builds
cargo build --features=dev-minimal
cargo build --features=dev-rust-only

# Test specific functionality
cargo build --features="dev-core,rust-lang"
cargo test --features="dev-minimal"
```

### For Production
```bash
# Full featured build (current behavior)
cargo build --release --features=production

# Community edition
cargo build --release --features=community
```

### For CI/CD
Update workflows to test multiple feature combinations:
```yaml
strategy:
  matrix:
    features: 
      - "dev-minimal"
      - "dev-core,rust-lang"  
      - "community"
      - "production"
```

## Breaking Changes

### Minimal Breaking Changes
- Default features change (affects users using default build)
- Some dependencies become optional (affects direct dependency usage)

### Migration Path
1. Document feature flag requirements clearly
2. Provide migration script for common use cases
3. Keep `alpha` feature for backward compatibility
4. Update README with new build instructions

## Dependencies to Consider Removing

### High Impact Removals
1. **Multiple crypto libraries** → Use only `ring` + `rustls`
2. **Arrow ecosystem** (4 crates) → Only if actually needed
3. **WASM runtime** → Only for plugin feature
4. **ML libraries** → Only for advanced analytics feature
5. **Monitoring stack** → Only for production feature

### Low Risk Removals
1. `color-eyre` → Use standard error handling in dev
2. `strum`/`strum_macros` → Replace with manual implementation
3. `pastey` → Evaluate if actually needed
4. Multiple hash libraries → Standardize on one

## Validation Plan

### Build Time Benchmarks
```bash
# Before optimization
time cargo build --features=default

# After optimization  
time cargo build --features=dev-minimal
time cargo build --features=production
```

### Functionality Testing
```bash
# Ensure all analysis features work
cargo test --features=dev-core,rust-lang
cargo test --features=community
cargo test --features=production --release
```

### CI Integration
- Add build time tracking to CI
- Test multiple feature combinations
- Validate no functionality regression