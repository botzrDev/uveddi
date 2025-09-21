//! Detector registry system for centralized management
//!
//! This module provides a registry system for managing different types of detectors,
//! allowing for dynamic registration, discovery, and execution of analysis algorithms.

use crate::analysis::detectors::base::{AnalysisContext, DetectorCategory};
use crate::analysis::AnalysisError;
use crate::ast::tree_sitter_impl::SourceLanguage;
use std::collections::HashMap;

/// Registry for managing detector instances
pub struct DetectorRegistry {
    /// Registry configuration
    config: RegistryConfig,
    /// Registered detector information
    detector_info: HashMap<String, DetectorInfo>,
}

/// Configuration for the detector registry
#[derive(Debug, Clone)]
pub struct RegistryConfig {
    /// Maximum parallel detector executions
    pub max_parallel: usize,
    /// Timeout per detector in seconds
    pub detector_timeout_secs: u64,
    /// Whether to continue on detector failures
    pub continue_on_failure: bool,
    /// Minimum confidence threshold for results
    pub min_confidence: f64,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            max_parallel: 4,
            detector_timeout_secs: 300, // 5 minutes
            continue_on_failure: true,
            min_confidence: 0.5,
        }
    }
}

/// Results from running multiple detectors
#[derive(Debug)]
pub struct RegistryResults {
    /// Results summary
    pub total_detectors: usize,
    /// Successful executions
    pub successful_detectors: usize,
    /// Failed executions
    pub failed_detectors: usize,
    /// Total execution time
    pub total_duration_ms: u64,
}

/// Error information for detector execution
#[derive(Debug)]
pub struct DetectorError {
    pub detector_name: String,
    pub error: AnalysisError,
    pub category: DetectorCategory,
}

impl DetectorRegistry {
    /// Creates a new detector registry
    pub fn new() -> Self {
        Self {
            config: RegistryConfig::default(),
            detector_info: HashMap::new(),
        }
    }

    /// Creates a registry with custom configuration
    pub fn with_config(config: RegistryConfig) -> Self {
        Self {
            config,
            detector_info: HashMap::new(),
        }
    }

    /// Registers detector information in the registry
    pub fn register_info(&mut self, info: DetectorInfo) -> Result<(), AnalysisError> {
        self.detector_info.insert(info.name.clone(), info);
        Ok(())
    }

    /// Lists all registered detectors
    pub fn list_detectors(&self) -> Vec<String> {
        self.detector_info.keys().cloned().collect()
    }

    /// Lists detectors by category
    pub fn list_by_category(&self, category: DetectorCategory) -> Vec<String> {
        self.detector_info
            .values()
            .filter(|info| info.category == category)
            .map(|info| info.name.clone())
            .collect()
    }

    /// Gets detector information
    pub fn get_detector_info(&self, name: &str) -> Option<&DetectorInfo> {
        self.detector_info.get(name)
    }

    /// Lists detectors that support a given language
    pub fn list_for_language(&self, language: SourceLanguage) -> Vec<String> {
        self.detector_info
            .values()
            .filter(|info| info.supported_languages.contains(&language))
            .map(|info| info.name.clone())
            .collect()
    }

    /// Gets the registry configuration
    pub fn config(&self) -> &RegistryConfig {
        &self.config
    }

    /// Updates the registry configuration
    pub fn update_config(&mut self, config: RegistryConfig) {
        self.config = config;
    }

    /// Creates a basic registry result
    pub fn create_results(
        &self,
        total: usize,
        successful: usize,
        failed: usize,
        duration_ms: u64,
    ) -> RegistryResults {
        RegistryResults {
            total_detectors: total,
            successful_detectors: successful,
            failed_detectors: failed,
            total_duration_ms: duration_ms,
        }
    }
}

/// Information about a registered detector
#[derive(Debug, Clone)]
pub struct DetectorInfo {
    pub name: String,
    pub category: DetectorCategory,
    pub supported_languages: Vec<SourceLanguage>,
    pub enabled: bool,
    pub description: String,
}

impl DetectorInfo {
    /// Creates new detector information
    pub fn new(
        name: impl Into<String>,
        category: DetectorCategory,
        supported_languages: Vec<SourceLanguage>,
    ) -> Self {
        Self {
            name: name.into(),
            category,
            supported_languages,
            enabled: true,
            description: String::new(),
        }
    }

    /// Sets the description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Sets the enabled state
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Checks if the detector supports a language
    pub fn supports_language(&self, language: &SourceLanguage) -> bool {
        self.supported_languages.contains(language)
    }
}

impl Default for DetectorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Factory for creating pre-configured detector registries
pub struct DetectorRegistryFactory;

impl DetectorRegistryFactory {
    /// Creates a registry with all built-in detectors
    pub fn create_standard_registry() -> DetectorRegistry {
        let mut registry = DetectorRegistry::new();

        // Register code duplication detector
        let _ = registry.register_info(
            DetectorInfo::new(
                "code_duplication",
                DetectorCategory::AntiPattern,
                vec![
                    SourceLanguage::Rust,
                    SourceLanguage::Python,
                    SourceLanguage::JavaScript,
                    SourceLanguage::TypeScript,
                ],
            )
            .with_description(
                "Detects duplicate code blocks using token-based and AST-based analysis",
            ),
        );

        // Add more detectors as they are refactored...

        registry
    }

    /// Creates a registry for performance-focused analysis
    pub fn create_performance_registry() -> DetectorRegistry {
        let config = RegistryConfig {
            max_parallel: 8,
            detector_timeout_secs: 120,
            continue_on_failure: true,
            min_confidence: 0.7,
        };

        DetectorRegistry::with_config(config)
    }

    /// Creates a registry for thorough analysis
    pub fn create_thorough_registry() -> DetectorRegistry {
        let config = RegistryConfig {
            max_parallel: 2,
            detector_timeout_secs: 600,
            continue_on_failure: false,
            min_confidence: 0.3,
        };

        DetectorRegistry::with_config(config)
    }
}
