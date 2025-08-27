//! Configuration Migration Utilities
//!
//! This module provides utilities for migrating from the old inconsistent
//! detector configuration patterns to the new standardized configuration system.
//! It maintains backward compatibility while enabling gradual migration.

use crate::analysis::detectors::anti_patterns::{
    code_duplication::DuplicationConfig, dead_code::DeadCodeConfig, god_object::GodObjectConfig,
    large_classes::LargeClassConfig,
};
use crate::analysis::standardized_config::{
    StandardConfigBuilder, StandardDetectorConfig,
};
use crate::analysis::IssueSeverity;
use crate::ast::tree_sitter_impl::SourceLanguage;
use crate::error::UveddiError;
use std::collections::HashMap;

/// Trait for migrating old configuration formats to standardized format
pub trait ConfigMigration<T> {
    /// Convert old configuration to standardized format
    fn to_standard(&self) -> Result<StandardDetectorConfig, UveddiError>;

    /// Convert standardized configuration back to old format (for compatibility)
    fn from_standard(config: &StandardDetectorConfig) -> Result<T, UveddiError>;
}

/// Migration implementation for GodObjectConfig
impl ConfigMigration<GodObjectConfig> for GodObjectConfig {
    fn to_standard(&self) -> Result<StandardDetectorConfig, UveddiError> {
        let mut builder = StandardConfigBuilder::new("god_object")
            .enabled(true)
            .severity(IssueSeverity::High);

        // Convert method thresholds
        for (language, threshold) in &self.method_thresholds {
            let lang_str = match language {
                SourceLanguage::Rust => "rust",
                SourceLanguage::Python => "python",
                SourceLanguage::JavaScript => "javascript",
                SourceLanguage::TypeScript => "typescript", // UV-XXX: Add TypeScript support
            };
            builder = builder.language_threshold(lang_str, "max_methods", *threshold as i64);
        }

        // Convert field thresholds
        for (language, threshold) in &self.field_thresholds {
            let lang_str = match language {
                SourceLanguage::Rust => "rust",
                SourceLanguage::Python => "python",
                SourceLanguage::JavaScript => "javascript",
                SourceLanguage::TypeScript => "typescript", // UV-XXX: Add TypeScript support
            };
            builder = builder.language_threshold(lang_str, "max_fields", *threshold as i64);
        }

        // Convert framework exclusions
        for framework in &self.framework_modules {
            builder = builder.exclude_framework(framework);
        }

        // Convert generated file patterns
        for pattern in &self.generated_file_patterns {
            builder = builder.exclude_pattern(pattern);
        }

        // Convert advanced settings
        if self.recognize_patterns {
            builder = builder.experimental(true);
        }

        builder.build()
    }

    fn from_standard(config: &StandardDetectorConfig) -> Result<GodObjectConfig, UveddiError> {
        let mut method_thresholds = HashMap::new();
        let mut field_thresholds = HashMap::new();

        // Extract language-specific thresholds
        for (language, overrides) in &config.language_overrides {
            if let Some(methods) = overrides.get("max_methods") {
                method_thresholds.insert(*language, methods.as_int()? as usize);
            }
            if let Some(fields) = overrides.get("max_fields") {
                field_thresholds.insert(*language, fields.as_int()? as usize);
            }
        }

        // Set defaults if not specified
        if method_thresholds.is_empty() {
            if let Some(default_methods) = config.thresholds.get("max_methods") {
                let default_val = default_methods.as_int()? as usize;
                method_thresholds.insert(SourceLanguage::Rust, default_val);
                method_thresholds.insert(SourceLanguage::Python, default_val);
                method_thresholds.insert(SourceLanguage::JavaScript, default_val);
            }
        }

        if field_thresholds.is_empty() {
            if let Some(default_fields) = config.thresholds.get("max_fields") {
                let default_val = default_fields.as_int()? as usize;
                field_thresholds.insert(SourceLanguage::Rust, default_val);
                field_thresholds.insert(SourceLanguage::Python, default_val);
                field_thresholds.insert(SourceLanguage::JavaScript, default_val);
            }
        }

        Ok(GodObjectConfig {
            method_thresholds,
            field_thresholds,
            framework_modules: config
                .exclusions
                .framework_modules
                .iter()
                .cloned()
                .collect(),
            generated_file_patterns: config.exclusions.generated_patterns.clone(),
            recognize_patterns: config.advanced.experimental_features,
            enable_behavioral_analysis: true, // Default
            enable_cohesion_analysis: true,   // Default
            trivial_method_cc_threshold: 2,   // Default
        })
    }
}

