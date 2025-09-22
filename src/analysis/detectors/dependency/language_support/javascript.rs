use super::LanguageDependencyParser;
use crate::analysis::detectors::dependency::types::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
pub struct PackageJson {
    pub name: Option<String>,
    pub version: Option<String>,
    pub dependencies: Option<HashMap<String, String>>,
    #[serde(rename = "devDependencies")]
    pub dev_dependencies: Option<HashMap<String, String>>,
    #[serde(rename = "peerDependencies")]
    pub peer_dependencies: Option<HashMap<String, String>>,
    #[serde(rename = "optionalDependencies")]
    pub optional_dependencies: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct PackageLockJson {
    pub name: String,
    pub version: String,
    #[serde(rename = "lockfileVersion")]
    pub lockfile_version: i32,
    pub dependencies: Option<HashMap<String, LockDependency>>,
    pub packages: Option<HashMap<String, LockPackage>>,
}

#[derive(Debug, Deserialize)]
pub struct LockDependency {
    pub version: String,
    pub resolved: Option<String>,
    pub dev: Option<bool>,
    pub optional: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct LockPackage {
    pub version: Option<String>,
    pub resolved: Option<String>,
    pub dev: Option<bool>,
    pub optional: Option<bool>,
}

pub struct JavaScriptDependencyParser {
    default_registry: String,
}

impl JavaScriptDependencyParser {
    pub fn new() -> Self {
        Self {
            default_registry: "https://registry.npmjs.org/".to_string(),
        }
    }

    fn parse_npm_version(&self, version_spec: &str) -> String {
        match version_spec {
            "latest" | "*" => version_spec.to_string(),
            spec if spec.starts_with("npm:") => spec.to_string(),
            spec if spec.starts_with("file:") || spec.starts_with("link:") => spec.to_string(),
            spec if spec.contains("://") => spec.to_string(),
            _ => version_spec.to_string(),
        }
    }

    fn determine_dependency_source(&self, version_spec: &str) -> DependencySource {
        if version_spec.starts_with("git+")
            || version_spec.contains("github.com")
            || version_spec.contains("gitlab.com")
        {
            let url = if version_spec.starts_with("git+") {
                version_spec.strip_prefix("git+").unwrap_or(version_spec)
            } else {
                version_spec
            };

            let (clean_url, branch) = if let Some(fragment_pos) = url.find('#') {
                let (base_url, fragment) = url.split_at(fragment_pos);
                let branch_name = fragment.strip_prefix('#').unwrap_or(fragment);
                (base_url.to_string(), Some(branch_name.to_string()))
            } else {
                (url.to_string(), None)
            };

            DependencySource::Git {
                url: clean_url,
                branch,
            }
        } else if version_spec.starts_with("file:") || version_spec.starts_with("link:") {
            let path = version_spec
                .strip_prefix("file:")
                .or_else(|| version_spec.strip_prefix("link:"))
                .unwrap_or(version_spec);
            DependencySource::Local(PathBuf::from(path))
        } else if version_spec.contains("://") {
            DependencySource::Registry(version_spec.to_string())
        } else {
            DependencySource::Registry(self.default_registry.clone())
        }
    }

    fn create_dependency_info(
        &self,
        name: &str,
        version_spec: &str,
        scope: DependencyScope,
    ) -> DependencyInfo {
        let source = self.determine_dependency_source(version_spec);
        let parsed_version = self.parse_npm_version(version_spec);

        DependencyInfo {
            name: name.to_string(),
            version: Some(parsed_version),
            source,
            scope,
            resolved_path: None,
        }
    }

    fn parse_package_json(&self, path: &Path) -> Result<Vec<DependencyInfo>, DependencyError> {
        let content = fs::read_to_string(path).map_err(|e| {
            DependencyError::ParseError(format!("Failed to read package.json: {}", e))
        })?;

        let package_json: PackageJson = serde_json::from_str(&content).map_err(|e| {
            DependencyError::ParseError(format!("Failed to parse package.json: {}", e))
        })?;

        let mut dependencies = Vec::new();

        if let Some(deps) = &package_json.dependencies {
            for (name, version) in deps {
                dependencies.push(self.create_dependency_info(
                    name,
                    version,
                    DependencyScope::Production,
                ));
            }
        }

        if let Some(dev_deps) = &package_json.dev_dependencies {
            for (name, version) in dev_deps {
                dependencies.push(self.create_dependency_info(
                    name,
                    version,
                    DependencyScope::Development,
                ));
            }
        }

        if let Some(peer_deps) = &package_json.peer_dependencies {
            for (name, version) in peer_deps {
                dependencies.push(self.create_dependency_info(
                    name,
                    version,
                    DependencyScope::Production,
                ));
            }
        }

        if let Some(optional_deps) = &package_json.optional_dependencies {
            for (name, version) in optional_deps {
                dependencies.push(self.create_dependency_info(
                    name,
                    version,
                    DependencyScope::Optional,
                ));
            }
        }

        Ok(dependencies)
    }

