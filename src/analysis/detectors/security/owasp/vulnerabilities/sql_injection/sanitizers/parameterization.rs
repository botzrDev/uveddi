use super::super::types::DetectionContext;
use regex::Regex;

/// Detects whether the line hints at parameterized query usage.
pub fn is_parameterized(context: &DetectionContext) -> bool {
    let line = context.line;
    if line.contains("prepare(") || line.contains("bind(") || line.contains("bind_param") {
        return true;
    }

    let placeholder_regex = Regex::new(r#"[?:$][a-zA-Z0-9_]+"#).unwrap();
    let positional_regex = Regex::new(r#"\?(\s|,|\)|\.)"#).unwrap();
    placeholder_regex.is_match(line) || positional_regex.is_match(line)
}
