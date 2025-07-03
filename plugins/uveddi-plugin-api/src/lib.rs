//! Uveddi Plugin API
//!
//! This crate defines the traits and types required to implement plugins for the Uveddi analysis engine.
//! Plugins can analyze dependency graphs and report architectural issues. See [`Plugin`] for the main trait.
//!
//! # Safety
//!
//! Plugins may be loaded in a sandboxed WASM environment. Avoid unsafe code and side effects.

pub mod models;

use models::{ArchitecturalIssue, DependencyGraph};
use thiserror::Error;

/// The main trait that all Uveddi plugins must implement.
///
/// # Example
/// ```rust
/// struct MyPlugin;
/// impl uveddi_plugin_api::Plugin for MyPlugin {
///     fn name(&self) -> &'static str { "my_plugin" }
///     fn description(&self) -> &'static str { "Detects custom issues" }
///     fn run(&self, graph: &uveddi_plugin_api::models::DependencyGraph)
///         -> Result<Vec<uveddi_plugin_api::models::ArchitecturalIssue>, uveddi_plugin_api::PluginError> {
///         Ok(vec![])
///     }
/// }
/// ```
///
/// # Safety
/// Plugins may be loaded in a sandboxed WASM environment. Avoid unsafe code and side effects.
pub trait Plugin {
    /// A unique name for the plugin.
    fn name(&self) -> &'static str;

    /// A description of what the plugin does.
    fn description(&self) -> &'static str;

    /// The main analysis function for the plugin.
    ///
    /// # Errors
    /// Returns `PluginError` if analysis fails.
    fn run(
        &self,
        graph: &DependencyGraph,
    ) -> Result<Vec<ArchitecturalIssue>, PluginError>;
}

/// Error type for plugin execution failures
#[derive(Debug, Error)]
pub enum PluginError {
    #[error("Plugin execution failed: {0}")]
    Execution(String),
}

impl PluginError {
    /// Create a new plugin error with a message
    pub fn new(message: String) -> Self {
        PluginError::Execution(message)
    }
}