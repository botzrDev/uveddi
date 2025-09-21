//! Configuration security detector - main entry point
//!
//! This module provides the main ConfigSecurityDetector that orchestrates
//! configuration file security analysis across multiple formats and languages.

use crate::analysis::detectors::security::types::{SecurityIssue, SecurityIssueType};
use crate::analysis::AnalysisError;
use std::path::PathBuf;

use super::analysis::{
    CredentialAnalyzer, DefaultsAnalyzer, MisconfigurationAnalyzer, PermissionAnalyzer,
};
use super::config::ConfigSecurityConfig;
use super::language_support::{EnvAnalyzer, TomlAnalyzer, YamlAnalyzer};
use super::patterns::{PatternMatcher, SecretPatternMatcher, VulnerabilityPatternMatcher};
use super::types::{ConfigIssue, ConfigType};
use super::validation::{ComplianceValidator, PolicyValidator};

/// Main configuration security detector
pub struct ConfigSecurityDetector {
    config: ConfigSecurityConfig,
    pattern_matchers: Vec<Box<dyn PatternMatcher>>,
    credential_analyzer: CredentialAnalyzer,
    misconfiguration_analyzer: MisconfigurationAnalyzer,
    permission_analyzer: PermissionAnalyzer,
    defaults_analyzer: DefaultsAnalyzer,
    yaml_analyzer: YamlAnalyzer,
    toml_analyzer: TomlAnalyzer,
    env_analyzer: EnvAnalyzer,
    policy_validator: PolicyValidator,
    compliance_validator: ComplianceValidator,
}

impl ConfigSecurityDetector {
    /// Create a new configuration security detector
    pub fn new(config: ConfigSecurityConfig) -> Result<Self, AnalysisError> {
        let secret_matcher = Box::new(SecretPatternMatcher::new(&config)?);
        let vuln_matcher = Box::new(VulnerabilityPatternMatcher::new(&config)?);
        let pattern_matchers: Vec<Box<dyn PatternMatcher>> = vec![secret_matcher, vuln_matcher];

        Ok(Self {
            credential_analyzer: CredentialAnalyzer::new(&config)?,
            misconfiguration_analyzer: MisconfigurationAnalyzer::new(&config)?,
            permission_analyzer: PermissionAnalyzer::new(&config)?,
            defaults_analyzer: DefaultsAnalyzer::new(&config)?,
            yaml_analyzer: YamlAnalyzer::new(&config)?,
            toml_analyzer: TomlAnalyzer::new(&config)?,
            env_analyzer: EnvAnalyzer::new(&config)?,
            policy_validator: PolicyValidator::new(&config)?,
            compliance_validator: ComplianceValidator::new(&config)?,
            config,
            pattern_matchers,
        })
    }

    /// Analyze a configuration file for security issues
    pub async fn analyze_config(
        &self,
        file_path: &PathBuf,
        content: &str,
    ) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let config_type = self.detect_config_type(file_path)?;
        // Detect configuration-specific security issues
        let config_issues = self.detect_security_issues(config_type, content)?;

        // Apply policy and compliance validation at the ConfigIssue level
        let validated_config_issues = self.validate_config_issues(config_issues)?;

        // Convert to SecurityIssue
        let mut issues = Vec::new();
        for config_issue in validated_config_issues {
            issues.push(self.convert_to_security_issue(config_issue, file_path)?);
        }

        Ok(issues)
    }

    /// Detect security issues in configuration content
    pub fn detect_security_issues(
        &self,
        config_type: ConfigType,
        content: &str,
    ) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Language-specific analysis
        issues.extend(match config_type {
            ConfigType::Yaml | ConfigType::Json => self.yaml_analyzer.analyze(content)?,
            ConfigType::Toml => self.toml_analyzer.analyze(content)?,
            ConfigType::Environment => self.env_analyzer.analyze(content)?,
        });

        // Cross-cutting security analysis
        issues.extend(self.credential_analyzer.analyze(content)?);
        issues.extend(self.misconfiguration_analyzer.analyze(content)?);
        issues.extend(self.permission_analyzer.analyze(content)?);
        issues.extend(self.defaults_analyzer.analyze(content)?);

        Ok(issues)
    }

    fn detect_config_type(&self, file_path: &PathBuf) -> Result<ConfigType, AnalysisError> {
        if let Some(extension) = file_path.extension().and_then(|ext| ext.to_str()) {
            match extension.to_lowercase().as_str() {
                "yaml" | "yml" => Ok(ConfigType::Yaml),
                "json" => Ok(ConfigType::Json),
                "toml" => Ok(ConfigType::Toml),
                "env" => Ok(ConfigType::Environment),
                _ => Err(AnalysisError::UnsupportedFileType(extension.to_string())),
            }
        } else {
            Err(AnalysisError::InvalidFilePath(file_path.clone()))
        }
    }

    fn convert_to_security_issue(
        &self,
        config_issue: ConfigIssue,
        file_path: &PathBuf,
    ) -> Result<SecurityIssue, AnalysisError> {
        Ok(SecurityIssue {
            issue_type: SecurityIssueType::ConfigurationVulnerability,
            severity: config_issue.severity,
            confidence_score: config_issue.confidence,
            title: config_issue.title,
            description: config_issue.description,
            file_path: file_path.clone(),
            line_number: config_issue.line_number,
            column_number: config_issue.column_number,
            snippet: config_issue.code_snippet,
            remediation: config_issue.remediation,
            references: config_issue.references,
            owasp_category: config_issue.owasp_category,
            cwe_id: config_issue.cwe_id,
            tags: config_issue.tags,
        })
    }

    fn validate_config_issues(
        &self,
        issues: Vec<ConfigIssue>,
    ) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut validated = Vec::new();

        for issue in issues {
            // Apply policy validation
            if self.policy_validator.validate_issue(&issue)? {
                // Apply compliance validation
                let compliant_issue = self.compliance_validator.enhance_issue(issue)?;
                validated.push(compliant_issue);
            }
        }

        Ok(validated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[tokio::test]
    async fn test_yaml_config_analysis() {
        let config = ConfigSecurityConfig::default();
        let detector = ConfigSecurityDetector::new(config).unwrap();

        let yaml_content = r#"
database:
  password: "hardcoded_password123"
  host: "localhost"
api:
  debug: true
  cors: "*"
"#;

        let file_path = PathBuf::from("/test/config.yaml");
        let issues = detector
            .analyze_config(&file_path, yaml_content)
            .await
            .unwrap();

        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.title.contains("hardcoded")));
    }

    #[tokio::test]
    async fn test_env_file_analysis() {
        let config = ConfigSecurityConfig::default();
        let detector = ConfigSecurityDetector::new(config).unwrap();

        let env_content = r#"
DATABASE_PASSWORD=secret123
API_KEY=sk-1234567890abcdef
DEBUG=true
"#;

        let file_path = PathBuf::from("/test/.env");
        let issues = detector
            .analyze_config(&file_path, env_content)
            .await
            .unwrap();

        assert!(!issues.is_empty());
    }

    #[test]
    fn test_config_type_detection() {
        let config = ConfigSecurityConfig::default();
        let detector = ConfigSecurityDetector::new(config).unwrap();

        assert_eq!(
            detector
                .detect_config_type(&PathBuf::from("config.yaml"))
                .unwrap(),
            ConfigType::Yaml
        );
        assert_eq!(
            detector
                .detect_config_type(&PathBuf::from("Cargo.toml"))
                .unwrap(),
            ConfigType::Toml
        );
        assert_eq!(
            detector.detect_config_type(&PathBuf::from(".env")).unwrap(),
            ConfigType::Environment
        );
    }
}
