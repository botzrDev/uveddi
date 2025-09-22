//! Output encoding analysis to detect context-aware escaping.

#[derive(Default)]
pub struct OutputEncodingAnalyzer;

impl OutputEncodingAnalyzer {
    pub fn encodes_output(&self, line: &str) -> bool {
        let lowered = line.to_lowercase();
        lowered.contains("encode_html")
            || lowered.contains("escape_html")
            || lowered.contains("html_escape")
    }
}
