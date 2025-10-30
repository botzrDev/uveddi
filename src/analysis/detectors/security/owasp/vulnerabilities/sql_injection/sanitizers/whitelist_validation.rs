use super::super::types::DetectionContext;

/// Detects allowlist/whitelist validation that restricts SQL fragments.
pub fn has_whitelist_validation(context: &DetectionContext) -> bool {
    let lower = context.line.to_lowercase();
    lower.contains("whitelist")
        || lower.contains("allowlist")
        || lower.contains("matches!(")
        || lower.contains("is_valid_identifier")
}
