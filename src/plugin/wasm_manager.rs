use wasmtime::{Engine, Config, Store};
use wasmtime::component::{Component, Linker, Instance, ResourceTable};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiView, add_to_linker_sync};
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
   
    pub description: String,
    pub author: String,
}

// Host state for WASI component model
pub struct PluginState {
    pub wasi_ctx: WasiCtx,
    pub resource_table: ResourceTable,
}

impl WasiView for PluginState {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi_ctx
    }
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.resource_table
    }
}

pub struct WasmPluginManager {
    engine: Engine,
    linker: Linker<PluginState>,
}

impl WasmPluginManager {
    pub fn new() -> Result<Self, UveddiError> {
        let mut config = Config::new();
        config.consume_fuel(true);
        config.wasm_component_model(true);
        let engine = Engine::new(&config)?;
        let mut linker = Linker::new(&engine);
        add_to_linker_sync(&mut linker)?;
        Ok(Self { engine, linker })
    }

    pub fn load_plugin(&self, wasm_path: &Path) -> Result<WasmPlugin, UveddiError> {
        let component = Component::from_file(&self.engine, wasm_path)?;
        let wasi_ctx = WasiCtxBuilder::new().inherit_stdio().build();
        let resource_table = ResourceTable::new();
        let state = PluginState { wasi_ctx, resource_table };
        let mut store = Store::new(&self.engine, state);
        let instance = self.linker.instantiate(&mut store, &component)?;
        Ok(WasmPlugin {
            store,
            instance,
            component,
        })
    }
}

pub struct WasmPlugin {
    store: Store<PluginState>,
    instance: Instance,
    component: Component,
}

impl WasmPlugin {
    pub fn get_info(&mut self) -> Result<WasmPluginInfo, UveddiError> {
        Ok(WasmPluginInfo {
            name: "WASM Plugin".to_string(),
            description: "A WASM-based plugin".to_string(),
            author: "Unknown".to_string(),
        })
    }
    pub fn analyze(&mut self, dependencies: &DependencyGraph) -> Result<Vec<ArchitecturalIssue>, UveddiError> {
        let wasm_deps: Vec<WasmDependency> = dependencies.edges.iter().map(|(from, to)| {
            WasmDependency {
                from_module: from.clone(),
                to_module: to.clone(),
                dependency_type: "import".to_string(),
            }
        }).collect();
        log::info!("WASM plugin analyze called with {} dependencies", wasm_deps.len());
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