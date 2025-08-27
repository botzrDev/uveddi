//! Workspace detection and multi-crate analysis support
//!
//! This module provides functionality to detect and analyze Rust workspaces,
//! handling both single-crate and multi-crate projects robustly.

use crate::analysis::errors::AnalysisError;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use tokio::fs as async_fs;
use tracing::{debug, info, warn};

/// Represents a project component (crate, package, module) within a workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectComponent {
    /// Name of the component from manifest file
    pub name: String,
    /// Absolute path to the component directory
    pub path: PathBuf,
    /// Path to the primary manifest file
    pub manifest_path: PathBuf,
    /// Dependencies declared in manifest
    pub dependencies: Vec<String>,
    /// Whether this is a workspace/project root
    pub is_root: bool,
    /// Source directories
    pub source_dirs: Vec<PathBuf>,
    /// Component type based on language
    pub component_type: ComponentType,
}

/// Backward compatibility alias for Rust crates
pub type CrateInfo = ProjectComponent;

/// Types of project components based on language
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ComponentType {
    RustCrate,
    PythonPackage,
    JavaScriptPackage,
    TypeScriptPackage,
    MixedLanguage,
}

/// Represents a multi-language workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceInfo {
    /// Root directory of the workspace
    pub root_path: PathBuf,
    /// Primary manifest files found
    pub manifest_paths: Vec<PathBuf>,
    /// All components in the workspace
    pub components: HashMap<String, ProjectComponent>,
    /// Workspace type
    pub workspace_type: WorkspaceType,
    /// Languages detected in the workspace
    pub languages: HashSet<String>,
}

/// Minimal Cargo.toml structure for parsing
#[derive(Debug, Deserialize)]
struct CargoManifest {
    package: Option<PackageInfo>,
    workspace: Option<WorkspaceManifest>,
    dependencies: Option<HashMap<String, toml::Value>>,
}

/// Python pyproject.toml structure
#[derive(Debug, Deserialize)]
struct PyProjectManifest {
    project: Option<PythonProjectInfo>,
    build_system: Option<HashMap<String, toml::Value>>,
    tool: Option<HashMap<String, toml::Value>>,
}

