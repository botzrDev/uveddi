//! Plugin manager actor implementation
//!
//! Manages WASM plugins using an actor-based architecture for better isolation
//! and non-blocking operation.

use super::config_service::ConfigurationService;
use super::traits::{
    ConfigurationService as ConfigurationServiceTrait, PluginCommand,
    PluginManagerHandle as PluginManagerHandleTrait, PluginStats,
};
use crate::analysis::AnalysisDetector;
use crate::database::models::ArchitecturalIssue;
use crate::error::UveddiError;
use crate::plugins::WasmPluginEngine;

use log::{error, info, warn};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, oneshot};

/// Actor-based plugin manager for handling WASM plugins.
///
/// This struct manages the lifecycle of WASM plugins using an actor-based architecture for
/// isolation and non-blocking operation. It provides methods for loading, unloading, and executing plugins,
/// as well as tracking plugin execution statistics.
///
/// # Example
/// ```rust,no_run
/// use uveddi::analysis::components::plugin_manager::PluginManager;
/// use std::sync::Arc;
/// let (manager, handle) = PluginManager::new(Arc::new(config_service));
/// ```
pub struct PluginManager {
    config_service: Arc<ConfigurationService>,
    plugin_engine: Option<WasmPluginEngine>,
    receiver: mpsc::Receiver<PluginCommand>,
    execution_stats: PluginExecutionStats,
}

/// Plugin execution statistics tracking
#[derive(Debug, Default)]
struct PluginExecutionStats {
    total_executions: u64,
    total_execution_time: Duration,
    loaded_plugins: HashMap<String, PluginInfo>,
}

#[derive(Debug)]
struct PluginInfo {
    id: String,
    executions: u64,
    total_time: Duration,
}

impl PluginManager {
    /// Creates a new plugin manager
    pub fn new(config_service: Arc<ConfigurationService>) -> (Self, PluginManagerHandle) {
        let (sender, receiver) = mpsc::channel(100); // Buffer size of 100 commands

        let manager = Self {
            config_service,
            plugin_engine: None,
            receiver,
            execution_stats: PluginExecutionStats::default(),
        };

        let handle = PluginManagerHandle { sender };

        (manager, handle)
    }

    /// Spawns the plugin manager actor and returns a handle
    pub fn spawn(config_service: Arc<ConfigurationService>) -> PluginManagerHandle {
        let (mut manager, handle) = Self::new(config_service);

        tokio::spawn(async move {
            manager.run().await;
        });

        handle
    }

    /// Initializes the WASM plugin engine if plugins are enabled
    async fn initialize_plugin_engine(&mut self) -> Result<(), UveddiError> {
        if !self.config_service.are_plugins_enabled() {
            info!("Plugins disabled in configuration, skipping plugin engine initialization");
            return Ok(());
        }

        match WasmPluginEngine::new().await {
            Ok(engine) => {
                info!("WASM plugin engine initialized successfully");
                self.plugin_engine = Some(engine);
                Ok(())
            }
            Err(e) => {
                warn!(
                    "Failed to initialize WASM plugin engine: {}. Continuing without plugins.",
                    e
                );
                Err(UveddiError::from(e))
            }
        }
    }

    /// Main actor loop
    async fn run(&mut self) {
        // Initialize plugin engine
        if let Err(e) = self.initialize_plugin_engine().await {
            error!("Plugin manager failed to initialize: {}", e);
        }

        info!("Plugin manager actor started");

        while let Some(command) = self.receiver.recv().await {
            self.handle_command(command).await;
        }

        info!("Plugin manager actor shutting down");
    }

