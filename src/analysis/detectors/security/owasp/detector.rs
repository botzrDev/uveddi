//! Main OWASP detector entry point
//!
//! This module provides the primary interface for OWASP Top 10 security
//! vulnerability detection, integrating all detection modules and analysis
//! capabilities.

use super::categories::CategoryRegistry;
use super::config::OwaspConfig;
use super::types::{OwaspCategory, OwaspCategoryDetector, OwaspVulnerability};
use crate::analysis::AnalysisError;
use crate::ast::ParsedFile;
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Main OWASP Top 10 detector
pub struct OwaspDetector {
    config: OwaspConfig,
    detectors: HashMap<OwaspCategory, Box<dyn OwaspCategoryDetector>>,
}

impl OwaspDetector {
    /// Create a new OWASP detector with default configuration
    pub fn new() -> Result<Self, AnalysisError> {
        Self::with_config(OwaspConfig::default())
    }

    /// Create a new OWASP detector with custom configuration
    pub fn with_config(config: OwaspConfig) -> Result<Self, AnalysisError> {
        info!("Initializing OWASP detector with config");

        let detectors = CategoryRegistry::get_detectors()
            .into_iter()
            .collect::<HashMap<_, _>>();

        Ok(Self { config, detectors })
    }

    /// Analyze a single file for OWASP Top 10 vulnerabilities
    pub async fn analyze_file(
        &self,
        file: &ParsedFile,
    ) -> Result<Vec<OwaspVulnerability>, AnalysisError> {
        info!(
            "Running OWASP Top 10 analysis on: {}",
            file.file_path.display()
        );

        let mut all_vulnerabilities = Vec::new();

        // Run each enabled category detector
        for (category, detector) in &self.detectors {
            if !self.config.is_category_enabled(category) {
                debug!("Skipping disabled category: {}", category.identifier());
                continue;
            }

            debug!("Running {} detector", category.identifier());

            match detector.detect(file).await {
                Ok(mut vulnerabilities) => {
                    // Filter by confidence threshold
                    vulnerabilities
                        .retain(|v| v.confidence_score >= self.config.confidence_threshold);

                    info!(
                        "Found {} vulnerabilities in category {} (after filtering)",
                        vulnerabilities.len(),
                        category.identifier()
                    );

                    all_vulnerabilities.extend(vulnerabilities);
                }
                Err(e) => {
                    warn!("Error running {} detector: {}", category.identifier(), e);
                    // Continue with other detectors
                }
            }
        }

        info!(
            "OWASP analysis completed: {} total vulnerabilities found",
            all_vulnerabilities.len()
        );

        Ok(all_vulnerabilities)
    }

    /// Update detector configuration
    pub fn update_config(&mut self, config: OwaspConfig) -> Result<(), AnalysisError> {
        info!("Updating OWASP detector configuration");
        self.config = config;
        Ok(())
    }

    /// Get current configuration
    pub fn get_config(&self) -> &OwaspConfig {
        &self.config
    }

    /// Get statistics about enabled detectors
    pub fn get_detector_statistics(&self) -> DetectorStatistics {
        let enabled_categories = self
            .config
            .enabled_categories
            .iter()
            .filter(|category| self.detectors.contains_key(category))
            .count();
        let total_categories = self.detectors.len();

        DetectorStatistics {
            total_categories,
            enabled_categories,
            confidence_threshold: self.config.confidence_threshold,
            ai_analysis_enabled: self.config.enable_ai_analysis,
            false_positive_reduction_enabled: self.config.enable_false_positive_reduction,
        }
    }

    /// Check if a specific OWASP category is supported
    pub fn supports_category(&self, category: &OwaspCategory) -> bool {
        self.detectors.contains_key(category)
    }

    /// Get list of all supported OWASP categories
    pub fn get_supported_categories(&self) -> Vec<OwaspCategory> {
        self.detectors.keys().cloned().collect()
    }
}

/// Statistics about the OWASP detector configuration
#[derive(Debug, Clone)]
pub struct DetectorStatistics {
    pub total_categories: usize,
    pub enabled_categories: usize,
    pub confidence_threshold: f64,
    pub ai_analysis_enabled: bool,
    pub false_positive_reduction_enabled: bool,
}

impl Default for OwaspDetector {
    fn default() -> Self {
        Self::new().expect("Failed to create default OWASP detector")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_owasp_detector_creation() {
        let detector = OwaspDetector::new();
        assert!(detector.is_ok());

        let detector = detector.unwrap();
        assert!(detector.detectors.len() >= 1);
    }

    #[tokio::test]
    async fn test_detector_with_custom_config() {
        let config = OwaspConfig {
            confidence_threshold: 0.8,
            enabled_categories: vec![OwaspCategory::Injection, OwaspCategory::BrokenAccessControl],
            ..Default::default()
        };

        let detector = OwaspDetector::with_config(config);
        assert!(detector.is_ok());

        let detector = detector.unwrap();
        assert_eq!(detector.config.confidence_threshold, 0.8);
        assert_eq!(detector.config.enabled_categories.len(), 2);
    }

    #[tokio::test]
    async fn test_detector_statistics() {
        let detector = OwaspDetector::new().unwrap();
        let stats = detector.get_detector_statistics();

        assert_eq!(stats.total_categories, 10);
        assert_eq!(stats.enabled_categories, 10); // Default config enables all
        assert!(stats.confidence_threshold > 0.0);
    }

    #[tokio::test]
    async fn test_category_support() {
        let detector = OwaspDetector::new().unwrap();

        assert!(detector.supports_category(&OwaspCategory::Injection));
        assert!(detector.supports_category(&OwaspCategory::BrokenAccessControl));

        let supported = detector.get_supported_categories();
        assert_eq!(supported.len(), 10);
    }

    #[tokio::test]
    async fn test_file_analysis() {
        let detector = OwaspDetector::new().unwrap();

        let file = ParsedFile {
            file_path: Arc::new(PathBuf::from("test.rs")),
            language: crate::ast::SourceLanguage::Rust,
            content: "pub fn test() {}".to_string(),
            tree: None,
        };

        // Create a temporary test file
        std::fs::write("test.rs", "pub fn test() {}").unwrap();

        let result = detector.analyze_file(&file).await;
        assert!(result.is_ok());

        // Clean up
        std::fs::remove_file("test.rs").unwrap();
    }
}
