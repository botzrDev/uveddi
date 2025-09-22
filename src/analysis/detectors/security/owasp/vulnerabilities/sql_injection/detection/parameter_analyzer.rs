use super::super::types::{
    DetectionContext, DetectionFinding, PatternCategory, SanitizationStatus, SqlInjectionPattern,
};

/// Analyse unsafe parameter usage and concatenation in SQL execution APIs.
pub fn analyze(
    context: &DetectionContext,
    patterns: &[SqlInjectionPattern],
    sanitization: &SanitizationStatus,
) -> Vec<DetectionFinding> {
    patterns
        .iter()
        .filter(|pattern| pattern.category == PatternCategory::Parameter)
        .filter_map(|pattern| pattern.evaluate(context, sanitization))
        .collect()
}
