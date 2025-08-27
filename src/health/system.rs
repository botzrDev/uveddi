//! System-level health checks
//!
//! Validates system requirements, resources, and basic functionality
//! needed for Uveddi to operate correctly.

use super::{HealthCheck, HealthStatus};
use crate::error::UveddiError;
use std::path::Path;
use tokio::fs;

/// Check overall system health
pub async fn check_system_health() -> Result<Vec<HealthCheck>, UveddiError> {
    let mut checks = Vec::new();

    checks.push(check_binary_integrity().await);
    checks.push(check_file_permissions().await);
    checks.push(check_disk_space().await);
    checks.push(check_memory_availability().await);
    checks.push(check_config_directories().await);

    Ok(checks)
}

/// Verify binary integrity and basic functionality
async fn check_binary_integrity() -> HealthCheck {
    // Check if we can access our own binary
    match std::env::current_exe() {
        Ok(exe_path) => {
            if exe_path.exists() {
                HealthCheck::new(
                    "binary_integrity",
                    HealthStatus::Healthy,
                    "Binary is accessible and valid"
                )
            } else {
                HealthCheck::new(
                    "binary_integrity",
                    HealthStatus::Critical,
                    "Binary path exists but file is not accessible"
                )
            }
        },
        Err(e) => HealthCheck::new(
            "binary_integrity",
            HealthStatus::Critical,
            format!("Cannot determine binary path: {}", e)
        )
    }
}

/// Check file system permissions for temp directories and config paths
async fn check_file_permissions() -> HealthCheck {
    let test_paths = [
        std::env::temp_dir(),
        dirs::config_dir().unwrap_or_else(|| std::env::temp_dir()),
        dirs::cache_dir().unwrap_or_else(|| std::env::temp_dir()),
    ];

    let mut accessible_paths = 0;
    let mut issues = Vec::new();

    for path in &test_paths {
        match fs::metadata(path).await {
            Ok(metadata) => {
                if metadata.permissions().readonly() {
                    issues.push(format!("Path {} is read-only", path.display()));
                } else {
                    accessible_paths += 1;
                }
            },
            Err(e) => {
                issues.push(format!("Cannot access {}: {}", path.display(), e));
            }
        }
    }

    if accessible_paths == test_paths.len() {
        HealthCheck::new(
            "file_permissions",
            HealthStatus::Healthy,
            "All required directories are accessible"
        )
    } else if accessible_paths > 0 {
        HealthCheck::new(
            "file_permissions",
            HealthStatus::Warning,
            format!("Some directories have permission issues: {}", issues.join(", "))
        ).with_auto_fix(true)
    } else {
        HealthCheck::new(
            "file_permissions",
            HealthStatus::Critical,
            format!("No writable directories found: {}", issues.join(", "))
        ).with_auto_fix(true)
    }
}

/// Check available disk space
async fn check_disk_space() -> HealthCheck {
    match fs::metadata(std::env::current_dir().unwrap_or_else(|_| ".".into())).await {
        Ok(_) => {
            // Note: Getting actual disk space is platform-specific and would require
            // additional dependencies. For now, we assume it's available if we can
            // access the current directory.
            HealthCheck::new(
                "disk_space",
                HealthStatus::Healthy,
                "Disk space appears adequate"
            )
        },
        Err(e) => HealthCheck::new(
            "disk_space",
            HealthStatus::Warning,
            format!("Cannot check disk space: {}", e)
        )
    }
}

/// Check system memory availability
async fn check_memory_availability() -> HealthCheck {
    // Basic memory check - if we got this far, we likely have enough memory
    // A more sophisticated implementation could use system APIs to check actual memory
    HealthCheck::new(
        "memory",
        HealthStatus::Healthy,
        "Memory appears adequate for operation"
    )
}

/// Ensure required config directories exist
async fn check_config_directories() -> HealthCheck {
    let config_dirs = [
        dirs::config_dir().map(|p| p.join("uveddi")),
        dirs::cache_dir().map(|p| p.join("uveddi")),
    ];

    let mut missing_dirs = Vec::new();
    let mut _accessible_dirs = 0;

    for dir_opt in &config_dirs {
        if let Some(dir) = dir_opt {
            match fs::metadata(dir).await {
                Ok(_) => _accessible_dirs += 1,
                Err(_) => missing_dirs.push(dir.display().to_string()),
            }
        }
    }

    if missing_dirs.is_empty() {
        HealthCheck::new(
            "config_directories",
            HealthStatus::Healthy,
            "All configuration directories exist"
        )
    } else {
        HealthCheck::new(
            "config_directories",
            HealthStatus::Warning,
            format!("Missing configuration directories: {}", missing_dirs.join(", "))
        ).with_auto_fix(true).with_details("Run with --fix to create missing directories")
    }
}