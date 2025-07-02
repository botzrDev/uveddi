// Plugin security module - TEMPORARILY DISABLED
// This module is currently disabled to prevent WASM runtime crashes during development
// TODO: Re-enable once WASM plugin integration is stabilized

use crate::error::UveddiError;
use serde::{Deserialize, Serialize};

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
            max_memory_mb: 16,             // 16MB memory limit
            max_execution_fuel: 1_000_000, // ~1M instructions
            max_output_size_kb: 16,        // 16KB output limit
        }
    }
}

#[allow(dead_code)]
pub fn create_secure_wasi_context(
    _manifest: &PluginManifest,
) -> Result<(), UveddiError> {
    // Temporarily disabled during development to prevent WASM runtime errors
    log::warn!("WASI context creation temporarily disabled during development");
    Err(UveddiError::Plugin("WASI context creation disabled".to_string()))
}

#[allow(dead_code)]
pub fn validate_resource_limits(_manifest: &PluginManifest) -> Result<(), UveddiError> {
    // Temporarily disabled during development
    log::warn!("Resource limit validation temporarily disabled during development");
    Ok(())
}
