//! Pattern registry for different SQL injection attack classes.

pub mod blind_injection;
pub mod error_based;
pub mod injection_patterns;
pub mod time_based;
pub mod union_attacks;

use crate::analysis::detectors::security::sql_injection::types::SqlInjectionPattern;

pub fn merge_patterns(
    mut base: Vec<SqlInjectionPattern>,
    mut extra: Vec<SqlInjectionPattern>,
) -> Vec<SqlInjectionPattern> {
    base.append(&mut extra);
    base
}

pub fn stable_dedup(patterns: &mut Vec<SqlInjectionPattern>) {
    patterns.sort_by_key(|p| p.id);
    patterns.dedup_by(|a, b| a.id == b.id);
}
