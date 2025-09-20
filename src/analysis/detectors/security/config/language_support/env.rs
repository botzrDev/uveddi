//! Environment variable file security analysis
//!
//! This module provides specialized security analysis for .env files and
//! other environment variable configuration files.

use crate::analysis::AnalysisError;
use super::super::config::ConfigSecurityConfig;
use super::super::types::{ConfigIssue, ConfigSeverity, ConfigType};
use super::{LanguageAnalyzer, utils};
use regex::Regex;
use std::collections::HashSet;

/// Analyzer for environment variable files (.env)
pub struct EnvAnalyzer {
    config: ConfigSecurityConfig,
    secret_patterns: Vec<EnvSecretPattern>,
}

/// Pattern for detecting secrets in environment variables
struct EnvSecretPattern {
    name: String,
    pattern: Regex,
    severity: ConfigSeverity,
    description: String,
}

impl LanguageAnalyzer for EnvAnalyzer {
    fn new(config: &ConfigSecurityConfig) -> Result<Self, AnalysisError> {
        let secret_patterns = Self::build_secret_patterns()?;

        Ok(Self {
            config: config.clone(),
            secret_patterns,
        })
    }

    fn analyze(&self, content: &str) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            let line_number = line_num + 1;

            // Skip comments and empty lines
            if line.trim().is_empty() || line.trim().starts_with('#') {
                continue;
            }

            // Parse environment variable assignment
            if let Some((key, value)) = self.parse_env_line(line) {
                issues.extend(self.analyze_env_variable(&key, &value, line_number)?);
            }
        }

        // Check for structural issues
        issues.extend(self.check_structural_issues(content)?);

        Ok(issues)
    }

    fn supported_type(&self) -> ConfigType {
        ConfigType::Environment
    }

    fn validate_syntax(&self, content: &str) -> Result<(), AnalysisError> {
        for (line_num, line) in content.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if !line.contains('=') {
                return Err(AnalysisError::ParseError(
                    format!("Invalid environment variable syntax at line {}: {}", line_num + 1, line)
                ));
            }
        }
        Ok(())
    }
}

impl EnvAnalyzer {
    fn build_secret_patterns() -> Result<Vec<EnvSecretPattern>, AnalysisError> {
        let patterns = vec![
            EnvSecretPattern {
                name: "AWS Access Key".to_string(),
                pattern: Regex::new(r"^AWS_ACCESS_KEY.*=.*AKIA[0-9A-Z]{16}")?,
                severity: ConfigSeverity::Critical,
                description: "AWS access key found in environment variable".to_string(),
            },
            EnvSecretPattern {
                name: "AWS Secret Key".to_string(),
                pattern: Regex::new(r"^AWS_SECRET.*=.*[A-Za-z0-9/+=]{40}")?,
                severity: ConfigSeverity::Critical,
                description: "AWS secret key found in environment variable".to_string(),
            },
            EnvSecretPattern {
                name: "Database Password".to_string(),
                pattern: Regex::new(r"(?i)^[A-Z_]*PASSWORD.*=.*[^\s]{6,}")?,
                severity: ConfigSeverity::High,
                description: "Database password found in environment variable".to_string(),
            },
            EnvSecretPattern {
                name: "API Key".to_string(),
                pattern: Regex::new(r"(?i)^[A-Z_]*API[_-]?KEY.*=.*[A-Za-z0-9_-]{20,}")?,
                severity: ConfigSeverity::High,
                description: "API key found in environment variable".to_string(),
            },
            EnvSecretPattern {
                name: "JWT Secret".to_string(),
                pattern: Regex::new(r"(?i)^[A-Z_]*JWT[_-]?SECRET.*=.*[A-Za-z0-9_+-=]{20,}")?,
                severity: ConfigSeverity::High,
                description: "JWT secret found in environment variable".to_string(),
            },
            EnvSecretPattern {
                name: "Private Key".to_string(),
                pattern: Regex::new(r"(?i)^[A-Z_]*PRIVATE[_-]?KEY.*=.*-----BEGIN.*PRIVATE KEY-----")?,
                severity: ConfigSeverity::Critical,
                description: "Private key found in environment variable".to_string(),
            },
            EnvSecretPattern {
                name: "Database URL".to_string(),
                pattern: Regex::new(r"^DATABASE_URL.*=.*(postgres|mysql|mongodb)://[^/]*:[^@]*@")?,
                severity: ConfigSeverity::Critical,
                description: "Database URL with credentials found".to_string(),
            },
        ];

        Ok(patterns)
    }

    fn parse_env_line(&self, line: &str) -> Option<(String, String)> {
        let line = line.trim();

        if let Some(equals_pos) = line.find('=') {
            let key = line[..equals_pos].trim().to_string();
            let value = line[equals_pos + 1..].trim();

            // Handle quoted values
            let value = if (value.starts_with('"') && value.ends_with('"')) ||
                          (value.starts_with('\'') && value.ends_with('\'')) {
                value[1..value.len()-1].to_string()
            } else {
                value.to_string()
            };

            Some((key, value))
        } else {
            None
        }
    }

