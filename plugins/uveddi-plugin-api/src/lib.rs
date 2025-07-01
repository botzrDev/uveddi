pub mod models;

use models::{ArchitecturalIssue, DependencyGraph};
use thiserror::Error;

/// The main trait that all Uveddi plugins must implement.
pub trait Plugin {
    /// A unique name for the plugin.
    fn name(&self) -> &'static str;

    /// A description of what the plugin does.
    fn description(&self) -> &'static str;

    /// The main analysis function for the plugin.
    fn run(
        &self,
        graph: &DependencyGraph,
    ) -> Result<Vec<ArchitecturalIssue>, PluginError>;
}

#[derive(Debug, Error)]
pub enum PluginError {
    #[error("Plugin execution failed: {0}")]
    Execution(String),
}

impl PluginError {
    pub fn new(message: String) -> Self {
        PluginError::Execution(message)
    }
}