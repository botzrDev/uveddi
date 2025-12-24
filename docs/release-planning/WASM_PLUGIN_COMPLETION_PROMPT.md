# WASM Plugin System Completion - Developer Prompt

## Project Context

You are working on **Uveddi**, a Rust-based code analysis tool. The project has a WASM plugin system that allows third-party plugins to extend analysis capabilities. The plugin system is partially implemented and needs to be completed for the v0.0.3 release.

## Current State Summary

The WASM plugin system is approximately **75% complete**. The infrastructure is in place, but there's a mismatch between the **core WASM module API** currently used and the **Component Model** that the example plugins expect.

### What's Working
- Plugin discovery and registry (`src/plugins/registry.rs`)
- Plugin manifest parsing and validation
- Security policy enforcement (`src/plugins/security.rs`)
- Plugin verification system (`src/plugins/verification.rs`)
- Plugin lifecycle management infrastructure (`src/plugins/lifecycle.rs`)
- CLI plugin commands (list, install, uninstall, info, verify)
- Basic plugin invocation via `WasmPluginAdapter::detect_issues_async()`
- JSON-based data serialization for plugin communication

### What's NOT Working
1. **Component Model mismatch**: Plugins use `wit_bindgen::generate!()` but runtime uses `wasmtime::Module` (core)
2. **Host functions not implemented**: `parse-ast`, `query-ast`, `log`, etc. are stubbed
3. **Knowledge plugin system**: Contains `todo!()` in loader
4. **Example plugins won't run**: They're Component Model format, runtime expects core modules

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    CLI Layer                                 │
│  src/cli/commands/plugin.rs                                  │
│  - list, install, uninstall, info, verify, stats, monitor   │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│              Application Layer                               │
│  src/application/plugin_manager.rs                           │
│  - ApplicationPluginManager orchestrates plugin operations   │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│              Plugin Engine Layer                             │
│  src/plugins/engine.rs                                       │
│  - WasmPluginEngine: main runtime orchestrator              │
│  - WasmPluginAdapter: implements AnalysisDetector trait     │
└─────────────────────────────────────────────────────────────┘
                              │
         ┌────────────────────┼────────────────────┐
         │                    │                    │
┌────────▼───────┐  ┌────────▼───────┐  ┌────────▼───────┐
│ PluginLifecycle │  │ PluginRegistry │  │  SecurityPolicy │
│    Manager      │  │                │  │                 │
│ lifecycle.rs    │  │ registry.rs    │  │ security.rs     │
└────────┬───────┘  └────────────────┘  └─────────────────┘
         │
┌────────▼───────────────────────────────────────────────────┐
│              WASM Runtime Layer                             │
│  - wasmtime::Engine, Module, Store, Instance               │
│  - Currently using CORE modules (not Component Model)       │
│  - WASI Preview 1 context configured                        │
└─────────────────────────────────────────────────────────────┘
```

---

## Key Files to Understand

| File | Purpose | Status |
|------|---------|--------|
| `src/plugins/mod.rs` | Module exports, WIT bindgen macro | Has `bindgen!()` for Component Model |
| `src/plugins/engine.rs` | WasmPluginEngine, WasmPluginAdapter | Working but uses core modules |
| `src/plugins/lifecycle.rs` | Plugin loading, ActivePlugin, analyze_file() | Needs Component Model conversion |
| `src/plugins/host_functions.rs` | HostContext, HostFunctions | Mostly stubbed |
| `src/plugins/types.rs` | PluginId, HostContext (WASM), ResourceLimits | Working |
| `src/plugins/security.rs` | SecurityPolicy, Permission | Working |
| `src/plugins/data_plane.rs` | AST serialization (JSON) | Working |
| `wit/core-analysis.wit` | WIT interface definition | Complete |
| `examples/plugins/complexity-analyzer/` | Example plugin using wit_bindgen | Won't work with current runtime |

---

## Task 1: Convert to Component Model

The biggest task is converting from core WASM modules to the Component Model.

### Current Code (lifecycle.rs:74-135)
```rust
// Currently using core Module API
let engine = wasmtime::Engine::new(&security_policy.configure_engine()?)?;
let module = wasmtime::Module::new(&engine, &binary)?;
let mut linker = wasmtime::Linker::<HostContext>::new(&engine);
// ... WASI setup ...
let instance = linker.instantiate(&mut store, &module)?;
```

### Target Code Pattern
```rust
// Should use Component Model API
use wasmtime::component::{Component, Linker};

