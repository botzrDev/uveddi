//! TOML configuration security analysis
//!
//! This module provides specialized security analysis for TOML configuration
//! files, commonly used in Rust projects (Cargo.toml) and Python (pyproject.toml).

use crate::analysis::AnalysisError;
use super::super::config::ConfigSecurityConfig;
use super::super::types::{ConfigIssue, ConfigSeverity, ConfigType};
use super::{LanguageAnalyzer, utils};
use toml::Value as TomlValue;

/// Analyzer for TOML configuration files
pub struct TomlAnalyzer {
    config: ConfigSecurityConfig,
}

impl LanguageAnalyzer for TomlAnalyzer {
    fn new(config: &ConfigSecurityConfig) -> Result<Self, AnalysisError> {
        Ok(Self {
            config: config.clone(),
        })
    }

    fn analyze(&self, content: &str) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();

        match content.parse::<TomlValue>() {
            Ok(toml_value) => {
                issues.extend(self.check_credentials(&toml_value)?);
                issues.extend(self.check_dependency_security(&toml_value)?);
                issues.extend(self.check_build_configuration(&toml_value)?);
                issues.extend(self.check_database_configuration(&toml_value)?);
            }
            Err(e) => {
                return Err(AnalysisError::ParseError(format!("Invalid TOML: {}", e)));
            }
        }

        Ok(issues)
    }

    fn supported_type(&self) -> ConfigType {
        ConfigType::Toml
    }

    fn validate_syntax(&self, content: &str) -> Result<(), AnalysisError> {
        content.parse::<TomlValue>()
            .map_err(|e| AnalysisError::ParseError(format!("Invalid TOML syntax: {}", e)))?;
        Ok(())
    }
}

