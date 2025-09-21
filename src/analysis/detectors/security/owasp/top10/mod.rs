//! OWASP Top 10 2021 security vulnerability detectors
//!
//! This module contains specialized detectors for each of the OWASP Top 10
//! security vulnerabilities, providing comprehensive coverage of modern
//! web application security risks.

pub mod a01_broken_access;
pub mod a02_crypto_failures;
pub mod a03_injection;
pub mod a04_insecure_design;
pub mod a05_security_config;
pub mod a06_vulnerable_components;
pub mod a07_auth_failures;
pub mod a08_software_integrity;
pub mod a09_logging_monitoring;
pub mod a10_ssrf;

// Re-exports for convenience
pub use a01_broken_access::BrokenAccessControlDetector;
pub use a02_crypto_failures::CryptographicFailuresDetector;
pub use a03_injection::InjectionDetector;
pub use a04_insecure_design::InsecureDesignDetector;
pub use a05_security_config::SecurityMisconfigurationDetector;
pub use a06_vulnerable_components::VulnerableComponentsDetector;
pub use a07_auth_failures::AuthenticationFailuresDetector;
pub use a08_software_integrity::DataIntegrityFailuresDetector;
pub use a09_logging_monitoring::LoggingFailuresDetector;
pub use a10_ssrf::ServerSideRequestForgeryDetector;

use super::types::{OwaspCategory, OwaspCategoryDetector};
use crate::analysis::AnalysisError;
use std::collections::HashMap;

/// Factory for creating OWASP Top 10 detectors
pub struct OwaspTop10Factory;

impl OwaspTop10Factory {
    /// Create all OWASP Top 10 detectors
    pub fn create_all_detectors() -> Result<HashMap<OwaspCategory, Box<dyn OwaspCategoryDetector>>, AnalysisError> {
        let mut detectors: HashMap<OwaspCategory, Box<dyn OwaspCategoryDetector>> = HashMap::new();

        detectors.insert(
            OwaspCategory::BrokenAccessControl,
            Box::new(BrokenAccessControlDetector::new()),
        );
        detectors.insert(
            OwaspCategory::CryptographicFailures,
            Box::new(CryptographicFailuresDetector::new()),
        );
        detectors.insert(
            OwaspCategory::Injection,
            Box::new(InjectionDetector::new()),
        );
        detectors.insert(
            OwaspCategory::InsecureDesign,
            Box::new(InsecureDesignDetector::new()),
        );
        detectors.insert(
            OwaspCategory::SecurityMisconfiguration,
            Box::new(SecurityMisconfigurationDetector::new()),
        );
        detectors.insert(
            OwaspCategory::VulnerableComponents,
            Box::new(VulnerableComponentsDetector::new()),
        );
        detectors.insert(
            OwaspCategory::AuthenticationFailures,
            Box::new(AuthenticationFailuresDetector::new()),
        );
        detectors.insert(
            OwaspCategory::DataIntegrityFailures,
            Box::new(DataIntegrityFailuresDetector::new()),
        );
        detectors.insert(
            OwaspCategory::LoggingFailures,
            Box::new(LoggingFailuresDetector::new()),
        );
        detectors.insert(
            OwaspCategory::ServerSideRequestForgery,
            Box::new(ServerSideRequestForgeryDetector::new()),
        );

        Ok(detectors)
    }

    /// Create a specific category detector
    pub fn create_detector(category: &OwaspCategory) -> Result<Box<dyn OwaspCategoryDetector>, AnalysisError> {
        match category {
            OwaspCategory::BrokenAccessControl => Ok(Box::new(BrokenAccessControlDetector::new())),
            OwaspCategory::CryptographicFailures => Ok(Box::new(CryptographicFailuresDetector::new())),
            OwaspCategory::Injection => Ok(Box::new(InjectionDetector::new())),
            OwaspCategory::InsecureDesign => Ok(Box::new(InsecureDesignDetector::new())),
            OwaspCategory::SecurityMisconfiguration => Ok(Box::new(SecurityMisconfigurationDetector::new())),
            OwaspCategory::VulnerableComponents => Ok(Box::new(VulnerableComponentsDetector::new())),
            OwaspCategory::AuthenticationFailures => Ok(Box::new(AuthenticationFailuresDetector::new())),
            OwaspCategory::DataIntegrityFailures => Ok(Box::new(DataIntegrityFailuresDetector::new())),
            OwaspCategory::LoggingFailures => Ok(Box::new(LoggingFailuresDetector::new())),
            OwaspCategory::ServerSideRequestForgery => Ok(Box::new(ServerSideRequestForgeryDetector::new())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_all_detectors() {
        let detectors = OwaspTop10Factory::create_all_detectors();
        assert!(detectors.is_ok());
        let detectors = detectors.unwrap();
        assert_eq!(detectors.len(), 10);
    }

    #[test]
    fn test_create_specific_detector() {
        let detector = OwaspTop10Factory::create_detector(&OwaspCategory::Injection);
        assert!(detector.is_ok());
    }
}