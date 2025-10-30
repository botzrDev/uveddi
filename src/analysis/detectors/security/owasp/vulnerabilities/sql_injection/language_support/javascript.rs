use super::super::patterns;
use super::super::types::{PatternCategory, SqlInjectionPattern, SqlInjectionType};
use crate::analysis::detectors::security::types::SecuritySeverity;
use crate::ast::SourceLanguage;

/// Build JavaScript/TypeScript specific pattern catalogue.
pub fn language_patterns() -> Vec<SqlInjectionPattern> {
    let mut base = patterns::collect_for(SourceLanguage::JavaScript);
    base.push(SqlInjectionPattern::new(
        "js_sequelize_query",
        r#"sequelize\.query\s*\(\s*`.*\$\{[^}]+}.*`"#,
        "Sequelize raw query constructed via template literal",
        0.9,
        SecuritySeverity::Critical,
        SqlInjectionType::Classical,
        "Sequelize raw query",
        PatternCategory::Dynamic,
    ));
    base.push(SqlInjectionPattern::new(
        "js_knex_raw",
        r#"knex\.raw\s*\(\s*`.*\$\{[^}]+}.*`"#,
        "Knex raw query constructed via template literal",
        0.85,
        SecuritySeverity::Critical,
        SqlInjectionType::Classical,
        "Knex raw query",
        PatternCategory::Dynamic,
    ));
    base
}
