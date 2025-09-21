//! OWASP Top 10 Category-based Vulnerability Detection
//!
//! This module organizes OWASP Top 10 vulnerability detection by category.

pub mod injection;
pub mod broken_auth;
pub mod sensitive_data;
pub mod xxe;
pub mod broken_access;
pub mod security_misconfig;
pub mod xss;
pub mod insecure_deserialization;
pub mod vulnerable_components;

pub use injection::InjectionDetector;
pub use broken_auth::BrokenAuthDetector;
pub use sensitive_data::SensitiveDataDetector;
pub use xxe::XXEDetector;
pub use broken_access::BrokenAccessDetector;
pub use security_misconfig::SecurityMisconfigDetector;
pub use xss::XSSDetector;
pub use insecure_deserialization::InsecureDeserializationDetector;
pub use vulnerable_components::VulnerableComponentsDetector;

use crate::analysis::detectors::security::owasp::types::{OwaspCategory, OwaspVulnerability};
use crate::analysis::AnalysisError;
use crate::ast::ParsedFile;

/// Registry for all OWASP category detectors
pub struct CategoryRegistry;

impl CategoryRegistry {
    /// Get all active category detectors
    pub fn get_detectors() -> Vec<Box<dyn OwaspCategoryDetector>> {
        vec![
            Box::new(InjectionDetector::new()),
            Box::new(BrokenAuthDetector::new()),
            Box::new(SensitiveDataDetector::new()),
            Box::new(XXEDetector::new()),
            Box::new(BrokenAccessDetector::new()),
            Box::new(SecurityMisconfigDetector::new()),
            Box::new(XSSDetector::new()),
            Box::new(InsecureDeserializationDetector::new()),
            Box::new(VulnerableComponentsDetector::new()),
        ]
    }
}

/// Trait for OWASP category-specific detectors
#[async_trait::async_trait]
pub trait OwaspCategoryDetector: Send + Sync {
    /// Detect vulnerabilities in the given file
    async fn detect(&self, file: &ParsedFile) -> Result<Vec<OwaspVulnerability>, AnalysisError>;

    /// Get the OWASP category this detector handles
    fn category(&self) -> OwaspCategory;

    /// Get detector description
    fn description(&self) -> &str;
}