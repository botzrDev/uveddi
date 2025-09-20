//! Main environment variable analyzer
//!
//! This module coordinates all environment variable security checks
//! including secret detection, validation, and structural analysis.

use crate::analysis::AnalysisError;
use super::super::super::config::ConfigSecurityConfig;
use super::super::super::types::{ConfigIssue, ConfigType};
use super::super::{LanguageAnalyzer, utils};
use super::secret_patterns::EnvSecretPatternChecker;
use super::validator::EnvValidator;
use super::parser::EnvParser;

/// Analyzer for environment variable files (.env)
pub struct EnvAnalyzer {
    config: ConfigSecurityConfig,
    secret_checker: EnvSecretPatternChecker,
    validator: EnvValidator,
}

impl LanguageAnalyzer for EnvAnalyzer {
    fn new(config: &ConfigSecurityConfig) -> Result<Self, AnalysisError> {
        let secret_checker = EnvSecretPatternChecker::new(config)?;
        let validator = EnvValidator::new(config);

        Ok(Self {
            config: config.clone(),
            secret_checker,
            validator,
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
            if let Some((key, value)) = EnvParser::parse_env_line(line) {
                issues.extend(self.analyze_env_variable(&key, &value, line_number)?);
            }
        }

        // Check for structural issues
        issues.extend(self.validator.check_structural_issues(content));

        Ok(issues)
    }

    fn supported_type(&self) -> ConfigType {
        ConfigType::Environment
    }

    fn validate_syntax(&self, content: &str) -> Result<(), AnalysisError> {
        EnvParser::validate_syntax(content)
    }
}

impl EnvAnalyzer {
    fn analyze_env_variable(
        &self,
        key: &str,
        value: &str,
        line_number: usize,
    ) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Check for secret patterns
        issues.extend(self.secret_checker.check_variable(key, value, line_number));

        // Check for common security issues
        issues.extend(self.validator.check_common_issues(key, value, line_number));

        // Check for file path security issues
        issues.extend(self.check_file_path_issues(key, value, line_number)?);

        Ok(issues)
    }

    fn check_file_path_issues(&self, key: &str, value: &str, line_number: usize) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();

        if self.is_sensitive_file_path(key, value) {
            issues.push(utils::create_config_issue(
                super::super::super::types::ConfigSeverity::Medium,
                "Sensitive File Path".to_string(),
                format!("Environment variable '{}' contains a sensitive file path", key),
                Some(line_number),
                "Ensure file paths don't expose sensitive information".to_string(),
                vec!["environment".to_string(), "file-path".to_string()],
            ).with_cwe(200));
        }

        Ok(issues)
    }

    fn is_sensitive_file_path(&self, key: &str, value: &str) -> bool {
        let key_lower = key.to_lowercase();
        let path_indicators = ["file", "path", "cert", "key", "config"];

        if !path_indicators.iter().any(|&indicator| key_lower.contains(indicator)) {
            return false;
        }

        let sensitive_paths = [
            "/etc/", "/root/", "/home/", "/.ssh/", "/var/", "/tmp/",
            "private", "secret", "credential", ".pem", ".key", ".p12",
        ];

        let value_lower = value.to_lowercase();
        sensitive_paths.iter().any(|&path| value_lower.contains(path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_analyzer_creation() {
        let config = ConfigSecurityConfig::default();
        let analyzer = EnvAnalyzer::new(&config).unwrap();
        assert_eq!(analyzer.supported_type(), ConfigType::Environment);
    }

    #[test]
    fn test_comprehensive_analysis() {
        let config = ConfigSecurityConfig::default();
        let analyzer = EnvAnalyzer::new(&config).unwrap();

        let env_content = r#"
# Database configuration
DB_PASSWORD=supersecret123
API_KEY=sk-1234567890abcdefghij
AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE

# Insecure settings
DEBUG=true
API_URL=http://api.example.com
LOCALHOST_HOST=localhost

# Default values
SECRET_KEY=changeme
"#;

        let issues = analyzer.analyze(env_content).unwrap();
        assert!(!issues.is_empty());

        // Should detect multiple types of issues
        assert!(issues.iter().any(|i| i.title.contains("Database Password")));
        assert!(issues.iter().any(|i| i.title.contains("API Key")));
        assert!(issues.iter().any(|i| i.title.contains("AWS Access Key")));
        assert!(issues.iter().any(|i| i.title.contains("Debug Mode")));
        assert!(issues.iter().any(|i| i.title.contains("HTTP URL")));
        assert!(issues.iter().any(|i| i.title.contains("Default Environment")));
    }

    #[test]
    fn test_syntax_validation() {
        let config = ConfigSecurityConfig::default();
        let analyzer = EnvAnalyzer::new(&config).unwrap();

        // Valid syntax
        let valid_content = r#"
KEY1=value1
KEY2="quoted value"
KEY3='single quoted'
# Comment
"#;
        assert!(analyzer.validate_syntax(valid_content).is_ok());

        // Invalid syntax
        let invalid_content = "INVALID LINE WITHOUT EQUALS";
        assert!(analyzer.validate_syntax(invalid_content).is_err());
    }

    #[test]
    fn test_file_path_detection() {
        let config = ConfigSecurityConfig::default();
        let analyzer = EnvAnalyzer::new(&config).unwrap();

        let issues = analyzer.check_file_path_issues("SSL_CERT_PATH", "/etc/ssl/private/cert.pem", 1).unwrap();
        assert!(issues.iter().any(|i| i.title.contains("Sensitive File Path")));
    }

    #[test]
    fn test_safe_configuration() {
        let config = ConfigSecurityConfig::default();
        let analyzer = EnvAnalyzer::new(&config).unwrap();

        let safe_content = r#"
PORT=3000
NODE_ENV=production
LOG_LEVEL=info
"#;

        let issues = analyzer.analyze(safe_content).unwrap();
        // Should have minimal or no issues for safe config
        assert!(issues.len() <= 1);
    }
}