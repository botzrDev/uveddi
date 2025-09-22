use super::super::types::DetectionContext;

/// Detects usage of escaping helpers that may mitigate injection risk.
pub fn uses_escape_sequences(context: &DetectionContext) -> bool {
    let lower = context.line.to_lowercase();
    lower.contains("escape")
        || lower.contains("quote")
        || lower.contains("sanitize_sql")
        || lower.contains("addslashes")
}
