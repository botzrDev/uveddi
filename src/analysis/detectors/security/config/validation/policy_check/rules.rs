//! Security policy rule definitions

use super::super::super::config::ConfigSecurityConfig;
use super::super::super::types::ConfigSeverity;

/// Security policy definition
#[derive(Debug, Clone)]
pub struct SecurityPolicy {
    pub name: String,
    pub enabled: bool,
    pub severity_threshold: ConfigSeverity,
    pub confidence_threshold: f64,
    pub allowed_tags: Option<Vec<String>>,
    pub blocked_tags: Option<Vec<String>>,
    pub allowed_cwe_ids: Option<Vec<u32>>,
    pub blocked_cwe_ids: Option<Vec<u32>>,
    pub file_pattern_filters: Vec<String>,
    pub line_suppression_patterns: Vec<String>,
}

impl SecurityPolicy {
    /// Create a new security policy
    pub fn new(name: String) -> Self {
        Self {
            name,
            enabled: true,
            severity_threshold: ConfigSeverity::Info,
            confidence_threshold: 0.5,
            allowed_tags: None,
            blocked_tags: None,
            allowed_cwe_ids: None,
            blocked_cwe_ids: None,
            file_pattern_filters: vec![],
            line_suppression_patterns: vec![],
        }
    }

    /// Set the severity threshold
    pub fn with_severity_threshold(mut self, threshold: ConfigSeverity) -> Self {
        self.severity_threshold = threshold;
        self
    }

    /// Set the confidence threshold
    pub fn with_confidence_threshold(mut self, threshold: f64) -> Self {
        self.confidence_threshold = threshold;
        self
    }

    /// Set allowed tags
    pub fn with_allowed_tags(mut self, tags: Vec<String>) -> Self {
        self.allowed_tags = Some(tags);
        self
    }

    /// Set blocked tags
    pub fn with_blocked_tags(mut self, tags: Vec<String>) -> Self {
        self.blocked_tags = Some(tags);
        self
    }

    /// Set allowed CWE IDs
    pub fn with_allowed_cwe_ids(mut self, ids: Vec<u32>) -> Self {
        self.allowed_cwe_ids = Some(ids);
        self
    }

    /// Set blocked CWE IDs
    pub fn with_blocked_cwe_ids(mut self, ids: Vec<u32>) -> Self {
        self.blocked_cwe_ids = Some(ids);
        self
    }

    /// Add file pattern filters
    pub fn with_file_patterns(mut self, patterns: Vec<String>) -> Self {
        self.file_pattern_filters = patterns;
        self
    }

    /// Add suppression patterns
    pub fn with_suppression_patterns(mut self, patterns: Vec<String>) -> Self {
        self.line_suppression_patterns = patterns;
        self
    }

    /// Enable or disable the policy
    pub fn set_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Build default security policies
pub fn build_default_policies(config: &ConfigSecurityConfig) -> Vec<SecurityPolicy> {
    let mut policies = Vec::new();

    // High-severity policy - only critical and high severity issues
    policies.push(
        SecurityPolicy::new("High Severity Policy".to_string())
            .with_severity_threshold(ConfigSeverity::High)
            .with_confidence_threshold(config.confidence_threshold)
            .with_blocked_tags(vec!["test".to_string(), "example".to_string()])
            .with_suppression_patterns(vec![
                "UVEDDI:IGNORE".to_string(),
                "SECURITY:OK".to_string(),
                "FALSE-POSITIVE".to_string(),
                "NOSONAR".to_string(),
            ]),
    );

    // Credential-specific policy
    policies.push(
        SecurityPolicy::new("Credential Protection Policy".to_string())
            .with_severity_threshold(ConfigSeverity::Medium)
            .with_confidence_threshold(0.6) // Lower threshold for credentials
            .with_allowed_tags(vec![
                "credential".to_string(),
                "secret".to_string(),
                "password".to_string(),
                "api-key".to_string(),
            ])
            .with_blocked_tags(vec!["test-credential".to_string()])
            .with_allowed_cwe_ids(vec![798, 521, 522]) // Common credential CWEs
            .with_file_patterns(vec![
                "*.test.*".to_string(),
                "*.example.*".to_string(),
                "**/test/**".to_string(),
            ])
            .with_suppression_patterns(vec![
                "TEST-CREDENTIAL".to_string(),
                "EXAMPLE-PASSWORD".to_string(),
            ]),
    );

    // Production environment policy
    policies.push(
        SecurityPolicy::new("Production Environment Policy".to_string())
            .with_severity_threshold(ConfigSeverity::Low)
            .with_confidence_threshold(0.8) // Higher threshold for production
            .with_blocked_tags(vec![
                "development".to_string(),
                "local".to_string(),
                "test".to_string(),
            ])
            .with_file_patterns(vec![
                "**/dev/**".to_string(),
                "**/test/**".to_string(),
                "**/local/**".to_string(),
            ]),
    );

    // Compliance-focused policy
    policies.push(
        SecurityPolicy::new("Compliance Policy".to_string())
            .set_enabled(config.enable_compliance_validation)
            .with_severity_threshold(ConfigSeverity::Info)
            .with_confidence_threshold(0.7)
            .with_allowed_cwe_ids(vec![
                // OWASP Top 10 related CWEs
                79, 89, 22, 352, 863, 94, 287, 798, 311, 918,
                // Common configuration CWEs
                1188, 489, 319, 326, 327, 250, 732,
            ]),
    );

    policies
}
