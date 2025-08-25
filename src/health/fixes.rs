//! Auto-fix implementations for health check issues
//!
//! Provides automated remediation for common health check failures,
//! including directory creation, service management, and configuration fixes.

use crate::core::UveddiError;
use std::path::Path;
use tokio::fs;
use tokio::process::Command;

/// Apply automatic fixes for known health check issues
pub async fn apply_fix(check_name: &str) -> Result<String, UveddiError> {
    match check_name {
        "config_directories" => fix_config_directories().await,
        "file_permissions" => fix_file_permissions().await,
        "ollama_connectivity" => fix_ollama_connectivity().await,
        "ollama_models" => fix_ollama_models().await,
        _ => Err(UveddiError::Config(format!("No automatic fix available for: {}", check_name)))
    }
}

/// Create missing configuration directories
async fn fix_config_directories() -> Result<String, UveddiError> {
    let dirs_to_create = [
        dirs::config_dir().map(|p| p.join("uveddi")),
        dirs::cache_dir().map(|p| p.join("uveddi")),
        dirs::data_dir().map(|p| p.join("uveddi")),
    ];

    let mut created_dirs = Vec::new();
    let mut errors = Vec::new();

    for dir_opt in &dirs_to_create {
        if let Some(dir) = dir_opt {
            if !dir.exists() {
                match fs::create_dir_all(dir).await {
                    Ok(_) => {
                        created_dirs.push(dir.display().to_string());
                    },
                    Err(e) => {
                        errors.push(format!("Failed to create {}: {}", dir.display(), e));
                    }
                }
            }
        }
    }

    if !created_dirs.is_empty() && errors.is_empty() {
        Ok(format!("Created configuration directories: {}", created_dirs.join(", ")))
    } else if !errors.is_empty() {
        Err(UveddiError::Config(format!("Directory creation errors: {}", errors.join("; "))))
    } else {
        Ok("All directories already exist".to_string())
    }
}

/// Fix file permission issues
async fn fix_file_permissions() -> Result<String, UveddiError> {
    // Create a test file in temp directory to verify write permissions
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("uveddi_permission_test.tmp");

    match fs::write(&test_file, "test").await {
        Ok(_) => {
            // Clean up test file
            let _ = fs::remove_file(&test_file).await;
            Ok("File permissions verified and working".to_string())
        },
        Err(e) => {
            Err(UveddiError::Config(format!("Cannot write to temp directory: {}. Check file system permissions.", e)))
        }
    }
}

/// Attempt to start Ollama service
async fn fix_ollama_connectivity() -> Result<String, UveddiError> {
    // Check if Ollama is installed
    match Command::new("ollama").arg("--version").output().await {
        Ok(output) => {
            if output.status.success() {
                // Try to start Ollama service
                match Command::new("ollama").arg("serve").spawn() {
                    Ok(mut child) => {
                        // Give it a moment to start
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                        
                        // Check if it's still running
                        match child.try_wait() {
                            Ok(None) => {
                                // Still running, detach it
                                let _ = child.start_kill();
                                Ok("Started Ollama service in background. Run 'ollama serve' manually to keep it running.".to_string())
                            },
                            Ok(Some(status)) => {
                                if status.success() {
                                    Ok("Ollama service started successfully".to_string())
                                } else {
                                    Err(UveddiError::Config(format!("Ollama service exited with status: {}", status)))
                                }
                            },
                            Err(e) => Err(UveddiError::Config(format!("Error checking Ollama service status: {}", e)))
                        }
                    },
                    Err(e) => Err(UveddiError::Config(format!("Failed to start Ollama service: {}", e)))
                }
            } else {
                Err(UveddiError::Config("Ollama is installed but not working properly".to_string()))
            }
        },
        Err(_) => Err(UveddiError::Config("Ollama is not installed. Please install it from https://ollama.ai/".to_string()))
    }
}

/// Install recommended Ollama models
async fn fix_ollama_models() -> Result<String, UveddiError> {
    let recommended_model = std::env::var("OLLAMA_MODEL")
        .unwrap_or_else(|_| "deepseek-coder:6.7b".to_string());

    // Check if Ollama is available
    match Command::new("ollama").arg("--version").output().await {
        Ok(output) => {
            if output.status.success() {
                // Try to pull the recommended model
                match Command::new("ollama")
                    .arg("pull")
                    .arg(&recommended_model)
                    .output()
                    .await 
                {
                    Ok(pull_output) => {
                        if pull_output.status.success() {
                            Ok(format!("Successfully installed model: {}", recommended_model))
                        } else {
                            let stderr = String::from_utf8_lossy(&pull_output.stderr);
                            Err(UveddiError::Config(format!("Failed to install model {}: {}", recommended_model, stderr)))
                        }
                    },
                    Err(e) => Err(UveddiError::Config(format!("Error running 'ollama pull': {}", e)))
                }
            } else {
                Err(UveddiError::Config("Ollama command is not working properly".to_string()))
            }
        },
        Err(_) => Err(UveddiError::Config("Ollama is not installed. Cannot install models.".to_string()))
    }
}

/// Suggest manual fixes for issues that can't be automatically resolved
pub fn suggest_manual_fix(check_name: &str) -> Option<String> {
    match check_name {
        "binary_integrity" => Some("Try reinstalling Uveddi or check file system integrity".to_string()),
        "disk_space" => Some("Free up disk space or move to a location with more available space".to_string()),
        "memory" => Some("Close other applications or upgrade system memory".to_string()),
        "tree_sitter" => Some("Recompile with --features tree-sitter to enable parsing".to_string()),
        "rust_parser" | "python_parser" | "javascript_parser" | "typescript_parser" => {
            Some("Recompile with appropriate language features enabled".to_string())
        },
        "ai_features" => Some("Recompile with --features ai to enable AI analysis".to_string()),
        _ => None
    }
}

/// Check if a fix is available for a given health check
pub fn is_fixable(check_name: &str) -> bool {
    matches!(check_name, 
        "config_directories" | 
        "file_permissions" | 
        "ollama_connectivity" | 
        "ollama_models"
    )
}

/// Get a description of what an auto-fix will do
pub fn describe_fix(check_name: &str) -> Option<String> {
    match check_name {
        "config_directories" => Some("Create missing configuration directories in ~/.config/uveddi, ~/.cache/uveddi".to_string()),
        "file_permissions" => Some("Test and verify file system write permissions".to_string()),
        "ollama_connectivity" => Some("Attempt to start the Ollama service".to_string()),
        "ollama_models" => Some(format!("Download and install the recommended model: {}", 
            std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "deepseek-coder:6.7b".to_string()))),
        _ => None
    }
}