//! Analyzer targeting stored procedure interactions.

use super::{evaluate_patterns, Analyzer, DetectionContext};
use crate::analysis::detectors::security::sql_injection::types::{
    DetectionFinding, SanitizerStatus, SqlInjectionPattern,
};

#[derive(Default)]
pub struct StoredProcedureAnalyzer;

impl Analyzer for StoredProcedureAnalyzer {
    fn analyze(
        &self,
        context: &DetectionContext<'_>,
        patterns: &[SqlInjectionPattern],
        sanitizer: SanitizerStatus,
    ) -> Vec<DetectionFinding> {
        if !(context.line.contains("CALL ") || context.line.contains("exec")) {
            return Vec::new();
        }

        evaluate_patterns(context, patterns.iter(), sanitizer)
    }
}
