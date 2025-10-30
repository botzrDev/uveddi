//! Configuration and thresholds for God Object detection

use crate::ast::tree_sitter_impl::SourceLanguage;
use std::collections::{HashMap, HashSet};

/// Configuration for advanced God Object detection
#[derive(Debug, Clone)]
pub struct GodObjectConfig {
    /// Method count thresholds per language
    pub method_thresholds: HashMap<SourceLanguage, usize>,
    /// Field count thresholds per language
    pub field_thresholds: HashMap<SourceLanguage, usize>,
    /// Framework modules that should be excluded from detection
    pub framework_modules: HashSet<String>,
    /// Generated code file patterns to exclude
    pub generated_file_patterns: Vec<String>,
    /// Design patterns to recognize and exclude
    pub recognize_patterns: bool,
    /// Enable behavioral analysis
    pub enable_behavioral_analysis: bool,
    /// Enable LCOM4 cohesion analysis
    pub enable_cohesion_analysis: bool,
    /// Cyclomatic complexity threshold for "trivial" methods
    pub trivial_method_cc_threshold: u32,
}

impl Default for GodObjectConfig {
    fn default() -> Self {
        let mut method_thresholds = HashMap::new();
        method_thresholds.insert(SourceLanguage::Rust, 30);
        method_thresholds.insert(SourceLanguage::Python, 25);
        method_thresholds.insert(SourceLanguage::JavaScript, 20);
        method_thresholds.insert(SourceLanguage::TypeScript, 15); // Stricter for TypeScript due to better type system

        let mut field_thresholds = HashMap::new();
        field_thresholds.insert(SourceLanguage::Rust, 20);
        field_thresholds.insert(SourceLanguage::Python, 15);
        field_thresholds.insert(SourceLanguage::JavaScript, 12);
        field_thresholds.insert(SourceLanguage::TypeScript, 10); // Stricter for TypeScript due to interfaces

        let mut framework_modules = HashSet::new();
        // Rust frameworks
        framework_modules.insert("axum".to_string());
        framework_modules.insert("rocket".to_string());
        framework_modules.insert("actix_web".to_string());
        framework_modules.insert("serde".to_string());
        framework_modules.insert("diesel".to_string());
        framework_modules.insert("sqlx".to_string());
        framework_modules.insert("tokio".to_string());

        // Python frameworks
        framework_modules.insert("django".to_string());
        framework_modules.insert("flask".to_string());
        framework_modules.insert("fastapi".to_string());
        framework_modules.insert("pydantic".to_string());
        framework_modules.insert("sqlalchemy".to_string());

        // JavaScript frameworks
        framework_modules.insert("react".to_string());
        framework_modules.insert("express".to_string());
        framework_modules.insert("vue".to_string());
        framework_modules.insert("axios".to_string());

        // TypeScript frameworks and libraries
        framework_modules.insert("angular".to_string());
        framework_modules.insert("nest".to_string());
        framework_modules.insert("nestjs".to_string());
        framework_modules.insert("typescript".to_string());
        framework_modules.insert("tsc".to_string());
        framework_modules.insert("next".to_string());
        framework_modules.insert("nuxt".to_string());
        framework_modules.insert("svelte".to_string());
        framework_modules.insert("solid-js".to_string());

        let generated_file_patterns = vec![
            "*_pb2.py".to_string(),
            "*_pb2_grpc.py".to_string(),
            "*.g.cs".to_string(),
            "*.generated.*".to_string(),
            "*_generated.*".to_string(),
        ];

        Self {
            method_thresholds,
            field_thresholds,
            framework_modules,
            generated_file_patterns,
            recognize_patterns: true,
            enable_behavioral_analysis: true,
            enable_cohesion_analysis: true,
            trivial_method_cc_threshold: 2,
        }
    }
}

impl GodObjectConfig {
    /// Create a new configuration with custom thresholds
    pub fn new() -> Self {
        Self::default()
    }