/// JavaScript/TypeScript package.json structure
#[derive(Debug, Deserialize)]
struct PackageJsonManifest {
    name: Option<String>,
    version: Option<String>,
    dependencies: Option<HashMap<String, String>>,
    #[serde(rename = "devDependencies")]
    dev_dependencies: Option<HashMap<String, String>>,
    workspaces: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct PackageInfo {
    name: String,
    version: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PythonProjectInfo {
    name: Option<String>,
    version: Option<String>,
    dependencies: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct WorkspaceManifest {
    members: Option<Vec<String>>,
}

/// Supported workspace types (multi-language Phase 1, UV-XXX)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum WorkspaceType {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Mixed,
    Unknown,
}

/// Workspace detector and analyzer
pub struct WorkspaceDetector;

impl WorkspaceDetector {
    /// Detect multi-language workspace and return workspace info
    pub async fn detect_workspace(path: &Path) -> Result<Option<WorkspaceInfo>, AnalysisError> {
        debug!("Detecting multi-language workspace for path: {:?}", path);

        // Start from the given path and walk up to find workspace root
        let start_path = if path.is_file() {
            path.parent().unwrap_or(path)
        } else {
            path
        };

        // Try different workspace detection strategies
        if let Some(workspace) = Self::detect_rust_workspace(start_path).await? {
            return Ok(Some(workspace));
        }

        if let Some(workspace) = Self::detect_python_workspace(start_path).await? {
            return Ok(Some(workspace));
        }

        if let Some(workspace) = Self::detect_javascript_workspace(start_path).await? {
            return Ok(Some(workspace));
        }

        if let Some(workspace) = Self::detect_mixed_workspace(start_path).await? {
            return Ok(Some(workspace));
        }

        // If no structured workspace found, create a loose file workspace
        if let Some(workspace) = Self::create_loose_file_workspace(start_path).await? {
            return Ok(Some(workspace));
        }

        debug!("No workspace detected at: {:?}", path);
        Ok(None)
    }

    /// Detect Rust workspace (Cargo.toml based)
    async fn detect_rust_workspace(
        start_path: &Path,
    ) -> Result<Option<WorkspaceInfo>, AnalysisError> {
        if let Some(workspace_root) = Self::find_cargo_workspace_root(start_path).await? {
            info!("Found Rust workspace root at: {:?}", workspace_root);
            return Self::analyze_rust_workspace(&workspace_root)
                .await
                .map(Some);
        }

        // Check for single Rust crate
        if let Some(component) = Self::detect_single_crate(start_path).await? {
            info!("Found single Rust crate at: {:?}", component.path);
            let mut components = HashMap::new();
            components.insert(component.name.clone(), component.clone());

            let mut languages = HashSet::new();
            languages.insert("Rust".to_string());

            return Ok(Some(WorkspaceInfo {
                root_path: component.path.clone(),
                manifest_paths: vec![component.manifest_path.clone()],
                components,
                workspace_type: WorkspaceType::Rust,
                languages,
            }));
        }

        Ok(None)
    }

    /// Detect Python workspace (pyproject.toml, setup.py, requirements.txt)
    async fn detect_python_workspace(
        start_path: &Path,
    ) -> Result<Option<WorkspaceInfo>, AnalysisError> {
        let mut current = start_path.to_path_buf();

        loop {
            // Check for Python project files
            let pyproject_path = current.join("pyproject.toml");
            let setup_py_path = current.join("setup.py");
            let requirements_path = current.join("requirements.txt");

            if pyproject_path.exists() || setup_py_path.exists() || requirements_path.exists() {
                info!("Found Python project at: {:?}", current);
                return Self::analyze_python_project(&current).await.map(Some);
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

    /// Detect JavaScript/TypeScript workspace (package.json)
    async fn detect_javascript_workspace(
        start_path: &Path,
    ) -> Result<Option<WorkspaceInfo>, AnalysisError> {
        let mut current = start_path.to_path_buf();

        loop {
            let package_json_path = current.join("package.json");
            let tsconfig_path = current.join("tsconfig.json");

            if package_json_path.exists() {
                info!("Found JavaScript/TypeScript project at: {:?}", current);
                let is_typescript = tsconfig_path.exists();
                return Self::analyze_javascript_project(&current, is_typescript)
                    .await
                    .map(Some);
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

    /// Detect mixed-language workspace
    async fn detect_mixed_workspace(
        start_path: &Path,
    ) -> Result<Option<WorkspaceInfo>, AnalysisError> {
        let mut components = HashMap::new();
        let mut manifest_paths = Vec::new();
        let mut languages = HashSet::new();

        // Scan for multiple language indicators in the same directory tree
        let current = start_path.to_path_buf();

        // Look for combinations of language manifests
        let cargo_path = current.join("Cargo.toml");
        let package_json_path = current.join("package.json");
        let pyproject_path = current.join("pyproject.toml");

        let mut found_languages = 0;

        if cargo_path.exists() {
            found_languages += 1;
            languages.insert("Rust".to_string());
            manifest_paths.push(cargo_path.clone());
        }

        if package_json_path.exists() {
            found_languages += 1;
            let tsconfig_path = current.join("tsconfig.json");
            if tsconfig_path.exists() {
                languages.insert("TypeScript".to_string());
            } else {
                languages.insert("JavaScript".to_string());
            }
            manifest_paths.push(package_json_path.clone());
        }

        if pyproject_path.exists() {
            found_languages += 1;
            languages.insert("Python".to_string());
            manifest_paths.push(pyproject_path.clone());
        }

        // If we found multiple languages, create a mixed workspace
        if found_languages > 1 {
            info!(
                "Found mixed-language workspace with {:?} at: {:?}",
                languages, current
            );

            // Create a synthetic component for the mixed workspace
            let mixed_component = ProjectComponent {
                name: current
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("mixed-workspace")
                    .to_string(),
                path: current.clone(),
                manifest_path: manifest_paths[0].clone(), // Use first manifest as primary
                dependencies: Vec::new(),
                is_root: true,
                source_dirs: Self::detect_source_dirs(&current),
                component_type: ComponentType::MixedLanguage,
            };

            components.insert(mixed_component.name.clone(), mixed_component);

            return Ok(Some(WorkspaceInfo {
                root_path: current,
                manifest_paths,
                components,
                workspace_type: WorkspaceType::Mixed,
                languages,
            }));
        }

        Ok(None)
    }

    /// Create a loose file workspace for directories without clear project structure
    async fn create_loose_file_workspace(
        start_path: &Path,
    ) -> Result<Option<WorkspaceInfo>, AnalysisError> {
        // Use the file discovery to check if there are any supported source files
        let discovered_files =
            crate::cli::analyze_command::AnalyzeCommand::discover_files_recursive(start_path)
                .map_err(|e| {
                    AnalysisError::workspace_discovery_error(
                        start_path.display().to_string(),
                        &format!("Failed to discover files: {}", e),
                    )
                })?;

        let mut languages = HashSet::new();

        // Detect languages from file extensions
        for file_path in &discovered_files {
            if let Some(extension) = file_path.extension() {
                let ext = extension.to_string_lossy().to_lowercase();
                match ext.as_str() {
                    "rs" => {
                        languages.insert("Rust".to_string());
                    }
                    "py" => {
                        languages.insert("Python".to_string());
                    }
                    "js" | "jsx" => {
                        languages.insert("JavaScript".to_string());
                    }
                    "ts" | "tsx" => {
                        languages.insert("TypeScript".to_string());
                    }
                    "java" => {
                        languages.insert("Java".to_string());
                    }
                    "cpp" | "c" | "h" | "hpp" => {
                        languages.insert("C++".to_string());
                    }
                    _ => {}
                }
            }
        }

        if !languages.is_empty() {
            info!(
                "Creating loose file workspace with languages {:?} at: {:?}",
                languages, start_path
            );

            let loose_component = ProjectComponent {
                name: start_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("loose-files")
                    .to_string(),
                path: start_path.to_path_buf(),
                manifest_path: start_path.to_path_buf(), // No specific manifest
                dependencies: Vec::new(),
                is_root: true,
                source_dirs: vec![start_path.to_path_buf()],
                component_type: if languages.len() > 1 {
                    ComponentType::MixedLanguage
                } else {
                    ComponentType::MixedLanguage // Default for loose files
                },
            };

            let mut components = HashMap::new();
            components.insert(loose_component.name.clone(), loose_component);

            let workspace_type = if languages.len() > 1 {
                WorkspaceType::Mixed
            } else if languages.contains("Rust") {
                WorkspaceType::Rust
            } else if languages.contains("Python") {
                WorkspaceType::Python
            } else if languages.contains("JavaScript") || languages.contains("TypeScript") {
                WorkspaceType::JavaScript
            } else {
                WorkspaceType::Unknown
            };

            return Ok(Some(WorkspaceInfo {
                root_path: start_path.to_path_buf(),
                manifest_paths: Vec::new(),
                components,
                workspace_type,
                languages,
            }));
        }

        Ok(None)
    }

    /// Find workspace root by walking up directory tree
    async fn find_workspace_root(start_path: &Path) -> Result<Option<PathBuf>, AnalysisError> {
        let mut current = start_path.to_path_buf();

        loop {
            let manifest_path = current.join("Cargo.toml");

            if manifest_path.exists() {
                // Read and parse the Cargo.toml to check if it's a workspace root
                match Self::parse_cargo_manifest(&manifest_path).await {
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

    /// Analyze a workspace and discover all member crates
    async fn analyze_workspace(workspace_root: &Path) -> Result<WorkspaceInfo, AnalysisError> {
        let manifest_path = workspace_root.join("Cargo.toml");
        let manifest = Self::parse_cargo_manifest(&manifest_path).await?;

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
                match Self::analyze_rust_member_crate(&member_path).await {
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
            manifest_paths: vec![manifest_path],
            components: crates,
            workspace_type: WorkspaceType::Rust,
            languages: {
                let mut langs = HashSet::new();
                langs.insert("Rust".to_string());
                langs
            },
        })
    }

    /// Recursively collect all Rust files from a directory
    pub async fn collect_rust_files(dir: &Path) -> Result<Vec<PathBuf>, AnalysisError> {
        let mut files = Vec::new();
        let mut read_dir = async_fs::read_dir(dir)
            .await
            .map_err(|e| AnalysisError::file_system_error(dir.display().to_string(), e))?;

        while let Some(entry) = read_dir
            .next_entry()
            .await
            .map_err(|e| AnalysisError::file_system_error(dir.display().to_string(), e))?
        {
            let path = entry.path();
            let file_type = entry
                .file_type()
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

    /// New multi-language workspace type detection entry point
    pub fn detect_workspace_type(path: &Path) -> WorkspaceType {
        if path.join("Cargo.toml").exists() {
            return WorkspaceType::Rust;
        }
        if Self::has_python_workspace(path) {
            return WorkspaceType::Python;
        }
        if Self::has_typescript_workspace(path) {
            return WorkspaceType::TypeScript;
        }
        if Self::has_javascript_workspace(path) {
            return WorkspaceType::JavaScript;
        }
        WorkspaceType::Unknown
    }

    pub fn has_python_workspace(path: &Path) -> bool {
        [
            "pyproject.toml",
            "setup.py",
            "requirements.txt",
            "Pipfile",
            "poetry.lock",
        ]
        .iter()
        .any(|f| path.join(f).exists())
    }
    pub fn has_typescript_workspace(path: &Path) -> bool {
        if path.join("tsconfig.json").exists() {
            return true;
        }
        if path.join("package.json").exists() {
            return Self::has_typescript_deps(path);
        }
        false
    }
    pub fn has_javascript_workspace(path: &Path) -> bool {
        path.join("package.json").exists()
    }
    fn has_typescript_deps(path: &Path) -> bool {
        let pkg = path.join("package.json");
        if !pkg.exists() {
            return false;
        }
        if let Ok(content) = std::fs::read_to_string(&pkg) {
            return content.contains("typescript");
        }
        false
    }

    /// Derive overall workspace type by scanning immediate children (detect Mixed)
    pub fn detect_composite_workspace_type(root: &Path) -> WorkspaceType {
        let mut types: HashSet<WorkspaceType> = HashSet::new();
        if let Ok(entries) = std::fs::read_dir(root) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    let t = Self::detect_workspace_type(&p);
                    if t != WorkspaceType::Unknown {
                        types.insert(t);
                    }
                }
            }
        }
        if types.is_empty() {
            return Self::detect_workspace_type(root);
        }
        if types.len() > 1 {
            WorkspaceType::Mixed
        } else {
            types.into_iter().next().unwrap_or(WorkspaceType::Unknown)
        }
    }

    /// Find Cargo workspace root (Rust specific)
    async fn find_cargo_workspace_root(
        start_path: &Path,
    ) -> Result<Option<PathBuf>, AnalysisError> {
        let mut current = start_path.to_path_buf();

        loop {
            let manifest_path = current.join("Cargo.toml");

            if manifest_path.exists() {
                // Read and parse the Cargo.toml to check if it's a workspace root
                match Self::parse_cargo_manifest(&manifest_path).await {
                    Ok(manifest) => {
                        if manifest.workspace.is_some() {
                            return Ok(Some(current));
                        }
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

    /// Detect single Rust crate
    async fn detect_single_crate(path: &Path) -> Result<Option<ProjectComponent>, AnalysisError> {
        let manifest_path = path.join("Cargo.toml");

        if !manifest_path.exists() {
            return Ok(None);
        }

        let manifest = Self::parse_cargo_manifest(&manifest_path).await?;

        // If it has workspace info, it's not a single crate
        if manifest.workspace.is_some() {
            return Ok(None);
        }

        // If it has package info, it's a single crate
        if let Some(package) = manifest.package {
            let dependencies = manifest
                .dependencies
                .unwrap_or_default()
                .keys()
                .map(|k| k.clone())
                .collect();

            let source_dirs = vec![path.join("src")];

            return Ok(Some(ProjectComponent {
                name: package.name,
                path: path.to_path_buf(),
                manifest_path,
                dependencies,
                is_root: true,
                source_dirs,
                component_type: ComponentType::RustCrate,
            }));
        }

        Ok(None)
    }

    /// Analyze Rust workspace
    async fn analyze_rust_workspace(workspace_root: &Path) -> Result<WorkspaceInfo, AnalysisError> {
        let manifest_path = workspace_root.join("Cargo.toml");
        let manifest = Self::parse_cargo_manifest(&manifest_path).await?;

        let workspace_manifest = manifest.workspace.ok_or_else(|| {
            AnalysisError::workspace_discovery_error(
                workspace_root.display().to_string(),
                "Cargo.toml does not contain workspace configuration",
            )
        })?;

        let members = workspace_manifest.members.unwrap_or_default();
        let mut components = HashMap::new();
        let mut languages = HashSet::new();
        languages.insert("Rust".to_string());

        // Analyze each workspace member
        for member_pattern in &members {
            let member_paths = Self::expand_member_pattern(workspace_root, member_pattern)?;

            for member_path in member_paths {
                match Self::analyze_rust_member_crate(&member_path).await {
                    Ok(component) => {
                        components.insert(component.name.clone(), component);
                    }
                    Err(e) => {
                        warn!("Failed to analyze member crate at {:?}: {}", member_path, e);
                    }
                }
            }
        }

        Ok(WorkspaceInfo {
            root_path: workspace_root.to_path_buf(),
            manifest_paths: vec![manifest_path],
            components,
            workspace_type: WorkspaceType::Rust,
            languages,
        })
    }

    /// Analyze Python project
    async fn analyze_python_project(project_root: &Path) -> Result<WorkspaceInfo, AnalysisError> {
        let mut manifest_paths = Vec::new();
        let mut dependencies = Vec::new();
        let mut project_name = project_root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("python-project")
            .to_string();

        // Check for pyproject.toml
        let pyproject_path = project_root.join("pyproject.toml");
        if pyproject_path.exists() {
            manifest_paths.push(pyproject_path.clone());
            if let Ok(content) = std::fs::read_to_string(&pyproject_path) {
                if let Ok(manifest) = toml::from_str::<PyProjectManifest>(&content) {
                    if let Some(project) = manifest.project {
                        if let Some(name) = project.name {
                            project_name = name;
                        }
                        if let Some(deps) = project.dependencies {
                            dependencies.extend(deps);
                        }
                    }
                }
            }
        }

        // Check for setup.py
        let setup_py_path = project_root.join("setup.py");
        if setup_py_path.exists() {
            manifest_paths.push(setup_py_path);
        }

        // Check for requirements.txt
        let requirements_path = project_root.join("requirements.txt");
        if requirements_path.exists() {
            manifest_paths.push(requirements_path);
        }

        let component = ProjectComponent {
            name: project_name.clone(),
            path: project_root.to_path_buf(),
            manifest_path: manifest_paths
                .get(0)
                .cloned()
                .unwrap_or_else(|| project_root.to_path_buf()),
            dependencies,
            is_root: true,
            source_dirs: Self::detect_python_source_dirs(project_root),
            component_type: ComponentType::PythonPackage,
        };

        let mut components = HashMap::new();
        components.insert(component.name.clone(), component);

        let mut languages = HashSet::new();
        languages.insert("Python".to_string());

        Ok(WorkspaceInfo {
            root_path: project_root.to_path_buf(),
            manifest_paths,
            components,
            workspace_type: WorkspaceType::Python,
            languages,
        })
    }

    /// Analyze JavaScript/TypeScript project
    async fn analyze_javascript_project(
        project_root: &Path,
        is_typescript: bool,
    ) -> Result<WorkspaceInfo, AnalysisError> {
        let package_json_path = project_root.join("package.json");
        let mut dependencies = Vec::new();
        let mut project_name = project_root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("js-project")
            .to_string();

        if let Ok(content) = std::fs::read_to_string(&package_json_path) {
            if let Ok(manifest) = serde_json::from_str::<PackageJsonManifest>(&content) {
                if let Some(name) = manifest.name {
                    project_name = name;
                }
                if let Some(deps) = manifest.dependencies {
                    dependencies.extend(deps.keys().cloned());
                }
                if let Some(dev_deps) = manifest.dev_dependencies {
                    dependencies.extend(dev_deps.keys().cloned());
                }
            }
        }

        let component = ProjectComponent {
            name: project_name.clone(),
            path: project_root.to_path_buf(),
            manifest_path: package_json_path.clone(),
            dependencies,
            is_root: true,
            source_dirs: Self::detect_js_source_dirs(project_root),
            component_type: if is_typescript {
                ComponentType::TypeScriptPackage
            } else {
                ComponentType::JavaScriptPackage
            },
        };

        let mut components = HashMap::new();
        components.insert(component.name.clone(), component);

        let mut languages = HashSet::new();
        languages.insert(
            if is_typescript {
                "TypeScript"
            } else {
                "JavaScript"
            }
            .to_string(),
        );

        Ok(WorkspaceInfo {
            root_path: project_root.to_path_buf(),
            manifest_paths: vec![package_json_path],
            components,
            workspace_type: if is_typescript {
                WorkspaceType::TypeScript
            } else {
                WorkspaceType::JavaScript
            },
            languages,
        })
    }

    /// Parse Cargo.toml manifest
    async fn parse_cargo_manifest(manifest_path: &Path) -> Result<CargoManifest, AnalysisError> {
        let content = async_fs::read_to_string(manifest_path).await.map_err(|e| {
            AnalysisError::file_system_error(manifest_path.display().to_string(), e)
        })?;

        toml::from_str(&content).map_err(|e| {
            AnalysisError::workspace_discovery_error(
                manifest_path.display().to_string(),
                format!("Failed to parse TOML: {}", e),
            )
        })
    }

    /// Get all source files from a workspace
    pub async fn get_workspace_source_files(
        workspace: &WorkspaceInfo,
    ) -> Result<Vec<PathBuf>, AnalysisError> {
        let mut all_files = Vec::new();

        for component in workspace.components.values() {
            for source_dir in &component.source_dirs {
                if source_dir.exists() {
                    let files = Self::collect_rust_files(source_dir).await?;
                    all_files.extend(files);
                }
            }
        }

        Ok(all_files)
    }

    /// Analyze Rust member crate
    async fn analyze_rust_member_crate(
        crate_path: &Path,
    ) -> Result<ProjectComponent, AnalysisError> {
        let manifest_path = crate_path.join("Cargo.toml");
        let manifest = Self::parse_cargo_manifest(&manifest_path).await?;

        let package = manifest.package.ok_or_else(|| {
            AnalysisError::workspace_discovery_error(
                crate_path.display().to_string(),
                "Cargo.toml does not contain package information",
            )
        })?;

        let dependencies = manifest
            .dependencies
            .unwrap_or_default()
            .keys()
            .map(|k| k.clone())
            .collect();

        let source_dirs = vec![crate_path.join("src")];

        Ok(ProjectComponent {
            name: package.name,
            path: crate_path.to_path_buf(),
            manifest_path,
            dependencies,
            is_root: false,
            source_dirs,
            component_type: ComponentType::RustCrate,
        })
    }

    /// Detect source directories for different project types
    fn detect_source_dirs(project_root: &Path) -> Vec<PathBuf> {
        let mut source_dirs = Vec::new();

        // Common source directory patterns
        let common_patterns = ["src", "lib", "source", "sources"];

        for pattern in &common_patterns {
            let dir = project_root.join(pattern);
            if dir.exists() && dir.is_dir() {
                source_dirs.push(dir);
            }
        }

        // If no standard source dirs found, use the project root
        if source_dirs.is_empty() {
            source_dirs.push(project_root.to_path_buf());
        }

        source_dirs
    }

    /// Detect Python source directories
    fn detect_python_source_dirs(project_root: &Path) -> Vec<PathBuf> {
        let mut source_dirs = Vec::new();

        // Python-specific patterns
        let python_patterns = [
            "src",
            "lib",
            project_root
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(""),
        ];

        for pattern in &python_patterns {
            if !pattern.is_empty() {
                let dir = project_root.join(pattern);
                if dir.exists() && dir.is_dir() {
                    source_dirs.push(dir);
                }
            }
        }

        // Check for package directories (directories with __init__.py)
        if let Ok(entries) = std::fs::read_dir(project_root) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_dir() && path.join("__init__.py").exists() {
                    source_dirs.push(path);
                }
            }
        }

        if source_dirs.is_empty() {
            source_dirs.push(project_root.to_path_buf());
        }

        source_dirs
    }

    /// Detect JavaScript/TypeScript source directories
    fn detect_js_source_dirs(project_root: &Path) -> Vec<PathBuf> {
        let mut source_dirs = Vec::new();

        // JS/TS-specific patterns
        let js_patterns = ["src", "lib", "source", "app", "pages", "components"];

        for pattern in &js_patterns {
            let dir = project_root.join(pattern);
            if dir.exists() && dir.is_dir() {
                source_dirs.push(dir);
            }
        }

        if source_dirs.is_empty() {
            source_dirs.push(project_root.to_path_buf());
        }

        source_dirs
    }

    /// Expand workspace member patterns (handle globs like "crates/*")
    fn expand_member_pattern(
        workspace_root: &Path,
        pattern: &str,
    ) -> Result<Vec<PathBuf>, AnalysisError> {
        let pattern_path = workspace_root.join(pattern);

        // If the pattern contains wildcards, expand them
        if pattern.contains('*') {
            // For now, implement basic wildcard support
            // In a full implementation, you'd use the glob crate
            let parent = pattern_path.parent().unwrap_or(workspace_root);
            let mut result = Vec::new();

            if let Ok(entries) = std::fs::read_dir(parent) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.is_dir() && path.join("Cargo.toml").exists() {
                        result.push(path);
                    }
                }
            }

            Ok(result)
        } else {
            // Direct path
            if pattern_path.exists() && pattern_path.join("Cargo.toml").exists() {
                Ok(vec![pattern_path])
            } else {
                Ok(Vec::new())
            }
        }
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
        write(crate_path.join("Cargo.toml"), manifest_content)
            .await
            .unwrap();

        // Create src directory
        tokio::fs::create_dir(crate_path.join("src")).await.unwrap();

        let workspace = WorkspaceDetector::detect_workspace(crate_path)
            .await
            .unwrap();
        assert!(workspace.is_some());

        let workspace = workspace.unwrap();
        assert_eq!(workspace.components.len(), 1);
        assert!(workspace.components.contains_key("test_crate"));
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
        write(workspace_path.join("Cargo.toml"), workspace_manifest)
            .await
            .unwrap();

        // Create member crates
        for crate_name in &["crate_a", "crate_b"] {
            let crate_path = workspace_path.join(crate_name);
            tokio::fs::create_dir_all(&crate_path).await.unwrap();

            let crate_manifest = format!(
                r#"
[package]
name = "{}"
version = "0.1.0"
"#,
                crate_name
            );
            write(crate_path.join("Cargo.toml"), crate_manifest)
                .await
                .unwrap();
            tokio::fs::create_dir(crate_path.join("src")).await.unwrap();
        }

        let workspace = WorkspaceDetector::detect_workspace(workspace_path)
            .await
            .unwrap();
        assert!(workspace.is_some());

        let workspace = workspace.unwrap();
        assert_eq!(workspace.components.len(), 2);
        assert!(workspace.components.contains_key("crate_a"));
        assert!(workspace.components.contains_key("crate_b"));
    }
}

#[cfg(test)]
mod multi_lang_tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_composite_workspace_type_single() {
        let temp = TempDir::new().unwrap();
        std::fs::write(
            temp.path().join("Cargo.toml"),
            "[package]\nname='x'\nversion='0.1.0'\n",
        )
        .unwrap();
        let t = WorkspaceDetector::detect_composite_workspace_type(temp.path());
        assert_eq!(t, WorkspaceType::Rust);
    }

    #[test]
    fn test_composite_workspace_type_mixed() {
        let temp = TempDir::new().unwrap();
        // Rust subdir
        std::fs::create_dir(temp.path().join("rust_mod")).unwrap();
        std::fs::write(
            temp.path().join("rust_mod").join("Cargo.toml"),
            "[package]\nname='x'\nversion='0.1.0'\n",
        )
        .unwrap();
        // Python subdir
        std::fs::create_dir(temp.path().join("py_mod")).unwrap();
        std::fs::write(
            temp.path().join("py_mod").join("pyproject.toml"),
            "[project]\nname='y'\nversion='0.1.0'\n",
        )
        .unwrap();
        let t = WorkspaceDetector::detect_composite_workspace_type(temp.path());
        assert_eq!(t, WorkspaceType::Mixed);
    }
}
