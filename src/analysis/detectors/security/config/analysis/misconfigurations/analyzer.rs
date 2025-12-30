//! Main misconfiguration analyzer
//!
//! This module coordinates all misconfiguration detection including rule-based
//! pattern matching and specialized configuration checks.

use super::super::super::config::ConfigSecurityConfig;
use super::super::super::types::ConfigIssue;
use super::checks::{DatabaseChecker, DebugChecker, SessionChecker};
use super::patterns::PatternBuilder;
use super::rules::{MisconfigurationPattern, MisconfigurationRule};
use crate::analysis::detectors::security::config::language_support::utils;
use crate::analysis::AnalysisError;
use serde_yaml::Value as YamlValue;

/// Analyzes configuration files for security misconfigurations
pub struct MisconfigurationAnalyzer {
    rules: Vec<MisconfigurationRule>,
    config: ConfigSecurityConfig,
    debug_checker: DebugChecker,
    session_checker: SessionChecker,
    database_checker: DatabaseChecker,
}

impl MisconfigurationAnalyzer {
    /// Create a new misconfiguration analyzer
    pub fn new(config: &ConfigSecurityConfig) -> Result<Self, AnalysisError> {
        let mut rules = MisconfigurationRule::build_default_rules()?;
        rules.extend(MisconfigurationRule::build_framework_rules()?);

        Ok(Self {
            rules,
            config: config.clone(),
            debug_checker: DebugChecker::new(config),
            session_checker: SessionChecker::new(config),
            database_checker: DatabaseChecker::new(config),
        })
    }

    /// Analyze content for security misconfigurations
    pub fn analyze(&self, content: &str) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Text-based pattern matching
        for (line_num, line) in content.lines().enumerate() {
            for rule in &self.rules {
                if let Some(issue) =
                    PatternBuilder::check_line_against_rule(line, line_num + 1, rule)?
                {
                    issues.push(issue);
                }
            }
        }

        // Structured analysis for YAML/JSON
        if let Ok(yaml_value) = serde_yaml::from_str::<YamlValue>(content) {
            issues.extend(self.analyze_structured_config(&yaml_value)?);
        }

        Ok(issues)
    }

    /// Analyze structured configuration data
    fn analyze_structured_config(
        &self,
        value: &YamlValue,
    ) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Run specialized checks
        issues.extend(self.debug_checker.check_debug_in_production(value)?);
        issues.extend(self.debug_checker.check_verbose_logging(value));

        issues.extend(self.session_checker.check_weak_session_config(value)?);

        issues.extend(
            self.database_checker
                .check_insecure_database_config(value)?,
        );
        issues.extend(self.database_checker.check_connection_pooling(value));

        // Check for structured patterns
        issues.extend(self.check_structured_patterns(value)?);

        Ok(issues)
    }

    /// Check structured patterns against the YAML value
    fn check_structured_patterns(
        &self,
        value: &YamlValue,
    ) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();

        for rule in &self.rules {
            if let MisconfigurationPattern::Structured(check_fn) = &rule.pattern {
                if check_fn(value) {
                    let mut issue = utils::create_config_issue(
                        rule.severity.clone(),
                        rule.name.clone(),
                        rule.description.clone(),
                        None,
                        rule.remediation.clone(),
                        vec!["misconfiguration".to_string()],
                    );

                    if let Some(cwe_id) = rule.cwe_id {
                        issue = issue.with_cwe(cwe_id);
                    }

                    issues.push(issue);
                }
            }
        }

        Ok(issues)
    }

    /// Add custom rules to the analyzer
    pub fn add_custom_rule(&mut self, rule: MisconfigurationRule) {
        self.rules.push(rule);
    }

    /// Get all available rules
    pub fn get_rules(&self) -> &[MisconfigurationRule] {
        &self.rules
    }

    /// Check for specific OWASP categories
    pub fn check_owasp_category(
        &self,
        content: &str,
        category: &str,
    ) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let all_issues = self.analyze(content)?;
        Ok(all_issues
            .into_iter()
            .filter(|issue| {
                // Filter by OWASP category if we had access to it in ConfigIssue
                // For now, filter by tags or title content
                issue
                    .tags
                    .iter()
                    .any(|tag| tag.contains(&category.to_lowercase()))
            })
            .collect())
    }

    /// Get statistics about detected misconfigurations
    pub fn get_analysis_stats(
        &self,
        content: &str,
    ) -> Result<MisconfigurationStats, AnalysisError> {
        let issues = self.analyze(content)?;

        let mut stats = MisconfigurationStats::default();
        stats.total_issues = issues.len();

        for issue in issues {
            match issue.severity {
                super::super::super::types::ConfigSeverity::Critical => stats.critical_count += 1,
                super::super::super::types::ConfigSeverity::High => stats.high_count += 1,
                super::super::super::types::ConfigSeverity::Medium => stats.medium_count += 1,
                super::super::super::types::ConfigSeverity::Low => stats.low_count += 1,
                super::super::super::types::ConfigSeverity::Info => stats.info_count += 1,
            }
        }

        Ok(stats)
    }
}

