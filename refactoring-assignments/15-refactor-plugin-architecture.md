# Assignment 15: Simplify Plugin Architecture

## Priority: MEDIUM
## Estimated Time: 4-5 hours
## Files: `/src/plugins/` directory

## Objective
Simplify the WebAssembly plugin system to reduce complexity while maintaining extensibility.

## Current Problem
- WebAssembly runtime adds significant overhead for simple extensions
- Complex plugin discovery and loading system
- Security concerns with plugin sandboxing
- High operational complexity for basic use cases

## Tasks

### 1. Evaluate Current Plugin System

#### A. Audit Plugin Usage:
```bash
# Find all plugin-related code
find src -name "*.rs" -exec grep -l "plugin\|wasm\|wasmtime" {} \;

# Analyze plugin system complexity
wc -l src/plugins/*.rs
find src/plugins -name "*.rs" -exec wc -l {} +

# Check plugin dependencies
grep -A 20 "wasmtime\|wasmer" Cargo.toml
```

#### B. Document Current Architecture:
```markdown
# Current Plugin System Analysis

## Components:
- Plugin discovery system
- WASM runtime integration
- Plugin API definitions
- Security sandboxing
- Plugin registry

## Complexity Metrics:
- Lines of code: ___
- Dependencies: ___
- Memory overhead: ___
- Performance impact: ___
```

### 2. Design Simplified Plugin Architecture

#### A. Hybrid Plugin System:
```rust
// Support both native and WASM plugins
pub enum PluginType {
    Native(Box<dyn NativePlugin>),
    Wasm(WasmPlugin),
}

pub trait NativePlugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn detect(&self, context: &AnalysisContext) -> Result<Vec<Finding>>;
}

pub struct WasmPlugin {
    module: wasmtime::Module,
    instance: wasmtime::Instance,
}
```

#### B. Plugin Categories:
1. **Built-in Extensions**: Compile-time plugins (fast)
2. **Dynamic Libraries**: Shared libraries (.so/.dll) for performance
3. **WASM Plugins**: For untrusted/sandboxed extensions
4. **Script Plugins**: Simple Lua/JavaScript for basic rules

### 3. Implement Native Plugin System

#### A. Native Plugin Interface:
```rust
// src/plugins/native/mod.rs
pub trait NativeDetector: Send + Sync {
    fn metadata(&self) -> DetectorMetadata;
    fn detect(&self, file: &FileContext) -> Result<Vec<Finding>>;
    fn languages(&self) -> &[Language];
}

pub struct DetectorMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub category: DetectorCategory,
}

// Built-in detector registry
pub struct NativeDetectorRegistry {
    detectors: Vec<Box<dyn NativeDetector>>,
}

impl NativeDetectorRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            detectors: Vec::new(),
        };

        // Register built-in detectors
        registry.register(Box::new(GodObjectDetector::default()));
        registry.register(Box::new(SecurityVulnerabilityDetector::default()));
        registry.register(Box::new(CodeSmellDetector::default()));

        registry
    }

    pub fn register(&mut self, detector: Box<dyn NativeDetector>) {
        self.detectors.push(detector);
    }

    pub fn find_detectors(&self, language: Language) -> Vec<&dyn NativeDetector> {
        self.detectors
            .iter()
            .filter(|d| d.languages().contains(&language))
            .map(|d| d.as_ref())
            .collect()
    }
}
```

#### B. Dynamic Library Support:
```rust
// src/plugins/dynamic/mod.rs
use libloading::{Library, Symbol};

pub struct DynamicPlugin {
    _library: Library,
    detector: Box<dyn NativeDetector>,
}

impl DynamicPlugin {
    pub unsafe fn load(path: &Path) -> Result<Self> {
        let library = Library::new(path)?;

        // Load create_detector function
        let create_detector: Symbol<unsafe extern fn() -> Box<dyn NativeDetector>> =
            library.get(b"create_detector")?;

        let detector = create_detector();

        Ok(Self {
            _library: library,
            detector,
        })
    }
}

// Plugin loading from directory
pub struct DynamicPluginLoader {
    plugins: Vec<DynamicPlugin>,
}

impl DynamicPluginLoader {
    pub fn load_from_directory(dir: &Path) -> Result<Self> {
        let mut plugins = Vec::new();

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map_or(false, |ext| ext == "so" || ext == "dll") {
                match unsafe { DynamicPlugin::load(&path) } {
                    Ok(plugin) => plugins.push(plugin),
                    Err(e) => eprintln!("Failed to load plugin {:?}: {}", path, e),
                }
            }
        }

        Ok(Self { plugins })
    }
}
```

