//! Security framework for WASM plugins using capability-based security

use crate::plugins::{
    errors::*,
    types::ResourceLimits,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;
#[cfg(feature = "wasm-plugins")]
use wasmtime_wasi::p2::WasiCtxBuilder;

/// Security policy for plugin execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    /// Resource limits for plugin execution
    pub resource_limits: ResourceLimits,
    /// Allowed permissions for the plugin
    pub permissions: HashSet<Permission>,
    /// Trusted publisher keys
    pub trusted_publishers: HashSet<String>,
    /// Whether to enforce code signing
    pub require_code_signing: bool,
    /// Whether to perform static analysis
    pub require_static_analysis: bool,
    /// Maximum plugin binary size in bytes
    pub max_binary_size: u64,
}

impl Default for SecurityPolicy {
    fn default() -> Self {
        Self {
            resource_limits: ResourceLimits::default(),
            permissions: HashSet::new(),
            trusted_publishers: HashSet::new(),
            require_code_signing: false, // Disabled by default for development
            require_static_analysis: true,
            max_binary_size: 50 * 1024 * 1024, // 50MB
        }
    }
}

/// Capability-based permissions for plugins
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    /// Read access to specific directory
    FileRead(PathBuf),
    /// Write access to specific directory  
    FileWrite(PathBuf),
    /// Network access to specific host:port
    NetworkConnect(String),
    /// Environment variable access
    EnvRead(String),
    /// Logging permission
    Logging,
    /// Configuration access
    ConfigRead,
    /// Temporary file creation
    TempFileCreate,
}

impl Permission {
    /// Check if this permission allows the requested operation
    pub fn allows(&self, requested: &Permission) -> bool {
        match (self, requested) {
            (Permission::FileRead(allowed), Permission::FileRead(requested)) => {
                requested.starts_with(allowed)
            }
            (Permission::FileWrite(allowed), Permission::FileWrite(requested)) => {
                requested.starts_with(allowed)
            }
            (Permission::NetworkConnect(allowed), Permission::NetworkConnect(requested)) => {
                allowed == requested || allowed == "*"
            }
            (Permission::EnvRead(allowed), Permission::EnvRead(requested)) => {
                allowed == requested || allowed == "*"
            }
            (a, b) => a == b,
        }
    }
}

impl SecurityPolicy {
    /// Create a new security policy
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a restrictive security policy for untrusted plugins
    pub fn restrictive() -> Self {
        Self {
            resource_limits: ResourceLimits {
                max_memory: 128 * 1024 * 1024, // 128MB
                max_fuel: 5_000_000,           // 5M instructions
                max_execution_time_ms: 10_000, // 10 seconds
                max_file_handles: 5,
            },
            permissions: [Permission::Logging, Permission::ConfigRead].into(),
            trusted_publishers: HashSet::new(),
            require_code_signing: true,
            require_static_analysis: true,
            max_binary_size: 10 * 1024 * 1024, // 10MB
        }
    }

    /// Create a permissive security policy for trusted plugins
    pub fn permissive() -> Self {
        Self {
            resource_limits: ResourceLimits {
                max_memory: 512 * 1024 * 1024, // 512MB
                max_fuel: 50_000_000,          // 50M instructions
                max_execution_time_ms: 60_000, // 60 seconds
                max_file_handles: 20,
            },
            permissions: [
                Permission::Logging,
                Permission::ConfigRead,
                Permission::TempFileCreate,
                Permission::EnvRead("*".to_string()),
            ]
            .into(),
            trusted_publishers: HashSet::new(),
            require_code_signing: false,
            require_static_analysis: true,
            max_binary_size: 100 * 1024 * 1024, // 100MB
        }
    }

    /// Add a permission to the policy
    pub fn add_permission(&mut self, permission: Permission) {
        self.permissions.insert(permission);
    }

    /// Remove a permission from the policy
    pub fn remove_permission(&mut self, permission: &Permission) {
        self.permissions.remove(permission);
    }

    /// Check if a permission is allowed
    pub fn has_permission(&self, requested: &Permission) -> bool {
        self.permissions.iter().any(|p| p.allows(requested))
    }

    /// Add a trusted publisher key
    pub fn add_trusted_publisher(&mut self, key: String) {
        self.trusted_publishers.insert(key);
    }

