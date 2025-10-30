use crate::analysis::detectors::security::config::FalsePositiveConfig;

#[derive(Clone, Debug)]
pub struct ValidationConfig {
    pub false_positive: FalsePositiveConfig,
    pub enable_input_validation: bool,
    pub enable_output_validation: bool,
    pub enable_sanitization_validation: bool,
    pub enable_language_support: bool,
}

impl ValidationConfig {
    pub fn from_false_positive(config: FalsePositiveConfig) -> Self {
        let enable_input = config.enable_heuristic_filtering || config.enable_contextual_filtering;
        let enable_output = config.enable_statistical_filtering;
        let enable_sanitizers = config.enable_ml_filtering;
        Self {
            enable_input_validation: enable_input,
            enable_output_validation: enable_output,
            enable_sanitization_validation: enable_sanitizers,
            enable_language_support: true,
            false_positive: config,
        }
    }

    pub fn agreement_threshold(&self) -> usize {
        self.false_positive.min_lines_threshold.max(1)
    }

    pub fn min_confidence_threshold(&self) -> f64 {
        self.false_positive.min_confidence_threshold
    }

    pub fn bayesian_enabled(&self) -> bool {
        self.false_positive.enable_bayesian_optimization
    }
}
