//! Base configuration types

use super::traits::DetectorConfig;
use crate::analysis::AnalysisError;
use serde::{Deserialize, Serialize};

/// Base configuration that can be extended by specific detectors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseConfig {
    /// Whether the detector is enabled
    pub enabled: bool,
    /// Minimum severity level to report
    pub min_severity: super::types::Severity,
    /// Maximum number of issues to report
    pub max_issues: Option<usize>,
    /// Whether to include low-confidence results
    pub include_low_confidence: bool,
    /// Custom thresholds for the detector
    pub thresholds: std::collections::HashMap<String, serde_json::Value>,
}

impl BaseConfig {
    /// Creates a new base configuration with defaults
    pub fn new() -> Self {
        Self {
            enabled: true,
            min_severity: super::types::Severity::Low,
            max_issues: None,
            include_low_confidence: false,
            thresholds: std::collections::HashMap::new(),
        }
    }

    /// Sets a threshold value
    pub fn with_threshold(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.thresholds.insert(key.into(), value);
        self
    }

    /// Gets a threshold value
    pub fn get_threshold<T>(&self, key: &str) -> Option<T>
    where
        T: serde::de::DeserializeOwned,
    {
        self.thresholds
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// Sets the minimum severity
    pub fn with_min_severity(mut self, severity: super::types::Severity) -> Self {
        self.min_severity = severity;
        self
    }

    /// Sets the maximum issues limit
    pub fn with_max_issues(mut self, max_issues: usize) -> Self {
        self.max_issues = Some(max_issues);
        self
    }

    /// Enables or disables the detector
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

impl Default for BaseConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl DetectorConfig for BaseConfig {
    fn validate(&self) -> Result<(), AnalysisError> {
        if let Some(max_issues) = self.max_issues {
            if max_issues == 0 {
                return Err(AnalysisError::ConfigError(
                    "max_issues must be greater than 0".to_string(),
                ));
            }
        }
        Ok(())
    }

    fn merge(&mut self, other: Self) {
        // Keep enabled state from other if it's more restrictive
        if !other.enabled {
            self.enabled = false;
        }

        // Use the higher severity threshold
        if other.min_severity > self.min_severity {
            self.min_severity = other.min_severity;
        }

        // Use the lower max_issues limit
        match (self.max_issues, other.max_issues) {
            (Some(current), Some(other_max)) => {
                self.max_issues = Some(current.min(other_max));
            }
            (None, Some(other_max)) => {
                self.max_issues = Some(other_max);
            }
            _ => {} // Keep current value
        }

        // Merge thresholds, preferring values from other
        for (key, value) in other.thresholds {
            self.thresholds.insert(key, value);
        }
    }

    fn default() -> Self {
        Self::default()
    }
}