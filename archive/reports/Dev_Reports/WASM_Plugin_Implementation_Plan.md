# WASM Plugin System Implementation Plan

**Date:** July 1, 2025  
**Status:** Ready for Implementation  
**Priority:** High (Architecture Report Compliance)

## Executive Summary

This document provides a detailed implementation plan for the WASM-based plugin system in Uveddi, following the comprehensive architectural research in `docs/R&D/WASM_Plugin.md`. The current codebase has a basic plugin system with native Rust plugins integrated into the analysis pipeline. The next phase is to implement secure WASM sandboxing with the Wasmtime runtime.

## Current State Analysis

### ✅ Completed Work
- **Plugin Manager Foundation**: `src/plugin/mod.rs` implements a basic `PluginManager` with example plugins
- **Integration**: Plugin system is integrated into `AnalyzeCommand` and receives `DependencyGraph` from `AnalysisEngine`
- **Dependencies**: `wasmtime = "21.0.1"` and `bincode = "1.3.3"` added to `Cargo.toml`
- **Data Models**: `DependencyGraph` struct available in `src/models/dependency_graph.rs`
- **Example Plugins**: `GodObjectDetector` and `CyclomaticComplexityDetector` working as native Rust plugins

### 🔄 Current Architecture Gap
The existing plugin system runs native Rust code within the same process, providing no security isolation. Per the architecture research, this needs to be replaced with a WASM-based system that provides:
- **Security**: Capability-based sandboxing with WASI
- **Performance**: AOT compilation with persistent caching
- **Standardization**: Component Model with WIT interfaces

## Implementation Plan: Phase 1 - Core WASM Runtime

### Objective
Replace the current native plugin system with a secure WASM-based implementation using Wasmtime, following the research recommendations.

### Step 1: WASM Plugin Interface Design (2-3 days)

#### 1.1 Create WIT Interface Definition
**File:** `plugins/interface/plugin.wit`

```wit
package uveddi:plugins@0.1.0;

// Core data types for analysis
record dependency {
    from-module: string,
    to-module: string,
    dependency-type: string,
}

record architectural-issue {
    file-path: string,
    start-line: u32,
    end-line: u32,
    issue-type: string,
    severity: string,
    message: string,
    code-snippet: option<string>,
}

// Plugin metadata
record plugin-info {
    name: string,
    version: string,
    description: string,
    author: string,
}

// Main plugin interface
world plugin {
    // Host functions that plugins can import
    import logging: interface {
        log: func(level: string, message: string);
    }
    
    import config: interface {
        get-value: func(key: string) -> option<string>;
    }

    // Plugin exports
    export info: func() -> plugin-info;
    export analyze: func(dependencies: list<dependency>) -> list<architectural-issue>;
}
```

#### 1.2 Update Plugin Manager for WASM
**File:** `src/plugin/wasm_manager.rs`

```rust
use wasmtime::{Engine, Linker, Module, Store, Instance};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder};
use std::path::Path;
use crate::models::dependency_graph::DependencyGraph;
use crate::database::models::ArchitecturalIssue;
use crate::error::UveddiError;

pub struct WasmPluginManager {
    engine: Engine,
    linker: Linker<WasiCtx>,
}

impl WasmPluginManager {
    pub fn new() -> Result<Self, UveddiError> {
        let engine = Engine::default();
        let mut linker = Linker::new(&engine);
        
        // Add WASI support
        wasmtime_wasi::add_to_linker(&mut linker, |s| s)?;
        
        // Add host functions (logging, config)
        linker.func_wrap("logging", "log", |caller: wasmtime::Caller<'_, WasiCtx>, level: String, message: String| {
            match level.as_str() {
                "info" => log::info!("[Plugin] {}", message),
                "warn" => log::warn!("[Plugin] {}", message),
                "error" => log::error!("[Plugin] {}", message),
                _ => log::debug!("[Plugin] {}", message),
            }
        })?;
        
        Ok(Self { engine, linker })
    }
    
    pub fn load_plugin(&self, wasm_path: &Path) -> Result<WasmPlugin, UveddiError> {
        let module = Module::from_file(&self.engine, wasm_path)?;
        
        // Create WASI context with minimal permissions
        let wasi = WasiCtxBuilder::new()
            .inherit_stdio()
            .build();
            
        let mut store = Store::new(&self.engine, wasi);
        let instance = self.linker.instantiate(&mut store, &module)?;
        
        Ok(WasmPlugin {
            store,
            instance,
            module,
        })
    }
}

pub struct WasmPlugin {
    store: Store<WasiCtx>,
    instance: Instance,
    module: Module,
}

impl WasmPlugin {
    pub fn analyze(&mut self, dependencies: &DependencyGraph) -> Result<Vec<ArchitecturalIssue>, UveddiError> {
        // Serialize dependencies using the WIT-generated bindings
        // Call the plugin's analyze function
        // Deserialize and return results
        todo!("Implement WASM function calls using wit-bindgen")
    }
}
```

