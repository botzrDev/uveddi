//! Access control analysis
//!
//! This module detects overly permissive access controls,
//! missing authentication, and inappropriate authorization settings.

use super::super::super::config::ConfigSecurityConfig;
use super::super::super::types::{ConfigIssue, ConfigSeverity};
use crate::analysis::AnalysisError;
use regex::Regex;

/// Rule for detecting access control issues
pub struct AccessControlRule {
    pub name: String,
    pub description: String,
    pub severity: ConfigSeverity,
    pub pattern: Regex,
    pub remediation: String,
    pub cwe_id: Option<u32>,
    pub owasp_category: Option<String>,
}

/// Checker for access control security issues
pub struct AccessControlChecker {
    rules: Vec<AccessControlRule>,
}

impl AccessControlChecker {
    pub fn new(_config: &ConfigSecurityConfig) -> Result<Self, AnalysisError> {
        let rules = Self::build_access_control_rules()?;
        Ok(Self { rules })
    }

    pub fn check_line(&self, line: &str, line_number: usize) -> Vec<ConfigIssue> {
        let mut issues = Vec::new();

        for rule in &self.rules {
            if rule.pattern.is_match(line) {
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

    fn build_access_control_rules() -> Result<Vec<AccessControlRule>, AnalysisError> {
        let rules = vec![
            AccessControlRule {
                name: "Allow All Access".to_string(),
                description: "Access control configured to allow all users/IPs".to_string(),
                severity: ConfigSeverity::High,
                pattern: Regex::new(
                    r#"(?i)(allow|permit)[\s]*[:=][\s]*['"]?(all|\*|0\.0\.0\.0/0|::/0)['"]?"#,
                )?,
                remediation: "Restrict access to specific users, groups, or IP ranges as needed."
                    .to_string(),
                cwe_id: Some(284),
                owasp_category: Some("A01:2021 - Broken Access Control".to_string()),
            },
            AccessControlRule {
                name: "No Authentication Required".to_string(),
                description: "Service configured without authentication".to_string(),
                severity: ConfigSeverity::High,
                pattern: Regex::new(
                    r#"(?i)(auth|authentication)[\s]*[:=][\s]*['"]?(false|no|none|disabled|off)['"]?"#,
                )?,
                remediation: "Enable proper authentication mechanisms for all services."
                    .to_string(),
                cwe_id: Some(306),
                owasp_category: Some(
                    "A07:2021 - Identification and Authentication Failures".to_string(),
                ),
            },
            AccessControlRule {
                name: "Guest Access Enabled".to_string(),
                description: "Guest or anonymous access is enabled".to_string(),
                severity: ConfigSeverity::Medium,
                pattern: Regex::new(
                    r#"(?i)(guest|anonymous|public)[\s]*[:=][\s]*['"]?(true|yes|enabled|on)['"]?"#,
                )?,
                remediation: "Disable guest access and require proper authentication.".to_string(),
                cwe_id: Some(306),
                owasp_category: Some(
                    "A07:2021 - Identification and Authentication Failures".to_string(),
                ),
            },
            AccessControlRule {
                name: "Default Credentials".to_string(),
                description: "Default or weak credentials detected".to_string(),
                severity: ConfigSeverity::Critical,
                pattern: Regex::new(
                    r#"(?i)(password|pwd|pass)[\s]*[:=][\s]*['"]?(admin|password|123456|default|guest|root)['"]?"#,
                )?,
                remediation: "Change default credentials to strong, unique passwords.".to_string(),
                cwe_id: Some(521),
                owasp_category: Some(
                    "A07:2021 - Identification and Authentication Failures".to_string(),
                ),
            },
            AccessControlRule {
                name: "Insecure API Access".to_string(),
                description: "API configured with insecure access controls".to_string(),
                severity: ConfigSeverity::High,
                pattern: Regex::new(
                    r#"(?i)(api[_-]?key|token)[\s]*[:=][\s]*['"]?(public|open|unrestricted)['"]?"#,
                )?,
                remediation: "Implement proper API authentication and authorization.".to_string(),
                cwe_id: Some(284),
                owasp_category: Some("A01:2021 - Broken Access Control".to_string()),
            },
            AccessControlRule {
                name: "CORS Allow All Origins".to_string(),
                description: "CORS configured to allow all origins".to_string(),
                severity: ConfigSeverity::Medium,
                pattern: Regex::new(
                    r#"(?i)(cors|cross[_-]?origin).*?(origin|allowed[_-]?origins)[\s]*[:=][\s]*['"]?\*['"]?"#,
                )?,
                remediation: "Restrict CORS to specific trusted origins.".to_string(),
                cwe_id: Some(942),
                owasp_category: Some("A01:2021 - Broken Access Control".to_string()),
            },
            AccessControlRule {
                name: "Unrestricted File Access".to_string(),
                description: "File access configured without restrictions".to_string(),
                severity: ConfigSeverity::Medium,
                pattern: Regex::new(
                    r#"(?i)(file[_-]?access|directory[_-]?listing)[\s]*[:=][\s]*['"]?(true|yes|enabled|unrestricted)['"]?"#,
                )?,
                remediation: "Restrict file access and disable directory listing.".to_string(),
                cwe_id: Some(552),
                owasp_category: Some("A01:2021 - Broken Access Control".to_string()),
            },
            AccessControlRule {
                name: "Administrative Interface Exposed".to_string(),
                description: "Administrative interface accessible without restrictions".to_string(),
                severity: ConfigSeverity::High,
                pattern: Regex::new(
                    r#"(?i)(admin|management|console).*?(public|exposed|unrestricted)[\s]*[:=][\s]*['"]?(true|yes)['"]?"#,
                )?,
                remediation: "Restrict administrative interfaces to authorized networks only."
                    .to_string(),
                cwe_id: Some(284),
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
    fn test_allow_all_detection() {
        let config = ConfigSecurityConfig::default();
        let checker = AccessControlChecker::new(&config).unwrap();

        let line = "allow: all";
        let issues = checker.check_line(line, 1);
        assert!(issues.iter().any(|i| i.title.contains("Allow All Access")));
    }

    #[test]
    fn test_no_auth_detection() {
        let config = ConfigSecurityConfig::default();
        let checker = AccessControlChecker::new(&config).unwrap();

        let line = "authentication: false";
        let issues = checker.check_line(line, 1);
        assert!(issues.iter().any(|i| i.title.contains("No Authentication")));
    }

    #[test]
    fn test_default_credentials_detection() {
        let config = ConfigSecurityConfig::default();
        let checker = AccessControlChecker::new(&config).unwrap();

        let line = "password: admin";
        let issues = checker.check_line(line, 1);
        assert!(issues
            .iter()
            .any(|i| i.title.contains("Default Credentials")));
    }

    #[test]
    fn test_cors_wildcard_detection() {
        let config = ConfigSecurityConfig::default();
        let checker = AccessControlChecker::new(&config).unwrap();

        let line = "cors_allowed_origins: *";
        let issues = checker.check_line(line, 1);
        assert!(issues.iter().any(|i| i.title.contains("CORS Allow All")));
    }
}
