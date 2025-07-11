//! Main WASM plugin engine that orchestrates the plugin system

use crate::{
    plugins::{errors::*, types::{PluginId, PluginStatus, PluginStats, ResourceLimits}, registry::*, security::*, data_plane::*, lifecycle::*},
};
use std::{collections::HashMap, path::Path, sync::Arc};
use tokio::sync::RwLock;

/// Main WASM plugin engine
#[derive(Debug)]
pub struct WasmPluginEngine {
    lifecycle_manager: PluginLifecycleManager,
    registry: PluginRegistry,
    data_plane: AstDataPlane,
    default_security_policy: SecurityPolicy,
    plugin_adapters: Arc<RwLock<HashMap<PluginId, WasmPluginAdapter>>>,
    enabled: bool,
}

impl WasmPluginEngine {
    /// Create a new WASM plugin engine
    /// Placeholder documentation for public items
    pub async fn new() -> Result<Self, PluginError> {
        #[cfg(not(feature = "wasm-plugins"))]
        {
            return Err(PluginError::Unsupported(
                "WASM plugins not enabled. Compile with --features wasm-plugins".to_string()
            ));
        }
        
        #[cfg(feature = "wasm-plugins")]
        {
            let plugins_dir = std::env::current_dir()?.join("plugins");
            
            Ok(Self {
                lifecycle_manager: PluginLifecycleManager::new(),
                registry: PluginRegistry::new(&plugins_dir).await
                    .map_err(|e| PluginError::Registry(e))?,
                data_plane: AstDataPlane::new()
                    .map_err(|e| PluginError::DataPlane(e))?,
                default_security_policy: SecurityPolicy::restrictive(),
                plugin_adapters: Arc::new(RwLock::new(HashMap::new())),
                enabled: true,
            })
        }
    }
    
    /// Create a new engine with custom configuration
    /// Placeholder documentation for public items
    pub async fn with_config(
        plugins_dir: &Path,
        security_policy: SecurityPolicy,
    ) -> Result<Self, PluginError> {
        #[cfg(not(feature = "wasm-plugins"))]
        {
            return Err(PluginError::Unsupported(
                "WASM plugins not enabled".to_string()
            ));
        }
        
        #[cfg(feature = "wasm-plugins")]
        {
            Ok(Self {
                lifecycle_manager: PluginLifecycleManager::new(),
                registry: PluginRegistry::new(plugins_dir).await
                    .map_err(|e| PluginError::Registry(e))?,
                data_plane: AstDataPlane::new()
                    .map_err(|e| PluginError::DataPlane(e))?,
                default_security_policy: security_policy,
                plugin_adapters: Arc::new(RwLock::new(HashMap::new())),
                enabled: true,
            })
        }
    }
    
    /// Load a plugin from the registry
    /// Placeholder documentation for public items
    pub async fn load_plugin(&mut self, plugin_id: &PluginId) -> Result<(), PluginError> {
        if !self.enabled {
            return Err(PluginError::Unsupported("Plugin engine is disabled".to_string()));
        }
        
        log::info!("Loading plugin: {}", plugin_id);
        
        // Get plugin metadata from registry
        let manifest = self.registry.get_manifest(plugin_id)
            .ok_or_else(|| PluginError::NotFound(plugin_id.to_string()))?
            .clone();
        
        // Load plugin binary
        let binary = self.registry.load_plugin_binary(plugin_id).await
            .map_err(|e| PluginError::Registry(e))?;
        
        // Use default security policy (could be customized per plugin)
        let security_policy = self.default_security_policy.clone();
        
        // Load the plugin
        self.lifecycle_manager.load_plugin(
            plugin_id.clone(),
            manifest.clone(),
            binary,
            security_policy,
        ).await?;
        
        // Create adapter for AnalysisDetector trait
        let adapter = WasmPluginAdapter::new(
            plugin_id.clone(),
            manifest,
            self.data_plane.clone(),
            self.lifecycle_manager.clone(),
        );
        
        self.plugin_adapters.write().await.insert(plugin_id.clone(), adapter);
        self.registry.record_usage(plugin_id);
        
        log::info!("Successfully loaded plugin: {}", plugin_id);
        Ok(())
    }
    
    /// Unload a plugin
    /// Placeholder documentation for public items
    pub async fn unload_plugin(&mut self, plugin_id: &PluginId) -> Result<(), PluginError> {
        log::info!("Unloading plugin: {}", plugin_id);
        
        // Remove adapter
        self.plugin_adapters.write().await.remove(plugin_id);
        
        // Unload from lifecycle manager
        self.lifecycle_manager.unload_plugin(plugin_id).await?;
        
        log::info!("Successfully unloaded plugin: {}", plugin_id);
        Ok(())
    }
    
