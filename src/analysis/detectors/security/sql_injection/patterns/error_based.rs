//! Patterns that surface error-based SQL injection indicators.

use crate::analysis::detectors::security::sql_injection::types::{
    SqlInjectionPattern, SqlInjectionType,
};
use crate::analysis::detectors::security::types::SecuritySeverity;

pub fn error_probe_patterns() -> Vec<SqlInjectionPattern> {
    vec![
        SqlInjectionPattern {
            id: "error_union_select",
            regex: r#"(?i)UNION\s+SELECT\s+NULL"#,
            description: "UNION SELECT NULL probe",
            confidence: 0.55,
            severity: SecuritySeverity::Medium,
            injection_type: SqlInjectionType::Union,
            context: "Error probing",
        },
        SqlInjectionPattern {
            id: "xp_cmdshell_usage",
            regex: r#"(?i)xp_cmdshell"#,
            description: "MSSQL xp_cmdshell invocation",
            confidence: 0.6,
            severity: SecuritySeverity::Critical,
            injection_type: SqlInjectionType::StoredProcedure,
            context: "Extended stored procedure",
        },
    ]
}
