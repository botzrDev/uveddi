//! Core data structures shared across the SQL injection detector modules.

use crate::analysis::detectors::security::types::SecuritySeverity;
use crate::ast::SourceLanguage;
use regex::Regex;

#[derive(Debug, Clone)]
pub struct SqlInjectionPattern {
    pub id: &'static str,
    pub regex: &'static str,
    pub description: &'static str,
    pub confidence: f64,
    pub severity: SecuritySeverity,
    pub injection_type: SqlInjectionType,
    pub context: &'static str,
}

impl SqlInjectionPattern {
    pub fn matches(&self, line: &str) -> Option<usize> {
        Regex::new(self.regex)
            .ok()
            .and_then(|re| re.find(line).map(|m| m.start()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqlInjectionType {
    Classical,
    Blind,
    Union,
    Boolean,
    TimeDelayed,
    NoSqlInjection,
    StoredProcedure,
}

impl SqlInjectionType {
    pub fn description(&self) -> &'static str {
        match self {
            SqlInjectionType::Classical => "Direct SQL injection vulnerability",
            SqlInjectionType::Blind => "Blind SQL injection vulnerability",
            SqlInjectionType::Union => "UNION-based SQL injection",
            SqlInjectionType::Boolean => "Boolean-based blind SQL injection",
            SqlInjectionType::TimeDelayed => "Time-delayed SQL injection",
            SqlInjectionType::NoSqlInjection => "NoSQL injection vulnerability",
            SqlInjectionType::StoredProcedure => "Stored procedure injection",
        }
    }
}

#[derive(Debug, Clone)]
pub struct DetectionMetadata {
    pub line_number: usize,
    pub column: usize,
    pub language: SourceLanguage,
    pub line_excerpt: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SanitizerStatus {
    Unknown,
    Sanitized,
    Unsanitized,
}

#[derive(Debug, Clone)]
pub struct DetectionFinding {
    pub pattern: SqlInjectionPattern,
    pub metadata: DetectionMetadata,
    pub sanitizer: SanitizerStatus,
}

#[derive(Debug, Clone, Default)]
pub struct LanguagePatternSet {
    pub query_patterns: Vec<SqlInjectionPattern>,
    pub parameter_patterns: Vec<SqlInjectionPattern>,
    pub dynamic_query_patterns: Vec<SqlInjectionPattern>,
    pub stored_procedure_patterns: Vec<SqlInjectionPattern>,
    pub orm_patterns: Vec<SqlInjectionPattern>,
}

impl LanguagePatternSet {
    pub fn all_patterns(&self) -> impl Iterator<Item = &SqlInjectionPattern> {
        self.query_patterns
            .iter()
            .chain(self.parameter_patterns.iter())
            .chain(self.dynamic_query_patterns.iter())
            .chain(self.stored_procedure_patterns.iter())
            .chain(self.orm_patterns.iter())
    }

    pub fn is_empty(&self) -> bool {
        self.query_patterns.is_empty()
            && self.parameter_patterns.is_empty()
            && self.dynamic_query_patterns.is_empty()
            && self.stored_procedure_patterns.is_empty()
            && self.orm_patterns.is_empty()
    }
}