    fn parse_package_lock_json(&self, path: &Path) -> Result<Vec<DependencyInfo>, DependencyError> {
        let content = fs::read_to_string(path).map_err(|e| {
            DependencyError::ParseError(format!("Failed to read package-lock.json: {}", e))
        })?;

        let lock_file: PackageLockJson = serde_json::from_str(&content).map_err(|e| {
            DependencyError::ParseError(format!("Failed to parse package-lock.json: {}", e))
        })?;

        let mut dependencies = Vec::new();

        if let Some(deps) = &lock_file.dependencies {
            for (name, dep_info) in deps {
                let scope = if dep_info.dev.unwrap_or(false) {
                    DependencyScope::Development
                } else if dep_info.optional.unwrap_or(false) {
                    DependencyScope::Optional
                } else {
                    DependencyScope::Production
                };

                let source = if let Some(resolved_url) = &dep_info.resolved {
                    if resolved_url.contains("github.com") || resolved_url.contains("gitlab.com") {
                        DependencySource::Git {
                            url: resolved_url.clone(),
                            branch: None,
                        }
                    } else {
                        DependencySource::Registry(self.default_registry.clone())
                    }
                } else {
                    DependencySource::Registry(self.default_registry.clone())
                };

                dependencies.push(DependencyInfo {
                    name: name.clone(),
                    version: Some(dep_info.version.clone()),
                    source,
                    scope,
                    resolved_path: None,
                });
            }
        }

        if let Some(packages) = &lock_file.packages {
            for (package_path, package_info) in packages {
                if package_path.is_empty() {
                    continue;
                }

                let name = if package_path.starts_with("node_modules/") {
                    package_path
                        .strip_prefix("node_modules/")
                        .unwrap_or(package_path)
                } else {
                    package_path
                };

                let scope = if package_info.dev.unwrap_or(false) {
                    DependencyScope::Development
                } else if package_info.optional.unwrap_or(false) {
                    DependencyScope::Optional
                } else {
                    DependencyScope::Production
                };

                let source = if let Some(resolved_url) = &package_info.resolved {
                    if resolved_url.contains("github.com") || resolved_url.contains("gitlab.com") {
                        DependencySource::Git {
                            url: resolved_url.clone(),
                            branch: None,
                        }
                    } else {
                        DependencySource::Registry(self.default_registry.clone())
                    }
                } else {
                    DependencySource::Registry(self.default_registry.clone())
                };

                if let Some(version) = &package_info.version {
                    dependencies.push(DependencyInfo {
                        name: name.to_string(),
                        version: Some(version.clone()),
                        source,
                        scope,
                        resolved_path: None,
                    });
                }
            }
        }

        Ok(dependencies)
    }

    fn parse_yarn_lock(&self, path: &Path) -> Result<Vec<DependencyInfo>, DependencyError> {
        let content = fs::read_to_string(path)
            .map_err(|e| DependencyError::ParseError(format!("Failed to read yarn.lock: {}", e)))?;

        let mut dependencies = Vec::new();
        let mut current_package: Option<String> = None;
        let mut current_version: Option<String> = None;
        let mut current_resolved: Option<String> = None;

        for line in content.lines() {
            let line = line.trim();

            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if line.ends_with(':') && !line.starts_with(' ') {
                if let (Some(name), Some(version)) = (&current_package, &current_version) {
                    let source = if let Some(resolved_url) = &current_resolved {
                        if resolved_url.contains("github.com")
                            || resolved_url.contains("gitlab.com")
                        {
                            DependencySource::Git {
                                url: resolved_url.clone(),
                                branch: None,
                            }
                        } else {
                            DependencySource::Registry(self.default_registry.clone())
                        }
                    } else {
                        DependencySource::Registry(self.default_registry.clone())
                    };

                    dependencies.push(DependencyInfo {
                        name: name.clone(),
                        version: Some(version.clone()),
                        source,
                        scope: DependencyScope::Production,
                        resolved_path: None,
                    });
                }

                let package_spec = line.trim_end_matches(':');
                if let Some(at_pos) = package_spec.find('@') {
                    if at_pos > 0 {
                        current_package = Some(package_spec[..at_pos].to_string());
                    } else if let Some(second_at) = package_spec[1..].find('@') {
                        current_package = Some(package_spec[..second_at + 1].to_string());
                    } else {
                        current_package = Some(package_spec.to_string());
                    }
                } else {
                    current_package = Some(package_spec.to_string());
                }
                current_version = None;
                current_resolved = None;
            } else if line.starts_with("version ") {
                current_version = Some(
                    line.strip_prefix("version ")
                        .unwrap_or("")
                        .trim_matches('"')
                        .to_string(),
                );
            } else if line.starts_with("resolved ") {
                current_resolved = Some(
                    line.strip_prefix("resolved ")
                        .unwrap_or("")
                        .trim_matches('"')
                        .to_string(),
                );
            } else if line.trim().is_empty() && current_package.is_some() {
                // End of package entry
                if let (Some(package), version) = (current_package.take(), current_version.take()) {
                    if let Some(package_name) = package.split('@').next() {
                        dependencies.push(DependencyInfo {
                            name: package_name.to_string(),
                            version,
                            source: DependencySource::Registry("npm".to_string()),
                            scope: DependencyScope::Production,
                            resolved_path: current_resolved.take().map(PathBuf::from),
                        });
                    }
                }
            }
        }

        // Handle last package if file doesn't end with empty line
        if let (Some(package), version) = (current_package, current_version) {
            if let Some(package_name) = package.split('@').next() {
                dependencies.push(DependencyInfo {
                    name: package_name.to_string(),
                    version,
                    source: DependencySource::Registry("npm".to_string()),
                    scope: DependencyScope::Production,
                    resolved_path: current_resolved.map(PathBuf::from),
                });
            }
        }

        Ok(dependencies)
    }
}
