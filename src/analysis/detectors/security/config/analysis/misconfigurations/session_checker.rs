//! Session and authentication configuration security checks
//!
//! This module provides specialized checks for session management and
//! authentication configuration security issues.

use super::super::super::config::ConfigSecurityConfig;
use super::super::super::types::{ConfigIssue, ConfigSeverity};
use crate::analysis::AnalysisError;
use serde_yaml::Value as YamlValue;

/// Checker for session-related misconfigurations
pub struct SessionChecker {
    config: ConfigSecurityConfig,
}

impl SessionChecker {
    pub fn new(config: &ConfigSecurityConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    /// Check for weak session configuration
    pub fn check_weak_session_config(
        &self,
        value: &YamlValue,
    ) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();

        if let YamlValue::Mapping(map) = value {
            for (key, val) in map {
                if let Some(key_str) = key.as_str() {
                    if key_str.to_lowercase().contains("session") {
                        if let YamlValue::Mapping(session_config) = val {
                            issues.extend(self.check_session_security(session_config));
                        }
                    }
                }
            }
        }

        Ok(issues)
    }

    fn check_session_security(&self, session_config: &serde_yaml::Mapping) -> Vec<ConfigIssue> {
        let mut issues = Vec::new();

        for (session_key, session_val) in session_config {
            if let Some(session_key_str) = session_key.as_str() {
                match session_key_str.to_lowercase().as_str() {
                    "secure" if session_val.as_bool() == Some(false) => {
                        issues.push(
                            ConfigIssue::new(
                                ConfigSeverity::Medium,
                                0.8,
                                "Insecure Session Cookie",
                                "Session cookies are not marked as secure",
                            )
                            .with_tag("session-security")
                            .with_remediation("Set session cookies to secure: true")
                            .with_cwe(614),
                        );
                    }
                    "httponly" if session_val.as_bool() == Some(false) => {
                        issues.push(
                            ConfigIssue::new(
                                ConfigSeverity::Medium,
                                0.8,
                                "Session Cookie XSS Risk",
                                "Session cookies are accessible via JavaScript",
                            )
                            .with_tag("session-security")
                            .with_remediation("Set session cookies to httpOnly: true")
                            .with_cwe(1004),
                        );
                    }
                    "samesite" => {
                        if let Some(samesite_val) = session_val.as_str() {
                            if samesite_val.to_lowercase() == "none" {
                                issues.push(
                                    ConfigIssue::new(
                                        ConfigSeverity::Low,
                                        0.6,
                                        "Session Cookie CSRF Risk",
                                        "Session cookies allow cross-site requests",
                                    )
                                    .with_tag("session-security")
                                    .with_remediation("Consider using SameSite=Strict or Lax")
                                    .with_cwe(352),
                                );
                            }
                        }
                    }
                    "timeout" => {
                        if let Some(timeout) = session_val.as_i64() {
                            if timeout > 86400 {
                                // More than 24 hours
                                issues.push(
                                    ConfigIssue::new(
                                        ConfigSeverity::Low,
                                        0.6,
                                        "Long Session Timeout",
                                        "Session timeout is longer than recommended",
                                    )
                                    .with_tag("session-security")
                                    .with_remediation(
                                        "Consider shorter session timeouts for security",
                                    ),
                                );
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        issues
    }

    /// Check for authentication configuration issues
    pub fn check_authentication_config(&self, value: &YamlValue) -> Vec<ConfigIssue> {
        let mut issues = Vec::new();

        if let YamlValue::Mapping(map) = value {
            for (key, val) in map {
                if let Some(key_str) = key.as_str() {
                    let key_lower = key_str.to_lowercase();
                    if key_lower.contains("auth") {
                        if let YamlValue::Mapping(auth_config) = val {
                            issues.extend(self.check_auth_security(auth_config));
                        }
                    }
                }
            }
        }

        issues
    }

    fn check_auth_security(&self, auth_config: &serde_yaml::Mapping) -> Vec<ConfigIssue> {
        let mut issues = Vec::new();

        // Check for weak authentication methods
        if let Some(method_val) = auth_config.get(&YamlValue::String("method".to_string())) {
            if let Some(method) = method_val.as_str() {
                match method.to_lowercase().as_str() {
                    "basic" => {
                        issues.push(
                            ConfigIssue::new(
                                ConfigSeverity::Medium,
                                0.7,
                                "Basic Authentication Used",
                                "Basic authentication is inherently insecure",
                            )
                            .with_tag("auth-security")
                            .with_remediation(
                                "Use stronger authentication methods like OAuth2 or JWT",
                            )
                            .with_cwe(308),
                        );
                    }
                    "plain" | "plaintext" => {
                        issues.push(
                            ConfigIssue::new(
                                ConfigSeverity::High,
                                0.9,
                                "Plaintext Authentication",
                                "Authentication uses plaintext credentials",
                            )
                            .with_tag("auth-security")
                            .with_remediation("Use encrypted authentication methods")
                            .with_cwe(256),
                        );
                    }
                    _ => {}
                }
            }
        }

        // Check for password policy settings
        if let Some(policy_val) = auth_config.get(&YamlValue::String("password_policy".to_string()))
        {
            if let YamlValue::Mapping(policy) = policy_val {
                if let Some(min_length) = policy.get(&YamlValue::String("min_length".to_string())) {
                    if let Some(length) = min_length.as_i64() {
                        if length < 8 {
                            issues.push(
                                ConfigIssue::new(
                                    ConfigSeverity::Medium,
                                    0.6,
                                    "Weak Password Policy",
                                    format!("Minimum password length is too short: {}", length),
                                )
                                .with_tag("password-policy")
                                .with_remediation(
                                    "Set minimum password length to at least 8 characters",
                                ),
                            );
                        }
                    }
                }
            }
        }

        issues
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_security() {
        let config = ConfigSecurityConfig::default();
        let checker = SessionChecker::new(&config);

        let yaml_content = r#"
session:
  secure: false
  httponly: false
  samesite: none
"#;
        let value: YamlValue = serde_yaml::from_str(yaml_content).unwrap();
        let issues = checker.check_weak_session_config(&value).unwrap();
        assert!(issues
            .iter()
            .any(|i| i.title.contains("Insecure Session Cookie")));
        assert!(issues
            .iter()
            .any(|i| i.title.contains("Session Cookie XSS Risk")));
        assert!(issues
            .iter()
            .any(|i| i.title.contains("Session Cookie CSRF Risk")));
    }

    #[test]
    fn test_authentication_config() {
        let config = ConfigSecurityConfig::default();
        let checker = SessionChecker::new(&config);

        let yaml_content = r#"
auth:
  method: basic
  password_policy:
    min_length: 4
"#;
        let value: YamlValue = serde_yaml::from_str(yaml_content).unwrap();
        let issues = checker.check_authentication_config(&value);
        assert!(issues
            .iter()
            .any(|i| i.title.contains("Basic Authentication")));
        assert!(issues
            .iter()
            .any(|i| i.title.contains("Weak Password Policy")));
    }
}
