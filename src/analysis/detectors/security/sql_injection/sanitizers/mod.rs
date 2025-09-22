//! Sanitization heuristics used to contextualize SQL injection findings.

use crate::analysis::detectors::security::sql_injection::types::SanitizerStatus;
use crate::ast::SourceLanguage;

pub mod escape_analysis;
pub mod input_validation;
pub mod output_encoding;
pub mod parameterization;
pub mod whitelist_validation;

use escape_analysis::EscapeAnalyzer;
use input_validation::InputValidationAnalyzer;
use output_encoding::OutputEncodingAnalyzer;
use parameterization::ParameterizationChecker;
use whitelist_validation::WhitelistValidationAnalyzer;

#[derive(Default)]
pub struct SanitizerEngine {
    parameterization: ParameterizationChecker,
    escape_analysis: EscapeAnalyzer,
    whitelist: WhitelistValidationAnalyzer,
    input_validation: InputValidationAnalyzer,
    output_encoding: OutputEncodingAnalyzer,
}

impl SanitizerEngine {
    pub fn evaluate(&self, line: &str, language: SourceLanguage) -> SanitizerStatus {
        if self.parameterization.is_parameterized(line, language) {
            return SanitizerStatus::Sanitized;
        }

        if self.escape_analysis.uses_escape_sequences(line) {
            return SanitizerStatus::Sanitized;
        }

        if self.whitelist.enforces_whitelist(line) || self.input_validation.validates_input(line) {
            return SanitizerStatus::Sanitized;
        }

        if self.output_encoding.encodes_output(line) {
            return SanitizerStatus::Sanitized;
        }

        SanitizerStatus::Unknown
    }
}