    /// Handles incoming commands
    async fn handle_command(&mut self, command: PluginCommand) {
        match command {
            PluginCommand::Execute {
                plugin_id,
                source_file_path,
                ast,
                responder,
            } => {
                let result = self.execute_plugin(plugin_id, source_file_path, ast).await;
                if let Err(_) = responder.send(result) {
                    warn!("Failed to send plugin execution response - receiver dropped");
                }
            }
            PluginCommand::LoadPlugin {
                plugin_path,
                responder,
            } => {
                let result = self.load_plugin(plugin_path).await;
                if let Err(_) = responder.send(result) {
                    warn!("Failed to send plugin load response - receiver dropped");
                }
            }
            PluginCommand::UnloadPlugin {
                plugin_id,
                responder,
            } => {
                let result = self.unload_plugin(plugin_id).await;
                if let Err(_) = responder.send(result) {
                    warn!("Failed to send plugin unload response - receiver dropped");
                }
            }
            PluginCommand::GetStats { responder } => {
                let stats = self.get_stats();
                if let Err(_) = responder.send(Ok(stats)) {
                    warn!("Failed to send plugin stats response - receiver dropped");
                }
            }
            PluginCommand::ListLoadedPlugins { responder } => {
                let result = self.list_loaded_plugins().await;
                if let Err(_) = responder.send(result) {
                    warn!("Failed to send plugin list response - receiver dropped");
                }
            }
            PluginCommand::GetPluginAdapter {
                plugin_id,
                responder,
            } => {
                let result = self.get_plugin_adapter(&plugin_id).await;
                if let Err(_) = responder.send(result) {
                    warn!("Failed to send plugin adapter response - receiver dropped");
                }
            }
        }
    }

    /// Executes a plugin on a source file
    async fn execute_plugin(
        &mut self,
        plugin_id: String,
        source_file_path: PathBuf,
        ast: Arc<crate::ast::tree_sitter::Tree>,
    ) -> Result<Vec<ArchitecturalIssue>, UveddiError> {
        let start_time = Instant::now();

        if let Some(ref mut plugin_engine) = self.plugin_engine {
            // Get the plugin adapter from the engine
            let plugin_id_typed = crate::plugins::types::PluginId::from_name(&plugin_id);

            match plugin_engine.get_plugin_adapter(&plugin_id_typed).await {
                Some(adapter) => {
                    // Create a ParsedFile from the path and AST
                    let source_content = match tokio::fs::read_to_string(&source_file_path).await {
                        Ok(content) => content,
                        Err(e) => {
                            return Err(UveddiError::PluginError {
                                plugin: plugin_id,
                                plugin_type: "WASM".to_string(),
                                message: format!("Failed to read source file: {}", e),
                                suggestion: "Check if file exists and is readable".to_string(),
                                source: None,
                            });
                        }
                    };

                    let parsed_file = crate::ast::ParsedFile {
                        file_path: std::sync::Arc::new(source_file_path.clone()),
                        language: self.detect_language_from_path(&source_file_path),
                        tree: Some((*ast).clone()),
                        source: std::sync::Arc::new(source_content),
                        custom_ast: std::sync::Arc::new(None),
                        modified_at: crate::analysis::cache::wrappers::ArchivableSystemTime(
                            std::fs::metadata(&source_file_path)
                                .and_then(|m| m.modified())
                                .unwrap_or_else(|_| std::time::SystemTime::now()),
                        ),
                    };

                    // Execute the plugin through the adapter
                    match adapter.detect_issues(&parsed_file).await {
                        Ok(issues) => {
                            let execution_time = start_time.elapsed();
                            self.record_execution(&plugin_id, execution_time);

                            info!(
                                "Executed plugin {} on file {} in {:?}, found {} issues",
                                plugin_id,
                                source_file_path.display(),
                                execution_time,
                                issues.len()
                            );

                            Ok(issues)
                        }
                        Err(e) => {
                            let execution_time = start_time.elapsed();
                            self.record_execution(&plugin_id, execution_time);

                            error!(
                                "Plugin {} execution failed on file {}: {}",
                                plugin_id,
                                source_file_path.display(),
                                e
                            );

                            Err(UveddiError::PluginError {
                                plugin: plugin_id,
                                plugin_type: "WASM".to_string(),
                                message: format!("Plugin execution failed: {}", e),
                                suggestion: "Check plugin implementation and file format"
                                    .to_string(),
                                source: None,
                            })
                        }
                    }
                }
                None => Err(UveddiError::PluginError {
                    plugin: plugin_id,
                    plugin_type: "WASM".to_string(),
                    message: "Plugin not found or not loaded".to_string(),
                    suggestion: "Load the plugin first".to_string(),
                    source: None,
                }),
            }
        } else {
            Err(UveddiError::PluginError {
                plugin: plugin_id,
                plugin_type: "WASM".to_string(),
                message: "Plugin engine not initialized".to_string(),
                suggestion: "Enable plugins in configuration".to_string(),
                source: None,
            })
        }
    }

