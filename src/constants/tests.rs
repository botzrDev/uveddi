//! Tests for application constants
//!
//! These tests ensure that all constants maintain their expected values
//! and relationships between different threshold values.

#[cfg(test)]
mod tests {
    use super::super::{detector_thresholds, severity_weights, tui_constants};

    #[test]
    fn test_rust_thresholds_are_reasonable() {
        // Test that Rust thresholds are within reasonable bounds
        assert!(detector_thresholds::rust::MAX_LOGICAL_LOC > 0);
        assert!(detector_thresholds::rust::MAX_LOGICAL_LOC <= 1000);

        assert!(detector_thresholds::rust::MAX_METHODS > 0);
        assert!(detector_thresholds::rust::MAX_METHODS <= 50);

        assert!(detector_thresholds::rust::MAX_FIELDS > 0);
        assert!(detector_thresholds::rust::MAX_FIELDS <= 30);

        assert!(detector_thresholds::rust::MAX_CYCLOMATIC_COMPLEXITY > 0);
        assert!(detector_thresholds::rust::MAX_CYCLOMATIC_COMPLEXITY <= 100);

        assert!(detector_thresholds::rust::MAX_COGNITIVE_COMPLEXITY > 0);
        assert!(detector_thresholds::rust::MAX_COGNITIVE_COMPLEXITY <= 100);

        assert!(detector_thresholds::rust::MAX_LCOM_SCORE >= 0.0);
        assert!(detector_thresholds::rust::MAX_LCOM_SCORE <= 1.0);

        assert!(detector_thresholds::rust::MAX_COUPLING > 0);
        assert!(detector_thresholds::rust::MAX_COUPLING <= 50);
    }

    #[test]
    fn test_python_thresholds_are_higher_than_rust() {
        // Python should generally have higher thresholds due to language characteristics
        assert!(
            detector_thresholds::python::MAX_LOGICAL_LOC
                >= detector_thresholds::rust::MAX_LOGICAL_LOC
        );
        assert!(detector_thresholds::python::MAX_METHODS >= detector_thresholds::rust::MAX_METHODS);
        assert!(detector_thresholds::python::MAX_FIELDS >= detector_thresholds::rust::MAX_FIELDS);
        assert!(
            detector_thresholds::python::MAX_CYCLOMATIC_COMPLEXITY
                >= detector_thresholds::rust::MAX_CYCLOMATIC_COMPLEXITY
        );
        assert!(
            detector_thresholds::python::MAX_COGNITIVE_COMPLEXITY
                >= detector_thresholds::rust::MAX_COGNITIVE_COMPLEXITY
        );
        assert!(
            detector_thresholds::python::MAX_COUPLING >= detector_thresholds::rust::MAX_COUPLING
        );
    }

    #[test]
    fn test_javascript_thresholds_are_highest() {
        // JavaScript should have the highest thresholds due to prototype-based patterns
        assert!(
            detector_thresholds::javascript::MAX_LOGICAL_LOC
                >= detector_thresholds::python::MAX_LOGICAL_LOC
        );
        assert!(
            detector_thresholds::javascript::MAX_METHODS
                >= detector_thresholds::python::MAX_METHODS
        );
        assert!(
            detector_thresholds::javascript::MAX_FIELDS >= detector_thresholds::python::MAX_FIELDS
        );
        assert!(
            detector_thresholds::javascript::MAX_CYCLOMATIC_COMPLEXITY
                >= detector_thresholds::python::MAX_CYCLOMATIC_COMPLEXITY
        );
        assert!(
            detector_thresholds::javascript::MAX_COGNITIVE_COMPLEXITY
                >= detector_thresholds::python::MAX_COGNITIVE_COMPLEXITY
        );
        assert!(
            detector_thresholds::javascript::MAX_COUPLING
                >= detector_thresholds::python::MAX_COUPLING
        );
    }

    #[test]
    fn test_severity_weights_sum_to_one() {
        let total = severity_weights::SIZE_WEIGHT
            + severity_weights::COMPLEXITY_WEIGHT
            + severity_weights::STRUCTURAL_WEIGHT;

        // Allow for small floating-point differences
        assert!(
            (total - 1.0).abs() < 0.001,
            "Severity weights should sum to 1.0, got {}",
            total
        );
    }

