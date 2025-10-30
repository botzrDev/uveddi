//! Main TOML analyzer
//!
//! This module coordinates all TOML security checks including credential detection,
//! dependency analysis, and build configuration validation.

use super::super::super::config::ConfigSecurityConfig;
use super::super::super::types::{ConfigIssue, ConfigSeverity, ConfigType};
use super::super::{utils, LanguageAnalyzer};
use super::{TomlBuildChecker, TomlCredentialChecker, TomlDependencyChecker};
use crate::analysis::AnalysisError;
use toml::Value as TomlValue;

/// Analyzer for TOML configuration files
pub struct TomlAnalyzer {
    config: ConfigSecurityConfig,
    credential_checker: TomlCredentialChecker,
    dependency_checker: TomlDependencyChecker,
    build_checker: TomlBuildChecker,
}

impl LanguageAnalyzer for TomlAnalyzer {
    fn new(config: &ConfigSecurityConfig) -> Result<Self, AnalysisError> {
        Ok(Self {
            config: config.clone(),
            credential_checker: TomlCredentialChecker::new(config),
            dependency_checker: TomlDependencyChecker::new(config),
            build_checker: TomlBuildChecker::new(config),
        })
    }

    fn analyze(&self, content: &str) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();

        match content.parse::<TomlValue>() {
            Ok(toml_value) => {
                // Check for credentials and secrets
                issues.extend(self.credential_checker.check_credentials(&toml_value)?);
                issues.extend(
                    self.credential_checker
                        .check_database_credentials(&toml_value),
                );

                // Check dependency security
                issues.extend(
                    self.dependency_checker
                        .check_dependency_security(&toml_value)?,
                );

                // Check build configuration
                issues.extend(self.build_checker.check_build_configuration(&toml_value)?);
                issues.extend(self.build_checker.check_development_settings(&toml_value));

                // Check for additional database configurations
                issues.extend(self.check_database_configuration(&toml_value)?);
            }
            Err(e) => {
                return Err(AnalysisError::ParseError {
                    message: format!("Invalid TOML: {}", e),
                });
            }
        }

        Ok(issues)
    }

    fn supported_type(&self) -> ConfigType {
        ConfigType::Toml
    }

    fn validate_syntax(&self, content: &str) -> Result<(), AnalysisError> {
        content
            .parse::<TomlValue>()
            .map_err(|e| AnalysisError::ParseError {
                message: format!("Invalid TOML syntax: {}", e),
            })?;
        Ok(())
    }
}

