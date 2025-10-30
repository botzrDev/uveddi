//! Validation orchestration for configuration security
//!
//! This module provides policy validation and compliance checking
//! for configuration security analysis results.

pub mod compliance;
pub mod policy_check;

// Re-export validators
pub use compliance::ComplianceValidator;
pub use policy_check::PolicyValidator;

use super::super::types::SecurityIssue;
use super::config::ConfigSecurityConfig;
use super::types::ConfigIssue;
use crate::analysis::AnalysisError;

/// Combined validation orchestrator
pub struct ValidationOrchestrator {
    policy_validator: PolicyValidator,
    compliance_validator: ComplianceValidator,
    config: ConfigSecurityConfig,
}

impl ValidationOrchestrator {
    /// Create a new validation orchestrator
    pub fn new(config: &ConfigSecurityConfig) -> Result<Self, AnalysisError> {
        Ok(Self {
            policy_validator: PolicyValidator::new(config)?,
            compliance_validator: ComplianceValidator::new(config)?,
            config: config.clone(),
        })
    }

    /// Validate configuration issues against policies and compliance standards
    pub async fn validate_issues(
        &self,
        issues: Vec<ConfigIssue>,
    ) -> Result<Vec<ConfigIssue>, AnalysisError> {
        let mut validated_issues = Vec::new();

        for issue in issues {
            // Apply policy validation
            if self.policy_validator.validate_issue(&issue)? {
                // Apply compliance enhancement
                let enhanced_issue = self.compliance_validator.enhance_issue(issue)?;
                validated_issues.push(enhanced_issue);
            }
        }

        Ok(validated_issues)
    }

    /// Validate security issues (for integration with main detector)
    pub async fn validate_security_issues(
        &self,
        issues: Vec<SecurityIssue>,
    ) -> Result<Vec<SecurityIssue>, AnalysisError> {
        let mut validated_issues = Vec::new();

        for issue in issues {
            // Apply policy validation for security issues
            if self.policy_validator.validate_security_issue(&issue)? {
                // Apply compliance enhancement for security issues
                let enhanced_issue = self.compliance_validator.enhance_security_issue(issue)?;
                validated_issues.push(enhanced_issue);
            }
        }

        Ok(validated_issues)
    }

    /// Check if validation is enabled
    pub fn is_validation_enabled(&self) -> bool {
        self.config.enable_policy_validation || self.config.enable_compliance_validation
    }

    /// Get validation statistics
    pub fn get_validation_stats(&self) -> ValidationStats {
        ValidationStats {
            policy_validation_enabled: self.config.enable_policy_validation,
            compliance_validation_enabled: self.config.enable_compliance_validation,
            active_policies: self.policy_validator.get_active_policy_count(),
            active_compliance_standards: self.compliance_validator.get_active_standards_count(),
        }
    }
}

/// Statistics about validation configuration
#[derive(Debug, Clone)]
pub struct ValidationStats {
    pub policy_validation_enabled: bool,
    pub compliance_validation_enabled: bool,
    pub active_policies: usize,
    pub active_compliance_standards: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::detectors::security::config::types::{ConfigSeverity, ConfigType};

    #[tokio::test]
    async fn test_validation_orchestrator() {
        let config = ConfigSecurityConfig::default();
        let orchestrator = ValidationOrchestrator::new(&config).unwrap();

        let test_issue =
            ConfigIssue::new(ConfigSeverity::High, 0.9, "Test Issue", "Test description");

        let issues = vec![test_issue];
        let validated = orchestrator.validate_issues(issues).await.unwrap();

        // Should validate at least some issues
        assert!(!validated.is_empty());
    }

    #[test]
    fn test_validation_stats() {
        let config = ConfigSecurityConfig::default();
        let orchestrator = ValidationOrchestrator::new(&config).unwrap();

        let stats = orchestrator.get_validation_stats();
        assert!(stats.policy_validation_enabled);
        assert!(stats.compliance_validation_enabled);
    }

    #[test]
    fn test_validation_enabled_check() {
        let config = ConfigSecurityConfig::default();
        let orchestrator = ValidationOrchestrator::new(&config).unwrap();

        assert!(orchestrator.is_validation_enabled());

        let disabled_config = ConfigSecurityConfig {
            enable_policy_validation: false,
            enable_compliance_validation: false,
            ..Default::default()
        };
        let disabled_orchestrator = ValidationOrchestrator::new(&disabled_config).unwrap();

        assert!(!disabled_orchestrator.is_validation_enabled());
    }
}
