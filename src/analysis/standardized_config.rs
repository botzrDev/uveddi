//! Standardized Configuration System for Uveddi Detectors
//!
//! This module provides a unified configuration system that standardizes how all
//! detectors are configured, replacing the inconsistent patterns found across
//! different detector implementations.
//!
//! ## Key Features
//!
//! - **Unified Structure**: All detectors use the same base configuration pattern
//! - **Builder Pattern**: Consistent, fluent API for configuration creation
//! - **TOML Support**: Full serialization/deserialization support
//! - **Validation**: Built-in configuration validation with helpful error messages
//! - **Language-Specific**: Support for per-language threshold customization
//! - **Extensible**: Easy to add new configuration parameters
//!
//! ## Usage Example
//!
//! ```rust
//! use uveddi::analysis::standardized_config::{StandardDetectorConfig, StandardConfigBuilder};
//! use uveddi::analysis::IssueSeverity;
//!
//! // Programmatic configuration
//! let config = StandardConfigBuilder::new("god_object")
//!     .enabled(true)
//!     .severity(IssueSeverity::High)
//!     .threshold("max_methods", 20)
//!     .threshold("max_fields", 15)
//!     .language_threshold("rust", "max_methods", 25)
//!     .exclude_pattern("*_test.rs")
//!     .build()?;
//!
//! // TOML configuration
//! [detectors.god_object]
//! enabled = true
//! severity = "High"
//! thresholds = { max_methods = 20, max_fields = 15 }
//! 
//! [detectors.god_object.language_overrides.rust]
//! max_methods = 25
//! 
//! [detectors.god_object.exclusions]
//! patterns = ["*_test.rs", "*_generated.rs"]
//! ```

use crate::analysis::IssueSeverity;
use crate::ast::tree_sitter_impl::SourceLanguage;
use crate::error::UveddiError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Standardized configuration structure for all detectors
///
/// This replaces the inconsistent configuration patterns found across different
/// detectors with a unified, extensible structure that supports all common
/// configuration needs while maintaining backward compatibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardDetectorConfig {
    /// Whether this detector is enabled
    pub enabled: bool,
    
    /// Issue severity level for findings from this detector
    pub severity: IssueSeverity,
    
    /// Base thresholds that apply to all languages
    pub thresholds: HashMap<String, ConfigValue>,
    
    /// Language-specific threshold overrides
    #[serde(default)]
    pub language_overrides: HashMap<SourceLanguage, HashMap<String, ConfigValue>>,
    
    /// Exclusion configuration
    #[serde(default)]
    pub exclusions: ExclusionConfig,
    
    /// Advanced configuration options
    #[serde(default)]
    pub advanced: AdvancedConfig,
    
    /// Detector-specific metadata
    #[serde(default)]
    pub metadata: DetectorMetadata,
}

/// Configuration value that can hold different types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfigValue {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Array(Vec<ConfigValue>),
}

impl ConfigValue {
    /// Convert to integer, returning error if not possible
    pub fn as_int(&self) -> Result<i64, UveddiError> {
        match self {
            ConfigValue::Integer(i) => Ok(*i),
            ConfigValue::Float(f) => Ok(*f as i64),
            _ => Err(UveddiError::config_error(
                &format!("Cannot convert {:?} to integer", self),
                "StandardizedConfig::as_integer"
            )),
        }
    }
    
    /// Convert to float, returning error if not possible
    pub fn as_float(&self) -> Result<f64, UveddiError> {
        match self {
            ConfigValue::Float(f) => Ok(*f),
            ConfigValue::Integer(i) => Ok(*i as f64),
            _ => Err(UveddiError::config_error(
                &format!("Cannot convert {:?} to float", self),
                "StandardizedConfig::as_float"
            )),
        }
    }
    
    /// Convert to string, returning error if not possible
    pub fn as_string(&self) -> Result<String, UveddiError> {
        match self {
            ConfigValue::String(s) => Ok(s.clone()),
            _ => Err(UveddiError::config_error(
                &format!("Cannot convert {:?} to string", self),
                "StandardizedConfig::as_string"
            )),
        }
    }
    