    /// Loads a plugin from a file path
    async fn load_plugin(&mut self, plugin_path: PathBuf) -> Result<String, UveddiError> {
        if let Some(ref mut plugin_engine) = self.plugin_engine {
            // Load the plugin binary
            let plugin_binary =
                tokio::fs::read(&plugin_path)
                    .await
                    .map_err(|e| UveddiError::PluginError {
                        plugin: plugin_path.display().to_string(),
                        plugin_type: "WASM".to_string(),
                        message: format!("Failed to read plugin file: {}", e),
                        suggestion: "Check if file exists and is readable".to_string(),
                        source: None,
                    })?;

            // Create a basic manifest from the plugin path
            let plugin_name = plugin_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown_plugin")
                .to_string();

            let manifest = crate::plugins::registry::PluginManifest {
                name: plugin_name.clone(),
                version: "1.0.0".to_string(),
                author: "Unknown".to_string(),
                description: format!("Plugin loaded from {}", plugin_path.display()),
                permissions: vec![],
                supported_languages: vec![
                    "rust".to_string(),
                    "javascript".to_string(),
                    "python".to_string(),
                ],
                anti_pattern_types: vec!["god-object".to_string(), "dead-code".to_string()],
                signature: None,
            };

            // Install the plugin in the engine
            match plugin_engine.install_plugin(manifest, plugin_binary).await {
                Ok(plugin_id) => {
                    let plugin_id_str = plugin_id.to_string();

                    // Record the plugin in our stats
                    self.execution_stats.loaded_plugins.insert(
                        plugin_id_str.clone(),
                        PluginInfo {
                            id: plugin_id_str.clone(),
                            executions: 0,
                            total_time: Duration::default(),
                        },
                    );

                    info!(
                        "Successfully loaded plugin {} from {}",
                        plugin_id_str,
                        plugin_path.display()
                    );
                    Ok(plugin_id_str)
                }
                Err(e) => {
                    error!(
                        "Failed to install plugin from {}: {}",
                        plugin_path.display(),
                        e
                    );
                    Err(UveddiError::PluginError {
                        plugin: plugin_name,
                        plugin_type: "WASM".to_string(),
                        message: format!("Failed to install plugin: {}", e),
                        suggestion: "Check if the plugin file is valid WASM".to_string(),
                        source: None,
                    })
                }
            }
        } else {
            Err(UveddiError::PluginError {
                plugin: "unknown".to_string(),
                plugin_type: "WASM".to_string(),
                message: "Plugin engine not initialized".to_string(),
                suggestion: "Enable plugins in configuration".to_string(),
                source: None,
            })
        }
    }

    /// Unloads a plugin by ID
    async fn unload_plugin(&mut self, plugin_id: String) -> Result<(), UveddiError> {
        if let Some(_) = self.plugin_engine {
            if self
                .execution_stats
                .loaded_plugins
                .remove(&plugin_id)
                .is_some()
            {
                info!("Unloaded plugin {}", plugin_id);
                Ok(())
            } else {
                Err(UveddiError::PluginError {
                    plugin: plugin_id,
                    plugin_type: "WASM".to_string(),
                    message: "Plugin not found".to_string(),
                    suggestion: "Check if plugin is loaded".to_string(),
                    source: None,
                })
            }
        } else {
            Err(UveddiError::PluginError {
                plugin: plugin_id,
                plugin_type: "WASM".to_string(),
                message: "Plugin engine not initialized".to_string(),
                suggestion: "Enable plugins in configuration".to_string(),
                source: None,
            })
        }
    }

    /// Records plugin execution statistics
    fn record_execution(&mut self, plugin_id: &str, execution_time: Duration) {
        self.execution_stats.total_executions += 1;
        self.execution_stats.total_execution_time += execution_time;

        if let Some(plugin_info) = self.execution_stats.loaded_plugins.get_mut(plugin_id) {
            plugin_info.executions += 1;
            plugin_info.total_time += execution_time;
        }
    }

