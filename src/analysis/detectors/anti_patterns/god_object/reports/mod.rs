//! Report formatting for God Object detection

mod json_formatter;
mod markdown_formatter;
mod structured_formatter;
mod utils;

pub use json_formatter::JsonFormatter;
pub use markdown_formatter::MarkdownFormatter;
pub use structured_formatter::StructuredFormatter;
use utils::*;

use crate::database::models::ArchitecturalIssue;
use serde_json::Value;
use std::collections::HashMap;

/// Main report formatter that coordinates different output formats
pub struct GodObjectReportFormatter;

impl GodObjectReportFormatter {
    /// Format issues as JSON
    pub fn format_as_json(issues: &[ArchitecturalIssue]) -> Result<String, serde_json::Error> {
        JsonFormatter::format(issues)
    }

    /// Format issues as Markdown report
    pub fn format_as_markdown(issues: &[ArchitecturalIssue]) -> String {
        MarkdownFormatter::format(issues)
    }

    /// Format issues as structured data for external consumption
    pub fn format_as_structured_data(issues: &[ArchitecturalIssue]) -> Value {
        StructuredFormatter::format(issues)
    }

    /// Generate a summary report for multiple files
    pub fn generate_summary_report(all_issues: &[ArchitecturalIssue]) -> String {
        MarkdownFormatter::generate_summary(all_issues)
    }

    /// Calculate severity distribution (shared utility)
    pub fn calculate_severity_distribution(
        issues: &[ArchitecturalIssue],
    ) -> HashMap<String, usize> {
        calculate_severity_distribution(issues)
    }
}
