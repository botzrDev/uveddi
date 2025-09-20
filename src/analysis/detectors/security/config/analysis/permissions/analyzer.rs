//! Main permission analyzer
//!
//! This module coordinates all permission-related security checks
//! including file permissions, user/group settings, access control, and privilege escalation.

use crate::analysis::AnalysisError;
use super::super::super::config::ConfigSecurityConfig;
use super::super::super::types::{ConfigIssue, ConfigSeverity};
use super::file_permissions::FilePermissionChecker;
use super::user_group::UserGroupChecker;
use super::access_control::AccessControlChecker;
use super::privilege_escalation::PrivilegeEscalationChecker;
use serde_yaml::Value as YamlValue;

/// Analyzes configuration files for permission and access control issues
pub struct PermissionAnalyzer {
    file_checker: FilePermissionChecker,
    user_group_checker: UserGroupChecker,
    access_control_checker: AccessControlChecker,
    privilege_checker: PrivilegeEscalationChecker,
    config: ConfigSecurityConfig,
}

impl PermissionAnalyzer {
    /// Create a new permission analyzer
    pub fn new(config: &ConfigSecurityConfig) -> Result<Self, AnalysisError> {
        let file_checker = FilePermissionChecker::new(config)?;
        let user_group_checker = UserGroupChecker::new(config)?;
        let access_control_checker = AccessControlChecker::new(config)?;
        let privilege_checker = PrivilegeEscalationChecker::new(config)?;

        Ok(Self {
            file_checker,
            user_group_checker,
            access_control_checker,
            privilege_checker,
            config: config.clone(),
        })
    }

    /// Analyze content for permission and access control issues
    pub fn analyze(&self, content: &str) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Line-by-line pattern matching
        for (line_num, line) in content.lines().enumerate() {
            let line_number = line_num + 1;

            // Check all permission categories
            issues.extend(self.file_checker.check_line(line, line_number));
            issues.extend(self.user_group_checker.check_line(line, line_number));
            issues.extend(self.access_control_checker.check_line(line, line_number));
            issues.extend(self.privilege_checker.check_line(line, line_number));
        }

        // Structured analysis for complex permission configurations
        if let Ok(yaml_value) = serde_yaml::from_str::<YamlValue>(content) {
            issues.extend(self.analyze_structured_permissions(&yaml_value)?);
        }

