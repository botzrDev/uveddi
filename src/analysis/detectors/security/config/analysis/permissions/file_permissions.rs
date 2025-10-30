//! File permission analysis
//!
//! This module detects overly permissive file permissions and
//! inappropriate access controls on sensitive files.

use super::super::super::config::ConfigSecurityConfig;
use super::super::super::types::{ConfigIssue, ConfigSeverity};
use crate::analysis::AnalysisError;
use regex::Regex;

/// Rule for detecting file permission issues
pub struct FilePermissionRule {
    pub name: String,
    pub description: String,
    pub severity: ConfigSeverity,
    pub pattern: Regex,
    pub remediation: String,
    pub cwe_id: Option<u32>,
    pub owasp_category: Option<String>,
}

/// Checker for file permission security issues
pub struct FilePermissionChecker {
    rules: Vec<FilePermissionRule>,
}

impl FilePermissionChecker {
    pub fn new(_config: &ConfigSecurityConfig) -> Result<Self, AnalysisError> {
        let rules = Self::build_file_permission_rules()?;
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

    fn build_file_permission_rules() -> Result<Vec<FilePermissionRule>, AnalysisError> {
        let rules = vec![
            FilePermissionRule {
                name: "World-Writable File".to_string(),
                description: "File or directory is world-writable".to_string(),
                severity: ConfigSeverity::High,
                pattern: Regex::new(
                    r#"(?i)(mode|chmod|permissions?)[\s]*[:=][\s]*['"]?[0-7]*[2367][0-7]*['"]?"#,
                )?,
                remediation:
                    "Remove world-write permissions. Use 644 for files and 755 for directories."
                        .to_string(),
                cwe_id: Some(732),
                owasp_category: Some("A01:2021 - Broken Access Control".to_string()),
            },
            FilePermissionRule {
                name: "World-Readable Sensitive File".to_string(),
                description: "Sensitive file is world-readable".to_string(),
                severity: ConfigSeverity::Medium,
                pattern: Regex::new(
                    r#"(?i)(secret|private|key|password).*?(mode|chmod|permissions?)[\s]*[:=][\s]*['"]?[0-7]*[4567][0-7]*['"]?"#,
                )?,
                remediation: "Restrict read permissions on sensitive files to owner only (600)."
                    .to_string(),
                cwe_id: Some(732),
                owasp_category: Some("A01:2021 - Broken Access Control".to_string()),
            },
            FilePermissionRule {
                name: "Executable Bit on Data File".to_string(),
                description: "Data file has execute permissions".to_string(),
                severity: ConfigSeverity::Low,
                pattern: Regex::new(
                    r#"(?i)(config|data|log|\.txt|\.json|\.xml|\.yaml).*?(mode|chmod|permissions?)[\s]*[:=][\s]*['"]?[0-7]*[1357][0-7]*['"]?"#,
                )?,
                remediation: "Remove execute permissions from data files.".to_string(),
                cwe_id: Some(732),
                owasp_category: Some("A01:2021 - Broken Access Control".to_string()),
            },
            FilePermissionRule {
                name: "Overly Permissive Directory".to_string(),
                description: "Directory has overly permissive access".to_string(),
                severity: ConfigSeverity::Medium,
                pattern: Regex::new(
                    r#"(?i)(directory|dir|folder).*?(mode|chmod|permissions?)[\s]*[:=][\s]*['"]?777['"]?"#,
                )?,
                remediation: "Use more restrictive directory permissions like 755 or 750."
                    .to_string(),
                cwe_id: Some(732),
                owasp_category: Some("A01:2021 - Broken Access Control".to_string()),
            },
            FilePermissionRule {
                name: "Unsafe Temporary File Permissions".to_string(),
                description: "Temporary files created with unsafe permissions".to_string(),
                severity: ConfigSeverity::Medium,
                pattern: Regex::new(
                    r#"(?i)(temp|tmp|temporary).*?(mode|chmod|permissions?)[\s]*[:=][\s]*['"]?[0-7]*[4567][4567][4567]['"]?"#,
                )?,
                remediation: "Create temporary files with restrictive permissions (600)."
                    .to_string(),
                cwe_id: Some(732),
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
    fn test_world_writable_detection() {
        let config = ConfigSecurityConfig::default();
        let checker = FilePermissionChecker::new(&config).unwrap();

        let line = "file_mode: 777";
        let issues = checker.check_line(line, 1);
        assert!(issues.iter().any(|i| i.title.contains("World-Writable")));
    }

    #[test]
    fn test_sensitive_file_permissions() {
        let config = ConfigSecurityConfig::default();
        let checker = FilePermissionChecker::new(&config).unwrap();

        let line = "private_key_mode: 644";
        let issues = checker.check_line(line, 1);
        assert!(issues
            .iter()
            .any(|i| i.title.contains("World-Readable Sensitive")));
    }

    #[test]
    fn test_safe_permissions_no_issues() {
        let config = ConfigSecurityConfig::default();
        let checker = FilePermissionChecker::new(&config).unwrap();

        let line = "file_mode: 600";
        let issues = checker.check_line(line, 1);
        assert!(issues.is_empty());
    }
}
