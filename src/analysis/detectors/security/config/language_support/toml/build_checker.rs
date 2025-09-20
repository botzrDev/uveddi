//! TOML build configuration security analysis
//!
//! This module checks for security issues in build configurations
//! including unsafe features, build scripts, and compiler settings.

use crate::analysis::AnalysisError;
use super::super::super::types::{ConfigIssue, ConfigSeverity};
use super::super::super::config::ConfigSecurityConfig;
use super::super::utils;
use toml::Value as TomlValue;

/// Build configuration security checker for TOML files
pub struct TomlBuildChecker {
    config: ConfigSecurityConfig,
}

impl TomlBuildChecker {
    pub fn new(config: &ConfigSecurityConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    /// Check for insecure build configurations
    pub fn check_build_configuration(&self, value: &TomlValue) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();

        if let TomlValue::Table(table) = value {
            // Check for dangerous build scripts
            if table.contains_key("build") {
                issues.push(utils::create_config_issue(
                    ConfigSeverity::Info,
                    "Build Script Present",
                    "Custom build script detected - review for security implications",
                    None,
                    "Ensure build scripts do not execute untrusted code",
                    vec!["build".to_string(), "code-execution".to_string()],
                ));
            }

            // Check for unsafe features in Rust
            if let Some(TomlValue::Table(features)) = table.get("features") {
                if features.contains_key("unsafe") ||
                   features.values().any(|v| {
                       if let TomlValue::Array(arr) = v {
                           arr.iter().any(|item| {
                               if let TomlValue::String(s) = item {
                                   s.contains("unsafe")
                               } else {
                                   false
                               }
                           })
                       } else {
                           false
                       }
                   }) {
                    issues.push(utils::create_config_issue(
                        ConfigSeverity::Medium,
                        "Unsafe Feature Flag",
                        "Configuration enables unsafe features",
                        None,
                        "Review unsafe feature usage for security implications",
                        vec!["unsafe".to_string(), "rust".to_string()],
                    ).with_cwe(242));
                }
            }

            // Check for profile configurations
            issues.extend(self.check_profile_security(table)?);

            // Check for workspace configurations
            issues.extend(self.check_workspace_security(table)?);
        }

        Ok(issues)
    }

    fn check_profile_security(&self, table: &toml::value::Table) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();

        if let Some(TomlValue::Table(profile_table)) = table.get("profile") {
            // Check release profile for security
            if let Some(TomlValue::Table(release_profile)) = profile_table.get("release") {
                // Check for debug info in release builds
                if let Some(TomlValue::Boolean(true)) = release_profile.get("debug") {
                    issues.push(utils::create_config_issue(
                        ConfigSeverity::Low,
                        "Debug Info in Release Build",
                        "Release profile includes debug information",
                        None,
                        "Consider disabling debug info for production releases",
                        vec!["build".to_string(), "debug".to_string(), "release".to_string()],
                    ).with_cwe(489));
                }

                // Check for missing optimization
                if let Some(TomlValue::Boolean(false)) = release_profile.get("opt-level") {
                    issues.push(utils::create_config_issue(
                        ConfigSeverity::Info,
                        "Optimization Disabled",
                        "Release profile has optimization disabled",
                        None,
                        "Enable optimization for production builds",
                        vec!["build".to_string(), "optimization".to_string()],
                    ));
                }
            }

            // Check dev profile for security implications
            if let Some(TomlValue::Table(dev_profile)) = profile_table.get("dev") {
                if let Some(TomlValue::Boolean(false)) = dev_profile.get("overflow-checks") {
                    issues.push(utils::create_config_issue(
                        ConfigSeverity::Medium,
                        "Overflow Checks Disabled",
                        "Development profile has overflow checks disabled",
                        None,
                        "Keep overflow checks enabled for safety",
                        vec!["build".to_string(), "overflow".to_string(), "safety".to_string()],
                    ).with_cwe(190));
                }
            }
        }

