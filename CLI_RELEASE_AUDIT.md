# CLI-Only Community Release - Audit Report

**Branch:** `release/cli-only`  
**Date:** October 6, 2025  
**Objective:** Strip down Uveddi to core CLI functionality for rapid community release

---

## Executive Summary

Uveddi is currently a comprehensive architectural analysis platform with ~280K lines of Rust code, 72K lines of test code, and three separate Node.js services (frontend, api-server, rendering-service). For a rapid community CLI release, we need to:

1. **Remove web infrastructure** (frontend, api-server, rendering-service, TUI, Serve command)
2. **Eliminate enterprise features** (security/auth, monitoring, deployment, chaos engineering, SLA monitoring)
3. **Streamline to core engine** (analysis, AST parsing, detection, reporting)
4. **Keep optional AI** (via feature flag for Ollama integration)
5. **Maintain plugin system** (optional via feature flag for extensibility)

---

## Current State Analysis

### Module Breakdown (by lines of code)

| Module | LOC | Status for CLI Release |
|--------|-----|------------------------|
| **analysis** | 128,948 | ✅ **KEEP** - Core engine |
| **ai** | 16,721 | ⚠️ **OPTIONAL** - Keep behind `ai` feature flag |
| **database** | 11,714 | ⚠️ **SIMPLIFY** - Keep SQLite basics, remove enterprise features |
| **security** | 10,995 | ❌ **REMOVE** - OAuth, JWT, Vault, RBAC not needed for CLI |
| **plugins** | 9,438 | ⚠️ **OPTIONAL** - Keep behind `wasm-plugins` feature flag |
| **report** | 8,237 | ✅ **KEEP** - Essential for output |
| **application** | 7,865 | ✅ **KEEP** - Core orchestration |
| **engine** | 7,041 | ✅ **KEEP** - AST parsing engine |
| **performance** | 6,982 | ❌ **REMOVE** - Benchmarking, regression detection, genetic optimization |
| **monitoring** | 6,688 | ❌ **REMOVE** - Prometheus, metrics, observability |
| **api** | 6,218 | ❌ **REMOVE** - REST API for web dashboard |
| **cli** | 6,116 | ✅ **KEEP** - Core CLI commands |
| **observability** | 5,736 | ❌ **REMOVE** - Distributed tracing, telemetry |
| **resilience** | 4,953 | ❌ **REMOVE** - Circuit breakers, alerting, degradation |
| **tui** | 4,124 | ❌ **REMOVE** - Terminal UI (not essential for CLI) |
| **deployment** | 3,891 | ❌ **REMOVE** - Blue/green, disaster recovery, k8s |
| **resource_management** | 3,863 | ⚠️ **SIMPLIFY** - Keep basic memory tracking only |
| **community** | 3,192 | ❌ **REMOVE** - Community analytics and database |
| **ast** | 2,676 | ✅ **KEEP** - Tree-sitter integration |

### External Services

