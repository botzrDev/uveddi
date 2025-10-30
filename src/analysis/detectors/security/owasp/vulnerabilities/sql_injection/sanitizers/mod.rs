pub mod escape_analysis;
pub mod input_validation;
pub mod output_encoding;
pub mod parameterization;
pub mod whitelist_validation;

use super::types::{DetectionContext, SanitizationStatus};

/// Evaluate sanitization signals present on the analysed line.
pub fn evaluate(context: &DetectionContext) -> SanitizationStatus {
    SanitizationStatus {
        parameterized: parameterization::is_parameterized(context),
        escaped: escape_analysis::uses_escape_sequences(context),
        whitelist_validated: whitelist_validation::has_whitelist_validation(context),
        input_validated: input_validation::performs_input_validation(context),
        output_encoded: output_encoding::performs_output_encoding(context),
    }
}
