use super::super::types::{PatternCategory, SqlInjectionPattern, SqlInjectionType};
use crate::analysis::detectors::security::types::SecuritySeverity;
use crate::ast::SourceLanguage;

/// General SQL injection patterns that focus on direct query construction.
pub fn patterns(language: SourceLanguage) -> Vec<SqlInjectionPattern> {
    match language {
        SourceLanguage::Rust => rust_patterns(),
        SourceLanguage::Python => python_patterns(),
        SourceLanguage::JavaScript | SourceLanguage::TypeScript => javascript_patterns(),
        _ => Vec::new(),
    }
}

fn rust_patterns() -> Vec<SqlInjectionPattern> {
    vec![
        SqlInjectionPattern::new(
            "rust_dynamic_select",
            r#"(?i)format!\s*\(.*SELECT.*"#,
            "String formatting in SQL SELECT statement",
            0.85,
            SecuritySeverity::Critical,
            SqlInjectionType::Classical,
            "Direct string interpolation",
            PatternCategory::Dynamic,
        ),
        SqlInjectionPattern::new(
            "rust_dynamic_insert",
            r#"(?i)format!\s*\(.*INSERT.*"#,
            "String formatting in SQL INSERT statement",
            0.85,
            SecuritySeverity::Critical,
            SqlInjectionType::Classical,
            "Direct string interpolation",
            PatternCategory::Dynamic,
        ),
        SqlInjectionPattern::new(
            "rust_dynamic_update",
            r#"(?i)format!\s*\(.*UPDATE.*"#,
            "String formatting in SQL UPDATE statement",
            0.85,
            SecuritySeverity::Critical,
            SqlInjectionType::Classical,
            "Direct string interpolation",
            PatternCategory::Dynamic,
        ),
        SqlInjectionPattern::new(
            "rust_dynamic_delete",
            r#"(?i)format!\s*\(.*DELETE.*"#,
            "String formatting in SQL DELETE statement",
            0.85,
            SecuritySeverity::Critical,
            SqlInjectionType::Classical,
            "Direct string interpolation",
            PatternCategory::Dynamic,
        ),
        SqlInjectionPattern::new(
            "rust_concatenated_where",
            r#"(?i)where\s+.*\+\s*"#,
            "String concatenation in WHERE clause",
            0.8,
            SecuritySeverity::High,
            SqlInjectionType::Classical,
            "String concatenation",
            PatternCategory::Parameter,
        ),
        SqlInjectionPattern::new(
            "rust_stored_procedure_exec",
            r#"(?i)format!\s*\(.*(EXEC|CALL)\s+['"]"#,
            "Stored procedure execution via format!",
            0.7,
            SecuritySeverity::High,
            SqlInjectionType::StoredProcedure,
            "Stored procedure invocation",
            PatternCategory::StoredProcedure,
        ),
    ]
}

fn python_patterns() -> Vec<SqlInjectionPattern> {
    vec![
        SqlInjectionPattern::new(
            "python_fstring_execute",
            r#"cursor\.execute\(f"#,
            "F-string formatting in SQL execution",
            0.9,
            SecuritySeverity::Critical,
            SqlInjectionType::Classical,
            "F-string interpolation",
            PatternCategory::Dynamic,
        ),
        SqlInjectionPattern::new(
            "python_percent_execute",
            r#"cursor\.execute\(".*%.*"\s*%"#,
            "Percent formatting in SQL execution",
            0.9,
            SecuritySeverity::Critical,
            SqlInjectionType::Classical,
            "Percent string formatting",
            PatternCategory::Parameter,
        ),
        SqlInjectionPattern::new(
            "python_concatenation_execute",
            r#"cursor\.execute\(".*\+.*"#,
            "String concatenation in SQL execution",
            0.85,
            SecuritySeverity::Critical,
            SqlInjectionType::Classical,
            "String concatenation",
            PatternCategory::Parameter,
        ),
        SqlInjectionPattern::new(
            "python_format_select",
            r#"\.format\(.*SELECT"#,
            "String format in SELECT statement",
            0.8,
            SecuritySeverity::High,
            SqlInjectionType::Classical,
            "Format method",
            PatternCategory::Dynamic,
        ),
        SqlInjectionPattern::new(
            "python_mongo_where",
            r#"find\(\s*\{\s*\$where"#,
            "MongoDB $where clause injection",
            0.75,
            SecuritySeverity::High,
            SqlInjectionType::NoSqlInjection,
            "NoSQL MongoDB",
            PatternCategory::Orm,
        ),
        SqlInjectionPattern::new(
            "python_mongo_aggregate",
            r#"aggregate\(\[\s*\{\s*\$match"#,
            "MongoDB aggregation pipeline injection",
            0.7,
            SecuritySeverity::Medium,
            SqlInjectionType::NoSqlInjection,
            "NoSQL aggregation",
            PatternCategory::Orm,
        ),
        SqlInjectionPattern::new(
            "python_stored_procedure_exec",
            r#"(?i)execute\(.*(EXEC|CALL)\s+['"]"#,
            "Stored procedure execution with dynamic input",
            0.7,
            SecuritySeverity::High,
            SqlInjectionType::StoredProcedure,
            "Stored procedure invocation",
            PatternCategory::StoredProcedure,
        ),
    ]
}

fn javascript_patterns() -> Vec<SqlInjectionPattern> {
    vec![
        SqlInjectionPattern::new(
            "js_template_literal_query",
            r#"query\(`.*\$\{.+}.*`\)"#,
            "Template literal in SQL query",
            0.85,
            SecuritySeverity::Critical,
            SqlInjectionType::Classical,
            "Template literals",
            PatternCategory::Dynamic,
        ),
        SqlInjectionPattern::new(
            "js_concatenated_query",
            r#"query\(".*\+.*"#,
            "String concatenation in SQL query",
            0.85,
            SecuritySeverity::Critical,
            SqlInjectionType::Classical,
            "String concatenation",
            PatternCategory::Parameter,
        ),
        SqlInjectionPattern::new(
            "js_eval_select",
            r#"eval\(.*SELECT"#,
            "Dynamic SQL execution via eval",
            0.95,
            SecuritySeverity::Critical,
            SqlInjectionType::Classical,
            "Dynamic execution",
            PatternCategory::Dynamic,
        ),
        SqlInjectionPattern::new(
            "js_mongo_where",
            r#"find\(\s*\{\s*\$where"#,
            "MongoDB $where clause injection",
            0.75,
            SecuritySeverity::High,
            SqlInjectionType::NoSqlInjection,
            "NoSQL MongoDB",
            PatternCategory::Orm,
        ),
        SqlInjectionPattern::new(
            "js_mongo_aggregate",
            r#"aggregate\(\[\s*\{\s*\$match"#,
            "MongoDB aggregation pipeline injection",
            0.7,
            SecuritySeverity::Medium,
            SqlInjectionType::NoSqlInjection,
            "NoSQL aggregation",
            PatternCategory::Orm,
        ),
        SqlInjectionPattern::new(
            "js_stored_procedure_exec",
            r#"(?i)query\(.*(EXEC|CALL)\s+['"]"#,
            "Stored procedure execution constructed dynamically",
            0.7,
            SecuritySeverity::High,
            SqlInjectionType::StoredProcedure,
            "Stored procedure invocation",
            PatternCategory::StoredProcedure,
        ),
    ]
}
