use super::super::types::{
    DetectionContext, DetectionFinding, PatternCategory, SanitizationStatus, SqlInjectionPattern,
};

/// Analyse stored procedure invocations susceptible to injection.
pub fn analyze(
    context: &DetectionContext,
    patterns: &[SqlInjectionPattern],
    sanitization: &SanitizationStatus,
) -> Vec<DetectionFinding> {
    patterns
        .iter()
        .filter(|pattern| pattern.category == PatternCategory::StoredProcedure)
        .filter_map(|pattern| pattern.evaluate(context, sanitization))
        .collect()
}
