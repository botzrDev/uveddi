//! Compliance validation for configuration security
//!
//! This module validates configuration security findings against
//! industry compliance standards like OWASP, CIS, NIST, etc.

pub mod coverage;
pub mod standards;
pub mod types;
pub mod validator;

// Re-export main types and validator
pub use coverage::ComplianceCoverage;
pub use types::{ComplianceRequirement, ComplianceStandard};
pub use validator::ComplianceValidator;

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
