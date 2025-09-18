//! God Object Anti-pattern Detector
//!
//! ## Overview
//! Detects "God Objects" - classes or structs that have accumulated too many responsibilities,
//! violating the Single Responsibility Principle. Also known as "Blob" or "Large Class"
//! anti-pattern, these objects become difficult to maintain, test, and understand.
//!
//! This module has been refactored from a single 1,941-line file into a modular structure
//! for better maintainability and separation of concerns.
//!
//! ## Module Structure
//! - `config`: Configuration and thresholds
//! - `detector`: Core trait and pattern types
//! - `metrics`: Complexity calculations
//! - `languages/`: Language-specific analyzers
//! - `reports/`: Report formatting utilities
//!
//! ## Usage
//!
//! ```rust
//! use uveddi::analysis::detectors::anti_patterns::god_object::{GodObjectDetector, GodObjectConfig};
//!
//! // Use default configuration
//! let detector = GodObjectDetector::default();
//!
//! // Create with custom configuration
//! let config = GodObjectConfig::with_uniform_thresholds(15, 10);
//! let detector = GodObjectDetector::with_config(config);
//! ```

use async_trait::async_trait;
use crate::analysis::{AnalysisDetector, AnalysisError};
use crate::ast::tree_sitter_impl::{ParsedFile, SourceLanguage};
use crate::database::models::{AntiPatternType, ArchitecturalIssue};
use tracing::{debug, info};

// Public module exports
pub mod config;
pub mod detector;
pub mod metrics;
pub mod languages;
pub mod reports;

// Tests module
#[cfg(test)]
mod tests;

// Re-export main types for backwards compatibility
pub use config::GodObjectConfig;
pub use detector::{ComplexityMetrics, DetectedPattern};
pub use reports::GodObjectReportFormatter;

// Re-export language analyzers
pub use languages::{
    RustGodObjectAnalyzer,
    PythonGodObjectAnalyzer,
    JavaScriptGodObjectAnalyzer,
    TypeScriptGodObjectAnalyzer,
};

/// Enhanced God Object detector with modular architecture
///
/// This detector identifies classes or structs that have grown too large while
/// reducing false positives through sophisticated pattern recognition, framework awareness,
/// and cohesion analysis.
///
/// ## Enhanced Features
///
/// - **Framework Detection**: Excludes known framework base classes and patterns
/// - **Pattern Recognition**: Detects Builder, Factory, DTO, and other design patterns
/// - **Generated Code Exclusion**: Skips auto-generated classes and structs
/// - **Behavioral Analysis**: Distinguishes data structures from behavior classes
/// - **Cohesion Analysis**: Uses LCOM4 metrics to measure class cohesion
/// - **Language-Specific Tuning**: Optimizes thresholds per language ecosystem
pub struct GodObjectDetector {
    /// Configuration for the enhanced detection logic
    config: GodObjectConfig,
}

impl GodObjectDetector {
    /// Creates a new `GodObjectDetector` with custom configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration for enhanced God Object detection
    ///
    /// # Examples
    ///
    /// ```rust
    /// use uveddi::analysis::detectors::anti_patterns::god_object::{GodObjectDetector, GodObjectConfig};
    ///
    /// let config = GodObjectConfig::default();
    /// let detector = GodObjectDetector::with_config(config);
    /// ```
    pub fn with_config(config: GodObjectConfig) -> Self {
        Self { config }
    }

    /// Creates a new `GodObjectDetector` with specified thresholds (legacy method).
    ///
    /// # Arguments
    ///
    /// * `method_threshold` - The maximum number of methods allowed.
    /// * `field_threshold` - The maximum number of fields allowed.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use uveddi::analysis::detectors::anti_patterns::god_object::GodObjectDetector;
    ///
    /// // A detector with strict thresholds for a project with small, focused classes.
    /// let strict_detector = GodObjectDetector::new(5, 3);
    ///
    /// // A detector with more lenient thresholds for a legacy or complex system.
    /// let lenient_detector = GodObjectDetector::new(20, 15);
    /// ```
    pub fn new(method_threshold: usize, field_threshold: usize) -> Self {
        let config = GodObjectConfig::with_uniform_thresholds(method_threshold, field_threshold);
        Self { config }
    }

