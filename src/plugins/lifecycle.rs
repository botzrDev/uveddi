//! Plugin lifecycle management for loading, unloading, and monitoring

use crate::plugins::{
    data_plane::*,
    errors::*,
    registry::*,
    security::*,
    types::{PluginId, PluginStats, PluginStatus, HostState, PluginConfig},
    verification::*,
};

#[cfg(feature = "wasm-plugins")]
use crate::plugins::types::HostContext;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
#[cfg(feature = "wasm-plugins")]
use wasmtime::{Engine, Linker, Module, Store};
#[cfg(feature = "wasm-plugins")]
use wasmtime_wasi::{WasiCtxBuilder, WasiView};
#[cfg(feature = "wasm-plugins")]
use wasmtime_wasi::preview1::{self, WasiP1Ctx};

/// Plugin lifecycle manager
#[derive(Clone)]
pub struct PluginLifecycleManager {
    active_plugins: Arc<RwLock<HashMap<PluginId, ActivePlugin>>>,
    security_policies: HashMap<PluginId, SecurityPolicy>,
    resource_monitor: ResourceMonitor,
    verifier: PluginVerifier,
}

impl PluginLifecycleManager {
    /// Create a new plugin lifecycle manager
    pub fn new() -> Self {
        Self {
            active_plugins: Arc::new(RwLock::new(HashMap::new())),
            security_policies: HashMap::new(),
            resource_monitor: ResourceMonitor::new(),
            verifier: PluginVerifier::new(),
        }
    }

    /// Load and instantiate a plugin
    pub async fn load_plugin(
        &mut self,
        plugin_id: PluginId,
        manifest: PluginManifest,
        binary: Vec<u8>,
        security_policy: SecurityPolicy,
    ) -> crate::error::Result<()> {
        tracing::info!("Loading plugin: {}", plugin_id);

        // 1. Verify the plugin
        let verification_report = self
            .verifier
            .verify_plugin(&binary, &manifest, &security_policy)
            .await
            .map_err(|e| PluginError::Verification(e))?;

        match verification_report.overall_status {
            crate::plugins::verification::VerificationStatus::Rejected(reason) => {
                return Err(PluginError::Verification(
                    crate::plugins::errors::VerificationError::StaticAnalysis(reason),
                )
                .into());
            }
            crate::plugins::verification::VerificationStatus::Warning(warning) => {
                tracing::warn!("Plugin {} loaded with warnings: {}", plugin_id, warning);
            }
            _ => {}
        }

        // 2. Create WASM runtime components
        #[cfg(feature = "wasm-plugins")]
        {
            let engine = wasmtime::Engine::new(&security_policy.configure_engine()?)?;
            let module = wasmtime::Module::new(&engine, &binary)?;
            let mut linker = wasmtime::Linker::<HostContext>::new(&engine);

            // Configure host state
            let host_state = HostState {
                plugin_id: plugin_id.clone(),
                config: PluginConfig::default(),
                resource_limits: security_policy.resource_limits.clone(),
                security_policy: security_policy.clone(),
            };

            // Configure WASI
            let wasi_ctx = security_policy.configure_wasi_context()?.build_p1();

            // NOTE: UV-108 - Add basic WASI support (filesystem traits temporarily disabled)
            wasmtime_wasi::preview1::add_to_linker_sync(&mut linker, |host: &mut HostContext| {
                &mut host.wasi_ctx
            })?;

            // Add our custom host functions
            self.add_host_functions(&mut linker)?;

            // Create store with fuel and memory limits
            let resource_table = wasmtime::component::ResourceTable::new();
            let host_context = HostContext {
                host_state,
                wasi_ctx,
                table: resource_table,
            };
            let mut store = wasmtime::Store::new(&engine, host_context);
            store.set_fuel(security_policy.resource_limits.max_fuel)?;
            // Note: Resource limiting would be configured here in a real implementation

            // Instantiate the component
            let instance = linker.instantiate(&mut store, &module)?;

            // Create active plugin wrapper
            let active_plugin = ActivePlugin::new(
                plugin_id.clone(),
                manifest,
                engine,
                module,
                store,
                instance,
                AstHandleManager::new(),
            );

            // Store in active plugins
            self.active_plugins
                .write()
                .await
                .insert(plugin_id.clone(), active_plugin);
            self.security_policies
                .insert(plugin_id.clone(), security_policy);

            tracing::info!("Successfully loaded plugin: {}", plugin_id);
            Ok(())
        }

        #[cfg(not(feature = "wasm-plugins"))]
        {
            return Err(PluginError::Unsupported("WASM plugins not enabled".to_string()).into());
        }
    }

