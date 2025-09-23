//! Main compliance validator
//!
//! This module coordinates compliance validation and enhancement of security issues.

use super::super::super::config::ConfigSecurityConfig;
use super::super::super::types::ConfigIssue;
use super::coverage::{ComplianceCoverage, CoverageCalculator};
use super::standards::StandardsBuilder;
use super::types::{ComplianceRequirement, ComplianceStandard};
use crate::analysis::detectors::security::types::SecurityIssue;
use crate::analysis::AnalysisError;
use std::collections::HashMap;

/// Compliance validator for configuration security issues
pub struct ComplianceValidator {
    standards: Vec<ComplianceStandard>,
    config: ConfigSecurityConfig,
}

impl ComplianceValidator {
    /// Create a new compliance validator
    pub fn new(config: &ConfigSecurityConfig) -> Result<Self, AnalysisError> {
        let standards = StandardsBuilder::build_compliance_standards(config);

        Ok(Self {
            standards,
            config: config.clone(),
        })
    }

    /// Enhance a configuration issue with compliance information
    pub fn enhance_issue(&self, mut issue: ConfigIssue) -> Result<ConfigIssue, AnalysisError> {
        if !self.config.enable_compliance_validation {
            return Ok(issue);
        }

        // Find applicable compliance requirements
        let mut compliance_mappings = Vec::new();
        let mut enhanced_remediation = issue.remediation.clone().unwrap_or_default();

        for standard in &self.standards {
            if !standard.enabled {
                continue;
            }

            for requirement in &standard.requirements {
                if self.requirement_applies_to_issue(&issue, requirement) {
                    compliance_mappings.push(format!("{}:{}", standard.name, requirement.id));

                    // Enhance remediation with compliance guidance
                    if !requirement.remediation_guidance.is_empty() {
                        enhanced_remediation.push_str(&format!(
                            "\n\nCompliance Guidance ({}): {}",
                            standard.name, requirement.remediation_guidance
                        ));
                    }
                }
            }
        }

        // Add compliance tags
        for mapping in compliance_mappings {
            issue = issue.with_tag(format!("compliance:{}", mapping));
        }

        if !enhanced_remediation.is_empty() {
            issue = issue.with_remediation(enhanced_remediation);
        }

        Ok(issue)
    }

    /// Enhance a security issue with compliance information
    pub fn enhance_security_issue(
        &self,
        issue: SecurityIssue,
    ) -> Result<SecurityIssue, AnalysisError> {
        // For now, return the issue as-is since SecurityIssue enhancement
        // would require modifying the SecurityIssue struct
        Ok(issue)
    }

    /// Get the count of active compliance standards
    pub fn get_active_standards_count(&self) -> usize {
        self.standards.iter().filter(|s| s.enabled).count()
    }

    /// Get compliance coverage report using the CoverageCalculator
    pub fn get_compliance_coverage(
        &self,
        issues: &[ConfigIssue],
    ) -> HashMap<String, ComplianceCoverage> {
        CoverageCalculator::calculate_coverage(&self.standards, issues)
    }

    /// Get all enabled standards
    pub fn get_enabled_standards(&self) -> Vec<&ComplianceStandard> {
        self.standards.iter().filter(|s| s.enabled).collect()
    }

    /// Check if a specific standard is enabled
    pub fn is_standard_enabled(&self, standard_name: &str) -> bool {
        self.standards
            .iter()
            .any(|s| s.name == standard_name && s.enabled)
    }

    /// Get requirements for a specific standard
    pub fn get_standard_requirements(
        &self,
        standard_name: &str,
    ) -> Option<&Vec<ComplianceRequirement>> {
        self.standards
            .iter()
            .find(|s| s.name == standard_name)
            .map(|s| &s.requirements)
    }

    /// Batch enhance multiple issues
    pub fn enhance_issues(
        &self,
        issues: Vec<ConfigIssue>,
    ) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut enhanced_issues = Vec::with_capacity(issues.len());

        for issue in issues {
            enhanced_issues.push(self.enhance_issue(issue)?);
        }