    /// Checks if a file should be excluded based on generated code patterns.
    fn is_generated_file(&self, file_path: &str, source: &str) -> Option<DetectedPattern> {
        // Check file name patterns
        for pattern in &self.config.generated_file_patterns {
            if self.matches_pattern(file_path, pattern) {
                return Some(DetectedPattern::GeneratedCode {
                    generator: "file_pattern".to_string(),
                    markers: vec![pattern.clone()],
                });
            }
        }

        // Check for comment markers in first 10 lines
        let lines: Vec<&str> = source.lines().take(10).collect();
        let generated_markers = [
            "auto-generated",
            "auto generated",
            "DO NOT EDIT",
            "Code generated",
            "This file was automatically generated",
            "<auto-generated",
        ];

        for line in lines {
            for marker in &generated_markers {
                if line.to_lowercase().contains(&marker.to_lowercase()) {
                    return Some(DetectedPattern::GeneratedCode {
                        generator: "comment_marker".to_string(),
                        markers: vec![marker.to_string()],
                    });
                }
            }
        }

        None
    }

    /// Helper method to match file patterns (simplified glob matching)
    fn matches_pattern(&self, path: &str, pattern: &str) -> bool {
        if pattern.starts_with('*') && pattern.ends_with('*') {
            let inner = &pattern[1..pattern.len() - 1];
            path.contains(inner)
        } else if pattern.starts_with('*') {
            path.ends_with(&pattern[1..])
        } else if pattern.ends_with('*') {
            path.starts_with(&pattern[..pattern.len() - 1])
        } else {
            path == pattern
        }
    }

    /// Dispatch analysis to language-specific analyzers
    async fn analyze_with_language_specific_analyzer(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        // Stage 1: Pre-AST Exclusion - Check for generated code
        if let Some(generated_pattern) = self.is_generated_file(
            &parsed_file.file_path.display().to_string(),
            &parsed_file.source,
        ) {
            debug!("Excluding file as generated code: {:?}", generated_pattern);
            return Ok(Vec::new());
        }

        // Dispatch to language-specific analyzer
        match parsed_file.language {
            SourceLanguage::Rust => {
                let analyzer = RustGodObjectAnalyzer::new(&self.config);
                analyzer.analyze(parsed_file)
            }
            SourceLanguage::Python => {
                let analyzer = PythonGodObjectAnalyzer::new(&self.config);
                analyzer.analyze(parsed_file)
            }
            SourceLanguage::JavaScript => {
                let analyzer = JavaScriptGodObjectAnalyzer::new(&self.config);
                analyzer.analyze(parsed_file)
            }
            SourceLanguage::TypeScript => {
                let analyzer = TypeScriptGodObjectAnalyzer::new(&self.config);
                analyzer.analyze(parsed_file)
            }
        }
    }
}

#[async_trait]
impl AnalysisDetector for GodObjectDetector {
    async fn detect_issues(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        debug!(
            "Running God Object detection on: {}",
            parsed_file.file_path.display()
        );

        let result = self.analyze_with_language_specific_analyzer(parsed_file).await;

        match &result {
            Ok(issues) => {
                if issues.is_empty() {
                    debug!(
                        "No God Object issues found in {}",
                        parsed_file.file_path.display()
                    );
                } else {
                    info!(
                        "Found {} God Object issues in {}",
                        issues.len(),
                        parsed_file.file_path.display()
                    );
                }
            }
            Err(e) => {
                debug!(
                    "Error analyzing {} for God Objects: {}",
                    parsed_file.file_path.display(),
                    e
                );
            }
        }

        result
    }

    fn get_detector_name(&self) -> &'static str {
        "GodObjectDetector"
    }

    fn get_anti_pattern_types(&self) -> Vec<AntiPatternType> {
        vec![AntiPatternType {
            anti_pattern_type_id: Some(1),
            name: "God Object".to_string(),
            description: "A class or struct that centralizes too many responsibilities, violating the Single Responsibility Principle. Enhanced with pattern recognition to reduce false positives.".to_string(),
            category: "Abstraction-Based".to_string(),
        }]
    }
}

impl Default for GodObjectDetector {
    /// Creates a `GodObjectDetector` with default configuration.
    ///
    /// Enables all enhanced detection features with language-specific thresholds.
    fn default() -> Self {
        Self::with_config(GodObjectConfig::default())
    }
}

// Legacy compatibility - export old field names for backwards compatibility
impl GodObjectDetector {
    /// Get method threshold for backwards compatibility
    pub fn method_threshold(&self) -> usize {
        self.config.get_method_threshold(SourceLanguage::Rust) // Default to Rust
    }

    /// Get field threshold for backwards compatibility
    pub fn field_threshold(&self) -> usize {
        self.config.get_field_threshold(SourceLanguage::Rust) // Default to Rust
    }
}