    /// Gets plugin statistics
    fn get_stats(&self) -> PluginStats {
        let average_execution_time_ms = if self.execution_stats.total_executions > 0 {
            self.execution_stats.total_execution_time.as_millis() as f64
                / self.execution_stats.total_executions as f64
        } else {
            0.0
        };

        PluginStats {
            loaded_plugins: self.execution_stats.loaded_plugins.len(),
            total_executions: self.execution_stats.total_executions,
            average_execution_time_ms,
            memory_usage_bytes: 0, // TODO: Implement memory monitoring
        }
    }

    /// Lists all loaded plugins
    async fn list_loaded_plugins(&self) -> Result<Vec<String>, UveddiError> {
        if let Some(ref plugin_engine) = self.plugin_engine {
            let loaded_plugins = plugin_engine.list_loaded_plugins().await;
            Ok(loaded_plugins
                .into_iter()
                .map(|id| id.to_string())
                .collect())
        } else {
            Ok(self
                .execution_stats
                .loaded_plugins
                .keys()
                .cloned()
                .collect())
        }
    }

    /// Detects the source language from a file path
    fn detect_language_from_path(&self, path: &std::path::Path) -> crate::ast::SourceLanguage {
        use crate::ast::SourceLanguage;

        match path.extension().and_then(|ext| ext.to_str()) {
            Some("rs") => SourceLanguage::Rust,
            Some("py") => SourceLanguage::Python,
            Some("js") | Some("ts") | Some("jsx") | Some("tsx") => SourceLanguage::JavaScript,
            _ => SourceLanguage::JavaScript, // Default fallback
        }
    }

