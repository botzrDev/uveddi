//! Whitelist-based validation heuristics.

#[derive(Default)]
pub struct WhitelistValidationAnalyzer;

impl WhitelistValidationAnalyzer {
    pub fn enforces_whitelist(&self, line: &str) -> bool {
        let lowered = line.to_lowercase();
        lowered.contains("allowlist") || lowered.contains("whitelist") || lowered.contains("match_allowed")
    }
}
