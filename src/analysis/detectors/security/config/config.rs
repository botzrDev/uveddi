//! Configuration structures for the configuration security detector

use serde::{Deserialize, Serialize};

/// Configuration for the configuration security detector
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigSecurityConfig {
    /// Enable credential exposure detection
    pub enable_credential_detection: bool,
    /// Enable misconfiguration detection
    pub enable_misconfiguration_detection: bool,
    /// Enable permission issue detection
    pub enable_permission_detection: bool,
    /// Enable insecure defaults detection
    pub enable_defaults_detection: bool,
    /// Confidence threshold for reporting issues
    pub confidence_threshold: f64,
    /// Maximum issues per file
    pub max_issues_per_file: usize,
    /// Enable compliance validation
    pub enable_compliance_validation: bool,
    /// Enable policy validation
    pub enable_policy_validation: bool,
}

impl Default for ConfigSecurityConfig {
    fn default() -> Self {
        Self {
            enable_credential_detection: true,
            enable_misconfiguration_detection: true,
            enable_permission_detection: true,
            enable_defaults_detection: true,
            confidence_threshold: 0.7,
            max_issues_per_file: 50,
            enable_compliance_validation: true,
            enable_policy_validation: true,
        }
    }
}

impl ConfigSecurityConfig {
    /// Create production configuration
    pub fn production() -> Self {
        Self {
            confidence_threshold: 0.8,
            max_issues_per_file: 100,
            ..Default::default()
        }
    }

    /// Create development configuration
    pub fn development() -> Self {
        Self {
            confidence_threshold: 0.5,
            max_issues_per_file: 20,
            enable_compliance_validation: false,
            ..Default::default()
        }
    }
}
