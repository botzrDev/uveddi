//! Modular validation pipeline for security findings.

pub mod config;
pub mod detector;
pub mod input_validation;
pub mod language_support;
pub mod output_validation;
pub mod sanitizers;
pub mod types;

pub use config::ValidationConfig;
pub use detector::{
    BayesianOptimizer, ConfidenceCalculator, FalsePositiveMitigator, ValidationEngine,
};
pub use types::{DynSecurityIssueFilter, ValidationStage};

#[cfg(test)]
mod tests;
