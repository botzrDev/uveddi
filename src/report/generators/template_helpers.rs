//! Template helper functions and utilities
//!
//! This module contains shared formatting utilities for report generation.

use crate::database::models::{AnalysisRun, ArchitecturalIssue};
use crate::report::generators::ReportGenerator;
use std::collections::HashMap;

impl ReportGenerator {
    /// Format file path for display in reports
    pub fn format_file_path(&self, path: &str) -> String {
        // Truncate very long paths for readability
        if path.len() > 80 {
            format!("...{}", &path[path.len() - 77..])
        } else {
            path.to_string()
        }
    }

    /// Generate severity icon class for HTML/CSS
    pub fn get_severity_icon(&self, severity: &str) -> &'static str {
        match severity {
            "Critical" => "fas fa-exclamation-circle",
            "High" => "fas fa-exclamation-triangle",
            "Medium" => "fas fa-exclamation",
            "Low" => "fas fa-info-circle",
            _ => "fas fa-question-circle",
        }
    }

    /// Generate severity color class for styling
    pub fn get_severity_color_class(&self, severity: &str) -> String {
        format!("severity-{}", severity.to_lowercase())
    }

    /// Format timestamp for display
    pub fn format_timestamp(&self, timestamp: &chrono::DateTime<chrono::Local>) -> String {
        timestamp.format("%B %d, %Y at %H:%M UTC").to_string()
    }

    /// Calculate percentage for metrics
    pub fn calculate_percentage(&self, part: usize, total: usize) -> f64 {
        if total == 0 {
            0.0
        } else {
            (part as f64 / total as f64) * 100.0
        }
    }
}
