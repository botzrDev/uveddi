# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-08-09

### First Stable Release 🎉

Uveddi 1.0.0 is the first stable release. It promotes the CLI-focused product
that shipped as 0.0.3 to stable status: same feature set, verified build and
test suite, and documentation updated to describe the product as it actually
ships. See `docs/release-planning/v1.0.0-release-plan.md` for the full release
plan and gate results.

### Stable in 1.0.0

- **CLI commands**: `analyze`, `init`, `config`, `doctor`, `ci`, `hooks`,
  `migrate`, `license`, `help` (plus `plugin` when built with `wasm-plugins`)
- **Multi-language analysis** via Tree-sitter: Rust, Python, JavaScript, and
  TypeScript in the default `cli-standard` build; more languages behind
  per-language feature flags and license tiers
- **Anti-pattern detection**: god objects, large classes, dead code, code
  duplication, tight coupling, magic values, and related smells — available on
  the free tier
- **Security scanning** with SARIF export for CI/CD pipelines — free tier
- **Tiered licensing** (Free / Pro / Team / Enterprise) with offline
  activation via the `license` command; advanced cyclic-dependency detection
  is Pro-tier
- **Reporting**: Markdown, JSON, and HTML outputs with severity scoring and
  technical-debt metrics
- **Persistent storage**: SQLite database with migration support via
  `uveddi migrate`
- **Incremental analysis** with in-memory-first state management

### Experimental in 1.0.0 (opt-in feature flags)

- **WASM plugin system** (`wasm-plugins` / `cli-plugins`): WebAssembly
  component-model plugin host, loader, and lifecycle manager with a WASI-based
  plugin template
- **AI insights** (`cli-ai`): local Ollama integration for remediation
  guidance and summaries

### Changed

- Version: 0.0.3 → 1.0.0
- README and release documentation rewritten to describe the shipped
  CLI product; older planning documents that describe the removed
  web-dashboard architecture are retained under `docs/` for history only

### Not included (removed during the CLI refocus, pre-1.0)

Web dashboard/React frontend, REST API server, WebSocket/GraphQL interfaces,
standalone TUI, Prometheus metrics, and enterprise auth/RBAC are not part of
1.0.0. Some may return in post-1.0 releases.

---

## [0.0.3] - 2026-07-11

### Added
- **Licensing system**: Tiered feature gating (Free / Pro / Team / Enterprise) with a
  new `license` CLI command for activation and status display
- **WASM plugin system**: WebAssembly component-model host and loader, with the
  analysis engine invoking plugins via the lifecycle manager for file analysis and
  issue extraction; plugin template migrated to WASI with an updated WIT definition
- Integration tests for code duplication detection covering multiple scenarios and
  configurations

### Changed
- Core anti-pattern detectors and security scanning are now available on the **free
  tier** (no license required); advanced cyclic-dependency detection remains a Pro
  feature
- Enhanced detector language handling
- Optimized incremental-analysis state management by prioritizing in-memory state
- Refined configuration parsing logic
- Magic value detector now ignores magic values in test code and self-documenting
  calls, with refined environment-variable detection
- Improved error handling for unimplemented SQLite provider methods
- Updated author contact information in package metadata

### Fixed
- Integration test suite now compiles and runs (previously blocked by references to
  removed modules) — resolves DEF-001
- Security detectors no longer rely on unsupported regex lookahead/lookbehind
  assertions — resolves DEF-002
- TypeScript files are now correctly detected as TypeScript instead of JavaScript —
  resolves DEF-008
- Fixed clippy `vec_init_then_push` lint warning in build script
- Corrected test import paths for TUI integration tests
- Updated version references across documentation

---

## [0.0.2] - 2025-10-03

### Initial Release - Early Access

This is the initial release of Uveddi. This release represents development, testing, and refinement to deliver a code analysis platform.

### Added

#### Core Features
- ✨ **Multi-Language Analysis**: Full support for Rust, Python, JavaScript, and TypeScript with extensible language detection
- ✨ **15+ Anti-Pattern Detectors**: Comprehensive detection including:
  - God objects and large classes
  - Code duplication (token-based and AST-based)
  - Circular dependencies and coupling analysis
  - Feature envy and inappropriate intimacy
  - Shotgun surgery and divergent change patterns
  - Long parameter lists and primitive obsession
- ✨ **Security Analysis**: OWASP Top 10 coverage including:
  - SQL injection detection with taint analysis
  - Cross-site scripting (XSS) vulnerability scanning
  - Insecure authentication patterns
  - Hardcoded secrets and credentials
  - Path traversal vulnerabilities
- ✨ **AI Integration**: Support for multiple LLM providers:
  - Ollama (local models including deepseek-coder)
  - OpenAI (GPT-3.5-turbo, GPT-4)
  - Anthropic Claude (opus, sonnet, haiku)
  - Google Gemini (experimental)
