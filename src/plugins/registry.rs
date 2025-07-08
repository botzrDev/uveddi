//! Plugin registry for discovery and metadata management

use crate::plugins::{errors::*, types::*, security::Permission};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use tokio::fs as async_fs;

/// Plugin registry for managing plugin metadata and discovery
#[derive(Debug)]
pub struct PluginRegistry {
    plugins: HashMap<PluginId, PluginMetadata>,
    registry_path: PathBuf,
    cache: RegistryCache,
}

impl PluginRegistry {
    /// Create a new plugin registry
    pub async fn new<P: AsRef<Path>>(registry_path: P) -> Result<Self, RegistryError> {
        let registry_path = registry_path.as_ref().to_path_buf();
        
        // Ensure registry directory exists
        if !registry_path.exists() {
            async_fs::create_dir_all(&registry_path).await?;
        }
        
        let mut registry = Self {
            plugins: HashMap::new(),
            registry_path,
            cache: RegistryCache::new(),
        };
        
        // Load existing plugins
        registry.discover_plugins().await?;
        
        Ok(registry)
    }
    
    /// Discover plugins in the registry directory
    pub async fn discover_plugins(&mut self) -> Result<(), RegistryError> {
        let mut entries = async_fs::read_dir(&self.registry_path).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            
            if path.is_dir() {
                if let Err(e) = self.load_plugin_from_directory(&path).await {
                    log::warn!("Failed to load plugin from {:?}: {}", path, e);
                }
            }
        }
        
