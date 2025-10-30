use super::super::patterns;
use super::super::types::{PatternCategory, SqlInjectionPattern, SqlInjectionType};
use crate::analysis::detectors::security::types::SecuritySeverity;
use crate::ast::SourceLanguage;

/// Build the Rust-specific pattern catalogue.
pub fn language_patterns() -> Vec<SqlInjectionPattern> {
    let mut base = patterns::collect_for(SourceLanguage::Rust);
    base.push(SqlInjectionPattern::new(
        "rust_sqlx_format_query",
        r#"sqlx::query(?:_as)?\s*\(\s*&?format!\s*\("#,
        "sqlx query constructed via format!",
        0.9,
        SecuritySeverity::Critical,
        SqlInjectionType::Classical,
        "sqlx dynamic query",
        PatternCategory::Dynamic,
    ));
    base.push(SqlInjectionPattern::new(
        "rust_diesel_sql_query",
        r#"diesel::sql_query\s*\(\s*&?format!\s*\("#,
        "Diesel raw SQL constructed via format!",
        0.9,
        SecuritySeverity::Critical,
        SqlInjectionType::Classical,
        "Diesel dynamic query",
        PatternCategory::Dynamic,
    ));
    base
}
