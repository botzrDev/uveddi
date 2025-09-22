use super::super::types::{PatternCategory, SqlInjectionPattern, SqlInjectionType};
use crate::analysis::detectors::security::types::SecuritySeverity;
use crate::ast::SourceLanguage;

/// Patterns targeting error-based SQL injection payloads.
pub fn patterns(language: SourceLanguage) -> Vec<SqlInjectionPattern> {
    match language {
        SourceLanguage::Rust
        | SourceLanguage::Python
        | SourceLanguage::JavaScript
        | SourceLanguage::TypeScript => vec![information_schema_pattern(), version_pattern()],
        _ => Vec::new(),
    }
}

fn information_schema_pattern() -> SqlInjectionPattern {
    SqlInjectionPattern::new(
        "error_based_information_schema",
        r#"(?i)INFORMATION_SCHEMA"#,
        "Information schema reference inside dynamic SQL",
        0.45,
        SecuritySeverity::Medium,
        SqlInjectionType::Classical,
        "Error-based enumeration",
        PatternCategory::Query,
    )
}

fn version_pattern() -> SqlInjectionPattern {
    SqlInjectionPattern::new(
        "error_based_version",
        r#"@@VERSION"#,
        "Database version disclosure in SQL statement",
        0.45,
        SecuritySeverity::Medium,
        SqlInjectionType::Classical,
        "Error-based enumeration",
        PatternCategory::Query,
    )
}
