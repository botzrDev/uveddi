//! Component Model plugin loader
//!
//! This module provides Component Model-based plugin loading and execution,
//! replacing the legacy core module approach.

use crate::plugins::{
    component_host::ComponentHostState,
    errors::PluginError,
    registry::PluginManifest,
    security::SecurityPolicy,
    types::{PluginConfig, PluginId, PluginStats, PluginStatus},
    wasm::CoreAnalysis,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use wasmtime::component::{Component, Linker};
use wasmtime::{Engine, Store};

/// Active Component Model plugin
/// Note: This struct stores store and bindings separately since CoreAnalysis
/// doesn't implement Clone
pub struct ComponentPlugin {
    /// Plugin identifier
    pub id: PluginId,
    /// Plugin manifest
    pub manifest: PluginManifest,
    /// Runtime statistics
    pub stats: PluginStats,
    /// Current status
    pub status: PluginStatus,
    /// Wasmtime engine
    engine: Engine,
    /// The compiled component
    component: Component,
    /// Store with host state - needs mutable access for calls
    store: std::sync::Mutex<Store<ComponentHostState>>,
    /// Linker for re-instantiation if needed
    linker: Arc<Linker<ComponentHostState>>,
}

impl std::fmt::Debug for ComponentPlugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComponentPlugin")
            .field("id", &self.id)
            .field("manifest", &self.manifest)
            .field("stats", &self.stats)
            .field("status", &self.status)
            .finish()
    }
}

/// Component Model plugin loader
pub struct ComponentPluginLoader {
    /// Active plugins
    plugins: Arc<RwLock<HashMap<PluginId, Arc<ComponentPlugin>>>>,
}

impl ComponentPluginLoader {
    /// Create a new component plugin loader
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Load a plugin using Component Model
    pub async fn load_plugin(
        &self,
        plugin_id: PluginId,
        manifest: PluginManifest,
        binary: Vec<u8>,
        security_policy: SecurityPolicy,
    ) -> Result<(), PluginError> {
        tracing::info!("Loading Component Model plugin: {}", plugin_id);

        // 1. Create engine with component model enabled
        let engine = Engine::new(&security_policy.configure_engine()?)
            .map_err(|e| PluginError::Loading(format!("Failed to create engine: {}", e)))?;

        // 2. Compile the component
        let component = Component::new(&engine, &binary)
            .map_err(|e| PluginError::Loading(format!("Failed to compile component: {}", e)))?;

        // 3. Create linker and add host functions
        let mut linker = Linker::<ComponentHostState>::new(&engine);
        
        // Add our CoreAnalysis host functions to the linker
        // This registers the imports that the guest component will call
        // Use HasSelf to indicate we provide the host state directly
        CoreAnalysis::add_to_linker::<_, wasmtime::component::HasSelf<ComponentHostState>>(&mut linker, |state| state)
            .map_err(|e| PluginError::Loading(format!("Failed to add host functions: {}", e)))?;

        // 4. Create host state with WASI context
        let wasi_ctx = security_policy.configure_wasi_context()?.build_p1();
        let host_state = ComponentHostState::new(
            plugin_id.clone(),
            PluginConfig::default(),
            security_policy.clone(),
            wasi_ctx,
        );

        // 5. Create store with fuel limits
        let mut store = Store::new(&engine, host_state);
        store.set_fuel(security_policy.resource_limits.max_fuel)
            .map_err(|e| PluginError::Loading(format!("Failed to set fuel: {}", e)))?;

        // 6. Instantiate the component
        let bindings = CoreAnalysis::instantiate(&mut store, &component, &linker)
            .map_err(|e| PluginError::Loading(format!("Failed to instantiate component: {}", e)))?;

        // 7. Call initialize on the plugin
        let config = crate::plugins::wasm::PluginConfig {
            severity_threshold: crate::plugins::wasm::SeverityLevel::Medium,
            max_issues_per_file: 100,
            include_patterns: vec![],
            exclude_patterns: vec![],
            rule_overrides: vec![],
            custom_settings: vec![],
        };
        let limits = crate::plugins::wasm::ResourceLimits {
            max_memory_mb: (security_policy.resource_limits.max_memory / 1024 / 1024) as u32,
            max_cpu_percent: 100,
            timeout_seconds: (security_policy.resource_limits.max_execution_time_ms / 1000) as u32,
            max_file_handles: security_policy.resource_limits.max_file_handles,
        };

        bindings.call_initialize(&mut store, &config, limits)
            .map_err(|e| PluginError::Loading(format!("Failed to initialize plugin: {}", e)))?
            .map_err(|e| PluginError::Loading(format!("Plugin initialization returned error: {}", e)))?;

        // 8. Get plugin info
        let plugin_info = bindings.call_get_info(&mut store)
            .map_err(|e| PluginError::Loading(format!("Failed to get plugin info: {}", e)))?;

        tracing::info!(
            "Loaded plugin: {} v{} by {}",
            plugin_info.name,
            plugin_info.version,
            plugin_info.author
        );

        // 9. Store the active plugin
        let active_plugin = Arc::new(ComponentPlugin {
            id: plugin_id.clone(),
            manifest,
            stats: PluginStats::default(),
            status: PluginStatus::Ready,
            engine,
            component,
            store: std::sync::Mutex::new(store),
            linker: Arc::new(linker),
        });

        self.plugins.write().await.insert(plugin_id.clone(), active_plugin);

        tracing::info!("Successfully loaded Component Model plugin: {}", plugin_id);
        Ok(())
    }

