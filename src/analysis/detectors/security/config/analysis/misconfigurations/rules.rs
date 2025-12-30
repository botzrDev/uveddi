//! Misconfiguration detection rules and patterns
//!
//! This module defines rules for detecting various security misconfigurations
//! in configuration files.

use super::super::super::types::ConfigSeverity;
use crate::analysis::AnalysisError;
use regex::Regex;
use serde_yaml::Value as YamlValue;

/// Rule for detecting security misconfigurations
pub struct MisconfigurationRule {
    pub name: String,
    pub description: String,
    pub severity: ConfigSeverity,
    pub pattern: MisconfigurationPattern,
    pub remediation: String,
    pub cwe_id: Option<u32>,
    pub owasp_category: Option<String>,
}

/// Pattern types for detecting misconfigurations
pub enum MisconfigurationPattern {
    /// Simple regex pattern matching
    Regex(Regex),
    /// Key-value pattern matching
    KeyValue {
        key_pattern: Regex,
        value_pattern: Option<Regex>,
    },
    /// Complex structured pattern matching
    Structured(Box<dyn Fn(&YamlValue) -> bool + Send + Sync>),
}

impl MisconfigurationRule {
    /// Build the default set of misconfiguration rules
    pub fn build_default_rules() -> Result<Vec<MisconfigurationRule>, AnalysisError> {
        let rules = vec![
            // Debug mode configurations
            MisconfigurationRule {
                name: "Debug Mode Enabled".to_string(),
                description: "Debug mode is enabled in configuration".to_string(),
                severity: ConfigSeverity::Medium,
                pattern: MisconfigurationPattern::KeyValue {
                    key_pattern: Regex::new(r"(?i)(debug|DEBUG)")?,
                    value_pattern: Some(Regex::new(r"(?i)(true|yes|1|on|enabled)")?),
                },
                remediation: "Disable debug mode in production environments. Set debug to false.".to_string(),
                cwe_id: Some(489),
                owasp_category: Some("A05:2021 - Security Misconfiguration".to_string()),
            },

            // CORS misconfigurations
            MisconfigurationRule {
                name: "Overly Permissive CORS".to_string(),
                description: "CORS is configured to allow all origins".to_string(),
                severity: ConfigSeverity::High,
                pattern: MisconfigurationPattern::KeyValue {
                    key_pattern: Regex::new(r"(?i)(cors|access[_-]?control[_-]?allow[_-]?origin)")?,
                    // Match wildcard with or without surrounding quotes
                    value_pattern: Some(Regex::new(r#"^["']?\*["']?$"#)?),
                },
                remediation: "Restrict CORS to specific trusted origins instead of using '*'.".to_string(),
                cwe_id: Some(942),
                owasp_category: Some("A05:2021 - Security Misconfiguration".to_string()),
            },

            // Insecure SSL/TLS configurations
            MisconfigurationRule {
                name: "SSL/TLS Disabled".to_string(),
                description: "SSL/TLS encryption is disabled".to_string(),
                severity: ConfigSeverity::High,
                pattern: MisconfigurationPattern::KeyValue {
                    key_pattern: Regex::new(r"(?i)(ssl|tls|https)[_-]?(enabled?|verify)")?,
                    value_pattern: Some(Regex::new(r"(?i)(false|no|0|off|disabled)")?),
                },
                remediation: "Enable SSL/TLS encryption for all communications. Set ssl_enabled to true.".to_string(),
                cwe_id: Some(319),
                owasp_category: Some("A02:2021 - Cryptographic Failures".to_string()),
            },

            // Weak encryption algorithms
            MisconfigurationRule {
                name: "Weak Encryption Algorithm".to_string(),
                description: "Weak or deprecated encryption algorithm configured".to_string(),
                severity: ConfigSeverity::High,
                pattern: MisconfigurationPattern::Regex(
                    Regex::new(r"(?i)(md5|sha1|des|3des|rc4|ssl_?v?[23])")?,
                ),
                remediation: "Use strong encryption algorithms like AES-256, SHA-256, or newer.".to_string(),
                cwe_id: Some(327),
                owasp_category: Some("A02:2021 - Cryptographic Failures".to_string()),
            },

            // Default ports
            MisconfigurationRule {
                name: "Default Port Usage".to_string(),
                description: "Service is using default/well-known ports".to_string(),
                severity: ConfigSeverity::Low,
                pattern: MisconfigurationPattern::Regex(
                    Regex::new(r"(?i)port[\s]*[:=][\s]*(21|22|23|80|443|3306|5432|6379|27017|8080|8000)\b")?,
                ),
                remediation: "Consider using non-default ports to reduce automated attack surface.".to_string(),
                cwe_id: Some(1188),
                owasp_category: Some("A05:2021 - Security Misconfiguration".to_string()),
            },

            // Insecure authentication
            MisconfigurationRule {
                name: "Basic Authentication".to_string(),
                description: "Basic authentication is enabled".to_string(),
                severity: ConfigSeverity::Medium,
                pattern: MisconfigurationPattern::Regex(
                    Regex::new(r"(?i)(basic[_-]?auth|auth[_-]?type[\s]*[:=][\s]*basic)")?
                ),
                remediation: "Use stronger authentication methods like OAuth 2.0, JWT, or certificate-based authentication.".to_string(),
                cwe_id: Some(308),
                owasp_category: Some("A07:2021 - Identification and Authentication Failures".to_string()),
            },

            // Logging sensitive data
            MisconfigurationRule {
                name: "Sensitive Data Logging".to_string(),
                description: "Configuration may log sensitive information".to_string(),
                severity: ConfigSeverity::Medium,
                pattern: MisconfigurationPattern::Regex(
                    Regex::new(r"(?i)(log[_-]?level[\s]*[:=][\s]*(debug|trace)|verbose[_-]?logging[\s]*[:=][\s]*true)")?
                ),
                remediation: "Avoid verbose logging in production to prevent exposure of sensitive data.".to_string(),
                cwe_id: Some(532),
                owasp_category: Some("A09:2021 - Security Logging and Monitoring Failures".to_string()),
            },
        ];

        Ok(rules)
    }

    /// Create additional security rules for specific frameworks
    pub fn build_framework_rules() -> Result<Vec<MisconfigurationRule>, AnalysisError> {
        let rules = vec![
            // Docker security
            MisconfigurationRule {
                name: "Privileged Container".to_string(),
                description: "Container is configured to run in privileged mode".to_string(),
                severity: ConfigSeverity::Critical,
                pattern: MisconfigurationPattern::KeyValue {
                    key_pattern: Regex::new(r"(?i)privileged")?,
                    value_pattern: Some(Regex::new(r"(?i)(true|yes)")?),
                },
                remediation: "Avoid running containers in privileged mode. Use specific capabilities instead.".to_string(),
                cwe_id: Some(250),
                owasp_category: Some("A05:2021 - Security Misconfiguration".to_string()),
            },

            // Database security
            MisconfigurationRule {
                name: "Database Auto-commit Disabled".to_string(),
                description: "Database auto-commit is disabled which may lead to data inconsistency".to_string(),
                severity: ConfigSeverity::Medium,
                pattern: MisconfigurationPattern::KeyValue {
                    key_pattern: Regex::new(r"(?i)auto[_-]?commit")?,
                    value_pattern: Some(Regex::new(r"(?i)(false|no|0|off)")?),
                },
                remediation: "Enable auto-commit or ensure proper transaction management.".to_string(),
                cwe_id: Some(362),
                owasp_category: Some("A05:2021 - Security Misconfiguration".to_string()),
            },

            // Web server security
            MisconfigurationRule {
                name: "Server Tokens Exposed".to_string(),
                description: "Web server is configured to expose server version information".to_string(),
                severity: ConfigSeverity::Low,
                pattern: MisconfigurationPattern::KeyValue {
                    key_pattern: Regex::new(r"(?i)(server[_-]?tokens|expose[_-]?headers)")?,
                    value_pattern: Some(Regex::new(r"(?i)(on|true|yes|1)")?),
                },
                remediation: "Disable server tokens to avoid information disclosure.".to_string(),
                cwe_id: Some(200),
                owasp_category: Some("A05:2021 - Security Misconfiguration".to_string()),
            },
        ];

        Ok(rules)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_rules_creation() {
        let rules = MisconfigurationRule::build_default_rules().unwrap();
        assert!(!rules.is_empty());
        assert!(rules.len() >= 7);
    }

    #[test]
    fn test_framework_rules_creation() {
        let rules = MisconfigurationRule::build_framework_rules().unwrap();
        assert!(!rules.is_empty());
        assert!(rules.len() >= 3);
    }

    #[test]
    fn test_rule_properties() {
        let rules = MisconfigurationRule::build_default_rules().unwrap();
        let debug_rule = rules
            .iter()
            .find(|r| r.name == "Debug Mode Enabled")
            .unwrap();

        assert_eq!(debug_rule.severity, ConfigSeverity::Medium);
        assert_eq!(debug_rule.cwe_id, Some(489));
        assert!(debug_rule.owasp_category.is_some());
    }
}