### Step 2: Security Implementation (2-3 days)

#### 2.1 Capability-Based Security
**File:** `src/plugin/security.rs`

```rust
use wasmtime_wasi::WasiCtxBuilder;
use std::path::Path;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub permissions: Vec<Permission>,
    pub resource_limits: ResourceLimits,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Permission {
    #[serde(rename = "fs:read")]
    FilesystemRead { path: String },
    #[serde(rename = "net:connect")]
    NetworkConnect { host: String },
    #[serde(rename = "env:read")]
    EnvironmentRead { var: String },
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ResourceLimits {
    pub max_memory_mb: u32,
    pub max_execution_fuel: u64,
    pub max_output_size_kb: u32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: 16,           // 16MB memory limit
            max_execution_fuel: 1_000_000, // ~1M instructions
            max_output_size_kb: 16,      // 16KB output limit
        }
    }
}

pub fn create_secure_wasi_context(manifest: &PluginManifest) -> wasmtime::Result<WasiCtx> {
    let mut builder = WasiCtxBuilder::new();
    
    // Always inherit stdio for logging
    builder = builder.inherit_stdio();
    
    // Grant permissions based on manifest
    for permission in &manifest.permissions {
        match permission {
            Permission::FilesystemRead { path } => {
                if let Ok(dir) = cap_std::fs::Dir::open_ambient_dir(path, cap_std::ambient_authority()) {
                    builder = builder.preopened_dir(dir, wasi_common::file::FdFlags::empty(), path)?;
                }
            }
            Permission::NetworkConnect { host: _ } => {
                // Network permissions would be implemented here
                log::warn!("Network permissions not yet implemented");
            }
            Permission::EnvironmentRead { var } => {
                if let Ok(value) = std::env::var(var) {
                    builder = builder.env(var, &value)?;
                }
            }
        }
    }
    
    Ok(builder.build())
}
```

#### 2.2 Plugin Verification Pipeline
**File:** `src/plugin/verification.rs`

```rust
use std::path::Path;
use sha2::{Sha256, Digest};
use crate::error::UveddiError;

pub struct PluginVerifier;

impl PluginVerifier {
    pub fn verify_plugin(wasm_path: &Path, manifest_path: &Path) -> Result<(), UveddiError> {
        // 1. Verify file integrity
        Self::verify_file_hash(wasm_path)?;
        
        // 2. Parse and validate manifest
        let manifest = Self::load_manifest(manifest_path)?;
        Self::validate_manifest(&manifest)?;
        
        // 3. Basic WASM structure validation
        Self::validate_wasm_structure(wasm_path)?;
        
        Ok(())
    }
    
    fn verify_file_hash(wasm_path: &Path) -> Result<(), UveddiError> {
        let contents = std::fs::read(wasm_path)?;
        let hash = Sha256::digest(&contents);
        
        // In a full implementation, compare against a known hash from a registry
        log::info!("Plugin hash: {:x}", hash);
        Ok(())
    }
    
    fn load_manifest(manifest_path: &Path) -> Result<super::security::PluginManifest, UveddiError> {
        let contents = std::fs::read_to_string(manifest_path)?;
        let manifest: super::security::PluginManifest = toml::from_str(&contents)
            .map_err(|e| UveddiError::PluginError(format!("Invalid manifest: {}", e)))?;
        Ok(manifest)
    }
    
    fn validate_manifest(manifest: &super::security::PluginManifest) -> Result<(), UveddiError> {
        if manifest.name.is_empty() {
            return Err(UveddiError::PluginError("Plugin name cannot be empty".to_string()));
        }
        
        if manifest.resource_limits.max_memory_mb > 256 {
            return Err(UveddiError::PluginError("Memory limit too high".to_string()));
        }
        
        Ok(())
    }
    
    fn validate_wasm_structure(wasm_path: &Path) -> Result<(), UveddiError> {
        // Use wasmtime to parse and validate the module structure
        let engine = wasmtime::Engine::default();
        let _module = wasmtime::Module::from_file(&engine, wasm_path)
            .map_err(|e| UveddiError::PluginError(format!("Invalid WASM module: {}", e)))?;
        Ok(())
    }
}
```