/// Migration implementation for DuplicationConfig
impl ConfigMigration<DuplicationConfig> for DuplicationConfig {
    fn to_standard(&self) -> Result<StandardDetectorConfig, UveddiError> {
        StandardConfigBuilder::new("code_duplication")
            .enabled(true)
            .severity(IssueSeverity::Medium)
            .threshold("min_tokens", self.min_tokens as i64)
            .threshold("min_lines", self.min_lines as i64)
            .threshold("similarity_threshold", self.similarity_threshold)
            .threshold("fingerprint_length", self.fingerprint_length as i64)
            .threshold("ignore_identifiers", self.ignore_identifiers)
            .threshold("ignore_literals", self.ignore_literals)
            .build()
    }

    fn from_standard(config: &StandardDetectorConfig) -> Result<DuplicationConfig, UveddiError> {
        Ok(DuplicationConfig {
            min_tokens: config.get_threshold_int("min_tokens", SourceLanguage::Rust)? as usize,
            min_lines: config.get_threshold_int("min_lines", SourceLanguage::Rust)? as usize,
            similarity_threshold: config
                .get_threshold_float("similarity_threshold", SourceLanguage::Rust)?,
            fingerprint_length: config
                .get_threshold_int("fingerprint_length", SourceLanguage::Rust)?
                as usize,
            ignore_identifiers: config
                .thresholds
                .get("ignore_identifiers")
                .map(|v| v.as_bool().unwrap_or(false))
                .unwrap_or(false),
            ignore_literals: config
                .thresholds
                .get("ignore_literals")
                .map(|v| v.as_bool().unwrap_or(false))
                .unwrap_or(false),
            ..Default::default()
        })
    }
}

/// Migration implementation for LargeClassConfig
impl ConfigMigration<LargeClassConfig> for LargeClassConfig {
    fn to_standard(&self) -> Result<StandardDetectorConfig, UveddiError> {
        let mut builder = StandardConfigBuilder::new("large_classes")
            .enabled(true)
            .severity(IssueSeverity::Medium);

        // Convert Rust thresholds
        builder = builder
            .language_threshold(
                "rust",
                "max_lines",
                self.rust_thresholds.max_logical_loc as i64,
            )
            .language_threshold(
                "rust",
                "max_methods",
                self.rust_thresholds.max_methods as i64,
            )
            .language_threshold("rust", "max_fields", self.rust_thresholds.max_fields as i64)
            .language_threshold(
                "rust",
                "max_complexity",
                self.rust_thresholds.max_cyclomatic_complexity as i64,
            );

        // Convert Python thresholds
        builder = builder
            .language_threshold(
                "python",
                "max_lines",
                self.python_thresholds.max_logical_loc as i64,
            )
            .language_threshold(
                "python",
                "max_methods",
                self.python_thresholds.max_methods as i64,
            )
            .language_threshold(
                "python",
                "max_fields",
                self.python_thresholds.max_fields as i64,
            )
            .language_threshold(
                "python",
                "max_complexity",
                self.python_thresholds.max_cyclomatic_complexity as i64,
            );

        // Convert JavaScript thresholds
        builder = builder
            .language_threshold(
                "javascript",
                "max_lines",
                self.javascript_thresholds.max_logical_loc as i64,
            )
            .language_threshold(
                "javascript",
                "max_methods",
                self.javascript_thresholds.max_methods as i64,
            )
            .language_threshold(
                "javascript",
                "max_fields",
                self.javascript_thresholds.max_fields as i64,
            )
            .language_threshold(
                "javascript",
                "max_complexity",
                self.javascript_thresholds.max_cyclomatic_complexity as i64,
            );

        // Convert feature flags
        if self.enable_lcom_analysis {
            builder = builder.experimental(true);
        }

        builder.build()
    }

