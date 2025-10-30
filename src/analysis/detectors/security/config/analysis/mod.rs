//! Analysis orchestration for configuration security
//!
//! This module coordinates different types of security analysis
//! on configuration files including credential detection,
//! misconfiguration analysis, permission validation, and
//! insecure defaults detection.

pub mod credentials;
pub mod defaults;
pub mod misconfigurations;
pub mod permissions;

// Re-export analyzers
pub use credentials::CredentialAnalyzer;
pub use defaults::DefaultsAnalyzer;
pub use misconfigurations::MisconfigurationAnalyzer;
pub use permissions::PermissionAnalyzer;

use super::config::ConfigSecurityConfig;
use super::types::{ConfigAnalysisContext, ConfigIssue};
use crate::analysis::AnalysisError;

/// Orchestrates all configuration security analysis
pub struct AnalysisOrchestrator {
    credential_analyzer: CredentialAnalyzer,
    misconfiguration_analyzer: MisconfigurationAnalyzer,
    permission_analyzer: PermissionAnalyzer,
    defaults_analyzer: DefaultsAnalyzer,
    config: ConfigSecurityConfig,
}

impl AnalysisOrchestrator {
    /// Create a new analysis orchestrator
    pub fn new(config: &ConfigSecurityConfig) -> Result<Self, AnalysisError> {
        Ok(Self {
            credential_analyzer: CredentialAnalyzer::new(config)?,
            misconfiguration_analyzer: MisconfigurationAnalyzer::new(config)?,
            permission_analyzer: PermissionAnalyzer::new(config)?,
            defaults_analyzer: DefaultsAnalyzer::new(config)?,
            config: config.clone(),
        })
    }

    /// Run comprehensive analysis on configuration content
    pub async fn analyze_comprehensive(
        &self,
        content: &str,
        context: &ConfigAnalysisContext,
    ) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut all_issues = Vec::new();

        // Run enabled analyzers in parallel
        if self.config.enable_credential_detection {
            all_issues.extend(self.credential_analyzer.analyze(content)?);
        }

        if self.config.enable_misconfiguration_detection {
            all_issues.extend(self.misconfiguration_analyzer.analyze(content)?);
        }

        if self.config.enable_permission_detection {
            all_issues.extend(self.permission_analyzer.analyze(content)?);
        }

        if self.config.enable_defaults_detection {
            all_issues.extend(self.defaults_analyzer.analyze(content)?);
        }

        // Filter and rank issues
        let filtered_issues = self.filter_and_rank_issues(all_issues)?;

        Ok(filtered_issues)
    }

    fn filter_and_rank_issues(
        &self,
        mut issues: Vec<ConfigIssue>,
    ) -> Result<Vec<ConfigIssue>, AnalysisError> {
        // Filter by confidence threshold
        issues.retain(|issue| issue.confidence >= self.config.confidence_threshold);

        // Sort by severity and confidence
        issues.sort_by(|a, b| {
            use super::types::ConfigSeverity;
            let severity_order = |s| match s {
                ConfigSeverity::Critical => 5,
                ConfigSeverity::High => 4,
                ConfigSeverity::Medium => 3,
                ConfigSeverity::Low => 2,
                ConfigSeverity::Info => 1,
            };

            let a_score = (severity_order(a.severity), (a.confidence * 100.0) as i32);
            let b_score = (severity_order(b.severity), (b.confidence * 100.0) as i32);

            b_score.cmp(&a_score) // Descending order
        });

        // Limit number of issues
        if issues.len() > self.config.max_issues_per_file {
            issues.truncate(self.config.max_issues_per_file);
        }

        Ok(issues)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::detectors::security::config::types::{ConfigSeverity, ConfigType};
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_comprehensive_analysis() {
        let config = ConfigSecurityConfig::default();
        let orchestrator = AnalysisOrchestrator::new(&config).unwrap();

        let content = r#"
database:
  password: "hardcoded_password"
  debug: true
server:
  cors: "*"
"#;

        let context = ConfigAnalysisContext {
            config_type: ConfigType::Yaml,
            file_path: "/test/config.yaml".to_string(),
            schema: None,
            environment: Some("production".to_string()),
            metadata: HashMap::new(),
        };

        let issues = orchestrator
            .analyze_comprehensive(content, &context)
            .await
            .unwrap();
        assert!(!issues.is_empty());
    }

    #[test]
    fn test_issue_filtering_and_ranking() {
        let config = ConfigSecurityConfig {
            confidence_threshold: 0.7,
            max_issues_per_file: 2,
            ..Default::default()
        };
        let orchestrator = AnalysisOrchestrator::new(&config).unwrap();

        let issues = vec![
            ConfigIssue::new(ConfigSeverity::Low, 0.5, "Low severity", "Low"),
            ConfigIssue::new(ConfigSeverity::High, 0.9, "High severity", "High"),
            ConfigIssue::new(ConfigSeverity::Medium, 0.8, "Medium severity", "Medium"),
            ConfigIssue::new(
                ConfigSeverity::Critical,
                0.95,
                "Critical severity",
                "Critical",
            ),
        ];

        let filtered = orchestrator.filter_and_rank_issues(issues).unwrap();

        // Should filter out low confidence issue and limit to 2 issues
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].severity, ConfigSeverity::Critical);
        assert_eq!(filtered[1].severity, ConfigSeverity::High);
    }
}
