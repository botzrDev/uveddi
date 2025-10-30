//! Main defaults analyzer
//!
//! This module coordinates all default value security checks
//! including passwords, cryptographic values, and database defaults.

use super::super::super::config::ConfigSecurityConfig;
use super::super::super::types::{ConfigIssue, ConfigSeverity};
use super::crypto_defaults::CryptoDefaultChecker;
use super::database_defaults::DatabaseDefaultChecker;
use super::password_defaults::PasswordDefaultChecker;
use crate::analysis::AnalysisError;
use regex::Regex;

/// Analyzes configuration files for insecure default values
pub struct DefaultsAnalyzer {
    password_checker: PasswordDefaultChecker,
    crypto_checker: CryptoDefaultChecker,
    database_checker: DatabaseDefaultChecker,
    common_patterns: Vec<CommonDefaultPattern>,
    config: ConfigSecurityConfig,
}

/// Common default patterns that don't fit into specific categories
struct CommonDefaultPattern {
    name: String,
    description: String,
    severity: ConfigSeverity,
    regex: Regex,
    remediation: String,
    cwe_id: Option<u32>,
}

impl DefaultsAnalyzer {
    /// Create a new defaults analyzer
    pub fn new(config: &ConfigSecurityConfig) -> Result<Self, AnalysisError> {
        let password_checker = PasswordDefaultChecker::new(config)?;
        let crypto_checker = CryptoDefaultChecker::new(config)?;
        let database_checker = DatabaseDefaultChecker::new(config)?;
        let common_patterns = Self::build_common_patterns()?;

        Ok(Self {
            password_checker,
            crypto_checker,
            database_checker,
            common_patterns,
            config: config.clone(),
        })
    }

    /// Analyze content for insecure default values
    pub fn analyze(&self, content: &str) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let line_number = line_num + 1;

            // Check all default categories
            issues.extend(self.password_checker.check_line(line, line_number));
            issues.extend(self.crypto_checker.check_line(line, line_number));
            issues.extend(self.database_checker.check_line(line, line_number));

