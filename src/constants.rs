//! Application-wide constants
//!
//! This module contains all magic numbers and threshold values used throughout the application,
//! organized by domain for better maintainability and configurability.

/// Detector threshold constants for various programming languages and metrics
pub mod detector_thresholds {
    /// Rust-specific thresholds based on language characteristics
    pub mod rust {
        /// Maximum logical lines of code for a class/struct
        /// Based on Clean Code by Robert Martin - maximum class size
        pub const MAX_LOGICAL_LOC: u32 = 400;

        /// Maximum number of methods in a class/struct
        /// Rust's ownership system enables smaller interfaces
        pub const MAX_METHODS: u32 = 10;

        /// Maximum number of fields in a struct
        /// Struct composition over inheritance reduces field needs
        pub const MAX_FIELDS: u32 = 15;

        /// Maximum cyclomatic complexity for a class
        /// Based on industry best practices for maintainable code
        pub const MAX_CYCLOMATIC_COMPLEXITY: u32 = 50;

        /// Maximum cognitive complexity for a class
        /// Cognitive complexity focuses on human readability
        pub const MAX_COGNITIVE_COMPLEXITY: u32 = 40;

        /// Maximum LCOM (Lack of Cohesion of Methods) score
        /// Values closer to 1.0 indicate poor cohesion
        pub const MAX_LCOM_SCORE: f64 = 0.8;

        /// Maximum coupling count for a class
        /// Based on coupling analysis best practices
        pub const MAX_COUPLING: u32 = 12;
    }

    /// Python-specific thresholds adjusted for language patterns
    pub mod python {
        /// Python classes tend to be larger due to language patterns
        pub const MAX_LOGICAL_LOC: u32 = 500;

        /// Python's dynamic nature allows for more methods
        pub const MAX_METHODS: u32 = 25;

        /// Python's flexibility allows for more fields
        pub const MAX_FIELDS: u32 = 20;

        /// Python's interpreted nature requires slightly higher complexity tolerance
        pub const MAX_CYCLOMATIC_COMPLEXITY: u32 = 60;

        /// Cognitive complexity threshold for Python
        pub const MAX_COGNITIVE_COMPLEXITY: u32 = 50;

        /// LCOM threshold for Python classes
        pub const MAX_LCOM_SCORE: f64 = 0.8;

        /// Coupling threshold for Python modules
        pub const MAX_COUPLING: u32 = 15;
    }

    /// JavaScript-specific thresholds for prototype-based patterns
    pub mod javascript {
        /// JavaScript classes can be larger due to prototype patterns
        pub const MAX_LOGICAL_LOC: u32 = 600;

        /// JavaScript prototype-based patterns allow more methods
        pub const MAX_METHODS: u32 = 30;

        /// JavaScript's flexible object model allows more fields
        pub const MAX_FIELDS: u32 = 25;

        /// JavaScript's event-driven nature requires higher complexity tolerance
        pub const MAX_CYCLOMATIC_COMPLEXITY: u32 = 70;

        /// Cognitive complexity threshold for JavaScript
        pub const MAX_COGNITIVE_COMPLEXITY: u32 = 60;

        /// LCOM threshold for JavaScript classes
        pub const MAX_LCOM_SCORE: f64 = 0.8;

        /// Coupling threshold for JavaScript modules
        pub const MAX_COUPLING: u32 = 18;
    }
}

/// TUI (Terminal User Interface) layout and form constants
pub mod tui_constants {
    /// Default form field values and constraints
    pub mod form_defaults {
        /// Default dead code detection confidence threshold
        pub const DEAD_CODE_CONFIDENCE: &str = "0.8";

        /// Default maximum lines of code threshold for large classes
        pub const LARGE_CLASSES_MAX_LOC: &str = "400";

        /// Default maximum methods threshold for large classes
        pub const LARGE_CLASSES_MAX_METHODS: &str = "20";

        /// Default maximum fields threshold for large classes
        pub const LARGE_CLASSES_MAX_FIELDS: &str = "15";

        /// Default maximum complexity threshold for large classes
        pub const LARGE_CLASSES_MAX_COMPLEXITY: &str = "50";

        /// Default maximum LCOM threshold for large classes
        pub const LARGE_CLASSES_MAX_LCOM: &str = "0.8";

        /// Default minimum severity threshold for large classes
        pub const LARGE_CLASSES_MIN_SEVERITY: &str = "25";
    }

    /// Form input constraints and validation values
    pub mod form_constraints {
        /// Minimum value for confidence inputs (0.0 to 1.0 range)
        pub const MIN_CONFIDENCE: f64 = 0.0;

        /// Maximum value for confidence inputs (0.0 to 1.0 range)
        pub const MAX_CONFIDENCE: f64 = 1.0;

        /// Decimal places for confidence inputs
        pub const CONFIDENCE_DECIMAL_PLACES: u8 = 2;

        /// Minimum value for count-based inputs (lines, methods, fields)
        pub const MIN_COUNT_VALUE: f64 = 1.0;

        /// Maximum severity value (0-100 range)
        pub const MAX_SEVERITY: f64 = 100.0;

        /// Minimum severity value
        pub const MIN_SEVERITY: f64 = 0.0;
    }
}

/// Severity calculation weights for different metrics
pub mod severity_weights {
    /// Weight for size-based metrics (LOC, methods, fields)
    pub const SIZE_WEIGHT: f64 = 0.4;

    /// Weight for complexity-based metrics (cyclomatic, cognitive)
    pub const COMPLEXITY_WEIGHT: f64 = 0.35;

    /// Weight for structural metrics (LCOM, coupling)
    pub const STRUCTURAL_WEIGHT: f64 = 0.25;
}

#[cfg(test)]
mod tests;