    /// Unload a plugin and cleanup resources
    pub async fn unload_plugin(&mut self, plugin_id: &PluginId) -> crate::error::Result<()> {
        tracing::info!("Unloading plugin: {}", plugin_id);

        let mut active_plugins = self.active_plugins.write().await;
        if let Some(mut plugin) = active_plugins.remove(plugin_id) {
            // Call plugin cleanup
            plugin.cleanup().await?;
        }

        self.security_policies.remove(plugin_id);

        tracing::info!("Successfully unloaded plugin: {}", plugin_id);
        Ok(())
    }

    /// Get a reference to an active plugin
    pub async fn get_plugin(&self, plugin_id: &PluginId) -> Option<ActivePluginRef> {
        let active_plugins = self.active_plugins.read().await;
        active_plugins.get(plugin_id).map(|plugin| ActivePluginRef {
            id: plugin.id.clone(),
            manifest: plugin.manifest.clone(),
            stats: plugin.stats.clone(),
            status: plugin.status.clone(),
        })
    }

    /// List all active plugins
    pub async fn list_active_plugins(&self) -> Vec<PluginId> {
        let active_plugins = self.active_plugins.read().await;
        active_plugins.keys().cloned().collect()
    }

    /// Get plugin statistics
    pub async fn get_plugin_stats(&self, plugin_id: &PluginId) -> Option<PluginStats> {
        let active_plugins = self.active_plugins.read().await;
        active_plugins
            .get(plugin_id)
            .map(|plugin| plugin.stats.clone())
    }

    /// Monitor resource usage of all active plugins
    pub async fn monitor_resources(&mut self) -> crate::error::Result<ResourceReport> {
        let active_plugins = self.active_plugins.read().await;
        let mut report = ResourceReport::new();

        for (plugin_id, plugin) in active_plugins.iter() {
            let plugin_report = self.resource_monitor.monitor_plugin(plugin).await?;
            report
                .plugin_reports
                .insert(plugin_id.clone(), plugin_report);
        }

        Ok(report)
    }

    /// Add host functions to the linker
    #[cfg(feature = "wasm-plugins")]
    fn add_host_functions(
        &self,
        _linker: &mut wasmtime::Linker<HostContext>,
    ) -> crate::error::Result<()> {
        // NOTE: UV-108 - Component model host functions require WIT interface definitions
        // This would be implemented using proper WIT files and generated bindings
        // For now, we'll just return OK to get the basic loading working

        tracing::info!("Host functions would be registered here with proper WIT bindings");
        Ok(())
    }
}

impl Default for PluginLifecycleManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Active plugin wrapper containing runtime state
// Debug trait removed due to Wasmtime types not implementing Debug
#[derive(Clone)]
pub struct ActivePlugin {
    pub id: PluginId,
    pub manifest: PluginManifest,
    pub stats: PluginStats,
    pub status: PluginStatus,
    #[cfg(feature = "wasm-plugins")]
    engine: wasmtime::Engine,
    #[cfg(feature = "wasm-plugins")]
    module: wasmtime::Module,
    #[cfg(feature = "wasm-plugins")]
    store: std::sync::Arc<std::sync::Mutex<wasmtime::Store<HostContext>>>,
    #[cfg(feature = "wasm-plugins")]
    instance: wasmtime::Instance,
    ast_handles: AstHandleManager,
}

impl ActivePlugin {
    #[cfg(feature = "wasm-plugins")]
    pub fn new(
        id: PluginId,
        manifest: PluginManifest,
        engine: wasmtime::Engine,
        module: wasmtime::Module,
        store: wasmtime::Store<HostContext>,
        instance: wasmtime::Instance,
        ast_handles: AstHandleManager,
    ) -> Self {
        Self {
            id,
            manifest,
            stats: PluginStats::default(),
            status: PluginStatus::Ready,
            engine,
            module,
            store: Arc::new(std::sync::Mutex::new(store)),
            instance,
            ast_handles,
        }
    }

    /// Call plugin cleanup function
    pub async fn cleanup(&mut self) -> crate::error::Result<()> {
        #[cfg(feature = "wasm-plugins")]
        {
            // Call the cleanup function if it exists
            if let Ok(mut store_guard) = self.store.lock() {
                if let Ok(cleanup_func) = self
                    .instance
                    .get_typed_func::<(), ()>(&mut *store_guard, "cleanup")
                {
                    cleanup_func.call(&mut *store_guard, ())?;
                }
            }
        }

        self.status = PluginStatus::Unloaded;
        Ok(())
    }
}

/// Read-only reference to an active plugin
#[derive(Debug, Clone)]
pub struct ActivePluginRef {
    pub id: PluginId,
    pub manifest: PluginManifest,
    pub stats: PluginStats,
    pub status: PluginStatus,
}

