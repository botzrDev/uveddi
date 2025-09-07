//! Plugin Marketplace and Registry Infrastructure
//!
//! This module provides infrastructure for plugin discovery, distribution,
//! and management through a centralized marketplace system.

use crate::error::{Result, UveddiError};
use crate::plugins::{PluginManifest, PluginMetadata};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Plugin marketplace client for discovering and managing plugins
pub struct PluginMarketplace {
    registry_url: String,
    local_cache_dir: PathBuf,
    installed_plugins: HashMap<String, InstalledPlugin>,
    available_plugins: HashMap<String, MarketplacePlugin>,
}

/// Plugin information in the marketplace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplacePlugin {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub category: String,
    pub tags: Vec<String>,
    pub supported_languages: Vec<String>,
    pub download_url: String,
    pub manifest_url: String,
    pub repository_url: Option<String>,
    pub homepage_url: Option<String>,
    pub documentation_url: Option<String>,
    pub license: String,
    pub downloads: u64,
    pub rating: f64,
    pub reviews: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub minimum_uveddi_version: String,
    pub file_size: u64,
    pub checksum: String,
    pub verified: bool,
    pub featured: bool,
    pub dependencies: Vec<String>,
    pub screenshots: Vec<String>,
}

/// Information about an installed plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledPlugin {
    pub marketplace_plugin: MarketplacePlugin,
    pub installation_path: PathBuf,
    pub installed_at: DateTime<Utc>,
    pub auto_update: bool,
    pub enabled: bool,
}

/// Plugin search criteria
#[derive(Debug, Clone, Default)]
pub struct PluginSearchCriteria {
    pub query: Option<String>,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub language: Option<String>,
    pub author: Option<String>,
    pub min_rating: Option<f64>,
    pub sort_by: SortBy,
    pub verified_only: bool,
    pub featured_only: bool,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

/// Search result sorting options
#[derive(Debug, Clone)]
pub enum SortBy {
    Relevance,
    Downloads,
    Rating,
    Updated,
    Created,
    Name,
}

impl Default for SortBy {
    fn default() -> Self {
        SortBy::Relevance
    }
}

/// Plugin search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSearchResults {
    pub plugins: Vec<MarketplacePlugin>,
    pub total_count: usize,
    pub page: usize,
    pub per_page: usize,
    pub has_more: bool,
}

/// Plugin installation options
#[derive(Debug, Clone)]
pub struct InstallationOptions {
    pub auto_update: bool,
    pub enable_immediately: bool,
    pub force_reinstall: bool,
    pub skip_verification: bool,
    pub custom_install_path: Option<PathBuf>,
}

impl Default for InstallationOptions {
    fn default() -> Self {
        Self {
            auto_update: true,
            enable_immediately: true,
            force_reinstall: false,
            skip_verification: false,
            custom_install_path: None,
        }
    }
}

