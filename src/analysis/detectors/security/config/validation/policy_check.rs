//! Security policy validation for configuration analysis
//!
//! This module validates configuration security findings against
//! organizational security policies and filtering rules.

use crate::analysis::AnalysisError;
use crate::analysis::detectors::security::types::SecurityIssue;
use super::super::config::ConfigSecurityConfig;
use super::super::types::{ConfigIssue, ConfigSeverity};
use std::collections::HashMap;

/// Policy validator for configuration security issues
pub struct PolicyValidator {
    policies: Vec<SecurityPolicy>,
    config: ConfigSecurityConfig,
}

/// Security policy definition
#[derive(Debug, Clone)]
pub struct SecurityPolicy {
    pub name: String,
    pub enabled: bool,
    pub severity_threshold: ConfigSeverity,
    pub confidence_threshold: f64,
    pub allowed_tags: Option<Vec<String>>,
    pub blocked_tags: Option<Vec<String>>,
    pub allowed_cwe_ids: Option<Vec<u32>>,
    pub blocked_cwe_ids: Option<Vec<u32>>,
    pub file_pattern_filters: Vec<String>,
    pub line_suppression_patterns: Vec<String>,
}

impl PolicyValidator {
    /// Create a new policy validator
    pub fn new(config: &ConfigSecurityConfig) -> Result<Self, AnalysisError> {
        let policies = Self::build_default_policies(config);

        Ok(Self {
            policies,
            config: config.clone(),
        })
    }

    /// Validate a configuration issue against policies
    pub fn validate_issue(&self, issue: &ConfigIssue) -> Result<bool, AnalysisError> {
        for policy in &self.policies {
            if !policy.enabled {
                continue;
            }

            // Check severity threshold
            if !self.meets_severity_threshold(issue.severity, policy.severity_threshold) {
                continue;
            }

            // Check confidence threshold
            if issue.confidence < policy.confidence_threshold {
                continue;
            }

            // Check tag filters
            if !self.passes_tag_filters(issue, policy)? {
                continue;
            }

            // Check CWE ID filters
            if !self.passes_cwe_filters(issue, policy)? {
                continue;
            }

            // If we reach here, the issue passes this policy
            return Ok(true);
        }

        // If no policy accepts the issue, check if we have a permissive default
        Ok(self.has_permissive_default())
    }

    /// Validate a security issue against policies
    pub fn validate_security_issue(&self, issue: &SecurityIssue) -> Result<bool, AnalysisError> {
        // Convert security issue to config issue for validation
        let config_issue = ConfigIssue::new(
            self.security_severity_to_config_severity(issue.severity),
            issue.confidence_score,
            issue.title.clone(),
            issue.description.clone(),
        );

        self.validate_issue(&config_issue)
    }

    /// Get the count of active policies
    pub fn get_active_policy_count(&self) -> usize {
        self.policies.iter().filter(|p| p.enabled).count()
    }

    /// Check if an issue should be suppressed based on line patterns
    pub fn is_suppressed_by_line(&self, line_content: &str) -> bool {
        for policy in &self.policies {
            if !policy.enabled {
                continue;
            }

            for pattern in &policy.line_suppression_patterns {
                if line_content.to_uppercase().contains(&pattern.to_uppercase()) {
                    return true;
                }
            }
        }

        false
    }

