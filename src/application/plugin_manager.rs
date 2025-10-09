//! Application layer plugin manager
//!
//! This module provides the application-layer integration for WASM plugins,
//! coordinating between the CLI, analysis engine, and plugin system.
//! It handles plugin lifecycle, configuration, and orchestration with
//! Uveddi's core analysis workflow.

use crate::analysis::AnalysisEngine;
use crate::database::ScalableDatabase;
use crate::error::UveddiError;
use crate::plugins::{
    HostContext, HostContextFactory, PluginId, PluginManifest, SecurityPolicy, WasmPluginEngine,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// Application-level plugin manager that coordinates plugin operations
/// with Uveddi's core systems
pub struct ApplicationPluginManager {
    /// WASM plugin engine for low-level plugin operations
    plugin_engine: Option<WasmPluginEngine>,
    /// Host context factory for creating plugin execution contexts
    host_context_factory: HostContextFactory,
    /// Application-level configuration for plugins
    config: PluginManagerConfig,
    /// Plugin status tracking
    plugin_status: Arc<RwLock<HashMap<PluginId, PluginStatus>>>,
    /// Feature flag to enable/disable plugin system
    enabled: bool,
}

/// Configuration for the application plugin manager
#[derive(Debug, Clone)]
pub struct PluginManagerConfig {
    /// Directory where plugins are stored
    pub plugins_directory: PathBuf,
    /// Default security policy for plugins
    pub default_security_policy: SecurityPolicy,
    /// Maximum number of plugins that can be loaded
    pub max_plugins: usize,
    /// Enable plugin hot-reloading
    pub enable_hot_reload: bool,
    /// Plugin execution timeout in seconds
    pub execution_timeout_seconds: u64,
}

impl Default for PluginManagerConfig {
    fn default() -> Self {
        Self {
            plugins_directory: PathBuf::from("./plugins"),
            default_security_policy: SecurityPolicy::restrictive(),
            max_plugins: 50,
            enable_hot_reload: false,
            execution_timeout_seconds: 30,
        }
    }
}

/// Status of a plugin in the application
#[derive(Debug, Clone, PartialEq)]
pub enum PluginStatus {
    /// Plugin is not loaded
    NotLoaded,
    /// Plugin is loading
    Loading,
    /// Plugin is ready for use
    Ready,
    /// Plugin encountered an error
    Error(String),
    /// Plugin is disabled
    Disabled,
}

impl ApplicationPluginManager {
    /// Create a new application plugin manager
    pub async fn new(
        database: Arc<ScalableDatabase>,
        analysis_engine: Arc<RwLock<AnalysisEngine>>,
        config: Option<PluginManagerConfig>,
    ) -> Result<Self, UveddiError> {
        let config = config.unwrap_or_default();

        // Check if WASM plugins are enabled
        let (plugin_engine, enabled) = match WasmPluginEngine::with_config(
            &config.plugins_directory,
            config.default_security_policy.clone(),
        )
        .await
        {
            Ok(engine) => (Some(engine), true),
            Err(e) => {
                warn!("WASM plugin engine not available: {}", e);
                (None, false)
            }
        };

        let host_context_factory = HostContextFactory::new(database, analysis_engine);

        Ok(Self {
            plugin_engine,
            host_context_factory,
            config,
            plugin_status: Arc::new(RwLock::new(HashMap::new())),
            enabled,
        })
    }

    /// Check if plugin system is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Install a plugin from binary and manifest files
    pub async fn install_plugin(
        &mut self,
        binary_path: &Path,
        manifest_path: &Path,
    ) -> Result<PluginId, UveddiError> {
        if !self.enabled {
            return Err(UveddiError::PluginError {
                plugin: "system".to_string(),
                plugin_type: "WASM".to_string(),
                message: "Plugin system is not enabled".to_string(),
                suggestion: "Compile with --features wasm-plugins".to_string(),
                source: None,
            });
        }

        info!(
            "Installing plugin from {} and {}",
            binary_path.display(),
            manifest_path.display()
        );

        // Read and validate binary
        let binary = tokio::fs::read(binary_path).await.map_err(|e| {
            UveddiError::io_error(
                "read plugin binary",
                binary_path.to_string_lossy().as_ref(),
                e,
            )
        })?;

        // Read and parse manifest
        let manifest_content = tokio::fs::read_to_string(manifest_path)
            .await
            .map_err(|e| {
                UveddiError::io_error(
                    "read plugin manifest",
                    manifest_path.to_string_lossy().as_ref(),
                    e,
                )
            })?;

        let manifest: PluginManifest =
            toml::from_str(&manifest_content).map_err(|e| UveddiError::PluginError {
                plugin: "unknown".to_string(),
                plugin_type: "WASM".to_string(),
                message: format!("Invalid manifest format: {}", e),
                suggestion: "Check TOML syntax and required fields".to_string(),
                source: Some(crate::plugins::errors::PluginError::Configuration(
                    e.to_string(),
                )),
            })?;

        // Validate plugin constraints
        self.validate_plugin_constraints(&manifest)?;

        // Set plugin status to loading
        let plugin_id = PluginId::from_name(&manifest.name);
        self.set_plugin_status(plugin_id.clone(), PluginStatus::Loading)
            .await;

        // Get plugin engine
        let engine = self
            .plugin_engine
            .as_mut()
            .ok_or_else(|| UveddiError::PluginError {
                plugin: manifest.name.clone(),
                plugin_type: "WASM".to_string(),
                message: "Plugin engine not available".to_string(),
                suggestion: "Ensure WASM plugins are enabled".to_string(),
                source: None,
            })?;

        // Install plugin in engine
        match engine.install_plugin(manifest.clone(), binary).await {
            Ok(installed_id) => {
                info!(
                    "Successfully installed plugin '{}' with ID: {}",
                    manifest.name, installed_id
                );
                self.set_plugin_status(plugin_id.clone(), PluginStatus::Ready)
                    .await;
                Ok(installed_id)
            }
            Err(e) => {
                error!("Failed to install plugin '{}': {}", manifest.name, e);
                self.set_plugin_status(plugin_id.clone(), PluginStatus::Error(e.to_string()))
                    .await;
                Err(e.into())
            }
        }
    }

    /// Uninstall a plugin
    pub async fn uninstall_plugin(&mut self, plugin_id: &PluginId) -> Result<(), UveddiError> {
        if !self.enabled {
            return Err(UveddiError::PluginError {
                plugin: plugin_id.to_string(),
                plugin_type: "WASM".to_string(),
                message: "Plugin system is not enabled".to_string(),
                suggestion: "Compile with --features wasm-plugins".to_string(),
                source: None,
            });
        }

        info!("Uninstalling plugin: {}", plugin_id);

        // Get plugin engine
        let engine = self
            .plugin_engine
            .as_mut()
            .ok_or_else(|| UveddiError::PluginError {
                plugin: plugin_id.to_string(),
                plugin_type: "WASM".to_string(),
                message: "Plugin engine not available".to_string(),
                suggestion: "Ensure WASM plugins are enabled".to_string(),
                source: None,
            })?;

        // Uninstall plugin
        match engine.uninstall_plugin(plugin_id).await {
            Ok(_) => {
                info!("Successfully uninstalled plugin: {}", plugin_id);
                self.set_plugin_status(plugin_id.clone(), PluginStatus::NotLoaded)
                    .await;
                Ok(())
            }
            Err(e) => {
                error!("Failed to uninstall plugin '{}': {}", plugin_id, e);
                Err(e.into())
            }
        }
    }

    /// List all installed plugins and their status
    pub async fn list_plugins(&self) -> Result<Vec<(PluginId, PluginStatus)>, UveddiError> {
        if !self.enabled {
            return Ok(Vec::new());
        }

        let status_map = self.plugin_status.read().await;
        Ok(status_map
            .iter()
            .map(|(id, status)| (id.clone(), status.clone()))
            .collect())
    }

    /// Get plugin information
    pub async fn get_plugin_info(
        &self,
        plugin_id: &PluginId,
    ) -> Result<Option<PluginInfo>, UveddiError> {
        if !self.enabled {
            return Ok(None);
        }

        let engine = self
            .plugin_engine
            .as_ref()
            .ok_or_else(|| UveddiError::PluginError {
                plugin: plugin_id.to_string(),
                plugin_type: "WASM".to_string(),
                message: "Plugin engine not available".to_string(),
                suggestion: "Ensure WASM plugins are enabled".to_string(),
                source: None,
            })?;

        // Get plugin status
        let status = {
            let status_map = self.plugin_status.read().await;
            status_map
                .get(plugin_id)
                .cloned()
                .unwrap_or(PluginStatus::NotLoaded)
        };

        // Try to get plugin stats from engine
        let stats = engine.get_plugin_stats(plugin_id).await;

        Ok(Some(PluginInfo {
            id: plugin_id.clone(),
            status,
            stats,
            last_error: None, // Could be extracted from engine
        }))
    }

    /// Create a host context for a plugin
    pub fn create_host_context(&self, plugin_id: PluginId) -> HostContext {
        self.host_context_factory
            .create_context(plugin_id, self.config.default_security_policy.clone())
    }

    /// Enable/disable a plugin
    pub async fn set_plugin_enabled(
        &mut self,
        plugin_id: &PluginId,
        enabled: bool,
    ) -> Result<(), UveddiError> {
        let new_status = if enabled {
            PluginStatus::Ready
        } else {
            PluginStatus::Disabled
        };
        self.set_plugin_status(plugin_id.clone(), new_status).await;
        info!(
            "Plugin {} {}",
            plugin_id,
            if enabled { "enabled" } else { "disabled" }
        );
        Ok(())
    }

    /// Get plugin system statistics
    pub async fn get_system_stats(&self) -> PluginSystemStats {
        if !self.enabled {
            return PluginSystemStats::default();
        }

        let status_map = self.plugin_status.read().await;
        let mut stats = PluginSystemStats::default();

        for status in status_map.values() {
            match status {
                PluginStatus::NotLoaded => stats.not_loaded_count += 1,
                PluginStatus::Loading => stats.loading_count += 1,
                PluginStatus::Ready => stats.ready_count += 1,
                PluginStatus::Error(_) => stats.error_count += 1,
                PluginStatus::Disabled => stats.disabled_count += 1,
            }
        }

        stats.total_plugins = status_map.len();
        stats.enabled = self.enabled;
        stats
    }

    /// Validate plugin constraints before installation
    fn validate_plugin_constraints(&self, manifest: &PluginManifest) -> Result<(), UveddiError> {
        // Check plugin count limit
        let current_count = self.plugin_status.blocking_read().len();
        if current_count >= self.config.max_plugins {
            return Err(UveddiError::PluginError {
                plugin: manifest.name.clone(),
                plugin_type: "WASM".to_string(),
                message: format!("Plugin limit exceeded ({} max)", self.config.max_plugins),
                suggestion: "Uninstall some plugins or increase the limit".to_string(),
                source: None,
            });
        }

        // Validate plugin requirements
        // TODO: Add min_uveddi_version field to PluginManifest
        // if manifest.min_uveddi_version.is_some() {
        //     debug!("Plugin requires Uveddi version: {:?}", manifest.min_uveddi_version);
        // }

        Ok(())
    }

    /// Set plugin status
    async fn set_plugin_status(&self, plugin_id: PluginId, status: PluginStatus) {
        let mut status_map = self.plugin_status.write().await;
        status_map.insert(plugin_id, status);
    }
}

/// Information about a plugin
#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub id: PluginId,
    pub status: PluginStatus,
    pub stats: Option<crate::plugins::types::PluginStats>,
    pub last_error: Option<String>,
}

/// System-wide plugin statistics
#[derive(Debug, Clone, Default)]
pub struct PluginSystemStats {
    pub enabled: bool,
    pub total_plugins: usize,
    pub ready_count: usize,
    pub loading_count: usize,
    pub not_loaded_count: usize,
    pub error_count: usize,
    pub disabled_count: usize,
}