### 4. Simplify WASM Plugin System

#### A. Lightweight WASM Interface:
```rust
// src/plugins/wasm/mod.rs
use wasmtime::*;

pub struct WasmPluginEngine {
    engine: Engine,
    linker: Linker<WasmContext>,
}

pub struct WasmContext {
    memory: Option<Memory>,
    analysis_data: String,
}

impl WasmPluginEngine {
    pub fn new() -> Result<Self> {
        let engine = Engine::default();
        let mut linker = Linker::new(&engine);

        // Define minimal host functions
        linker.func_wrap("env", "log_info", |msg_ptr: i32, msg_len: i32| {
            // Host logging function
        })?;

        linker.func_wrap("env", "get_file_content", |_ctx: Caller<'_, WasmContext>| -> i32 {
            // Return pointer to file content
            0
        })?;

        Ok(Self { engine, linker })
    }

    pub fn load_plugin(&self, wasm_bytes: &[u8]) -> Result<WasmPlugin> {
        let module = Module::new(&self.engine, wasm_bytes)?;
        let mut store = Store::new(&self.engine, WasmContext {
            memory: None,
            analysis_data: String::new(),
        });

        let instance = self.linker.instantiate(&mut store, &module)?;

        Ok(WasmPlugin {
            store,
            instance,
        })
    }
}

pub struct WasmPlugin {
    store: Store<WasmContext>,
    instance: Instance,
}

impl WasmPlugin {
    pub fn detect(&mut self, file_content: &str) -> Result<Vec<Finding>> {
        // Call WASM detect function
        let detect_func = self.instance
            .get_typed_func::<(i32, i32), i32>(&mut self.store, "detect")?;

        // Prepare input data
        self.store.data_mut().analysis_data = file_content.to_string();

        // Call WASM function
        let result_ptr = detect_func.call(&mut self.store, (0, file_content.len() as i32))?;

        // Parse result (simplified)
        Ok(vec![])
    }
}
```

### 5. Create Script Plugin System

#### A. Lua Script Support:
```rust
// src/plugins/script/lua.rs
use mlua::{Lua, Result as LuaResult, Table, UserData, UserDataMethods};

pub struct LuaPlugin {
    lua: Lua,
    script_name: String,
}

impl LuaPlugin {
    pub fn new(script_path: &Path) -> Result<Self> {
        let lua = Lua::new();
        let script_content = std::fs::read_to_string(script_path)?;

        // Setup Lua environment
        let globals = lua.globals();

        // Provide file context to Lua
        let file_context = lua.create_table()?;
        file_context.set("get_content", lua.create_function(|_, ()| {
            Ok("file content here")
        })?)?;

        globals.set("file", file_context)?;

        // Execute script
        lua.load(&script_content).exec()?;

        Ok(Self {
            lua,
            script_name: script_path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string(),
        })
    }

    pub fn detect(&self, file_content: &str) -> Result<Vec<Finding>> {
        let detect_func: mlua::Function = self.lua.globals().get("detect")?;
        let results: Table = detect_func.call(file_content)?;

        // Convert Lua table to findings
        let mut findings = Vec::new();
        for pair in results.pairs::<i32, Table>() {
            let (_, finding_table) = pair?;
            let message: String = finding_table.get("message")?;
            let line: u32 = finding_table.get("line")?;

            findings.push(Finding {
                message,
                line,
                severity: Severity::Warning,
                // ... other fields
            });
        }

        Ok(findings)
    }
}

// Example Lua plugin script:
// function detect(file_content)
//     local findings = {}
//     -- Simple detection logic
//     if string.find(file_content, "TODO") then
//         table.insert(findings, {
//             message = "TODO comment found",
//             line = 1,
//             severity = "info"
//         })
//     end
//     return findings
// end
```

### 6. Implement Plugin Manager