    #[test]
    fn test_severity_weights_are_positive() {
        assert!(severity_weights::SIZE_WEIGHT > 0.0);
        assert!(severity_weights::COMPLEXITY_WEIGHT > 0.0);
        assert!(severity_weights::STRUCTURAL_WEIGHT > 0.0);
    }

    #[test]
    fn test_tui_form_constraints_are_valid() {
        // Confidence constraints
        assert_eq!(tui_constants::form_constraints::MIN_CONFIDENCE, 0.0);
        assert_eq!(tui_constants::form_constraints::MAX_CONFIDENCE, 1.0);
        assert!(tui_constants::form_constraints::CONFIDENCE_DECIMAL_PLACES <= 10);

        // Count constraints
        assert!(tui_constants::form_constraints::MIN_COUNT_VALUE > 0.0);

        // Severity constraints
        assert_eq!(tui_constants::form_constraints::MIN_SEVERITY, 0.0);
        assert_eq!(tui_constants::form_constraints::MAX_SEVERITY, 100.0);
    }

    #[test]
    fn test_tui_form_defaults_are_parseable() {
        // Test that all default string values can be parsed
        assert!(tui_constants::form_defaults::DEAD_CODE_CONFIDENCE
            .parse::<f64>()
            .is_ok());
        assert!(tui_constants::form_defaults::LARGE_CLASSES_MAX_LOC
            .parse::<u32>()
            .is_ok());
        assert!(tui_constants::form_defaults::LARGE_CLASSES_MAX_METHODS
            .parse::<u32>()
            .is_ok());
        assert!(tui_constants::form_defaults::LARGE_CLASSES_MAX_FIELDS
            .parse::<u32>()
            .is_ok());
        assert!(tui_constants::form_defaults::LARGE_CLASSES_MAX_COMPLEXITY
            .parse::<u32>()
            .is_ok());
        assert!(tui_constants::form_defaults::LARGE_CLASSES_MAX_LCOM
            .parse::<f64>()
            .is_ok());
        assert!(tui_constants::form_defaults::LARGE_CLASSES_MIN_SEVERITY
            .parse::<u32>()
            .is_ok());
    }

    #[test]
    fn test_form_defaults_match_detector_thresholds() {
        // The form defaults should match the Rust detector thresholds (as they're the primary language)
        let form_loc: u32 = tui_constants::form_defaults::LARGE_CLASSES_MAX_LOC
            .parse()
            .unwrap();
        let form_methods: u32 = tui_constants::form_defaults::LARGE_CLASSES_MAX_METHODS
            .parse()
            .unwrap();
        let form_fields: u32 = tui_constants::form_defaults::LARGE_CLASSES_MAX_FIELDS
            .parse()
            .unwrap();
        let form_complexity: u32 = tui_constants::form_defaults::LARGE_CLASSES_MAX_COMPLEXITY
            .parse()
            .unwrap();
        let form_lcom: f64 = tui_constants::form_defaults::LARGE_CLASSES_MAX_LCOM
            .parse()
            .unwrap();

        assert_eq!(form_loc, detector_thresholds::rust::MAX_LOGICAL_LOC);
        assert_eq!(form_methods, detector_thresholds::rust::MAX_METHODS);
        assert_eq!(form_fields, detector_thresholds::rust::MAX_FIELDS);
        assert_eq!(
            form_complexity,
            detector_thresholds::rust::MAX_CYCLOMATIC_COMPLEXITY
        );
        assert_eq!(form_lcom, detector_thresholds::rust::MAX_LCOM_SCORE);
    }

    #[test]
    fn test_form_defaults_within_constraints() {
        // Test that all form defaults are within the defined constraints
        let confidence: f64 = tui_constants::form_defaults::DEAD_CODE_CONFIDENCE
            .parse()
            .unwrap();
        assert!(confidence >= tui_constants::form_constraints::MIN_CONFIDENCE);
        assert!(confidence <= tui_constants::form_constraints::MAX_CONFIDENCE);

        let severity: f64 = tui_constants::form_defaults::LARGE_CLASSES_MIN_SEVERITY
            .parse()
            .unwrap();
        assert!(severity >= tui_constants::form_constraints::MIN_SEVERITY);
        assert!(severity <= tui_constants::form_constraints::MAX_SEVERITY);

        let loc: f64 = tui_constants::form_defaults::LARGE_CLASSES_MAX_LOC
            .parse()
            .unwrap();
        assert!(loc >= tui_constants::form_constraints::MIN_COUNT_VALUE);
    }
}