| Service | Size | Status |
|---------|------|--------|
| **frontend/** (React) | ~15K LOC | ❌ **REMOVE** - Web dashboard |
| **api-server/** (Express) | ~5K LOC | ❌ **REMOVE** - Backend API |
| **rendering-service/** (Puppeteer) | ~3K LOC | ❌ **REMOVE** - HTML rendering |

### Feature Flags (Current)

**Profiles:**
- `minimal` - Core dependencies only
- `standard` - Includes tree-sitter + prometheus
- `full` - Everything (security, web, TUI, plugins)

**Specialized Features:**
- `tree-sitter` - Multi-language parsing ✅ KEEP
- `ai` / `local-ai` - LLM integration ⚠️ OPTIONAL
- `wasm-plugins` - Plugin system ⚠️ OPTIONAL
- `security` - Auth, encryption, RBAC ❌ REMOVE
- `web` - Web server infrastructure ❌ REMOVE
- `tui` - Terminal UI ❌ REMOVE
- `prometheus` - Metrics ❌ REMOVE
- `chaos` - Chaos engineering ❌ REMOVE
- `memory-optimization` - Advanced memory features ❌ REMOVE

### CLI Commands (Current)

| Command | Description | Status |
|---------|-------------|--------|
| `analyze` | Core analysis functionality | ✅ **KEEP** |
| `config` | Configuration management | ✅ **KEEP** |
| `doctor` | Health diagnostics | ✅ **KEEP** |
| `help` | Help system | ✅ **KEEP** |
| `init` | Project initialization | ✅ **KEEP** |
| `hooks` | Git hooks | ⚠️ **EVALUATE** - Useful for CLI? |
| `ci` | CI/CD integration | ⚠️ **EVALUATE** - Useful for CLI? |
| `ui` | Launch web dashboard | ❌ **REMOVE** |
| `serve` | Start web services | ❌ **REMOVE** |
| `tui` | Terminal UI | ❌ **REMOVE** |
| `plugin` | Plugin management | ⚠️ **OPTIONAL** |

### Dependencies Analysis

**Total Dependencies:** ~256 crate entries

**Essential (Core Engine):**
- `clap` - CLI parsing ✅
- `tree-sitter*` - AST parsing ✅
- `serde*` - Serialization ✅
- `tokio` - Async runtime ✅
- `rayon` - Parallelism ✅
- `anyhow`, `thiserror` - Error handling ✅
- `regex`, `walkdir`, `ignore` - File processing ✅
- `petgraph` - Dependency graphs ✅
- `tera` - Template engine for reports ✅
- `rusqlite` - Lightweight persistence ✅

**Optional (Can be feature-gated):**
- `reqwest` - HTTP client (for AI) ⚠️
- `wasmtime*` - WASM runtime (for plugins) ⚠️
- `ratatui`, `crossterm` - TUI ❌

**Remove (Enterprise/Web):**
- `axum`, `tower*` - Web server ❌
- `tokio-tungstenite` - WebSockets ❌
- `prometheus`, `metrics*` - Monitoring ❌
- `casbin`, `oauth2`, `jsonwebtoken` - Security ❌
- `vaultrs` - Secrets management ❌
- `mimalloc`, `rkyv`, `memmap2` - Advanced memory ❌
- `fail`, `changepoint`, `linfa` - Chaos/ML ❌
- `influxdb2` - Time series DB ❌

**Reduction Estimate:** Remove ~120-150 dependencies (40-60% reduction)

---

## Recommended CLI-Only Architecture

### Core Components to Keep

```
uveddi/
├── src/
│   ├── cli/          ✅ Command-line interface
│   ├── analysis/     ✅ Core analysis engine (~129K LOC)
│   ├── ast/          ✅ Tree-sitter integration
│   ├── engine/       ✅ AST parsing engine
│   ├── core/         ✅ Core utilities and types
│   ├── report/       ✅ Output formatting (JSON, Markdown, HTML)
│   ├── application/  ✅ Orchestration (simplified)
│   ├── database/     ✅ Basic SQLite persistence (simplified)
│   ├── cache/        ✅ AST caching
│   ├── error/        ✅ Error handling
│   ├── config/       ✅ Configuration management
│   ├── health/       ✅ Doctor command support
│   ├── hooks/        ⚠️ Git hooks (evaluate need)
│   ├── models/       ✅ Core data models
│   ├── progress/     ✅ Progress tracking
│   ├── templates/    ✅ Report templates
│   ├── ai/           ⚠️ Optional AI integration (feature flag)
│   └── plugins/      ⚠️ Optional plugin system (feature flag)
│
├── tests/            ✅ Keep core analysis tests, remove enterprise tests
├── docs/             ✅ Simplify to CLI-focused documentation
├── examples/         ✅ Keep CLI examples only
└── Cargo.toml        ⚠️ Dramatically simplified dependencies
```

### Components to Remove

```
REMOVE:
├── frontend/                  ❌ React web dashboard
├── api-server/                ❌ Express API server
├── rendering-service/         ❌ Puppeteer rendering
├── src/api/                   ❌ REST API endpoints
├── src/tui/                   ❌ Terminal UI
├── src/security/              ❌ Auth/RBAC/encryption
├── src/monitoring/            ❌ Prometheus/metrics
├── src/observability/         ❌ Distributed tracing
├── src/deployment/            ❌ K8s/blue-green/DR
├── src/resilience/            ❌ Circuit breakers
├── src/performance/           ❌ Benchmarking/regression
├── src/chaos/                 ❌ Chaos engineering
├── src/sla/                   ❌ SLA monitoring
├── src/community/             ❌ Community analytics
├── src/resource_management/   ❌ Advanced memory management
├── src/service_orchestration/ ❌ Service orchestration
├── src/ingestion/             ❌ Data ingestion
├── src/infrastructure/        ❌ Infrastructure code
├── src/semantic_search/       ❌ Semantic search
├── src/testing/               ❌ Performance testing framework
├── k8s/                       ❌ Kubernetes configs
├── helm/                      ❌ Helm charts
├── terraform/                 ❌ Terraform configs
├── docker/                    ❌ Docker configs (keep simple Dockerfile only)
├── monitoring/                ❌ Monitoring configs
└── systemd/                   ❌ Systemd services
```

---

## Implementation Plan

### Phase 1: Feature Flag Cleanup (Quick Wins)

**Goal:** Define new minimal feature set for CLI

```toml
[features]
# New CLI-focused profiles
default = ["cli-core"]

# Absolute minimum for CLI functionality
cli-core = [
    "dep:rusqlite",
    "dep:bincode", 
    "dep:rayon",
    "dep:tokio-stream",
    "dep:async-stream"
]

# Standard CLI with all language support
cli-standard = ["cli-core", "tree-sitter"]

# CLI with AI support (requires Ollama)
cli-ai = ["cli-standard", "ai", "local-ai", "dep:reqwest"]

# CLI with plugin support
cli-plugins = ["cli-standard", "wasm-plugins"]

# Everything for CLI (AI + plugins)
cli-full = ["cli-ai", "cli-plugins"]

# Individual language support (keep existing)
rust-lang = ["dep:tree-sitter-rust"]
python-lang = ["dep:tree-sitter-python"]
javascript-lang = ["dep:tree-sitter-javascript"]
javascript-lang = ["dep:tree-sitter-typescript"]

# Language packs (keep existing)
languages-core = ["rust-lang", "python-lang"]
languages-web = ["javascript-lang", "typescript-lang"]
languages-all = ["languages-core", "languages-web"]

# Full tree-sitter support
tree-sitter = ["dep:tree-sitter", "languages-all"]

# Optional features
ai = []
local-ai = ["ai", "dep:reqwest"]
wasm-plugins = ["dep:wasmtime", "dep:wasmtime-wasi", "dep:wit-bindgen"]
ast-cache = ["dep:notify"]

# DEPRECATED/REMOVED: web, tui, security, prometheus, chaos, etc.
```

### Phase 2: Remove External Services

```bash
# Remove web services
rm -rf frontend/
rm -rf api-server/
rm -rf rendering-service/

# Remove infrastructure configs
rm -rf k8s/
rm -rf helm/
rm -rf terraform/
rm -rf monitoring/
rm -rf systemd/
rm -rf docker/ (keep simple Dockerfile)
```

### Phase 3: Remove Source Modules

```bash
# Remove enterprise modules
rm -rf src/api/
rm -rf src/tui/
rm -rf src/security/
rm -rf src/monitoring/
rm -rf src/observability/
rm -rf src/deployment/
rm -rf src/resilience/
rm -rf src/performance/
rm -rf src/chaos/
rm -rf src/sla/
rm -rf src/community/
rm -rf src/service_orchestration/
rm -rf src/ingestion/
rm -rf src/infrastructure/
rm -rf src/semantic_search/
rm -rf src/testing/

# Simplify resource management
# Keep basic memory tracking, remove advanced features
```

### Phase 4: Prune Dependencies

Remove from `Cargo.toml`:
- Web: `axum`, `tower*`, `tokio-tungstenite`
- Monitoring: `prometheus`, `metrics*`, `influxdb2`
- Security: `casbin`, `oauth2`, `jsonwebtoken`, `vaultrs`, `openidconnect`
- Memory: `mimalloc`, `rkyv`, `memmap2`, `bumpalo*`
- TUI: `ratatui`, `crossterm`, `tui-input`
- Chaos: `fail`, `changepoint`, `linfa`, `statrs`, `ndarray-stats`
- Advanced: `arrow*`, `config`

Make optional (move to feature gates):
- `reqwest` → only for `ai` feature
- `wasmtime*` → only for `wasm-plugins` feature
- `notify` → only for `ast-cache` feature

### Phase 5: Update CLI Commands

**Remove from `src/main.rs`:**
```rust
Commands::Ui       // Remove web dashboard launch
Commands::Serve    // Remove service orchestration
Commands::Tui      // Remove terminal UI
```

**Evaluate:**
```rust
Commands::Hooks    // Git hooks - useful for CLI users?
Commands::Ci       // CI integration - useful but maybe overkill?
```

**Keep:**
```rust
Commands::Analyze  // Core functionality
Commands::Config   // Essential
Commands::Doctor   // Diagnostics
Commands::Help     // Essential
Commands::Init     // Project setup
Commands::Plugin   // Optional (feature-gated)
```

### Phase 6: Simplify Tests

```bash
# Remove enterprise test suites
rm -rf tests/security_tests.rs
rm -rf tests/performance/
rm -rf tests/chaos/
rm -rf tests/deployment/

# Keep core tests
tests/analysis/     ✅
tests/cli_*.rs      ✅
tests/config.rs     ✅
tests/database_*.rs ✅ (simplify)
tests/coverage/     ✅
```

### Phase 7: Documentation Cleanup

```bash
# Remove enterprise docs
rm -rf docs/deployment/
rm -rf docs/archive/ (selectively)
rm docs/KUBERNETES_*.md
rm docs/MONITORING_*.md
rm docs/SECURITY_*.md
rm docs/SLA_*.md

# Keep/update
docs/getting-started/  ✅ Update for CLI focus
docs/examples/         ✅ CLI examples only
docs/development/      ✅ Simplified development guide
README.md              ⚠️ Rewrite for CLI focus
CONTRIBUTING.md        ⚠️ Simplify
```

### Phase 8: Update Build Configuration

**New recommended build:**
```bash
# Development (fast iteration)
cargo build --features cli-standard

# Release (with AI support)
cargo build --release --features cli-ai

# Minimal (no languages, no AI)
cargo build --release --features cli-core
```

---

## Expected Outcomes

### Binary Size Reduction
- **Current (full build):** ~80-120 MB
- **CLI-only (standard):** ~20-30 MB
- **CLI-only (minimal):** ~10-15 MB

### Compilation Time Improvement
- **Current (full):** ~5-8 minutes
- **CLI-only (standard):** ~2-3 minutes
- **CLI-only (minimal):** ~1-2 minutes

### Dependency Reduction
- **Current:** ~256 dependencies
- **CLI-only:** ~100-120 dependencies
- **Reduction:** ~50% fewer dependencies

### Code Reduction
- **Remove:** ~80K LOC of Rust (enterprise features)
- **Remove:** ~25K LOC of JavaScript/TypeScript (web services)
- **Keep:** ~150K LOC of Rust (core engine + tests)

### Maintenance Benefits
- Simpler dependency tree
- Faster CI/CD builds
- Easier to onboard contributors
- Clearer project scope
- Reduced security surface area

---

## Risk Assessment

### Low Risk
✅ Removing web services (frontend, api-server, rendering-service)  
✅ Removing monitoring/observability (Prometheus, distributed tracing)  
✅ Removing enterprise security (OAuth, JWT, Vault)  
✅ Removing deployment infrastructure (k8s, helm, terraform)  
✅ Removing chaos engineering and performance frameworks  

### Medium Risk
⚠️ Simplifying database module - ensure core persistence still works  
⚠️ Removing TUI - some users may prefer interactive mode  
⚠️ Making AI optional - need to ensure graceful degradation  
⚠️ Making plugins optional - need clear feature gate documentation  

### High Risk (Handle Carefully)
🔴 Modifying core analysis engine - DO NOT BREAK  
🔴 Removing too much from `application/` orchestration  
🔴 Over-simplifying error handling  

---

## Migration Path for Existing Users

For users who want web dashboard or enterprise features:

1. **Keep full version on `main` branch**
2. **CLI-only on `release/cli-only` branch**
3. **Document both versions in README**
4. **Provide migration guide for enterprise → CLI**

Alternatively:
- Rename project: `uveddi` (CLI) vs `uveddi-enterprise` (full)
- Separate repositories
- Monorepo with workspace structure

---

## Next Steps

1. ✅ **Audit complete** - This document
2. ⏭️ **Get approval** - Confirm removal scope
3. ⏭️ **Create backup branch** - `git branch backup-pre-cli-strip`
4. ⏭️ **Execute Phase 1** - Update feature flags
5. ⏭️ **Execute Phase 2** - Remove external services
6. ⏭️ **Execute Phase 3-4** - Remove modules and dependencies
7. ⏭️ **Execute Phase 5** - Update CLI commands
8. ⏭️ **Test build** - Ensure `cargo build --features cli-standard` works
9. ⏭️ **Update docs** - CLI-focused documentation
10. ⏭️ **Final testing** - Run test suite, verify core functionality
11. ⏭️ **Tag release** - `v1.0.0-cli-community`

---

## Recommendation

**Go ahead with CLI-only release:**
- Clear scope reduction (remove ~50% of code)
- Fast path to community release
- Maintainable and focused project
- Easy to understand for contributors
- Web/enterprise features can be separate project or paid tier

**Success Criteria:**
- ✅ Builds in under 3 minutes
- ✅ Binary under 30 MB
- ✅ Works without any web services
- ✅ AI and plugins optional via feature flags
- ✅ All core analysis tests pass
- ✅ Documentation reflects CLI-only scope
