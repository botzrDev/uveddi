//! Core SQL injection pattern definitions shared across languages.

use crate::analysis::detectors::security::sql_injection::types::{
    SqlInjectionPattern, SqlInjectionType,
};
use crate::analysis::detectors::security::types::SecuritySeverity;

pub fn rust_query_patterns() -> Vec<SqlInjectionPattern> {
    vec![
        SqlInjectionPattern {
            id: "rust_format_select",
            regex: r#"format!.*SELECT.*"#,
            description: "String formatting in SQL SELECT statement",
            confidence: 0.85,
            severity: SecuritySeverity::Critical,
            injection_type: SqlInjectionType::Classical,
            context: "Direct string interpolation",
        },
        SqlInjectionPattern {
            id: "rust_format_insert",
            regex: r#"format!.*INSERT.*"#,
            description: "String formatting in SQL INSERT statement",
            confidence: 0.85,
            severity: SecuritySeverity::Critical,
            injection_type: SqlInjectionType::Classical,
            context: "Direct string interpolation",
        },
        SqlInjectionPattern {
            id: "rust_format_update",
            regex: r#"format!.*UPDATE.*"#,
            description: "String formatting in SQL UPDATE statement",
            confidence: 0.85,
            severity: SecuritySeverity::Critical,
            injection_type: SqlInjectionType::Classical,
            context: "Direct string interpolation",
        },
        SqlInjectionPattern {
            id: "rust_format_delete",
            regex: r#"format!.*DELETE.*"#,
            description: "String formatting in SQL DELETE statement",
            confidence: 0.85,
            severity: SecuritySeverity::Critical,
            injection_type: SqlInjectionType::Classical,
            context: "Direct string interpolation",
        },
        SqlInjectionPattern {
            id: "rust_concat_where",
            regex: r#"query.*WHERE.*\+.*"#,
            description: "String concatenation in WHERE clause",
            confidence: 0.8,
            severity: SecuritySeverity::High,
            injection_type: SqlInjectionType::Classical,
            context: "String concatenation",
        },
    ]
}

pub fn python_query_patterns() -> Vec<SqlInjectionPattern> {
    vec![
        SqlInjectionPattern {
            id: "py_fstring_execute",
            regex: r#"cursor\.execute\(f".*""#,
            description: "F-string formatting in SQL execution",
            confidence: 0.9,
            severity: SecuritySeverity::Critical,
            injection_type: SqlInjectionType::Classical,
            context: "F-string interpolation",
        },
        SqlInjectionPattern {
            id: "py_percent_execute",
            regex: r#"cursor\.execute\(".*%.*".*%"#,
            description: "String formatting in SQL execution",
            confidence: 0.9,
            severity: SecuritySeverity::Critical,
            injection_type: SqlInjectionType::Classical,
            context: "Percent string formatting",
        },
        SqlInjectionPattern {
            id: "py_concat_execute",
            regex: r#"cursor\.execute\(".*\+.*"#,
            description: "String concatenation in SQL execution",
            confidence: 0.85,
            severity: SecuritySeverity::Critical,
            injection_type: SqlInjectionType::Classical,
            context: "String concatenation",
        },
        SqlInjectionPattern {
            id: "py_dot_format",
            regex: r#"\\.format\(.*SELECT.*"#,
            description: "String format in SELECT statement",
            confidence: 0.8,
            severity: SecuritySeverity::High,
            injection_type: SqlInjectionType::Classical,
            context: "Format method",
        },
    ]
}

pub fn python_nosql_patterns() -> Vec<SqlInjectionPattern> {
    vec![SqlInjectionPattern {
        id: "py_mongo_where",
        regex: r#"find\(.*\$where.*"#,
        description: "MongoDB $where clause injection",
        confidence: 0.75,
        severity: SecuritySeverity::High,
        injection_type: SqlInjectionType::NoSqlInjection,
        context: "NoSQL MongoDB",
    }]
}

pub fn javascript_query_patterns() -> Vec<SqlInjectionPattern> {
    vec![
        SqlInjectionPattern {
            id: "js_template_literal",
            regex: r#"query\(`.*\$\{.*\}.*`\)"#,
            description: "Template literal in SQL query",
            confidence: 0.85,
            severity: SecuritySeverity::Critical,
            injection_type: SqlInjectionType::Classical,
            context: "Template literals",
        },
        SqlInjectionPattern {
            id: "js_concat_query",
            regex: r#"query\(\".*\+.*"#,
            description: "String concatenation in SQL query",
            confidence: 0.85,
            severity: SecuritySeverity::Critical,
            injection_type: SqlInjectionType::Classical,
            context: "String concatenation",
        },
        SqlInjectionPattern {
            id: "js_eval_sql",
            regex: r#"eval\(.*SELECT.*"#,
            description: "Dynamic SQL execution via eval",
            confidence: 0.95,
            severity: SecuritySeverity::Critical,
            injection_type: SqlInjectionType::Classical,
            context: "Dynamic execution",
        },
    ]
}

pub fn javascript_nosql_patterns() -> Vec<SqlInjectionPattern> {
    vec![
        SqlInjectionPattern {
            id: "js_mongo_where",
            regex: r#"find\(.*\$where.*"#,
            description: "MongoDB $where clause injection",
            confidence: 0.75,
            severity: SecuritySeverity::High,
            injection_type: SqlInjectionType::NoSqlInjection,
            context: "NoSQL MongoDB",
        },
        SqlInjectionPattern {
            id: "js_mongo_aggregate",
            regex: r#"aggregate\(\[.*\$.*\]\)"#,
            description: "MongoDB aggregation pipeline injection",
            confidence: 0.7,
            severity: SecuritySeverity::Medium,
            injection_type: SqlInjectionType::NoSqlInjection,
            context: "NoSQL aggregation",
        },
    ]
}