impl TomlAnalyzer {
    /// Check for database configuration issues
    fn check_database_configuration(
        &self,
        value: &TomlValue,
    ) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();
        self.check_database_recursive(value, "", &mut issues);
        Ok(issues)
    }

    fn check_database_recursive(
        &self,
        value: &TomlValue,
        path: &str,
        issues: &mut Vec<ConfigIssue>,
    ) {
        match value {
            TomlValue::Table(table) => {
                // Check if this looks like a database configuration
                let is_db_config = table.keys().any(|k| {
                    let key_lower = k.to_lowercase();
                    key_lower.contains("database")
                        || key_lower.contains("db")
                        || key_lower == "host"
                        || key_lower == "port"
                        || key_lower == "username"
                });

                if is_db_config {
                    // Check for SSL/TLS settings
                    if let Some(TomlValue::Boolean(false)) = table.get("ssl") {
                        issues.push(
                            utils::create_config_issue(
                                ConfigSeverity::High,
                                "Database SSL Disabled",
                                "Database connection does not use SSL encryption",
                                None,
                                "Enable SSL/TLS for database connections",
                                vec!["database".to_string(), "encryption".to_string()],
                            )
                            .with_cwe(319),
                        );
                    }

                    // Check for default ports
                    if let Some(TomlValue::Integer(port)) = table.get("port") {
                        let default_ports = [3306, 5432, 1433, 27017, 6379];
                        if default_ports.contains(&(*port as u16)) {
                            issues.push(
                                utils::create_config_issue(
                                    ConfigSeverity::Low,
                                    "Default Database Port",
                                    format!("Database using default port: {}", port),
                                    None,
                                    "Consider using non-default ports",
                                    vec!["database".to_string(), "default-config".to_string()],
                                )
                                .with_cwe(1188),
                            );
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

    /// Check for Python-specific TOML configurations (pyproject.toml)
    pub fn check_python_project(&self, value: &TomlValue) -> Vec<ConfigIssue> {
        let mut issues = Vec::new();

        if let TomlValue::Table(table) = value {
            // Check for Poetry configuration
            if let Some(TomlValue::Table(tool)) = table.get("tool") {
                if let Some(TomlValue::Table(poetry)) = tool.get("poetry") {
                    issues.extend(self.check_poetry_config(poetry));
                }

                // Check for other Python tools
                if let Some(TomlValue::Table(black)) = tool.get("black") {
                    issues.extend(self.check_formatter_config(black, "black"));
                }
            }
        }

        issues
    }

    fn check_poetry_config(&self, poetry: &toml::value::Table) -> Vec<ConfigIssue> {
        let mut issues = Vec::new();

        // Check for insecure Poetry sources
        if let Some(TomlValue::Array(sources)) = poetry.get("source") {
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

        issues
    }

    fn check_formatter_config(
        &self,
        formatter_config: &toml::value::Table,
        tool_name: &str,
    ) -> Vec<ConfigIssue> {
        let mut issues = Vec::new();

        // Check for overly permissive line length
        if let Some(TomlValue::Integer(line_length)) = formatter_config.get("line-length") {
            if *line_length > 200 {
                issues.push(utils::create_config_issue(
                    ConfigSeverity::Info,
                    "Very Long Line Length",
                    format!(
                        "{} configured with very long line length: {}",
                        tool_name, line_length
                    ),
                    None,
                    "Consider using shorter line lengths for better readability",
                    vec![tool_name.to_string(), "formatting".to_string()],
                ));
            }
        }

        issues
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toml_analyzer_creation() {
        let config = ConfigSecurityConfig::default();
        let analyzer = TomlAnalyzer::new(&config).unwrap();
        assert_eq!(analyzer.supported_type(), ConfigType::Toml);
    }

    #[test]
    fn test_comprehensive_analysis() {
        let config = ConfigSecurityConfig::default();
        let analyzer = TomlAnalyzer::new(&config).unwrap();

        let toml_content = r#"
[database]
password = "SuperSecret123!"
ssl = false
port = 5432

[dependencies]
serde = "*"
tokio = { git = "http://github.com/tokio-rs/tokio.git" }

[features]
default = ["unsafe-optimizations"]

[build]
script = "build.rs"
"#;

        let issues = analyzer.analyze(toml_content).unwrap();
        assert!(!issues.is_empty());

        // Should detect multiple types of issues
        assert!(issues.iter().any(|i| i.title.contains("Credential")));
        assert!(issues.iter().any(|i| i.title.contains("SSL Disabled")));
        assert!(issues.iter().any(|i| i.title.contains("Wildcard")));
        assert!(issues.iter().any(|i| i.title.contains("Insecure Git")));
        assert!(issues.iter().any(|i| i.title.contains("Unsafe")));
    }

    #[test]
    fn test_python_project_analysis() {
        let config = ConfigSecurityConfig::default();
        let analyzer = TomlAnalyzer::new(&config).unwrap();

        let toml_content = r#"
[tool.poetry]
[[tool.poetry.source]]
name = "internal"
url = "http://internal.pypi.com/simple/"

[tool.black]
line-length = 300
"#;

        let parsed: TomlValue = toml_content.parse().unwrap();
        let issues = analyzer.check_python_project(&parsed);
        assert!(issues
            .iter()
            .any(|i| i.title.contains("Insecure Poetry Source")));
        assert!(issues
            .iter()
            .any(|i| i.title.contains("Very Long Line Length")));
    }

    #[test]
    fn test_syntax_validation() {
        let config = ConfigSecurityConfig::default();
        let analyzer = TomlAnalyzer::new(&config).unwrap();

        // Valid TOML
        assert!(analyzer
            .validate_syntax("[section]\nkey = \"value\"")
            .is_ok());

        // Invalid TOML
        assert!(analyzer.validate_syntax("[section\nkey = value").is_err());
    }
}
