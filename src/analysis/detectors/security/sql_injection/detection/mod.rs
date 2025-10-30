//! Detection pipeline coordinating all SQL injection analyzers.

use crate::analysis::detectors::security::sql_injection::config::SqlInjectionConfig;
use crate::analysis::detectors::security::sql_injection::sanitizers::SanitizerEngine;
use crate::analysis::detectors::security::sql_injection::types::{
    DetectionFinding, DetectionMetadata, LanguagePatternSet, SanitizerStatus, SqlInjectionPattern,
};
use crate::ast::SourceLanguage;

pub mod dynamic_query;
pub mod orm_analyzer;
pub mod parameter_analyzer;
pub mod query_analyzer;
pub mod stored_procedure;

use dynamic_query::DynamicQueryAnalyzer;
use orm_analyzer::OrmAnalyzer;
use parameter_analyzer::ParameterAnalyzer;
use query_analyzer::QueryAnalyzer;
use stored_procedure::StoredProcedureAnalyzer;

pub struct DetectionContext<'a> {
    pub line: &'a str,
    pub line_number: usize,
    pub language: SourceLanguage,
}

pub struct DetectionPipeline {
    config: SqlInjectionConfig,
    sanitizer: SanitizerEngine,
    query_analyzer: QueryAnalyzer,
    parameter_analyzer: ParameterAnalyzer,
    dynamic_analyzer: DynamicQueryAnalyzer,
    stored_procedure_analyzer: StoredProcedureAnalyzer,
    orm_analyzer: OrmAnalyzer,
}

impl DetectionPipeline {
    pub fn new(config: SqlInjectionConfig) -> Self {
        Self {
            sanitizer: SanitizerEngine::default(),
            config: config.clone(),
            query_analyzer: QueryAnalyzer::default(),
            parameter_analyzer: ParameterAnalyzer::default(),
            dynamic_analyzer: DynamicQueryAnalyzer::default(),
            stored_procedure_analyzer: StoredProcedureAnalyzer::default(),
            orm_analyzer: OrmAnalyzer::default(),
        }
    }

    pub fn analyze_line(
        &self,
        context: DetectionContext<'_>,
        language_patterns: &LanguagePatternSet,
    ) -> Vec<DetectionFinding> {
        let sanitizer_status = self.sanitizer.evaluate(context.line, context.language);

        let mut findings = Vec::new();

        if self.config.enable_query_analysis {
            findings.extend(self.run_analyzer(
                &self.query_analyzer,
                &context,
                &language_patterns.query_patterns,
                sanitizer_status,
            ));
        }

        if self.config.enable_parameter_analysis {
            findings.extend(self.run_analyzer(
                &self.parameter_analyzer,
                &context,
                &language_patterns.parameter_patterns,
                sanitizer_status,
            ));
        }

        if self.config.enable_dynamic_query_analysis {
            findings.extend(self.run_analyzer(
                &self.dynamic_analyzer,
                &context,
                &language_patterns.dynamic_query_patterns,
                sanitizer_status,
            ));
        }

        if self.config.enable_stored_procedure_analysis {
            findings.extend(self.run_analyzer(
                &self.stored_procedure_analyzer,
                &context,
                &language_patterns.stored_procedure_patterns,
                sanitizer_status,
            ));
        }

        if self.config.enable_orm_analysis {
            findings.extend(self.run_analyzer(
                &self.orm_analyzer,
                &context,
                &language_patterns.orm_patterns,
                sanitizer_status,
            ));
        }

        findings
    }

    fn run_analyzer<A: Analyzer>(
        &self,
        analyzer: &A,
        ctx: &DetectionContext<'_>,
        patterns: &[SqlInjectionPattern],
        sanitizer: SanitizerStatus,
    ) -> Vec<DetectionFinding> {
        analyzer.analyze(ctx, patterns, sanitizer)
    }
}

pub trait Analyzer: Send + Sync {
    fn analyze(
        &self,
        context: &DetectionContext<'_>,
        patterns: &[SqlInjectionPattern],
        sanitizer: SanitizerStatus,
    ) -> Vec<DetectionFinding>;
}

fn build_detection_metadata(context: &DetectionContext<'_>, column: usize) -> DetectionMetadata {
    DetectionMetadata {
        line_number: context.line_number,
        column,
        language: context.language,
        line_excerpt: context.line.trim().to_string(),
    }
}

pub(crate) fn evaluate_patterns<'a, I>(
    context: &DetectionContext<'_>,
    patterns: I,
    sanitizer: SanitizerStatus,
) -> Vec<DetectionFinding>
where
    I: IntoIterator<Item = &'a SqlInjectionPattern>,
{
    let mut findings = Vec::new();

    for pattern in patterns {
        if let Some(column) = pattern.matches(context.line) {
            let metadata = build_detection_metadata(context, column);
            findings.push(DetectionFinding {
                pattern: pattern.clone(),
                metadata,
                sanitizer,
            });
        }
    }

    findings
}