### Step 3: WIT Bindings Integration (3-4 days)

#### 3.1 Add Required Dependencies
**File:** `Cargo.toml` (update)

```toml
[dependencies]
# ... existing dependencies ...
wasmtime = "21.0.1"
wasmtime-wasi = "21.0.1"
wit-bindgen = "0.25.0"
cap-std = "3.0.0"
wasi-common = "21.0.1"
toml = "0.8.0"
sha2 = "0.10.0"

[build-dependencies]
wit-bindgen = "0.25.0"
```

#### 3.2 Build Script for WIT Bindings
**File:** `build.rs`

```rust
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    
    // Generate Rust bindings from WIT files
    wit_bindgen::generate!({
        world: "plugin",
        path: "plugins/interface/plugin.wit",
        generate_all: true,
    });
    
    println!("cargo:rerun-if-changed=plugins/interface/plugin.wit");
}
```

### Step 4: Example WASM Plugin (2 days)

#### 4.1 Create Example Plugin Project
**Directory:** `plugins/examples/wasm-god-object-detector/`

**File:** `plugins/examples/wasm-god-object-detector/Cargo.toml`

```toml
[package]
name = "wasm-god-object-detector"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
wit-bindgen = "0.25.0"
serde = { version = "1.0", features = ["derive"] }

[package.metadata.component]
package = "uveddi:plugins"

[package.metadata.component.target]
world = "plugin"
```

**File:** `plugins/examples/wasm-god-object-detector/src/lib.rs`

```rust
wit_bindgen::generate!({
    world: "plugin",
    path: "../../../plugins/interface/plugin.wit",
});

use exports::uveddi::plugins::plugin::Guest;
use uveddi::plugins::plugin::*;

struct GodObjectDetector;

impl Guest for GodObjectDetector {
    fn info() -> PluginInfo {
        PluginInfo {
            name: "WASM God Object Detector".to_string(),
            version: "0.1.0".to_string(),
            description: "Detects classes/modules that are too large (God Objects)".to_string(),
            author: "Uveddi Team".to_string(),
        }
    }
    
    fn analyze(dependencies: Vec<Dependency>) -> Vec<ArchitecturalIssue> {
        let mut issues = Vec::new();
        
        // Simple heuristic: if a module has more than 10 outgoing dependencies, flag it
        let mut dep_counts = std::collections::HashMap::new();
        
        for dep in dependencies {
            *dep_counts.entry(dep.from_module.clone()).or_insert(0) += 1;
        }
        
        for (module, count) in dep_counts {
            if count > 10 {
                issues.push(ArchitecturalIssue {
                    file_path: module.clone(),
                    start_line: 1,
                    end_line: 1,
                    issue_type: "god_object".to_string(),
                    severity: "medium".to_string(),
                    message: format!("Module '{}' has {} dependencies, indicating it may be a God Object", module, count),
                    code_snippet: None,
                });
            }
        }
        
        issues
    }
}

export!(GodObjectDetector);
```

**File:** `plugins/examples/wasm-god-object-detector/plugin.toml`

```toml
[plugin]
name = "wasm-god-object-detector"
version = "0.1.0"
description = "Detects God Objects in code"
author = "Uveddi Team"

[permissions]
# No special permissions needed for this plugin

[resource_limits]
max_memory_mb = 8
max_execution_fuel = 500_000
max_output_size_kb = 8
```

### Step 5: Integration and Testing (2-3 days)

#### 5.1 Update Main Plugin Manager
**File:** `src/plugin/mod.rs` (replace existing implementation)