            // Check common patterns
            issues.extend(self.check_common_patterns(line, line_number));
        }

        // Additional checks for common patterns
        issues.extend(self.check_common_default_patterns(content)?);

        Ok(issues)
    }

    fn build_common_patterns() -> Result<Vec<CommonDefaultPattern>, AnalysisError> {
        let patterns = vec![
            CommonDefaultPattern {
                name: "Example Configuration".to_string(),
                description: "Example or template configuration detected".to_string(),
                severity: ConfigSeverity::Medium,
                regex: Regex::new(
                    r#"(?i)(example|template|sample|demo|placeholder)[\s]*[:=][\s]*['"]?[^'"\n]*['"]?"#,
                )?,
                remediation: "Replace example values with actual configuration.".to_string(),
                cwe_id: Some(1188),
            },
            CommonDefaultPattern {
                name: "TODO Configuration".to_string(),
                description: "TODO or placeholder comment in configuration".to_string(),
                severity: ConfigSeverity::Low,
                regex: Regex::new(
                    r#"(?i)(todo|fixme|change[_-]?me|replace[_-]?me)[\s]*[:=][\s]*['"]?[^'"\n]*['"]?"#,
                )?,
                remediation: "Complete the configuration by replacing TODO items.".to_string(),
                cwe_id: Some(1188),
            },
            CommonDefaultPattern {
                name: "Localhost Configuration".to_string(),
                description: "Localhost or 127.0.0.1 in production configuration".to_string(),
                severity: ConfigSeverity::Medium,
                regex: Regex::new(
                    r#"(?i)(host|server|url)[\s]*[:=][\s]*['"]?(localhost|127\.0\.0\.1)['"]?"#,
                )?,
                remediation: "Use environment-specific hostnames instead of localhost.".to_string(),
                cwe_id: Some(1188),
            },
            CommonDefaultPattern {
                name: "Test Mode Enabled".to_string(),
                description: "Test or debug mode enabled in configuration".to_string(),
                severity: ConfigSeverity::Medium,
                regex: Regex::new(
                    r#"(?i)(test[_-]?mode|debug[_-]?mode|dev[_-]?mode)[\s]*[:=][\s]*['"]?(true|yes|1|on)['"]?"#,
                )?,
                remediation: "Disable test/debug modes in production environments.".to_string(),
                cwe_id: Some(489),
            },
        ];

        Ok(patterns)
    }

    fn check_common_patterns(&self, line: &str, line_number: usize) -> Vec<ConfigIssue> {
        let mut issues = Vec::new();

        for pattern in &self.common_patterns {
            if pattern.regex.is_match(line) {
                let issue = ConfigIssue::new(
                    pattern.severity,
                    0.8, // Medium confidence for regex matches
                    pattern.name.clone(),
                    pattern.description.clone(),
                )
                .with_location(line_number, 1)
                .with_snippet(line.to_string())
                .with_remediation(pattern.remediation.clone());

                let final_issue = if let Some(cwe_id) = pattern.cwe_id {
                    issue.with_cwe(cwe_id)
                } else {
                    issue
                };

                issues.push(final_issue);
            }
        }

        issues
    }

    fn check_common_default_patterns(
        &self,
        content: &str,
    ) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Check for multiple default indicators in the same file
        let default_indicators = [
            "admin",
            "password",
            "secret",
            "test",
            "demo",
            "example",
            "localhost",
            "127.0.0.1",
            "changeme",
            "default",
        ];

        let mut found_indicators = 0;
        for indicator in &default_indicators {
            if content.to_lowercase().contains(indicator) {
                found_indicators += 1;
            }
        }

        // If many default indicators are found, flag the entire file
        if found_indicators >= 5 {
            issues.push(
                ConfigIssue::new(
                    ConfigSeverity::High,
                    0.9,
                    "Multiple Default Values".to_string(),
                    format!(
                        "Configuration file contains {} default/example values",
                        found_indicators
                    ),
                )
                .with_remediation(
                    "Review and replace all default values with production-appropriate settings.",
                )
                .with_tag("multiple-defaults")
                .with_cwe(1188),
            );
        }

        // Check for common insecure patterns
        issues.extend(self.check_insecure_urls(content)?);
        issues.extend(self.check_development_indicators(content)?);

        Ok(issues)
    }

    fn check_insecure_urls(&self, content: &str) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();

        let insecure_url_pattern = Regex::new(r#"(?i)http://[^\s"']*"#)?;
        for (line_num, line) in content.lines().enumerate() {
            if insecure_url_pattern.is_match(line) {
                issues.push(
                    ConfigIssue::new(
                        ConfigSeverity::Medium,
                        0.9,
                        "Insecure HTTP URL".to_string(),
                        "HTTP URL detected - should use HTTPS for security".to_string(),
                    )
                    .with_location(line_num + 1, 1)
                    .with_snippet(line.to_string())
                    .with_remediation("Replace HTTP URLs with HTTPS equivalents.")
                    .with_cwe(319),
                );
            }
        }

        Ok(issues)
    }

    fn check_development_indicators(
        &self,
        content: &str,
    ) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();

        let dev_indicators = [
            ("development", "Development settings"),
            ("staging", "Staging environment settings"),
            ("test", "Test environment settings"),
            ("debug", "Debug settings"),
            ("verbose", "Verbose logging"),
        ];

        for (indicator, description) in &dev_indicators {
            if content.to_lowercase().contains(indicator) {
                // Only flag if it appears multiple times or in key positions
                let count = content.to_lowercase().matches(indicator).count();
                if count >= 2 {
                    issues.push(
                        ConfigIssue::new(
                            ConfigSeverity::Low,
                            0.7,
                            "Development Configuration".to_string(),
                            format!("{} detected in configuration", description),
                        )
                        .with_remediation("Ensure development settings are not used in production.")
                        .with_tag("development-config")
                        .with_cwe(489),
                    );
                    break; // Only report once per file
                }
            }
        }

        Ok(issues)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults_analyzer_creation() {
        let config = ConfigSecurityConfig::default();
        let analyzer = DefaultsAnalyzer::new(&config).unwrap();
        // Basic creation test passes
    }

    #[test]
    fn test_basic_analysis() {
        let config = ConfigSecurityConfig::default();
        let analyzer = DefaultsAnalyzer::new(&config).unwrap();

        let config_content = r#"
password: admin
encryption_key: secret
"#;

        let issues = analyzer.analyze(config_content).unwrap();
        assert!(!issues.is_empty());
    }

    #[test]
    fn test_insecure_url_detection() {
        let config = ConfigSecurityConfig::default();
        let analyzer = DefaultsAnalyzer::new(&config).unwrap();

        let config_content = "api_url: http://api.example.com";
        let issues = analyzer.analyze(config_content).unwrap();
        assert!(issues.iter().any(|i| i.title.contains("Insecure HTTP URL")));
    }
}
