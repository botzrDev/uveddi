use crate::analysis::detectors::dependency::types::*;
use super::LanguageDependencyParser;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;

#[derive(Debug, Deserialize)]
pub struct CargoManifest {
    pub package: Option<CargoPackage>,
    pub dependencies: Option<HashMap<String, CargoDependency>>,
    #[serde(rename = "dev-dependencies")]
    pub dev_dependencies: Option<HashMap<String, CargoDependency>>,
    #[serde(rename = "build-dependencies")]
    pub build_dependencies: Option<HashMap<String, CargoDependency>>,
    pub workspace: Option<CargoWorkspace>,
}

#[derive(Debug, Deserialize)]
pub struct CargoPackage {
    pub name: String,
    pub version: String,
    pub authors: Option<Vec<String>>,
    pub description: Option<String>,
    pub license: Option<String>,
    pub repository: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum CargoDependency {
    Simple(String),
    Detailed {
        version: Option<String>,
        git: Option<String>,
        branch: Option<String>,
        tag: Option<String>,
        rev: Option<String>,
        path: Option<String>,
        registry: Option<String>,
        optional: Option<bool>,
        features: Option<Vec<String>>,
        #[serde(rename = "default-features")]
        default_features: Option<bool>,
    },
}

#[derive(Debug, Deserialize)]
pub struct CargoWorkspace {
    pub members: Option<Vec<String>>,
    pub dependencies: Option<HashMap<String, CargoDependency>>,
}

#[derive(Debug, Deserialize)]
pub struct CargoLockfile {
    pub version: u32,
    #[serde(rename = "package")]
    pub packages: Vec<CargoLockPackage>,
}

#[derive(Debug, Deserialize)]
pub struct CargoLockPackage {
    pub name: String,
    pub version: String,
    pub source: Option<String>,
    pub checksum: Option<String>,
    pub dependencies: Option<Vec<String>>,
}

pub struct RustDependencyParser {
    registry_url: String,
}

impl RustDependencyParser {
    pub fn new() -> Self {
        Self {
            registry_url: "https://crates.io".to_string(),
        }
    }

    fn parse_cargo_dependency(
        &self,
        name: &str,
        dep: &CargoDependency,
        scope: DependencyScope,
    ) -> DependencyInfo {
        match dep {
            CargoDependency::Simple(version) => DependencyInfo {
                name: name.to_string(),
                version: Some(version.clone()),
                source: DependencySource::Registry(self.registry_url.clone()),
                scope,
                resolved_path: None,
            },
            CargoDependency::Detailed {
                version,
                git,
                path,
                registry,
                optional,
                ..
            } => {
                let source = if let Some(git_url) = git {
                    DependencySource::Git {
                        url: git_url.clone(),
                        branch: None, // Could extract from branch/tag/rev fields
                    }
                } else if let Some(local_path) = path {
                    DependencySource::Local(PathBuf::from(local_path))
                } else if let Some(reg) = registry {
                    DependencySource::Registry(reg.clone())
                } else {
                    DependencySource::Registry(self.registry_url.clone())
                };

                let final_scope = if optional.unwrap_or(false) {
                    DependencyScope::Optional
                } else {
                    scope
                };

                DependencyInfo {
                    name: name.to_string(),
                    version: version.clone(),
                    source,
                    scope: final_scope,
                    resolved_path: None,
                }
            }
        }
    }

