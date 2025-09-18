# Assignment 11: Create Feature Profiles

## Priority: HIGH
## Estimated Time: 3-4 hours
## Dependencies: Assignment 10 (Eliminate Unused Features)

## Objective
Replace complex feature flag combinations with simple, user-friendly profiles.

## Current Problem
After elimination, still have complex feature interdependencies. Users need simple profiles for common use cases.

## Tasks

### 1. Design Profile Structure

Based on user personas and use cases:

```toml
[features]
# User-facing profiles (primary interface)
default = ["standard"]
minimal = ["core-analysis"]
standard = ["minimal", "reporting", "local-cache"]
full = ["standard", "ai-integration", "web-dashboard", "all-languages"]
enterprise = ["full", "security", "monitoring", "advanced-caching"]

# Language support (modular)
rust-lang = ["tree-sitter-rust"]
python-lang = ["tree-sitter-python"]
js-lang = ["tree-sitter-javascript"]
ts-lang = ["tree-sitter-typescript"]
web-langs = ["js-lang", "ts-lang"]
all-languages = ["rust-lang", "python-lang", "web-langs"]

# Integration modules (can be mixed with profiles)
ai-integration = ["ollama-api", "openai-api"]
web-dashboard = ["api-server", "websockets", "frontend-assets"]
security = ["authentication", "authorization", "audit-logging"]
monitoring = ["prometheus", "health-checks", "metrics"]

# Internal implementation features (not user-facing)
core-analysis = ["ast-parsing", "detector-engine"]
reporting = ["html-reports", "json-export", "markdown-export"]
local-cache = ["file-cache", "memory-cache"]
advanced-caching = ["local-cache", "redis-cache", "cache-clustering"]
```

### 2. Implement Profile Hierarchy

#### Core Principle: Additive Profiles
- Each profile builds on previous level
- Users can mix profiles with specific integrations
- Clear upgrade path from minimal -> standard -> full

#### Dependency Chain:
```
minimal
  ↓
standard (+ reporting, caching)
  ↓
full (+ AI, web dashboard, all languages)
  ↓
enterprise (+ security, monitoring, advanced features)
```

### 3. Create Profile Definitions

#### Minimal Profile
**Target Users**: CI/CD systems, embedded analysis
**Features**: Core analysis only, minimal dependencies
```toml
minimal = [
    "core-analysis",
    "rust-lang",  # Always include Rust support
    "json-export"  # Minimal reporting
]
```

#### Standard Profile
**Target Users**: Individual developers, small teams
**Features**: Full local analysis with reporting
```toml
standard = [
    "minimal",
    "reporting",
    "local-cache",
    "python-lang",  # Common second language
    "html-reports"
]
```

#### Full Profile
**Target Users**: Development teams, larger projects
**Features**: All analysis features, AI integration
```toml
full = [
    "standard",
    "ai-integration",
    "web-dashboard",
    "all-languages",
    "advanced-reporting"
]
```

#### Enterprise Profile
**Target Users**: Organizations, enterprise deployments
**Features**: Security, monitoring, advanced deployment features
```totml
enterprise = [
    "full",
    "security",
    "monitoring",
    "advanced-caching",
    "audit-logging",
    "multi-tenant"
]
```

### 4. Refactor Existing Feature Flags

#### Map Current Features to New Structure:
```bash
# Create mapping file: feature_migration_map.md
# For each current feature, document:
# - Which profile(s) it belongs to
# - Whether it becomes internal implementation detail
# - If it needs renaming for clarity
```

#### Example Migration:
```toml
# Old complex features:
dev-core = ["dep:tokio-stream", "dep:async-stream", "prometheus"]
tree-sitter = ["tree-sitter-rust", "tree-sitter-python", ...]

# New profile-based features:
standard = ["core-analysis", "rust-lang", "python-lang", "reporting"]
monitoring = ["prometheus", "health-checks"]
```

### 5. Update Conditional Compilation

#### Replace Complex Feature Checks:
```rust
// Old: Complex feature combinations
#[cfg(all(feature = "dev-core", feature = "tree-sitter"))]

// New: Profile-based checks
#[cfg(feature = "standard")]

// Or specific feature checks for internal implementation
#[cfg(feature = "rust-lang")]
```

#### Create Feature Compatibility Layer:
```rust
// In src/features/mod.rs
#[cfg(feature = "standard")]
pub const RUST_SUPPORT: bool = true;

#[cfg(feature = "standard")]
pub const REPORTING: bool = true;

#[cfg(feature = "minimal")]
pub const AI_INTEGRATION: bool = false;

#[cfg(feature = "full")]
pub const AI_INTEGRATION: bool = true;
```

### 6. Update Build System

#### Simplify CI/CD Matrix:
```yaml
# .github/workflows/test.yml
strategy:
  matrix:
    profile: [minimal, standard, full]
    os: [ubuntu-latest, windows-latest, macos-latest]

steps:
  - name: Test profile
    run: cargo test --features ${{ matrix.profile }}
```

