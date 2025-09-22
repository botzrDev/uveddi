# Feature Migration Guide

**Effective Date:** December 2024
**Migration Deadline:** v1.0.0 release (deprecated features will be removed)

## Overview

Uveddi has simplified its feature flag system from 42 features to 22 essential features for better user experience and build performance. This guide helps you migrate from the old feature system to the new streamlined structure.

## Quick Migration Reference

### Profile Features (Build Selection)

| Old Feature | New Feature | Migration Action |
|-------------|-------------|------------------|
| `dev-ultra-minimal` | `minimal` | ✅ **Direct replacement** |
| `dev-minimal` | `minimal` | ✅ **Direct replacement** |
| `dev-core` | `standard` | ✅ **Recommended upgrade** |
| `dev-full` | `standard` | ✅ **Direct replacement** |
| `production` | `full` | ✅ **Direct replacement** |
| `production-secure` | `full` | ⚠️ **Security handled at runtime** |

### Language Features (Unchanged)

| Feature | Status | Notes |
|---------|--------|-------|
| `rust-lang` | ✅ **Kept** | Individual language control |
| `python-lang` | ✅ **Kept** | Individual language control |
| `javascript-lang` | ✅ **Kept** | Individual language control |
| `typescript-lang` | ✅ **Kept** | Individual language control |
| `tree-sitter` | ✅ **Enhanced** | Now uses `languages-all` |

### New Language Packs

| New Feature | Includes | Use Case |
|-------------|----------|----------|
| `languages-core` | `rust-lang` + `python-lang` | Backend development |
| `languages-web` | `javascript-lang` + `typescript-lang` | Frontend development |
| `languages-all` | All language features | Full-stack development |

### Web Features (Simplified)

| Old Feature | New Feature | Migration Action |
|-------------|-------------|------------------|
| `web-client` | `web` | ✅ **Use consolidated feature** |
| `web-server` | `web` | ✅ **Use consolidated feature** |
| `web-full` | `web` | ✅ **Direct replacement** |

### Removed Features

| Removed Feature | Replacement | Migration Action |
|-----------------|-------------|------------------|
| `alpha` | `full` | Use full profile instead |
| `zero-cost` | `standard` | Use standard profile |
| `full-featured` | `full` | Use full profile |
| `enterprise` | `full` | Use full profile |
| `dev-rust-only` | `minimal` + `rust-lang` | Use individual language |
| `dev-python-only` | `minimal` + `python-lang` | Use individual language |
| `dev-js-only` | `minimal` + `javascript-lang` | Use individual language |
| `dev-ts-only` | `minimal` + `typescript-lang` | Use individual language |

## Migration Examples

### Basic Builds

```bash
# OLD → NEW

# Development builds
cargo build --features dev-minimal      # → cargo build --features minimal
cargo build --features dev-core        # → cargo build --features standard
cargo build --features dev-full        # → cargo build --features standard

# Production builds
cargo build --features production      # → cargo build --features full
cargo build --features production-secure # → cargo build --features full
```

### Language-Specific Builds

```bash
# OLD → NEW

# Backend only (Rust + Python)
cargo build --features dev-rust-only   # → cargo build --features "minimal,languages-core"
cargo build --features dev-python-only # → cargo build --features "minimal,languages-core"

# Frontend only (JS + TS)
cargo build --features dev-js-only     # → cargo build --features "minimal,languages-web"
cargo build --features dev-ts-only     # → cargo build --features "minimal,languages-web"

# All languages
cargo build --features dev-full        # → cargo build --features standard
```

### Web Features

```bash
# OLD → NEW

# Web functionality
cargo build --features web-client      # → cargo build --features web
cargo build --features web-server      # → cargo build --features web
cargo build --features web-full        # → cargo build --features web

# With other features
cargo build --features "dev-core,web-full" # → cargo build --features "standard,web"
```

### Capability Combinations

