use crate::analysis::detectors::security::validation::config::ValidationConfig;
use crate::analysis::detectors::security::validation::types::{
    DynSecurityIssueFilter, ValidationStage,
};

mod command_sanitizer;
mod custom_sanitizer;
mod html_sanitizer;
mod path_sanitizer;
mod sql_sanitizer;

pub use command_sanitizer::CommandSanitizerFilter;
pub use custom_sanitizer::CustomSanitizerFilter;
pub use html_sanitizer::HtmlSanitizerFilter;
pub use path_sanitizer::PathSanitizerFilter;
pub use sql_sanitizer::SqlSanitizerFilter;

pub fn build_stage(config: &ValidationConfig) -> ValidationStage {
    let mut filters: Vec<DynSecurityIssueFilter> = Vec::new();

    if let Some(filter) = html_sanitizer::build_filter(config) {
        filters.push(filter);
    }
    if let Some(filter) = sql_sanitizer::build_filter(config) {
        filters.push(filter);
    }
    if let Some(filter) = path_sanitizer::build_filter(config) {
        filters.push(filter);
    }
    if let Some(filter) = command_sanitizer::build_filter(config) {
        filters.push(filter);
    }
    if let Some(filter) = custom_sanitizer::build_filter(config) {
        filters.push(filter);
    }

    ValidationStage::with_filters("sanitization", filters)
}
