use super::super::types::DetectionContext;

/// Detects output encoding or canonicalisation prior to query execution.
pub fn performs_output_encoding(context: &DetectionContext) -> bool {
    let lower = context.line.to_lowercase();
    lower.contains("encode")
        || lower.contains("to_sql_literal")
        || lower.contains("escape_str")
        || lower.contains("canonicalize")
}