#### Create Profile Validation:
```bash
# scripts/validate-profiles.sh
#!/bin/bash
set -e

echo "Validating feature profiles..."

profiles=("minimal" "standard" "full" "enterprise")

for profile in "${profiles[@]}"; do
    echo "Testing profile: $profile"
    cargo check --features "$profile"
    cargo test --features "$profile" --lib
done

echo "All profiles validated successfully!"
```

### 7. Create Profile Documentation

#### User Guide:
```markdown
# Uveddi Feature Profiles

## Quick Start
- **Minimal**: `cargo install --features minimal` - Basic analysis only
- **Standard**: `cargo install --features standard` - Recommended for most users
- **Full**: `cargo install --features full` - Complete feature set
- **Enterprise**: `cargo install --features enterprise` - Organizational deployments

## Custom Combinations
```bash
# Standard analysis with AI but no web dashboard
cargo build --features "standard,ai-integration"

# Minimal with specific language support
cargo build --features "minimal,python-lang,js-lang"
```

#### Migration Guide:
```markdown
# Migration from Complex Features

## Common Migration Patterns:
- `--features dev-core` → `--features standard`
- `--features production` → `--features full`
- `--features tree-sitter` → `--features all-languages`
```

### 8. Backward Compatibility

#### Create Temporary Aliases:
```toml
# Temporary backward compatibility (mark as deprecated)
dev-core = ["standard"]        # Deprecated: use 'standard'
dev-minimal = ["minimal"]      # Deprecated: use 'minimal'
production = ["full"]          # Deprecated: use 'full'
tree-sitter = ["all-languages"] # Deprecated: use 'all-languages'
```

#### Add Deprecation Warnings:
```rust
#[cfg(feature = "dev-core")]
compile_error!("Feature 'dev-core' is deprecated. Use 'standard' instead.");

// Or for softer transition:
#[cfg(feature = "dev-core")]
const _: () = {
    #[deprecated(note = "Feature 'dev-core' is deprecated. Use 'standard' instead.")]
    const DEPRECATED_FEATURE: () = ();
    let _ = DEPRECATED_FEATURE;
};
```

### 9. Update Documentation

#### README.md:
```markdown
## Installation

### Feature Profiles
Choose the profile that best fits your needs:

- **Minimal**: `cargo install uveddi --features minimal`
- **Standard**: `cargo install uveddi` (default)
- **Full**: `cargo install uveddi --features full`
- **Enterprise**: `cargo install uveddi --features enterprise`

### Language Support
Add specific languages as needed:
```bash
cargo install uveddi --features "standard,python-lang,js-lang"
```

#### Cargo.toml:
```toml
# Add helpful comments
[features]
# ===============================
# User-Facing Feature Profiles
# ===============================
# Choose one primary profile:

default = ["standard"]           # Recommended for most users
minimal = [...]                 # CI/CD and embedded use
standard = [...]                # Individual developers
full = [...]                    # Teams with AI/web features
enterprise = [...]              # Enterprise deployments

# ===============================
# Optional Feature Modules
# ===============================
# Mix with profiles as needed:

# Language support
rust-lang = [...]
python-lang = [...]
# ... etc
```

### 10. Validation and Testing

#### Profile Compatibility Matrix:
```bash
# Test all profile combinations
profiles=("minimal" "standard" "full" "enterprise")
languages=("rust-lang" "python-lang" "js-lang")

for profile in "${profiles[@]}"; do
    for lang in "${languages[@]}"; do
        echo "Testing: $profile + $lang"
        cargo check --features "$profile,$lang"
    done
done
```

#### Performance Benchmarking:
```bash
# Compare build times and binary sizes
for profile in minimal standard full enterprise; do
    echo "Benchmarking profile: $profile"
    time cargo build --release --features "$profile"
    ls -lh target/release/uveddi
done
```

## Success Criteria
- [ ] 4-5 clear user-facing profiles defined
- [ ] Profile hierarchy working (additive)
- [ ] All existing functionality accessible via profiles
- [ ] CI/CD simplified to test profiles
- [ ] Documentation updated with clear usage examples
- [ ] Backward compatibility maintained
- [ ] Build time and complexity reduced

## Breaking Changes
Document any breaking changes:
- Feature flag names changed
- Default behavior modifications
- Removed feature combinations

## Verification Commands
```bash
# Test all profiles build successfully
cargo check --features minimal
cargo check --features standard
cargo check --features full
cargo check --features enterprise

# Test profile combinations
cargo check --features "standard,ai-integration"
cargo check --features "minimal,python-lang"

# Verify backward compatibility aliases work
cargo check --features dev-core
cargo check --features production
```

## Completion Notes
_To be filled by AI developer:_
- Profiles created: ___
- Feature flags reduced from ___ to ___
- Build matrix simplified from ___ to ___ combinations
- Backward compatibility aliases: ___
- Documentation updates: ___