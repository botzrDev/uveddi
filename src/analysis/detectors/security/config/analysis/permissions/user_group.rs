//! User and group permission analysis
//!
//! This module detects inappropriate user and group assignments
//! in configuration files, including root user execution.

use crate::analysis::AnalysisError;
use super::super::super::types::{ConfigIssue, ConfigSeverity};
use super::super::super::config::ConfigSecurityConfig;
use regex::Regex;

/// Rule for detecting user/group permission issues
pub struct UserGroupRule {
    pub name: String,
    pub description: String,
    pub severity: ConfigSeverity,
    pub user_pattern: Regex,
    pub group_pattern: Option<Regex>,
    pub remediation: String,
    pub cwe_id: Option<u32>,
    pub owasp_category: Option<String>,
}

/// Checker for user and group security issues
pub struct UserGroupChecker {
    rules: Vec<UserGroupRule>,
}

impl UserGroupChecker {
    pub fn new(_config: &ConfigSecurityConfig) -> Result<Self, AnalysisError> {
        let rules = Self::build_user_group_rules()?;
        Ok(Self { rules })
    }

    pub fn check_line(&self, line: &str, line_number: usize) -> Vec<ConfigIssue> {
        let mut issues = Vec::new();

        for rule in &self.rules {
            let mut matched = false;

            // Check user pattern
            if rule.user_pattern.is_match(line) {
                matched = true;
            }

            // Check group pattern if present
            if let Some(ref group_pattern) = rule.group_pattern {
                if group_pattern.is_match(line) {
                    matched = true;
                }
            }

            if matched {
                let issue = ConfigIssue::new(
                    rule.severity,
                    0.9, // High confidence for regex matches
                    rule.name.clone(),
                    rule.description.clone(),
                )
                .with_location(line_number, 1)
                .with_snippet(line.to_string())
                .with_remediation(rule.remediation.clone());

                let mut final_issue = if let Some(cwe_id) = rule.cwe_id {
                    issue.with_cwe(cwe_id)
                } else {
                    issue
                };

                if let Some(ref owasp_cat) = rule.owasp_category {
                    final_issue = final_issue.with_tag(owasp_cat.clone());
                }

                issues.push(final_issue);
            }
        }

        issues
    }

    fn build_user_group_rules() -> Result<Vec<UserGroupRule>, AnalysisError> {
        let rules = vec![
            UserGroupRule {
                name: "Root User Execution".to_string(),
                description: "Service configured to run as root user".to_string(),
                severity: ConfigSeverity::High,
                user_pattern: Regex::new(r#"(?i)(user|run[_-]?as[_-]?user)[\s]*[:=][\s]*['"]?(root|0)['"]?"#)?,
                group_pattern: None,
                remediation: "Create a dedicated service user with minimal privileges instead of using root.".to_string(),
                cwe_id: Some(250),
                owasp_category: Some("A01:2021 - Broken Access Control".to_string()),
            },

            UserGroupRule {
                name: "Privileged Group Assignment".to_string(),
                description: "Service assigned to privileged group".to_string(),
                severity: ConfigSeverity::Medium,
                user_pattern: Regex::new(r#"(?i)(group|run[_-]?as[_-]?group)[\s]*[:=][\s]*['"]?(root|wheel|admin|sudo)['"]?"#)?,
                group_pattern: None,
                remediation: "Use a dedicated service group with minimal privileges.".to_string(),
                cwe_id: Some(250),
                owasp_category: Some("A01:2021 - Broken Access Control".to_string()),
            },

            UserGroupRule {
                name: "System Account Usage".to_string(),
                description: "Application configured to use system account".to_string(),
                severity: ConfigSeverity::Medium,
                user_pattern: Regex::new(r#"(?i)(user|run[_-]?as[_-]?user)[\s]*[:=][\s]*['"]?(bin|daemon|sys|sync|mail|www-data|nobody)['"]?"#)?,
                group_pattern: None,
                remediation: "Create dedicated application users instead of using system accounts.".to_string(),
                cwe_id: Some(250),
                owasp_category: Some("A01:2021 - Broken Access Control".to_string()),
            },

            UserGroupRule {
                name: "UID Zero Assignment".to_string(),
                description: "Explicit UID 0 (root) assignment detected".to_string(),
                severity: ConfigSeverity::High,
                user_pattern: Regex::new(r#"(?i)(uid|user[_-]?id)[\s]*[:=][\s]*['"]?0['"]?"#)?,
                group_pattern: None,
                remediation: "Use non-zero UID for application processes.".to_string(),
                cwe_id: Some(250),
                owasp_category: Some("A01:2021 - Broken Access Control".to_string()),
            },

            UserGroupRule {
                name: "GID Zero Assignment".to_string(),
                description: "Explicit GID 0 (root group) assignment detected".to_string(),
                severity: ConfigSeverity::High,
                user_pattern: Regex::new(r#"(?i)(gid|group[_-]?id)[\s]*[:=][\s]*['"]?0['"]?"#)?,
                group_pattern: None,
                remediation: "Use non-zero GID for application processes.".to_string(),
                cwe_id: Some(250),
                owasp_category: Some("A01:2021 - Broken Access Control".to_string()),
            },

            UserGroupRule {
                name: "Database Root User".to_string(),
                description: "Database configured to use root/admin user".to_string(),
                severity: ConfigSeverity::High,
                user_pattern: Regex::new(r#"(?i)(db[_-]?user|database[_-]?user|username)[\s]*[:=][\s]*['"]?(root|admin|sa|postgres|mysql)['"]?"#)?,
                group_pattern: None,
                remediation: "Create dedicated database users with minimal required privileges.".to_string(),
                cwe_id: Some(250),
                owasp_category: Some("A01:2021 - Broken Access Control".to_string()),
            },
        ];

        Ok(rules)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_root_user_detection() {
        let config = ConfigSecurityConfig::default();
        let checker = UserGroupChecker::new(&config).unwrap();

        let line = "run_as_user: root";
        let issues = checker.check_line(line, 1);
        assert!(issues.iter().any(|i| i.title.contains("Root User")));
    }

    #[test]
    fn test_privileged_group_detection() {
        let config = ConfigSecurityConfig::default();
        let checker = UserGroupChecker::new(&config).unwrap();

        let line = "group: wheel";
        let issues = checker.check_line(line, 1);
        assert!(issues.iter().any(|i| i.title.contains("Privileged Group")));
    }

    #[test]
    fn test_uid_zero_detection() {
        let config = ConfigSecurityConfig::default();
        let checker = UserGroupChecker::new(&config).unwrap();

        let line = "uid: 0";
        let issues = checker.check_line(line, 1);
        assert!(issues.iter().any(|i| i.title.contains("UID Zero")));
    }

    #[test]
    fn test_safe_user_assignment() {
        let config = ConfigSecurityConfig::default();
        let checker = UserGroupChecker::new(&config).unwrap();

        let line = "user: appuser";
        let issues = checker.check_line(line, 1);
        assert!(issues.is_empty());
    }
}