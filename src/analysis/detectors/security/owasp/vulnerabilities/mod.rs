//! Vulnerability-specific OWASP security detectors

pub mod command_injection;
pub mod csrf;
pub mod path_traversal;
pub mod session_management;
pub mod sql_injection;

// Re-exports
pub use command_injection::CommandInjectionDetector;
pub use csrf::CsrfDetector;
pub use path_traversal::PathTraversalDetector;
pub use session_management::SessionManagementDetector;
pub use sql_injection::SqlInjectionDetector;

use crate::analysis::detectors::security::owasp::types::OwaspCategoryDetector;

/// Registry for all vulnerability-specific detectors
pub struct VulnerabilityRegistry;

impl VulnerabilityRegistry {
    /// Get all available vulnerability detectors
    pub fn get_detectors() -> Vec<Box<dyn OwaspCategoryDetector>> {
        vec![
            Box::new(SqlInjectionDetector::new()),
            Box::new(CommandInjectionDetector::new()),
            Box::new(PathTraversalDetector::new()),
            Box::new(CsrfDetector::new()),
            Box::new(SessionManagementDetector::new()),
        ]
    }

    /// Get detector names
    pub fn get_detector_names() -> Vec<&'static str> {
        vec![
            "SqlInjectionDetector",
            "CommandInjectionDetector",
            "PathTraversalDetector",
            "CsrfDetector",
            "SessionManagementDetector",
        ]
    }
}

/// Create all vulnerability detectors
pub fn create_all_detectors() -> Vec<Box<dyn OwaspCategoryDetector>> {
    VulnerabilityRegistry::get_detectors()
}

/// Get count of available detectors
pub fn detector_count() -> usize {
    5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detector_count() {
        assert_eq!(detector_count(), 5);
        assert_eq!(VulnerabilityRegistry::get_detector_names().len(), 5);
    }

    #[test]
    fn test_create_detectors() {
        let detectors = create_all_detectors();
        assert_eq!(detectors.len(), 5);
    }
}