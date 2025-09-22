use crate::analysis::detectors::dependency::types::*;
use super::LanguageDependencyParser;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;

#[derive(Debug, Clone)]
pub struct PythonRequirement {
    pub name: String,
    pub version_spec: Option<String>,
    pub extras: Vec<String>,
    pub url: Option<String>,
    pub is_editable: bool,
}

#[derive(Debug, Deserialize)]
pub struct PyprojectToml {
    pub project: Option<PyprojectProject>,
    pub build_system: Option<BuildSystem>,
}

#[derive(Debug, Deserialize)]
pub struct PyprojectProject {
    pub name: String,
    pub dependencies: Option<Vec<String>>,
    #[serde(rename = "optional-dependencies")]
    pub optional_dependencies: Option<HashMap<String, Vec<String>>>,
}

#[derive(Debug, Deserialize)]
pub struct BuildSystem {
    pub requires: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct PipfileTom {
    pub packages: Option<HashMap<String, PipfileDependency>>,
    #[serde(rename = "dev-packages")]
    pub dev_packages: Option<HashMap<String, PipfileDependency>>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum PipfileDependency {
    Simple(String),
    Detailed {
        version: Option<String>,
        git: Option<String>,
        path: Option<String>,
        editable: Option<bool>,
    },
}

pub struct PythonDependencyParser {
    default_index: String,
}

impl PythonDependencyParser {
    pub fn new() -> Self {
        Self {
            default_index: "https://pypi.org/simple/".to_string(),
        }
    }

    fn parse_requirement_line(&self, line: &str) -> Result<PythonRequirement, DependencyError> {
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            return Err(DependencyError::ParseError("Empty or comment line".to_string()));
        }

        let (is_editable, line) = if line.starts_with("-e ") {
            (true, &line[3..])
        } else {
            (false, line)
        };

        if line.contains("://") {
            let name = self.extract_name_from_url(line).unwrap_or_else(|| "unknown".to_string());
            return Ok(PythonRequirement {
                name,
                version_spec: None,
                extras: Vec::new(),
                url: Some(line.to_string()),
                is_editable,
            });
        }

        let (name_and_extras, version_spec) = if let Some(pos) = line.find(&['>', '<', '=', '!', '~'][..]) {
            line.split_at(pos)
        } else {
            (line, "")
        };

        let (name, extras) = if let Some(bracket_pos) = name_and_extras.find('[') {
            let name = name_and_extras[..bracket_pos].trim();
            let extras_str = &name_and_extras[bracket_pos + 1..];
            if let Some(close_bracket) = extras_str.find(']') {
                let extras: Vec<String> = extras_str[..close_bracket]
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect();
                (name, extras)
            } else {
                (name, Vec::new())
            }
        } else {
            (name_and_extras.trim(), Vec::new())
        };

        let version_spec = if version_spec.trim().is_empty() {
            None
        } else {
            Some(version_spec.trim().split(';').next().unwrap_or("").to_string())
        };

        Ok(PythonRequirement {
            name: name.to_string(),
            version_spec,
            extras,
            url: None,
            is_editable,
        })
    }

    fn extract_name_from_url(&self, url: &str) -> Option<String> {
        if url.starts_with("git+") {
            let path = url.split('/').last()?;
            Some(path.strip_suffix(".git").unwrap_or(path).to_string())
        } else {
            let path = url.split('/').last()?;
            Some(path.split('#').next()?.to_string())
        }
    }

    fn parse_pipfile_dependency(
        &self,
        name: &str,
        dep: &PipfileDependency,
        scope: DependencyScope,
    ) -> DependencyInfo {
        match dep {
            PipfileDependency::Simple(version) => DependencyInfo {
                name: name.to_string(),
                version: Some(version.clone()),
                source: DependencySource::Registry(self.default_index.clone()),
                scope,
                resolved_path: None,
            },
            PipfileDependency::Detailed { version, git, path, editable, .. } => {
                let source = if let Some(git_url) = git {
                    DependencySource::Git { url: git_url.clone(), branch: None }
                } else if let Some(local_path) = path {
                    DependencySource::Local(PathBuf::from(local_path))
                } else {
                    DependencySource::Registry(self.default_index.clone())
                };

                let final_scope = if editable.unwrap_or(false) {
                    DependencyScope::Development
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

    fn parse_requirements_txt(&self, path: &Path) -> Result<Vec<DependencyInfo>, DependencyError> {
        let content = fs::read_to_string(path)
            .map_err(|e| DependencyError::ParseError(format!("Failed to read requirements.txt: {}", e)))?;

        let mut dependencies = Vec::new();

        for line in content.lines() {
            if let Ok(req) = self.parse_requirement_line(line) {
                let source = if let Some(url) = req.url {
                    if url.starts_with("git+") {
                        DependencySource::Git {
                            url: url.strip_prefix("git+").unwrap_or(&url).to_string(),
                            branch: None,
                        }
                    } else if url.starts_with("file://") || req.is_editable {
                        DependencySource::Local(PathBuf::from(url.strip_prefix("file://").unwrap_or(&url)))
                    } else {
                        DependencySource::Unknown
                    }
                } else {
                    DependencySource::Registry(self.default_index.clone())
                };

                let scope = if req.is_editable {
                    DependencyScope::Development
                } else {
                    DependencyScope::Production
                };

                dependencies.push(DependencyInfo {
                    name: req.name,
                    version: req.version_spec,
                    source,
                    scope,
                    resolved_path: None,
                });
            }
        }

        Ok(dependencies)
    }

    fn parse_pyproject_toml(&self, path: &Path) -> Result<Vec<DependencyInfo>, DependencyError> {
        let content = fs::read_to_string(path)
            .map_err(|e| DependencyError::ParseError(format!("Failed to read pyproject.toml: {}", e)))?;

        let pyproject: PyprojectToml = toml::from_str(&content)
            .map_err(|e| DependencyError::ParseError(format!("Failed to parse pyproject.toml: {}", e)))?;

        let mut dependencies = Vec::new();

        if let Some(project) = &pyproject.project {
            if let Some(deps) = &project.dependencies {
                for dep_str in deps {
                    if let Ok(req) = self.parse_requirement_line(dep_str) {
                        dependencies.push(DependencyInfo {
                            name: req.name,
                            version: req.version_spec,
                            source: DependencySource::Registry(self.default_index.clone()),
                            scope: DependencyScope::Production,
                            resolved_path: None,
                        });
                    }
                }
            }

            if let Some(optional_deps) = &project.optional_dependencies {
                for (group_name, deps) in optional_deps {
                    let scope = if group_name == "dev" || group_name == "test" {
                        DependencyScope::Development
                    } else {
                        DependencyScope::Optional
                    };

                    for dep_str in deps {
                        if let Ok(req) = self.parse_requirement_line(dep_str) {
                            dependencies.push(DependencyInfo {
                                name: req.name,
                                version: req.version_spec,
                                source: DependencySource::Registry(self.default_index.clone()),
                                scope,
                                resolved_path: None,
                            });
                        }
                    }
                }
            }
        }

        if let Some(build_system) = &pyproject.build_system {
            for dep_str in &build_system.requires {
                if let Ok(req) = self.parse_requirement_line(dep_str) {
                    dependencies.push(DependencyInfo {
                        name: req.name,
                        version: req.version_spec,
                        source: DependencySource::Registry(self.default_index.clone()),
                        scope: DependencyScope::Build,
                        resolved_path: None,
                    });
                }
            }
        }

        Ok(dependencies)
    }

    fn parse_pipfile(&self, path: &Path) -> Result<Vec<DependencyInfo>, DependencyError> {
        let content = fs::read_to_string(path)
            .map_err(|e| DependencyError::ParseError(format!("Failed to read Pipfile: {}", e)))?;

        let pipfile: PipfileTom = toml::from_str(&content)
            .map_err(|e| DependencyError::ParseError(format!("Failed to parse Pipfile: {}", e)))?;

        let mut dependencies = Vec::new();
