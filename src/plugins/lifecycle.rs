//! Plugin lifecycle management for loading, unloading, and monitoring

use crate::plugins::{
    data_plane::*,
    errors::*,
    registry::*,
    security::*,
    types::{HostState, PluginConfig, PluginId, PluginStats, PluginStatus},
    verification::*,
};

#[cfg(feature = "wasm-plugins")]
use crate::plugins::types::HostContext;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
#[cfg(feature = "wasm-plugins")]
use wasmtime::component::{Component, Linker};
#[cfg(feature = "wasm-plugins")]
use wasmtime::{Engine, Store};
#[cfg(feature = "wasm-plugins")]
use wasmtime_wasi::preview1::{self, WasiP1Ctx};
#[cfg(feature = "wasm-plugins")]
use wasmtime_wasi::{WasiCtxBuilder, WasiView};
#[cfg(feature = "wasm-plugins")]
use crate::plugins::wasm::CoreAnalysis;

/// Result type for plugin analysis
#[derive(Debug, Clone)]
pub struct PluginAnalysisResult {
    pub issues: Vec<PluginIssueResult>,
    pub metrics: PluginMetrics,
    pub duration_ms: u32,
}

/// Issue detected by a plugin
#[derive(Debug, Clone)]
pub struct PluginIssueResult {
    pub id: String,
    pub severity: String,
    pub category: String,
    pub message: String,
    pub description: Option<String>,
    pub file: String,
    pub start_line: u32,
    pub start_column: u32,
    pub end_line: u32,
    pub end_column: u32,
    pub suggestion: Option<String>,
}

/// Metrics from plugin analysis
#[derive(Debug, Clone, Default)]
pub struct PluginMetrics {
    pub lines_of_code: u32,
    pub complexity: u32,
    pub maintainability_index: f64,
}

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
            let component = Component::new(&engine, &binary)?;
            let mut linker = Linker::<HostContext>::new(&engine);

            // Configure host state
            let host_state = HostState {
                plugin_id: plugin_id.clone(),
                config: PluginConfig::default(),
                resource_limits: security_policy.resource_limits.clone(),
                security_policy: security_policy.clone(),
            };

            // Configure WASI context
            let wasi_ctx = security_policy.configure_wasi_context()?.build_p1();

            // Add our custom host functions to the linker
            // Note: WASI Preview 1 uses core modules, not Component Model, so we skip it
            // and rely on our host functions providing necessary functionality
            crate::plugins::wasm::CoreAnalysis::add_to_linker::<_, wasmtime::component::HasSelf<HostContext>>(&mut linker, |host| host)?;

            // Create store with fuel and memory limits
            let resource_table = wasmtime::component::ResourceTable::new();

            // Create database and analysis engine (placeholders for now)
            // TODO: Pass real dependencies to load_plugin via PluginLifecycleManager fields
            let database_config = crate::database::DatabaseConfig::default();
            let database = crate::database::ScalableDatabase::new(database_config).await
                .map_err(|e| PluginError::Loading(format!("Failed to create database: {}", e)))?;
            let analysis_engine = crate::analysis::AnalysisEngine::new()
                .map_err(|e| PluginError::Loading(format!("Failed to create analysis engine: {}", e)))?;

            let host_context = HostContext {
                host_state,
                wasi_ctx,
                table: resource_table,
                database: Arc::new(database),
                analysis_engine: Arc::new(RwLock::new(analysis_engine)),
                config_store: Arc::new(RwLock::new(HashMap::new())),
                ast_cache: Arc::new(RwLock::new(HashMap::new())),
            };

            let mut store = wasmtime::Store::new(&engine, host_context);
            store.set_fuel(security_policy.resource_limits.max_fuel)?;

            // Instantiate the component
            let bindings = CoreAnalysis::instantiate(&mut store, &component, &linker)?;

            // Create active plugin wrapper
            let active_plugin = ActivePlugin::new(
                plugin_id.clone(),
                manifest,
                engine,
                component,
                store,
                bindings,
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
        _linker: &mut wasmtime::component::Linker<HostContext>,
    ) -> crate::error::Result<()> {
        // NOTE: Host functions are now added via CoreAnalysis::add_to_linker in load_plugin
        Ok(())
    }

    /// Analyze a file using a specific plugin
    ///
    /// This method invokes the plugin's analyze function with the given file data
    /// and returns the analysis results.
    #[cfg(feature = "wasm-plugins")]
    pub async fn analyze_with_plugin(
        &self,
        plugin_id: &PluginId,
        file_path: &str,
        file_content: &str,
        language: &str,
    ) -> Result<PluginAnalysisResult, PluginError> {
        let active_plugins = self.active_plugins.read().await;
        let plugin = active_plugins
            .get(plugin_id)
            .ok_or_else(|| PluginError::NotFound(plugin_id.to_string()))?;

        plugin.analyze_file(file_path, file_content, language)
    }
}

