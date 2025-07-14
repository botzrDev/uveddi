//! Type definitions for the WASM plugin system

use crate::plugins::SecurityPolicy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
#[cfg(feature = "wasm-plugins")]
use wasmtime_wasi::TrappableError;

/// Unique identifier for a plugin instance
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PluginId(pub Uuid);

impl PluginId {
    /// Create a new PluginId with a random UUID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    /// Create a PluginId from a name (UUID v5).
    pub fn from_name(name: &str) -> Self {
        Self(Uuid::new_v5(&Uuid::NAMESPACE_DNS, name.as_bytes()))
    }
    /// Get the inner Uuid reference.
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl From<Uuid> for PluginId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<PluginId> for Uuid {
    fn from(id: PluginId) -> Self {
        id.0
    }
}

impl std::fmt::Display for PluginId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Plugin configuration passed to plugins at initialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub severity_threshold: f32,
    pub max_issues_per_file: u32,
    pub custom_settings: HashMap<String, String>,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            severity_threshold: 0.5,
            max_issues_per_file: 100,
            custom_settings: HashMap::new(),
        }
    }
}

/// Host state passed to WASM instances
#[derive(Debug)]
pub struct HostState {
    pub plugin_id: PluginId,
    pub config: PluginConfig,
    pub resource_limits: ResourceLimits,
    pub security_policy: SecurityPolicy, // Add this field
}

/// Wrapper type for WASI context to avoid orphan rule issues
#[cfg(feature = "wasm-plugins")]
pub struct HostContext {
    pub host_state: HostState,
    pub wasi_ctx: wasmtime_wasi::WasiCtx,
    pub resource_table: wasmtime_wasi::ResourceTable, // Added for WASI Host trait
}

#[cfg(feature = "wasm-plugins")]
impl wasmtime_wasi::WasiView for HostContext {
    fn ctx(&mut self) -> &mut wasmtime_wasi::WasiCtx {
        &mut self.wasi_ctx
    }
    fn table(&mut self) -> &mut wasmtime_wasi::ResourceTable {
        &mut self.resource_table
    }
}

// Implement the required Host traits for HostContext
#[cfg(feature = "wasm-plugins")]
impl wasmtime_wasi::bindings::cli::environment::Host for HostContext {
    fn get_environment(&mut self) -> Result<Vec<(String, String)>, anyhow::Error> {
        // NOTE: UV-108 - Security policy controls environment variable access
        let mut env_vars = Vec::new();
        // NOTE: UV-108 - Use security_policy.permissions, not resource_limits.permissions
        for permission in &self.host_state.security_policy.permissions {
            if let crate::plugins::security::Permission::EnvRead(var) = permission {
                if var == "*" {
                    env_vars.extend(std::env::vars());
                    break;
                } else if let Ok(value) = std::env::var(var) {
                    env_vars.push((var.clone(), value));
                }
            }
        }
        Ok(env_vars)
    }
    fn get_arguments(&mut self) -> Result<Vec<String>, anyhow::Error> {
        // NOTE: UV-108 - Use custom_settings for arguments if present
        if let Some(args) = self.host_state.config.custom_settings.get("arguments") {
            Ok(args.split_whitespace().map(|s| s.to_string()).collect())
        } else {
            Ok(Vec::new())
        }
    }
    fn initial_cwd(&mut self) -> Result<Option<String>, anyhow::Error> {
        Ok(Some("/tmp/plugin_workspace".to_string()))
    }
}

#[cfg(feature = "wasm-plugins")]
impl wasmtime_wasi::bindings::cli::exit::Host for HostContext {
    fn exit(&mut self, status: Result<(), ()>) -> Result<(), anyhow::Error> {
        match status {
            Ok(()) => log::info!("Plugin {} exited successfully", self.host_state.plugin_id),
            Err(()) => log::warn!("Plugin {} exited with error", self.host_state.plugin_id),
        }
        Ok(())
    }
}

// NOTE: UV-108 - Temporarily removing complex filesystem traits to focus on core functionality
// These would be implemented for full WASI filesystem support
// #[cfg(feature = "wasm-plugins")]
// impl wasmtime_wasi::bindings::filesystem::types::Host for HostContext { ... }

