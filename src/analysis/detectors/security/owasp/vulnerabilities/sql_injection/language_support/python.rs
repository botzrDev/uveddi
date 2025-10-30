use super::super::patterns;
use super::super::types::{PatternCategory, SqlInjectionPattern, SqlInjectionType};
use crate::analysis::detectors::security::types::SecuritySeverity;
use crate::ast::SourceLanguage;

/// Build the Python-specific pattern catalogue.
pub fn language_patterns() -> Vec<SqlInjectionPattern> {
    let mut base = patterns::collect_for(SourceLanguage::Python);
    base.push(SqlInjectionPattern::new(
        "python_sqlalchemy_text",
        r#"sqlalchemy\.text\(f?"#,
        "SQLAlchemy text query constructed dynamically",
        0.85,
        SecuritySeverity::Critical,
        SqlInjectionType::Classical,
        "SQLAlchemy dynamic query",
        PatternCategory::Dynamic,
    ));
    base.push(SqlInjectionPattern::new(
        "python_django_raw",
        r#"\.raw\(f?"#,
        "Django raw SQL constructed dynamically",
        0.8,
        SecuritySeverity::High,
        SqlInjectionType::Classical,
        "Django ORM raw SQL",
        PatternCategory::Dynamic,
    ));
    base
}
