use super::super::types::{
    DetectionContext, DetectionFinding, PatternCategory, SanitizationStatus, SqlInjectionPattern,
};

/// Analyse dynamic SQL construction (string formatting, interpolation, eval).
pub fn analyze(
    context: &DetectionContext,
    patterns: &[SqlInjectionPattern],
    sanitization: &SanitizationStatus,
) -> Vec<DetectionFinding> {
    patterns
        .iter()
        .filter(|pattern| pattern.category == PatternCategory::Dynamic)
        .filter_map(|pattern| pattern.evaluate(context, sanitization))
        .collect()
}