impl Default for PluginLifecycleManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Active plugin wrapper containing runtime state
// Debug and Clone traits removed due to Wasmtime types not implementing them
pub struct ActivePlugin {
    /// Unique identifier for the plugin
    pub id: PluginId,
    /// Plugin manifest with metadata
    pub manifest: PluginManifest,
    /// Runtime statistics for the plugin
    pub stats: PluginStats,
    /// Current status of the plugin
    pub status: PluginStatus,
    #[cfg(feature = "wasm-plugins")]
    engine: wasmtime::Engine,
    #[cfg(feature = "wasm-plugins")]
    component: wasmtime::component::Component,
    #[cfg(feature = "wasm-plugins")]
    store: std::sync::Arc<std::sync::Mutex<wasmtime::Store<HostContext>>>,
    #[cfg(feature = "wasm-plugins")]
    bindings: CoreAnalysis,
    ast_handles: AstHandleManager,
}

impl ActivePlugin {
    #[cfg(feature = "wasm-plugins")]
    pub fn new(
        id: PluginId,
        manifest: PluginManifest,
        engine: wasmtime::Engine,
        component: wasmtime::component::Component,
        store: wasmtime::Store<HostContext>,
        bindings: CoreAnalysis,
        ast_handles: AstHandleManager,
    ) -> Self {
        Self {
            id,
            manifest,
            stats: PluginStats::default(),
            status: PluginStatus::Ready,
            engine,
            component,
            store: Arc::new(std::sync::Mutex::new(store)),
            bindings,
            ast_handles,
        }
    }

    /// Call plugin cleanup function
    pub async fn cleanup(&mut self) -> crate::error::Result<()> {
        #[cfg(feature = "wasm-plugins")]
        {
            // Call the cleanup function if it exists
            if let Ok(mut store_guard) = self.store.lock() {
                // Ignore result of cleanup for now
                let _ = self.bindings.call_cleanup(&mut *store_guard);
            }
        }

        self.status = PluginStatus::Unloaded;
        Ok(())
    }

    /// Analyze a file using this plugin
    ///
    /// This invokes the plugin's `analyze` export function with the given file data.
    /// The file content and metadata are passed as JSON, and results are returned as JSON.
    #[cfg(feature = "wasm-plugins")]
    pub fn analyze_file(
        &self,
        file_path: &str,
        file_content: &str,
        language: &str,
    ) -> Result<PluginAnalysisResult, PluginError> {
        use std::time::Instant;
        let start_time = Instant::now();

        // Get mutable access to the store
        let mut store_guard = self.store.lock().map_err(|e| {
            PluginError::Execution(format!("Failed to acquire store lock: {}", e))
        })?;

        // Prepare input
        // Use a simple hash calculation
        let content_hash = {
            let mut hash: u64 = 0;
            for byte in file_content.bytes() {
                hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
            }
            format!("{:016x}", hash)
        };

        // Construct source file record
        let source_file = crate::plugins::wasm::SourceFile {
            path: file_path.to_string(),
            content: file_content.to_string(),
            language: language.to_string(),
            size: file_content.len() as u32,
            hash: content_hash,
            ast: None, // We don't provide pre-parsed AST yet
        };

        // Call the component function
        let result = self.bindings.call_analyze(&mut *store_guard, &source_file)
            .map_err(|e| PluginError::Execution(format!("Component analyze call failed: {}", e)))?;
            
        // Handle the Result<AnalysisResult, String> returned by the plugin
        match result {
            Ok(analysis) => self.convert_analysis_result(analysis, start_time.elapsed().as_millis() as u32),
            Err(e) => Err(PluginError::Execution(format!("Plugin analysis reported error: {}", e))),
        }
    }

