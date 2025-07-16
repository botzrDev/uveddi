//! Demonstration of the new standardized configuration system
//!
//! This example shows how to use the new standardized configuration patterns
//! that replace the inconsistent detector configurations across the codebase.

use std::collections::HashMap;
use uveddi::analysis::{
    StandardConfigBuilder, StandardDetectorConfig, AnalysisConfig,
    IssueSeverity, ConfigValue, ExclusionConfig
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Uveddi Standardized Configuration Demo");
    println!("==========================================\n");

    // 1. Create standardized configurations using the builder pattern
    println!("1. Creating standardized detector configurations...");
    
    let god_object_config = StandardConfigBuilder::new("god_object")
        .enabled(true)
        .severity(IssueSeverity::High)
        .threshold("max_methods", 25)
        .threshold("max_fields", 20)
        .language_threshold("rust", "max_methods", 30)
        .language_threshold("python", "max_methods", 20)
        .exclude_pattern("*_test.rs")
        .exclude_framework("serde")
        .description("Detects classes with too many responsibilities")
        .build()?;

    let duplication_config = StandardConfigBuilder::new("code_duplication")
        .enabled(true)
        .severity(IssueSeverity::Medium)
        .threshold("min_tokens", 50)
        .threshold("similarity_threshold", 0.85)
        .threshold("ignore_identifiers", true)
        .exclude_pattern("*_generated.rs")
        .build()?;

    let large_class_config = StandardConfigBuilder::new("large_classes")
        .enabled(true)
        .severity(IssueSeverity::Medium)
        .threshold("max_lines", 400)
        .threshold("max_methods", 25)
        .language_threshold("javascript", "max_lines", 300)
        .build()?;

    println!("✅ Created {} detector configurations", 3);

    // 2. Demonstrate configuration access patterns
    println!("\n2. Demonstrating configuration access patterns...");
    
    // Language-specific threshold access
    use uveddi::ast::tree_sitter_impl::SourceLanguage;
    let rust_methods = god_object_config.get_threshold_int("max_methods", SourceLanguage::Rust)?;
    let python_methods = god_object_config.get_threshold_int("max_methods", SourceLanguage::Python)?;
    let default_methods = god_object_config.get_threshold_int("max_methods", SourceLanguage::JavaScript)?;
    
    println!("God Object max_methods thresholds:");
    println!("  - Rust: {}", rust_methods);
    println!("  - Python: {}", python_methods);
    println!("  - JavaScript (default): {}", default_methods);

    // 3. Create a complete analysis configuration
    println!("\n3. Creating complete analysis configuration...");
    
    let mut analysis_config = AnalysisConfig::default();
    
    // Add standardized detector configurations
    analysis_config.set_standard_detector_config("god_object", god_object_config);
    analysis_config.set_standard_detector_config("code_duplication", duplication_config);
    analysis_config.set_standard_detector_config("large_classes", large_class_config);

    // Add a dead code detector with minimal configuration
    let dead_code_config = StandardDetectorConfig::default_for_detector("dead_code");
    analysis_config.set_standard_detector_config("dead_code", dead_code_config);

    println!("✅ Analysis configuration created with {} detectors", 
             analysis_config.standard_detectors.len());

    // 4. Demonstrate configuration migration
    println!("\n4. Demonstrating configuration migration...");
    
    let migrated_config = analysis_config.migrate_to_standardized()?;
    println!("✅ Successfully migrated configuration");
    
    let enabled_detectors = migrated_config.get_enabled_detectors();
    println!("Enabled detectors: {:?}", enabled_detectors);

    // 5. Export to TOML format
    println!("\n5. Exporting to TOML format...");
    
    let toml_output = migrated_config.to_standardized_toml()?;
    println!("TOML Configuration Preview:");
    println!("{}", &toml_output[..std::cmp::min(500, toml_output.len())]);
    if toml_output.len() > 500 {
        println!("... (truncated)");
    }

    // 6. Demonstrate effective configuration resolution
    println!("\n6. Demonstrating effective configuration resolution...");
    
    let effective_god_object = analysis_config.get_effective_detector_config("god_object");
    let effective_unknown = analysis_config.get_effective_detector_config("unknown_detector");
    
    println!("Effective god_object config enabled: {}", effective_god_object.enabled);
    println!("Effective unknown detector config enabled: {}", effective_unknown.enabled);

    // 7. Show configuration constants
    println!("\n7. Available configuration constants:");
    use uveddi::analysis::constants;
    
    println!("God Object defaults:");
    println!("  - Default max methods: {}", constants::god_object::DEFAULT_MAX_METHODS);
    println!("  - Default max fields: {}", constants::god_object::DEFAULT_MAX_FIELDS);
    println!("  - Rust max methods: {}", constants::god_object::RUST_MAX_METHODS);
    
    println!("\nCode Duplication defaults:");
    println!("  - Default min tokens: {}", constants::code_duplication::DEFAULT_MIN_TOKENS);
    println!("  - Default similarity: {}", constants::code_duplication::DEFAULT_SIMILARITY_THRESHOLD);

    println!("\n🎉 Standardized configuration demo completed successfully!");
    println!("\nKey Benefits Demonstrated:");
    println!("✅ Unified configuration structure across all detectors");
    println!("✅ Language-specific threshold overrides");
    println!("✅ Fluent builder pattern for easy configuration");
    println!("✅ TOML serialization/deserialization support");
    println!("✅ Configuration validation with helpful error messages");
    println!("✅ Backward compatibility with migration utilities");
    println!("✅ Extracted configuration constants for consistency");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standardized_config_creation() {
        let config = StandardConfigBuilder::new("god_object")
            .enabled(true)
            .severity(IssueSeverity::High)
            .threshold("max_methods", 20)
            .build()
            .expect("Config creation should succeed");

        assert!(config.enabled);
        assert_eq!(config.severity, IssueSeverity::High);
        assert!(config.thresholds.contains_key("max_methods"));
    }

    #[test]
    fn test_language_specific_thresholds() {
        let config = StandardConfigBuilder::new("god_object")
            .threshold("max_methods", 20)
            .language_threshold("rust", "max_methods", 25)
            .build()
            .expect("Config creation should succeed");

        use uveddi::ast::tree_sitter_impl::SourceLanguage;
        
        let rust_threshold = config.get_threshold_int("max_methods", SourceLanguage::Rust)
            .expect("Rust threshold should exist");
        let python_threshold = config.get_threshold_int("max_methods", SourceLanguage::Python)
            .expect("Python threshold should fall back to default");

        assert_eq!(rust_threshold, 25);
        assert_eq!(python_threshold, 20); // Falls back to default
    }

    #[test]
    fn test_configuration_validation() {
        // Valid configuration
        let valid_result = StandardConfigBuilder::new("god_object")
            .threshold("max_methods", 20)
            .threshold("max_fields", 15)
            .build();
        assert!(valid_result.is_ok());

        // Invalid configuration (missing required threshold)
        let invalid_result = StandardConfigBuilder::new("god_object")
            .threshold("max_methods", 20)
            // Missing max_fields
            .build();
        assert!(invalid_result.is_err());
    }

    #[test]
    fn test_analysis_config_integration() {
        let mut analysis_config = AnalysisConfig::default();
        
        let detector_config = StandardConfigBuilder::new("god_object")
            .enabled(true)
            .threshold("max_methods", 30)
            .threshold("max_fields", 20)
            .build()
            .expect("Config creation should succeed");

        analysis_config.set_standard_detector_config("god_object", detector_config);

        let effective_config = analysis_config.get_effective_detector_config("god_object");
        assert!(effective_config.enabled);
        
        let enabled_detectors = analysis_config.get_enabled_detectors();
        assert!(enabled_detectors.contains(&"god_object".to_string()));
    }
}