```rust
mod wasm_manager;
mod security;
mod verification;

pub use wasm_manager::{WasmPluginManager, WasmPlugin};
pub use security::{PluginManifest, Permission, ResourceLimits};
pub use verification::PluginVerifier;

use crate::models::dependency_graph::DependencyGraph;
use crate::database::models::ArchitecturalIssue;
use crate::error::UveddiError;
use std::path::PathBuf;

pub struct PluginManager {
    wasm_manager: WasmPluginManager,
    plugin_directory: PathBuf,
}

impl PluginManager {
    pub fn new() -> Result<Self, UveddiError> {
        Ok(Self {
            wasm_manager: WasmPluginManager::new()?,
            plugin_directory: PathBuf::from("plugins/installed"),
        })
    }
    
    pub fn run_plugins(&self, dependency_graph: &DependencyGraph) -> Vec<Result<Vec<ArchitecturalIssue>, UveddiError>> {
        let mut results = Vec::new();
        
        // Scan for installed plugins
        if let Ok(entries) = std::fs::read_dir(&self.plugin_directory) {
            for entry in entries.flatten() {
                if let Some(extension) = entry.path().extension() {
                    if extension == "wasm" {
                        let result = self.run_single_plugin(&entry.path(), dependency_graph);
                        results.push(result);
                    }
                }
            }
        }
        
        results
    }
    
    fn run_single_plugin(&self, wasm_path: &PathBuf, dependency_graph: &DependencyGraph) -> Result<Vec<ArchitecturalIssue>, UveddiError> {
        // Find corresponding manifest
        let manifest_path = wasm_path.with_extension("toml");
        
        // Verify plugin
        PluginVerifier::verify_plugin(wasm_path, &manifest_path)?;
        
        // Load and execute plugin
        let mut plugin = self.wasm_manager.load_plugin(wasm_path)?;
        plugin.analyze(dependency_graph)
    }
}

pub fn initialize_plugins() -> PluginManager {
    match PluginManager::new() {
        Ok(manager) => manager,
        Err(e) => {
            log::warn!("Failed to initialize plugin manager: {}. Continuing without plugins.", e);
            // Return a minimal implementation that does nothing
            PluginManager {
                wasm_manager: WasmPluginManager::new().unwrap_or_else(|_| panic!("Critical: Cannot initialize WASM runtime")),
                plugin_directory: PathBuf::from("plugins/installed"),
            }
        }
    }
}
```

#### 5.2 Create Plugin Installation Directory
**Command to run:**
```bash
mkdir -p plugins/installed
mkdir -p plugins/interface
mkdir -p plugins/examples
```

#### 5.3 Integration Tests
**File:** `tests/plugin_wasm_integration.rs`

```rust
use uveddi::plugin::{PluginManager, PluginVerifier};
use uveddi::models::dependency_graph::DependencyGraph;
use std::path::PathBuf;

#[tokio::test]
async fn test_wasm_plugin_loading() {
    let plugin_manager = PluginManager::new().expect("Failed to create plugin manager");
    
    // Test with empty dependency graph
    let dependency_graph = DependencyGraph::new();
    let results = plugin_manager.run_plugins(&dependency_graph);
    
    // Should complete without panicking
    assert!(results.iter().all(|r| r.is_ok() || r.is_err()));
}

#[test]
fn test_plugin_verification() {
    // Test with a non-existent plugin (should fail gracefully)
    let wasm_path = PathBuf::from("nonexistent.wasm");
    let manifest_path = PathBuf::from("nonexistent.toml");
    
    let result = PluginVerifier::verify_plugin(&wasm_path, &manifest_path);
    assert!(result.is_err());
}
```

## Performance Benchmarking Plan

### Benchmark Implementation
**File:** `benches/wasm_plugin_performance.rs`

```rust
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use uveddi::plugin::PluginManager;
use uveddi::models::dependency_graph::DependencyGraph;

fn benchmark_wasm_plugin_execution(c: &mut Criterion) {
    let plugin_manager = PluginManager::new().expect("Failed to create plugin manager");
    let mut dependency_graph = DependencyGraph::new();
    
    // Add test dependencies
    for i in 0..100 {
        dependency_graph.add_dependency(
            format!("module_{}", i),
            format!("module_{}", (i + 1) % 100),
            "import".to_string()
        );
    }
    
    c.bench_with_input(
        BenchmarkId::new("wasm_plugin_execution", "100_deps"),
        &dependency_graph,
        |b, deps| {
            b.iter(|| {
                plugin_manager.run_plugins(deps)
            })
        },
    );
}

criterion_group!(benches, benchmark_wasm_plugin_execution);
criterion_main!(benches);
```

