//! OWASP Top 10 2021 vulnerability detection framework
//!
//! This module provides comprehensive coverage of the OWASP Top 10 security
//! vulnerabilities with specialized detection strategies for each category.
//! It correlates architectural anti-patterns with security risks to provide
//! developers with contextual understanding of vulnerabilities.
//!
//! ## Module Structure
//!
//! - `types`: Core types and enums for OWASP vulnerability representation
//! - `config`: Configuration types for detector behavior
//! - `detector`: Main detector entry point and orchestration
//! - `top10/`: Individual detectors for each OWASP Top 10 category
//! - `analysis/`: Risk analysis, severity calculation, and remediation guidance
//! - `language_support/`: Language-specific security analysis
//! - `reporting/`: Compliance reporting and risk assessment
//!
//! ## Usage
//!
//! ```rust
//! use uveddi::analysis::detectors::security::owasp::{OwaspDetector, OwaspConfig};
//!
//! // Create detector with default configuration
//! let detector = OwaspDetector::new()?;
//!
//! // Analyze a file for OWASP vulnerabilities
//! let vulnerabilities = detector.analyze_file(&parsed_file).await?;
//!
//! // Generate comprehensive analysis report
//! let analysis = detector.comprehensive_analysis(&parsed_file).await?;
//! ```

pub mod analysis;
pub mod config;
pub mod detector;
pub mod language_support;
pub mod reporting;
pub mod top10;
pub mod types;

// Re-exports for convenience
pub use config::OwaspConfig;
pub use detector::{OwaspDetector, OwaspComprehensiveResult, DetectorStatistics};
pub use types::{OwaspCategory, OwaspVulnerability, OwaspCategoryDetector};

// Analysis and reporting re-exports
pub use analysis::{OwaspAnalysisOrchestrator, OwaspAnalysisResult};
pub use reporting::{OwaspReportingCoordinator, OwaspAnalysisReport, ExecutiveSummary};

// Language support re-exports
pub use language_support::{
    LanguageSupportCoordinator, WebFrameworkAnalyzer, ApiSecurityAnalyzer, DatabaseSecurityAnalyzer,
};

/// OWASP Top 10 2021 categories as a constant array for iteration
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

/// Get all OWASP Top 10 identifiers
pub fn get_all_identifiers() -> Vec<&'static str> {
    OWASP_TOP_10_CATEGORIES
        .iter()
        .map(|category| category.identifier())
        .collect()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_identifiers() {
        assert_eq!(get_all_identifiers().len(), 10);
        assert!(get_all_identifiers().contains(&"A01:2021"));
        assert!(get_all_identifiers().contains(&"A10:2021"));
    }

    #[test]
    fn test_get_category_by_identifier() {
        assert_eq!(
            get_category_by_identifier("A03:2021"),
            Some(OwaspCategory::Injection)
        );
        assert_eq!(
            get_category_by_identifier("A01:2021"),
            Some(OwaspCategory::BrokenAccessControl)
        );
        assert_eq!(get_category_by_identifier("A99:2021"), None);
    }

    #[test]
    fn test_owasp_constants() {
        assert_eq!(OWASP_TOP_10_CATEGORIES.len(), 10);
        assert_eq!(OWASP_TOP_10_CATEGORIES[0], OwaspCategory::BrokenAccessControl);
        assert_eq!(OWASP_TOP_10_CATEGORIES[9], OwaspCategory::ServerSideRequestForgery);
    }

    #[test]
    fn test_detector_factory_functions() {
        let high_sensitivity = create_high_sensitivity_detector();
        assert!(high_sensitivity.is_ok());

        let low_fp = create_low_false_positive_detector();
        assert!(low_fp.is_ok());

        let high_detector = high_sensitivity.unwrap();
        let low_detector = low_fp.unwrap();

        assert!(high_detector.get_config().confidence_threshold < low_detector.get_config().confidence_threshold);
    }
}