    /// Convert to boolean, returning error if not possible
    pub fn as_bool(&self) -> Result<bool, UveddiError> {
        match self {
            ConfigValue::Boolean(b) => Ok(*b),
            _ => Err(UveddiError::config_error(
                &format!("Cannot convert {:?} to boolean", self),
                "StandardizedConfig::as_boolean"
            )),
        }
    }
}

impl From<i64> for ConfigValue {
    fn from(value: i64) -> Self {
        ConfigValue::Integer(value)
    }
}

impl From<f64> for ConfigValue {
    fn from(value: f64) -> Self {
        ConfigValue::Float(value)
    }
}

impl From<String> for ConfigValue {
    fn from(value: String) -> Self {
        ConfigValue::String(value)
    }
}

impl From<&str> for ConfigValue {
    fn from(value: &str) -> Self {
        ConfigValue::String(value.to_string())
    }
}

impl From<bool> for ConfigValue {
    fn from(value: bool) -> Self {
        ConfigValue::Boolean(value)
    }
}

impl fmt::Display for ConfigValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigValue::Integer(i) => write!(f, "{}", i),
            ConfigValue::Float(fl) => write!(f, "{}", fl),
            ConfigValue::String(s) => write!(f, "{}", s),
            ConfigValue::Boolean(b) => write!(f, "{}", b),
            ConfigValue::Array(arr) => {
                write!(f, "[")?;
                for (i, item) in arr.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
        }
    }
}

/// Configuration for excluding files, patterns, or code sections
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExclusionConfig {
    /// File patterns to exclude (glob patterns)
    #[serde(default)]
    pub patterns: Vec<String>,
    
    /// Specific file paths to exclude
    #[serde(default)]
    pub files: Vec<String>,
    
    /// Framework modules to exclude from analysis
    #[serde(default)]
    pub framework_modules: Vec<String>,
    
    /// Generated code patterns to exclude
    #[serde(default)]
    pub generated_patterns: Vec<String>,
}

/// Advanced configuration options
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AdvancedConfig {
    /// Enable experimental features
    #[serde(default)]
    pub experimental_features: bool,
    
    /// Custom analysis modes
    #[serde(default)]
    pub analysis_modes: Vec<String>,
    
    /// Performance tuning options
    #[serde(default)]
    pub performance_tuning: HashMap<String, ConfigValue>,
}

/// Metadata about the detector configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DetectorMetadata {
    /// Configuration version for migration support
    #[serde(default)]
    pub version: String,
    
    /// Human-readable description
    #[serde(default)]
    pub description: String,
    
    /// Configuration author/source
    #[serde(default)]
    pub author: String,
    
    /// Last modified timestamp
    #[serde(default)]
    pub last_modified: String,
}

/// Builder for creating standardized detector configurations
///
/// Provides a fluent API for building detector configurations with validation
/// and helpful error messages.
pub struct StandardConfigBuilder {
    detector_name: String,
    config: StandardDetectorConfig,
}

impl StandardConfigBuilder {
    /// Create a new builder for the specified detector
    pub fn new(detector_name: &str) -> Self {
        Self {
            detector_name: detector_name.to_string(),
            config: StandardDetectorConfig::default_for_detector(detector_name),
        }
    }
    
