use super::super::types::{
    DetectionContext, DetectionFinding, PatternCategory, SanitizationStatus, SqlInjectionPattern,
};

/// Analyse static SQL query statements for injection indicators.
pub fn analyze(
    context: &DetectionContext,
    patterns: &[SqlInjectionPattern],
    sanitization: &SanitizationStatus,
) -> Vec<DetectionFinding> {
    patterns
        .iter()
        .filter(|pattern| pattern.category == PatternCategory::Query)
        .filter_map(|pattern| pattern.evaluate(context, sanitization))
        .collect()
}
