pub mod blind_injection;
pub mod error_based;
pub mod injection_patterns;
pub mod time_based;
pub mod union_attacks;

use super::types::SqlInjectionPattern;
use crate::ast::SourceLanguage;

/// Collect all pattern definitions relevant for the given language.
pub fn collect_for(language: SourceLanguage) -> Vec<SqlInjectionPattern> {
    let mut patterns = Vec::new();
    patterns.extend(injection_patterns::patterns(language));
    patterns.extend(union_attacks::patterns(language));
    patterns.extend(blind_injection::patterns(language));
    patterns.extend(time_based::patterns(language));
    patterns.extend(error_based::patterns(language));
    patterns
}
