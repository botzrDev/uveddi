use crate::analysis::detectors::security::types::SecuritySeverity;
use crate::ast::SourceLanguage;
use regex::Regex;
use std::path::Path;

/// Categorises a SQL injection pattern so specialised analyzers can process it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PatternCategory {
    Query,
    Parameter,
    Dynamic,
    StoredProcedure,
    Orm,
}

/// SQL injection classification used for metadata and remediation guidance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

/// Represents a single SQL injection detection heuristic.
#[derive(Debug, Clone)]
pub struct SqlInjectionPattern {
    pub id: &'static str,
    pub pattern: String,
    pub description: String,
    pub confidence: f64,
    pub severity: SecuritySeverity,
    pub injection_type: SqlInjectionType,
    pub context: String,
    pub category: PatternCategory,
    pub languages: Vec<SourceLanguage>,
}

impl SqlInjectionPattern {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: &'static str,
        pattern: &str,
        description: &str,
        confidence: f64,
        severity: SecuritySeverity,
        injection_type: SqlInjectionType,
        context: &str,
        category: PatternCategory,
    ) -> Self {
        Self {
            id,
            pattern: pattern.to_string(),
            description: description.to_string(),
            confidence,
            severity,
            injection_type,
            context: context.to_string(),
            category,
            languages: Vec::new(),
        }
    }

    pub fn for_languages(mut self, languages: Vec<SourceLanguage>) -> Self {
        self.languages = languages;
        self
    }

    pub fn supports_language(&self, language: &SourceLanguage) -> bool {
        self.languages.is_empty() || self.languages.iter().any(|lang| lang == language)
    }

    fn regex(&self) -> Option<Regex> {
        Regex::new(&self.pattern).ok()
    }

    pub fn evaluate(
        &self,
        context: &DetectionContext,
        sanitization: &SanitizationStatus,
    ) -> Option<DetectionFinding> {
        if !self.supports_language(&context.language) {
            return None;
        }

        let regex = self.regex()?;
        let captures = regex.find(context.line)?;
        Some(DetectionFinding::new(
            self,
            context.line_number,
            Some(captures.start()),
            captures.end() - captures.start(),
            sanitization,
        ))
    }
}

/// Provides the necessary context for analysing a single source line.
#[derive(Debug, Clone)]
pub struct DetectionContext<'a> {
    pub file_path: &'a Path,
    pub language: SourceLanguage,
    pub line: &'a str,
    pub line_number: usize,
}

impl<'a> DetectionContext<'a> {
    pub fn new(
        file_path: &'a Path,
        language: SourceLanguage,
        line: &'a str,
        line_number: usize,
    ) -> Self {
        Self {
            file_path,
            language,
            line,
            line_number,
        }
    }
}

/// Tracks sanitization heuristics detected on the analysed source line.
#[derive(Debug, Clone, Copy, Default)]
pub struct SanitizationStatus {
    pub parameterized: bool,
    pub escaped: bool,
    pub whitelist_validated: bool,
    pub input_validated: bool,
    pub output_encoded: bool,
}

impl SanitizationStatus {
    pub fn any(&self) -> bool {
        self.parameterized
            || self.escaped
            || self.whitelist_validated
            || self.input_validated
            || self.output_encoded
    }

    pub fn to_metadata_pairs(&self) -> [(&'static str, bool); 5] {
        [
            ("parameterized", self.parameterized),
            ("escaped", self.escaped),
            ("whitelist_validated", self.whitelist_validated),
            ("input_validated", self.input_validated),
            ("output_encoded", self.output_encoded),
        ]
    }
}

/// Result returned by the detection analyzers for a single line.
#[derive(Debug, Clone)]
pub struct DetectionFinding {
    pub pattern: SqlInjectionPattern,
    pub line_number: usize,
    pub column: Option<usize>,
    pub match_length: usize,
    pub sanitization: SanitizationStatus,
}

impl DetectionFinding {
    pub fn new(
        pattern: &SqlInjectionPattern,
        line_number: usize,
        column: Option<usize>,
        match_length: usize,
        sanitization: &SanitizationStatus,
    ) -> Self {
        Self {
            pattern: pattern.clone(),
            line_number,
            column,
            match_length,
            sanitization: *sanitization,
        }
    }
}
