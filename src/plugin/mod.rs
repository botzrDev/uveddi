use std::sync::Arc;
use uveddi_plugin_api::models::{ArchitecturalIssue, DependencyGraph};
use uveddi_plugin_api::{Plugin, PluginError};

/// Manages the discovery and execution of plugins.
pub struct PluginManager {
    plugins: Vec<Arc<dyn Plugin>>,
}

impl PluginManager {
    /// Creates a new, empty `PluginManager`.
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
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
        self.plugins
            .iter()
            .map(|plugin| plugin.run(graph))
            .collect()
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
    manager
}