/// Statistics about misconfiguration analysis
#[derive(Debug, Default)]
pub struct MisconfigurationStats {
    pub total_issues: usize,
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    pub info_count: usize,
}

impl MisconfigurationStats {
    /// Calculate risk score based on issue severity distribution
    pub fn risk_score(&self) -> f64 {
        let weighted_score = (self.critical_count as f64 * 10.0)
            + (self.high_count as f64 * 7.0)
            + (self.medium_count as f64 * 4.0)
            + (self.low_count as f64 * 2.0)
            + (self.info_count as f64 * 1.0);

        // Normalize to 0-100 scale
        (weighted_score / (self.total_issues as f64).max(1.0) * 10.0).min(100.0)
    }

    /// Get the most severe issue level present
    pub fn max_severity(&self) -> super::super::super::types::ConfigSeverity {
        if self.critical_count > 0 {
            super::super::super::types::ConfigSeverity::Critical
        } else if self.high_count > 0 {
            super::super::super::types::ConfigSeverity::High
        } else if self.medium_count > 0 {
            super::super::super::types::ConfigSeverity::Medium
        } else if self.low_count > 0 {
            super::super::super::types::ConfigSeverity::Low
        } else {
            super::super::super::types::ConfigSeverity::Info
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_misconfiguration_analyzer_creation() {
        let config = ConfigSecurityConfig::default();
        let analyzer = MisconfigurationAnalyzer::new(&config).unwrap();
        assert!(!analyzer.rules.is_empty());
    }

    #[test]
    fn test_comprehensive_analysis() {
        let config = ConfigSecurityConfig::default();
        let analyzer = MisconfigurationAnalyzer::new(&config).unwrap();

        let config_content = r#"
# Application configuration
environment: production
debug: true
cors_allow_origin: "*"
access_control_allow_origin: "*"

# Session configuration (structured for YAML parsing)
session:
  secure: false
  httponly: false

# Database configuration
ssl_enabled: false
database_port: 3306

# Logging
log_level: debug
"#;

        let issues = analyzer.analyze(config_content).unwrap();
        assert!(!issues.is_empty());

        // Should detect multiple types of issues
        assert!(issues.iter().any(|i| i.title.contains("Debug")));
        assert!(issues.iter().any(|i| i.title.contains("CORS")));
        assert!(issues.iter().any(|i| i.title.contains("Session")));
        assert!(issues.iter().any(|i| i.title.contains("SSL")));
    }

    #[test]
    fn test_analysis_stats() {
        let config = ConfigSecurityConfig::default();
        let analyzer = MisconfigurationAnalyzer::new(&config).unwrap();

        let config_content = r#"
debug: true
cors: "*"
ssl_enabled: false
"#;

        let stats = analyzer.get_analysis_stats(config_content).unwrap();
        assert!(stats.total_issues > 0);
        assert!(stats.risk_score() > 0.0);
    }

    #[test]
    fn test_safe_configuration() {
        let config = ConfigSecurityConfig::default();
        let analyzer = MisconfigurationAnalyzer::new(&config).unwrap();

        let safe_content = r#"
environment: production
debug: false
cors:
  allow_origin: "https://trusted-domain.com"
session:
  secure: true
  httponly: true
database:
  ssl: true
  port: 5433
log_level: info
"#;

        let issues = analyzer.analyze(safe_content).unwrap();
        // Should have minimal issues for well-configured content
        assert!(issues.len() <= 1);
    }
}
