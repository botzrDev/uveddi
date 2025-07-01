use wasmtime::{Engine, Linker, Module, Store, Instance, Config};
use wasmtime_wasi::{WasiPreview1Ctx, WasiPreview1CtxBuilder};
use std::path::Path;
use crate::models::dependency_graph::DependencyGraph;
use crate::database::models::ArchitecturalIssue;
use crate::error::UveddiError;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct WasmDependency {
    pub from_module: String,
    pub to_module: String,
    pub dependency_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WasmArchitecturalIssue {
    pub file_path: String,
    pub start_line: u32,
    pub end_line: u32,
    pub issue_type: String,
    pub severity: String,
    pub message: String,
    pub code_snippet: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WasmPluginInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
}

pub struct WasmPluginManager {
    engine: Engine,
    linker: Linker<WasiPreview1Ctx>,
}

impl WasmPluginManager {
    pub fn new() -> Result<Self, UveddiError> {
        let mut config = Config::new();
        config.consume_fuel(true);
        config.wasm_component_model(true);
        
        let engine = Engine::new(&config)?;
        let mut linker = Linker::new(&engine);
        
        // Add WASI support
        wasmtime_wasi::preview1::wasi_snapshot_preview1::add_to_linker(&mut linker, |s| s)?;
        
        // Add host functions (logging, config)
        linker.func_wrap(
            "logging", 
            "log", 
            |_caller: wasmtime::Caller<WasiPreview1Ctx>, level: i32, message_ptr: i32, message_len: i32| -> i32 {
                // For now, just log to console - in a real implementation we'd extract string from memory
                match level {
                    0 => log::debug!("[Plugin] Message at ptr={}, len={}", message_ptr, message_len),
                    1 => log::info!("[Plugin] Message at ptr={}, len={}", message_ptr, message_len),
                    2 => log::warn!("[Plugin] Message at ptr={}, len={}", message_ptr, message_len),
                    3 => log::error!("[Plugin] Message at ptr={}, len={}", message_ptr, message_len),
                    _ => log::debug!("[Plugin] Message at ptr={}, len={}", message_ptr, message_len),
                }
                0 // Success
            }
        )?;
        
        linker.func_wrap(
            "config",
            "get-value",
            |_caller: wasmtime::Caller<WasiPreview1Ctx>, key_ptr: i32, key_len: i32| -> i32 {
                // For now return -1 (not found) - in real implementation we'd extract key and lookup
                log::debug!("[Plugin] Config lookup for key at ptr={}, len={}", key_ptr, key_len);
                -1 // Not found
            }
        )?;
        
        Ok(Self { engine, linker })
    }
    
    pub fn load_plugin(&self, wasm_path: &Path) -> Result<WasmPlugin, UveddiError> {
        let module = Module::from_file(&self.engine, wasm_path)?;
        
        // Create WASI context with minimal permissions
        let wasi = WasiPreview1CtxBuilder::new()
            .inherit_stdio()
            .build();
        let mut store = Store::new(&self.engine, wasi);
        // Remove or update add_fuel if not available in new API
        // store.add_fuel(1_000_000)?; // 1M instructions limit
        
        let instance = self.linker.instantiate(&mut store, &module)?;
        
        Ok(WasmPlugin {
            store,
            instance,
            module,
        })
    }
}

pub struct WasmPlugin {
    store: Store<WasiPreview1Ctx>,
    instance: Instance,
    module: Module,
}

impl WasmPlugin {
    pub fn get_info(&mut self) -> Result<WasmPluginInfo, UveddiError> {
        // For now, return a default - in real implementation this would call the WASM function
        Ok(WasmPluginInfo {
            name: "WASM Plugin".to_string(),
            version: "0.1.0".to_string(), 
            description: "A WASM-based plugin".to_string(),
            author: "Unknown".to_string(),
        })
    }
    
    pub fn analyze(&mut self, dependencies: &DependencyGraph) -> Result<Vec<ArchitecturalIssue>, UveddiError> {
        // Convert our dependency graph format to WASM format
        let wasm_deps: Vec<WasmDependency> = dependencies.edges.iter().map(|(from, to)| {
            WasmDependency {
                from_module: from.clone(),
                to_module: to.clone(),
                dependency_type: "import".to_string(),
            }
        }).collect();
        
        // For now, return empty results - in real implementation this would:
        // 1. Serialize dependencies to WASM memory
        // 2. Call the plugin's analyze function
        // 3. Deserialize and return results
        log::info!("WASM plugin analyze called with {} dependencies", wasm_deps.len());
        
        // Return stub result for testing
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_wasm_manager_creation() {
        let manager = WasmPluginManager::new();
        assert!(manager.is_ok());
    }
    
    #[test]  
    fn test_dependency_conversion() {
        let mut graph = DependencyGraph::new();
        graph.add_edge("module_a".to_string(), "module_b".to_string());
        
        let deps: Vec<WasmDependency> = graph.edges.iter().map(|(from, to)| {
            WasmDependency {
                from_module: from.clone(),
                to_module: to.clone(),
                dependency_type: "import".to_string(),
            }
        }).collect();
        
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].from_module, "module_a");
        assert_eq!(deps[0].to_module, "module_b");
    }
}