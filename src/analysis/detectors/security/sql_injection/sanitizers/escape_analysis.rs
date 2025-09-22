//! Escape sequence heuristics providing context for detections.

#[derive(Default)]
pub struct EscapeAnalyzer;

impl EscapeAnalyzer {
    pub fn uses_escape_sequences(&self, line: &str) -> bool {
        let lowered = line.to_lowercase();
        lowered.contains("escape_string")
            || lowered.contains("quote_identifier")
            || lowered.contains("mysql_real_escape_string")
    }
}