        Ok(issues)
    }

    fn check_workspace_security(&self, table: &toml::value::Table) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();

        if let Some(TomlValue::Table(workspace)) = table.get("workspace") {
            // Check for workspace members with external paths
            if let Some(TomlValue::Array(members)) = workspace.get("members") {
                for member in members {
                    if let TomlValue::String(member_path) = member {
                        if self.is_external_workspace_member(member_path) {
                            issues.push(utils::create_config_issue(
                                ConfigSeverity::Medium,
                                "External Workspace Member",
                                format!("Workspace member points to external path: {}", member_path),
                                None,
                                "Ensure external workspace members are trusted",
                                vec!["workspace".to_string(), "external".to_string(), "path".to_string()],
                            ).with_cwe(22));
                        }
                    }
                }
            }

            // Check for workspace exclude patterns
            if let Some(TomlValue::Array(exclude)) = workspace.get("exclude") {
                for excluded in exclude {
                    if let TomlValue::String(exclude_path) = excluded {
                        if exclude_path.contains("security") || exclude_path.contains("audit") {
                            issues.push(utils::create_config_issue(
                                ConfigSeverity::Low,
                                "Security Directory Excluded",
                                format!("Workspace excludes security-related directory: {}", exclude_path),
                                None,
                                "Review excluded security directories",
                                vec!["workspace".to_string(), "exclude".to_string(), "security".to_string()],
                            ));
                        }
                    }
                }
            }
        }

        Ok(issues)
    }

    fn is_external_workspace_member(&self, path: &str) -> bool {
        // Check for paths that go outside the project directory
        path.starts_with("../") || path.starts_with("/") || path.contains("../")
    }

    /// Check for development-specific build configurations
    pub fn check_development_settings(&self, value: &TomlValue) -> Vec<ConfigIssue> {
        let mut issues = Vec::new();

        if let TomlValue::Table(table) = value {
            // Check for development dependencies in production
            if let Some(TomlValue::Table(dev_deps)) = table.get("dev-dependencies") {
                if dev_deps.len() > 20 {
                    issues.push(utils::create_config_issue(
                        ConfigSeverity::Info,
                        "Large Development Dependency Set",
                        format!("Project has {} development dependencies", dev_deps.len()),
                        None,
                        "Consider reducing development dependencies for faster builds",
                        vec!["build".to_string(), "dev-dependencies".to_string()],
                    ));
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
    fn test_unsafe_feature_detection() {
        let config = ConfigSecurityConfig::default();
        let checker = TomlBuildChecker::new(&config);

        let toml_content = r#"
[features]
default = ["unsafe-optimizations"]
unsafe = []
"#;

        let parsed: TomlValue = toml_content.parse().unwrap();
        let issues = checker.check_build_configuration(&parsed).unwrap();
        assert!(issues.iter().any(|i| i.title.contains("Unsafe Feature Flag")));
    }

    #[test]
    fn test_build_script_detection() {
        let config = ConfigSecurityConfig::default();
        let checker = TomlBuildChecker::new(&config);

        let toml_content = r#"
[package]
name = "test"
build = "build.rs"
"#;

        let parsed: TomlValue = toml_content.parse().unwrap();
        let issues = checker.check_build_configuration(&parsed).unwrap();
        assert!(issues.iter().any(|i| i.title.contains("Build Script Present")));
    }

    #[test]
    fn test_profile_security() {
        let config = ConfigSecurityConfig::default();
        let checker = TomlBuildChecker::new(&config);

        let toml_content = r#"
[profile.release]
debug = true
opt-level = 0

[profile.dev]
overflow-checks = false
"#;

        let parsed: TomlValue = toml_content.parse().unwrap();
        let issues = checker.check_build_configuration(&parsed).unwrap();
        assert!(issues.iter().any(|i| i.title.contains("Debug Info in Release")));
        assert!(issues.iter().any(|i| i.title.contains("Overflow Checks Disabled")));
    }

    #[test]
    fn test_workspace_security() {
        let config = ConfigSecurityConfig::default();
        let checker = TomlBuildChecker::new(&config);

        let toml_content = r#"
[workspace]
members = ["../external-crate", "local-crate"]
exclude = ["security-audit", "tests"]
"#;

        let parsed: TomlValue = toml_content.parse().unwrap();
        let issues = checker.check_build_configuration(&parsed).unwrap();
        assert!(issues.iter().any(|i| i.title.contains("External Workspace Member")));
        assert!(issues.iter().any(|i| i.title.contains("Security Directory Excluded")));
    }

    #[test]
    fn test_safe_build_configuration() {
        let config = ConfigSecurityConfig::default();
        let checker = TomlBuildChecker::new(&config);

        let toml_content = r#"
[package]
name = "safe-crate"
version = "1.0.0"

[features]
default = ["production"]
production = []

[profile.release]
debug = false
opt-level = 3
"#;

        let parsed: TomlValue = toml_content.parse().unwrap();
        let issues = checker.check_build_configuration(&parsed).unwrap();
        // Should have minimal issues for well-configured build
        assert!(issues.len() <= 1);
    }
}