use super::super::types::{PatternCategory, SqlInjectionPattern, SqlInjectionType};
use crate::analysis::detectors::security::types::SecuritySeverity;
use crate::ast::SourceLanguage;

/// Patterns related to blind/boolean SQL injection techniques.
pub fn patterns(language: SourceLanguage) -> Vec<SqlInjectionPattern> {
    match language {
        SourceLanguage::Rust
        | SourceLanguage::Python
        | SourceLanguage::JavaScript
        | SourceLanguage::TypeScript => vec![boolean_pattern(), tautology_pattern()],
        _ => Vec::new(),
    }
}

fn boolean_pattern() -> SqlInjectionPattern {
    SqlInjectionPattern::new(
        "blind_boolean_true",
        r#"(?i)'?\s*or\s*'?1'?\s*=\s*'?1"#,
        "Boolean-based SQL injection tautology",
        0.6,
        SecuritySeverity::High,
        SqlInjectionType::Boolean,
        "Boolean-based toggle",
        PatternCategory::Query,
    )
}

fn tautology_pattern() -> SqlInjectionPattern {
    SqlInjectionPattern::new(
        "blind_tautology",
        r#"(?i)AND\s+\d\s*=\s*\d"#,
        "Boolean tautology inside SQL predicate",
        0.55,
        SecuritySeverity::High,
        SqlInjectionType::Blind,
        "Blind SQL injection",
        PatternCategory::Query,
    )
}