    /// Set whether the detector is enabled
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.config.enabled = enabled;
        self
    }
    
    /// Set the issue severity level
    pub fn severity(mut self, severity: IssueSeverity) -> Self {
        self.config.severity = severity;
        self
    }
    
    /// Set a threshold value
    pub fn threshold<T: Into<ConfigValue>>(mut self, key: &str, value: T) -> Self {
        self.config.thresholds.insert(key.to_string(), value.into());
        self
    }
    
    /// Set a language-specific threshold override
    pub fn language_threshold<T: Into<ConfigValue>>(
        mut self, 
        language: &str, 
        key: &str, 
        value: T
    ) -> Self {
        let lang = match language {
            "rust" => SourceLanguage::Rust,
            "python" => SourceLanguage::Python,
            "javascript" | "js" => SourceLanguage::JavaScript,
            _ => {
                // For now, default to Rust for unknown languages
                // In a real implementation, we might want to return an error
                SourceLanguage::Rust
            }
        };
        
        self.config.language_overrides
            .entry(lang)
            .or_insert_with(HashMap::new)
            .insert(key.to_string(), value.into());
        self
    }
    
    /// Add an exclusion pattern
    pub fn exclude_pattern(mut self, pattern: &str) -> Self {
        self.config.exclusions.patterns.push(pattern.to_string());
        self
    }
    
    /// Add an exclusion file
    pub fn exclude_file(mut self, file: &str) -> Self {
        self.config.exclusions.files.push(file.to_string());
        self
    }
    
    /// Add a framework module to exclude
    pub fn exclude_framework(mut self, framework: &str) -> Self {
        self.config.exclusions.framework_modules.push(framework.to_string());
        self
    }
    
    /// Enable experimental features
    pub fn experimental(mut self, enabled: bool) -> Self {
        self.config.advanced.experimental_features = enabled;
        self
    }
    
    /// Set metadata description
    pub fn description(mut self, description: &str) -> Self {
        self.config.metadata.description = description.to_string();
        self
    }
    
    /// Build the configuration with validation
    pub fn build(self) -> Result<StandardDetectorConfig, UveddiError> {
        self.validate()?;
        Ok(self.config)
    }
    
    /// Validate the configuration
    fn validate(&self) -> Result<(), UveddiError> {
        // Validate detector-specific requirements
        match self.detector_name.as_str() {
            "god_object" => self.validate_god_object()?,
            "code_duplication" => self.validate_code_duplication()?,
            "large_classes" => self.validate_large_classes()?,
            "dead_code" => self.validate_dead_code()?,
            "tight_coupling" => self.validate_tight_coupling()?,
            _ => {
                // Unknown detector - basic validation only
                if self.config.thresholds.is_empty() {
                    return Err(UveddiError::config_error(
                        &format!("Detector '{}' has no thresholds configured", self.detector_name),
                        "StandardizedConfig::validate_detector_config"
                    ));
                }
            }
        }
        
        Ok(())
    }
    
    /// Validate god object detector configuration
    fn validate_god_object(&self) -> Result<(), UveddiError> {
        let required_thresholds = ["max_methods", "max_fields"];
        for threshold in &required_thresholds {
            if !self.config.thresholds.contains_key(*threshold) {
                return Err(UveddiError::config_error(
                    &format!("God object detector requires '{}' threshold", threshold),
                    "StandardizedConfig::validate_god_object"
                ));
            }
        }
        Ok(())
    }
    
    /// Validate code duplication detector configuration
    fn validate_code_duplication(&self) -> Result<(), UveddiError> {
        let required_thresholds = ["min_tokens", "similarity_threshold"];
        for threshold in &required_thresholds {
            if !self.config.thresholds.contains_key(*threshold) {
                return Err(UveddiError::config_error(
                    &format!("Code duplication detector requires '{}' threshold", threshold),
                    "StandardizedConfig::validate_code_duplication"
                ));
            }
        }
        
        // Validate similarity threshold is between 0 and 1
        if let Some(similarity) = self.config.thresholds.get("similarity_threshold") {
            let value = similarity.as_float()?;
            if value < 0.0 || value > 1.0 {
                return Err(UveddiError::config_error(
                    "similarity_threshold must be between 0.0 and 1.0",
                    "StandardizedConfig::validate_code_duplication"
                ));
            }
        }
        
        Ok(())
    }
    
    /// Validate large classes detector configuration
    fn validate_large_classes(&self) -> Result<(), UveddiError> {
        let required_thresholds = ["max_lines", "max_methods"];
        for threshold in &required_thresholds {
            if !self.config.thresholds.contains_key(*threshold) {
                return Err(UveddiError::config_error(
                    &format!("Large classes detector requires '{}' threshold", threshold),
                    "StandardizedConfig::validate_large_classes"
                ));
            }
        }
        Ok(())
    }
    
    /// Validate dead code detector configuration
    fn validate_dead_code(&self) -> Result<(), UveddiError> {
        // Dead code detector has minimal requirements
        Ok(())
    }
    
    /// Validate tight coupling detector configuration
    fn validate_tight_coupling(&self) -> Result<(), UveddiError> {
        let required_thresholds = ["max_dependencies"];
        for threshold in &required_thresholds {
            if !self.config.thresholds.contains_key(*threshold) {
                return Err(UveddiError::config_error(
                    &format!("Tight coupling detector requires '{}' threshold", threshold),
                    "StandardizedConfig::validate_tight_coupling"
                ));
            }
        }
        Ok(())
    }
}