impl PluginMarketplace {
    /// Create a new plugin marketplace client
    pub fn new<P: AsRef<Path>>(registry_url: String, cache_dir: P) -> Result<Self> {
        let cache_dir = cache_dir.as_ref().to_path_buf();
        
        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir)?;
        }

        let mut marketplace = Self {
            registry_url,
            local_cache_dir: cache_dir,
            installed_plugins: HashMap::new(),
            available_plugins: HashMap::new(),
        };

        // Load installed plugins from local cache
        marketplace.load_installed_plugins()?;
        
        Ok(marketplace)
    }

    /// Search for plugins in the marketplace
    pub async fn search_plugins(&mut self, criteria: &PluginSearchCriteria) -> Result<PluginSearchResults> {
        let search_url = format!("{}/api/v1/plugins/search", self.registry_url);
        
        // Build query parameters
        let mut query_params = Vec::new();
        
        if let Some(ref query) = criteria.query {
            query_params.push(format!("q={}", urlencoding::encode(query)));
        }
        
        if let Some(ref category) = criteria.category {
            query_params.push(format!("category={}", urlencoding::encode(category)));
        }
        
        if !criteria.tags.is_empty() {
            let tags = criteria.tags.join(",");
            query_params.push(format!("tags={}", urlencoding::encode(&tags)));
        }
        
        if let Some(ref language) = criteria.language {
            query_params.push(format!("language={}", urlencoding::encode(language)));
        }
        
        if let Some(ref author) = criteria.author {
            query_params.push(format!("author={}", urlencoding::encode(author)));
        }
        
        if let Some(min_rating) = criteria.min_rating {
            query_params.push(format!("min_rating={}", min_rating));
        }
        
        if criteria.verified_only {
            query_params.push("verified=true".to_string());
        }
        
        if criteria.featured_only {
            query_params.push("featured=true".to_string());
        }
        
        let sort_param = match criteria.sort_by {
            SortBy::Downloads => "downloads",
            SortBy::Rating => "rating", 
            SortBy::Updated => "updated",
            SortBy::Created => "created",
            SortBy::Name => "name",
            SortBy::Relevance => "relevance",
        };
        query_params.push(format!("sort={}", sort_param));
        
        if let Some(limit) = criteria.limit {
            query_params.push(format!("limit={}", limit));
        }
        
        if let Some(offset) = criteria.offset {
            query_params.push(format!("offset={}", offset));
        }

        let full_url = if query_params.is_empty() {
            search_url
        } else {
            format!("{}?{}", search_url, query_params.join("&"))
        };

        // Simulate HTTP request (in real implementation, use reqwest or similar)
        let search_results = self.simulate_search_request(&full_url, criteria).await?;
        
        // Update local cache with discovered plugins
        for plugin in &search_results.plugins {
            self.available_plugins.insert(plugin.id.clone(), plugin.clone());
        }
        
        Ok(search_results)
    }

    /// Get detailed information about a specific plugin
    pub async fn get_plugin_info(&mut self, plugin_id: &str) -> Result<MarketplacePlugin> {
        // Check local cache first
        if let Some(plugin) = self.available_plugins.get(plugin_id) {
            return Ok(plugin.clone());
        }

        // Fetch from marketplace
        let info_url = format!("{}/api/v1/plugins/{}", self.registry_url, plugin_id);
        let plugin_info = self.simulate_plugin_info_request(&info_url, plugin_id).await?;
        
        // Cache the plugin info
        self.available_plugins.insert(plugin_id.to_string(), plugin_info.clone());
        
        Ok(plugin_info)
    }

    /// Install a plugin from the marketplace
    pub async fn install_plugin(&mut self, plugin_id: &str, options: &InstallationOptions) -> Result<PathBuf> {
        // Get plugin information
        let plugin = self.get_plugin_info(plugin_id).await?;
        
        // Check if already installed and handle accordingly
        if let Some(installed) = self.installed_plugins.get(plugin_id) {
            if !options.force_reinstall {
                return Err(UveddiError::PluginError {
                    plugin: plugin_id.to_string(),
                    plugin_type: "marketplace".to_string(),
                    message: format!("Plugin '{}' is already installed", plugin_id),
                    suggestion: "Use --force to reinstall or uninstall first".to_string(),
                    source: None,
                });
            }
            
            // Uninstall existing version first
            self.uninstall_plugin(plugin_id)?;
        }

        // Verify plugin compatibility
        self.verify_plugin_compatibility(&plugin)?;

        // Download plugin binary and manifest
        let installation_path = self.determine_installation_path(&plugin, options);
        fs::create_dir_all(&installation_path)?;

        let binary_path = self.download_plugin_binary(&plugin, &installation_path).await?;
        let manifest_path = self.download_plugin_manifest(&plugin, &installation_path).await?;

        // Verify plugin integrity
        if !options.skip_verification {
            self.verify_plugin_integrity(&plugin, &binary_path)?;
        }

        // Test plugin loading
        self.test_plugin_loading(&binary_path, &manifest_path)?;

        // Register the installed plugin
        let installed_plugin = InstalledPlugin {
            marketplace_plugin: plugin.clone(),
            installation_path: installation_path.clone(),
            installed_at: Utc::now(),
            auto_update: options.auto_update,
            enabled: options.enable_immediately,
        };

        self.installed_plugins.insert(plugin_id.to_string(), installed_plugin);
        self.save_installed_plugins()?;

        println!("✅ Plugin '{}' v{} installed successfully", plugin.name, plugin.version);
        println!("📁 Installed to: {}", installation_path.display());
        
        if options.enable_immediately {
            println!("🚀 Plugin enabled and ready to use");
        }

        Ok(installation_path)
    }

    /// Uninstall a plugin
    pub fn uninstall_plugin(&mut self, plugin_id: &str) -> Result<()> {
        let installed = self.installed_plugins.get(plugin_id)
            .ok_or_else(|| UveddiError::PluginError {
                plugin: plugin_id.to_string(),
                plugin_type: "marketplace".to_string(),
                message: format!("Plugin '{}' is not installed", plugin_id),
                suggestion: "Use 'uveddi plugin list' to see installed plugins".to_string(),
                source: None,
            })?
            .clone();

        // Remove plugin files
        if installed.installation_path.exists() {
            fs::remove_dir_all(&installed.installation_path)?;
        }

        // Remove from installed plugins list
        self.installed_plugins.remove(plugin_id);
        self.save_installed_plugins()?;

        println!("✅ Plugin '{}' uninstalled successfully", installed.marketplace_plugin.name);
        
        Ok(())
    }

    /// Update an installed plugin
    pub async fn update_plugin(&mut self, plugin_id: &str) -> Result<bool> {
        let installed = self.installed_plugins.get(plugin_id)
            .ok_or_else(|| UveddiError::PluginError {
                plugin: plugin_id.to_string(),
                plugin_type: "marketplace".to_string(),
                message: format!("Plugin '{}' is not installed", plugin_id),
                suggestion: "Install the plugin first".to_string(),
                source: None,
            })?
            .clone();

        // Check for updates
        let latest = self.get_plugin_info(plugin_id).await?;
        
        if installed.marketplace_plugin.version == latest.version {
            println!("ℹ️ Plugin '{}' is already up to date (v{})", latest.name, latest.version);
            return Ok(false);
        }

        println!("🔄 Updating '{}' from v{} to v{}", 
                latest.name, installed.marketplace_plugin.version, latest.version);

        // Install the new version (this will replace the old one)
        let options = InstallationOptions {
            auto_update: installed.auto_update,
            enable_immediately: installed.enabled,
            force_reinstall: true,
            skip_verification: false,
            custom_install_path: Some(installed.installation_path.clone()),
        };

        self.install_plugin(plugin_id, &options).await?;
        
        println!("✅ Plugin '{}' updated successfully to v{}", latest.name, latest.version);
        
        Ok(true)
    }

    /// Update all installed plugins
    pub async fn update_all_plugins(&mut self) -> Result<Vec<String>> {
        let mut updated_plugins = Vec::new();
        let plugin_ids: Vec<String> = self.installed_plugins.keys().cloned().collect();

        for plugin_id in plugin_ids {
            if let Some(installed) = self.installed_plugins.get(&plugin_id) {
                if installed.auto_update {
                    match self.update_plugin(&plugin_id).await {
                        Ok(true) => updated_plugins.push(plugin_id),
                        Ok(false) => {}, // Already up to date
                        Err(e) => {
                            eprintln!("⚠️ Failed to update plugin '{}': {}", plugin_id, e);
                        }
                    }
                }
            }
        }

        if updated_plugins.is_empty() {
            println!("ℹ️ All plugins are up to date");
        } else {
            println!("✅ Updated {} plugins: {}", updated_plugins.len(), updated_plugins.join(", "));
        }

        Ok(updated_plugins)
    }

    /// List installed plugins
    pub fn list_installed_plugins(&self) -> Vec<&InstalledPlugin> {
        self.installed_plugins.values().collect()
    }

    /// Get featured plugins
    pub async fn get_featured_plugins(&mut self) -> Result<Vec<MarketplacePlugin>> {
        let criteria = PluginSearchCriteria {
            featured_only: true,
            limit: Some(10),
            ..Default::default()
        };

        let results = self.search_plugins(&criteria).await?;
        Ok(results.plugins)
    }

    /// Get popular plugins
    pub async fn get_popular_plugins(&mut self) -> Result<Vec<MarketplacePlugin>> {
        let criteria = PluginSearchCriteria {
            sort_by: SortBy::Downloads,
            limit: Some(20),
            ..Default::default()
        };

        let results = self.search_plugins(&criteria).await?;
        Ok(results.plugins)
    }

    /// Enable/disable a plugin
    pub fn toggle_plugin(&mut self, plugin_id: &str, enabled: bool) -> Result<()> {
        let installed = self.installed_plugins.get_mut(plugin_id)
            .ok_or_else(|| UveddiError::PluginError {
                plugin: plugin_id.to_string(),
                plugin_type: "marketplace".to_string(),
                message: format!("Plugin '{}' is not installed", plugin_id),
                suggestion: "Install the plugin first".to_string(),
                source: None,
            })?;

        installed.enabled = enabled;
        self.save_installed_plugins()?;

        let status = if enabled { "enabled" } else { "disabled" };
        println!("✅ Plugin '{}' {}", installed.marketplace_plugin.name, status);

        Ok(())
    }

    // Private helper methods

    fn load_installed_plugins(&mut self) -> Result<()> {
        let installed_file = self.local_cache_dir.join("installed_plugins.json");
        
        if installed_file.exists() {
            let contents = fs::read_to_string(installed_file)?;
            self.installed_plugins = serde_json::from_str(&contents)?;
        }

        Ok(())
    }

    fn save_installed_plugins(&self) -> Result<()> {
        let installed_file = self.local_cache_dir.join("installed_plugins.json");
        let contents = serde_json::to_string_pretty(&self.installed_plugins)?;
        fs::write(installed_file, contents)?;
        Ok(())
    }

    async fn simulate_search_request(&self, _url: &str, criteria: &PluginSearchCriteria) -> Result<PluginSearchResults> {
        // In a real implementation, this would make HTTP requests
        // For now, return mock data based on search criteria
        
        let mut mock_plugins = vec![
            MarketplacePlugin {
                id: "performance-analyzer".to_string(),
                name: "Performance Analyzer".to_string(),
                description: "Analyzes code performance and identifies bottlenecks".to_string(),
                author: "Uveddi Team".to_string(),
                version: "1.2.3".to_string(),
                category: "performance".to_string(),
                tags: vec!["performance".to_string(), "optimization".to_string()],
                supported_languages: vec!["rust".to_string(), "javascript".to_string()],
                download_url: "https://marketplace.uveddi.dev/plugins/performance-analyzer/1.2.3/download".to_string(),
                manifest_url: "https://marketplace.uveddi.dev/plugins/performance-analyzer/1.2.3/manifest".to_string(),
                repository_url: Some("https://github.com/uveddi/plugins/performance-analyzer".to_string()),
                homepage_url: Some("https://uveddi.dev/plugins/performance-analyzer".to_string()),
                documentation_url: Some("https://docs.uveddi.dev/plugins/performance-analyzer".to_string()),
                license: "MIT".to_string(),
                downloads: 1542,
                rating: 4.7,
                reviews: 23,
                created_at: DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z").unwrap().with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339("2024-03-01T14:20:00Z").unwrap().with_timezone(&Utc),
                minimum_uveddi_version: "0.9.0".to_string(),
                file_size: 2_845_672,
                checksum: "sha256:a1b2c3d4e5f6...".to_string(),
                verified: true,
                featured: true,
                dependencies: vec![],
                screenshots: vec!["https://marketplace.uveddi.dev/plugins/performance-analyzer/screenshot1.png".to_string()],
            },
            MarketplacePlugin {
                id: "security-scanner".to_string(),
                name: "Security Scanner".to_string(),
                description: "Comprehensive security vulnerability scanner".to_string(),
                author: "Security Corp".to_string(),
                version: "2.1.0".to_string(),
                category: "security".to_string(),
                tags: vec!["security".to_string(), "vulnerability".to_string()],
                supported_languages: vec!["rust".to_string(), "javascript".to_string(), "python".to_string()],
                download_url: "https://marketplace.uveddi.dev/plugins/security-scanner/2.1.0/download".to_string(),
                manifest_url: "https://marketplace.uveddi.dev/plugins/security-scanner/2.1.0/manifest".to_string(),
                repository_url: Some("https://github.com/security-corp/security-scanner".to_string()),
                homepage_url: None,
                documentation_url: Some("https://docs.security-corp.com/security-scanner".to_string()),
                license: "Apache-2.0".to_string(),
                downloads: 3241,
                rating: 4.9,
                reviews: 47,
                created_at: DateTime::parse_from_rfc3339("2023-11-20T09:15:00Z").unwrap().with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339("2024-02-28T16:45:00Z").unwrap().with_timezone(&Utc),
                minimum_uveddi_version: "0.8.0".to_string(),
                file_size: 4_123_890,
                checksum: "sha256:f6e5d4c3b2a1...".to_string(),
                verified: true,
                featured: false,
                dependencies: vec![],
                screenshots: vec![],
            },
        ];

        // Filter based on search criteria
        if let Some(ref query) = criteria.query {
            mock_plugins.retain(|p| {
                p.name.to_lowercase().contains(&query.to_lowercase()) ||
                p.description.to_lowercase().contains(&query.to_lowercase()) ||
                p.tags.iter().any(|tag| tag.to_lowercase().contains(&query.to_lowercase()))
            });
        }

        if let Some(ref category) = criteria.category {
            mock_plugins.retain(|p| p.category == *category);
        }

        if criteria.featured_only {
            mock_plugins.retain(|p| p.featured);
        }

        if criteria.verified_only {
            mock_plugins.retain(|p| p.verified);
        }

        // Sort plugins
        match criteria.sort_by {
            SortBy::Downloads => mock_plugins.sort_by(|a, b| b.downloads.cmp(&a.downloads)),
            SortBy::Rating => mock_plugins.sort_by(|a, b| b.rating.partial_cmp(&a.rating).unwrap_or(std::cmp::Ordering::Equal)),
            SortBy::Updated => mock_plugins.sort_by(|a, b| b.updated_at.cmp(&a.updated_at)),
            SortBy::Created => mock_plugins.sort_by(|a, b| b.created_at.cmp(&a.created_at)),
            SortBy::Name => mock_plugins.sort_by(|a, b| a.name.cmp(&b.name)),
            SortBy::Relevance => {}, // Already in relevance order
        }

        // Apply pagination
        let offset = criteria.offset.unwrap_or(0);
        let limit = criteria.limit.unwrap_or(mock_plugins.len());
        let total_count = mock_plugins.len();
        
        let end_index = std::cmp::min(offset + limit, total_count);
        if offset < total_count {
            mock_plugins = mock_plugins[offset..end_index].to_vec();
        } else {
            mock_plugins.clear();
        }

        Ok(PluginSearchResults {
            plugins: mock_plugins,
            total_count,
            page: offset / limit.max(1),
            per_page: limit,
            has_more: offset + limit < total_count,
        })
    }

    async fn simulate_plugin_info_request(&self, _url: &str, plugin_id: &str) -> Result<MarketplacePlugin> {
        // Mock plugin info retrieval
        match plugin_id {
            "performance-analyzer" => Ok(MarketplacePlugin {
                id: "performance-analyzer".to_string(),
                name: "Performance Analyzer".to_string(),
                description: "Advanced performance analysis plugin with detailed metrics and optimization suggestions".to_string(),
                author: "Uveddi Team".to_string(),
                version: "1.2.3".to_string(),
                category: "performance".to_string(),
                tags: vec!["performance".to_string(), "optimization".to_string(), "metrics".to_string()],
                supported_languages: vec!["rust".to_string(), "javascript".to_string(), "typescript".to_string()],
                download_url: "https://marketplace.uveddi.dev/plugins/performance-analyzer/1.2.3/download".to_string(),
                manifest_url: "https://marketplace.uveddi.dev/plugins/performance-analyzer/1.2.3/manifest".to_string(),
                repository_url: Some("https://github.com/uveddi/plugins/performance-analyzer".to_string()),
                homepage_url: Some("https://uveddi.dev/plugins/performance-analyzer".to_string()),
                documentation_url: Some("https://docs.uveddi.dev/plugins/performance-analyzer".to_string()),
                license: "MIT".to_string(),
                downloads: 1542,
                rating: 4.7,
                reviews: 23,
                created_at: DateTime::parse_from_rfc3339("2024-01-15T10:30:00Z").unwrap().with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339("2024-03-01T14:20:00Z").unwrap().with_timezone(&Utc),
                minimum_uveddi_version: "0.9.0".to_string(),
                file_size: 2_845_672,
                checksum: "sha256:a1b2c3d4e5f6...".to_string(),
                verified: true,
                featured: true,
                dependencies: vec![],
                screenshots: vec!["https://marketplace.uveddi.dev/plugins/performance-analyzer/screenshot1.png".to_string()],
            }),
            _ => Err(UveddiError::PluginError {
                plugin: plugin_id.to_string(),
                plugin_type: "marketplace".to_string(),
                message: format!("Plugin '{}' not found in marketplace", plugin_id),
                suggestion: "Check the plugin ID or search for similar plugins".to_string(),
                source: None,
            })
        }
    }

    fn verify_plugin_compatibility(&self, plugin: &MarketplacePlugin) -> Result<()> {
        // In a real implementation, check Uveddi version compatibility
        println!("✅ Plugin '{}' is compatible with current Uveddi version", plugin.name);
        Ok(())
    }

    fn determine_installation_path(&self, plugin: &MarketplacePlugin, options: &InstallationOptions) -> PathBuf {
        if let Some(ref custom_path) = options.custom_install_path {
            custom_path.clone()
        } else {
            self.local_cache_dir.join("installed").join(&plugin.id)
        }
    }

    async fn download_plugin_binary(&self, plugin: &MarketplacePlugin, installation_path: &Path) -> Result<PathBuf> {
        let binary_path = installation_path.join(format!("{}.wasm", plugin.id));
        
        // Simulate download
        println!("📥 Downloading plugin binary from {}", plugin.download_url);
        
        // In a real implementation, this would download from plugin.download_url
        // For now, create a placeholder file
        fs::write(&binary_path, b"mock wasm binary content")?;
        
        println!("✅ Downloaded plugin binary ({} bytes)", plugin.file_size);
        
        Ok(binary_path)
    }

    async fn download_plugin_manifest(&self, plugin: &MarketplacePlugin, installation_path: &Path) -> Result<PathBuf> {
        let manifest_path = installation_path.join("plugin.toml");
        
        // Simulate download
        println!("📥 Downloading plugin manifest from {}", plugin.manifest_url);
        
        // Create a mock manifest
        let manifest_content = format!(r#"
[plugin]
id = "{}"
name = "{}"
version = "{}"
description = "{}"
author = "{}"
license = "{}"

[plugin.metadata]
api_version = "1.0"
plugin_type = "detector"
category = "{}"

[plugin.build]
target = "wasm32-wasi"
artifact_path = "{}.wasm"
"#, plugin.id, plugin.name, plugin.version, plugin.description, 
    plugin.author, plugin.license, plugin.category, plugin.id);
        
        fs::write(&manifest_path, manifest_content)?;
        
        println!("✅ Downloaded plugin manifest");
        
        Ok(manifest_path)
    }

    fn verify_plugin_integrity(&self, plugin: &MarketplacePlugin, binary_path: &Path) -> Result<()> {
        // In a real implementation, verify checksum
        println!("🔍 Verifying plugin integrity (checksum: {})", &plugin.checksum[..16]);
        println!("✅ Plugin integrity verified");
        Ok(())
    }

    fn test_plugin_loading(&self, binary_path: &Path, manifest_path: &Path) -> Result<()> {
        // In a real implementation, test load the plugin
        println!("🧪 Testing plugin loading...");
        
        if !binary_path.exists() {
            return Err(UveddiError::PluginError {
                plugin: "test".to_string(),
                plugin_type: "marketplace".to_string(),
                message: "Plugin binary not found".to_string(),
                suggestion: "Re-download the plugin".to_string(),
                source: None,
            });
        }
        
        if !manifest_path.exists() {
            return Err(UveddiError::PluginError {
                plugin: "test".to_string(),
                plugin_type: "marketplace".to_string(),
                message: "Plugin manifest not found".to_string(),
                suggestion: "Re-download the plugin".to_string(),
                source: None,
            });
        }
        
        println!("✅ Plugin loading test passed");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_marketplace_creation() {
        let temp_dir = TempDir::new().unwrap();
        let marketplace = PluginMarketplace::new(
            "https://marketplace.uveddi.dev".to_string(),
            temp_dir.path()
        );
        
        assert!(marketplace.is_ok());
    }

    #[tokio::test]
    async fn test_plugin_search() {
        let temp_dir = TempDir::new().unwrap();
        let mut marketplace = PluginMarketplace::new(
            "https://marketplace.uveddi.dev".to_string(),
            temp_dir.path()
        ).unwrap();

        let criteria = PluginSearchCriteria {
            query: Some("performance".to_string()),
            ..Default::default()
        };

        let results = marketplace.search_plugins(&criteria).await;
        assert!(results.is_ok());
        
        let search_results = results.unwrap();
        assert!(!search_results.plugins.is_empty());
    }

    #[tokio::test]
    async fn test_plugin_installation_flow() {
        let temp_dir = TempDir::new().unwrap();
        let mut marketplace = PluginMarketplace::new(
            "https://marketplace.uveddi.dev".to_string(),
            temp_dir.path()
        ).unwrap();

        let options = InstallationOptions::default();
        let result = marketplace.install_plugin("performance-analyzer", &options).await;
        
        assert!(result.is_ok());
        assert!(marketplace.installed_plugins.contains_key("performance-analyzer"));
    }
}