    fn analyze_env_variable(
        &self,
        key: &str,
        value: &str,
        line_number: usize,
    ) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Check against secret patterns
        let full_line = format!("{}={}", key, value);
        for pattern in &self.secret_patterns {
            if pattern.pattern.is_match(&full_line) {
                let confidence = self.calculate_confidence(key, value);

                if confidence >= self.config.confidence_threshold {
                    issues.push(utils::create_config_issue(
                        pattern.severity,
                        format!("Environment Variable: {}", pattern.name),
                        pattern.description.clone(),
                        Some(line_number),
                        "Store sensitive values in secure secret management systems",
                        vec!["environment".to_string(), "credential".to_string()],
                    ).with_cwe(798));
                }
            }
        }

        // Check for common security issues
        issues.extend(self.check_common_issues(key, value, line_number)?);

        Ok(issues)
    }

    fn check_common_issues(
        &self,
        key: &str,
        value: &str,
        line_number: usize,
    ) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Check for default/weak values
        if self.is_default_value(value) {
            issues.push(utils::create_config_issue(
                ConfigSeverity::Medium,
                "Default Environment Value",
                format!("Environment variable '{}' has a default/placeholder value", key),
                Some(line_number),
                "Replace with actual configuration values",
                vec!["environment".to_string(), "default-value".to_string()],
            ));
        }

        // Check for debug settings in environment variables
        if key.to_uppercase().contains("DEBUG") && value.to_lowercase() == "true" {
            issues.push(utils::create_config_issue(
                ConfigSeverity::Low,
                "Debug Mode in Environment",
                "Debug mode enabled via environment variable",
                Some(line_number),
                "Disable debug mode in production environments",
                vec!["environment".to_string(), "debug".to_string()],
            ).with_cwe(489));
        }

        // Check for insecure protocols
        if value.starts_with("http://") && !value.contains("localhost") && !value.contains("127.0.0.1") {
            issues.push(utils::create_config_issue(
                ConfigSeverity::Medium,
                "Insecure Protocol in Environment",
                format!("Environment variable '{}' uses insecure HTTP protocol", key),
                Some(line_number),
                "Use HTTPS instead of HTTP for remote connections",
                vec!["environment".to_string(), "insecure-transport".to_string()],
            ).with_cwe(319));
        }

        // Check for file paths that might be sensitive
        if self.is_sensitive_file_path(key, value) {
            issues.push(utils::create_config_issue(
                ConfigSeverity::Low,
                "Sensitive File Path",
                format!("Environment variable '{}' points to potentially sensitive file", key),
                Some(line_number),
                "Ensure file permissions are properly restricted",
                vec!["environment".to_string(), "file-access".to_string()],
            ).with_cwe(200));
        }

        Ok(issues)
    }

    fn check_structural_issues(&self, content: &str) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();
        let mut env_vars = HashSet::new();

        // Check for duplicate variable definitions
        for (line_num, line) in content.lines().enumerate() {
            if let Some((key, _)) = self.parse_env_line(line) {
                if env_vars.contains(&key) {
                    issues.push(utils::create_config_issue(
                        ConfigSeverity::Medium,
                        "Duplicate Environment Variable",
                        format!("Environment variable '{}' is defined multiple times", key),
                        Some(line_num + 1),
                        "Remove duplicate variable definitions",
                        vec!["environment".to_string(), "duplicate".to_string()],
                    ));
                } else {
                    env_vars.insert(key);
                }
            }
        }

        // Check for common missing variables that should be set
        let important_vars = ["NODE_ENV", "RAILS_ENV", "FLASK_ENV", "ENVIRONMENT"];
        let has_env_indicator = important_vars.iter().any(|&var| env_vars.contains(var));

        if !has_env_indicator && env_vars.len() > 5 {
            issues.push(utils::create_config_issue(
                ConfigSeverity::Info,
                "Missing Environment Indicator",
                "No environment indicator variable found (NODE_ENV, RAILS_ENV, etc.)",
                None,
                "Consider adding an environment indicator variable",
                vec!["environment".to_string(), "best-practice".to_string()],
            ));
        }

        Ok(issues)
    }

    fn calculate_confidence(&self, key: &str, value: &str) -> f64 {
        let mut confidence = 0.8;

        // Higher confidence for well-known secret variable names
        let secret_indicators = ["password", "secret", "key", "token", "private"];
        if secret_indicators.iter().any(|&indicator| key.to_lowercase().contains(indicator)) {
            confidence = (confidence * 1.2).min(1.0);
        }

        // Lower confidence for test/development values
        let test_indicators = ["test", "dev", "example", "demo", "local"];
        if test_indicators.iter().any(|&indicator| {
            key.to_lowercase().contains(indicator) || value.to_lowercase().contains(indicator)
        }) {
            confidence *= 0.4;
        }

        // Higher confidence for production indicators
        let prod_indicators = ["prod", "production", "live"];
        if prod_indicators.iter().any(|&indicator| key.to_lowercase().contains(indicator)) {
            confidence = (confidence * 1.3).min(1.0);
        }

        // Adjust based on value characteristics
        if self.looks_like_real_secret(value) {
            confidence = (confidence * 1.1).min(1.0);
        }

        confidence
    }

    fn is_default_value(&self, value: &str) -> bool {
        let default_values = [
            "changeme", "change-me", "replace-me", "your-secret-here",
            "your_secret_here", "insert-key-here", "put-your-key-here",
            "example", "test", "demo", "placeholder", "todo", "fixme",
            "secret", "password", "admin", "123456",
        ];

        let value_lower = value.to_lowercase();
        default_values.iter().any(|&default| value_lower == default || value_lower.contains(default))
    }

    fn looks_like_real_secret(&self, value: &str) -> bool {
        if value.len() < 8 {
            return false;
        }

        // Check for complexity indicators
        let has_upper = value.chars().any(|c| c.is_uppercase());
        let has_lower = value.chars().any(|c| c.is_lowercase());
        let has_digit = value.chars().any(|c| c.is_numeric());
        let has_special = value.chars().any(|c| !c.is_alphanumeric());

        let complexity_score = [has_upper, has_lower, has_digit, has_special]
            .iter()
            .filter(|&&x| x)
            .count();

        complexity_score >= 2 && value.len() >= 12
    }

    fn is_sensitive_file_path(&self, key: &str, value: &str) -> bool {
        let key_lower = key.to_lowercase();
        let path_indicators = ["file", "path", "cert", "key", "config"];

        if !path_indicators.iter().any(|&indicator| key_lower.contains(indicator)) {
            return false;
        }

        let sensitive_paths = ["/etc/", "/root/", "id_rsa", ".pem", ".p12", ".pfx", "private"];
        sensitive_paths.iter().any(|&path| value.contains(path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_variable_parsing() {
        let config = ConfigSecurityConfig::default();
        let analyzer = EnvAnalyzer::new(&config).unwrap();

        assert_eq!(
            analyzer.parse_env_line("API_KEY=secret123"),
            Some(("API_KEY".to_string(), "secret123".to_string()))
        );

        assert_eq!(
            analyzer.parse_env_line("DATABASE_URL=\"postgres://user:pass@localhost/db\""),
            Some(("DATABASE_URL".to_string(), "postgres://user:pass@localhost/db".to_string()))
        );

        assert_eq!(analyzer.parse_env_line("# Comment line"), None);
    }

    #[test]
    fn test_secret_detection() {
        let config = ConfigSecurityConfig::default();
        let analyzer = EnvAnalyzer::new(&config).unwrap();

        let env_content = r#"
# Database configuration
DATABASE_PASSWORD=SuperSecret123!
AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE
JWT_SECRET=your-jwt-secret-here

# Application settings
DEBUG=true
API_URL=http://api.example.com
"#;

        let issues = analyzer.analyze(env_content).unwrap();
        assert!(!issues.is_empty());

        let secret_issues: Vec<_> = issues.iter()
            .filter(|i| i.tags.contains(&"credential".to_string()))
            .collect();
        assert!(!secret_issues.is_empty());
    }

    #[test]
    fn test_default_value_detection() {
        let config = ConfigSecurityConfig::default();
        let analyzer = EnvAnalyzer::new(&config).unwrap();

        let env_content = r#"
API_KEY=changeme
SECRET_TOKEN=your-secret-here
DATABASE_PASSWORD=replace-me
"#;

        let issues = analyzer.analyze(env_content).unwrap();
        assert!(!issues.is_empty());

        let default_issues: Vec<_> = issues.iter()
            .filter(|i| i.title.contains("Default"))
            .collect();
        assert!(!default_issues.is_empty());
    }

    #[test]
    fn test_duplicate_variable_detection() {
        let config = ConfigSecurityConfig::default();
        let analyzer = EnvAnalyzer::new(&config).unwrap();

        let env_content = r#"
API_KEY=first_value
DATABASE_URL=postgres://localhost/db
API_KEY=second_value
"#;

        let issues = analyzer.analyze(env_content).unwrap();
        assert!(issues.iter().any(|i| i.title.contains("Duplicate")));
    }

    #[test]
    fn test_insecure_protocol_detection() {
        let config = ConfigSecurityConfig::default();
        let analyzer = EnvAnalyzer::new(&config).unwrap();

        let env_content = r#"
API_URL=http://api.example.com
WEBHOOK_URL=http://remote.server.com/hook
LOCAL_URL=http://localhost:3000
"#;

        let issues = analyzer.analyze(env_content).unwrap();
        let insecure_issues: Vec<_> = issues.iter()
            .filter(|i| i.title.contains("Insecure Protocol"))
            .collect();

        // Should detect remote HTTP URLs but not localhost
        assert_eq!(insecure_issues.len(), 2);
    }

    #[test]
    fn test_syntax_validation() {
        let config = ConfigSecurityConfig::default();
        let analyzer = EnvAnalyzer::new(&config).unwrap();

        // Valid env file
        assert!(analyzer.validate_syntax("KEY=value\nOTHER_KEY=other_value").is_ok());

        // Invalid env file (missing =)
        assert!(analyzer.validate_syntax("KEY value\nOTHER_KEY=other_value").is_err());
    }
}