        Ok(enhanced_issues)
    }

    /// Get compliance summary for issues
    pub fn get_compliance_summary(&self, issues: &[ConfigIssue]) -> ComplianceSummaryResult {
        let coverage = self.get_compliance_coverage(issues);
        let summary = CoverageCalculator::generate_summary(&coverage);

        ComplianceSummaryResult {
            coverage,
            summary,
            total_issues: issues.len(),
            enhanced_issues: issues
                .iter()
                .filter(|issue| issue.tags.iter().any(|tag| tag.starts_with("compliance:")))
                .count(),
        }
    }

    fn requirement_applies_to_issue(
        &self,
        issue: &ConfigIssue,
        requirement: &ComplianceRequirement,
    ) -> bool {
        // Check CWE ID match
        if let Some(cwe_id) = issue.cwe_id {
            if requirement.applicable_cwe_ids.contains(&cwe_id) {
                return true;
            }
        }

        // Check tag match
        for req_tag in &requirement.applicable_tags {
            if issue.tags.iter().any(|issue_tag| {
                issue_tag.to_lowercase().contains(&req_tag.to_lowercase())
                    || req_tag.to_lowercase().contains(&issue_tag.to_lowercase())
            }) {
                return true;
            }
        }

        // Check severity threshold
        if let Some(&score) = requirement.severity_mapping.get(&issue.severity) {
            return score > 0.5; // Threshold for applicability
        }

        false
    }
}

/// Combined compliance summary result
#[derive(Debug)]
pub struct ComplianceSummaryResult {
    pub coverage: HashMap<String, ComplianceCoverage>,
    pub summary: super::coverage::ComplianceSummary,
    pub total_issues: usize,
    pub enhanced_issues: usize,
}

impl ComplianceSummaryResult {
    /// Get enhancement rate
    pub fn enhancement_rate(&self) -> f64 {
        if self.total_issues > 0 {
            (self.enhanced_issues as f64 / self.total_issues as f64) * 100.0
        } else {
            0.0
        }
    }

    /// Check if compliance goals are met
    pub fn meets_compliance_goals(&self, threshold: f64) -> bool {
        self.summary.overall_percentage >= threshold
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::super::types::ConfigSeverity;
    use super::*;

    #[test]
    fn test_compliance_validator_creation() {
        let config = ConfigSecurityConfig::default();
        let validator = ComplianceValidator::new(&config).unwrap();
        assert!(validator.get_active_standards_count() > 0);
    }

    #[test]
    fn test_issue_enhancement() {
        let config = ConfigSecurityConfig::default();
        let validator = ComplianceValidator::new(&config).unwrap();

        let issue = ConfigIssue::new(
            ConfigSeverity::High,
            0.8,
            "Test Security Issue",
            "This is a test issue",
        )
        .with_cwe(79)
        .with_tag("injection".to_string());

        let enhanced = validator.enhance_issue(issue).unwrap();
        assert!(enhanced
            .tags
            .iter()
            .any(|tag| tag.starts_with("compliance:")));
    }

    #[test]
    fn test_compliance_coverage() {
        let config = ConfigSecurityConfig::default();
        let validator = ComplianceValidator::new(&config).unwrap();

        let issues = vec![
            ConfigIssue::new(
                ConfigSeverity::High,
                0.8,
                "Access Control Issue",
                "Test issue",
            )
            .with_cwe(284),
            ConfigIssue::new(
                ConfigSeverity::Medium,
                0.6,
                "Crypto Issue",
                "Test crypto issue",
            )
            .with_cwe(327),
        ];

        let coverage = validator.get_compliance_coverage(&issues);
        assert!(!coverage.is_empty());
    }

    #[test]
    fn test_compliance_summary() {
        let config = ConfigSecurityConfig::default();
        let validator = ComplianceValidator::new(&config).unwrap();

        let issues =
            vec![
                ConfigIssue::new(ConfigSeverity::High, 0.8, "Test Issue", "Test description")
                    .with_tag("compliance:OWASP-Top-10-2021:A01".to_string()),
            ];

        let summary = validator.get_compliance_summary(&issues);
        assert_eq!(summary.total_issues, 1);
        assert_eq!(summary.enhanced_issues, 1);
        assert_eq!(summary.enhancement_rate(), 100.0);
    }
}
