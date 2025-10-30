use super::super::types::{
    DetectionContext, DetectionFinding, PatternCategory, SanitizationStatus, SqlInjectionPattern,
};

/// Analyse ORM and NoSQL query APIs for injection patterns.
pub fn analyze(
    context: &DetectionContext,
    patterns: &[SqlInjectionPattern],
    sanitization: &SanitizationStatus,
) -> Vec<DetectionFinding> {
    patterns
        .iter()
        .filter(|pattern| pattern.category == PatternCategory::Orm)
        .filter_map(|pattern| pattern.evaluate(context, sanitization))
        .collect()
}