    /// Load all plugins from the registry
    /// Placeholder documentation for public items
    pub async fn load_all_plugins(&mut self) -> Result<Vec<PluginId>, PluginError> {
        let mut loaded_plugins = Vec::new();
        let plugin_ids = self.registry.list_plugins();
        
        for plugin_id in plugin_ids {
            match self.load_plugin(&plugin_id).await {
                Ok(()) => {
                    loaded_plugins.push(plugin_id);
                }
                Err(e) => {
                    log::error!("Failed to load plugin {}: {}", plugin_id, e);
                    self.registry.update_status(&plugin_id, PluginStatus::Error(e.to_string()));
                }
            }
        }
        
        log::info!("Loaded {} plugins", loaded_plugins.len());
        Ok(loaded_plugins)
    }
    
    /// Install a new plugin
    /// Placeholder documentation for public items
    pub async fn install_plugin(
        &mut self,
        manifest: PluginManifest,
        binary: Vec<u8>,
    ) -> Result<PluginId, PluginError> {
        log::info!("Installing plugin: {}", manifest.name);
        
        // Register in the registry
        let plugin_id = self.registry.register_plugin(manifest, binary).await
            .map_err(|e| PluginError::Registry(e))?;
        
        // Automatically load the plugin
        self.load_plugin(&plugin_id).await?;
        
        log::info!("Successfully installed plugin: {}", plugin_id);
        Ok(plugin_id)
    }
    
    /// Uninstall a plugin
    /// Placeholder documentation for public items
    pub async fn uninstall_plugin(&mut self, plugin_id: &PluginId) -> Result<(), PluginError> {
        log::info!("Uninstalling plugin: {}", plugin_id);
        
        // Unload if loaded
        if self.plugin_adapters.read().await.contains_key(plugin_id) {
            self.unload_plugin(plugin_id).await?;
        }
        
        // Unregister from registry
        self.registry.unregister_plugin(plugin_id).await
            .map_err(|e| PluginError::Registry(e))?;
        
        log::info!("Successfully uninstalled plugin: {}", plugin_id);
        Ok(())
    }
    
    /// Get a plugin adapter for use with the analysis engine
    /// Placeholder documentation for public items
    pub async fn get_plugin_adapter(&self, plugin_id: &PluginId) -> Option<WasmPluginAdapter> {
        self.plugin_adapters.read().await.get(plugin_id).cloned()
    }
    
    /// Get all plugin adapters
    /// Placeholder documentation for public items
    pub async fn get_all_plugin_adapters(&self) -> Vec<WasmPluginAdapter> {
        self.plugin_adapters.read().await.values().cloned().collect()
    }
    
    /// List loaded plugins
    /// Placeholder documentation for public items
    pub async fn list_loaded_plugins(&self) -> Vec<PluginId> {
        self.plugin_adapters.read().await.keys().cloned().collect()
    }
    
    /// Get plugin statistics
    /// Placeholder documentation for public items
    pub async fn get_plugin_stats(&self, plugin_id: &PluginId) -> Option<PluginStats> {
        self.lifecycle_manager.get_plugin_stats(plugin_id).await
    }
    
    /// Get registry statistics
    /// Placeholder documentation for public items
    pub fn get_registry_stats(&self) -> crate::plugins::registry::RegistryStatistics {
        self.registry.get_statistics()
    }
    
    /// Monitor resource usage of all plugins
    /// Placeholder documentation for public items
    pub async fn monitor_resources(&mut self) -> Result<ResourceReport, PluginError> {
        self.lifecycle_manager.monitor_resources().await
    }
    
    /// Enable or disable the plugin engine
    /// Placeholder documentation for public items
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        log::info!("Plugin engine {}", if enabled { "enabled" } else { "disabled" });
    }
    
    /// Check if plugin engine is enabled
    /// Placeholder documentation for public items
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

/// Adapter that implements AnalysisDetector for WASM plugins
#[derive(Debug, Clone)]
pub struct WasmPluginAdapter {
    plugin_id: PluginId,
    manifest: PluginManifest,
    data_plane: AstDataPlane,
    lifecycle_manager: PluginLifecycleManager,
}

impl WasmPluginAdapter {
    pub fn new(
        plugin_id: PluginId,
        manifest: PluginManifest,
        data_plane: AstDataPlane,
        lifecycle_manager: PluginLifecycleManager,
    ) -> Self {
        Self {
            plugin_id,
            manifest,
            data_plane,
            lifecycle_manager,
        }
    }
    
