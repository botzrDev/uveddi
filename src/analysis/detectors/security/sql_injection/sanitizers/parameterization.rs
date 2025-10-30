//! Parameterization heuristics for SQL queries.

use crate::ast::SourceLanguage;

#[derive(Default)]
pub struct ParameterizationChecker;

impl ParameterizationChecker {
    pub fn is_parameterized(&self, line: &str, language: SourceLanguage) -> bool {
        let lowered = line.to_lowercase();

        if lowered.contains("prepare(")
            || lowered.contains("bind_param")
            || lowered.contains("bind(")
        {
            return true;
        }

        match language {
            SourceLanguage::Python => {
                lowered.contains("execute(") && lowered.contains(", (") && lowered.contains("%s")
            }
            SourceLanguage::Rust => lowered.contains("query!(") || lowered.contains("execute!("),
            SourceLanguage::JavaScript | SourceLanguage::TypeScript => lowered.contains("?"),
        }
    }
}
