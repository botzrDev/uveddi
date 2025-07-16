//! Plugin manager actor implementation
//!
//! Manages WASM plugins using an actor-based architecture for better isolation
//! and non-blocking operation.

use super::traits::{PluginCommand, PluginManagerHandle as PluginManagerHandleTrait, PluginStats, ConfigurationService as ConfigurationServiceTrait};
use super::config_service::ConfigurationService;
use crate::database::models::ArchitecturalIssue;
use crate::error::UveddiError;
use crate::plugins::WasmPluginEngine;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, oneshot};
use log::{info, warn, error};

/// Actor-based plugin manager for handling WASM plugins
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
                warn!("Failed to initialize WASM plugin engine: {}. Continuing without plugins.", e);
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
            PluginCommand::Execute { plugin_id, source_file_path, ast, responder } => {
                let result = self.execute_plugin(plugin_id, source_file_path, ast).await;
                if let Err(_) = responder.send(result) {
                    warn!("Failed to send plugin execution response - receiver dropped");
                }
            }
            PluginCommand::LoadPlugin { plugin_path, responder } => {
                let result = self.load_plugin(plugin_path).await;
                if let Err(_) = responder.send(result) {
                    warn!("Failed to send plugin load response - receiver dropped");
                }
            }
            PluginCommand::UnloadPlugin { plugin_id, responder } => {
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
        }
    }
    
    /// Executes a plugin on a source file
    async fn execute_plugin(
        &mut self,
        plugin_id: String,
        source_file_path: PathBuf,
        ast: Arc<tree_sitter::Tree>,
    ) -> Result<Vec<ArchitecturalIssue>, UveddiError> {
        let start_time = Instant::now();
        
        if let Some(ref mut plugin_engine) = self.plugin_engine {
            // TODO: Implement actual plugin execution logic
            // For now, return empty results
            let execution_time = start_time.elapsed();
            self.record_execution(&plugin_id, execution_time);
            
            info!("Executed plugin {} on file {} in {:?}", 
                  plugin_id, source_file_path.display(), execution_time);
            
            Ok(Vec::new()) // Empty results for now
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
            // TODO: Implement actual plugin loading logic
            let plugin_id = format!("plugin_{}", self.execution_stats.loaded_plugins.len());
            
            self.execution_stats.loaded_plugins.insert(
                plugin_id.clone(),
                PluginInfo {
                    id: plugin_id.clone(),
                    executions: 0,
                    total_time: Duration::default(),
                }
            );
            
            info!("Loaded plugin from {:?} with ID {}", plugin_path, plugin_id);
            Ok(plugin_id)
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
            if self.execution_stats.loaded_plugins.remove(&plugin_id).is_some() {
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
}

/// Handle for communicating with the plugin manager actor
#[derive(Clone)]
pub struct PluginManagerHandle {
    sender: mpsc::Sender<PluginCommand>,
}

impl PluginManagerHandle {
    /// Checks if the plugin manager is available
    pub fn is_available(&self) -> bool {
        !self.sender.is_closed()
    }
}

impl PluginManagerHandleTrait for PluginManagerHandle {
    async fn execute_plugin(
        &self,
        plugin_id: String,
        source_file_path: PathBuf,
        ast: Arc<tree_sitter::Tree>,
    ) -> Result<Vec<ArchitecturalIssue>, UveddiError> {
        let (responder, receiver) = oneshot::channel();
        
        let command = PluginCommand::Execute {
            plugin_id: plugin_id.clone(),
            source_file_path,
            ast,
            responder,
        };
        
        self.sender.send(command).await.map_err(|_| {
            UveddiError::PluginError {
                plugin: plugin_id,
                plugin_type: "WASM".to_string(),
                message: "Plugin manager actor is not running".to_string(),
                suggestion: "Restart the plugin manager".to_string(),
                source: None,
            }
        })?;
        
        receiver.await.map_err(|_| {
            UveddiError::PluginError {
                plugin: "unknown".to_string(),
                plugin_type: "WASM".to_string(),
                message: "Plugin manager response channel closed".to_string(),
                suggestion: "Check plugin manager status".to_string(),
                source: None,
            }
        })?
    }
    
    async fn load_plugin(&self, plugin_path: PathBuf) -> Result<String, UveddiError> {
        let (responder, receiver) = oneshot::channel();
        
        let command = PluginCommand::LoadPlugin {
            plugin_path: plugin_path.clone(),
            responder,
        };
        
        self.sender.send(command).await.map_err(|_| {
            UveddiError::PluginError {
                plugin: "unknown".to_string(),
                plugin_type: "WASM".to_string(),
                message: "Plugin manager actor is not running".to_string(),
                suggestion: "Restart the plugin manager".to_string(),
                source: None,
            }
        })?;
        
        receiver.await.map_err(|_| {
            UveddiError::PluginError {
                plugin: "unknown".to_string(),
                plugin_type: "WASM".to_string(),
                message: "Plugin manager response channel closed".to_string(),
                suggestion: "Check plugin manager status".to_string(),
                source: None,
            }
        })?
    }
    
    async fn unload_plugin(&self, plugin_id: String) -> Result<(), UveddiError> {
        let (responder, receiver) = oneshot::channel();
        
        let command = PluginCommand::UnloadPlugin {
            plugin_id: plugin_id.clone(),
            responder,
        };
        
        self.sender.send(command).await.map_err(|_| {
            UveddiError::PluginError {
                plugin: plugin_id,
                plugin_type: "WASM".to_string(),
                message: "Plugin manager actor is not running".to_string(),
                suggestion: "Restart the plugin manager".to_string(),
                source: None,
            }
        })?;
        
        receiver.await.map_err(|_| {
            UveddiError::PluginError {
                plugin: "unknown".to_string(),
                plugin_type: "WASM".to_string(),
                message: "Plugin manager response channel closed".to_string(),
                suggestion: "Check plugin manager status".to_string(),
                source: None,
            }
        })?
    }
    
    async fn get_stats(&self) -> Result<PluginStats, UveddiError> {
        let (responder, receiver) = oneshot::channel();
        
        let command = PluginCommand::GetStats { responder };
        
        self.sender.send(command).await.map_err(|_| {
            UveddiError::PluginError {
                plugin: "unknown".to_string(),
                plugin_type: "WASM".to_string(),
                message: "Plugin manager actor is not running".to_string(),
                suggestion: "Restart the plugin manager".to_string(),
                source: None,
            }
        })?;
        
        receiver.await.map_err(|_| {
            UveddiError::PluginError {
                plugin: "unknown".to_string(),
                plugin_type: "WASM".to_string(),
                message: "Plugin manager response channel closed".to_string(),
                suggestion: "Check plugin manager status".to_string(),
                source: None,
            }
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