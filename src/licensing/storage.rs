//! License storage operations
//!
//! This module handles reading and writing license files to disk.
//! Licenses are stored in ~/.uveddi/license.json

use std::fs;
use std::path::PathBuf;

use super::errors::LicenseError;
use super::license::License;

/// Get the path to the license file
pub fn get_license_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".uveddi").join("license.json")
}

/// Get the path to the uveddi config directory
pub fn get_config_dir() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".uveddi")
}

/// Ensure the config directory exists
fn ensure_config_dir() -> Result<(), LicenseError> {
    let config_dir = get_config_dir();
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).map_err(|e| {
            LicenseError::storage_error(format!(
                "Failed to create config directory at {}: {}",
                config_dir.display(),
                e
            ))
        })?;
    }
    Ok(())
}

/// Load a license from disk
pub fn load_license() -> Result<License, LicenseError> {
    let license_path = get_license_path();
    
    if !license_path.exists() {
        return Err(LicenseError::not_activated());
    }
    
    let content = fs::read_to_string(&license_path).map_err(|e| {
        LicenseError::storage_error(format!(
            "Failed to read license file at {}: {}",
            license_path.display(),
            e
        ))
    })?;
    
    let license: License = serde_json::from_str(&content).map_err(|e| {
        LicenseError::storage_error(format!(
            "Failed to parse license file: {}. Try re-activating your license.",
            e
        ))
    })?;
    
    Ok(license)
}

/// Save a license to disk
pub fn save_license(license: &License) -> Result<(), LicenseError> {
    ensure_config_dir()?;
    
    let license_path = get_license_path();
    let content = serde_json::to_string_pretty(license)?;
    
    fs::write(&license_path, content).map_err(|e| {
        LicenseError::storage_error(format!(
            "Failed to write license file at {}: {}",
            license_path.display(),
            e
        ))
    })?;
    
    // Set restrictive permissions on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&license_path)?.permissions();
        perms.set_mode(0o600); // Owner read/write only
        fs::set_permissions(&license_path, perms)?;
    }
    
    Ok(())
}

/// Delete the license file (for deactivation)
pub fn delete_license() -> Result<(), LicenseError> {
    let license_path = get_license_path();
    
    if license_path.exists() {
        fs::remove_file(&license_path).map_err(|e| {
            LicenseError::storage_error(format!(
                "Failed to delete license file: {}",
                e
            ))
        })?;
    }
    
    Ok(())
}

/// Check if a license file exists
pub fn license_exists() -> bool {
    get_license_path().exists()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_get_license_path() {
        let path = get_license_path();
        assert!(path.to_string_lossy().contains(".uveddi"));
        assert!(path.to_string_lossy().contains("license.json"));
    }
}
