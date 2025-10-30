# Uveddi Feature Flags & Build Profiles – v1.0.0

**Generated:** 2025-10-09
**For Release:** 1.0.0 Commercial CLI
**Status:** Product Surface Freeze

## Overview
This document captures the complete feature flag matrix for Uveddi's 1.0.0 commercial release. It documents which features are enabled by default, which are optional, and which are deprecated or out-of-scope for the CLI-only release.

## CLI-Focused Profiles (Active)

### `cli-core` (Minimal CLI Functionality)
**Purpose:** Absolute minimum for CLI functionality
**Default:** No
**Customer-Facing:** Yes (advanced users)
**Dependencies:**
- `rusqlite` - SQLite database support
- `bincode` - Binary serialization
- `rayon` - Parallel processing
- `tokio-stream` - Async streaming
- `async-stream` - Async stream macros
- `reqwest` - HTTP client

**Enabled When:**
- Users want minimal binary size
- Minimal dependencies required
- Quick installation on constrained environments

---

### `cli-standard` ⭐ (RECOMMENDED DEFAULT)
**Purpose:** Standard CLI with all language support
**Default:** Yes (default feature)
**Customer-Facing:** Yes
**Dependencies:**
- All from `cli-core`
- `tree-sitter` - Language parsing
- `engine-integration` - Analysis engine
- `ast-cache` - AST caching
- `analysis-cache` - Analysis result caching

**Enabled When:**
- Default installation
- Standard commercial usage
- Recommended for all users

**Scope Coverage:**
- ✅ Multi-language parsing (Rust, Python, JS, TS)
- ✅ Anti-pattern detection
- ✅ Persistent caching
- ✅ Deterministic analysis

---

### `cli-ai` (CLI with AI Support)
**Purpose:** CLI with AI-powered analysis and explanations
**Default:** No
**Customer-Facing:** Yes (optional)
**Dependencies:**
- All from `cli-standard`
- `ai` - AI integration
- `local-ai` - Local Ollama support
- `reqwest` - AI API communication

**Enabled When:**
- Users have Ollama instance available
- AI-powered explanations desired
- Enhanced remediation guidance needed

**Scope Coverage:**
- ✅ Ollama integration
- ✅ AI-generated explanations
- ✅ Architectural recommendations
- ❌ SaaS-hosted AI (deferred)

---

### `cli-plugins` (CLI with Plugin Support)
**Purpose:** Enable WebAssembly plugin execution
**Default:** No
**Customer-Facing:** Yes (advanced)
**Dependencies:**
- All from `cli-standard`
- `wasm-plugins` - WASM runtime
- `wasmtime` - WASM execution engine
- `wasmtime-wasi` - WASI support
- `cap-std` - Capability-based security
- `wit-bindgen` - WIT bindings

**Enabled When:**
- Custom detector development
- Organization-specific rules
- Extended analysis capabilities

**Scope Coverage:**
- ✅ WASM plugin loading
- ✅ Custom detector execution
- ✅ Sandboxed plugin environment

---

### `cli-full` (Everything for CLI)
**Purpose:** Complete CLI with all optional features
**Default:** No
**Customer-Facing:** Yes (power users)
**Dependencies:**
- All from `cli-ai`
- All from `cli-plugins`

**Enabled When:**
- Maximum functionality needed
- Development/testing environments
- Advanced commercial deployments

---

## Language Parsing Features

### Individual Language Support

#### `rust-lang`
**Purpose:** Rust language parsing
**Default:** Via `cli-standard`
**Customer-Facing:** Yes
**Dependencies:** `tree-sitter-rust`

#### `python-lang`
**Purpose:** Python language parsing
**Default:** Via `cli-standard`
**Customer-Facing:** Yes
**Dependencies:** `tree-sitter-python`

#### `javascript-lang`
**Purpose:** JavaScript language parsing
**Default:** Via `cli-standard`
**Customer-Facing:** Yes
**Dependencies:** `tree-sitter-javascript`

