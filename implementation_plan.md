# WASM Plugin System Completion - Implementation Plan

This plan outlines the changes needed to complete the WASM plugin system for Uveddi v0.0.3, converting from core WASM modules to the WebAssembly Component Model.

## Problem Summary

The plugin system is ~75% complete but has a **Component Model mismatch**:
- **Example plugins** use `wit_bindgen::generate!()` and export Component Model interfaces
- **Runtime** uses `wasmtime::Module` (core modules) instead of `wasmtime::component::Component`
- **Host functions** are mostly stubbed with placeholder implementations

## User Review Required

> [!IMPORTANT]
> **Breaking Change**: The existing [HostContext](file:///home/austingreen/Documents/botzr/projects/uveddi/src/plugins/host_functions.rs#23-37) type in [host_functions.rs](file:///home/austingreen/Documents/botzr/projects/uveddi/src/plugins/host_functions.rs) conflicts with the one in [types.rs](file:///home/austingreen/Documents/botzr/projects/uveddi/src/plugins/types.rs). I will consolidate these into a single, unified type in [types.rs](file:///home/austingreen/Documents/botzr/projects/uveddi/src/plugins/types.rs) that works with both WASI and the Component Model host traits.

> [!WARNING]
> **Plugin API Version**: The example plugins use `wit-bindgen = "0.30.0"` but the main crate uses `wit-bindgen = "0.44.0"`. I recommend updating the example plugins to match the main crate version for compatibility.

---

## Proposed Changes

### Component 1: Type System Updates

#### [MODIFY] [types.rs](file:///home/austingreen/Documents/botzr/projects/uveddi/src/plugins/types.rs)

Consolidate [HostContext](file:///home/austingreen/Documents/botzr/projects/uveddi/src/plugins/host_functions.rs#23-37) to work with both WASI and Component Model:
- Add component-model-specific fields
- Ensure `WasiView` trait implementation remains compatible
- Add any necessary resource table fields

---

### Component 2: Component Model Migration

#### [MODIFY] [lifecycle.rs](file:///home/austingreen/Documents/botzr/projects/uveddi/src/plugins/lifecycle.rs)

Convert from core WASM modules to Component Model:

```diff
-use wasmtime::{Engine, Linker, Module, Store};
+use wasmtime::component::{Component, Linker};

// In load_plugin():
-let module = wasmtime::Module::new(&engine, &binary)?;
-let mut linker = wasmtime::Linker::<HostContext>::new(&engine);
+let component = Component::new(&engine, &binary)?;
+let mut linker = Linker::<HostContext>::new(&engine);

// Instantiation using generated bindings:
-let instance = linker.instantiate(&mut store, &module)?;
+let (bindings, _instance) = CoreAnalysis::instantiate(&mut store, &component, &linker)?;
```

Update [ActivePlugin](file:///home/austingreen/Documents/botzr/projects/uveddi/src/plugins/lifecycle.rs#275-294) struct:
- Store generated bindings instead of raw `wasmtime::Instance`
- Remove direct memory/function access patterns
- Use typed binding methods for plugin calls

#### [MODIFY] [mod.rs](file:///home/austingreen/Documents/botzr/projects/uveddi/src/plugins/mod.rs)

The `bindgen!()` macro is already present but needs the host trait implementations:

```rust
#[cfg(feature = "wasm-plugins")]
pub mod wasm {
    wasmtime::component::bindgen!({
        path: "wit/core-analysis.wit",
        world: "core-analysis",
    });
}
```

---

### Component 3: Host Function Implementation

#### [MODIFY] [host_functions.rs](file:///home/austingreen/Documents/botzr/projects/uveddi/src/plugins/host_functions.rs)

The current [HostContext](file:///home/austingreen/Documents/botzr/projects/uveddi/src/plugins/host_functions.rs#23-37) in this file conflicts with [types.rs](file:///home/austingreen/Documents/botzr/projects/uveddi/src/plugins/types.rs). Changes:
1. Remove duplicate [HostContext](file:///home/austingreen/Documents/botzr/projects/uveddi/src/plugins/host_functions.rs#23-37) definition
2. Implement the WIT-generated host trait on the unified [HostContext](file:///home/austingreen/Documents/botzr/projects/uveddi/src/plugins/host_functions.rs#23-37) from [types.rs](file:///home/austingreen/Documents/botzr/projects/uveddi/src/plugins/types.rs)

**Priority 1 Host Functions:**

| Function | Implementation |
|----------|---------------|
| [log(level, message)](file:///home/austingreen/Documents/botzr/projects/uveddi/src/plugins/host_functions.rs#294-308) | Map to `tracing` crate |
| `parse-ast(code, language)` | Use tree-sitter parsers |
| `query-ast(node, query)` | Use tree-sitter query API |

**Priority 2 Host Functions:**

| Function | Implementation |
|----------|---------------|
| `read-file(path)` | Check permissions via [SecurityPolicy](file:///home/austingreen/Documents/botzr/projects/uveddi/src/plugins/security.rs#12-26), then read |
| `get-config(key)` | Access config HashMap in HostContext |
| `calculate-hash(algorithm, content)` | Use `sha2` crate |

Example implementation for [log](file:///home/austingreen/Documents/botzr/projects/uveddi/src/plugins/host_functions.rs#294-308):
```rust
impl core_analysis::Host for HostContext {
    fn log(&mut self, level: LogLevel, message: String) {
        match level {
            LogLevel::Trace => tracing::trace!("[Plugin] {}", message),
            LogLevel::Debug => tracing::debug!("[Plugin] {}", message),
            LogLevel::Info => tracing::info!("[Plugin] {}", message),
            LogLevel::Warn => tracing::warn!("[Plugin] {}", message),
            LogLevel::Error => tracing::error!("[Plugin] {}", message),
        }
    }
    // ... other host functions
}
```

---

### Component 4: Engine Integration

#### [MODIFY] [engine.rs](file:///home/austingreen/Documents/botzr/projects/uveddi/src/plugins/engine.rs)

Update [detect_issues_async()](file:///home/austingreen/Documents/botzr/projects/uveddi/src/plugins/engine.rs#386-447) to use component bindings:
- Call `bindings.call_analyze()` instead of manual memory manipulation
- Convert between WIT types and internal Uveddi types
- Handle result conversions properly

---

### Component 5: Knowledge Plugin System

#### [MODIFY] [knowledge.rs](file:///home/austingreen/Documents/botzr/projects/uveddi/src/plugins/knowledge.rs)

Fix the `todo!()` at line 948:
```rust
pub async fn load_plugin(
    &self,
    plugin_path: &PathBuf,
) -> Result<KnowledgePluginPackage, PluginError> {
    // 1. Read plugin binary
    let binary = tokio::fs::read(plugin_path).await
        .map_err(|e| PluginError::Loading(e.to_string()))?;
    
    // 2. Parse manifest from sibling .toml file
    let manifest_path = plugin_path.with_extension("toml");
    let manifest = self.load_manifest(&manifest_path).await?;
    
    // 3. Create Component and instantiate
    // ... Component Model instantiation
    
    Ok(KnowledgePluginPackage { metadata, implementation })
}
```

---

### Component 6: Example Plugin Updates

#### [MODIFY] [Cargo.toml](file:///home/austingreen/Documents/botzr/projects/uveddi/examples/plugins/complexity-analyzer/Cargo.toml)

Update wit-bindgen version for compatibility:
```diff
[dependencies]
-wit-bindgen = "0.30.0"
+wit-bindgen = "0.44.0"
```

---

## Verification Plan

### Automated Tests

1. **Build verification**:
```bash
cargo build --features wasm-plugins
cargo test --features wasm-plugins
```

2. **Plugin build verification**:
```bash
cd examples/plugins/complexity-analyzer
cargo build --target wasm32-wasip1 --release
```

### Manual Verification

1. **Install and run plugin**:
```bash
uveddi plugin install ./target/wasm32-wasip1/release/complexity_analyzer.wasm ./plugin.toml
uveddi analyze ./test-project --plugins
```

2. **Verify output** contains plugin-detected issues

---

## Implementation Order

1. **First**: Update [types.rs](file:///home/austingreen/Documents/botzr/projects/uveddi/src/plugins/types.rs) - consolidate HostContext
2. **Second**: Implement host traits in [host_functions.rs](file:///home/austingreen/Documents/botzr/projects/uveddi/src/plugins/host_functions.rs) 
3. **Third**: Update [lifecycle.rs](file:///home/austingreen/Documents/botzr/projects/uveddi/src/plugins/lifecycle.rs) - Component Model migration
4. **Fourth**: Update [engine.rs](file:///home/austingreen/Documents/botzr/projects/uveddi/src/plugins/engine.rs) - use new bindings
5. **Fifth**: Fix [knowledge.rs](file:///home/austingreen/Documents/botzr/projects/uveddi/src/plugins/knowledge.rs) loader
6. **Sixth**: Update example plugin, build and test

---

## Risk Assessment

| Risk | Mitigation |
|------|-----------|
| wasmtime API changes | Pin to wasmtime 36.0.1 already in Cargo.toml |
| WIT bindgen compatibility | Align versions between host and guest |
| Breaking existing non-plugin code | Feature-gate all changes with `#[cfg(feature = "wasm-plugins")]` |
| Performance regression | Benchmark before/after if needed |