#### A. Unified Plugin Manager:
```rust
// src/plugins/manager.rs
pub struct PluginManager {
    native_registry: NativeDetectorRegistry,
    dynamic_loader: Option<DynamicPluginLoader>,
    wasm_engine: Option<WasmPluginEngine>,
    script_plugins: Vec<LuaPlugin>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            native_registry: NativeDetectorRegistry::new(),
            dynamic_loader: None,
            wasm_engine: None,
            script_plugins: Vec::new(),
        }
    }

    pub fn load_plugins(&mut self, config: &PluginConfig) -> Result<()> {
        // Load dynamic libraries
        if let Some(plugin_dir) = &config.plugin_directory {
            self.dynamic_loader = Some(DynamicPluginLoader::load_from_directory(plugin_dir)?);
        }

        // Initialize WASM engine if needed
        if config.enable_wasm_plugins {
            self.wasm_engine = Some(WasmPluginEngine::new()?);
        }

        // Load script plugins
        if let Some(script_dir) = &config.script_directory {
            for entry in std::fs::read_dir(script_dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.extension().map_or(false, |ext| ext == "lua") {
                    match LuaPlugin::new(&path) {
                        Ok(plugin) => self.script_plugins.push(plugin),
                        Err(e) => eprintln!("Failed to load script plugin {:?}: {}", path, e),
                    }
                }
            }
        }

        Ok(())
    }

    pub fn detect_all(&self, context: &AnalysisContext) -> Result<Vec<Finding>> {
        let mut all_findings = Vec::new();

        // Run native detectors
        for detector in self.native_registry.find_detectors(context.language) {
            let findings = detector.detect(context)?;
            all_findings.extend(findings);
        }

        // Run dynamic library plugins
        if let Some(loader) = &self.dynamic_loader {
            for plugin in &loader.plugins {
                let findings = plugin.detector.detect(context)?;
                all_findings.extend(findings);
            }
        }

        // Run script plugins
        for plugin in &self.script_plugins {
            let findings = plugin.detect(&context.file_content)?;
            all_findings.extend(findings);
        }

        Ok(all_findings)
    }

    pub fn list_plugins(&self) -> Vec<PluginInfo> {
        let mut plugins = Vec::new();

        // Native plugins
        for detector in &self.native_registry.detectors {
            plugins.push(PluginInfo {
                name: detector.metadata().name,
                version: detector.metadata().version,
                plugin_type: PluginType::Native,
                enabled: true,
            });
        }

        // Script plugins
        for plugin in &self.script_plugins {
            plugins.push(PluginInfo {
                name: plugin.script_name.clone(),
                version: "1.0.0".to_string(),
                plugin_type: PluginType::Script,
                enabled: true,
            });
        }

        plugins
    }
}

pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub plugin_type: PluginType,
    pub enabled: bool,
}

pub enum PluginType {
    Native,
    Dynamic,
    Wasm,
    Script,
}
```

### 7. Create Plugin Configuration

#### A. Plugin Configuration:
```rust
// src/plugins/config.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub plugin_directory: Option<PathBuf>,
    pub script_directory: Option<PathBuf>,
    pub enable_wasm_plugins: bool,
    pub enable_dynamic_plugins: bool,
    pub plugin_whitelist: Option<Vec<String>>,
    pub plugin_blacklist: Option<Vec<String>>,
    pub security_level: PluginSecurityLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginSecurityLevel {
    Strict,    // Only native plugins
    Moderate,  // Native + script plugins
    Permissive, // All plugin types including WASM
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            plugin_directory: Some(PathBuf::from("./plugins")),
            script_directory: Some(PathBuf::from("./scripts")),
            enable_wasm_plugins: false, // Disabled by default for simplicity
            enable_dynamic_plugins: true,
            plugin_whitelist: None,
            plugin_blacklist: None,
            security_level: PluginSecurityLevel::Moderate,
        }
    }
}
```

### 8. Update Feature Flags

#### A. Granular Plugin Features:
```toml
# Update Cargo.toml
[features]
default = ["native-plugins"]

# Plugin system features
native-plugins = []
dynamic-plugins = ["libloading"]
wasm-plugins = ["wasmtime"]
script-plugins = ["mlua"]
all-plugins = ["native-plugins", "dynamic-plugins", "wasm-plugins", "script-plugins"]

# Security levels
plugin-security-strict = ["native-plugins"]
plugin-security-moderate = ["native-plugins", "script-plugins"]
plugin-security-permissive = ["all-plugins"]
```

### 9. Performance Optimization

