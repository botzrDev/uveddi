//! Privilege escalation analysis
//!
//! This module detects inappropriate privilege escalation configurations,
//! including sudo settings, container privileges, and database admin access.

use super::super::super::config::ConfigSecurityConfig;
use super::super::super::types::{ConfigIssue, ConfigSeverity};
use crate::analysis::AnalysisError;
use regex::Regex;

/// Rule for detecting privilege escalation issues
pub struct PrivilegeEscalationRule {
    pub name: String,
    pub description: String,
    pub severity: ConfigSeverity,
    pub pattern: Regex,
    pub remediation: String,
    pub cwe_id: Option<u32>,
    pub owasp_category: Option<String>,
}

/// Checker for privilege escalation security issues
pub struct PrivilegeEscalationChecker {
    rules: Vec<PrivilegeEscalationRule>,
}

impl PrivilegeEscalationChecker {
    pub fn new(_config: &ConfigSecurityConfig) -> Result<Self, AnalysisError> {
        let rules = Self::build_privilege_escalation_rules()?;
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

    fn build_privilege_escalation_rules() -> Result<Vec<PrivilegeEscalationRule>, AnalysisError> {
        let rules = vec![
            PrivilegeEscalationRule {
                name: "Passwordless Sudo".to_string(),
                description: "Sudo configured without password requirement".to_string(),
                severity: ConfigSeverity::Critical,
                pattern: Regex::new(r"(?i)NOPASSWD\s*:\s*ALL")?,
                remediation: "Require passwords for sudo access and limit sudo privileges."
                    .to_string(),
                cwe_id: Some(250),
                owasp_category: Some("A01:2021 - Broken Access Control".to_string()),
            },
            PrivilegeEscalationRule {
                name: "Unrestricted Sudo Access".to_string(),
                description: "User granted unrestricted sudo access".to_string(),
                severity: ConfigSeverity::High,
                pattern: Regex::new(r"(?i)(sudo|sudoers).*?ALL\s*=\s*\(ALL\)\s*ALL")?,
                remediation: "Limit sudo access to specific commands only.".to_string(),
                cwe_id: Some(250),
                owasp_category: Some("A01:2021 - Broken Access Control".to_string()),
            },
            PrivilegeEscalationRule {
                name: "Privileged Container".to_string(),
                description: "Container configured with privileged access".to_string(),
                severity: ConfigSeverity::High,
                pattern: Regex::new(r"(?i)privileged[\s]*[:=][\s]*true")?,
                remediation: "Remove privileged mode and use specific capabilities instead."
                    .to_string(),
                cwe_id: Some(250),
                owasp_category: Some("A01:2021 - Broken Access Control".to_string()),
            },
            PrivilegeEscalationRule {
                name: "Container Capability ALL".to_string(),
                description: "Container granted all capabilities".to_string(),
                severity: ConfigSeverity::High,
                pattern: Regex::new(
                    r#"(?i)(cap[_-]?add|capabilities)[\s]*[:=][\s]*['"]?ALL['"]?"#,
                )?,
                remediation: "Grant only specific required capabilities to containers.".to_string(),
                cwe_id: Some(250),
                owasp_category: Some("A01:2021 - Broken Access Control".to_string()),
            },
            PrivilegeEscalationRule {
                name: "Database Admin Privileges".to_string(),
                description: "Database user configured with administrative privileges".to_string(),
                severity: ConfigSeverity::Medium,
                pattern: Regex::new(
                    r#"(?i)(privileges?|grants?)[\s]*[:=][\s]*['"]?(all|admin|root|superuser)['"]?"#,
                )?,
                remediation: "Grant only necessary database privileges to application users."
                    .to_string(),
                cwe_id: Some(250),
                owasp_category: Some("A01:2021 - Broken Access Control".to_string()),
            },
            PrivilegeEscalationRule {
                name: "SUID/SGID Binary".to_string(),
                description: "SUID or SGID bit set on executable".to_string(),
                severity: ConfigSeverity::Medium,
                pattern: Regex::new(
                    r#"(?i)(suid|sgid|set[_-]?uid|set[_-]?gid)[\s]*[:=][\s]*['"]?(true|yes|1)['"]?"#,
                )?,
                remediation: "Avoid SUID/SGID binaries unless absolutely necessary.".to_string(),
                cwe_id: Some(250),
                owasp_category: Some("A01:2021 - Broken Access Control".to_string()),
            },
            PrivilegeEscalationRule {
                name: "Dangerous System Capabilities".to_string(),
                description: "Container granted dangerous system capabilities".to_string(),
                severity: ConfigSeverity::High,
                pattern: Regex::new(
                    r#"(?i)(cap[_-]?add|capabilities).*?(SYS_ADMIN|SYS_PTRACE|SYS_MODULE|DAC_OVERRIDE|NET_ADMIN)"#,
                )?,
                remediation: "Remove dangerous capabilities like SYS_ADMIN and use alternatives."
                    .to_string(),
                cwe_id: Some(250),
                owasp_category: Some("A01:2021 - Broken Access Control".to_string()),
            },
            PrivilegeEscalationRule {
                name: "Host Namespace Access".to_string(),
                description: "Container sharing host namespaces".to_string(),
                severity: ConfigSeverity::High,
                pattern: Regex::new(
                    r#"(?i)(host[_-]?network|host[_-]?pid|host[_-]?ipc)[\s]*[:=][\s]*['"]?(true|yes)['"]?"#,
                )?,
                remediation: "Use isolated namespaces instead of sharing host namespaces."
                    .to_string(),
                cwe_id: Some(250),
                owasp_category: Some("A01:2021 - Broken Access Control".to_string()),
            },
            PrivilegeEscalationRule {
                name: "Process Privilege Escalation".to_string(),
                description: "Process configured to allow privilege escalation".to_string(),
                severity: ConfigSeverity::Medium,
                pattern: Regex::new(
                    r#"(?i)(allow[_-]?privilege[_-]?escalation)[\s]*[:=][\s]*['"]?(true|yes)['"]?"#,
                )?,
                remediation: "Set allowPrivilegeEscalation to false.".to_string(),
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
    fn test_passwordless_sudo_detection() {
        let config = ConfigSecurityConfig::default();
        let checker = PrivilegeEscalationChecker::new(&config).unwrap();

        let line = "user ALL=(ALL) NOPASSWD: ALL";
        let issues = checker.check_line(line, 1);
        assert!(issues.iter().any(|i| i.title.contains("Passwordless Sudo")));
    }

    #[test]
    fn test_privileged_container_detection() {
        let config = ConfigSecurityConfig::default();
        let checker = PrivilegeEscalationChecker::new(&config).unwrap();

        let line = "privileged: true";
        let issues = checker.check_line(line, 1);
        assert!(issues
            .iter()
            .any(|i| i.title.contains("Privileged Container")));
    }

    #[test]
    fn test_dangerous_capabilities_detection() {
        let config = ConfigSecurityConfig::default();
        let checker = PrivilegeEscalationChecker::new(&config).unwrap();

        let line = "cap_add: SYS_ADMIN";
        let issues = checker.check_line(line, 1);
        assert!(issues
            .iter()
            .any(|i| i.title.contains("Dangerous System Capabilities")));
    }

    #[test]
    fn test_host_namespace_detection() {
        let config = ConfigSecurityConfig::default();
        let checker = PrivilegeEscalationChecker::new(&config).unwrap();

        let line = "hostNetwork: true";
        let issues = checker.check_line(line, 1);
        assert!(issues.iter().any(|i| i.title.contains("Host Namespace")));
    }

    #[test]
    fn test_safe_configuration() {
        let config = ConfigSecurityConfig::default();
        let checker = PrivilegeEscalationChecker::new(&config).unwrap();

        let line = "privileged: false";
        let issues = checker.check_line(line, 1);
        assert!(issues.is_empty());
    }
}