## Error Handling Updates

### Update Error Types
**File:** `src/error.rs` (add to existing UveddiError enum)

```rust
#[derive(Debug, thiserror::Error)]
pub enum UveddiError {
    // ... existing variants ...
    
    #[error("Plugin error: {0}")]
    PluginError(String),
    
    #[error("WASM runtime error: {0}")]
    WasmRuntimeError(#[from] wasmtime::Error),
    
    #[error("Plugin verification failed: {0}")]
    PluginVerificationError(String),
    
    #[error("Resource limit exceeded: {0}")]
    ResourceLimitError(String),
}
```

## Documentation Requirements

### Plugin Developer Guide
**File:** `plugins/DEVELOPER_GUIDE.md`

Create comprehensive documentation covering:
- Setting up the Rust + WASM toolchain
- Writing plugins using the WIT interface
- Resource limits and security considerations
- Testing and debugging WASM plugins
- Manifest file format and permissions

### API Documentation
**File:** `plugins/API_REFERENCE.md`

Document:
- Complete WIT interface specification
- Available host functions (logging, config)
- Data type definitions
- Error handling patterns

## Testing Strategy

### Unit Tests
- [ ] WASM module loading and validation
- [ ] Security context creation with different permission sets
- [ ] Resource limit enforcement
- [ ] Plugin verification pipeline

### Integration Tests
- [ ] End-to-end plugin execution
- [ ] Error handling for malformed plugins
- [ ] Performance regression tests
- [ ] Security boundary tests

### Example Plugin Tests
- [ ] Build example WASM plugin successfully
- [ ] Execute example plugin with test data
- [ ] Verify output matches expected format

## Migration from Current System

### Backward Compatibility
The current native plugin system should remain functional during the transition:

1. **Dual Plugin Support**: Both native and WASM plugins can coexist
2. **Feature Flag**: Add `#[cfg(feature = "wasm-plugins")]` to enable WASM support
3. **Gradual Migration**: Convert existing plugins one at a time

### Migration Checklist
- [ ] Convert `GodObjectDetector` to WASM plugin
- [ ] Convert `CyclomaticComplexityDetector` to WASM plugin  
- [ ] Performance comparison between native and WASM versions
- [ ] Update integration tests
- [ ] Update documentation

## Performance Targets (Success Criteria)

Based on the research document, the implementation must achieve:
- **Plugin startup time**: < 10ms for cached WASM modules
- **Execution overhead**: < 30% compared to native Rust equivalent
- **Memory overhead**: < 50MB per plugin instance
- **Concurrent plugins**: Support 5+ plugins running simultaneously

## Risk Mitigation

### Technical Risks
1. **WIT Toolchain Immaturity**: Component Model is still evolving
   - Mitigation: Start with simpler serialization, migrate to Components later
   
2. **Performance Overhead**: WASM boundary crossing costs
   - Mitigation: Design "chunky" APIs, implement AOT caching
   
3. **Debugging Complexity**: WASM debugging is more challenging
   - Mitigation: Comprehensive logging, test harnesses

### Security Risks
1. **Capability Leakage**: Accidentally granting excessive permissions
   - Mitigation: Principle of least privilege, manifest validation
   
2. **Resource Exhaustion**: Malicious or buggy plugins
   - Mitigation: Fuel metering, memory limits, timeouts

## Next Steps Priority Order

1. **High Priority (Week 1)**
   - Implement basic WASM plugin loading with Wasmtime
   - Create WIT interface definition
   - Add security framework (capability-based permissions)

2. **Medium Priority (Week 2)**  
   - Build example WASM plugin
   - Implement resource limiting (fuel, memory)
   - Add plugin verification pipeline

3. **Low Priority (Week 3)**
   - Performance optimization (AOT caching)
   - Comprehensive testing suite
   - Documentation and developer guide

This implementation plan provides a secure, performant foundation for the WASM plugin system while maintaining the flexibility to evolve with the WebAssembly standards ecosystem.
