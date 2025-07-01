use std::path::Path;
use sha2::{Sha256, Digest};
use crate::error::UveddiError;
use crate::plugin::security::{PluginManifest, validate_resource_limits};

pub struct PluginVerifier;

impl PluginVerifier {
    pub fn verify_plugin(wasm_path: &Path, manifest_path: &Path) -> Result<PluginManifest, UveddiError> {
        // 1. Verify file integrity
        Self::verify_file_hash(wasm_path)?;
        
        // 2. Parse and validate manifest
        let manifest = Self::load_manifest(manifest_path)?;
        Self::validate_manifest(&manifest)?;
        
        // 3. Basic WASM structure validation
        Self::validate_wasm_structure(wasm_path)?;
        
        // 4. Validate resource limits
        validate_resource_limits(&manifest.resource_limits)?;
        
        log::info!("Plugin verification successful: {}", manifest.name);
        Ok(manifest)
    }
    
    fn verify_file_hash(wasm_path: &Path) -> Result<(), UveddiError> {
        let contents = std::fs::read(wasm_path)
            .map_err(|e| UveddiError::PluginError(format!("Failed to read WASM file: {}", e)))?;
        
        let hash = Sha256::digest(&contents);
        
        // In a full implementation, we would compare against a known hash from a registry
        // For now, we just log the hash for integrity verification
        log::info!("Plugin hash: {:x}", hash);
        
        // Basic size validation - reject very large or very small files
        if contents.len() < 100 {
            return Err(UveddiError::PluginError("WASM file too small to be valid".to_string()));
        }
        
        if contents.len() > 50 * 1024 * 1024 { // 50MB limit
            return Err(UveddiError::PluginError("WASM file too large (>50MB)".to_string()));
        }
        
        // Verify WASM magic bytes
        if contents.len() >= 4 && &contents[0..4] != b"\0asm" {
            return Err(UveddiError::PluginError("Invalid WASM magic bytes".to_string()));
        }
        
        Ok(())
    }
    
    fn load_manifest(manifest_path: &Path) -> Result<PluginManifest, UveddiError> {
        let contents = std::fs::read_to_string(manifest_path)
            .map_err(|e| UveddiError::PluginError(format!("Failed to read manifest: {}", e)))?;
        
        let manifest: PluginManifest = toml::from_str(&contents)
            .map_err(|e| UveddiError::PluginError(format!("Invalid manifest TOML: {}", e)))?;
        
        Ok(manifest)
    }
    
    fn validate_manifest(manifest: &PluginManifest) -> Result<(), UveddiError> {
        // Validate plugin name
        if manifest.name.is_empty() {
            return Err(UveddiError::PluginError("Plugin name cannot be empty".to_string()));
        }
        
        if manifest.name.len() > 100 {
            return Err(UveddiError::PluginError("Plugin name too long (>100 chars)".to_string()));
        }
        
        // Validate plugin name contains only safe characters
        if !manifest.name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err(UveddiError::PluginError(
                "Plugin name can only contain alphanumeric characters, hyphens, and underscores".to_string()
            ));
        }
        
        // Validate version format (basic semver check)
        if !Self::is_valid_version(&manifest.version) {
            return Err(UveddiError::PluginError("Invalid version format (use semver)".to_string()));
        }
        
        // Validate permissions count (prevent permission spam)
        if manifest.permissions.len() > 20 {
            return Err(UveddiError::PluginError("Too many permissions requested (max 20)".to_string()));
        }
        