        log::info!("Discovered {} plugins", self.plugins.len());
        Ok(())
    }
    
    /// Load a plugin from a directory
    async fn load_plugin_from_directory(&mut self, plugin_dir: &Path) -> Result<(), RegistryError> {
        let manifest_path = plugin_dir.join("plugin.toml");
        let binary_path = plugin_dir.join("plugin.wasm");
        
        if !manifest_path.exists() {
            return Err(RegistryError::Validation(
                "Plugin manifest (plugin.toml) not found".to_string()
            ));
        }
        
        if !binary_path.exists() {
            return Err(RegistryError::Validation(
                "Plugin binary (plugin.wasm) not found".to_string()
            ));
        }
        
        // Load and parse manifest
        let manifest_content = async_fs::read_to_string(&manifest_path).await?;
        let manifest: PluginManifest = toml::from_str(&manifest_content)
            .map_err(|e| RegistryError::ManifestParse(e.to_string()))?;
        
        // Create plugin metadata
        let plugin_id = PluginId::from_name(&manifest.name);
        let metadata = PluginMetadata {
            id: plugin_id.clone(),
            manifest,
            binary_path,
            manifest_path,
            installation_time: std::time::SystemTime::now(),
            last_used: None,
            usage_count: 0,
            status: PluginStatus::Ready,
        };
        
        self.plugins.insert(plugin_id, metadata);
        Ok(())
    }
    
    /// Register a new plugin
    pub async fn register_plugin(
        &mut self,
        manifest: PluginManifest,
        binary: Vec<u8>,
    ) -> Result<PluginId, RegistryError> {
        let plugin_id = PluginId::from_name(&manifest.name);
        
        // Check if plugin already exists
        if self.plugins.contains_key(&plugin_id) {
            return Err(RegistryError::Validation(
                format!("Plugin '{}' already exists", manifest.name)
            ));
        }
        
        // Create plugin directory
        let plugin_dir = self.registry_path.join(&manifest.name);
        async_fs::create_dir_all(&plugin_dir).await?;
        
        // Write manifest file
        let manifest_path = plugin_dir.join("plugin.toml");
        let manifest_content = toml::to_string_pretty(&manifest)
            .map_err(|e| RegistryError::ManifestParse(e.to_string()))?;
        async_fs::write(&manifest_path, manifest_content).await?;
        
        // Write binary file
        let binary_path = plugin_dir.join("plugin.wasm");
        async_fs::write(&binary_path, binary).await?;
        
        // Create metadata
        let metadata = PluginMetadata {
            id: plugin_id.clone(),
            manifest,
            binary_path,
            manifest_path,
            installation_time: std::time::SystemTime::now(),
            last_used: None,
            usage_count: 0,
            status: PluginStatus::Ready,
        };
        
        self.plugins.insert(plugin_id.clone(), metadata);
        self.cache.invalidate(&plugin_id);
        
        log::info!("Registered plugin: {}", plugin_id);
        Ok(plugin_id)
    }
    
    /// Unregister a plugin
    pub async fn unregister_plugin(&mut self, plugin_id: &PluginId) -> Result<(), RegistryError> {
        let metadata = self.plugins.remove(plugin_id)
            .ok_or_else(|| RegistryError::Validation(
                format!("Plugin {} not found", plugin_id)
            ))?;
        
        // Remove plugin directory
        let plugin_dir = metadata.binary_path.parent().unwrap();
        async_fs::remove_dir_all(plugin_dir).await?;
        
        self.cache.invalidate(plugin_id);
        
        log::info!("Unregistered plugin: {}", plugin_id);
        Ok(())
    }
    
    /// Get plugin metadata
    pub fn get_plugin(&self, plugin_id: &PluginId) -> Option<&PluginMetadata> {
        self.plugins.get(plugin_id)
    }
    
    /// Get plugin manifest
    pub fn get_manifest(&self, plugin_id: &PluginId) -> Option<&PluginManifest> {
        self.plugins.get(plugin_id).map(|metadata| &metadata.manifest)
    }
    
    /// Load plugin binary
    pub async fn load_plugin_binary(&self, plugin_id: &PluginId) -> Result<Vec<u8>, RegistryError> {
        let metadata = self.plugins.get(plugin_id)
            .ok_or_else(|| RegistryError::Validation(
                format!("Plugin {} not found", plugin_id)
            ))?;
        
        let binary = async_fs::read(&metadata.binary_path).await?;
        Ok(binary)
    }
    
    /// List all plugin IDs
    pub fn list_plugins(&self) -> Vec<PluginId> {
        self.plugins.keys().cloned().collect()
    }
    
    /// List plugins by supported language
    pub fn list_plugins_for_language(&self, language: &str) -> Vec<PluginId> {
        self.plugins
            .iter()
            .filter(|(_, metadata)| {
                metadata.manifest.supported_languages.contains(&language.to_string())
            })
            .map(|(id, _)| id.clone())
            .collect()
    }
    
    /// List plugins by anti-pattern type
    pub fn list_plugins_for_anti_pattern(&self, anti_pattern: &str) -> Vec<PluginId> {
        self.plugins
            .iter()
            .filter(|(_, metadata)| {
                metadata.manifest.anti_pattern_types.contains(&anti_pattern.to_string())
            })
            .map(|(id, _)| id.clone())
            .collect()
    }
    
    /// Update plugin usage statistics
    pub fn record_usage(&mut self, plugin_id: &PluginId) {
        if let Some(metadata) = self.plugins.get_mut(plugin_id) {
            metadata.usage_count += 1;
            metadata.last_used = Some(std::time::SystemTime::now());
        }
    }
    
    /// Update plugin status
    pub fn update_status(&mut self, plugin_id: &PluginId, status: PluginStatus) {
        if let Some(metadata) = self.plugins.get_mut(plugin_id) {
            metadata.status = status;
        }
    }
    
    /// Get registry statistics
    pub fn get_statistics(&self) -> RegistryStatistics {
        let total_plugins = self.plugins.len();
        let ready_plugins = self.plugins.values()
            .filter(|m| m.status == PluginStatus::Ready)
            .count();
        let error_plugins = self.plugins.values()
            .filter(|m| matches!(m.status, PluginStatus::Error(_)))
            .count();
        
        let languages: std::collections::HashSet<String> = self.plugins.values()
            .flat_map(|m| &m.manifest.supported_languages)
            .cloned()
            .collect();
        
        let anti_patterns: std::collections::HashSet<String> = self.plugins.values()
            .flat_map(|m| &m.manifest.anti_pattern_types)
            .cloned()
            .collect();
        
        RegistryStatistics {
            total_plugins,
            ready_plugins,
            error_plugins,
            supported_languages: languages.into_iter().collect(),
            supported_anti_patterns: anti_patterns.into_iter().collect(),
        }
    }
}

/// Plugin metadata stored in the registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub id: PluginId,
    pub manifest: PluginManifest,
    pub binary_path: PathBuf,
    pub manifest_path: PathBuf,
    pub installation_time: std::time::SystemTime,
    pub last_used: Option<std::time::SystemTime>,
    pub usage_count: u64,
    pub status: PluginStatus,
}

