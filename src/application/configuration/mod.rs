//! Configuration management for analysis operations
//!
//! This module provides structured configuration types for different aspects
//! of the analysis process, including target analysis, output generation,
//! and AI integration settings.

pub mod analysis_config;
pub mod ai_config;
pub mod output_config;
pub mod validation;

pub use analysis_config::AnalysisConfig;
pub use ai_config::AiConfig;
pub use output_config::OutputConfig;
pub use validation::ConfigValidator;

use crate::error::UveddiError;
use std::path::PathBuf;

/// Combined configuration for analysis operations
#[derive(Debug, Clone)]
pub struct ApplicationConfig {
    /// Analysis target and execution settings
    pub analysis: AnalysisConfig,
    /// Output format and file settings
    pub output: OutputConfig,
    /// AI integration settings
    pub ai: AiConfig,
}

impl ApplicationConfig {
    /// Create a new application configuration
    pub fn new(
        target_path: PathBuf,
        output_format: String,
        output_file: Option<PathBuf>,
    ) -> Self {
        Self {
            analysis: AnalysisConfig::new(target_path),
            output: OutputConfig::new(output_format, output_file),
            ai: AiConfig::default(),
        }
    }

    /// Validate the complete configuration
    pub fn validate(&self) -> Result<(), UveddiError> {
        self.analysis.validate()?;
        self.output.validate()?;
        self.ai.validate()?;
        Ok(())
    }

    /// Merge with another configuration, taking values from other where present
    pub fn merge_with(mut self, other: ApplicationConfig) -> Self {
        self.analysis = self.analysis.merge_with(other.analysis);
        self.output = self.output.merge_with(other.output);
        self.ai = self.ai.merge_with(other.ai);
        self
    }
}