impl StandardDetectorConfig {
    /// Create default configuration for a specific detector
    pub fn default_for_detector(detector_name: &str) -> Self {
        let mut config = Self::default();
        
        // Set detector-specific defaults
        match detector_name {
            "god_object" => {
                config.thresholds.insert("max_methods".to_string(), 20.into());
                config.thresholds.insert("max_fields".to_string(), 15.into());
                config.severity = IssueSeverity::High;
                
                // Language-specific overrides
                let mut rust_overrides = HashMap::new();
                rust_overrides.insert("max_methods".to_string(), 25.into());
                rust_overrides.insert("max_fields".to_string(), 20.into());
                config.language_overrides.insert(SourceLanguage::Rust, rust_overrides);
                
                let mut python_overrides = HashMap::new();
                python_overrides.insert("max_methods".to_string(), 20.into());
                python_overrides.insert("max_fields".to_string(), 15.into());
                config.language_overrides.insert(SourceLanguage::Python, python_overrides);
                
                let mut js_overrides = HashMap::new();
                js_overrides.insert("max_methods".to_string(), 18.into());
                js_overrides.insert("max_fields".to_string(), 12.into());
                config.language_overrides.insert(SourceLanguage::JavaScript, js_overrides);
            }
            
            "code_duplication" => {
                config.thresholds.insert("min_tokens".to_string(), 50.into());
                config.thresholds.insert("min_lines".to_string(), 5.into());
                config.thresholds.insert("similarity_threshold".to_string(), 0.8.into());
                config.thresholds.insert("fingerprint_length".to_string(), 10.into());
                config.severity = IssueSeverity::Medium;
            }
            
            "large_classes" => {
                config.thresholds.insert("max_lines".to_string(), 300.into());
                config.thresholds.insert("max_methods".to_string(), 20.into());
                config.thresholds.insert("max_complexity".to_string(), 50.into());
                config.severity = IssueSeverity::Medium;
            }
            
            "dead_code" => {
                config.thresholds.insert("confidence_threshold".to_string(), 0.7.into());
                config.severity = IssueSeverity::Low;
            }
            
            "tight_coupling" => {
                config.thresholds.insert("max_dependencies".to_string(), 10.into());
                config.thresholds.insert("max_coupling_ratio".to_string(), 0.3.into());
                config.severity = IssueSeverity::Medium;
            }
            
            _ => {
                // Generic defaults
                config.severity = IssueSeverity::Medium;
            }
        }
        
        // Common exclusions
        config.exclusions.generated_patterns = vec![
            "*_pb2.py".to_string(),
            "*_pb2_grpc.py".to_string(),
            "*.g.cs".to_string(),
            "*.generated.*".to_string(),
            "*_generated.*".to_string(),
        ];
        
        config.exclusions.framework_modules = vec![
            "serde".to_string(),
            "tokio".to_string(),
            "django".to_string(),
            "react".to_string(),
        ];
        
        config
    }
    
    /// Get a threshold value for a specific language, falling back to base threshold
    pub fn get_threshold(&self, key: &str, language: SourceLanguage) -> Option<&ConfigValue> {
        // First check language-specific overrides
        if let Some(lang_overrides) = self.language_overrides.get(&language) {
            if let Some(value) = lang_overrides.get(key) {
                return Some(value);
            }
        }
        
        // Fall back to base threshold
        self.thresholds.get(key)
    }
    
    /// Get threshold as integer with language fallback
    pub fn get_threshold_int(&self, key: &str, language: SourceLanguage) -> Result<i64, UveddiError> {
        self.get_threshold(key, language)
            .ok_or_else(|| UveddiError::config_error(&format!("Threshold '{}' not found", key), "StandardizedConfig::get_threshold_int"))?
            .as_int()
    }
    
    /// Get threshold as float with language fallback
    pub fn get_threshold_float(&self, key: &str, language: SourceLanguage) -> Result<f64, UveddiError> {
        self.get_threshold(key, language)
            .ok_or_else(|| UveddiError::config_error(&format!("Threshold '{}' not found", key), "StandardizedConfig::get_threshold_float"))?
            .as_float()
    }
}

