use super::super::types::{PatternCategory, SqlInjectionPattern, SqlInjectionType};
use crate::analysis::detectors::security::types::SecuritySeverity;
use crate::ast::SourceLanguage;

/// Patterns that detect UNION-based SQL injection attempts.
pub fn patterns(language: SourceLanguage) -> Vec<SqlInjectionPattern> {
    match language {
        SourceLanguage::Rust => vec![rust_union_pattern()],
        SourceLanguage::Python => vec![python_union_pattern()],
        SourceLanguage::JavaScript | SourceLanguage::TypeScript => vec![javascript_union_pattern()],
        _ => Vec::new(),
    }
}

fn rust_union_pattern() -> SqlInjectionPattern {
    SqlInjectionPattern::new(
        "rust_union_query",
        r#"(?i)SELECT\s+.*UNION\s+SELECT"#,
        "UNION-based SQL query construction detected",
        0.7,
        SecuritySeverity::High,
        SqlInjectionType::Union,
        "Union-based query",
        PatternCategory::Query,
    )
}

fn python_union_pattern() -> SqlInjectionPattern {
    SqlInjectionPattern::new(
        "python_union_execute",
        r#"(?i)UNION\s+SELECT"#,
        "UNION operator in SQL execution",
        0.75,
        SecuritySeverity::High,
        SqlInjectionType::Union,
        "Union-based query",
        PatternCategory::Query,
    )
}

fn javascript_union_pattern() -> SqlInjectionPattern {
    SqlInjectionPattern::new(
        "js_union_query",
        r#"(?i)UNION\s+SELECT"#,
        "UNION clause detected in SQL statement",
        0.75,
        SecuritySeverity::High,
        SqlInjectionType::Union,
        "Union-based query",
        PatternCategory::Query,
    )
}
