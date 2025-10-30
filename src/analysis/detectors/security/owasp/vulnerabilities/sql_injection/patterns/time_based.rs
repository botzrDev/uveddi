use super::super::types::{PatternCategory, SqlInjectionPattern, SqlInjectionType};
use crate::analysis::detectors::security::types::SecuritySeverity;
use crate::ast::SourceLanguage;

/// Patterns that identify time-based SQL injection payloads.
pub fn patterns(language: SourceLanguage) -> Vec<SqlInjectionPattern> {
    match language {
        SourceLanguage::Rust
        | SourceLanguage::Python
        | SourceLanguage::JavaScript
        | SourceLanguage::TypeScript => vec![sleep_pattern(), benchmark_pattern()],
        _ => Vec::new(),
    }
}

fn sleep_pattern() -> SqlInjectionPattern {
    SqlInjectionPattern::new(
        "time_based_sleep",
        r#"(?i)(SLEEP\s*\(|PG_SLEEP\s*\()"#,
        "Time-based SQL injection using sleep functions",
        0.55,
        SecuritySeverity::High,
        SqlInjectionType::TimeDelayed,
        "Time-delayed execution",
        PatternCategory::Query,
    )
}

fn benchmark_pattern() -> SqlInjectionPattern {
    SqlInjectionPattern::new(
        "time_based_benchmark",
        r#"(?i)BENCHMARK\s*\("#,
        "Time-based SQL injection using benchmark",
        0.5,
        SecuritySeverity::High,
        SqlInjectionType::TimeDelayed,
        "Time-delayed execution",
        PatternCategory::Query,
    )
}