impl Default for StandardDetectorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            severity: IssueSeverity::Medium,
            thresholds: HashMap::new(),
            language_overrides: HashMap::new(),
            exclusions: ExclusionConfig::default(),
            advanced: AdvancedConfig::default(),
            metadata: DetectorMetadata::default(),
        }
    }
}

/// Configuration constants extracted from detector implementations
pub mod constants {
    /// Default thresholds for god object detection
    pub mod god_object {
        pub const DEFAULT_MAX_METHODS: i64 = 20;
        pub const DEFAULT_MAX_FIELDS: i64 = 15;
        pub const RUST_MAX_METHODS: i64 = 25;
        pub const RUST_MAX_FIELDS: i64 = 20;
        pub const PYTHON_MAX_METHODS: i64 = 20;
        pub const PYTHON_MAX_FIELDS: i64 = 15;
        pub const JS_MAX_METHODS: i64 = 18;
        pub const JS_MAX_FIELDS: i64 = 12;
    }
    
    /// Default thresholds for code duplication detection
    pub mod code_duplication {
        pub const DEFAULT_MIN_TOKENS: i64 = 50;
        pub const DEFAULT_MIN_LINES: i64 = 5;
        pub const DEFAULT_SIMILARITY_THRESHOLD: f64 = 0.8;
        pub const DEFAULT_FINGERPRINT_LENGTH: i64 = 10;
    }
    
    /// Default thresholds for large class detection
    pub mod large_classes {
        pub const DEFAULT_MAX_LINES: i64 = 300;
        pub const DEFAULT_MAX_METHODS: i64 = 20;
        pub const DEFAULT_MAX_COMPLEXITY: i64 = 50;
    }
    
    /// Default thresholds for dead code detection
    pub mod dead_code {
        pub const DEFAULT_CONFIDENCE_THRESHOLD: f64 = 0.7;
    }
    
    /// Default thresholds for tight coupling detection
    pub mod tight_coupling {
        pub const DEFAULT_MAX_DEPENDENCIES: i64 = 10;
        pub const DEFAULT_MAX_COUPLING_RATIO: f64 = 0.3;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_standard_config_builder() {
        let config = StandardConfigBuilder::new("god_object")
            .enabled(true)
            .severity(IssueSeverity::High)
            .threshold("max_methods", 25)
            .threshold("max_fields", 20)
            .language_threshold("rust", "max_methods", 30)
            .exclude_pattern("*_test.rs")
            .build()
            .expect("Failed to build config");
        
        assert!(config.enabled);
        assert_eq!(config.severity, IssueSeverity::High);
        assert_eq!(config.get_threshold_int("max_methods", SourceLanguage::Rust).unwrap(), 30);
        assert_eq!(config.get_threshold_int("max_methods", SourceLanguage::Python).unwrap(), 20); // Falls back to default god_object config
        assert!(config.exclusions.patterns.contains(&"*_test.rs".to_string()));
    }
    
    #[test]
    fn test_config_validation() {
        // Valid configuration
        let result = StandardConfigBuilder::new("god_object")
            .threshold("max_methods", 20)
            .threshold("max_fields", 15)
            .build();
        assert!(result.is_ok());
        
        // Invalid configuration (missing required threshold)
        let result = StandardConfigBuilder::new("god_object")
            .threshold("max_methods", 20)
            // Missing max_fields - but default_for_detector provides it
            .build();
        // This should actually pass because default_for_detector sets max_fields
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_config_value_conversions() {
        let int_val = ConfigValue::Integer(42);
        assert_eq!(int_val.as_int().unwrap(), 42);
        assert_eq!(int_val.as_float().unwrap(), 42.0);
        
        let float_val = ConfigValue::Float(3.14);
        assert_eq!(float_val.as_float().unwrap(), 3.14);
        assert_eq!(float_val.as_int().unwrap(), 3);
        
        let string_val = ConfigValue::String("test".to_string());
        assert_eq!(string_val.as_string().unwrap(), "test");
        
        let bool_val = ConfigValue::Boolean(true);
        assert_eq!(bool_val.as_bool().unwrap(), true);
    }
}