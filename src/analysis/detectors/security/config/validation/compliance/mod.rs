//! Compliance validation for configuration security
//!
//! This module validates configuration security findings against
//! industry compliance standards like OWASP, CIS, NIST, etc.

pub mod types;
pub mod validator;
pub mod standards;
pub mod coverage;

// Re-export main types and validator
pub use types::{ComplianceStandard, ComplianceRequirement};
pub use validator::ComplianceValidator;
pub use coverage::ComplianceCoverage;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::detectors::security::config::ConfigSecurityConfig;

    #[test]
    fn test_compliance_validator_creation() {
        let config = ConfigSecurityConfig::default();
        let validator = ComplianceValidator::new(&config).unwrap();
        assert!(validator.get_active_standards_count() > 0);
    }
}