    /// Check if a publisher is trusted
    pub fn is_trusted_publisher(&self, key: &str) -> bool {
        self.trusted_publishers.contains(key)
    }

    /// Configure WASI context based on this policy
    #[cfg(feature = "wasm-plugins")]
    pub fn configure_wasi_context(&self) -> Result<WasiCtxBuilder, PluginError> {
        let mut builder = WasiCtxBuilder::new();

        // Configure standard I/O based on permissions
        if self.has_permission(&Permission::Logging) {
            builder.inherit_stdout().inherit_stderr();
        }

        // Configure environment variables
        for permission in &self.permissions {
            if let Permission::EnvRead(var) = permission {
                if var == "*" {
                    builder.inherit_env();
                    break;
                } else {
                    if let Ok(value) = std::env::var(var) {
                        builder.env(var, &value);
                    }
                }
            }
        }

        // Configure file system access using new v34 API
        for permission in &self.permissions {
            match permission {
                Permission::FileRead(path) => {
                    builder
                        .preopened_dir(
                            path,
                            path.to_str().unwrap_or("/sandbox"),
                            wasmtime_wasi::DirPerms::READ,
                            wasmtime_wasi::FilePerms::READ,
                        )
                        .map_err(|e| PluginError::SecurityViolation(e.to_string()))?;
                }
                Permission::FileWrite(path) => {
                    builder
                        .preopened_dir(
                            path,
                            path.to_str().unwrap_or("/sandbox"),
                            wasmtime_wasi::DirPerms::all(),
                            wasmtime_wasi::FilePerms::all(),
                        )
                        .map_err(|e| PluginError::SecurityViolation(e.to_string()))?;
                }
                _ => {}
            }
        }

        Ok(builder)
    }

    #[cfg(not(feature = "wasm-plugins"))]
    pub fn configure_wasi_context(&self) -> Result<(), PluginError> {
        Err(PluginError::Unsupported(
            "WASM plugins not enabled".to_string(),
        ))
    }

    /// Configure Wasmtime engine based on this policy
    #[cfg(feature = "wasm-plugins")]
    pub fn configure_engine(&self) -> Result<wasmtime::Config, PluginError> {
        let mut config = wasmtime::Config::new();

        // Enable component model
        config.wasm_component_model(true);

        // Configure fuel for CPU limiting
        config.consume_fuel(true);

        // Configure memory limits
        config.max_wasm_stack(1024 * 1024); // 1MB stack

        // Disable features that could be security risks
        config.wasm_simd(false);
        config.wasm_threads(false);
        config.wasm_bulk_memory(true);
        config.wasm_reference_types(true);

        // Enable epoch interruption for timeouts
        config.epoch_interruption(true);

        Ok(config)
    }

    #[cfg(not(feature = "wasm-plugins"))]
    pub fn configure_engine(&self) -> Result<(), PluginError> {
        Err(PluginError::Unsupported(
            "WASM plugins not enabled".to_string(),
        ))
    }

    /// Validate that a plugin binary complies with this policy
    pub fn validate_binary(&self, binary: &[u8]) -> Result<(), PluginError> {
        // Check binary size
        if binary.len() as u64 > self.max_binary_size {
            return Err(PluginError::SecurityViolation(format!(
                "Binary size {} bytes exceeds limit of {} bytes",
                binary.len(),
                self.max_binary_size
            )));
        }

        // Basic WASM validation
        self.validate_wasm_binary(binary)?;

        Ok(())
    }

    /// Basic WASM binary validation
    fn validate_wasm_binary(&self, binary: &[u8]) -> Result<(), PluginError> {
        // Check WASM magic number
        if binary.len() < 4 || &binary[0..4] != b"\0asm" {
            return Err(PluginError::SecurityViolation(
                "Invalid WASM magic number".to_string(),
            ));
        }

        // Check WASM version
        if binary.len() < 8 || &binary[4..8] != &[1, 0, 0, 0] {
            return Err(PluginError::SecurityViolation(
                "Unsupported WASM version".to_string(),
            ));
        }

        Ok(())
    }
}

/// Runtime security monitor for plugin execution
#[derive(Debug)]
pub struct SecurityMonitor {
    policy: SecurityPolicy,
    start_time: std::time::Instant,
    fuel_consumed: u64,
    memory_peak: u64,
}

