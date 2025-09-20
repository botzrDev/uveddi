//! Misconfiguration detection in configuration files
//!
//! This module detects insecure service configurations, weak encryption
//! settings, exposed debug endpoints, and overly permissive CORS settings.

use crate::analysis::AnalysisError;
use super::super::config::ConfigSecurityConfig;
use super::super::types::{ConfigIssue, ConfigSeverity};
use regex::Regex;
use serde_yaml::Value as YamlValue;

/// Analyzes configuration files for security misconfigurations
pub struct MisconfigurationAnalyzer {
    rules: Vec<MisconfigurationRule>,
    config: ConfigSecurityConfig,
}

/// Rule for detecting security misconfigurations
struct MisconfigurationRule {
    name: String,
    description: String,
    severity: ConfigSeverity,
    pattern: MisconfigurationPattern,
    remediation: String,
    cwe_id: Option<u32>,
    owasp_category: Option<String>,
}

/// Pattern types for detecting misconfigurations
enum MisconfigurationPattern {
    /// Simple regex pattern matching
    Regex(Regex),
    /// Key-value pattern matching
    KeyValue { key_pattern: Regex, value_pattern: Option<Regex> },
    /// Complex structured pattern matching
    Structured(Box<dyn Fn(&YamlValue) -> bool + Send + Sync>),
}

impl MisconfigurationAnalyzer {
    /// Create a new misconfiguration analyzer
    pub fn new(config: &ConfigSecurityConfig) -> Result<Self, AnalysisError> {
        let rules = Self::build_misconfiguration_rules()?;

        Ok(Self {
            rules,
            config: config.clone(),
        })
    }

    /// Analyze content for security misconfigurations
    pub fn analyze(&self, content: &str) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Text-based pattern matching
        for (line_num, line) in content.lines().enumerate() {
            for rule in &self.rules {
                if let Some(issue) = self.check_line_against_rule(line, line_num + 1, rule)? {
                    issues.push(issue);
                }
            }
        }

        // Structured analysis for YAML/JSON
        if let Ok(yaml_value) = serde_yaml::from_str::<YamlValue>(content) {
            issues.extend(self.analyze_structured_config(&yaml_value)?);
        }

