//! Workspace detection and multi-crate analysis support
//!
//! This module provides functionality to detect and analyze Rust workspaces,
//! handling both single-crate and multi-crate projects robustly.

use crate::analysis::errors::AnalysisError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tokio::fs as async_fs;
use tracing::{debug, info, warn};

/// Represents a Rust crate within a workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrateInfo {
    /// Name of the crate from Cargo.toml
    pub name: String,
    /// Absolute path to the crate directory
    pub path: PathBuf,
    /// Path to the Cargo.toml file
    pub manifest_path: PathBuf,
    /// Dependencies declared in Cargo.toml
    pub dependencies: Vec<String>,
    /// Whether this is a workspace root
    pub is_workspace_root: bool,
    /// Source directories (usually just "src")
    pub source_dirs: Vec<PathBuf>,
}

/// Represents a Rust workspace containing multiple crates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceInfo {
    /// Root directory of the workspace
    pub root_path: PathBuf,
    /// Path to the root Cargo.toml
    pub manifest_path: PathBuf,
    /// All crates in the workspace
    pub crates: HashMap<String, CrateInfo>,
    /// Workspace members from Cargo.toml
    pub members: Vec<String>,
}

/// Minimal Cargo.toml structure for parsing
#[derive(Debug, Deserialize)]
struct CargoManifest {
    package: Option<PackageInfo>,
    workspace: Option<WorkspaceManifest>,
    dependencies: Option<HashMap<String, toml::Value>>,
}

#[derive(Debug, Deserialize)]
struct PackageInfo {
    name: String,
    version: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WorkspaceManifest {
    members: Option<Vec<String>>,
}

/// Workspace detector and analyzer
pub struct WorkspaceDetector;

impl WorkspaceDetector {
    /// Detect if a path is part of a Rust workspace and return workspace info
    pub async fn detect_workspace(path: &Path) -> Result<Option<WorkspaceInfo>, AnalysisError> {
        debug!("Detecting workspace for path: {:?}", path);

        // Start from the given path and walk up to find workspace root
        let start_path = if path.is_file() {
            path.parent().unwrap_or(path)
        } else {
            path
        };

        // First, try to find a workspace root by walking up the directory tree
        if let Some(workspace_root) = Self::find_workspace_root(start_path).await? {
            info!("Found workspace root at: {:?}", workspace_root);
            return Self::analyze_workspace(&workspace_root).await.map(Some);
        }

        // If no workspace found, check if we have a single crate
        if let Some(single_crate) = Self::detect_single_crate(start_path).await? {
            info!("Found single crate at: {:?}", single_crate.path);
            return Ok(Some(Self::single_crate_to_workspace(single_crate)));
        }

        debug!("No workspace or crate detected at: {:?}", path);
        Ok(None)
    }

    /// Find workspace root by walking up directory tree
    async fn find_workspace_root(start_path: &Path) -> Result<Option<PathBuf>, AnalysisError> {
        let mut current = start_path.to_path_buf();

        loop {
            let manifest_path = current.join("Cargo.toml");
            
            if manifest_path.exists() {
                // Read and parse the Cargo.toml to check if it's a workspace root
                match Self::parse_manifest(&manifest_path).await {
                    Ok(manifest) => {
                        if manifest.workspace.is_some() {
                            return Ok(Some(current));
                        }
                        // Keep looking up for workspace root, but remember this as a potential single crate
                    }
                    Err(e) => {
                        warn!("Failed to parse manifest at {:?}: {}", manifest_path, e);
                    }
                }
            }

            // Move up one directory
            if let Some(parent) = current.parent() {
                current = parent.to_path_buf();
            } else {
                break;
            }
        }

        Ok(None)
    }

    /// Detect a single crate (no workspace)
    async fn detect_single_crate(path: &Path) -> Result<Option<CrateInfo>, AnalysisError> {
        let manifest_path = path.join("Cargo.toml");
        
        if !manifest_path.exists() {
            return Ok(None);
        }

        let manifest = Self::parse_manifest(&manifest_path).await?;
        
        // If it has workspace info, it's not a single crate
        if manifest.workspace.is_some() {
            return Ok(None);
        }

        // If it has package info, it's a single crate
        if let Some(package) = manifest.package {
            let dependencies = manifest.dependencies
                .unwrap_or_default()
                .keys()
                .map(|k| k.clone())
                .collect();

            let source_dirs = vec![path.join("src")];

            return Ok(Some(CrateInfo {
                name: package.name,
                path: path.to_path_buf(),
                manifest_path,
                dependencies,
                is_workspace_root: false,
                source_dirs,
            }));
        }

        Ok(None)
    }

