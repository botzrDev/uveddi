//! AI-powered insights for report enhancement
//!
//! This module contains AI integration for generating enhanced explanations and recommendations.

use crate::report::generators::ReportGenerator;

impl ReportGenerator {
    /// Check if AI explanations should be included
    pub fn should_include_ai_explanations(&self) -> bool {
        self.include_ai_explanations
    }

    /// Check if code snippets should be included
    pub fn should_include_code_snippets(&self) -> bool {
        self.include_code_snippets
    }

    /// Check if diagrams should be included
    pub fn should_include_diagrams(&self) -> bool {
        self.include_diagrams
    }

    /// Check if severity summary should be included
    pub fn should_include_severity_summary(&self) -> bool {
        self.include_severity_summary
    }

    /// Check if remediation steps should be included
    pub fn should_include_remediation_steps(&self) -> bool {
        self.include_remediation_steps
    }
}