        Ok(issues)
    }

    fn build_misconfiguration_rules() -> Result<Vec<MisconfigurationRule>, AnalysisError> {
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
                    value_pattern: Some(Regex::new(r"^\*$")?),
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
                    Regex::new(r"(?i)port[\s]*[:=][\s]*(21|22|23|80|443|3306|5432|6379|27017|8080|8000)(?!\d)")?,
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

    fn check_line_against_rule(
        &self,
        line: &str,
        line_number: usize,
        rule: &MisconfigurationRule,
    ) -> Result<Option<ConfigIssue>, AnalysisError> {
        let matches = match &rule.pattern {
            MisconfigurationPattern::Regex(regex) => regex.is_match(line),
            MisconfigurationPattern::KeyValue { key_pattern, value_pattern } => {
                if key_pattern.is_match(line) {
                    if let Some(val_pattern) = value_pattern {
                        // Extract value part and check against value pattern
                        if let Some(colon_pos) = line.find(':') {
                            let value_part = &line[colon_pos + 1..].trim();
                            val_pattern.is_match(value_part)
                        } else if let Some(equals_pos) = line.find('=') {
                            let value_part = &line[equals_pos + 1..].trim();
                            val_pattern.is_match(value_part)
                        } else {
                            false
                        }
                    } else {
                        true
                    }
                } else {
                    false
                }
            },
            MisconfigurationPattern::Structured(_) => false, // Handled separately
        };

        if matches {
            let confidence = self.calculate_confidence(rule, line);

            let issue = ConfigIssue::new(
                rule.severity,
                confidence,
                rule.name.clone(),
                rule.description.clone(),
            )
            .with_location(line_number, 0)
            .with_snippet(line.to_string())
            .with_remediation(rule.remediation.clone())
            .with_tag("misconfiguration")
            .with_owasp(rule.owasp_category.clone().unwrap_or_default())
            .with_cwe(rule.cwe_id.unwrap_or(1188));

            Ok(Some(issue))
        } else {
            Ok(None)
        }
    }

    fn analyze_structured_config(&self, value: &YamlValue) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Check for dangerous combinations
        issues.extend(self.check_debug_in_production(value)?);
        issues.extend(self.check_weak_session_config(value)?);
        issues.extend(self.check_insecure_database_config(value)?);

        Ok(issues)
    }

    fn check_debug_in_production(&self, value: &YamlValue) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();

        if let YamlValue::Mapping(map) = value {
            let is_production = map.iter().any(|(k, v)| {
                k.as_str().map_or(false, |key| key.to_lowercase().contains("env")) &&
                v.as_str().map_or(false, |val| val.to_lowercase().contains("prod"))
            });

            let debug_enabled = map.iter().any(|(k, v)| {
                k.as_str().map_or(false, |key| key.to_lowercase().contains("debug")) &&
                v.as_bool().unwrap_or(false)
            });

            if is_production && debug_enabled {
                issues.push(ConfigIssue::new(
                    ConfigSeverity::High,
                    0.9,
                    "Debug Mode in Production",
                    "Debug mode is enabled in production environment",
                )
                .with_tag("production-debug")
                .with_remediation("Disable debug mode in production environments")
                .with_cwe(489));
            }
        }

        Ok(issues)
    }

    fn check_weak_session_config(&self, value: &YamlValue) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();

        if let YamlValue::Mapping(map) = value {
            for (key, val) in map {
                if let Some(key_str) = key.as_str() {
                    if key_str.to_lowercase().contains("session") {
                        if let YamlValue::Mapping(session_config) = val {
                            // Check for insecure session settings
                            for (session_key, session_val) in session_config {
                                if let Some(session_key_str) = session_key.as_str() {
                                    match session_key_str.to_lowercase().as_str() {
                                        "secure" if session_val.as_bool() == Some(false) => {
                                            issues.push(ConfigIssue::new(
                                                ConfigSeverity::Medium,
                                                0.8,
                                                "Insecure Session Cookie",
                                                "Session cookies are not marked as secure",
                                            )
                                            .with_tag("session-security")
                                            .with_remediation("Set session cookies to secure: true")
                                            .with_cwe(614));
                                        },
                                        "httponly" if session_val.as_bool() == Some(false) => {
                                            issues.push(ConfigIssue::new(
                                                ConfigSeverity::Medium,
                                                0.8,
                                                "Session Cookie XSS Risk",
                                                "Session cookies are accessible via JavaScript",
                                            )
                                            .with_tag("session-security")
                                            .with_remediation("Set session cookies to httpOnly: true")
                                            .with_cwe(1004));
                                        },
                                        _ => {},
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

    fn check_insecure_database_config(&self, value: &YamlValue) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut issues = Vec::new();

        if let YamlValue::Mapping(map) = value {
            for (key, val) in map {
                if let Some(key_str) = key.as_str() {
                    if key_str.to_lowercase().contains("database") || key_str.to_lowercase().contains("db") {
                        if let YamlValue::Mapping(db_config) = val {
                            // Check for SSL disabled
                            if let Some(ssl_val) = db_config.get(&YamlValue::String("ssl".to_string())) {
                                if ssl_val.as_bool() == Some(false) {
                                    issues.push(ConfigIssue::new(
                                        ConfigSeverity::High,
                                        0.85,
                                        "Database SSL Disabled",
                                        "Database connection does not use SSL encryption",
                                    )
                                    .with_tag("database-security")
                                    .with_remediation("Enable SSL for database connections")
                                    .with_cwe(319));
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(issues)
    }

    fn calculate_confidence(&self, rule: &MisconfigurationRule, line: &str) -> f64 {
        let mut confidence = 0.8; // Base confidence

        // Increase confidence for production contexts
        let prod_indicators = ["prod", "production", "live", "release"];
        if prod_indicators.iter().any(|&indicator| line.to_lowercase().contains(indicator)) {
            confidence = (confidence * 1.3).min(1.0);
        }

        // Decrease confidence for test/dev contexts
        let test_indicators = ["test", "dev", "development", "local", "example"];
        if test_indicators.iter().any(|&indicator| line.to_lowercase().contains(indicator)) {
            confidence *= 0.6;
        }

        // Adjust based on rule severity
        match rule.severity {
            ConfigSeverity::Critical => confidence * 1.1,
            ConfigSeverity::High => confidence,
            ConfigSeverity::Medium => confidence * 0.9,
            ConfigSeverity::Low => confidence * 0.8,
            ConfigSeverity::Info => confidence * 0.7,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_mode_detection() {
        let config = ConfigSecurityConfig::default();
        let analyzer = MisconfigurationAnalyzer::new(&config).unwrap();

        let content = r#"
debug: true
server:
  debug_mode: enabled
"#;

        let issues = analyzer.analyze(content).unwrap();
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.title.contains("Debug")));
    }

    #[test]
    fn test_cors_misconfiguration() {
        let config = ConfigSecurityConfig::default();
        let analyzer = MisconfigurationAnalyzer::new(&config).unwrap();

        let content = r#"
cors:
  origin: "*"
access_control_allow_origin: "*"
"#;

        let issues = analyzer.analyze(content).unwrap();
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.title.contains("CORS")));
    }

    #[test]
    fn test_ssl_disabled() {
        let config = ConfigSecurityConfig::default();
        let analyzer = MisconfigurationAnalyzer::new(&config).unwrap();

        let content = r#"
database:
  ssl_enabled: false
  tls_verify: no
"#;

        let issues = analyzer.analyze(content).unwrap();
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.title.contains("SSL") || i.title.contains("TLS")));
    }

    #[test]
    fn test_production_debug_detection() {
        let config = ConfigSecurityConfig::default();
        let analyzer = MisconfigurationAnalyzer::new(&config).unwrap();

        let content = r#"
environment: production
debug: true
"#;

        let issues = analyzer.analyze(content).unwrap();
        assert!(!issues.is_empty());

        // Should detect both general debug and production-specific debug
        let debug_issues: Vec<_> = issues.iter()
            .filter(|i| i.title.to_lowercase().contains("debug"))
            .collect();
        assert!(!debug_issues.is_empty());
    }
}