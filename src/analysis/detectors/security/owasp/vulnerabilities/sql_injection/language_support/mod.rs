pub mod javascript;
pub mod python;
pub mod rust;

use super::patterns;
use super::types::SqlInjectionPattern;
use crate::ast::SourceLanguage;

/// Retrieve patterns tailored for a specific source language.
pub fn patterns_for(language: &SourceLanguage) -> Vec<SqlInjectionPattern> {
    match language {
        SourceLanguage::Rust => rust::language_patterns(),
        SourceLanguage::Python => python::language_patterns(),
        SourceLanguage::JavaScript | SourceLanguage::TypeScript => javascript::language_patterns(),
        _ => patterns::collect_for(language.clone()),
    }
}
