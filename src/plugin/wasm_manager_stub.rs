// WASM plugin manager - TEMPORARILY DISABLED
// This module is currently disabled to prevent runtime crashes during development
// TODO: Re-enable once WASM plugin integration is stabilized

use crate::error::UveddiError;
use std::path::Path;

/// Stubbed WASM plugin manager for development
pub struct WasmPluginManager {
    _placeholder: (),
}

impl WasmPluginManager {
    pub fn new() -> Result<Self, UveddiError> {
        // Always return an error to prevent WASM manager creation
        Err(UveddiError::Plugin(
            "WASM plugin manager is disabled during development".to_string(),
        ))
    }

    pub fn load_plugin(&self, _wasm_path: &Path) -> Result<WasmPlugin, UveddiError> {
        Err(UveddiError::Plugin("WASM plugins are disabled".to_string()))
    }
}

/// Stubbed WASM plugin for development
pub struct WasmPlugin {
    _placeholder: (),
}

impl WasmPlugin {
    pub fn analyze(
        &mut self,
        _graph: &crate::models::dependency_graph::DependencyGraph,
    ) -> Result<Vec<uveddi_plugin_api::models::ArchitecturalIssue>, UveddiError> {
        Err(UveddiError::Plugin("WASM plugins are disabled".to_string()))
    }
}