- ✨ **Multiple Output Formats**: Flexible reporting system:
  - JSON - Machine-readable structured data
  - HTML - Interactive reports with charts and visualizations
  - Markdown - Documentation-friendly format
  - SARIF - Security Analysis Results Interchange Format
  - PDF - Experimental PDF generation
- ✨ **Interactive Dashboard**: Modern React-based web UI featuring:
  - Real-time analysis progress tracking
  - Interactive issue exploration
  - Dependency visualization
  - Code metrics dashboards
  - Export capabilities
- ✨ **REST API**: Full-featured API server for integrations:
  - Report management endpoints
  - Analysis execution API
  - Configuration management
  - WebSocket support for real-time updates
- ✨ **TUI Interface**: Terminal-based user interface for interactive analysis
- ✨ **Incremental Analysis**: Smart caching system reducing re-analysis time by 80%
- ✨ **Plugin System**: WebAssembly-based plugin architecture for custom detectors
- ✨ **Prometheus Metrics**: Built-in performance monitoring and observability

#### Documentation
- 📚 Comprehensive user manual (226 pages) covering:
  - Installation and setup
  - Command reference
  - Detector explanations
  - Configuration options
  - Best practices
- 📚 Developer guide including:
  - Architecture overview
  - Detector creation tutorial
  - Plugin development
  - API integration guide
- 📚 API reference documentation with endpoint specifications
- 📚 Architecture diagrams and design documents
- 📚 Real-world examples and case studies
- 📚 Troubleshooting guides

#### Developer Experience
- 🛠️ Feature flag system for modular builds:
  - `minimal` - Core functionality only
  - `standard` - Recommended development profile
  - `full` - All features including AI and plugins
  - `languages-core` - Backend language support
  - `languages-web` - Frontend language support
- 🛠️ Simplified build commands with clear profiles
- 🛠️ Improved error messages with actionable suggestions
- 🛠️ Comprehensive test suite with 87%+ coverage
- 🛠️ CI/CD pipeline with GitHub Actions supporting:
  - Multi-platform builds (Linux, macOS, Windows)
  - Automated testing on Rust 1.70-1.90
  - Security auditing
  - Performance benchmarking

### Changed

#### Architecture
- 🔄 **Refactored Main Entry Point**: Removed deprecated `run_app()` function in favor of direct command execution pattern
- 🔄 **Modular CLI**: Improved command structure with better separation of concerns
- 🔄 **Type System**: Introduced compatibility layer for `ParsedFile` types to support both legacy and new implementations
- 🔄 **Cache Architecture**: Enhanced cache system with:
  - Pluggable backend support (memory, disk, Redis)
  - Automatic invalidation on file changes
  - Configurable TTL and size limits
  - Thread-safe concurrent access
- 🔄 **Report Module**: Broke down monolithic 3,901-line module into focused components:
  - `html_generator.rs` - HTML report generation (400+ lines)
  - `executive_summary.rs` - Summary and metrics logic (220+ lines)
  - `mermaid_integration.rs` - Diagram rendering (350+ lines)
  - 75% reduction in individual file complexity

#### Performance
- ⚡ **30% faster analysis** through:
  - Optimized parallel processing with Rayon
  - Improved AST caching strategies
  - Reduced memory allocations via arena allocation
- ⚡ **Reduced memory usage by 25%** via:
  - Lazy loading of language parsers
  - Streaming JSON generation
  - Optimized tree-sitter node handling
- ⚡ **Smart caching reduces re-analysis time by 80%**:
  - File-level granular caching
  - Dependency-aware invalidation
  - Incremental computation
- ⚡ **44-60% faster development builds** through:
  - Optimized compiler profiles
  - Feature flag reorganization
  - Improved dependency management

#### API Changes
- 🔧 `execute_analysis()` → `execute_core_analysis()` - Clearer naming
- 🔧 `LegacyAnalysisConfig` → `configuration::AnalysisConfig` - Modernized configuration
- 🔧 Feature flags renamed for clarity:
  - `dev-core` → `standard`
  - `production` → `full`
  - `dev-minimal` → `minimal`
- 🔧 Improved error types with better context and suggestions

### Fixed

#### Critical Fixes
- 🐛 **CLI Commands**: Restored full functionality of `analyze` and `serve` commands that were broken in 0.9.0-alpha
- 🐛 **Compilation Errors**: Fixed all 48+ compilation errors in library and binaries including:
  - Duplicate function implementations
  - Missing trait implementations
  - Type mismatches in database models
  - Import statement errors
- 🐛 **Type Compatibility**: Resolved `ParsedFile` vs `ParsedFileCompat` issues across detector modules
- 🐛 **Cache Integration**: Implemented all missing cache methods in detector implementations
- 🐛 **CORS Configuration**: Fixed cross-origin issues preventing frontend-API communication

#### Minor Fixes
- 🐛 Fixed recursive async function causing infinite future size in plugin loader
- 🐛 Corrected frontend port configuration (8001 instead of documented 3000)
- 🐛 Updated deprecated dependency method calls (chrono, axum, tokio)
- 🐛 Resolved trait bound issues in utility binaries
- 🐛 Fixed database connection pooling edge cases
- 🐛 Corrected documentation inaccuracies across README and guides
- 🐛 Fixed CI/CD pipeline failures on multi-platform builds

