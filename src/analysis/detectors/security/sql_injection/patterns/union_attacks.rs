//! Patterns that capture UNION-based SQL injection attempts.

use crate::analysis::detectors::security::sql_injection::types::{
    SqlInjectionPattern, SqlInjectionType,
};
use crate::analysis::detectors::security::types::SecuritySeverity;

pub fn common_union_patterns() -> Vec<SqlInjectionPattern> {
    vec![
        SqlInjectionPattern {
            id: "union_select_case_insensitive",
            regex: r#"(?i)UNION\s+SELECT"#,
            description: "UNION-based SQL injection sequence",
            confidence: 0.7,
            severity: SecuritySeverity::High,
            injection_type: SqlInjectionType::Union,
            context: "UNION SELECT clause",
        },
        SqlInjectionPattern {
            id: "union_all_select",
            regex: r#"(?i)UNION\s+ALL\s+SELECT"#,
            description: "UNION ALL pattern used for data exfiltration",
            confidence: 0.65,
            severity: SecuritySeverity::High,
            injection_type: SqlInjectionType::Union,
            context: "UNION ALL SELECT",
        },
    ]
}