        Ok(issues)
    }

    fn analyze_structured_permissions(&self, value: &YamlValue) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();

        issues.extend(self.check_kubernetes_security_context(value)?);
        issues.extend(self.check_database_user_permissions(value)?);
        issues.extend(self.check_service_user_configuration(value)?);

        Ok(issues)
    }

    fn check_kubernetes_security_context(&self, value: &YamlValue) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();

        if let YamlValue::Mapping(map) = value {
            for (key, val) in map {
                if key.as_str() == Some("securityContext") {
                    if let YamlValue::Mapping(security_context) = val {
                        // Check for runAsRoot
                        if let Some(run_as_user) = security_context.get(&YamlValue::String("runAsUser".to_string())) {
                            if run_as_user.as_u64() == Some(0) {
                                issues.push(ConfigIssue::new(
                                    ConfigSeverity::High,
                                    0.9,
                                    "Container Running as Root",
                                    "Kubernetes container configured to run as root user (UID 0)",
                                )
                                .with_tag("kubernetes-security")
                                .with_remediation("Set runAsUser to a non-root UID (e.g., 1000)")
                                .with_cwe(250));
                            }
                        }

                        // Check for privileged containers
                        if let Some(privileged) = security_context.get(&YamlValue::String("privileged".to_string())) {
                            if privileged.as_bool() == Some(true) {
                                issues.push(ConfigIssue::new(
                                    ConfigSeverity::Critical,
                                    0.95,
                                    "Privileged Container in Security Context",
                                    "Container configured with privileged access in security context",
                                )
                                .with_tag("kubernetes-security")
                                .with_remediation("Remove privileged: true and use specific capabilities")
                                .with_cwe(250));
                            }
                        }
                    }
                }
            }
        }

        Ok(issues)
    }

    fn check_database_user_permissions(&self, value: &YamlValue) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();

        self.check_database_permissions_recursive(value, "", &mut issues);

        Ok(issues)
    }

    fn check_database_permissions_recursive(&self, value: &YamlValue, path: &str, issues: &mut Vec<ConfigIssue>) {
        match value {
            YamlValue::Mapping(map) => {
                // Look for database configuration patterns
                for (key, val) in map {
                    if let Some(key_str) = key.as_str() {
                        let key_lower = key_str.to_lowercase();

                        // Check for database admin users
                        if key_lower.contains("user") && key_lower.contains("database") {
                            if let Some(user_str) = val.as_str() {
                                if ["root", "admin", "sa", "postgres", "mysql"].contains(&user_str) {
                                    issues.push(ConfigIssue::new(
                                        ConfigSeverity::High,
                                        0.85,
                                        "Database Admin User in Configuration",
                                        format!("Database configured to use admin user: {}", user_str),
                                    )
                                    .with_remediation("Create dedicated database users with minimal privileges")
                                    .with_cwe(250));
                                }
                            }
                        }

                        let new_path = if path.is_empty() {
                            key_str.to_string()
                        } else {
                            format!("{}.{}", path, key_str)
                        };

                        self.check_database_permissions_recursive(val, &new_path, issues);
                    }
                }
            }
            YamlValue::Sequence(seq) => {
                for (index, item) in seq.iter().enumerate() {
                    let new_path = format!("{}[{}]", path, index);
                    self.check_database_permissions_recursive(item, &new_path, issues);
                }
            }
            _ => {}
        }
    }

    fn check_service_user_configuration(&self, value: &YamlValue) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();

        if let YamlValue::Mapping(map) = value {
            for (key, val) in map {
                if let Some(key_str) = key.as_str() {
                    let key_lower = key_str.to_lowercase();

                    // Check for service user configurations
                    if key_lower.contains("service") || key_lower.contains("daemon") {
                        if let YamlValue::Mapping(service_map) = val {
                            // Check user setting
                            if let Some(user_val) = service_map.get(&YamlValue::String("user".to_string())) {
                                if let Some(user_str) = user_val.as_str() {
                                    if user_str == "root" || user_str == "0" {
                                        issues.push(ConfigIssue::new(
                                            ConfigSeverity::High,
                                            0.9,
                                            "Service Running as Root",
                                            format!("Service '{}' configured to run as root", key_str),
                                        )
                                        .with_remediation("Create a dedicated service user with minimal privileges")
                                        .with_cwe(250));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(issues)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_analyzer_creation() {
        let config = ConfigSecurityConfig::default();
        let analyzer = PermissionAnalyzer::new(&config).unwrap();
        // Basic creation test passes
    }

    #[test]
    fn test_comprehensive_analysis() {
        let config = ConfigSecurityConfig::default();
        let analyzer = PermissionAnalyzer::new(&config).unwrap();

        let config_content = r#"
file_mode: 777
user: root
authentication: false
privileged: true
"#;

        let issues = analyzer.analyze(config_content).unwrap();
        assert!(!issues.is_empty());

        // Should detect multiple types of issues
        assert!(issues.iter().any(|i| i.title.contains("World-Writable")));
        assert!(issues.iter().any(|i| i.title.contains("Root User")));
        assert!(issues.iter().any(|i| i.title.contains("Authentication")));
        assert!(issues.iter().any(|i| i.title.contains("Privileged")));
    }

    #[test]
    fn test_kubernetes_security_context() {
        let config = ConfigSecurityConfig::default();
        let analyzer = PermissionAnalyzer::new(&config).unwrap();

        let k8s_config = r#"
apiVersion: v1
kind: Pod
spec:
  securityContext:
    runAsUser: 0
    privileged: true
"#;

        let issues = analyzer.analyze(k8s_config).unwrap();
        assert!(issues.iter().any(|i| i.title.contains("Container Running as Root")));
        assert!(issues.iter().any(|i| i.title.contains("Privileged Container")));
    }

    #[test]
    fn test_safe_configuration() {
        let config = ConfigSecurityConfig::default();
        let analyzer = PermissionAnalyzer::new(&config).unwrap();

        let safe_config = r#"
file_mode: 644
user: appuser
authentication: true
privileged: false
"#;

        let issues = analyzer.analyze(safe_config).unwrap();
        assert!(issues.is_empty());
    }
}