    /// Get the plugin ID
    /// Placeholder documentation for public items
    pub fn plugin_id(&self) -> &PluginId {
        &self.plugin_id
    }
    
    /// Get the plugin manifest
    /// Placeholder documentation for public items
    pub fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }
}

// TODO: Re-enable when Sync issues are resolved
/*
impl AnalysisDetector for WasmPluginAdapter {
    fn detect_issues(&self, file: &ParsedFile) -> Result<Vec<ArchitecturalIssue>, crate::analysis::AnalysisError> {
        #[cfg(not(feature = "wasm-plugins"))]
        {
            return Err(crate::analysis::AnalysisError::PluginError(
                "WASM plugins not enabled".to_string()
            ));
        }
        
        #[cfg(feature = "wasm-plugins")]
        {
            // This would be implemented as an async function in a real implementation
            // For now, we'll return a placeholder
            let rt = tokio::runtime::Runtime::new()
                .map_err(|e| crate::analysis::AnalysisError::PluginError(e.to_string()))?;
            
            rt.block_on(async {
                self.detect_issues_async(file).await
            }).map_err(|e| crate::analysis::AnalysisError::PluginError(e.to_string()))
        }
    }
    
    fn get_anti_pattern_types(&self) -> Vec<AntiPatternType> {
        self.manifest.anti_pattern_types.iter()
            .filter_map(|pattern_str| {
                // Map string patterns to AntiPatternType enum
                match pattern_str.as_str() {
                    "god-object" => Some(AntiPatternType::GodObject),
                    "large-classes" => Some(AntiPatternType::LargeClasses),
                    "dead-code" => Some(AntiPatternType::DeadCode),
                    "code-duplication" => Some(AntiPatternType::CodeDuplication),
                    "tight-coupling" => Some(AntiPatternType::TightCoupling),
                    "cyclic-dependencies" => Some(AntiPatternType::CyclicDependencies),
                    "long-methods" => Some(AntiPatternType::LongMethods),
                    "magic-values" => Some(AntiPatternType::MagicValues),
                    _ => None, // Unknown pattern type
                }
            })
            .collect()
    }
    
    fn get_detector_name(&self) -> &'static str {
        // We need to return a static string, so we'll use a generic name
        // In a real implementation, this might be handled differently
        "wasm-plugin-detector"
    }
}
*/

impl WasmPluginAdapter {
    /// Async version of detect_issues
    #[cfg(feature = "wasm-plugins")]
    async fn detect_issues_async(&self, file: &ParsedFile) -> Result<Vec<ArchitecturalIssue>, PluginError> {
        // 1. Serialize AST to Arrow format
        let ast_buffer = self.data_plane.serialize_ast(file)?;
        
        // 2. Get plugin instance (simplified - in real implementation this would be more complex)
        // For now, we'll return empty results
        let issues = Vec::new();
        
        Ok(issues)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_plugin_engine_creation() {
        #[cfg(feature = "wasm-plugins")]
        {
            let temp_dir = TempDir::new().unwrap();
            let engine = WasmPluginEngine::with_config(
                temp_dir.path(),
                SecurityPolicy::permissive(),
            ).await;
            
            assert!(engine.is_ok());
            let engine = engine.unwrap();
            assert!(engine.is_enabled());
        }
        
        #[cfg(not(feature = "wasm-plugins"))]
        {
            let engine = WasmPluginEngine::new().await;
            assert!(engine.is_err());
        }
    }
    
    #[test]
    fn test_plugin_adapter() {
        let plugin_id = PluginId::new();
        let manifest = PluginManifest {
            name: "test-plugin".to_string(),
            version: "1.0.0".to_string(),
            author: "Test Author".to_string(),
            description: "Test plugin".to_string(),
            permissions: vec![],
            supported_languages: vec!["rust".to_string()],
            anti_pattern_types: vec!["god-object".to_string(), "unknown-pattern".to_string()],
            signature: None,
        };
        
        let data_plane = AstDataPlane::new().unwrap();
        let lifecycle_manager = PluginLifecycleManager::new();
        
        let adapter = WasmPluginAdapter::new(
            plugin_id.clone(),
            manifest,
            data_plane,
            lifecycle_manager,
        );
        
        assert_eq!(adapter.plugin_id(), &plugin_id);
        // TODO: Re-enable when WasmPluginAdapter implements AnalysisDetector trait
        // assert_eq!(adapter.get_detector_name(), "wasm-plugin-detector");
        
        // let anti_patterns = adapter.get_anti_pattern_types();
        // assert_eq!(anti_patterns.len(), 1); // Only "god-object" should be recognized
        // assert!(anti_patterns.contains(&AntiPatternType::GodObject));
    }
}