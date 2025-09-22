//! Input validation heuristics that indicate sanitization is present.

#[derive(Default)]
pub struct InputValidationAnalyzer;

impl InputValidationAnalyzer {
    pub fn validates_input(&self, line: &str) -> bool {
        let lowered = line.to_lowercase();
        lowered.contains("validate_")
            || lowered.contains("is_valid_input")
            || lowered.contains("sanitize(")
    }
}
