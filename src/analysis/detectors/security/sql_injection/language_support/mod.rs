//! Language-specific helpers for SQL injection detection.

use crate::analysis::detectors::security::sql_injection::types::LanguagePatternSet;
use crate::ast::SourceLanguage;

pub mod javascript;
pub mod python;
pub mod rust;

pub fn patterns_for(language: SourceLanguage) -> LanguagePatternSet {
    match language {
        SourceLanguage::Rust => rust::patterns(),
        SourceLanguage::Python => python::patterns(),
        SourceLanguage::JavaScript | SourceLanguage::TypeScript => javascript::patterns(),
    }
}
