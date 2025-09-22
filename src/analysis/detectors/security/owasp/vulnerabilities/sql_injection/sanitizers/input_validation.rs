use super::super::types::DetectionContext;

/// Detects input validation routines that operate before query execution.
pub fn performs_input_validation(context: &DetectionContext) -> bool {
    let lower = context.line.to_lowercase();
    lower.contains("validate")
        || lower.contains("is_numeric")
        || lower.contains("parse::<")
        || lower.contains("regex::")
        || lower.contains("sanitize_input")
}
