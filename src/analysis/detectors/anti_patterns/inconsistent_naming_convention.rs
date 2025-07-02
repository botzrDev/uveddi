//! Detector for the Inconsistent Naming Convention anti-pattern.

use crate::analysis::{AnalysisDetector, AnalysisError};
use crate::ast::tree_sitter::ParsedFile;
use crate::database::models::{AntiPatternType, ArchitecturalIssue};

#[derive(Default)]
pub struct InconsistentNamingConventionDetector;

impl InconsistentNamingConventionDetector {
    pub fn new() -> Self {
        Self {}
    }
    // TODO: Implement detection logic for inconsistent naming conventions
    // - Mixed camelCase and snake_case in same scope
    // - Inconsistent capitalization for constants
    // - Non-descriptive variable names (a, b, temp, data)
    // - Violating language-specific naming conventions
}

impl AnalysisDetector for InconsistentNamingConventionDetector {
    fn get_detector_name(&self) -> &'static str {
        "InconsistentNamingConventionDetector"
    }

    fn get_anti_pattern_types(&self) -> Vec<AntiPatternType> {
        vec![AntiPatternType {
            anti_pattern_type_id: None,
            name: "Inconsistent Naming Convention".to_string(),
            description: "Mixed naming conventions within the same scope or violation of language-specific naming standards.".to_string(),
            category: "Code Style".to_string(),
        }]
    }

    fn detect_issues(
        &self,
        _parsed_file: &ParsedFile,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        // TODO: Implement actual detection logic
        Ok(vec![])
    }
}
