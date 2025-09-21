use crate::analysis::detectors::security::validation::config::ValidationConfig;
use crate::analysis::detectors::security::validation::types::{
    DynSecurityIssueFilter, ValidationStage,
};

mod content_validation;
mod encoding_validation;
mod header_validation;
mod response_validation;

pub use content_validation::ContentFrequencyFilter;
pub use encoding_validation::EncodingValidationFilter;
pub use header_validation::HeaderValidationFilter;
pub use response_validation::ResponseValidationFilter;

pub fn build_stage(config: &ValidationConfig) -> ValidationStage {
    let mut filters: Vec<DynSecurityIssueFilter> = Vec::new();

    if let Some(filter) = response_validation::build_filter(config) {
        filters.push(filter);
    }

    if let Some(filter) = encoding_validation::build_filter(config) {
        filters.push(filter);
    }

    if config.false_positive.enable_statistical_filtering {
        if let Some(filter) = content_validation::build_filter(config) {
            filters.push(filter);
        }
    }

    if let Some(filter) = header_validation::build_filter(config) {
        filters.push(filter);
    }

    ValidationStage::with_filters("output-validation", filters)
}