impl TomlAnalyzer {
    /// Check for hardcoded credentials in TOML
    fn check_credentials(&self, value: &TomlValue) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();
        self.check_credentials_recursive(value, "", &mut issues);
        Ok(issues)
    }

    fn check_credentials_recursive(&self, value: &TomlValue, path: &str, issues: &mut Vec<ConfigIssue>) {
        match value {
            TomlValue::Table(table) => {
                for (key, val) in table {
                    let new_path = if path.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", path, key)
                    };

                    if utils::is_sensitive_key(key) {
                        if let TomlValue::String(val_str) = val {
                            if self.looks_like_credential(val_str) {
                                issues.push(utils::create_config_issue(
                                    ConfigSeverity::High,
                                    "Hardcoded Credential in TOML",
                                    format!("Found credential at path: {}", new_path),
                                    None,
                                    "Use environment variables or secure configuration management",
                                    vec!["credential".to_string(), "toml".to_string()],
                                ).with_cwe(798));
                            }
                        }
                    }

                    self.check_credentials_recursive(val, &new_path, issues);
                }
            }
            TomlValue::Array(array) => {
                for (index, item) in array.iter().enumerate() {
                    let new_path = format!("{}[{}]", path, index);
                    self.check_credentials_recursive(item, &new_path, issues);
                }
            }
            _ => {}
        }
    }

    /// Check for dependency security issues in Cargo.toml or pyproject.toml
    fn check_dependency_security(&self, value: &TomlValue) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();

        if let TomlValue::Table(table) = value {
            // Check Rust dependencies
            self.check_rust_dependencies(table, &mut issues);

            // Check Python dependencies
            self.check_python_dependencies(table, &mut issues);
        }

        Ok(issues)
    }

    fn check_rust_dependencies(&self, table: &toml::map::Map<String, TomlValue>, issues: &mut Vec<ConfigIssue>) {
        let dep_sections = ["dependencies", "dev-dependencies", "build-dependencies"];

        for section in &dep_sections {
            if let Some(TomlValue::Table(deps)) = table.get(*section) {
                for (dep_name, dep_spec) in deps {
                    // Check for git dependencies with insecure protocols
                    if let TomlValue::Table(spec_table) = dep_spec {
                        if let Some(TomlValue::String(git_url)) = spec_table.get("git") {
                            if git_url.starts_with("http://") {
                                issues.push(utils::create_config_issue(
                                    ConfigSeverity::Medium,
                                    "Insecure Git Dependency",
                                    format!("Dependency '{}' uses insecure HTTP git URL", dep_name),
                                    None,
                                    "Use HTTPS URLs for git dependencies",
                                    vec!["dependency".to_string(), "insecure-transport".to_string()],
                                ).with_cwe(319));
                            }
                        }

                        // Check for path dependencies outside project
                        if let Some(TomlValue::String(path)) = spec_table.get("path") {
                            if path.starts_with("..") && path.contains("../") {
                                issues.push(utils::create_config_issue(
                                    ConfigSeverity::Low,
                                    "External Path Dependency",
                                    format!("Dependency '{}' references external path: {}", dep_name, path),
                                    None,
                                    "Verify that external path dependencies are trusted",
                                    vec!["dependency".to_string(), "path-traversal".to_string()],
                                ).with_cwe(22));
                            }
                        }
                    }

                    // Check for wildcard version specifications
                    if let TomlValue::String(version) = dep_spec {
                        if version == "*" {
                            issues.push(utils::create_config_issue(
                                ConfigSeverity::Medium,
                                "Wildcard Dependency Version",
                                format!("Dependency '{}' uses wildcard version", dep_name),
                                None,
                                "Pin dependencies to specific versions for reproducible builds",
                                vec!["dependency".to_string(), "version-pinning".to_string()],
                            ));
                        }
                    }
                }
            }
        }
    }

    fn check_python_dependencies(&self, table: &toml::map::Map<String, TomlValue>, issues: &mut Vec<ConfigIssue>) {
        // Check for pyproject.toml structure
        if let Some(TomlValue::Table(project)) = table.get("project") {
            if let Some(TomlValue::Array(dependencies)) = project.get("dependencies") {
                for dep in dependencies {
                    if let TomlValue::String(dep_str) = dep {
                        // Check for insecure index URLs
                        if dep_str.contains("--index-url") && dep_str.contains("http://") {
                            issues.push(utils::create_config_issue(
                                ConfigSeverity::High,
                                "Insecure Python Index",
                                "Python dependency uses insecure HTTP index URL",
                                None,
                                "Use HTTPS URLs for Python package indexes",
                                vec!["dependency".to_string(), "python".to_string()],
                            ).with_cwe(319));
                        }
                    }
                }
            }
        }

        // Check tool.poetry sections
        if let Some(TomlValue::Table(tool)) = table.get("tool") {
            if let Some(TomlValue::Table(poetry)) = tool.get("poetry") {
                if let Some(TomlValue::Array(sources)) = poetry.get("source") {
                    for source in sources {
                        if let TomlValue::Table(source_table) = source {
                            if let Some(TomlValue::String(url)) = source_table.get("url") {
                                if url.starts_with("http://") {
                                    issues.push(utils::create_config_issue(
                                        ConfigSeverity::High,
                                        "Insecure Poetry Source",
                                        "Poetry source uses insecure HTTP URL",
                                        None,
                                        "Use HTTPS URLs for Poetry package sources",
                                        vec!["dependency".to_string(), "poetry".to_string()],
                                    ).with_cwe(319));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Check for insecure build configurations
    fn check_build_configuration(&self, value: &TomlValue) -> Result<Vec<ConfigIssue>, AnalysisError> {
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
        }

        Ok(issues)
    }

    /// Check for database configuration issues
    fn check_database_configuration(&self, value: &TomlValue) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();

        self.check_database_recursive(value, "", &mut issues);

        Ok(issues)
    }

    fn check_database_recursive(&self, value: &TomlValue, path: &str, issues: &mut Vec<ConfigIssue>) {
        match value {
            TomlValue::Table(table) => {
                // Check if this looks like a database configuration
                let is_db_config = table.keys().any(|k| {
                    let key_lower = k.to_lowercase();
                    key_lower.contains("database") || key_lower.contains("db") ||
                    key_lower == "host" || key_lower == "port" || key_lower == "username"
                });

                if is_db_config {
                    // Check for SSL/TLS settings
                    if let Some(TomlValue::Boolean(false)) = table.get("ssl") {
                        issues.push(utils::create_config_issue(
                            ConfigSeverity::High,
                            "Database SSL Disabled",
                            "Database connection does not use SSL encryption",
                            None,
                            "Enable SSL/TLS for database connections",
                            vec!["database".to_string(), "encryption".to_string()],
                        ).with_cwe(319));
                    }

                    // Check for default ports
                    if let Some(TomlValue::Integer(port)) = table.get("port") {
                        let default_ports = [3306, 5432, 1433, 27017, 6379];
                        if default_ports.contains(&(*port as u16)) {
                            issues.push(utils::create_config_issue(
                                ConfigSeverity::Low,
                                "Default Database Port",
                                format!("Database using default port: {}", port),
                                None,
                                "Consider using non-default ports",
                                vec!["database".to_string(), "default-config".to_string()],
                            ).with_cwe(1188));
                        }
                    }
                }

                // Recurse into nested tables
                for (key, val) in table {
                    let new_path = if path.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", path, key)
                    };
                    self.check_database_recursive(val, &new_path, issues);
                }
            }
            TomlValue::Array(array) => {
                for (index, item) in array.iter().enumerate() {
                    let new_path = format!("{}[{}]", path, index);
                    self.check_database_recursive(item, &new_path, issues);
                }
            }
            _ => {}
        }
    }

    fn looks_like_credential(&self, value: &str) -> bool {
        if value.is_empty() || value.len() < 8 {
            return false;
        }

        let fake_indicators = ["test", "example", "demo", "placeholder", "changeme"];
        if fake_indicators.iter().any(|&indicator| value.to_lowercase().contains(indicator)) {
            return false;
        }

        let has_complexity = value.chars().any(|c| !c.is_alphanumeric()) ||
                           (value.chars().any(|c| c.is_uppercase()) && value.chars().any(|c| c.is_lowercase()));

        has_complexity && value.len() <= 256
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toml_credential_detection() {
        let config = ConfigSecurityConfig::default();
        let analyzer = TomlAnalyzer::new(&config).unwrap();

        let toml_content = r#"
[database]
password = "SuperSecret123!"
api_key = "sk-1234567890abcdef"

[server]
debug = true
"#;

        let issues = analyzer.analyze(toml_content).unwrap();
        assert!(!issues.is_empty());

        let cred_issues: Vec<_> = issues.iter()
            .filter(|i| i.title.contains("Credential"))
            .collect();
        assert!(!cred_issues.is_empty());
    }

    #[test]
    fn test_rust_dependency_analysis() {
        let config = ConfigSecurityConfig::default();
        let analyzer = TomlAnalyzer::new(&config).unwrap();

        let toml_content = r#"
[dependencies]
serde = "*"
tokio = { git = "http://github.com/tokio-rs/tokio.git" }
local_crate = { path = "../../external/crate" }
"#;

        let issues = analyzer.analyze(toml_content).unwrap();
        assert!(!issues.is_empty());

        assert!(issues.iter().any(|i| i.title.contains("Wildcard")));
        assert!(issues.iter().any(|i| i.title.contains("Insecure Git")));
        assert!(issues.iter().any(|i| i.title.contains("External Path")));
    }

    #[test]
    fn test_python_dependency_analysis() {
        let config = ConfigSecurityConfig::default();
        let analyzer = TomlAnalyzer::new(&config).unwrap();

        let toml_content = r#"
[project]
dependencies = [
    "requests --index-url http://pypi.example.com/simple/"
]

[tool.poetry]
[[tool.poetry.source]]
name = "internal"
url = "http://internal.pypi.com/simple/"
"#;

        let issues = analyzer.analyze(toml_content).unwrap();
        assert!(!issues.is_empty());

        assert!(issues.iter().any(|i| i.title.contains("Python Index") || i.title.contains("Poetry Source")));
    }

    #[test]
    fn test_syntax_validation() {
        let config = ConfigSecurityConfig::default();
        let analyzer = TomlAnalyzer::new(&config).unwrap();

        // Valid TOML
        assert!(analyzer.validate_syntax("[section]\nkey = \"value\"").is_ok());

        // Invalid TOML
        assert!(analyzer.validate_syntax("[section\nkey = value").is_err());
    }

    #[test]
    fn test_unsafe_feature_detection() {
        let config = ConfigSecurityConfig::default();
        let analyzer = TomlAnalyzer::new(&config).unwrap();

        let toml_content = r#"
[features]
default = ["unsafe-optimizations"]
unsafe = []
"#;

        let issues = analyzer.analyze(toml_content).unwrap();
        assert!(issues.iter().any(|i| i.title.contains("Unsafe")));
    }
}