/// Plugin manifest file structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub permissions: Vec<Permission>,
    pub supported_languages: Vec<String>,
    pub anti_pattern_types: Vec<String>,
    pub signature: Option<String>,
}

impl PluginManifest {
    /// Create a new plugin manifest
    pub fn new(name: String, version: String, author: String) -> Self {
        Self {
            name,
            version,
            author,
            description: String::new(),
            permissions: Vec::new(),
            supported_languages: Vec::new(),
            anti_pattern_types: Vec::new(),
            signature: None,
        }
    }
    
    /// Add a permission to the manifest
    pub fn add_permission(&mut self, permission: Permission) {
        self.permissions.push(permission);
    }
    
    /// Add a supported language
    pub fn add_language(&mut self, language: String) {
        self.supported_languages.push(language);
    }
    
    /// Add an anti-pattern type
    pub fn add_anti_pattern_type(&mut self, anti_pattern: String) {
        self.anti_pattern_types.push(anti_pattern);
    }
}

/// Registry cache for performance optimization
#[derive(Debug)]
struct RegistryCache {
    binary_cache: HashMap<PluginId, Vec<u8>>,
}

impl RegistryCache {
    fn new() -> Self {
        Self {
            binary_cache: HashMap::new(),
        }
    }
    
    fn invalidate(&mut self, plugin_id: &PluginId) {
        self.binary_cache.remove(plugin_id);
    }
}

/// Registry statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryStatistics {
    pub total_plugins: usize,
    pub ready_plugins: usize,
    pub error_plugins: usize,
    pub supported_languages: Vec<String>,
    pub supported_anti_patterns: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_plugin_registry() {
        let temp_dir = TempDir::new().unwrap();
        let mut registry = PluginRegistry::new(temp_dir.path()).await.unwrap();
        
        let manifest = PluginManifest {
            name: "test-plugin".to_string(),
            version: "1.0.0".to_string(),
            author: "Test Author".to_string(),
            description: "Test plugin".to_string(),
            permissions: vec![Permission::Logging],
            supported_languages: vec!["rust".to_string()],
            anti_pattern_types: vec!["test-pattern".to_string()],
            signature: None,
        };
        
        let binary = b"\0asm\x01\0\0\0"; // Minimal WASM binary
        let plugin_id = registry.register_plugin(manifest.clone(), binary.to_vec()).await.unwrap();
        
        assert_eq!(registry.list_plugins().len(), 1);
        assert!(registry.get_plugin(&plugin_id).is_some());
        
        let loaded_binary = registry.load_plugin_binary(&plugin_id).await.unwrap();
        assert_eq!(loaded_binary, binary);
        
        registry.unregister_plugin(&plugin_id).await.unwrap();
        assert_eq!(registry.list_plugins().len(), 0);
    }
    
    #[test]
    fn test_plugin_manifest() {
        let mut manifest = PluginManifest::new(
            "test-plugin".to_string(),
            "1.0.0".to_string(),
            "Test Author".to_string(),
        );
        
        manifest.add_permission(Permission::Logging);
        manifest.add_language("rust".to_string());
        manifest.add_anti_pattern_type("god-object".to_string());
        
        assert_eq!(manifest.permissions.len(), 1);
        assert_eq!(manifest.supported_languages.len(), 1);
        assert_eq!(manifest.anti_pattern_types.len(), 1);
    }
    
    #[tokio::test]
    async fn test_plugin_discovery() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create a plugin directory structure
        let plugin_dir = temp_dir.path().join("test-plugin");
        fs::create_dir_all(&plugin_dir).unwrap();
        
        let manifest = PluginManifest::new(
            "test-plugin".to_string(),
            "1.0.0".to_string(),
            "Test Author".to_string(),
        );
        
        let manifest_content = toml::to_string_pretty(&manifest).unwrap();
        fs::write(plugin_dir.join("plugin.toml"), manifest_content).unwrap();
        fs::write(plugin_dir.join("plugin.wasm"), b"\0asm\x01\0\0\0").unwrap();
        
        let registry = PluginRegistry::new(temp_dir.path()).await.unwrap();
        assert_eq!(registry.list_plugins().len(), 1);
    }
}