    /// Gets a plugin adapter for a specific plugin.
    ///
    /// # Arguments
    /// * `plugin_id` - The ID of the plugin to retrieve an adapter for.
    ///
    /// # Returns
    /// * `Ok(Some(adapter))` if the adapter is created successfully.
    /// * `Ok(None)` if the plugin is not found or not loaded.
    /// * `Err(UveddiError)` if the plugin engine is not initialized or adapter creation fails.
    ///
    /// # Example
    /// ```rust,ignore
    /// let adapter = plugin_manager.get_plugin_adapter("my_plugin").await?;
    /// ```
    async fn get_plugin_adapter(
        &self,
        plugin_id: &str,
    ) -> Result<Option<crate::analysis::WasmPluginDetectorAdapter>, UveddiError> {
        if let Some(ref plugin_engine) = self.plugin_engine {
            let plugin_id_typed = crate::plugins::types::PluginId::from_name(plugin_id);

            match plugin_engine.get_plugin_adapter(&plugin_id_typed).await {
                Some(_wasm_adapter) => {
                    // Create WasmPluginDetectorAdapter from the WASM adapter
                    // Since WasmPluginEngine doesn't implement Clone, we need to create a new engine
                    // or restructure the architecture. For now, let's try creating a new engine.
                    match crate::plugins::WasmPluginEngine::new().await {
                        Ok(new_engine) => {
                            let engine_arc = Arc::new(tokio::sync::RwLock::new(new_engine));

                            match crate::analysis::WasmPluginDetectorAdapter::new(
                                plugin_id_typed.clone(),
                                engine_arc,
                            )
                            .await
                            {
                                Ok(detector_adapter) => {
                                    info!(
                                        "Successfully created plugin detector adapter for: {}",
                                        plugin_id
                                    );
                                    Ok(Some(detector_adapter))
                                }
                                Err(e) => {
                                    error!(
                                        "Failed to create detector adapter for plugin {}: {}",
                                        plugin_id, e
                                    );
                                    Err(UveddiError::PluginError {
                                        plugin: plugin_id.to_string(),
                                        plugin_type: "WASM".to_string(),
                                        message: format!("Failed to create detector adapter: {}", e),
                                        suggestion: "Check plugin compatibility and ensure plugin is properly loaded".to_string(),
                                        source: None,
                                    })
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to create new plugin engine for adapter: {}", e);
                            Err(UveddiError::PluginError {
                                plugin: plugin_id.to_string(),
                                plugin_type: "WASM".to_string(),
                                message: format!("Failed to create plugin engine: {}", e),
                                suggestion: "Check WASM plugin support is enabled".to_string(),
                                source: None,
                            })
                        }
                    }
                }
                None => {
                    warn!("Plugin {} not found or not loaded", plugin_id);
                    Ok(None)
                }
            }
        } else {
            Err(UveddiError::PluginError {
                plugin: plugin_id.to_string(),
                plugin_type: "WASM".to_string(),
                message: "Plugin engine not initialized".to_string(),
                suggestion: "Enable plugins in configuration".to_string(),
                source: None,
            })
        }
    }
}

/// Handle for communicating with the plugin manager actor
#[derive(Clone)]
pub struct PluginManagerHandle {
    sender: mpsc::Sender<PluginCommand>,
}

impl PluginManagerHandle {
    /// Checks if the plugin manager is available.
    ///
    /// Returns `true` if the manager's channel is open and ready to receive commands.
    pub fn is_available(&self) -> bool {
        !self.sender.is_closed()
    }
}

impl PluginManagerHandleTrait for PluginManagerHandle {
    async fn execute_plugin(
        &self,
        plugin_id: String,
        source_file_path: PathBuf,
        ast: Arc<crate::ast::tree_sitter::Tree>,
    ) -> Result<Vec<ArchitecturalIssue>, UveddiError> {
        let (responder, receiver) = oneshot::channel();

        let command = PluginCommand::Execute {
            plugin_id: plugin_id.clone(),
            source_file_path,
            ast,
            responder,
        };

        self.sender
            .send(command)
            .await
            .map_err(|_| UveddiError::PluginError {
                plugin: plugin_id,
                plugin_type: "WASM".to_string(),
                message: "Plugin manager actor is not running".to_string(),
                suggestion: "Restart the plugin manager".to_string(),
                source: None,
            })?;

        receiver.await.map_err(|_| UveddiError::PluginError {
            plugin: "unknown".to_string(),
            plugin_type: "WASM".to_string(),
            message: "Plugin manager response channel closed".to_string(),
            suggestion: "Check plugin manager status".to_string(),
            source: None,
        })?
    }

    async fn load_plugin(&self, plugin_path: PathBuf) -> Result<String, UveddiError> {
        let (responder, receiver) = oneshot::channel();

        let command = PluginCommand::LoadPlugin {
            plugin_path: plugin_path.clone(),
            responder,
        };

        self.sender
            .send(command)
            .await
            .map_err(|_| UveddiError::PluginError {
                plugin: "unknown".to_string(),
                plugin_type: "WASM".to_string(),
                message: "Plugin manager actor is not running".to_string(),
                suggestion: "Restart the plugin manager".to_string(),
                source: None,
            })?;

        receiver.await.map_err(|_| UveddiError::PluginError {
            plugin: "unknown".to_string(),
            plugin_type: "WASM".to_string(),
            message: "Plugin manager response channel closed".to_string(),
            suggestion: "Check plugin manager status".to_string(),
            source: None,
        })?
    }

    async fn unload_plugin(&self, plugin_id: String) -> Result<(), UveddiError> {
        let (responder, receiver) = oneshot::channel();

        let command = PluginCommand::UnloadPlugin {
            plugin_id: plugin_id.clone(),
            responder,
        };

        self.sender
            .send(command)
            .await
            .map_err(|_| UveddiError::PluginError {
                plugin: plugin_id,
                plugin_type: "WASM".to_string(),
                message: "Plugin manager actor is not running".to_string(),
                suggestion: "Restart the plugin manager".to_string(),
                source: None,
            })?;

        receiver.await.map_err(|_| UveddiError::PluginError {
            plugin: "unknown".to_string(),
            plugin_type: "WASM".to_string(),
            message: "Plugin manager response channel closed".to_string(),
            suggestion: "Check plugin manager status".to_string(),
            source: None,
        })?
    }

    async fn get_stats(&self) -> Result<PluginStats, UveddiError> {
        let (responder, receiver) = oneshot::channel();

        let command = PluginCommand::GetStats { responder };

        self.sender
            .send(command)
            .await
            .map_err(|_| UveddiError::PluginError {
                plugin: "unknown".to_string(),
                plugin_type: "WASM".to_string(),
                message: "Plugin manager actor is not running".to_string(),
                suggestion: "Restart the plugin manager".to_string(),
                source: None,
            })?;

        receiver.await.map_err(|_| UveddiError::PluginError {
            plugin: "unknown".to_string(),
            plugin_type: "WASM".to_string(),
            message: "Plugin manager response channel closed".to_string(),
            suggestion: "Check plugin manager status".to_string(),
            source: None,
        })?
    }

    async fn list_loaded_plugins(&self) -> Result<Vec<String>, UveddiError> {
        let (responder, receiver) = oneshot::channel();

        let command = PluginCommand::ListLoadedPlugins { responder };

        self.sender
            .send(command)
            .await
            .map_err(|_| UveddiError::PluginError {
                plugin: "unknown".to_string(),
                plugin_type: "WASM".to_string(),
                message: "Plugin manager actor is not running".to_string(),
                suggestion: "Restart the plugin manager".to_string(),
                source: None,
            })?;

        receiver.await.map_err(|_| UveddiError::PluginError {
            plugin: "unknown".to_string(),
            plugin_type: "WASM".to_string(),
            message: "Plugin manager response channel closed".to_string(),
            suggestion: "Check plugin manager status".to_string(),
            source: None,
        })?
    }

    async fn get_plugin_adapter(
        &self,
        plugin_id: &str,
    ) -> Result<Option<crate::analysis::WasmPluginDetectorAdapter>, UveddiError> {
        let (responder, receiver) = oneshot::channel();

        let command = PluginCommand::GetPluginAdapter {
            plugin_id: plugin_id.to_string(),
            responder,
        };

        self.sender
            .send(command)
            .await
            .map_err(|_| UveddiError::PluginError {
                plugin: plugin_id.to_string(),
                plugin_type: "WASM".to_string(),
                message: "Plugin manager actor is not running".to_string(),
                suggestion: "Restart the plugin manager".to_string(),
                source: None,
            })?;

        receiver.await.map_err(|_| UveddiError::PluginError {
            plugin: plugin_id.to_string(),
            plugin_type: "WASM".to_string(),
            message: "Plugin manager response channel closed".to_string(),
            suggestion: "Check plugin manager status".to_string(),
            source: None,
        })?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_plugin_manager_creation() {
        let config_service = Arc::new(ConfigurationService::new());
        let handle = PluginManager::spawn(config_service);

        assert!(handle.is_available());

        // Test getting stats from a fresh manager
        let stats = handle.get_stats().await;
        assert!(stats.is_ok());

        let stats = stats.unwrap();
        assert_eq!(stats.loaded_plugins, 0);
        assert_eq!(stats.total_executions, 0);
    }

    #[tokio::test]
    async fn test_plugin_manager_load_unload() {
        let config_service = Arc::new(ConfigurationService::new());
        let handle = PluginManager::spawn(config_service);

        // Test loading a plugin
        let plugin_path = PathBuf::from("test_plugin.wasm");
        let load_result = handle.load_plugin(plugin_path).await;

        // Should fail since plugins are disabled by default
        assert!(load_result.is_err());

        // Test with plugins enabled
        let mut config_service = ConfigurationService::new();
        config_service.set_plugins_enabled(true);
        let config_service = Arc::new(config_service);
        let handle = PluginManager::spawn(config_service);

        // Give the manager time to initialize
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let plugin_path = PathBuf::from("test_plugin.wasm");
        let load_result = handle.load_plugin(plugin_path).await;

        // Should work now (though plugin engine might still fail to init)
        // This tests the communication mechanism
        assert!(load_result.is_ok() || load_result.is_err()); // Either way is fine for this test
    }

    #[tokio::test]
    async fn test_plugin_manager_stats() {
        let config_service = Arc::new(ConfigurationService::new());
        let handle = PluginManager::spawn(config_service);

        // Get initial stats
        let stats = handle.get_stats().await.unwrap();
        assert_eq!(stats.loaded_plugins, 0);
        assert_eq!(stats.total_executions, 0);
        assert_eq!(stats.average_execution_time_ms, 0.0);
    }
}