    fn parse_version_constraint(version: &str) -> String {
        // Handle Cargo version constraints like "^1.0", "~1.2", ">=1.0,<2.0"
        if version.starts_with('^') || version.starts_with('~') {
            version.to_string()
        } else if version.contains(',') {
            // Multiple constraints
            version.to_string()
        } else if version.starts_with(">=") || version.starts_with("<=") ||
                 version.starts_with('>') || version.starts_with('<') {
            version.to_string()
        } else {
            // Exact version or simple version
            version.to_string()
        }
    }
}

impl Default for RustDependencyParser {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageDependencyParser for RustDependencyParser {
    fn parse_manifest(
        &self,
        manifest_path: &Path,
    ) -> Result<Vec<DependencyInfo>, DependencyError> {
        let content = fs::read_to_string(manifest_path)
            .map_err(|e| DependencyError::ParseError(format!("Failed to read Cargo.toml: {}", e)))?;

        let manifest: CargoManifest = toml::from_str(&content)
            .map_err(|e| DependencyError::ParseError(format!("Failed to parse Cargo.toml: {}", e)))?;

        let mut dependencies = Vec::new();

        // Parse production dependencies
        if let Some(deps) = &manifest.dependencies {
            for (name, dep) in deps {
                dependencies.push(self.parse_cargo_dependency(name, dep, DependencyScope::Production));
            }
        }

        // Parse development dependencies
        if let Some(dev_deps) = &manifest.dev_dependencies {
            for (name, dep) in dev_deps {
                dependencies.push(self.parse_cargo_dependency(name, dep, DependencyScope::Development));
            }
        }

        // Parse build dependencies
        if let Some(build_deps) = &manifest.build_dependencies {
            for (name, dep) in build_deps {
                dependencies.push(self.parse_cargo_dependency(name, dep, DependencyScope::Build));
            }
        }

        // Parse workspace dependencies if present
        if let Some(workspace) = &manifest.workspace {
            if let Some(workspace_deps) = &workspace.dependencies {
                for (name, dep) in workspace_deps {
                    dependencies.push(self.parse_cargo_dependency(name, dep, DependencyScope::Production));
                }
            }
        }

        Ok(dependencies)
    }

    fn parse_lockfile(
        &self,
        lockfile_path: &Path,
    ) -> Result<Vec<DependencyInfo>, DependencyError> {
        let content = fs::read_to_string(lockfile_path)
            .map_err(|e| DependencyError::ParseError(format!("Failed to read Cargo.lock: {}", e)))?;

        let lockfile: CargoLockfile = toml::from_str(&content)
            .map_err(|e| DependencyError::ParseError(format!("Failed to parse Cargo.lock: {}", e)))?;

        let mut dependencies = Vec::new();

        for package in lockfile.packages {
            let source = if let Some(source_str) = package.source {
                if source_str.starts_with("registry+") {
                    DependencySource::Registry(self.registry_url.clone())
                } else if source_str.starts_with("git+") {
                    // Parse git URL from source string
                    let url = source_str.strip_prefix("git+").unwrap_or(&source_str);
                    DependencySource::Git {
                        url: url.split('#').next().unwrap_or(url).to_string(),
                        branch: None,
                    }
                } else {
                    DependencySource::Unknown
                }
            } else {
                // Local package (likely the root package or path dependency)
                DependencySource::Unknown
            };

            dependencies.push(DependencyInfo {
                name: package.name,
                version: Some(package.version),
                source,
                scope: DependencyScope::Production, // Lockfile doesn't distinguish scopes
                resolved_path: None,
            });
        }

        Ok(dependencies)
    }

    fn supported_manifests(&self) -> Vec<&'static str> {
        vec!["Cargo.toml"]
    }

    fn supported_lockfiles(&self) -> Vec<&'static str> {
        vec!["Cargo.lock"]
    }

    fn language_name(&self) -> &'static str {
        "Rust"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    #[test]
    fn test_parse_simple_cargo_toml() {
        let parser = RustDependencyParser::new();
        let mut temp_file = NamedTempFile::new().unwrap();

        let content = r#"
[package]
name = "test-crate"
version = "0.1.0"

[dependencies]
serde = "1.0"
tokio = { version = "1.0", features = ["full"] }
log = { version = "0.4", optional = true }

[dev-dependencies]
criterion = "0.5"
"#;

        std::fs::write(temp_file.path(), content).unwrap();

        let result = parser.parse_manifest(temp_file.path()).unwrap();
        assert_eq!(result.len(), 4);

        // Check serde dependency
        let serde_dep = result.iter().find(|d| d.name == "serde").unwrap();
        assert_eq!(serde_dep.version, Some("1.0".to_string()));
        assert_eq!(serde_dep.scope, DependencyScope::Production);

        // Check optional dependency
        let log_dep = result.iter().find(|d| d.name == "log").unwrap();
        assert_eq!(log_dep.scope, DependencyScope::Optional);

        // Check dev dependency
        let criterion_dep = result.iter().find(|d| d.name == "criterion").unwrap();
        assert_eq!(criterion_dep.scope, DependencyScope::Development);
    }
}