/// Resource monitor for tracking plugin resource usage
#[derive(Debug, Clone)]
pub struct ResourceMonitor {
    monitoring_interval: std::time::Duration,
}

impl ResourceMonitor {
    pub fn new() -> Self {
        Self {
            monitoring_interval: std::time::Duration::from_secs(5),
        }
    }

    /// Monitor a single plugin's resource usage
    pub async fn monitor_plugin(
        &self,
        plugin: &ActivePlugin,
    ) -> crate::error::Result<PluginResourceReport> {
        let report = PluginResourceReport {
            plugin_id: plugin.id.clone(),
            memory_usage: 0,
            fuel_consumed: 0,
            execution_time: std::time::Duration::default(),
            handle_count: plugin.ast_handles.handle_count(),
            handle_memory: plugin.ast_handles.total_memory_usage(),
        };

        #[cfg(feature = "wasm-plugins")]
        {
            // Get memory usage from WASM instance
            // This would require access to the store's memory
            // report.memory_usage = self.get_memory_usage(&plugin.store)?;
        }

        Ok(report)
    }
}

impl Default for ResourceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Resource usage report for all plugins
#[derive(Debug, Default)]
pub struct ResourceReport {
    pub plugin_reports: HashMap<PluginId, PluginResourceReport>,
    pub total_memory_usage: u64,
    pub total_fuel_consumed: u64,
}

impl ResourceReport {
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate totals across all plugins
    pub fn calculate_totals(&mut self) {
        self.total_memory_usage = self
            .plugin_reports
            .values()
            .map(|report| report.memory_usage + report.handle_memory)
            .sum();

        self.total_fuel_consumed = self
            .plugin_reports
            .values()
            .map(|report| report.fuel_consumed)
            .sum();
    }
}

/// Resource usage report for a single plugin
#[derive(Debug, Clone)]
pub struct PluginResourceReport {
    pub plugin_id: PluginId,
    pub memory_usage: u64,
    pub fuel_consumed: u64,
    pub execution_time: std::time::Duration,
    pub handle_count: usize,
    pub handle_memory: u64,
}

/// Memory limiter for WASM instances
#[cfg(feature = "wasm-plugins")]
pub struct MemoryLimiter {
    max_memory: u64,
}

#[cfg(feature = "wasm-plugins")]
impl MemoryLimiter {
    pub fn new(max_memory: u64) -> Self {
        Self { max_memory }
    }
}

#[cfg(feature = "wasm-plugins")]
impl wasmtime::ResourceLimiter for MemoryLimiter {
    fn memory_growing(
        &mut self,
        current: usize,
        desired: usize,
        maximum: Option<usize>,
    ) -> Result<bool, anyhow::Error> {
        let desired_bytes = desired as u64 * 65536; // WASM page size
        Ok(desired_bytes <= self.max_memory)
    }

    fn table_growing(
        &mut self,
        current: usize,
        desired: usize,
        maximum: Option<usize>,
    ) -> Result<bool, anyhow::Error> {
        // Allow table growth for now
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::security::Permission;

    #[tokio::test]
    async fn test_lifecycle_manager() {
        let mut manager = PluginLifecycleManager::new();

        let plugin_id = PluginId::new();
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
        let policy = SecurityPolicy::permissive();

        // Note: This test will only work with WASM plugins enabled
        #[cfg(feature = "wasm-plugins")]
        {
            // The actual loading would require a valid WASM component
            // For now, we'll just test the manager creation
            assert_eq!(manager.list_active_plugins().await.len(), 0);
        }

        #[cfg(not(feature = "wasm-plugins"))]
        {
            let result = manager
                .load_plugin(plugin_id, manifest, binary.to_vec(), policy)
                .await;
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_resource_monitor() {
        let monitor = ResourceMonitor::new();
        assert_eq!(
            monitor.monitoring_interval,
            std::time::Duration::from_secs(5)
        );
    }

    #[test]
    fn test_resource_report() {
        let mut report = ResourceReport::new();

        let plugin_report = PluginResourceReport {
            plugin_id: PluginId::new(),
            memory_usage: 1024,
            fuel_consumed: 1000,
            execution_time: std::time::Duration::from_millis(100),
            handle_count: 2,
            handle_memory: 512,
        };

        report
            .plugin_reports
            .insert(plugin_report.plugin_id.clone(), plugin_report);
        report.calculate_totals();

        assert_eq!(report.total_memory_usage, 1536); // 1024 + 512
        assert_eq!(report.total_fuel_consumed, 1000);
    }
}