// NOTE: UV-108 - Temporarily removing complex filesystem traits to focus on core functionality
// These would be implemented for full WASI filesystem support
// #[cfg(feature = "wasm-plugins")]
// impl wasmtime_wasi::bindings::filesystem::types::HostDirectoryEntryStream for HostContext { ... }

// NOTE: UV-108 - Temporarily removing complex filesystem traits to focus on core functionality
// These would be implemented for full WASI filesystem support
// #[cfg(feature = "wasm-plugins")]
// impl wasmtime_wasi::bindings::filesystem::types::HostDescriptor for HostContext { ... }

/// Resource limits enforced on plugin execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory in bytes (default: 256MB)
    pub max_memory: u64,
    /// Maximum fuel (instruction count) per invocation
    pub max_fuel: u64,
    /// Maximum execution time in milliseconds
    pub max_execution_time_ms: u64,
    /// Maximum number of file handles
    pub max_file_handles: u32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory: 256 * 1024 * 1024, // 256MB
            max_fuel: 10_000_000,          // 10M instructions
            max_execution_time_ms: 30_000, // 30 seconds
            max_file_handles: 10,
        }
    }
}

/// Status of a plugin instance
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PluginStatus {
    /// Plugin is loading
    Loading,
    /// Plugin is ready for use
    Ready,
    /// Plugin encountered an error
    Error(String),
    /// Plugin is unloading
    Unloading,
    /// Plugin has been unloaded
    Unloaded,
}

/// Plugin execution statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PluginStats {
    /// Total number of invocations
    pub invocations: u64,
    /// Total execution time in milliseconds
    pub total_execution_time_ms: u64,
    /// Average execution time in milliseconds
    pub avg_execution_time_ms: f64,
    /// Total fuel consumed
    pub total_fuel_consumed: u64,
    /// Peak memory usage in bytes
    pub peak_memory_usage: u64,
    /// Number of errors encountered
    pub error_count: u64,
    /// Last error message
    pub last_error: Option<String>,
}

impl PluginStats {
    pub fn record_execution(
        &mut self,
        execution_time_ms: u64,
        fuel_consumed: u64,
        memory_usage: u64,
    ) {
        self.invocations += 1;
        self.total_execution_time_ms += execution_time_ms;
        self.avg_execution_time_ms = self.total_execution_time_ms as f64 / self.invocations as f64;
        self.total_fuel_consumed += fuel_consumed;
        self.peak_memory_usage = self.peak_memory_usage.max(memory_usage);
    }

    pub fn record_error(&mut self, error: String) {
        self.error_count += 1;
        self.last_error = Some(error);
    }
}

/// Handle to AST data stored in plugin memory
#[derive(Debug, Clone)]
pub struct AstHandle {
    pub id: u32,
    pub size: u64,
    pub language: String,
}

/// Plugin execution context
#[derive(Debug)]
pub struct ExecutionContext {
    pub start_time: std::time::Instant,
    pub fuel_limit: u64,
    pub memory_limit: u64,
}

impl ExecutionContext {
    pub fn new(limits: &ResourceLimits) -> Self {
        Self {
            start_time: std::time::Instant::now(),
            fuel_limit: limits.max_fuel,
            memory_limit: limits.max_memory,
        }
    }

    pub fn elapsed_ms(&self) -> u64 {
        self.start_time.elapsed().as_millis() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_plugin_id_generation() {
        let id1 = PluginId::new();
        let id2 = PluginId::new();
        assert_ne!(id1, id2);
    }
    #[test]
    fn test_plugin_id_conversions() {
        let uuid = Uuid::new_v4();
        let id = PluginId::from(uuid);
        assert_eq!(Uuid::from(id.clone()), uuid);
        assert_eq!(id.as_uuid(), &uuid);
    }
    #[test]
    fn test_plugin_status_serialization() {
        let status = PluginStatus::Ready;
        let serialized = serde_json::to_string(&status).unwrap();
        let deserialized: PluginStatus = serde_json::from_str(&serialized).unwrap();
        assert_eq!(status, deserialized);
    }
}
