//! Plugin lifecycle management for loading, unloading, and monitoring

use crate::plugins::{errors::*, types::*, registry::*, security::*, verification::*, data_plane::*};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Plugin lifecycle manager
#[derive(Debug, Clone)]
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
    ) -> Result<(), PluginError> {
        log::info!("Loading plugin: {}", plugin_id);
        
        // 1. Verify the plugin
        let verification_report = self.verifier
            .verify_plugin(&binary, &manifest, &security_policy)
            .await?;
        
        match verification_report.overall_status {
            crate::plugins::verification::VerificationStatus::Rejected(reason) => {
                return Err(PluginError::Verification(
                    crate::plugins::errors::VerificationError::StaticAnalysis(reason)
                ));
            }
            crate::plugins::verification::VerificationStatus::Warning(warning) => {
                log::warn!("Plugin {} loaded with warnings: {}", plugin_id, warning);
            }
            _ => {}
        }
        
        // 2. Create WASM runtime components
        #[cfg(feature = "wasm-plugins")]
        {
            let engine = wasmtime::Engine::new(&security_policy.configure_engine()?)?;
            let component = wasmtime::component::Component::new(&engine, &binary)?;
            let mut linker = wasmtime::component::Linker::new(&engine);
            
            // Configure host state
            let host_state = HostState {
                plugin_id: plugin_id.clone(),
                config: PluginConfig::default(),
                resource_limits: security_policy.resource_limits.clone(),
            };
            
            // Configure WASI
            let wasi_ctx = security_policy.configure_wasi_context()?.build();
            
            // Add WASI to linker
            wasmtime_wasi::add_to_linker_sync(&mut linker)?;
            
            // Add our custom host functions
            self.add_host_functions(&mut linker)?;
            
            // Create store with fuel and memory limits
            let mut store = wasmtime::Store::new(&engine, (host_state, wasi_ctx));
            store.set_fuel(security_policy.resource_limits.max_fuel)?;
            store.limiter(|_| &mut MemoryLimiter::new(security_policy.resource_limits.max_memory));
            
            // Instantiate the component
            let instance = linker.instantiate(&mut store, &component)?;
            
            // Create active plugin wrapper
            let active_plugin = ActivePlugin::new(
                plugin_id.clone(),
                manifest,
                engine,
                component,
                store,
                instance,
                AstHandleManager::new(),
            );
            
            // Store in active plugins
            self.active_plugins.write().await.insert(plugin_id.clone(), active_plugin);
            self.security_policies.insert(plugin_id.clone(), security_policy);
            
            log::info!("Successfully loaded plugin: {}", plugin_id);
        }
        
        #[cfg(not(feature = "wasm-plugins"))]
        {
            return Err(PluginError::Unsupported("WASM plugins not enabled".to_string()));
        }
        
        Ok(())
    }
    
    /// Unload a plugin and cleanup resources
    pub async fn unload_plugin(&mut self, plugin_id: &PluginId) -> Result<(), PluginError> {
        log::info!("Unloading plugin: {}", plugin_id);
        
        let mut active_plugins = self.active_plugins.write().await;
        if let Some(mut plugin) = active_plugins.remove(plugin_id) {
            // Call plugin cleanup
            plugin.cleanup().await?;
        }
        
        self.security_policies.remove(plugin_id);
        
        log::info!("Successfully unloaded plugin: {}", plugin_id);
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
        active_plugins.get(plugin_id).map(|plugin| plugin.stats.clone())
    }
    
    /// Monitor resource usage of all active plugins
    pub async fn monitor_resources(&mut self) -> Result<ResourceReport, PluginError> {
        let active_plugins = self.active_plugins.read().await;
        let mut report = ResourceReport::new();
        
        for (plugin_id, plugin) in active_plugins.iter() {
            let plugin_report = self.resource_monitor.monitor_plugin(plugin).await?;
            report.plugin_reports.insert(plugin_id.clone(), plugin_report);
        }
        
        Ok(report)
    }
    
    /// Add host functions to the linker
    #[cfg(feature = "wasm-plugins")]
    fn add_host_functions(
        &self,
        linker: &mut wasmtime::component::Linker<(HostState, wasmtime_wasi::WasiCtx)>,
    ) -> Result<(), PluginError> {
        // Add logging function
        linker.func_wrap(
            "logging",
            "log",
            |_caller: wasmtime::Caller<'_, (HostState, wasmtime_wasi::WasiCtx)>, 
             level: String, 
             message: String| {
                match level.as_str() {
                    "error" => log::error!("[Plugin] {}", message),
                    "warn" => log::warn!("[Plugin] {}", message),
                    "info" => log::info!("[Plugin] {}", message),
                    "debug" => log::debug!("[Plugin] {}", message),
                    _ => log::trace!("[Plugin] {}", message),
                }
            },
        )?;
        
        // Add config function
        linker.func_wrap(
            "config",
            "get-value",
            |caller: wasmtime::Caller<'_, (HostState, wasmtime_wasi::WasiCtx)>, 
             key: String| -> Option<String> {
                let (host_state, _) = caller.data();
                host_state.config.custom_settings.get(&key).cloned()
            },
        )?;
        
        linker.func_wrap(
            "config",
            "get-language",
            |_caller: wasmtime::Caller<'_, (HostState, wasmtime_wasi::WasiCtx)>| -> String {
                "rust".to_string() // This would be dynamic in real implementation
            },
        )?;
        
        Ok(())
    }
}