    /// Analyze a workspace and discover all member crates
    async fn analyze_workspace(workspace_root: &Path) -> Result<WorkspaceInfo, AnalysisError> {
        let manifest_path = workspace_root.join("Cargo.toml");
        let manifest = Self::parse_manifest(&manifest_path).await?;

        let workspace_manifest = manifest.workspace.ok_or_else(|| {
            AnalysisError::workspace_discovery_error(
                workspace_root.display().to_string(),
                "Cargo.toml does not contain workspace configuration",
            )
        })?;

        let members = workspace_manifest.members.unwrap_or_default();
        let mut crates = HashMap::new();

        // Analyze each workspace member
        for member_pattern in &members {
            let member_paths = Self::expand_member_pattern(workspace_root, member_pattern)?;
            
            for member_path in member_paths {
                match Self::analyze_member_crate(&member_path).await {
                    Ok(crate_info) => {
                        crates.insert(crate_info.name.clone(), crate_info);
                    }
                    Err(e) => {
                        warn!("Failed to analyze member crate at {:?}: {}", member_path, e);
                        // Continue with other crates instead of failing completely
                    }
                }
            }
        }

        Ok(WorkspaceInfo {
            root_path: workspace_root.to_path_buf(),
            manifest_path,
            crates,
            members,
        })
    }

    /// Expand workspace member patterns (handle globs like "crates/*")
    fn expand_member_pattern(workspace_root: &Path, pattern: &str) -> Result<Vec<PathBuf>, AnalysisError> {
        let pattern_path = workspace_root.join(pattern);
        
        // If the pattern contains wildcards, expand them
        if pattern.contains('*') {
            match glob::glob(&pattern_path.to_string_lossy()) {
                Ok(paths) => {
                    let mut result = Vec::new();
                    for path_result in paths {
                        match path_result {
                            Ok(path) => {
                                if path.join("Cargo.toml").exists() {
                                    result.push(path);
                                }
                            }
                            Err(e) => {
                                warn!("Glob pattern error: {}", e);
                            }
                        }
                    }
                    Ok(result)
                }
                Err(e) => Err(AnalysisError::workspace_discovery_error(
                    workspace_root.display().to_string(),
                    format!("Failed to expand glob pattern '{}': {}", pattern, e),
                )),
            }
        } else {
            // Direct path
            if pattern_path.join("Cargo.toml").exists() {
                Ok(vec![pattern_path])
            } else {
                Ok(vec![])
            }
        }
    }

    /// Analyze a single member crate
    async fn analyze_member_crate(crate_path: &Path) -> Result<CrateInfo, AnalysisError> {
        let manifest_path = crate_path.join("Cargo.toml");
        let manifest = Self::parse_manifest(&manifest_path).await?;

        let package = manifest.package.ok_or_else(|| {
            AnalysisError::crate_analysis_error(
                "unknown".to_string(),
                crate_path.display().to_string(),
                "Cargo.toml missing [package] section",
            )
        })?;

        let dependencies = manifest.dependencies
            .unwrap_or_default()
            .keys()
            .map(|k| k.clone())
            .collect();

        let source_dirs = vec![crate_path.join("src")];

        Ok(CrateInfo {
            name: package.name,
            path: crate_path.to_path_buf(),
            manifest_path,
            dependencies,
            is_workspace_root: false,
            source_dirs,
        })
    }

    /// Convert a single crate into a workspace info structure
    fn single_crate_to_workspace(crate_info: CrateInfo) -> WorkspaceInfo {
        let mut crates = HashMap::new();
        let crate_name = crate_info.name.clone();
        crates.insert(crate_name.clone(), crate_info.clone());

        WorkspaceInfo {
            root_path: crate_info.path.clone(),
            manifest_path: crate_info.manifest_path.clone(),
            crates,
            members: vec![".".to_string()],
        }
    }