    fn build_default_policies(config: &ConfigSecurityConfig) -> Vec<SecurityPolicy> {
        let mut policies = Vec::new();

        // High-severity policy - only critical and high severity issues
        policies.push(SecurityPolicy {
            name: "High Severity Policy".to_string(),
            enabled: true,
            severity_threshold: ConfigSeverity::High,
            confidence_threshold: config.confidence_threshold,
            allowed_tags: None,
            blocked_tags: Some(vec!["test".to_string(), "example".to_string()]),
            allowed_cwe_ids: None,
            blocked_cwe_ids: None,
            file_pattern_filters: vec![],
            line_suppression_patterns: vec![
                "UVEDDI:IGNORE".to_string(),
                "SECURITY:OK".to_string(),
                "FALSE-POSITIVE".to_string(),
                "NOSONAR".to_string(),
            ],
        });

        // Credential-specific policy
        policies.push(SecurityPolicy {
            name: "Credential Protection Policy".to_string(),
            enabled: true,
            severity_threshold: ConfigSeverity::Medium,
            confidence_threshold: 0.6, // Lower threshold for credentials
            allowed_tags: Some(vec![
                "credential".to_string(),
                "secret".to_string(),
                "password".to_string(),
                "api-key".to_string(),
            ]),
            blocked_tags: Some(vec!["test-credential".to_string()]),
            allowed_cwe_ids: Some(vec![798, 521, 522]), // Common credential CWEs
            blocked_cwe_ids: None,
            file_pattern_filters: vec![
                "*.test.*".to_string(),
                "*.example.*".to_string(),
                "**/test/**".to_string(),
            ],
            line_suppression_patterns: vec![
                "TEST-CREDENTIAL".to_string(),
                "EXAMPLE-PASSWORD".to_string(),
            ],
        });

        // Production environment policy
        policies.push(SecurityPolicy {
            name: "Production Environment Policy".to_string(),
            enabled: true,
            severity_threshold: ConfigSeverity::Low,
            confidence_threshold: 0.8, // Higher threshold for production
            allowed_tags: None,
            blocked_tags: Some(vec![
                "development".to_string(),
                "local".to_string(),
                "test".to_string(),
            ]),
            allowed_cwe_ids: None,
            blocked_cwe_ids: None,
            file_pattern_filters: vec![
                "**/dev/**".to_string(),
                "**/test/**".to_string(),
                "**/local/**".to_string(),
            ],
            line_suppression_patterns: vec![],
        });

        // Compliance-focused policy
        policies.push(SecurityPolicy {
            name: "Compliance Policy".to_string(),
            enabled: config.enable_compliance_validation,
            severity_threshold: ConfigSeverity::Info,
            confidence_threshold: 0.7,
            allowed_tags: None,
            blocked_tags: None,
            allowed_cwe_ids: Some(vec![
                // OWASP Top 10 related CWEs
                79, 89, 22, 352, 863, 94, 287, 798, 311, 918,
                // Common configuration CWEs
                1188, 489, 319, 326, 327, 250, 732,
            ]),
            blocked_cwe_ids: None,
            file_pattern_filters: vec![],
            line_suppression_patterns: vec![],
        });

        policies
    }

    fn meets_severity_threshold(&self, issue_severity: ConfigSeverity, threshold: ConfigSeverity) -> bool {
        let severity_rank = |s: ConfigSeverity| -> u8 {
            match s {
                ConfigSeverity::Critical => 5,
                ConfigSeverity::High => 4,
                ConfigSeverity::Medium => 3,
                ConfigSeverity::Low => 2,
                ConfigSeverity::Info => 1,
            }
        };

        severity_rank(issue_severity) >= severity_rank(threshold)
    }