#### `typescript-lang`
**Purpose:** TypeScript language parsing
**Default:** Via `cli-standard`
**Customer-Facing:** Yes
**Dependencies:** `tree-sitter-typescript`

### Language Packs

#### `languages-core`
**Purpose:** Backend languages (Rust, Python)
**Default:** Via `cli-standard`
**Customer-Facing:** Yes
**Includes:** `rust-lang`, `python-lang`

#### `languages-web`
**Purpose:** Frontend languages (JavaScript, TypeScript)
**Default:** Via `cli-standard`
**Customer-Facing:** Yes
**Includes:** `javascript-lang`, `typescript-lang`

#### `languages-all`
**Purpose:** All supported languages
**Default:** Via `cli-standard`
**Customer-Facing:** Yes
**Includes:** `languages-core`, `languages-web`

#### `tree-sitter`
**Purpose:** Full tree-sitter support with all languages
**Default:** Via `cli-standard`
**Customer-Facing:** Yes
**Includes:** `tree-sitter` (dep), `languages-all`

---

## Optional Features

### `ast-cache`
**Purpose:** AST caching with file watching
**Default:** Via `cli-standard`
**Customer-Facing:** Yes
**Dependencies:** `notify` (file system watching)

### `analysis-cache`
**Purpose:** Analysis result caching with invalidation
**Default:** Via `cli-standard`
**Customer-Facing:** Yes
**Dependencies:** `notify` (file system watching)

### `ai`
**Purpose:** AI integration base feature
**Default:** No
**Customer-Facing:** Yes (via `cli-ai`)
**Dependencies:** None (marker feature)

### `local-ai`
**Purpose:** Local Ollama AI support
**Default:** No
**Customer-Facing:** Yes (via `cli-ai`)
**Dependencies:** `ai`, `reqwest`

### `wasm-plugins`
**Purpose:** WebAssembly plugin system
**Default:** No
**Customer-Facing:** Yes (via `cli-plugins`)
**Dependencies:** `wasmtime`, `wasmtime-wasi`, `cap-std`, `wit-bindgen`

### `engine-integration`
**Purpose:** Analysis engine refactor integration
**Default:** Via `cli-standard`
**Customer-Facing:** No (internal)
**Dependencies:** `tree-sitter`

---

## Deprecated Features (Backwards Compatibility Only)

These features remain in Cargo.toml for backwards compatibility but map to current CLI profiles. They are NOT customer-facing and should not be documented in user-facing materials.

### Legacy Profile Mappings
- `minimal` → `cli-core` ⚠️ DEPRECATED
- `standard` → `cli-standard` ⚠️ DEPRECATED
- `full` → `cli-full` ⚠️ DEPRECATED
- `dev-core` → `cli-standard` ⚠️ DEPRECATED
- `dev-minimal` → `cli-core` ⚠️ DEPRECATED
- `dev-ultra-minimal` → `cli-core` ⚠️ DEPRECATED
- `dev-full` → `cli-standard` ⚠️ DEPRECATED

---

## Out-of-Scope / Deferred Features

These features are defined as empty markers in Cargo.toml but provide NO functionality in the 1.0.0 CLI release. They are placeholders for future roadmap items or removed enterprise features.

### Enterprise Features (Removed for CLI Release)
- `production` → Maps to `cli-full` but adds no functionality ❌
- `production-secure` → Maps to `cli-full` but adds no functionality ❌
- `security` → Empty marker (security scanning available via `--security` flag) ⚠️
- `crypto-minimal` → Empty marker, no crypto features ❌
- `crypto-full` → Empty marker, no crypto features ❌

### Web/API Features (Out of Scope)
- `web` → Empty marker ❌ NOT SUPPORTED
- `web-client` → Empty marker ❌ NOT SUPPORTED
- `web-server` → Empty marker ❌ NOT SUPPORTED
- `web-full` → Empty marker ❌ NOT SUPPORTED
- `graphql-api` → Empty marker ❌ NOT SUPPORTED

