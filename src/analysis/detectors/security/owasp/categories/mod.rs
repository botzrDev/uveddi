//! OWASP Top 10 Category-based Vulnerability Detection
//!
//! This module organizes OWASP Top 10 vulnerability detection by category.

pub mod broken_access;
pub mod broken_auth;
pub mod injection;
pub mod insecure_deserialization;
pub mod security_misconfig;
pub mod sensitive_data;
pub mod vulnerable_components;
pub mod xss;
pub mod xxe;

pub use broken_access::BrokenAccessDetector;
pub use broken_auth::BrokenAuthDetector;
pub use injection::InjectionDetector;
pub use insecure_deserialization::InsecureDeserializationDetector;
pub use security_misconfig::SecurityMisconfigDetector;
pub use sensitive_data::SensitiveDataDetector;
pub use vulnerable_components::VulnerableComponentsDetector;
pub use xss::XSSDetector;
pub use xxe::XXEDetector;

use crate::analysis::detectors::security::owasp::types::{OwaspCategory, OwaspCategoryDetector};

/// Registry for all OWASP category detectors
pub struct CategoryRegistry;

impl CategoryRegistry {
    /// Get all active category detectors
    pub fn get_detectors() -> Vec<(OwaspCategory, Box<dyn OwaspCategoryDetector>)> {
        vec![
            (
                OwaspCategory::Injection,
                Box::new(InjectionDetector::new()) as Box<_>,
            ),
            (
                OwaspCategory::AuthenticationFailures,
                Box::new(BrokenAuthDetector::new()) as Box<_>,
            ),
            (
                OwaspCategory::CryptographicFailures,
                Box::new(SensitiveDataDetector::new()) as Box<_>,
            ),
            (
                OwaspCategory::ServerSideRequestForgery,
                Box::new(XXEDetector::new()) as Box<_>,
            ),
            (
                OwaspCategory::BrokenAccessControl,
                Box::new(BrokenAccessDetector::new()) as Box<_>,
            ),
            (
                OwaspCategory::SecurityMisconfiguration,
                Box::new(SecurityMisconfigDetector::new()) as Box<_>,
            ),
            (
                OwaspCategory::InsecureDesign,
                Box::new(XSSDetector::new()) as Box<_>,
            ),
            (
                OwaspCategory::DataIntegrityFailures,
                Box::new(InsecureDeserializationDetector::new()) as Box<_>,
            ),
            (
                OwaspCategory::VulnerableComponents,
                Box::new(VulnerableComponentsDetector::new()) as Box<_>,
            ),
        ]
    }
}
