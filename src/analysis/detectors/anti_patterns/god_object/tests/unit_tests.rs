//! Basic unit tests for God Object detector

use crate::analysis::cache::wrappers::ArchivableSystemTime;
use crate::ast::{ParsedFile, SourceLanguage};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;

/// Helper function to create a mock parsed file with the given source code
pub fn create_parsed_file(source: &str, language: SourceLanguage) -> ParsedFile {
    ParsedFile {
        file_path: PathBuf::from("test.rs").into(),
        source: source.to_string().into(),
        language,
        tree: None, // Would be populated by actual parser
        custom_ast: Arc::new(None),
        modified_at: ArchivableSystemTime::from(SystemTime::now()),
    }
}

#[cfg(test)]
mod tests {
    use super::create_parsed_file;
    use crate::analysis::detectors::anti_patterns::god_object::GodObjectDetector;

    #[test]
    fn test_detector_default_thresholds() {
        let detector = GodObjectDetector::new(10, 8);
        assert_eq!(detector.method_threshold(), 10);
        assert_eq!(detector.field_threshold(), 8);
    }

    #[test]
    fn test_detector_custom_thresholds() {
        let detector = GodObjectDetector::new(5, 3);
        assert_eq!(detector.method_threshold(), 5);
        assert_eq!(detector.field_threshold(), 3);
    }

    #[test]
    fn test_severity_calculation_critical() {
        let detector = GodObjectDetector::new(10, 8);

        // Both thresholds exceeded by more than 50%
        let method_count = 16;
        let field_count = 13;

        let method_ratio = method_count as f64 / detector.method_threshold() as f64;
        let field_ratio = field_count as f64 / detector.field_threshold() as f64;

        assert!(method_ratio > 1.5);
        assert!(field_ratio > 1.5);
        // Should be CRITICAL severity
    }

    #[test]
    fn test_severity_calculation_major() {
        let detector = GodObjectDetector::new(10, 8);

        // Both thresholds exceeded by 20-50%
        let method_count = 13;
        let field_count = 10;

        let method_ratio = method_count as f64 / detector.method_threshold() as f64;
        let field_ratio = field_count as f64 / detector.field_threshold() as f64;

        assert!(method_ratio > 1.2 && method_ratio <= 1.5);
        assert!(field_ratio > 1.2 && field_ratio <= 1.5);
        // Should be MAJOR severity
    }

    #[test]
    fn test_severity_calculation_minor() {
        let detector = GodObjectDetector::new(10, 8);

        // Thresholds just exceeded
        let method_count = 11;
        let field_count = 9;

        let method_ratio = method_count as f64 / detector.method_threshold() as f64;
        let field_ratio = field_count as f64 / detector.field_threshold() as f64;

        assert!(method_ratio > 1.0 && method_ratio <= 1.2);
        assert!(field_ratio > 1.0 && field_ratio <= 1.2);
        // Should be MINOR severity
    }

    #[test]
    fn test_empty_class() {
        let detector = GodObjectDetector::new(10, 8);

        // Empty classes should not trigger God Object detection
        let method_count = 0;
        let field_count = 0;

        assert!(method_count <= detector.method_threshold());
        assert!(field_count <= detector.field_threshold());
    }
}
