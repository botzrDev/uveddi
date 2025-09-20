//! Core policy validation engine

use super::{enforcer::PolicyEnforcer, rules::{SecurityPolicy, build_default_policies}};
use crate::analysis::AnalysisError;
use crate::analysis::detectors::security::types::SecurityIssue;
use super::super::super::config::ConfigSecurityConfig;
use super::super::super::types::{ConfigIssue, ConfigSeverity};
use std::collections::HashMap;

/// Policy validator for configuration security issues
pub struct PolicyValidator {
    policies: Vec<SecurityPolicy>,
    config: ConfigSecurityConfig,
}

impl PolicyValidator {
    /// Create a new policy validator
    pub fn new(config: &ConfigSecurityConfig) -> Result<Self, AnalysisError> {
        let policies = build_default_policies(config);

        Ok(Self {
            policies,
            config: config.clone(),
        })
    }

    /// Validate a configuration issue against policies
    pub fn validate_issue(&self, issue: &ConfigIssue) -> Result<bool, AnalysisError> {
        for policy in &self.policies {
            if PolicyEnforcer::validate_issue_against_policy(issue, policy)? {
                return Ok(true);
            }
        }

        // If no policy accepts the issue, check if we have a permissive default
        Ok(self.has_permissive_default())
    }

    /// Validate a security issue against policies
    pub fn validate_security_issue(&self, issue: &SecurityIssue) -> Result<bool, AnalysisError> {
        // Convert security issue to config issue for validation
        let config_issue = ConfigIssue::new(
            PolicyEnforcer::security_severity_to_config_severity(issue.severity),
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

            if PolicyEnforcer::is_suppressed_by_line(line_content, &policy.line_suppression_patterns) {
                return true;
            }
        }

        false
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

    fn has_permissive_default(&self) -> bool {
        // If no policies are enabled, default to permissive
        !self.policies.iter().any(|p| p.enabled)
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
        let custom_policy = SecurityPolicy::new("Custom Policy".to_string())
            .with_severity_threshold(ConfigSeverity::Medium)
            .with_confidence_threshold(0.5);

        validator.add_policy(custom_policy);
        assert_eq!(validator.get_active_policy_count(), initial_count + 1);

        // Remove the custom policy
        assert!(validator.remove_policy("Custom Policy"));
        assert_eq!(validator.get_active_policy_count(), initial_count);
    }
}