    /// Parse a Cargo.toml manifest file
    async fn parse_manifest(manifest_path: &Path) -> Result<CargoManifest, AnalysisError> {
        let content = async_fs::read_to_string(manifest_path)
            .await
            .map_err(|e| AnalysisError::file_system_error(manifest_path.display().to_string(), e))?;

        toml::from_str(&content).map_err(|e| {
            AnalysisError::workspace_discovery_error(
                manifest_path.display().to_string(),
                format!("Failed to parse TOML: {}", e),
            )
        })
    }

    /// Get all source files from a workspace
    pub async fn get_workspace_source_files(workspace: &WorkspaceInfo) -> Result<Vec<PathBuf>, AnalysisError> {
        let mut all_files = Vec::new();

        for crate_info in workspace.crates.values() {
            for source_dir in &crate_info.source_dirs {
                if source_dir.exists() {
                    let files = Self::collect_rust_files(source_dir).await?;
                    all_files.extend(files);
                }
            }
        }

        Ok(all_files)
    }

    /// Recursively collect all Rust files from a directory
    pub async fn collect_rust_files(dir: &Path) -> Result<Vec<PathBuf>, AnalysisError> {
        let mut files = Vec::new();
        let mut read_dir = async_fs::read_dir(dir)
            .await
            .map_err(|e| AnalysisError::file_system_error(dir.display().to_string(), e))?;

        while let Some(entry) = read_dir.next_entry()
            .await
            .map_err(|e| AnalysisError::file_system_error(dir.display().to_string(), e))? 
        {
            let path = entry.path();
            let file_type = entry.file_type()
                .await
                .map_err(|e| AnalysisError::file_system_error(path.display().to_string(), e))?;

            if file_type.is_file() {
                if let Some(extension) = path.extension() {
                    if extension == "rs" {
                        files.push(path);
                    }
                }
            } else if file_type.is_dir() {
                // Skip target directories and hidden directories
                if let Some(dir_name) = path.file_name() {
                    let dir_name = dir_name.to_string_lossy();
                    if !dir_name.starts_with('.') && dir_name != "target" {
                        let sub_files = Box::pin(Self::collect_rust_files(&path)).await?;
                        files.extend(sub_files);
                    }
                }
            }
        }

        Ok(files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs::write;

    #[tokio::test]
    async fn test_single_crate_detection() {
        let temp_dir = TempDir::new().unwrap();
        let crate_path = temp_dir.path();

        // Create a simple Cargo.toml
        let manifest_content = r#"
[package]
name = "test_crate"
version = "0.1.0"

[dependencies]
serde = "1.0"
"#;
        write(crate_path.join("Cargo.toml"), manifest_content).await.unwrap();

        // Create src directory
        tokio::fs::create_dir(crate_path.join("src")).await.unwrap();

        let workspace = WorkspaceDetector::detect_workspace(crate_path).await.unwrap();
        assert!(workspace.is_some());

        let workspace = workspace.unwrap();
        assert_eq!(workspace.crates.len(), 1);
        assert!(workspace.crates.contains_key("test_crate"));
    }

    #[tokio::test]
    async fn test_workspace_detection() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_path = temp_dir.path();

        // Create workspace Cargo.toml
        let workspace_manifest = r#"
[workspace]
members = ["crate_a", "crate_b"]
"#;
        write(workspace_path.join("Cargo.toml"), workspace_manifest).await.unwrap();

        // Create member crates
        for crate_name in &["crate_a", "crate_b"] {
            let crate_path = workspace_path.join(crate_name);
            tokio::fs::create_dir_all(&crate_path).await.unwrap();
            
            let crate_manifest = format!(r#"
[package]
name = "{}"
version = "0.1.0"
"#, crate_name);
            write(crate_path.join("Cargo.toml"), crate_manifest).await.unwrap();
            tokio::fs::create_dir(crate_path.join("src")).await.unwrap();
        }

        let workspace = WorkspaceDetector::detect_workspace(workspace_path).await.unwrap();
        assert!(workspace.is_some());

        let workspace = workspace.unwrap();
        assert_eq!(workspace.crates.len(), 2);
        assert!(workspace.crates.contains_key("crate_a"));
        assert!(workspace.crates.contains_key("crate_b"));
    }
}