```bash
# OLD → NEW

# Security-focused build
cargo build --features "production,security"     # → cargo build --features full

# Memory-optimized build
cargo build --features "dev-core,memory-optimization" # → cargo build --features "standard,memory-optimization"

# TUI development
cargo build --features "dev-core,tui"           # → cargo build --features "standard,tui"
```

## CI/CD Migration

### GitHub Actions

```yaml
# OLD workflow
jobs:
  test:
    strategy:
      matrix:
        features: [dev-minimal, dev-core, production]
    steps:
      - run: cargo test --features ${{ matrix.features }}

# NEW workflow
jobs:
  test:
    strategy:
      matrix:
        features: [minimal, standard, full]
    steps:
      - run: cargo test --features ${{ matrix.features }}
```

### Docker Builds

```dockerfile
# OLD Dockerfile
RUN cargo build --release --features production

# NEW Dockerfile
RUN cargo build --release --features full
```

## Common Use Cases

### Development Workflow

```bash
# Fast iteration (no parsing)
cargo build --features minimal

# Standard development (with parsing)
cargo build --features standard

# Full local testing
cargo build --features full
```

### Language-Specific Projects

```bash
# Rust-only project
cargo build --features "minimal,rust-lang"

# Python data analysis
cargo build --features "minimal,languages-core"

# React frontend
cargo build --features "minimal,languages-web"

# Full-stack application
cargo build --features standard  # includes all languages
```

### Production Deployment

```bash
# Secure production build
cargo build --release --features full

# With specific capabilities
cargo build --release --features "full,memory-optimization"

# Without security (if needed)
cargo build --release --features "standard,tui,wasm-plugins"
```

## Feature Gate Code Changes

If you've written code using feature gates, most should work unchanged:

### No Changes Needed ✅

```rust
#[cfg(feature = "security")]        // ✅ Still works
#[cfg(feature = "wasm-plugins")]    // ✅ Still works
#[cfg(feature = "ai")]              // ✅ Still works
#[cfg(feature = "tree-sitter")]     // ✅ Still works
#[cfg(feature = "rust-lang")]       // ✅ Still works
```

### Updates Required ⚠️

```rust
// OLD - Don't use profile features in code
#[cfg(feature = "dev-core")]        // ❌ Deprecated
#[cfg(feature = "production")]      // ❌ Deprecated

// NEW - Use capability features instead
#[cfg(feature = "security")]        // ✅ Correct approach
#[cfg(feature = "prometheus")]      // ✅ Correct approach
```

## Deprecation Timeline

| Version | Status | Action |
|---------|--------|--------|
| **v0.9.x** | ⚠️ **Deprecated** | Old features work but show warnings |
| **v0.10.x** | ⚠️ **Last Support** | Old features supported, migration recommended |
| **v1.0.0** | ❌ **Removed** | Old features removed, must use new system |

## Getting Help

### Validation Commands

Check your current feature usage:
```bash
# List all features in your project
grep -r "features.*=" Cargo.toml

# Check for deprecated features
grep -r "dev-\|production" .github/workflows/
```

### Migration Verification

Test new features work:
```bash
# Test new profiles
cargo check --features minimal
cargo check --features standard
cargo check --features full

# Test language packs
cargo check --features "minimal,languages-core"
cargo check --features "minimal,languages-web"
```

### Support Resources

- **Documentation:** `docs/user-guide/feature-flags.md`
- **Examples:** `docs/examples/`
- **Issues:** Report migration problems on GitHub
- **Community:** Discussion in GitHub Discussions

## Summary

The new feature system provides:
- ✅ **Simpler selection:** 3 clear profiles instead of 6 dev variants
- ✅ **Better performance:** Optimized dependency resolution
- ✅ **Clearer intent:** Language packs express developer focus
- ✅ **Maintained flexibility:** All capabilities still available

Most migrations are straightforward replacements. The new system is more intuitive and provides better build performance while maintaining all functionality.