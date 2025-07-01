use std::sync::Arc;
use std::path::PathBuf;
use uveddi_plugin_api::models::{ArchitecturalIssue, DependencyGraph};
use uveddi_plugin_api::{Plugin, PluginError};
use crate::error::UveddiError;

mod wasm_manager;
mod security;
mod verification;

pub use wasm_manager::{WasmPluginManager, WasmPlugin};
pub use security::{PluginManifest, Permission, ResourceLimits};
pub use verification::PluginVerifier;

/// Manages the discovery and execution of plugins.
pub struct PluginManager {
    plugins: Vec<Arc<dyn Plugin>>,
    wasm_manager: Option<WasmPluginManager>,
    plugin_directory: PathBuf,
}

impl PluginManager {
    /// Creates a new, empty `PluginManager`.
    pub fn new() -> Self {
        let wasm_manager = match WasmPluginManager::new() {
            Ok(manager) => Some(manager),
            Err(e) => {
                log::warn!("Failed to initialize WASM plugin manager: {}. WASM plugins will be disabled.", e);
                None
            }
        };
        
        Self {
            plugins: Vec::new(),
            wasm_manager,
            plugin_directory: PathBuf::from("plugins/installed"),
        }
    }

    /// Registers a new plugin.
    pub fn register(&mut self, plugin: Arc<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    /// Runs all registered plugins on the given dependency graph.
    pub fn run_plugins(
        &self,
        graph: &DependencyGraph,
    ) -> Vec<Result<Vec<ArchitecturalIssue>, PluginError>> {
        let mut results = Vec::new();
        
        // Run native plugins
        for plugin in &self.plugins {
            results.push(plugin.run(graph));
        }
        
        // Run WASM plugins if available
        if let Some(wasm_manager) = &self.wasm_manager {
            let wasm_results = self.run_wasm_plugins(wasm_manager, graph);
            for result in wasm_results {
                // Convert UveddiError to PluginError for compatibility
                let converted_result = result.map_err(|e| PluginError::Execution(e.to_string()));
                results.push(converted_result);
            }
        }
        
        results
    }
    
    /// Runs WASM plugins on the given dependency graph.
    fn run_wasm_plugins(
        &self,
        wasm_manager: &WasmPluginManager,
        graph: &DependencyGraph,
    ) -> Vec<Result<Vec<ArchitecturalIssue>, UveddiError>> {
        let mut results = Vec::new();
        
        // Convert from plugin API DependencyGraph to our internal format
        let mut internal_graph = crate::models::dependency_graph::DependencyGraph::new();
        
        // For now, we'll create a simple conversion - in a full implementation
        // we'd need to properly map between the two formats
        for dep in graph.get_all_dependencies() {
            let from_module = dep.from_file.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();
            internal_graph.add_edge(from_module, dep.to_module.clone());
        }
        
        // Scan for installed WASM plugins
        if let Ok(entries) = std::fs::read_dir(&self.plugin_directory) {
            for entry in entries.flatten() {
                if let Some(extension) = entry.path().extension() {
                    if extension == "wasm" {
                        let result = self.run_single_wasm_plugin(wasm_manager, &entry.path(), &internal_graph);
                        results.push(result);
                    }
                }
            }
        }
        
        results
    }
    
    /// Runs a single WASM plugin.
    fn run_single_wasm_plugin(
        &self,
        wasm_manager: &WasmPluginManager,
        wasm_path: &PathBuf,
        _graph: &crate::models::dependency_graph::DependencyGraph,
    ) -> Result<Vec<ArchitecturalIssue>, UveddiError> {
        // Find corresponding manifest
        let manifest_path = wasm_path.with_extension("toml");
        
        // Verify plugin
        let _manifest = PluginVerifier::verify_plugin(wasm_path, &manifest_path)?;
        
        // Load and execute plugin
        let mut plugin = wasm_manager.load_plugin(wasm_path)?;
        let wasm_issues = plugin.analyze(_graph)?;
        
        // Convert WASM issues back to our internal format
        let mut issues = Vec::new();
        for wasm_issue in wasm_issues {
            // Create a new ArchitecturalIssue using the plugin API format
            let issue = ArchitecturalIssue {
                issue_id: None,
                analysis_run_id: 0,
                anti_pattern_type_id: 0,
                file_path: wasm_issue.file_path,
                start_line: Some(wasm_issue.start_line as i32),
                end_line: Some(wasm_issue.end_line as i32),
                severity: wasm_issue.severity,
                description: wasm_issue.message,
                code_snippet: wasm_issue.code_snippet,
                ai_explanation: None,
            };
            issues.push(issue);
        }
        
        Ok(issues)
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Example plugin: Detects God Objects.
struct GodObjectDetector;

impl Plugin for GodObjectDetector {
    fn name(&self) -> &'static str {
        "God Object Detector"
    }

    fn description(&self) -> &'static str {
        "Detects potential 'God Objects' in the codebase, which are objects that know too much or do too much."
    }

    fn run(
        &self,
        graph: &DependencyGraph,
    ) -> Result<Vec<ArchitecturalIssue>, PluginError> {
        // Stub implementation for now.
        // In a real implementation, this would analyze the graph
        // to find nodes with an excessive number of dependencies.
        println!("Running God Object Detector plugin...");
        Ok(vec![])
    }
}

/// Example plugin: Detects Cyclomatic Complexity
struct CyclomaticComplexityDetector;

impl Plugin for CyclomaticComplexityDetector {
    fn name(&self) -> &'static str {
        "Cyclomatic Complexity Detector"
    }

    fn description(&self) -> &'static str {
        "Detects functions with high cyclomatic complexity."
    }

    fn run(
        &self,
        _graph: &DependencyGraph,
    ) -> Result<Vec<ArchitecturalIssue>, PluginError> {
        println!("Running Cyclomatic Complexity Detector plugin...");
        Ok(vec![])
    }
}

/// Initializes the plugin manager and registers all built-in plugins.
pub fn initialize_plugins() -> PluginManager {
    let mut manager = PluginManager::new();
    manager.register(Arc::new(GodObjectDetector));
    manager.register(Arc::new(CyclomaticComplexityDetector));
    
    // Log plugin initialization status
    if manager.wasm_manager.is_some() {
        log::info!("Plugin manager initialized with WASM support");
    } else {
        log::info!("Plugin manager initialized without WASM support (native plugins only)");
    }
    
    manager
}