### TUI Features (Out of Scope)
- `tui` → Empty marker ❌ NOT SUPPORTED

### Monitoring/Observability (Out of Scope)
- `prometheus` → Empty marker ❌ NOT SUPPORTED
- `sla-monitoring` → Empty marker ❌ NOT SUPPORTED
- `regression-detection` → Empty marker ❌ NOT SUPPORTED

### Advanced Features (Out of Scope)
- `memory-optimization` → Empty marker ❌ NOT SUPPORTED
- `mimalloc` → Empty marker ❌ NOT SUPPORTED
- `chaos` → Empty marker ❌ NOT SUPPORTED
- `performance-testing` → Empty marker ❌ NOT SUPPORTED
- `image-rendering` → Empty marker ❌ NOT SUPPORTED
- `interactive-reports` → Empty marker ❌ NOT SUPPORTED
- `yaml` → Empty marker ❌ NOT SUPPORTED
- `reqwest` → Empty marker (reqwest is always available) ⚠️

---

## Recommended Build Configurations

### Standard Commercial Installation
```bash
cargo build --release --features cli-standard
```
**Use Case:** Default recommended build for all commercial users

### AI-Enhanced Installation
```bash
cargo build --release --features cli-ai
```
**Use Case:** Users with Ollama for AI-powered analysis

### Plugin-Enabled Installation
```bash
cargo build --release --features cli-plugins
```
**Use Case:** Organizations with custom detector requirements

### Complete Installation
```bash
cargo build --release --features cli-full
```
**Use Case:** Development, testing, or maximum functionality deployments

### Minimal Installation
```bash
cargo build --release --features cli-core
```
**Use Case:** Size-constrained environments, minimal dependencies

---

## Unresolved Gaps & Action Items

### 1. Security Feature Clarity ⚠️
**Issue:** The `security` feature flag is empty but security scanning is available via CLI flags
**Impact:** Potential customer confusion
**Recommendation:**
- Document that security scanning is always available via `uveddi analyze --security`
- Consider removing empty `security` feature flag or making it functional
- Update docs to clarify no build-time feature required

**Owner:** Phase A3 (Documentation alignment)

### 2. Empty Feature Flag Cleanup 🔧
**Issue:** Many empty feature flags remain for backwards compatibility
**Impact:** Cargo.toml clutter, potential confusion
**Recommendation:**
- Add deprecation warnings to empty features
- Plan removal in v2.0.0
- Document migration path in CHANGELOG

**Owner:** Post-launch technical debt

### 3. Image Rendering Feature Mismatch 🔧
**Issue:** `image-rendering` flag is empty but CLI has `--enable-image-rendering` flag
**Impact:** Feature works without build flag, unclear requirement
**Recommendation:**
- Either make feature flag functional or remove it entirely
- Clarify that image rendering requires external service, not build flag
- Update CLI help text

**Owner:** Phase A3 (Documentation alignment)

### 4. Default Feature Verification ✅
**Issue:** Need to verify `cli-standard` is truly the default in all contexts
**Impact:** Installation success KPI
**Recommendation:**
- Test `cargo build` without explicit features
- Verify binaries include expected parsers
- Document in installation guide

**Owner:** Phase A4 (Build validation)

---

## Verification Checklist

- ✅ All CLI-focused profiles documented with purpose, dependencies, scope
- ✅ Language features mapped to customer use cases
- ✅ Deprecated features identified with migration path
- ✅ Out-of-scope features clearly marked
- ✅ Recommended builds documented
- ⚠️ Unresolved gaps identified for Phase A3
- ⚠️ Empty features flagged for cleanup

---

**Document Status:** Complete
**Next Review:** Phase A3 (Documentation Alignment)
**Sign-Off Required:** PM review before Phase A4
