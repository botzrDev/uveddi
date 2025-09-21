use crate::analysis::detectors::security::validation::config::ValidationConfig;
use crate::analysis::detectors::security::validation::types::{
    DynSecurityIssueFilter, ValidationStage,
};

mod api_validation;
mod data_type_validation;
mod file_validation;
mod form_validation;
mod parameter_validation;

pub use api_validation::ApiValidationFilter;
pub use data_type_validation::DataTypeValidationFilter;
pub use file_validation::FileValidationFilter;
pub use form_validation::CommentSuppressionFilter;
pub use parameter_validation::ParameterValidationFilter;

pub fn build_stage(config: &ValidationConfig) -> ValidationStage {
    let mut filters: Vec<DynSecurityIssueFilter> = Vec::new();

    if config.false_positive.enable_heuristic_filtering {
        if let Some(filter) = parameter_validation::build_filter(config) {
            filters.push(filter);
        }
        if let Some(filter) = form_validation::build_filter(config) {
            filters.push(filter);
        }
        if let Some(filter) = data_type_validation::build_filter(config) {
            filters.push(filter);
        }
    }

    if config.false_positive.enable_contextual_filtering {
        if let Some(filter) = file_validation::build_filter(config) {
            filters.push(filter);
        }
        if let Some(filter) = api_validation::build_filter(config) {
            filters.push(filter);
        }
    }

    ValidationStage::with_filters("input-validation", filters)
}
