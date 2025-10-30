use crate::analysis::detectors::security::validation::config::ValidationConfig;
use crate::analysis::detectors::security::validation::types::{
    DynSecurityIssueFilter, ValidationStage,
};

mod javascript;
mod python;
mod rust;

pub use javascript::JavaScriptLanguageFilter;
pub use python::PythonLanguageFilter;
pub use rust::RustLanguageFilter;

pub fn build_stage(config: &ValidationConfig) -> ValidationStage {
    let mut filters: Vec<DynSecurityIssueFilter> = Vec::new();

    if let Some(filter) = rust::build_filter(config) {
        filters.push(filter);
    }
    if let Some(filter) = python::build_filter(config) {
        filters.push(filter);
    }
    if let Some(filter) = javascript::build_filter(config) {
        filters.push(filter);
    }

    ValidationStage::with_filters("language-support", filters)
}