    #[cfg(feature = "wasm-plugins")]
    fn convert_analysis_result(
        &self, 
        analysis: crate::plugins::wasm::AnalysisResult,
        duration_ms: u32
    ) -> Result<PluginAnalysisResult, PluginError> {
        let issues = analysis.issues.into_iter().map(|issue| {
            PluginIssueResult {
                id: issue.id,
                severity: format!("{:?}", issue.severity),
                category: format!("{:?}", issue.category),
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

        Ok(PluginAnalysisResult {
            issues,
            metrics: PluginMetrics {
                lines_of_code: analysis.metrics.lines_of_code,
                complexity: analysis.metrics.complexity,
                maintainability_index: analysis.metrics.maintainability_index,
            },
            duration_ms,
        })
    }

    /// Parse analysis result from JSON
    #[cfg(feature = "wasm-plugins")]
    fn parse_analysis_result(&self, json_str: &str, duration_ms: u32) -> Result<PluginAnalysisResult, PluginError> {
        let value: serde_json::Value = serde_json::from_str(json_str)
            .map_err(|e| PluginError::Execution(format!("Result JSON parse failed: {}", e)))?;

        let mut issues = Vec::new();
        if let Some(issues_array) = value.get("issues").and_then(|v| v.as_array()) {
            for issue_val in issues_array {
                let issue = PluginIssueResult {
                    id: issue_val.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    severity: issue_val.get("severity").and_then(|v| v.as_str()).unwrap_or("medium").to_string(),
                    category: issue_val.get("category").and_then(|v| v.as_str()).unwrap_or("quality").to_string(),
                    message: issue_val.get("message").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    description: issue_val.get("description").and_then(|v| v.as_str()).map(|s| s.to_string()),
                    file: issue_val.get("file").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    start_line: issue_val.get("span").and_then(|v| v.get("start")).and_then(|v| v.get("line")).and_then(|v| v.as_u64()).unwrap_or(1) as u32,
                    start_column: issue_val.get("span").and_then(|v| v.get("start")).and_then(|v| v.get("column")).and_then(|v| v.as_u64()).unwrap_or(1) as u32,
                    end_line: issue_val.get("span").and_then(|v| v.get("end")).and_then(|v| v.get("line")).and_then(|v| v.as_u64()).unwrap_or(1) as u32,
                    end_column: issue_val.get("span").and_then(|v| v.get("end")).and_then(|v| v.get("column")).and_then(|v| v.as_u64()).unwrap_or(1) as u32,
                    suggestion: issue_val.get("suggestion").and_then(|v| v.as_str()).map(|s| s.to_string()),
                };
                issues.push(issue);
            }
        }

        let metrics = if let Some(metrics_val) = value.get("metrics") {
            PluginMetrics {
                lines_of_code: metrics_val.get("lines_of_code").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                complexity: metrics_val.get("complexity").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                maintainability_index: metrics_val.get("maintainability_index").and_then(|v| v.as_f64()).unwrap_or(100.0),
            }
        } else {
            PluginMetrics::default()
        };

        Ok(PluginAnalysisResult {
            issues,
            metrics,
            duration_ms,
        })
    }
}

/// Read-only reference to an active plugin
#[derive(Debug, Clone)]
pub struct ActivePluginRef {
    /// Unique identifier for the plugin
    pub id: PluginId,
    /// Plugin manifest with metadata
    pub manifest: PluginManifest,
    /// Runtime statistics for the plugin
    pub stats: PluginStats,
    /// Current status of the plugin
    pub status: PluginStatus,
}

/// Resource monitor for tracking plugin resource usage
#[derive(Debug, Clone)]
pub struct ResourceMonitor {
    /// Interval between resource checks
    monitoring_interval: std::time::Duration,
}

impl ResourceMonitor {
    /// Create a new resource monitor with default settings
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
    /// Individual resource reports for each plugin
    pub plugin_reports: HashMap<PluginId, PluginResourceReport>,
    /// Total memory usage across all plugins
    pub total_memory_usage: u64,
    /// Total fuel consumed across all plugins
    pub total_fuel_consumed: u64,
}

impl ResourceReport {
    /// Create a new empty resource report
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
    /// Unique identifier of the plugin
    pub plugin_id: PluginId,
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// Fuel consumed during execution
    pub fuel_consumed: u64,
    /// Total execution time
    pub execution_time: std::time::Duration,
    /// Number of active handles
    pub handle_count: usize,
    /// Memory used by handles
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
