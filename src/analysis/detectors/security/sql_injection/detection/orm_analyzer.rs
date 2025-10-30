//! Analyzer that inspects ORM-specific query construction for injection risks.

use super::{evaluate_patterns, Analyzer, DetectionContext};
use crate::analysis::detectors::security::sql_injection::types::{
    DetectionFinding, SanitizerStatus, SqlInjectionPattern,
};

#[derive(Default)]
pub struct OrmAnalyzer;

impl Analyzer for OrmAnalyzer {
    fn analyze(
        &self,
        context: &DetectionContext<'_>,
        patterns: &[SqlInjectionPattern],
        sanitizer: SanitizerStatus,
    ) -> Vec<DetectionFinding> {
        if !(context.line.contains("find(")
            || context.line.contains("filter(")
            || context.line.contains("aggregate"))
        {
            return Vec::new();
        }

        evaluate_patterns(context, patterns.iter(), sanitizer)
    }
}