impl SecurityMonitor {
    pub fn new(policy: SecurityPolicy) -> Self {
        Self {
            policy,
            start_time: std::time::Instant::now(),
            fuel_consumed: 0,
            memory_peak: 0,
        }
    }

    /// Check if execution time limit is exceeded
    pub fn check_time_limit(&self) -> Result<(), PluginError> {
        let elapsed = self.start_time.elapsed().as_millis() as u64;
        if elapsed > self.policy.resource_limits.max_execution_time_ms {
            return Err(PluginError::ResourceLimit(format!(
                "Execution time limit exceeded: {}ms > {}ms",
                elapsed, self.policy.resource_limits.max_execution_time_ms
            )));
        }
        Ok(())
    }

    /// Record fuel consumption
    pub fn record_fuel(&mut self, fuel: u64) {
        self.fuel_consumed += fuel;
    }

    /// Check if fuel limit is exceeded
    pub fn check_fuel_limit(&self) -> Result<(), PluginError> {
        if self.fuel_consumed > self.policy.resource_limits.max_fuel {
            return Err(PluginError::ResourceLimit(format!(
                "Fuel limit exceeded: {} > {}",
                self.fuel_consumed, self.policy.resource_limits.max_fuel
            )));
        }
        Ok(())
    }

    /// Record memory usage
    pub fn record_memory(&mut self, memory: u64) {
        self.memory_peak = self.memory_peak.max(memory);
    }

    /// Check if memory limit is exceeded
    pub fn check_memory_limit(&self) -> Result<(), PluginError> {
        if self.memory_peak > self.policy.resource_limits.max_memory {
            return Err(PluginError::ResourceLimit(format!(
                "Memory limit exceeded: {} > {}",
                self.memory_peak, self.policy.resource_limits.max_memory
            )));
        }
        Ok(())
    }

    /// Get execution statistics
    pub fn get_stats(&self) -> SecurityStats {
        SecurityStats {
            execution_time_ms: self.start_time.elapsed().as_millis() as u64,
            fuel_consumed: self.fuel_consumed,
            memory_peak: self.memory_peak,
            policy_violations: 0, // Would be tracked in real implementation
        }
    }
}

/// Security execution statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityStats {
    pub execution_time_ms: u64,
    pub fuel_consumed: u64,
    pub memory_peak: u64,
    pub policy_violations: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_allows() {
        let read_src = Permission::FileRead(PathBuf::from("/src"));
        let read_src_subdir = Permission::FileRead(PathBuf::from("/src/main.rs"));
        let read_other = Permission::FileRead(PathBuf::from("/other"));

        assert!(read_src.allows(&read_src_subdir));
        assert!(!read_src.allows(&read_other));

        let env_all = Permission::EnvRead("*".to_string());
        let env_specific = Permission::EnvRead("PATH".to_string());

        assert!(env_all.allows(&env_specific));
        assert!(!env_specific.allows(&env_all));
    }

    #[test]
    fn test_security_policy() {
        let mut policy = SecurityPolicy::restrictive();

        assert!(!policy.has_permission(&Permission::FileRead(PathBuf::from("/tmp"))));

        policy.add_permission(Permission::FileRead(PathBuf::from("/tmp")));
        assert!(policy.has_permission(&Permission::FileRead(PathBuf::from("/tmp/file.txt"))));
    }

    #[test]
    fn test_wasm_validation() {
        let policy = SecurityPolicy::default();

        // Valid WASM binary (minimal)
        let valid_wasm = b"\0asm\x01\0\0\0";
        assert!(policy.validate_binary(valid_wasm).is_ok());

        // Invalid magic number
        let invalid_magic = b"invalid\x01\0\0\0";
        assert!(policy.validate_binary(invalid_magic).is_err());

        // Invalid version
        let invalid_version = b"\0asm\x02\0\0\0";
        assert!(policy.validate_binary(invalid_version).is_err());
    }

    #[test]
    fn test_security_monitor() {
        let policy = SecurityPolicy::restrictive();
        let mut monitor = SecurityMonitor::new(policy);

        monitor.record_fuel(1000);
        monitor.record_memory(1024 * 1024);

        assert!(monitor.check_fuel_limit().is_ok());
        assert!(monitor.check_memory_limit().is_ok());

        let stats = monitor.get_stats();
        assert_eq!(stats.fuel_consumed, 1000);
        assert_eq!(stats.memory_peak, 1024 * 1024);
    }
}
