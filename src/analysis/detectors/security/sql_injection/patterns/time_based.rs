//! Patterns for time-based SQL injection detection.

use crate::analysis::detectors::security::sql_injection::types::{
    SqlInjectionPattern, SqlInjectionType,
};
use crate::analysis::detectors::security::types::SecuritySeverity;

pub fn time_delay_patterns() -> Vec<SqlInjectionPattern> {
    vec![
        SqlInjectionPattern {
            id: "sleep_function",
            regex: r#"(?i)SLEEP\s*\("#,
            description: "Time-based injection using SLEEP function",
            confidence: 0.5,
            severity: SecuritySeverity::High,
            injection_type: SqlInjectionType::TimeDelayed,
            context: "Time delay function",
        },
        SqlInjectionPattern {
            id: "benchmark_function",
            regex: r#"(?i)BENCHMARK\s*\("#,
            description: "Time-based injection using BENCHMARK function",
            confidence: 0.5,
            severity: SecuritySeverity::High,
            injection_type: SqlInjectionType::TimeDelayed,
            context: "Benchmark execution",
        },
    ]
}
