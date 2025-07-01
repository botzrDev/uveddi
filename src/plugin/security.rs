use wasmtime_wasi::WasiCtxBuilder;
use serde::{Deserialize, Serialize};
use crate::error::UveddiError;

#[derive(Debug, Deserialize, Serialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub permissions: Vec<Permission>,
    pub resource_limits: ResourceLimits,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Permission {
    #[serde(rename = "fs:read")]
    FilesystemRead { path: String },
    #[serde(rename = "net:connect")]
    NetworkConnect { host: String },
    #[serde(rename = "env:read")]
    EnvironmentRead { var: String },
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ResourceLimits {
    pub max_memory_mb: u32,
    pub max_execution_fuel: u64,
    pub max_output_size_kb: u32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: 16,           // 16MB memory limit
            max_execution_fuel: 1_000_000, // ~1M instructions
            max_output_size_kb: 16,      // 16KB output limit
        }
    }
}

pub fn create_secure_wasi_context(manifest: &PluginManifest) -> Result<wasmtime_wasi::WasiCtx, UveddiError> {
    let mut builder = WasiCtxBuilder::new();
    
    // Always inherit stdio for logging
    builder.inherit_stdio();
    
    // Grant permissions based on manifest
    for permission in &manifest.permissions {
        match permission {
            Permission::FilesystemRead { path } => {
                // For security, we only allow reading from specific allowed directories
                if is_safe_path(path) {
                    log::info!("Granting filesystem read access to: {}", path);
                    // In a full implementation, we would use cap-std here
                    // For now, we just log the permission grant
                } else {
                    log::warn!("Denying unsafe filesystem access to: {}", path);
                    return Err(UveddiError::PluginError(
                        format!("Unsafe filesystem access requested: {}", path)
                    ));
                }
            }
            Permission::NetworkConnect { host } => {
                // Network permissions are not implemented yet for security
                log::warn!("Network permissions not yet implemented for host: {}", host);
                return Err(UveddiError::PluginError(
                    "Network permissions not yet supported".to_string()
                ));
            }
            Permission::EnvironmentRead { var } => {
                // Only allow reading specific safe environment variables
                if is_safe_env_var(var) {
                    if let Ok(value) = std::env::var(var) {
                        log::info!("Granting environment variable access: {}", var);
                        builder = builder.env(var, &value)?;
                    }
                } else {
                    log::warn!("Denying access to sensitive environment variable: {}", var);
                    return Err(UveddiError::PluginError(
                        format!("Access to sensitive environment variable denied: {}", var)
                    ));
                }
            }
        }
    }
    
    Ok(builder.build())
}

/// Check if a file path is safe for plugin access
fn is_safe_path(path: &str) -> bool {
    let safe_paths = [
        "/tmp/",
        "./plugins/",
        "./temp/",
        "./cache/",
    ];
    
    // Don't allow access to system directories or parent directory traversal
    if path.contains("..") || path.starts_with('/') && !safe_paths.iter().any(|safe| path.starts_with(safe)) {
        return false;
    }
    
    // Only allow relative paths within safe directories
    safe_paths.iter().any(|safe| path.starts_with(safe))
}

/// Check if an environment variable is safe to expose to plugins
fn is_safe_env_var(var: &str) -> bool {
    let safe_vars = [
        "UVEDDI_PLUGIN_CONFIG",
        "UVEDDI_DEBUG_LEVEL",
        "TEMP",
        "TMP",
    ];
    
    // Block sensitive variables
    let blocked_vars = [
        "HOME",
        "PATH", 
        "USER",
        "USERNAME",
        "OPENAI_API_KEY",
        "DATABASE_URL",
        "JWT_SECRET",
    ];
    
    if blocked_vars.iter().any(|blocked| var == *blocked) {
        return false;
    }
    
    safe_vars.iter().any(|safe| var == *safe)
}

pub fn validate_resource_limits(limits: &ResourceLimits) -> Result<(), UveddiError> {
    // Enforce maximum limits for security
    const MAX_MEMORY_MB: u32 = 256;
    const MAX_FUEL: u64 = 10_000_000;
    const MAX_OUTPUT_KB: u32 = 1024;
    
    if limits.max_memory_mb > MAX_MEMORY_MB {
        return Err(UveddiError::PluginError(
            format!("Memory limit {} MB exceeds maximum allowed {} MB", 
                   limits.max_memory_mb, MAX_MEMORY_MB)
        ));
    }
    
    if limits.max_execution_fuel > MAX_FUEL {
        return Err(UveddiError::PluginError(
            format!("Execution fuel {} exceeds maximum allowed {}", 
                   limits.max_execution_fuel, MAX_FUEL)
        ));
    }
    
    if limits.max_output_size_kb > MAX_OUTPUT_KB {
        return Err(UveddiError::PluginError(
            format!("Output size {} KB exceeds maximum allowed {} KB", 
                   limits.max_output_size_kb, MAX_OUTPUT_KB)
        ));
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_safe_path_validation() {
        assert!(is_safe_path("./plugins/test.wasm"));
        assert!(is_safe_path("/tmp/plugin_cache"));
        assert!(!is_safe_path("../../../etc/passwd"));
        assert!(!is_safe_path("/home/user/secrets"));
        assert!(!is_safe_path("./plugins/../../../sensitive"));
    }
    
    #[test]
    fn test_safe_env_var_validation() {
        assert!(is_safe_env_var("UVEDDI_PLUGIN_CONFIG"));
        assert!(is_safe_env_var("TEMP"));
        assert!(!is_safe_env_var("OPENAI_API_KEY"));
        assert!(!is_safe_env_var("HOME"));
        assert!(!is_safe_env_var("DATABASE_URL"));
    }
    
    #[test]
    fn test_resource_limits_validation() {
        let valid_limits = ResourceLimits::default();
        assert!(validate_resource_limits(&valid_limits).is_ok());
        
        let invalid_limits = ResourceLimits {
            max_memory_mb: 512, // Too high
            max_execution_fuel: 1_000_000,
            max_output_size_kb: 16,
        };
        assert!(validate_resource_limits(&invalid_limits).is_err());
    }
    
    #[test]
    fn test_manifest_deserialization() {
        let manifest_toml = r#"
name = "test-plugin"
version = "1.0.0"

[[permissions]]
type = "fs:read"
path = "./plugins/"

[[permissions]]
type = "env:read"
var = "UVEDDI_PLUGIN_CONFIG"

[resource_limits]
max_memory_mb = 32
max_execution_fuel = 2000000
max_output_size_kb = 64
"#;
        
        let manifest: Result<PluginManifest, _> = toml::from_str(manifest_toml);
        assert!(manifest.is_ok());
        
        let manifest = manifest.unwrap();
        assert_eq!(manifest.name, "test-plugin");
        assert_eq!(manifest.permissions.len(), 2);
        assert_eq!(manifest.resource_limits.max_memory_mb, 32);
    }
}