#### A. Plugin Performance Monitoring:
```rust
// src/plugins/metrics.rs
pub struct PluginMetrics {
    pub execution_time: Duration,
    pub memory_usage: u64,
    pub findings_count: usize,
}

pub struct PluginPerformanceTracker {
    metrics: HashMap<String, Vec<PluginMetrics>>,
}

impl PluginPerformanceTracker {
    pub fn track_execution<F, R>(&mut self, plugin_name: &str, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = f();
        let execution_time = start.elapsed();

        let metrics = PluginMetrics {
            execution_time,
            memory_usage: 0, // Could implement memory tracking
            findings_count: 0, // Could be passed in
        };

        self.metrics
            .entry(plugin_name.to_string())
            .or_insert_with(Vec::new)
            .push(metrics);

        result
    }

    pub fn get_average_execution_time(&self, plugin_name: &str) -> Option<Duration> {
        self.metrics.get(plugin_name).map(|metrics| {
            let total: Duration = metrics.iter().map(|m| m.execution_time).sum();
            total / metrics.len() as u32
        })
    }
}
```

### 10. Create Plugin Development Kit

#### A. Plugin Template Generator:
```bash
#!/bin/bash
# scripts/create-plugin.sh

PLUGIN_NAME=$1
PLUGIN_TYPE=${2:-native}

if [ -z "$PLUGIN_NAME" ]; then
    echo "Usage: $0 <plugin_name> [native|script|wasm]"
    exit 1
fi

case $PLUGIN_TYPE in
    "native")
        # Generate native plugin template
        mkdir -p "plugins/$PLUGIN_NAME"
        cat > "plugins/$PLUGIN_NAME/lib.rs" << EOF
use uveddi_plugin_api::*;

pub struct ${PLUGIN_NAME}Detector;

impl NativeDetector for ${PLUGIN_NAME}Detector {
    fn metadata(&self) -> DetectorMetadata {
        DetectorMetadata {
            name: "$PLUGIN_NAME".to_string(),
            version: "1.0.0".to_string(),
            description: "Description of $PLUGIN_NAME detector".to_string(),
            author: "Your Name".to_string(),
            category: DetectorCategory::CodeQuality,
        }
    }

    fn detect(&self, context: &FileContext) -> Result<Vec<Finding>> {
        let mut findings = Vec::new();

        // TODO: Implement detection logic

        Ok(findings)
    }

    fn languages(&self) -> &[Language] {
        &[Language::Rust, Language::Python]
    }
}

#[no_mangle]
pub extern "C" fn create_detector() -> Box<dyn NativeDetector> {
    Box::new(${PLUGIN_NAME}Detector)
}
EOF
        ;;

    "script")
        # Generate Lua script template
        mkdir -p "scripts"
        cat > "scripts/$PLUGIN_NAME.lua" << EOF
function detect(file_content)
    local findings = {}

    -- TODO: Implement detection logic
    -- Example:
    -- if string.find(file_content, "pattern") then
    --     table.insert(findings, {
    --         message = "Issue found",
    --         line = 1,
    --         severity = "warning"
    --     })
    -- end

    return findings
end
EOF
        ;;
esac

echo "Plugin template created: $PLUGIN_NAME ($PLUGIN_TYPE)"
```

## Success Criteria
- [ ] Plugin system complexity reduced by 50%
- [ ] Multiple plugin types supported (native, script, WASM)
- [ ] Performance overhead <10% for native plugins
- [ ] Easy plugin development with templates
- [ ] Backwards compatibility with existing plugins
- [ ] Clear security boundaries per plugin type

## Performance Targets
- **Native plugins**: <1ms execution overhead
- **Script plugins**: <5ms execution overhead
- **WASM plugins**: <20ms execution overhead
- **Plugin loading**: <100ms for all plugins

## Verification Commands
```bash
# Test plugin system
cargo test --features all-plugins

# Benchmark plugin performance
cargo bench plugin_performance

# Test plugin loading
./scripts/test-plugin-loading.sh

# Create sample plugin
./scripts/create-plugin.sh sample_detector native
```

## Migration Guide
Document migration path for existing WASM plugins to new simplified system.

## Completion Notes
_To be filled by AI developer:_
- Plugin types implemented: ___
- Performance improvement: ___
- Complexity reduction: ___
- Migration effort for existing plugins: ___
- Security considerations addressed: ___