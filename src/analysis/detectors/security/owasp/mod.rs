//! OWASP Top 10 2021 vulnerability detection framework
//!
//! This module provides comprehensive coverage of the OWASP Top 10 security
//! vulnerabilities with specialized detection strategies for each category.

pub mod categories;
pub mod config;
pub mod detector;
pub mod language_support;
pub mod scanners;
pub mod types;
pub mod vulnerabilities;

// Core re-exports
pub use config::OwaspConfig;
pub use detector::{OwaspDetector, OwaspComprehensiveResult, DetectorStatistics};
pub use types::{OwaspCategory, OwaspVulnerability, OwaspCategoryDetector};

// Note: Analysis and reporting modules removed per assignment requirements

// Category detector re-exports
pub use categories::{
    InjectionDetector, BrokenAuthDetector, SensitiveDataDetector, XXEDetector,
    BrokenAccessDetector, SecurityMisconfigDetector, XSSDetector,
    InsecureDeserializationDetector, VulnerableComponentsDetector, CategoryRegistry,
};

// Language support re-exports
pub use language_support::{
    RustOwaspAnalyzer, PythonOwaspAnalyzer, JavaScriptOwaspAnalyzer,
    LanguageSupportCoordinator, WebFrameworkAnalyzer, ApiSecurityAnalyzer, DatabaseSecurityAnalyzer,
};

// Scanner re-exports
pub use scanners::{
    StaticScanner, PatternScanner, FlowScanner, DependencyScanner,
    Scanner, ScannerOrchestrator, UnifiedScanResult,
};

// Vulnerability detector re-exports
pub use vulnerabilities::{
    SqlInjectionDetector, CommandInjectionDetector, PathTraversalDetector,
    CSRFDetector, SessionManagementDetector, VulnerabilityRegistry,
};

/// OWASP Top 10 2021 categories
pub const OWASP_TOP_10_CATEGORIES: [OwaspCategory; 10] = [
    OwaspCategory::BrokenAccessControl,
    OwaspCategory::CryptographicFailures,
    OwaspCategory::Injection,
    OwaspCategory::InsecureDesign,
    OwaspCategory::SecurityMisconfiguration,
    OwaspCategory::VulnerableComponents,
    OwaspCategory::AuthenticationFailures,
    OwaspCategory::DataIntegrityFailures,
    OwaspCategory::LoggingFailures,
    OwaspCategory::ServerSideRequestForgery,
];

/// Get OWASP category by identifier
pub fn get_category_by_identifier(identifier: &str) -> Option<OwaspCategory> {
    match identifier {
        "A01:2021" => Some(OwaspCategory::BrokenAccessControl),
        "A02:2021" => Some(OwaspCategory::CryptographicFailures),
        "A03:2021" => Some(OwaspCategory::Injection),
        "A04:2021" => Some(OwaspCategory::InsecureDesign),
        "A05:2021" => Some(OwaspCategory::SecurityMisconfiguration),
        "A06:2021" => Some(OwaspCategory::VulnerableComponents),
        "A07:2021" => Some(OwaspCategory::AuthenticationFailures),
        "A08:2021" => Some(OwaspCategory::DataIntegrityFailures),
        "A09:2021" => Some(OwaspCategory::LoggingFailures),
        "A10:2021" => Some(OwaspCategory::ServerSideRequestForgery),
        _ => None,
    }
}

/// Create a detector with high sensitivity configuration
pub fn create_high_sensitivity_detector() -> Result<OwaspDetector, crate::analysis::AnalysisError> {
    let config = OwaspConfig::high_sensitivity();
    OwaspDetector::with_config(config)
}

/// Create a detector optimized for low false positives
pub fn create_low_false_positive_detector() -> Result<OwaspDetector, crate::analysis::AnalysisError> {
    let config = OwaspConfig::low_false_positives();
    OwaspDetector::with_config(config)
}