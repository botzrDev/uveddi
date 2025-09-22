//! Analyzer focused on dynamically constructed SQL query strings.

use super::{evaluate_patterns, Analyzer, DetectionContext};
use crate::analysis::detectors::security::sql_injection::types::{
    DetectionFinding, SanitizerStatus, SqlInjectionPattern,
};

#[derive(Default)]
pub struct DynamicQueryAnalyzer;

impl Analyzer for DynamicQueryAnalyzer {
    fn analyze(
        &self,
        context: &DetectionContext<'_>,
        patterns: &[SqlInjectionPattern],
        sanitizer: SanitizerStatus,
    ) -> Vec<DetectionFinding> {
        if !(context.line.contains("format!")
            || context.line.contains("format(")
            || context.line.contains("template")
            || context.line.contains("String::from"))
        {
            return Vec::new();
        }

        evaluate_patterns(context, patterns.iter(), sanitizer)
    }
}
