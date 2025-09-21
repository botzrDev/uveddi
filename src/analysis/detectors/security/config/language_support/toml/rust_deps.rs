//! Rust dependency security analysis for TOML files
//!
//! This module checks for security issues in Rust dependency configurations
//! in Cargo.toml files.

use super::super::super::config::ConfigSecurityConfig;
use super::super::super::types::{ConfigIssue, ConfigSeverity};
use super::super::utils;
use crate::analysis::AnalysisError;
use toml::Value as TomlValue;

/// Rust dependency security checker
pub struct RustDependencyChecker {
    config: ConfigSecurityConfig,
}

impl RustDependencyChecker {
    pub fn new(config: &ConfigSecurityConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    /// Check for Rust dependency security issues in Cargo.toml
    pub fn check_rust_dependencies(
        &self,
        deps: &TomlValue,
        section: &str,
    ) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();

        if let TomlValue::Table(deps_table) = deps {
            for (dep_name, dep_config) in deps_table {
                // Check for git dependencies without specific commit/tag
                if let TomlValue::Table(config_table) = dep_config {
                    if let Some(TomlValue::String(git_url)) = config_table.get("git") {
                        let has_specific_ref = config_table.get("rev").is_some()
                            || config_table.get("tag").is_some()
                            || config_table.get("branch").is_some();

                        if !has_specific_ref {
                            issues.push(utils::create_config_issue(
                                ConfigSeverity::Medium,
                                "Git Dependency Without Specific Reference",
                                format!("Dependency '{}' uses git without specific commit/tag", dep_name),
                                None,
                                "Pin git dependencies to specific commits or tags for reproducible builds",
                                vec!["dependency".to_string(), "git".to_string(), "toml".to_string()],
                            ).with_cwe(1188));
                        }

                        // Check for insecure git URLs
                        if git_url.starts_with("http://") {
                            issues.push(
                                utils::create_config_issue(
                                    ConfigSeverity::Medium,
                                    "Insecure Git Dependency URL",
                                    format!("Dependency '{}' uses insecure HTTP git URL", dep_name),
                                    None,
                                    "Use HTTPS or SSH URLs for git dependencies",
                                    vec![
                                        "dependency".to_string(),
                                        "git".to_string(),
                                        "http".to_string(),
                                    ],
                                )
                                .with_cwe(319),
                            );
                        }
                    }

                    // Check for path dependencies that might be insecure
                    if let Some(TomlValue::String(path)) = config_table.get("path") {
                        if self.is_insecure_path(path) {
                            issues.push(
                                utils::create_config_issue(
                                    ConfigSeverity::Low,
                                    "Potentially Insecure Path Dependency",
                                    format!(
                                        "Dependency '{}' uses path that might be insecure: {}",
                                        dep_name, path
                                    ),
                                    None,
                                    "Ensure path dependencies point to trusted locations",
                                    vec![
                                        "dependency".to_string(),
                                        "path".to_string(),
                                        "toml".to_string(),
                                    ],
                                )
                                .with_cwe(22),
                            );
                        }
                    }
                }

                // Check for wildcard version constraints
                if let TomlValue::String(version) = dep_config {
                    if self.is_insecure_version_constraint(version) {
                        issues.push(utils::create_config_issue(
                            ConfigSeverity::Low,
                            "Insecure Version Constraint",
                            format!("Dependency '{}' uses wildcard or very loose version constraint: {}", dep_name, version),
                            None,
                            "Use more specific version constraints to avoid unexpected updates",
                            vec!["dependency".to_string(), "version".to_string(), "toml".to_string()],
                        ).with_cwe(1188));
                    }
                }

                // Check for known vulnerable packages
                if self.is_known_vulnerable_package(dep_name) {
                    issues.push(
                        utils::create_config_issue(
                            ConfigSeverity::High,
                            "Known Vulnerable Dependency",
                            format!(
                                "Dependency '{}' is known to have security vulnerabilities",
                                dep_name
                            ),
                            None,
                            "Update to a secure version or find an alternative package",
                            vec![
                                "dependency".to_string(),
                                "vulnerability".to_string(),
                                "toml".to_string(),
                            ],
                        )
                        .with_cwe(1104),
                    );
                }

                // Check for deprecated crates
                if self.is_deprecated_crate(dep_name) {
                    issues.push(utils::create_config_issue(
                        ConfigSeverity::Medium,
                        "Deprecated Rust Crate",
                        format!(
                            "Dependency '{}' is deprecated and may have security issues",
                            dep_name
                        ),
                        None,
                        "Replace with an actively maintained alternative",
                        vec![
                            "dependency".to_string(),
                            "deprecated".to_string(),
                            "toml".to_string(),
                        ],
                    ));
                }
            }
        }

        Ok(issues)
    }
    /// Check for dependency source configurations
    pub fn check_dependency_sources(
        &self,
        table: &toml::value::Table,
    ) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();
        // Check for custom registries or sources
        if let Some(TomlValue::Table(source_table)) = table.get("source") {
            for (source_name, source_config) in source_table {
                if let TomlValue::Table(config) = source_config {
                    if let Some(TomlValue::String(registry_url)) = config.get("registry") {
                        if registry_url.starts_with("http://") {
                            issues.push(
                                utils::create_config_issue(
                                    ConfigSeverity::High,
                                    "Insecure Registry URL",
                                    format!("Registry '{}' uses insecure HTTP URL", source_name),
                                    None,
                                    "Use HTTPS URLs for package registries",
                                    vec![
                                        "dependency".to_string(),
                                        "registry".to_string(),
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

        // Check for patch sources
        if let Some(TomlValue::Table(patch_table)) = table.get("patch") {
            for (registry, patches) in patch_table {
                if let TomlValue::Table(patches_table) = patches {
                    for (package, patch_config) in patches_table {
                        if let TomlValue::Table(config) = patch_config {
                            if let Some(TomlValue::String(git_url)) = config.get("git") {
                                if git_url.starts_with("http://") {
                                    issues.push(
                                        utils::create_config_issue(
                                            ConfigSeverity::Medium,
                                            "Insecure Patch Git URL",
                                            format!(
                                                "Patch for '{}' uses insecure HTTP git URL",
                                                package
                                            ),
                                            None,
                                            "Use HTTPS or SSH URLs for git patches",
                                            vec![
                                                "dependency".to_string(),
                                                "patch".to_string(),
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
            }
        }

        Ok(issues)
    }
    fn is_insecure_path(&self, path: &str) -> bool {
        // Check for potentially insecure paths
        let insecure_patterns = [
            "../",   // Parent directory traversal
            "/tmp/", // Temporary directories
            "/var/tmp/",
            "~",     // Home directory shortcuts
            "$HOME", // Environment variable paths
        ];

        insecure_patterns
            .iter()
            .any(|&pattern| path.contains(pattern))
    }

    fn is_insecure_version_constraint(&self, version: &str) -> bool {
        // Check for very loose version constraints
        version == "*" || version.starts_with(">=0")
    }

    fn is_known_vulnerable_package(&self, package_name: &str) -> bool {
        // Common Rust packages with known vulnerabilities (simplified list)
        let vulnerable_packages = [
            "openssl",   // Often has vulnerabilities
            "time",      // Had vulnerabilities in older versions
            "chrono",    // Had some security issues
            "hyper",     // Had HTTP/2 vulnerabilities
            "actix-web", // Had some security patches
        ];

        // This is a simplified check - in practice, you'd want to check against
        // a vulnerability database like RustSec Advisory Database
        vulnerable_packages.contains(&package_name)
    }

    fn is_deprecated_crate(&self, crate_name: &str) -> bool {
        // Common deprecated Rust crates
        let deprecated_crates = [
            "rustc-serialize", // Replaced by serde
            "log4rs",          // Consider other logging solutions
            "old-tokio",       // Use current tokio versions
        ];

        deprecated_crates.contains(&crate_name)
    }

    /// Check for workspace-level dependency configurations
    pub fn check_workspace_dependencies(&self, table: &toml::value::Table) -> Vec<ConfigIssue> {
        let mut issues = Vec::new();

        if let Some(TomlValue::Table(workspace)) = table.get("workspace") {
            if let Some(TomlValue::Table(workspace_deps)) = workspace.get("dependencies") {
                for (dep_name, dep_config) in workspace_deps {
                    if let TomlValue::String(version) = dep_config {
                        if self.is_insecure_version_constraint(version) {
                            issues.push(utils::create_config_issue(
                                ConfigSeverity::Low,
                                "Loose Workspace Dependency Constraint",
                                format!("Workspace dependency '{}' has loose version constraint: {}", dep_name, version),
                                None,
                                "Use specific version constraints for workspace dependencies",
                                vec!["workspace".to_string(), "dependency".to_string(), "version".to_string()],
                            ).with_cwe(1188));
                        }
                    }
                }
            }
        }

        issues
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_dependency_without_ref() {
        let config = ConfigSecurityConfig::default();
        let checker = RustDependencyChecker::new(&config);

        let mut deps_table = toml::value::Table::new();
        let mut dep_config = toml::value::Table::new();
        dep_config.insert(
            "git".to_string(),
            TomlValue::String("https://github.com/user/repo.git".to_string()),
        );
        deps_table.insert("my_crate".to_string(), TomlValue::Table(dep_config));

        let deps = TomlValue::Table(deps_table);
        let issues = checker
            .check_rust_dependencies(&deps, "dependencies")
            .unwrap();
        assert!(issues.iter().any(|i| i
            .title
            .contains("Git Dependency Without Specific Reference")));
    }

    #[test]
    fn test_insecure_git_url() {
        let config = ConfigSecurityConfig::default();
        let checker = RustDependencyChecker::new(&config);

        let mut deps_table = toml::value::Table::new();
        let mut dep_config = toml::value::Table::new();
        dep_config.insert(
            "git".to_string(),
            TomlValue::String("http://github.com/user/repo.git".to_string()),
        );
        deps_table.insert("my_crate".to_string(), TomlValue::Table(dep_config));

        let deps = TomlValue::Table(deps_table);
        let issues = checker
            .check_rust_dependencies(&deps, "dependencies")
            .unwrap();
        assert!(issues
            .iter()
            .any(|i| i.title.contains("Insecure Git Dependency URL")));
    }

    #[test]
    fn test_wildcard_version() {
        let config = ConfigSecurityConfig::default();
        let checker = RustDependencyChecker::new(&config);

        let mut deps_table = toml::value::Table::new();
        deps_table.insert("some_crate".to_string(), TomlValue::String("*".to_string()));

        let deps = TomlValue::Table(deps_table);
        let issues = checker
            .check_rust_dependencies(&deps, "dependencies")
            .unwrap();
        assert!(issues
            .iter()
            .any(|i| i.title.contains("Insecure Version Constraint")));
    }

    #[test]
    fn test_insecure_path_dependency() {
        let config = ConfigSecurityConfig::default();
        let checker = RustDependencyChecker::new(&config);

        let mut deps_table = toml::value::Table::new();
        let mut dep_config = toml::value::Table::new();
        dep_config.insert(
            "path".to_string(),
            TomlValue::String("../../../some/path".to_string()),
        );
        deps_table.insert("local_crate".to_string(), TomlValue::Table(dep_config));

        let deps = TomlValue::Table(deps_table);
        let issues = checker
            .check_rust_dependencies(&deps, "dependencies")
            .unwrap();
        assert!(issues
            .iter()
            .any(|i| i.title.contains("Potentially Insecure Path Dependency")));
    }
}