    /// Analyze a file using a plugin
    pub async fn analyze_file(
        &self,
        plugin_id: &PluginId,
        file_path: &str,
        file_content: &str,
        language: &str,
    ) -> Result<crate::plugins::lifecycle::PluginAnalysisResult, PluginError> {
        use crate::plugins::lifecycle::{PluginAnalysisResult, PluginIssueResult, PluginMetrics};
        use crate::plugins::wasm::SourceFile;
        use std::time::Instant;

        let plugins = self.plugins.read().await;
        let plugin = plugins.get(plugin_id)
            .ok_or_else(|| PluginError::NotFound(plugin_id.to_string()))?;

        let start_time = Instant::now();

        // Prepare source file for analysis
        let source_file = SourceFile {
            path: file_path.to_string(),
            content: file_content.to_string(),
            language: language.to_string(),
            size: file_content.len() as u32,
            hash: format!("{:x}", md5::compute(file_content.as_bytes())),
            ast: None, // Plugin can parse its own AST if needed
        };

        // Get mutable access to store
        let mut store = plugin.store.lock()
            .map_err(|e| PluginError::Execution(format!("Failed to acquire store lock: {}", e)))?;

        // Re-instantiate bindings to call the analyze function
        // This is needed because CoreAnalysis doesn't implement Clone
        let bindings = CoreAnalysis::instantiate(&mut *store, &plugin.component, &plugin.linker)
            .map_err(|e| PluginError::Execution(format!("Failed to get bindings: {}", e)))?;

        // Call the plugin's analyze function
        let result = bindings.call_analyze(&mut *store, &source_file)
            .map_err(|e| PluginError::Execution(format!("Analysis call failed: {}", e)))?
            .map_err(|e| PluginError::Execution(format!("Plugin returned error: {}", e)))?;

        let duration_ms = start_time.elapsed().as_millis() as u32;

        // Convert WIT types to internal types
        let issues: Vec<PluginIssueResult> = result.issues.into_iter().map(|issue| {
            PluginIssueResult {
                id: issue.id,
                severity: format!("{:?}", issue.severity).to_lowercase(),
                category: format!("{:?}", issue.category).to_lowercase(),
                message: issue.message,
                description: issue.description,
                file: issue.file,
                start_line: issue.span.start.line,
                start_column: issue.span.start.column,
                end_line: issue.span.end.line,
                end_column: issue.span.end.column,
                suggestion: issue.suggestion,
            }
        }).collect();

        let metrics = PluginMetrics {
            lines_of_code: result.metrics.lines_of_code,
            complexity: result.metrics.complexity,
            maintainability_index: result.metrics.maintainability_index,
        };

        Ok(PluginAnalysisResult {
            issues,
            metrics,
            duration_ms,
        })
    }

    /// Unload a plugin
    pub async fn unload_plugin(&self, plugin_id: &PluginId) -> Result<(), PluginError> {
        let mut plugins = self.plugins.write().await;
        
        if let Some(plugin) = plugins.get(plugin_id) {
            // Call cleanup on the plugin
            let mut store = plugin.store.lock()
                .map_err(|e| PluginError::Execution(format!("Failed to acquire store lock: {}", e)))?;

            // Re-instantiate to call cleanup
            if let Ok(bindings) = CoreAnalysis::instantiate(&mut *store, &plugin.component, &plugin.linker) {
                let _ = bindings.call_cleanup(&mut *store);
            }
        }

        plugins.remove(plugin_id);
        tracing::info!("Unloaded Component Model plugin: {}", plugin_id);
        Ok(())
    }

    /// List active plugins
    pub async fn list_plugins(&self) -> Vec<PluginId> {
        self.plugins.read().await.keys().cloned().collect()
    }

    /// Check if a plugin is loaded
    pub async fn has_plugin(&self, plugin_id: &PluginId) -> bool {
        self.plugins.read().await.contains_key(plugin_id)
    }
}

impl Default for ComponentPluginLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for ComponentPluginLoader {
    fn clone(&self) -> Self {
        Self {
            plugins: self.plugins.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_component_loader_creation() {
        let loader = ComponentPluginLoader::new();
        assert!(loader.list_plugins().await.is_empty());
    }
}