    fn passes_tag_filters(&self, issue: &ConfigIssue, policy: &SecurityPolicy) -> Result<bool, AnalysisError> {
        // Check blocked tags first
        if let Some(blocked_tags) = &policy.blocked_tags {
            for blocked_tag in blocked_tags {
                if issue.tags.iter().any(|tag| tag.contains(blocked_tag)) {
                    return Ok(false);
                }
            }
        }

        // Check allowed tags
        if let Some(allowed_tags) = &policy.allowed_tags {
            let has_allowed_tag = issue.tags.iter().any(|tag| {
                allowed_tags.iter().any(|allowed| tag.contains(allowed))
            });

            if !has_allowed_tag {
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn passes_cwe_filters(&self, issue: &ConfigIssue, policy: &SecurityPolicy) -> Result<bool, AnalysisError> {
        if let Some(cwe_id) = issue.cwe_id {
            // Check blocked CWE IDs
            if let Some(blocked_cwe_ids) = &policy.blocked_cwe_ids {
                if blocked_cwe_ids.contains(&cwe_id) {
                    return Ok(false);
                }
            }

            // Check allowed CWE IDs
            if let Some(allowed_cwe_ids) = &policy.allowed_cwe_ids {
                if !allowed_cwe_ids.contains(&cwe_id) {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    fn has_permissive_default(&self) -> bool {
        // If no policies are enabled, default to permissive
        !self.policies.iter().any(|p| p.enabled)
    }

    fn security_severity_to_config_severity(&self, severity: crate::analysis::detectors::security::types::SecuritySeverity) -> ConfigSeverity {
        match severity {
            crate::analysis::detectors::security::types::SecuritySeverity::Critical => ConfigSeverity::Critical,
            crate::analysis::detectors::security::types::SecuritySeverity::High => ConfigSeverity::High,
            crate::analysis::detectors::security::types::SecuritySeverity::Medium => ConfigSeverity::Medium,
            crate::analysis::detectors::security::types::SecuritySeverity::Low => ConfigSeverity::Low,
            crate::analysis::detectors::security::types::SecuritySeverity::Info => ConfigSeverity::Info,
        }
    }

    /// Add a custom policy
    pub fn add_policy(&mut self, policy: SecurityPolicy) {
        self.policies.push(policy);
    }

    /// Remove a policy by name
    pub fn remove_policy(&mut self, name: &str) -> bool {
        let initial_len = self.policies.len();
        self.policies.retain(|p| p.name != name);
        self.policies.len() < initial_len
    }

    /// Update policy configuration
    pub fn update_policy<F>(&mut self, name: &str, update_fn: F) -> bool
    where
        F: FnOnce(&mut SecurityPolicy),
    {
        if let Some(policy) = self.policies.iter_mut().find(|p| p.name == name) {
            update_fn(policy);
            true
        } else {
            false
        }
    }

    /// Get policy statistics
    pub fn get_policy_stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        stats.insert("total_policies".to_string(), self.policies.len());
        stats.insert("enabled_policies".to_string(), self.policies.iter().filter(|p| p.enabled).count());
        stats.insert("policies_with_tag_filters".to_string(),
                     self.policies.iter().filter(|p| p.allowed_tags.is_some() || p.blocked_tags.is_some()).count());
        stats.insert("policies_with_cwe_filters".to_string(),
                     self.policies.iter().filter(|p| p.allowed_cwe_ids.is_some() || p.blocked_cwe_ids.is_some()).count());
        stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_validation() {
        let config = ConfigSecurityConfig::default();
        let validator = PolicyValidator::new(&config).unwrap();

        // High severity issue should pass
        let high_issue = ConfigIssue::new(
            ConfigSeverity::High,
            0.9,
            "High Severity Issue",
            "Test description",
        );

        assert!(validator.validate_issue(&high_issue).unwrap());

        // Low confidence issue should not pass
        let low_confidence_issue = ConfigIssue::new(
            ConfigSeverity::High,
            0.3,
            "Low Confidence Issue",
            "Test description",
        );

        assert!(!validator.validate_issue(&low_confidence_issue).unwrap());
    }

    #[test]
    fn test_tag_filtering() {
        let config = ConfigSecurityConfig::default();
        let validator = PolicyValidator::new(&config).unwrap();

        // Issue with blocked tag should not pass credential policy
        let test_issue = ConfigIssue::new(
            ConfigSeverity::High,
            0.8,
            "Test Credential",
            "Test description",
        ).with_tag("test-credential");

        // Should still pass high severity policy but might fail credential policy
        let result = validator.validate_issue(&test_issue).unwrap();
        assert!(result); // High severity policy should accept it
    }

    #[test]
    fn test_line_suppression() {
        let config = ConfigSecurityConfig::default();
        let validator = PolicyValidator::new(&config).unwrap();

        assert!(validator.is_suppressed_by_line("password=secret # UVEDDI:IGNORE"));
        assert!(!validator.is_suppressed_by_line("password=secret # normal comment"));
    }

    #[test]
    fn test_policy_management() {
        let config = ConfigSecurityConfig::default();
        let mut validator = PolicyValidator::new(&config).unwrap();

        let initial_count = validator.get_active_policy_count();

        // Add a custom policy
        let custom_policy = SecurityPolicy {
            name: "Custom Policy".to_string(),
            enabled: true,
            severity_threshold: ConfigSeverity::Medium,
            confidence_threshold: 0.5,
            allowed_tags: None,
            blocked_tags: None,
            allowed_cwe_ids: None,
            blocked_cwe_ids: None,
            file_pattern_filters: vec![],
            line_suppression_patterns: vec![],
        };

        validator.add_policy(custom_policy);
        assert_eq!(validator.get_active_policy_count(), initial_count + 1);

        // Remove the custom policy
        assert!(validator.remove_policy("Custom Policy"));
        assert_eq!(validator.get_active_policy_count(), initial_count);
    }

    #[test]
    fn test_severity_threshold() {
        let config = ConfigSecurityConfig::default();
        let validator = PolicyValidator::new(&config).unwrap();

        assert!(validator.meets_severity_threshold(ConfigSeverity::Critical, ConfigSeverity::High));
        assert!(validator.meets_severity_threshold(ConfigSeverity::High, ConfigSeverity::High));
        assert!(!validator.meets_severity_threshold(ConfigSeverity::Medium, ConfigSeverity::High));
    }
}