    fn from_standard(config: &StandardDetectorConfig) -> Result<LargeClassConfig, UveddiError> {
        use crate::analysis::detectors::anti_patterns::large_classes::{
            LanguageThresholds, SeverityWeights,
        };

        let rust_thresholds = LanguageThresholds {
            max_logical_loc: config.get_threshold_int("max_lines", SourceLanguage::Rust)? as u32,
            max_methods: config.get_threshold_int("max_methods", SourceLanguage::Rust)? as u32,
            max_fields: config.get_threshold_int("max_fields", SourceLanguage::Rust)? as u32,
            max_cyclomatic_complexity: config
                .get_threshold_int("max_complexity", SourceLanguage::Rust)?
                as u32,
            max_cognitive_complexity: 50, // Default
            max_lcom_score: 0.8,          // Default
            max_coupling: 10,             // Default
        };

        let python_thresholds = LanguageThresholds {
            max_logical_loc: config.get_threshold_int("max_lines", SourceLanguage::Python)? as u32,
            max_methods: config.get_threshold_int("max_methods", SourceLanguage::Python)? as u32,
            max_fields: config.get_threshold_int("max_fields", SourceLanguage::Python)? as u32,
            max_cyclomatic_complexity: config
                .get_threshold_int("max_complexity", SourceLanguage::Python)?
                as u32,
            max_cognitive_complexity: 50, // Default
            max_lcom_score: 0.8,          // Default
            max_coupling: 10,             // Default
        };

        let javascript_thresholds = LanguageThresholds {
            max_logical_loc: config.get_threshold_int("max_lines", SourceLanguage::JavaScript)?
                as u32,
            max_methods: config.get_threshold_int("max_methods", SourceLanguage::JavaScript)?
                as u32,
            max_fields: config.get_threshold_int("max_fields", SourceLanguage::JavaScript)? as u32,
            max_cyclomatic_complexity: config
                .get_threshold_int("max_complexity", SourceLanguage::JavaScript)?
                as u32,
            max_cognitive_complexity: 50, // Default
            max_lcom_score: 0.8,          // Default
            max_coupling: 10,             // Default
        };

        Ok(LargeClassConfig {
            rust_thresholds,
            python_thresholds,
            javascript_thresholds,
            enable_lcom_analysis: config.advanced.experimental_features,
            enable_coupling_analysis: true, // Default
            severity_weights: SeverityWeights::default(),
        })
    }
}

/// Migration implementation for DeadCodeConfig
impl ConfigMigration<DeadCodeConfig> for DeadCodeConfig {
    fn to_standard(&self) -> Result<StandardDetectorConfig, UveddiError> {
        let mut builder = StandardConfigBuilder::new("dead_code")
            .enabled(true)
            .severity(IssueSeverity::Low)
            .threshold("min_confidence", self.min_confidence);

        // Convert exclusions
        for pattern in &self.ignore_patterns {
            builder = builder.exclude_pattern(pattern);
        }

        // Note: ignore_files field doesn't exist in DeadCodeConfig, this was likely a mistake
        // The actual field is ignore_patterns which is handled above

        builder.build()
    }

    fn from_standard(config: &StandardDetectorConfig) -> Result<DeadCodeConfig, UveddiError> {
        Ok(DeadCodeConfig {
            min_confidence: config.get_threshold_float("min_confidence", SourceLanguage::Rust)?,
            ignore_patterns: config.exclusions.patterns.clone(),
            // Note: ignore_files field doesn't exist in DeadCodeConfig
            // The actual field is ignore_patterns which is handled above
            ..Default::default()
        })
    }
}

/// Utility functions for configuration migration
pub struct ConfigMigrationUtils;

impl ConfigMigrationUtils {
    /// Migrate a detector configuration from old format to new format
    pub fn migrate_detector_config(
        detector_name: &str,
        old_config: &dyn std::any::Any,
    ) -> Result<StandardDetectorConfig, UveddiError> {
        match detector_name {
            "god_object" => {
                if let Some(config) = old_config.downcast_ref::<GodObjectConfig>() {
                    config.to_standard()
                } else {
                    Err(UveddiError::config_error(
                        "Invalid god_object configuration type",
                        "config_migration::convert_detector_config",
                    ))
                }
            }
            "code_duplication" => {
                if let Some(config) = old_config.downcast_ref::<DuplicationConfig>() {
                    config.to_standard()
                } else {
                    Err(UveddiError::config_error(
                        "Invalid code_duplication configuration type",
                        "config_migration::convert_detector_config",
                    ))
                }
            }
            "large_classes" => {
                if let Some(config) = old_config.downcast_ref::<LargeClassConfig>() {
                    config.to_standard()
                } else {
                    Err(UveddiError::config_error(
                        "Invalid large_classes configuration type",
                        "config_migration::convert_detector_config",
                    ))
                }
            }
            "dead_code" => {
                if let Some(config) = old_config.downcast_ref::<DeadCodeConfig>() {
                    config.to_standard()
                } else {
                    Err(UveddiError::config_error(
                        "Invalid dead_code configuration type",
                        "config_migration::convert_detector_config",
                    ))
                }
            }
            _ => Err(UveddiError::config_error(
                &format!("Unknown detector type: {}", detector_name),
                "config_migration::convert_detector_config",
            )),
        }
    }