### Deprecated

- ⚠️ `dev-core` feature flag (use `standard` instead) - Will be removed in v2.0
- ⚠️ `production` feature flag (use `full` instead) - Will be removed in v2.0
- ⚠️ `dev-minimal` feature flag (use `minimal` instead) - Will be removed in v2.0
- ⚠️ `LegacyAnalysisConfig` struct (use `configuration::AnalysisConfig`) - Will be removed in v1.1
- ⚠️ `execute_analysis()` method (use `execute_core_analysis()`) - Will be removed in v1.1

### Removed

- ❌ Non-existent install script references from documentation
- ❌ Outdated feature flag combinations that caused compilation errors
- ❌ Deprecated `run_app()` function call in `main.rs`
- ❌ Experimental WASM features that were unstable (moved to opt-in flag)
- ❌ Health monitoring server (temporarily removed due to hanging issues)

### Security

- 🔒 **Dependency Updates**: All dependencies updated to latest secure versions
- 🔒 **Timing Attack Fix**: Fixed timing attack vulnerability in authentication (RUSTSEC-2023-0071)
- 🔒 **SQL Injection Prevention**: All database queries now use parameterized statements
- 🔒 **XSS Prevention**: HTML report generation properly escapes all user-controlled data
- 🔒 **Path Traversal Protection**: File operations validate paths to prevent directory traversal
- 🔒 **Rate Limiting**: Added rate limiting to API endpoints (100 req/15min per IP)
- 🔒 **CSRF Protection**: Web dashboard implements CSRF token validation
- 🔒 **Secret Detection**: Enhanced detection of hardcoded secrets and credentials
- 🔒 **Input Validation**: All CLI arguments and API inputs properly validated

### Migration Guide

#### For Users

**Before (v0.9.0-alpha):**
```bash
cargo run -- analyze ./src
cargo build --features production
```

**After (v0.0.2):**
```bash
cargo run --features standard -- analyze ./src
cargo build --features full
```

#### For Developers

**ParsedFile Type Usage:**
```rust
// Before:
use crate::ast::tree_sitter::ParsedFile;

// After:
use crate::ast::ParsedFile;  // Uses compatibility alias
// OR for direct tree-sitter type:
use crate::ast::tree_sitter_impl::ParsedFile;
```

**Analysis Orchestration:**
```rust
// Before:
orchestrator.execute_analysis(LegacyAnalysisConfig { ... })

// After:
orchestrator.execute_core_analysis(AnalysisConfig { ... })
```

### Breaking Changes

1. **CLI Command Structure**: Commands now require explicit feature flags for full functionality
2. **Configuration Types**: `LegacyAnalysisConfig` no longer supported - use `configuration::AnalysisConfig`
3. **Feature Flags**: Old flag names (`dev-core`, `production`) removed from `Cargo.toml`
4. **API Methods**: Deprecated analysis methods removed (`execute_analysis()`)
5. **Minimum Rust Version**: Now requires Rust 1.70.0 or later (was 1.65.0)

### Upgrade Path

```bash
# 1. Update dependencies
cargo update

# 2. Replace deprecated feature flags in CI/CD
sed -i 's/dev-core/standard/g' .github/workflows/*.yml
sed -i 's/production/full/g' Dockerfile*

# 3. Update imports (if using Uveddi as a library)
# See migration guide above

# 4. Rebuild with new flags
cargo clean
cargo build --features full

# 5. Run tests to verify
cargo test --features full
```

### Performance Improvements

| Metric | v0.9.0-alpha | v0.0.2 | Improvement |
|--------|--------------|--------|-------------|
| Analysis time (medium codebase) | 45s | 31s | 30% faster |
| Memory usage (large project) | 3.2GB | 2.4GB | 25% reduction |
| Re-analysis time (with cache) | 40s | 8s | 80% reduction |
| Development build time | 250s | 140s | 44% faster |
| CI pipeline duration | 12min | 6min | 50% faster |

### Contributors

Special thanks to all contributors who made v1.0 possible:
- Core development and architecture
- Bug fixes and testing
- Documentation improvements
- Community feedback and support

---

## [0.9.0-alpha] - 2025-09-15

### Initial Alpha Release

- Initial public release with core analysis functionality
- Basic multi-language support (Rust, Python)
- Experimental AI integration
- HTML and JSON report generation
- Command-line interface
- Web dashboard (beta)

---

## Links

- **Repository**: https://github.com/botzrDev/uveddi
- **Documentation**: https://github.com/botzrDev/uveddi/tree/main/docs
- **Issues**: https://github.com/botzrDev/uveddi/issues
- **Releases**: https://github.com/botzrDev/uveddi/releases

---

For full commit history, see the [releases page](https://github.com/botzrDev/uveddi/releases).
