//! Python dependency security analysis for TOML files
//!
//! This module checks for security issues in Python dependency configurations
//! in pyproject.toml files.

use super::super::super::config::ConfigSecurityConfig;
use super::super::super::types::{ConfigIssue, ConfigSeverity};
use super::super::utils;
use crate::analysis::AnalysisError;
use toml::Value as TomlValue;

/// Python dependency security checker
pub struct PythonDependencyChecker {
    config: ConfigSecurityConfig,
}

impl PythonDependencyChecker {
    pub fn new(config: &ConfigSecurityConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    /// Check for Python dependency security issues in pyproject.toml
    pub fn check_python_dependencies(
        &self,
        deps: &TomlValue,
    ) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();

        if let TomlValue::Array(deps_array) = deps {
            for dep in deps_array {
                if let TomlValue::String(dep_str) = dep {
                    // Parse dependency string (e.g., "package>=1.0.0")
                    let dep_name = dep_str
                        .split(['=', '>', '<', '~', '!'])
                        .next()
                        .unwrap_or(dep_str);

                    // Check for known vulnerable Python packages
                    if self.is_known_vulnerable_python_package(dep_name) {
                        issues.push(utils::create_config_issue(
                            ConfigSeverity::High,
                            "Known Vulnerable Python Dependency",
                            format!("Python dependency '{}' is known to have security vulnerabilities", dep_name),
                            None,
                            "Update to a secure version or find an alternative package",
                            vec!["dependency".to_string(), "python".to_string(), "vulnerability".to_string()],
                        ).with_cwe(1104));
                    }

                    // Check for very loose version constraints
                    if !dep_str.contains(['=', '>', '<', '~']) {
                        issues.push(utils::create_config_issue(
                            ConfigSeverity::Low,
                            "Unpinned Python Dependency",
                            format!("Python dependency '{}' has no version constraint", dep_name),
                            None,
                            "Pin Python dependencies to specific versions for reproducible builds",
                            vec!["dependency".to_string(), "python".to_string(), "version".to_string()],
                        ).with_cwe(1188));
                    }

                    // Check for development dependencies in production
                    if self.is_development_package(dep_name) {
                        issues.push(utils::create_config_issue(
                            ConfigSeverity::Low,
                            "Development Dependency in Production",
                            format!(
                                "Development package '{}' included in production dependencies",
                                dep_name
                            ),
                            None,
                            "Move development packages to optional dependencies",
                            vec![
                                "dependency".to_string(),
                                "python".to_string(),
                                "development".to_string(),
                            ],
                        ));
                    }
                }
            }
        }

        Ok(issues)
    }

    /// Check for Poetry-specific dependency configurations
    pub fn check_poetry_dependencies(&self, table: &toml::value::Table) -> Vec<ConfigIssue> {
        let mut issues = Vec::new();

        // Check for Poetry sources
        if let Some(TomlValue::Array(sources)) = table.get("source") {
            for source in sources {
                if let TomlValue::Table(source_table) = source {
                    if let Some(TomlValue::String(url)) = source_table.get("url") {
                        if url.starts_with("http://") {
                            issues.push(
                                utils::create_config_issue(
                                    ConfigSeverity::High,
                                    "Insecure Poetry Source",
                                    format!("Poetry source uses HTTP: {}", url),
                                    None,
                                    "Use HTTPS URLs for Poetry package sources",
                                    vec![
                                        "poetry".to_string(),
                                        "http".to_string(),
                                        "toml".to_string(),
                                    ],
                                )
                                .with_cwe(319),
                            );
                        }
                    }
                }
            }
        }

        // Check for Poetry dependencies with git sources
        if let Some(TomlValue::Table(dependencies)) = table.get("dependencies") {
            for (dep_name, dep_config) in dependencies {
                if let TomlValue::Table(config_table) = dep_config {
                    if let Some(TomlValue::String(git_url)) = config_table.get("git") {
                        if git_url.starts_with("http://") {
                            issues.push(
                                utils::create_config_issue(
                                    ConfigSeverity::Medium,
                                    "Insecure Poetry Git Dependency",
                                    format!(
                                        "Poetry dependency '{}' uses insecure HTTP git URL",
                                        dep_name
                                    ),
                                    None,
                                    "Use HTTPS or SSH URLs for git dependencies",
                                    vec![
                                        "poetry".to_string(),
                                        "git".to_string(),
                                        "http".to_string(),
                                    ],
                                )
                                .with_cwe(319),
                            );
                        }
                    }
                }
            }
        }

        issues
    }

