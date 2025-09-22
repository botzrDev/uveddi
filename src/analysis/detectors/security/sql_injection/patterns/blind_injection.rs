//! Patterns associated with boolean/blind SQL injection attempts.

use crate::analysis::detectors::security::sql_injection::types::{
    SqlInjectionPattern, SqlInjectionType,
};
use crate::analysis::detectors::security::types::SecuritySeverity;

pub fn boolean_based_patterns() -> Vec<SqlInjectionPattern> {
    vec![
        SqlInjectionPattern {
            id: "boolean_or_1_eq_1",
            regex: r#"(?i)'?\s*OR\s+'?1'?\s*=\s*'?1"#,
            description: "Classic boolean-based SQL injection",
            confidence: 0.6,
            severity: SecuritySeverity::High,
            injection_type: SqlInjectionType::Boolean,
            context: "Boolean tautology",
        },
        SqlInjectionPattern {
            id: "boolean_or_true",
            regex: r#"(?i)OR\s+TRUE"#,
            description: "Boolean TRUE appended to query",
            confidence: 0.55,
            severity: SecuritySeverity::Medium,
            injection_type: SqlInjectionType::Boolean,
            context: "Boolean clause",
        },
    ]
}
