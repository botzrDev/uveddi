//! Core detector traits
//!
//! This module defines the fundamental interfaces that all detectors must implement.

use super::types::{DetectorCategory, AnalysisContext};
use crate::analysis::AnalysisError;
use async_trait::async_trait;
use std::any::Any;

/// Core trait that all detectors must implement
#[async_trait]
pub trait Detector: Send + Sync {
    /// The configuration type for this detector
    type Config: DetectorConfig;
    /// The output type produced by this detector
    type Output: DetectorOutput;

    /// Returns the unique name of this detector
    fn name(&self) -> &'static str;

    /// Returns the category this detector belongs to
    fn category(&self) -> DetectorCategory;

    /// Returns the languages this detector supports
    fn supported_languages(&self) -> &[crate::ast::tree_sitter_impl::SourceLanguage];

    /// Performs the detection analysis
    async fn detect(&self, context: &AnalysisContext) -> Result<Self::Output, AnalysisError>;

    /// Returns the detector's configuration
    fn config(&self) -> &Self::Config;

    /// Updates the detector's configuration
    fn update_config(&mut self, config: Self::Config);

    /// Returns whether this detector is enabled
    fn is_enabled(&self) -> bool {
        true
    }

    /// Returns the detector as Any for downcasting
    fn as_any(&self) -> &dyn Any;
}

/// Configuration trait for detector settings
pub trait DetectorConfig: Clone + Send + Sync {
    /// Validates the configuration
    fn validate(&self) -> Result<(), AnalysisError>;

    /// Merges another configuration into this one
    fn merge(&mut self, other: Self);

    /// Returns the default configuration
    fn default() -> Self;

    /// Serializes configuration to JSON
    fn to_json(&self) -> Result<String, AnalysisError>
    where
        Self: serde::Serialize,
    {
        serde_json::to_string_pretty(self).map_err(|e| AnalysisError::ConfigError(e.to_string()))
    }

    /// Deserializes configuration from JSON
    fn from_json(json: &str) -> Result<Self, AnalysisError>
    where
        Self: serde::de::DeserializeOwned,
    {
        serde_json::from_str(json).map_err(|e| AnalysisError::ConfigError(e.to_string()))
    }
}

/// Output trait for detector results
pub trait DetectorOutput: Send + Sync {
    /// Returns the severity level of the findings
    fn severity(&self) -> super::types::Severity;

    /// Returns the list of issues found
    fn issues(&self) -> &[super::types::Issue];

    /// Returns detection metrics if available
    fn metrics(&self) -> Option<super::types::DetectionMetrics>;

    /// Returns whether any issues were found
    fn has_issues(&self) -> bool {
        !self.issues().is_empty()
    }

    /// Returns the total number of issues
    fn issue_count(&self) -> usize {
        self.issues().len()
    }

    /// Combines this output with another
    fn combine(self, other: Self) -> Self
    where
        Self: Sized;
}