    /// Create a standardized configuration from TOML
    pub fn from_toml(
        detector_name: &str,
        toml_str: &str,
    ) -> Result<StandardDetectorConfig, UveddiError> {
        let config: StandardDetectorConfig = toml::from_str(toml_str).map_err(|e| {
            UveddiError::config_error(
                &format!("TOML parsing error: {}", e),
                "config_migration::from_toml",
            )
        })?;

        // Validate the configuration
        StandardConfigBuilder::new(detector_name)
            .enabled(config.enabled)
            .severity(config.severity.clone())
            .build()?;

        Ok(config)
    }

    /// Convert standardized configuration to TOML
    pub fn to_toml(config: &StandardDetectorConfig) -> Result<String, UveddiError> {
        toml::to_string_pretty(config).map_err(|e| {
            UveddiError::config_error(
                &format!("TOML serialization error: {}", e),
                "config_migration::to_toml",
            )
        })
    }

    /// Validate that a configuration is compatible with a detector
    pub fn validate_compatibility(
        detector_name: &str,
        config: &StandardDetectorConfig,
    ) -> Result<(), UveddiError> {
        match detector_name {
            "god_object" => {
                if !config.thresholds.contains_key("max_methods")
                    && !config
                        .language_overrides
                        .values()
                        .any(|overrides| overrides.contains_key("max_methods"))
                {
                    return Err(UveddiError::config_error(
                        "god_object detector requires max_methods threshold",
                        "config_migration::validate_compatibility",
                    ));
                }
            }
            "code_duplication" => {
                if !config.thresholds.contains_key("similarity_threshold") {
                    return Err(UveddiError::config_error(
                        "code_duplication detector requires similarity_threshold",
                        "config_migration::validate_compatibility",
                    ));
                }
            }
            "large_classes" => {
                if !config.thresholds.contains_key("max_lines")
                    && !config
                        .language_overrides
                        .values()
                        .any(|overrides| overrides.contains_key("max_lines"))
                {
                    return Err(UveddiError::config_error(
                        "large_classes detector requires max_lines threshold",
                        "config_migration::validate_compatibility",
                    ));
                }
            }
            _ => {
                // Unknown detector - basic validation only
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::standardized_config::constants;

    #[test]
    fn test_god_object_migration() {
        let old_config = GodObjectConfig::default();
        let standard_config = old_config.to_standard().expect("Migration should succeed");

        assert!(standard_config.enabled);
        assert_eq!(standard_config.severity, IssueSeverity::High);

        // Test round-trip conversion
        let converted_back = GodObjectConfig::from_standard(&standard_config)
            .expect("Reverse migration should succeed");

        assert!(!converted_back.method_thresholds.is_empty());
        assert!(!converted_back.field_thresholds.is_empty());
    }

    #[test]
    fn test_duplication_config_migration() {
        let old_config = DuplicationConfig::default();
        let standard_config = old_config.to_standard().expect("Migration should succeed");

        assert!(standard_config.enabled);
        assert_eq!(standard_config.severity, IssueSeverity::Medium);

        // Test round-trip conversion
        let converted_back = DuplicationConfig::from_standard(&standard_config)
            .expect("Reverse migration should succeed");

        assert_eq!(converted_back.min_tokens, old_config.min_tokens);
        assert_eq!(
            converted_back.similarity_threshold,
            old_config.similarity_threshold
        );
    }

    #[test]
    fn test_toml_serialization() {
        let config = StandardDetectorConfig::default_for_detector("god_object");
        let toml_str =
            ConfigMigrationUtils::to_toml(&config).expect("TOML serialization should succeed");

        let parsed_config = ConfigMigrationUtils::from_toml("god_object", &toml_str)
            .expect("TOML parsing should succeed");

        assert_eq!(config.enabled, parsed_config.enabled);
        assert_eq!(config.severity, parsed_config.severity);
    }

    #[test]
    fn test_validation() {
        let config = StandardDetectorConfig::default_for_detector("god_object");

        // Should pass validation
        ConfigMigrationUtils::validate_compatibility("god_object", &config)
            .expect("Validation should pass");

        // Should fail validation with empty config
        let empty_config = StandardDetectorConfig::default();
        let result = ConfigMigrationUtils::validate_compatibility("god_object", &empty_config);
        assert!(result.is_err());
    }
}
