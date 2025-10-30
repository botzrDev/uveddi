//! Analyzer that checks parameter usage in SQL statements.

use super::{evaluate_patterns, Analyzer, DetectionContext};
use crate::analysis::detectors::security::sql_injection::types::{
    DetectionFinding, SanitizerStatus, SqlInjectionPattern,
};

#[derive(Default)]
pub struct ParameterAnalyzer;

impl Analyzer for ParameterAnalyzer {
    fn analyze(
        &self,
        context: &DetectionContext<'_>,
        patterns: &[SqlInjectionPattern],
        sanitizer: SanitizerStatus,
    ) -> Vec<DetectionFinding> {
        if !context.line.contains('=') && !context.line.contains("execute") {
            return Vec::new();
        }

        evaluate_patterns(context, patterns.iter(), sanitizer)
    }
}
