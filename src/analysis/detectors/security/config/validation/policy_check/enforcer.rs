//! Policy enforcement logic for security validation

use super::super::super::types::{ConfigIssue, ConfigSeverity};
use super::rules::SecurityPolicy;
use crate::analysis::AnalysisError;

/// Policy enforcement utilities
pub struct PolicyEnforcer;

impl PolicyEnforcer {
    /// Check if an issue meets the severity threshold
    pub fn meets_severity_threshold(
        issue_severity: ConfigSeverity,
        threshold: ConfigSeverity,
    ) -> bool {
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

    /// Check if an issue passes tag filters
    pub fn passes_tag_filters(
        issue: &ConfigIssue,
        policy: &SecurityPolicy,
    ) -> Result<bool, AnalysisError> {
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
            let has_allowed_tag = issue
                .tags
                .iter()
                .any(|tag| allowed_tags.iter().any(|allowed| tag.contains(allowed)));

            if !has_allowed_tag {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Check if an issue passes CWE ID filters
    pub fn passes_cwe_filters(
        issue: &ConfigIssue,
        policy: &SecurityPolicy,
    ) -> Result<bool, AnalysisError> {
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

    /// Check if a line content triggers suppression patterns
    pub fn is_suppressed_by_line(line_content: &str, suppression_patterns: &[String]) -> bool {
        for pattern in suppression_patterns {
            if line_content
                .to_uppercase()
                .contains(&pattern.to_uppercase())
            {
                return true;
            }
        }
        false
    }

    /// Validate a single issue against a policy
    pub fn validate_issue_against_policy(
        issue: &ConfigIssue,
        policy: &SecurityPolicy,
    ) -> Result<bool, AnalysisError> {
        if !policy.enabled {
            return Ok(false);
        }

        // Check severity threshold
        if !Self::meets_severity_threshold(issue.severity, policy.severity_threshold) {
            return Ok(false);
        }

        // Check confidence threshold
        if issue.confidence < policy.confidence_threshold {
            return Ok(false);
        }

        // Check tag filters
        if !Self::passes_tag_filters(issue, policy)? {
            return Ok(false);
        }

        // Check CWE ID filters
        if !Self::passes_cwe_filters(issue, policy)? {
            return Ok(false);
        }

        Ok(true)
    }

    /// Convert security severity to config severity
    pub fn security_severity_to_config_severity(
        severity: crate::analysis::detectors::security::types::SecuritySeverity,
    ) -> ConfigSeverity {
        match severity {
            crate::analysis::detectors::security::types::SecuritySeverity::Critical => {
                ConfigSeverity::Critical
            }
            crate::analysis::detectors::security::types::SecuritySeverity::High => {
                ConfigSeverity::High
            }
            crate::analysis::detectors::security::types::SecuritySeverity::Medium => {
                ConfigSeverity::Medium
            }
            crate::analysis::detectors::security::types::SecuritySeverity::Low => {
                ConfigSeverity::Low
            }
            crate::analysis::detectors::security::types::SecuritySeverity::Info => {
                ConfigSeverity::Info
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_threshold() {
        assert!(PolicyEnforcer::meets_severity_threshold(
            ConfigSeverity::Critical,
            ConfigSeverity::High
        ));
        assert!(PolicyEnforcer::meets_severity_threshold(
            ConfigSeverity::High,
            ConfigSeverity::High
        ));
        assert!(!PolicyEnforcer::meets_severity_threshold(
            ConfigSeverity::Medium,
            ConfigSeverity::High
        ));
    }

    #[test]
    fn test_tag_filtering() {
        let policy = SecurityPolicy::new("Test Policy".to_string())
            .with_blocked_tags(vec!["test".to_string()]);

        let issue = ConfigIssue::new(ConfigSeverity::High, 0.8, "Test Issue", "Description")
            .with_tag("test-credential");

        assert!(!PolicyEnforcer::passes_tag_filters(&issue, &policy).unwrap());
    }

    #[test]
    fn test_line_suppression() {
        let patterns = vec!["UVEDDI:IGNORE".to_string()];
        assert!(PolicyEnforcer::is_suppressed_by_line(
            "password=secret # UVEDDI:IGNORE",
            &patterns
        ));
        assert!(!PolicyEnforcer::is_suppressed_by_line(
            "password=secret # normal comment",
            &patterns
        ));
    }
}