    /// Create configuration with uniform thresholds for all languages
    pub fn with_uniform_thresholds(method_threshold: usize, field_threshold: usize) -> Self {
        let mut config = Self::default();
        for (_, threshold) in config.method_thresholds.iter_mut() {
            *threshold = method_threshold;
        }
        for (_, threshold) in config.field_thresholds.iter_mut() {
            *threshold = field_threshold;
        }
        config
    }

    /// Get method threshold for a specific language
    pub fn get_method_threshold(&self, language: SourceLanguage) -> usize {
        self.method_thresholds.get(&language).copied().unwrap_or(10)
    }

    /// Get field threshold for a specific language
    pub fn get_field_threshold(&self, language: SourceLanguage) -> usize {
        self.field_thresholds.get(&language).copied().unwrap_or(8)
    }

    /// Set method threshold for a specific language
    pub fn set_method_threshold(&mut self, language: SourceLanguage, threshold: usize) {
        self.method_thresholds.insert(language, threshold);
    }

    /// Set field threshold for a specific language
    pub fn set_field_threshold(&mut self, language: SourceLanguage, threshold: usize) {
        self.field_thresholds.insert(language, threshold);
    }

    /// Add a framework module to exclude from detection
    pub fn add_framework_module(&mut self, module: String) {
        self.framework_modules.insert(module);
    }

    /// Add a generated file pattern to exclude from detection
    pub fn add_generated_file_pattern(&mut self, pattern: String) {
        self.generated_file_patterns.push(pattern);
    }

    /// Enable or disable pattern recognition
    pub fn set_pattern_recognition(&mut self, enabled: bool) {
        self.recognize_patterns = enabled;
    }

    /// Enable or disable behavioral analysis
    pub fn set_behavioral_analysis(&mut self, enabled: bool) {
        self.enable_behavioral_analysis = enabled;
    }

    /// Enable or disable cohesion analysis
    pub fn set_cohesion_analysis(&mut self, enabled: bool) {
        self.enable_cohesion_analysis = enabled;
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        // Check that thresholds are reasonable
        for (language, threshold) in &self.method_thresholds {
            if *threshold == 0 {
                return Err(format!(
                    "Method threshold for {:?} cannot be zero",
                    language
                ));
            }
            if *threshold > 1000 {
                return Err(format!(
                    "Method threshold for {:?} is unreasonably high: {}",
                    language, threshold
                ));
            }
        }

        for (language, threshold) in &self.field_thresholds {
            if *threshold == 0 {
                return Err(format!("Field threshold for {:?} cannot be zero", language));
            }
            if *threshold > 1000 {
                return Err(format!(
                    "Field threshold for {:?} is unreasonably high: {}",
                    language, threshold
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_creation() {
        let config = GodObjectConfig::default();
        assert!(config.method_thresholds.len() > 0);
        assert!(config.field_thresholds.len() > 0);
        assert!(config.framework_modules.len() > 0);
        assert!(config.recognize_patterns);
    }

    #[test]
    fn test_uniform_thresholds() {
        let config = GodObjectConfig::with_uniform_thresholds(15, 10);
        for threshold in config.method_thresholds.values() {
            assert_eq!(*threshold, 15);
        }
        for threshold in config.field_thresholds.values() {
            assert_eq!(*threshold, 10);
        }
    }

    #[test]
    fn test_language_specific_thresholds() {
        let config = GodObjectConfig::default();
        let rust_method_threshold = config.get_method_threshold(SourceLanguage::Rust);
        let typescript_method_threshold = config.get_method_threshold(SourceLanguage::TypeScript);

        // TypeScript should have stricter thresholds
        assert!(typescript_method_threshold < rust_method_threshold);
    }

    #[test]
    fn test_config_validation() {
        let config = GodObjectConfig::default();
        assert!(config.validate().is_ok());

        let mut invalid_config = GodObjectConfig::default();
        invalid_config
            .method_thresholds
            .insert(SourceLanguage::Rust, 0);
        assert!(invalid_config.validate().is_err());
    }
}
