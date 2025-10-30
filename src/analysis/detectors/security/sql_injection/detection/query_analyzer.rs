//! Analyzer focusing on direct SQL query construction issues.

use super::{evaluate_patterns, Analyzer, DetectionContext};
use crate::analysis::detectors::security::sql_injection::types::{
    DetectionFinding, SanitizerStatus, SqlInjectionPattern,
};

#[derive(Default)]
pub struct QueryAnalyzer;

impl Analyzer for QueryAnalyzer {
    fn analyze(
        &self,
        context: &DetectionContext<'_>,
        patterns: &[SqlInjectionPattern],
        sanitizer: SanitizerStatus,
    ) -> Vec<DetectionFinding> {
        if context.line.trim().is_empty() {
            return Vec::new();
        }

        evaluate_patterns(context, patterns.iter(), sanitizer)
    }
}
