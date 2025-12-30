//! TOML dependency security analysis
//!
//! This module coordinates dependency security checks for both Rust and Python
//! dependencies in TOML configuration files.

use super::super::super::config::ConfigSecurityConfig;
use super::super::super::types::ConfigIssue;
use super::{PythonDependencyChecker, RustDependencyChecker};
use crate::analysis::AnalysisError;
use toml::Value as TomlValue;

/// Main dependency security checker for TOML files
pub struct TomlDependencyChecker {
    config: ConfigSecurityConfig,
    rust_checker: RustDependencyChecker,
    python_checker: PythonDependencyChecker,
}

impl TomlDependencyChecker {
    pub fn new(config: &ConfigSecurityConfig) -> Self {
        Self {
            config: config.clone(),
            rust_checker: RustDependencyChecker::new(config),
            python_checker: PythonDependencyChecker::new(config),
        }
    }

    /// Check for dependency security issues in Cargo.toml or pyproject.toml
    pub fn check_dependency_security(
        &self,
        value: &TomlValue,
    ) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();

        if let TomlValue::Table(table) = value {
            // Check Rust dependencies (Cargo.toml)
            if let Some(dependencies) = table.get("dependencies") {
                issues.extend(
                    self.rust_checker
                        .check_rust_dependencies(dependencies, "dependencies")?,
                );
            }
            if let Some(dev_deps) = table.get("dev-dependencies") {
                issues.extend(
                    self.rust_checker
                        .check_rust_dependencies(dev_deps, "dev-dependencies")?,
                );
            }
            if let Some(build_deps) = table.get("build-dependencies") {
                issues.extend(
                    self.rust_checker
                        .check_rust_dependencies(build_deps, "build-dependencies")?,
                );
            }

            // Check for insecure dependency sources
            issues.extend(self.rust_checker.check_dependency_sources(table)?);

            // Check workspace dependencies
            issues.extend(self.rust_checker.check_workspace_dependencies(table));

            // Check Python dependencies (pyproject.toml)
            if let Some(project) = table.get("project") {
                if let TomlValue::Table(project_table) = project {
                    if let Some(dependencies) = project_table.get("dependencies") {
                        issues.extend(
                            self.python_checker
                                .check_python_dependencies(dependencies)?,
                        );
                    }
                }
            }

            // Check for Poetry configuration
            if let Some(TomlValue::Table(tool)) = table.get("tool") {
                if let Some(TomlValue::Table(poetry)) = tool.get("poetry") {
                    issues.extend(self.python_checker.check_poetry_dependencies(poetry));
                }

                // Check pip configuration
                issues.extend(self.python_checker.check_pip_configuration(tool));
            }
        }

        Ok(issues)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_dependency_analysis() {
        let config = ConfigSecurityConfig::default();
        let checker = TomlDependencyChecker::new(&config);

        let toml_content = r#"[dependencies]
serde = "*"
tokio = { git = "http://github.com/tokio-rs/tokio.git" }
local_crate = { path = "../../external/crate" }
"#;

        let parsed: TomlValue = toml::from_str(toml_content).unwrap();
        let issues = checker.check_dependency_security(&parsed).unwrap();
        assert!(!issues.is_empty());

        assert!(issues
            .iter()
            .any(|i| i.title.contains("Version Constraint")));
        assert!(issues.iter().any(|i| i.title.contains("Insecure Git")));
        assert!(issues.iter().any(|i| i.title.contains("Path Dependency")));
    }

    #[test]
    fn test_python_dependency_analysis() {
        let config = ConfigSecurityConfig::default();
        let checker = TomlDependencyChecker::new(&config);

        let toml_content = r#"[project]
dependencies = [
    "requests",
    "pillow>=8.0.0"
]

[tool.poetry]
[[tool.poetry.source]]
name = "internal"
url = "http://internal.pypi.com/simple/"
"#;

        let parsed: TomlValue = toml::from_str(toml_content).unwrap();
        let issues = checker.check_dependency_security(&parsed).unwrap();
        assert!(!issues.is_empty());

        assert!(issues
            .iter()
            .any(|i| i.title.contains("Python") || i.title.contains("Poetry")));
    }

    #[test]
    fn test_mixed_dependency_analysis() {
        let config = ConfigSecurityConfig::default();
        let checker = TomlDependencyChecker::new(&config);

        let toml_content = r#"# Rust dependencies
[dependencies]
serde = "1.0"
tokio = { git = "https://github.com/tokio-rs/tokio.git", tag = "v1.0.0" }

# Python dependencies
[project]
dependencies = ["requests>=2.25.0"]

[tool.poetry]
dependencies = {}
"#;

        let parsed: TomlValue = toml::from_str(toml_content).unwrap();
        let issues = checker.check_dependency_security(&parsed).unwrap();
        // Should analyze both Rust and Python dependencies
        // Well-configured dependencies should have minimal issues
        assert!(issues.len() <= 2);
    }

    #[test]
    fn test_safe_dependencies() {
        let config = ConfigSecurityConfig::default();
        let checker = TomlDependencyChecker::new(&config);

        let toml_content = r#"[dependencies]
serde = "1.0"
tokio = { version = "1.0", features = ["full"] }

[project]
dependencies = ["requests>=2.25.0,<3.0"]
"#;

        let parsed: TomlValue = toml::from_str(toml_content).unwrap();
        let issues = checker.check_dependency_security(&parsed).unwrap();
        // Should have minimal issues for well-configured dependencies
        assert!(issues.len() <= 1);
    }
}