    /// Check for pip configuration issues
    pub fn check_pip_configuration(&self, table: &toml::value::Table) -> Vec<ConfigIssue> {
        let mut issues = Vec::new();

        // Check for pip configuration
        if let Some(TomlValue::Table(pip)) = table.get("pip") {
            // Check for insecure index URLs
            if let Some(TomlValue::String(index_url)) = pip.get("index-url") {
                if index_url.starts_with("http://") {
                    issues.push(
                        utils::create_config_issue(
                            ConfigSeverity::High,
                            "Insecure Pip Index URL",
                            "Pip configured with insecure HTTP index URL",
                            None,
                            "Use HTTPS URLs for pip package indexes",
                            vec!["pip".to_string(), "http".to_string(), "toml".to_string()],
                        )
                        .with_cwe(319),
                    );
                }
            }

            // Check for extra index URLs
            if let Some(TomlValue::Array(extra_urls)) = pip.get("extra-index-url") {
                for url in extra_urls {
                    if let TomlValue::String(url_str) = url {
                        if url_str.starts_with("http://") {
                            issues.push(
                                utils::create_config_issue(
                                    ConfigSeverity::High,
                                    "Insecure Extra Index URL",
                                    format!("Extra index URL uses HTTP: {}", url_str),
                                    None,
                                    "Use HTTPS URLs for all package indexes",
                                    vec!["pip".to_string(), "http".to_string(), "toml".to_string()],
                                )
                                .with_cwe(319),
                            );
                        }
                    }
                }
            }
        }

        issues
    }

    fn is_known_vulnerable_python_package(&self, package_name: &str) -> bool {
        // Common Python packages with known vulnerabilities (simplified list)
        let vulnerable_packages = [
            "pillow",   // Often has vulnerabilities
            "requests", // Had some security issues
            "urllib3",  // Has had vulnerabilities
            "pyyaml",   // Had code execution vulnerabilities
            "jinja2",   // Had template injection issues
        ];

        // This is a simplified check - in practice, you'd want to check against
        // a vulnerability database like PyUp Safety DB
        vulnerable_packages.contains(&package_name)
    }

    fn is_development_package(&self, package_name: &str) -> bool {
        let dev_packages = [
            "pytest",
            "black",
            "flake8",
            "mypy",
            "tox",
            "coverage",
            "sphinx",
            "pre-commit",
            "isort",
            "bandit",
            "safety",
        ];

        dev_packages.contains(&package_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_python_dependency_analysis() {
        let config = ConfigSecurityConfig::default();
        let checker = PythonDependencyChecker::new(&config);

        let deps = toml::Value::Array(vec![
            toml::Value::String("requests".to_string()),
            toml::Value::String("pillow>=8.0.0".to_string()),
            toml::Value::String("pytest".to_string()),
        ]);

        let issues = checker.check_python_dependencies(&deps).unwrap();
        assert!(issues.iter().any(|i| i.title.contains("Vulnerable Python")));
        assert!(issues.iter().any(|i| i.title.contains("Unpinned Python")));
        assert!(issues
            .iter()
            .any(|i| i.title.contains("Development Dependency")));
    }

    #[test]
    fn test_poetry_source_analysis() {
        let config = ConfigSecurityConfig::default();
        let checker = PythonDependencyChecker::new(&config);

        let mut poetry_table = toml::value::Table::new();
        let sources = toml::Value::Array(vec![toml::Value::Table({
            let mut source = toml::value::Table::new();
            source.insert(
                "name".to_string(),
                toml::Value::String("internal".to_string()),
            );
            source.insert(
                "url".to_string(),
                toml::Value::String("http://internal.pypi.com/simple/".to_string()),
            );
            source
        })]);
        poetry_table.insert("source".to_string(), sources);

        let issues = checker.check_poetry_dependencies(&poetry_table);
        assert!(issues
            .iter()
            .any(|i| i.title.contains("Insecure Poetry Source")));
    }

    #[test]
    fn test_pip_configuration() {
        let config = ConfigSecurityConfig::default();
        let checker = PythonDependencyChecker::new(&config);

        let mut pip_table = toml::value::Table::new();
        let mut pip_config = toml::value::Table::new();
        pip_config.insert(
            "index-url".to_string(),
            toml::Value::String("http://pypi.example.com/simple/".to_string()),
        );
        pip_table.insert("pip".to_string(), toml::Value::Table(pip_config));

        let issues = checker.check_pip_configuration(&pip_table);
        assert!(issues
            .iter()
            .any(|i| i.title.contains("Insecure Pip Index")));
    }
}