impl Default for PluginLifecycleManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Active plugin wrapper containing runtime state
#[derive(Debug)]
pub struct ActivePlugin {
    pub id: PluginId,
    pub manifest: PluginManifest,
    pub stats: PluginStats,
    pub status: PluginStatus,
    #[cfg(feature = "wasm-plugins")]
    engine: wasmtime::Engine,
    #[cfg(feature = "wasm-plugins")]
    component: wasmtime::component::Component,
    #[cfg(feature = "wasm-plugins")]
    store: std::sync::Arc<std::sync::Mutex<wasmtime::Store<(HostState, wasmtime_wasi::WasiCtx)>>>,
    #[cfg(feature = "wasm-plugins")]
    instance: wasmtime::component::Instance,
    ast_handles: AstHandleManager,
}

impl ActivePlugin {
    #[cfg(feature = "wasm-plugins")]
    pub fn new(
        id: PluginId,
        manifest: PluginManifest,
        engine: wasmtime::Engine,
        component: wasmtime::component::Component,
        store: wasmtime::Store<(HostState, wasmtime_wasi::WasiCtx)>,
        instance: wasmtime::component::Instance,
        ast_handles: AstHandleManager,
    ) -> Self {
        Self {
            id,
            manifest,
            stats: PluginStats::default(),
            status: PluginStatus::Ready,
            engine,
            component,
            store,
            instance,
            ast_handles,
        }
    }
    
    /// Call plugin cleanup function
    pub async fn cleanup(&mut self) -> Result<(), PluginError> {
        #[cfg(feature = "wasm-plugins")]
        {
            // Call the cleanup function if it exists
            if let Ok(cleanup_func) = self.instance.get_typed_func::<(), ()>(&mut self.store, "cleanup") {
                cleanup_func.call(&mut self.store, ())?;
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
    pub async fn monitor_plugin(&self, plugin: &ActivePlugin) -> Result<PluginResourceReport, PluginError> {
        let mut report = PluginResourceReport {
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
        self.total_memory_usage = self.plugin_reports.values()
            .map(|report| report.memory_usage + report.handle_memory)
            .sum();
        
        self.total_fuel_consumed = self.plugin_reports.values()
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
    fn memory_growing(&mut self, current: usize, desired: usize, maximum: Option<usize>) -> bool {
        let desired_bytes = desired as u64 * 65536; // WASM page size
        desired_bytes <= self.max_memory
    }
    
    fn table_growing(&mut self, current: u32, desired: u32, maximum: Option<u32>) -> bool {
        // Allow table growth for now
        true
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
            let result = manager.load_plugin(plugin_id, manifest, binary.to_vec(), policy).await;
            assert!(result.is_err());
        }
    }
    
    #[test]
    fn test_resource_monitor() {
        let monitor = ResourceMonitor::new();
        assert_eq!(monitor.monitoring_interval, std::time::Duration::from_secs(5));
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
        
        report.plugin_reports.insert(plugin_report.plugin_id.clone(), plugin_report);
        report.calculate_totals();
        
        assert_eq!(report.total_memory_usage, 1536); // 1024 + 512
        assert_eq!(report.total_fuel_consumed, 1000);
    }
}