        log::info!("Manifest validation passed for plugin: {}", manifest.name);
        Ok(())
    }
    
    fn validate_wasm_structure(wasm_path: &Path) -> Result<(), UveddiError> {
        // Use wasmtime to parse and validate the module structure
        let engine = wasmtime::Engine::default();
        let module = wasmtime::Module::from_file(&engine, wasm_path)
            .map_err(|e| UveddiError::PluginError(format!("Invalid WASM module: {}", e)))?;
        
        // Validate that the module has the expected exports
        let mut has_info_export = false;
        let mut has_analyze_export = false;
        
        for export in module.exports() {
            match export.name() {
                "info" => has_info_export = true,
                "analyze" => has_analyze_export = true,
                _ => {} // Other exports are allowed
            }
        }
        
        if !has_info_export {
            log::warn!("WASM module missing 'info' export function");
        }
        
        if !has_analyze_export {
            log::warn!("WASM module missing 'analyze' export function");
        }
        
        // Check for suspicious imports that might indicate malicious behavior
        for import in module.imports() {
            let module_name = import.module();
            let name = import.name();
            
            // Block suspicious imports
            let blocked_imports = [
                ("env", "system"),
                ("env", "exec"),
                ("wasi_snapshot_preview1", "fd_write"), // Allow for now but log
            ];
            
            for (blocked_module, blocked_name) in &blocked_imports {
                if module_name == *blocked_module && name == *blocked_name {
                    if name == "fd_write" {
                        log::debug!("Plugin imports fd_write (stdio access)");
                    } else {
                        return Err(UveddiError::PluginError(
                            format!("Blocked import: {}::{}", module_name, name)
                        ));
                    }
                }
            }
        }
        
        log::info!("WASM structure validation passed");
        Ok(())
    }
    
    fn is_valid_version(version: &str) -> bool {
        // Basic semver validation - should match pattern: major.minor.patch
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() != 3 {
            return false;
        }
        
        // Each part should be a number
        parts.iter().all(|part| part.parse::<u32>().is_ok())
    }
    
    pub fn create_plugin_hash(wasm_path: &Path) -> Result<String, UveddiError> {
        let contents = std::fs::read(wasm_path)
            .map_err(|e| UveddiError::PluginError(format!("Failed to read WASM file: {}", e)))?;
        
        let hash = Sha256::digest(&contents);
        Ok(format!("{:x}", hash))
    }
    
    pub fn verify_plugin_signature(wasm_path: &Path, expected_hash: &str) -> Result<bool, UveddiError> {
        let actual_hash = Self::create_plugin_hash(wasm_path)?;
        Ok(actual_hash == expected_hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    
    #[test]
    fn test_version_validation() {
        assert!(PluginVerifier::is_valid_version("1.0.0"));
        assert!(PluginVerifier::is_valid_version("0.1.2"));
        assert!(PluginVerifier::is_valid_version("10.20.30"));
        
        assert!(!PluginVerifier::is_valid_version("1.0"));
        assert!(!PluginVerifier::is_valid_version("1.0.0.0"));
        assert!(!PluginVerifier::is_valid_version("1.0.a"));
        assert!(!PluginVerifier::is_valid_version("invalid"));
    }
    
    #[test]
    fn test_wasm_magic_bytes_validation() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        
        // Test invalid WASM file (wrong magic bytes)
        let invalid_wasm = temp_dir.path().join("invalid.wasm");
        fs::write(&invalid_wasm, b"invalid wasm content")?;
        
        let result = PluginVerifier::verify_file_hash(&invalid_wasm);
        assert!(result.is_err());
        
        // Test valid WASM magic bytes (but incomplete file)
        let valid_magic_wasm = temp_dir.path().join("valid_magic.wasm");
        let mut wasm_content = b"\0asm".to_vec();
        wasm_content.extend_from_slice(&[1, 0, 0, 0]); // Version
        wasm_content.extend_from_slice(&[0; 100]); // Padding to meet size requirement
        fs::write(&valid_magic_wasm, &wasm_content)?;
        
        let result = PluginVerifier::verify_file_hash(&valid_magic_wasm);
        assert!(result.is_ok());
        
        Ok(())
    }
    
    #[test]
    fn test_manifest_validation() {
        let valid_manifest = PluginManifest {
            name: "test-plugin".to_string(),
            version: "1.0.0".to_string(),
            permissions: vec![],
            resource_limits: Default::default(),
        };
        
        assert!(PluginVerifier::validate_manifest(&valid_manifest).is_ok());
        
        // Test invalid name
        let invalid_name_manifest = PluginManifest {
            name: "".to_string(),
            version: "1.0.0".to_string(),
            permissions: vec![],
            resource_limits: Default::default(),
        };
        
        assert!(PluginVerifier::validate_manifest(&invalid_name_manifest).is_err());
        
        // Test invalid version
        let invalid_version_manifest = PluginManifest {
            name: "test-plugin".to_string(),
            version: "invalid-version".to_string(),
            permissions: vec![],
            resource_limits: Default::default(),
        };
        
        assert!(PluginVerifier::validate_manifest(&invalid_version_manifest).is_err());
    }
    
    #[test]
    fn test_plugin_hash_creation() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("test.wasm");
        fs::write(&test_file, b"test content")?;
        
        let hash = PluginVerifier::create_plugin_hash(&test_file)?;
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64); // SHA256 hex string length
        
        // Test that same content produces same hash
        let hash2 = PluginVerifier::create_plugin_hash(&test_file)?;
        assert_eq!(hash, hash2);
        
        Ok(())
    }
}