let engine = wasmtime::Engine::new(&config)?;
let component = Component::new(&engine, &binary)?;
let mut linker = Linker::<HostContext>::new(&engine);

// Add host function implementations for WIT imports
// The bindgen!() macro in mod.rs generates traits you need to implement

let (bindings, _instance) = CoreAnalysis::instantiate(&mut store, &component, &linker)?;

// Call plugin exports via generated bindings
let result = bindings.call_analyze(&mut store, &source_file)?;
```

### Reference: WIT Interface (wit/core-analysis.wit)

```wit
// Plugin exports these functions
export initialize: func(config: plugin-config, limits: resource-limits) -> result<_, string>;
export analyze: func(file: source-file) -> result<analysis-result, string>;
export get-info: func() -> plugin-info;
export cleanup: func() -> result<_, string>;

// Plugin can call these host functions
import log: func(level: log-level, message: string);
import parse-ast: func(code: string, language: string) -> result<ast-node, string>;
import query-ast: func(node: ast-node, query: string) -> result<list<ast-node>, string>;
import read-file: func(path: string) -> result<string, string>;
// ... more imports in the WIT file
```

### Steps to Complete Task 1

1. **Update `src/plugins/types.rs`**:
   - Ensure `HostContext` implements traits required by Component Model
   - Add `wasmtime::component::ResourceTable` if needed

2. **Update `src/plugins/lifecycle.rs`**:
   - Change `wasmtime::Module` to `wasmtime::component::Component`
   - Change `wasmtime::Linker` to `wasmtime::component::Linker`
   - Use generated bindings from `mod.rs` to instantiate
   - Update `ActivePlugin` to store component bindings instead of raw instance

3. **Implement host traits in `src/plugins/host_functions.rs`**:
   - The `bindgen!()` macro generates traits for host imports
   - Implement these traits on `HostContext` or a wrapper type
   - Example:
     ```rust
     impl core_analysis::Host for HostContext {
         fn log(&mut self, level: LogLevel, message: String) {
             // Actual implementation
         }
         fn parse_ast(&mut self, code: String, language: String) -> Result<AstNode, String> {
             // Actual implementation using tree-sitter
         }
     }
     ```

4. **Update `src/plugins/engine.rs`**:
   - Modify `detect_issues_async()` to use component bindings
   - Convert between WIT types and internal Uveddi types

---

## Task 2: Implement Host Functions

The plugins need these host functions to work:

### Priority 1 (Required for basic functionality)
| Function | Purpose | Implementation Notes |
|----------|---------|---------------------|
| `log` | Plugin logging | Map to tracing crate |
| `parse-ast` | Parse code to AST | Use existing `AstParser` in `src/ast/` |
| `query-ast` | Tree-sitter queries | Use tree-sitter query API |

### Priority 2 (For full functionality)
| Function | Purpose | Implementation Notes |
|----------|---------|---------------------|
| `read-file` | File system access | Check permissions via SecurityPolicy |
| `get-config` | Configuration access | Use HostContext config store |
| `calculate-hash` | Hashing | Use sha2 or similar crate |

### Implementation Location
`src/plugins/host_functions.rs` - The `HostFunctions` struct has stub implementations. Replace with real ones.

### Example Implementation (parse-ast)
```rust
fn parse_ast(&mut self, code: String, language: String) -> Result<AstNode, String> {
    // Check permission
    self.context.check_permission(Permission::ConfigRead)?;

    // Get the appropriate tree-sitter parser
    let parser = match language.as_str() {
        "rust" => tree_sitter_rust::language(),
        "python" => tree_sitter_python::language(),
        "javascript" => tree_sitter_javascript::language(),
        // ... etc
        _ => return Err(format!("Unsupported language: {}", language)),
    };

    // Parse the code
    let mut ts_parser = tree_sitter::Parser::new();
    ts_parser.set_language(parser).map_err(|e| e.to_string())?;
    let tree = ts_parser.parse(&code, None).ok_or("Parse failed")?;

    // Convert tree-sitter Node to WIT AstNode type
    let ast_node = convert_to_wit_ast_node(tree.root_node(), &code);
    Ok(ast_node)
}
```

---

## Task 3: Fix Knowledge Plugin System

Location: `src/plugins/knowledge.rs:948`

```rust
// Current code
fn load_plugin(&mut self, path: &Path) -> Result<(), KnowledgeError> {
    todo!("Implement plugin loading")
}
```

This needs to:
1. Load the WASM component from the path
2. Instantiate with proper host context
3. Call `initialize()` export
4. Register in the knowledge plugin registry

---

## Task 4: Test with Example Plugins

After completing Tasks 1-3, verify by:

1. **Build an example plugin**:
   ```bash
   cd examples/plugins/complexity-analyzer
   cargo build --target wasm32-wasip1 --release
   ```

2. **Install the plugin**:
   ```bash
   uveddi plugin install ./target/wasm32-wasip1/release/complexity_analyzer.wasm ./manifest.toml
   ```

3. **Run analysis**:
   ```bash
   uveddi analyze ./test-project --plugins
   ```

---

## Technical References

### Wasmtime Component Model Docs
- https://docs.wasmtime.dev/api/wasmtime/component/index.html
- https://component-model.bytecodealliance.org/

### WIT Bindgen
- https://github.com/bytecodealliance/wit-bindgen

### Key Wasmtime Types
```rust
use wasmtime::component::{
    Component,      // Compiled component (replaces Module)
    Linker,         // Links host functions (component version)
    Instance,       // Running instance
    ResourceTable,  // For managing resources
};
```

### Cargo Features
- `wasm-plugins` - Enables WASM plugin support
- `cli-plugins` - CLI + plugins
- `cli-full` - Full CLI with all features

### Build Command
```bash
cargo build --features wasm-plugins
cargo build --features cli-plugins  # For CLI binary
```

---

## Expected Deliverables

1. **Modified `src/plugins/lifecycle.rs`**: Uses Component Model API
2. **Implemented host functions in `src/plugins/host_functions.rs`**: At least log, parse-ast, query-ast
3. **Updated `src/plugins/engine.rs`**: Works with component bindings
4. **Working example plugin**: complexity-analyzer runs successfully
5. **Passing tests**: `cargo test --features wasm-plugins`

---

## Constraints & Guidelines

1. **Don't break existing non-plugin functionality** - The analysis engine should work without plugins
2. **Maintain security model** - All host function calls must check permissions via SecurityPolicy
3. **Handle errors gracefully** - Plugin failures shouldn't crash the host
4. **Keep backwards compatibility** - Existing manifest format should still work
5. **Use existing types** - Leverage `ArchitecturalIssue`, `ParsedFile`, etc. where possible

---

## Questions to Ask If Stuck

1. "How do I implement the host trait generated by bindgen!()?"
2. "What's the correct way to pass complex types (like AstNode) between host and guest?"
3. "How do I handle async operations in host functions?"
4. "What's the proper error handling pattern for Component Model?"

---

## Success Criteria

The WASM plugin system is complete when:
- [ ] `cargo build --features wasm-plugins` succeeds
- [ ] `cargo test --features wasm-plugins` passes
- [ ] Example complexity-analyzer plugin compiles to WASM
- [ ] Plugin can be installed via CLI
- [ ] Plugin's `analyze()` function is called during analysis
- [ ] Plugin results appear in analysis output
- [ ] Host functions (log, parse-ast) work from plugin code
