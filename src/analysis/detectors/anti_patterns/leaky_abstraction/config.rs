//! Configuration management for leaky abstraction detection.

use crate::analysis::detectors::anti_patterns::leaky_abstraction::types::{
    ArchitecturalConfig, ArchitecturalLayer,
};
use std::collections::{HashMap, HashSet};

/// Configuration builder for architectural analysis.
pub struct ConfigBuilder {
    layer_mappings: HashMap<String, ArchitecturalLayer>,
    infrastructure_modules: HashSet<String>,
    internal_patterns: Vec<String>,
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigBuilder {
    /// Creates a new configuration builder.
    pub fn new() -> Self {
        Self {
            layer_mappings: HashMap::new(),
            infrastructure_modules: HashSet::new(),
            internal_patterns: Vec::new(),
        }
    }

    /// Adds a layer mapping pattern.
    pub fn add_layer_mapping(mut self, pattern: String, layer: ArchitecturalLayer) -> Self {
        self.layer_mappings.insert(pattern, layer);
        self
    }

    /// Adds an infrastructure module.
    pub fn add_infrastructure_module(mut self, module: String) -> Self {
        self.infrastructure_modules.insert(module);
        self
    }

    /// Adds an internal pattern.
    pub fn add_internal_pattern(mut self, pattern: String) -> Self {
        self.internal_patterns.push(pattern);
        self
    }

    /// Builds the final configuration.
    pub fn build(self) -> ArchitecturalConfig {
        ArchitecturalConfig {
            layer_mappings: self.layer_mappings,
            infrastructure_modules: self.infrastructure_modules,
            internal_patterns: self.internal_patterns,
        }
    }
}

/// Provides default configuration based on common project conventions.
pub fn create_default_config() -> ArchitecturalConfig {
    let mut layer_mappings = HashMap::new();

    // Common patterns for different layers
    layer_mappings.insert(
        "**/controllers/**".to_string(),
        ArchitecturalLayer::Presentation,
    );
    layer_mappings.insert("**/views/**".to_string(), ArchitecturalLayer::Presentation);
    layer_mappings.insert("**/ui/**".to_string(), ArchitecturalLayer::Presentation);
    layer_mappings.insert(
        "**/handlers/**".to_string(),
        ArchitecturalLayer::Presentation,
    );

    layer_mappings.insert(
        "**/services/**".to_string(),
        ArchitecturalLayer::Application,
    );
    layer_mappings.insert(
        "**/use_cases/**".to_string(),
        ArchitecturalLayer::Application,
    );
    layer_mappings.insert(
        "**/application/**".to_string(),
        ArchitecturalLayer::Application,
    );

    layer_mappings.insert("**/domain/**".to_string(), ArchitecturalLayer::Domain);
    layer_mappings.insert("**/models/**".to_string(), ArchitecturalLayer::Domain);
    layer_mappings.insert("**/entities/**".to_string(), ArchitecturalLayer::Domain);

    layer_mappings.insert(
        "**/repositories/**".to_string(),
        ArchitecturalLayer::Infrastructure,
    );
    layer_mappings.insert(
        "**/infrastructure/**".to_string(),
        ArchitecturalLayer::Infrastructure,
    );
    layer_mappings.insert(
        "**/adapters/**".to_string(),
        ArchitecturalLayer::Infrastructure,
    );
    layer_mappings.insert(
        "**/external/**".to_string(),
        ArchitecturalLayer::Infrastructure,
    );

    let infrastructure_modules = create_default_infrastructure_modules();
    let internal_patterns = create_default_internal_patterns();

    ArchitecturalConfig {
        layer_mappings,
        infrastructure_modules,
        internal_patterns,
    }
}

/// Creates a default set of infrastructure modules.
fn create_default_infrastructure_modules() -> HashSet<String> {
    let mut infrastructure_modules = HashSet::new();

    // Rust frameworks/ORMs
    infrastructure_modules.insert("diesel".to_string());
    infrastructure_modules.insert("sqlx".to_string());
    infrastructure_modules.insert("sea_orm".to_string());
    infrastructure_modules.insert("tokio".to_string());
    infrastructure_modules.insert("axum".to_string());
    infrastructure_modules.insert("warp".to_string());
    infrastructure_modules.insert("actix_web".to_string());
    infrastructure_modules.insert("reqwest".to_string());

    // Python frameworks/ORMs
    infrastructure_modules.insert("django".to_string());
    infrastructure_modules.insert("flask".to_string());
    infrastructure_modules.insert("fastapi".to_string());
    infrastructure_modules.insert("sqlalchemy".to_string());
    infrastructure_modules.insert("requests".to_string());
    infrastructure_modules.insert("psycopg2".to_string());

    // JavaScript/TypeScript frameworks
    infrastructure_modules.insert("express".to_string());
    infrastructure_modules.insert("prisma".to_string());
    infrastructure_modules.insert("mongoose".to_string());
    infrastructure_modules.insert("axios".to_string());
    infrastructure_modules.insert("react".to_string());
    infrastructure_modules.insert("vue".to_string());

    infrastructure_modules
}

/// Creates a default set of internal patterns.
fn create_default_internal_patterns() -> Vec<String> {
    vec![
        "internal".to_string(),
        "impl".to_string(),
        "detail".to_string(),
        "_internal".to_string(),
        "private".to_string(),
    ]
}

/// Utility functions for configuration validation and management.
pub mod utils {
    use super::*;

    /// Validates that a configuration is properly set up.
    pub fn validate_config(config: &ArchitecturalConfig) -> Result<(), String> {
        if config.layer_mappings.is_empty() {
            return Err("No layer mappings defined".to_string());
        }

        if config.infrastructure_modules.is_empty() {
            return Err("No infrastructure modules defined".to_string());
        }

        Ok(())
    }

    /// Merges two configurations, with the second taking precedence.
    pub fn merge_configs(
        base: ArchitecturalConfig,
        override_config: ArchitecturalConfig,
    ) -> ArchitecturalConfig {
        let mut merged_layer_mappings = base.layer_mappings;
        for (pattern, layer) in override_config.layer_mappings {
            merged_layer_mappings.insert(pattern, layer);
        }

        let mut merged_infrastructure_modules = base.infrastructure_modules;
        for module in override_config.infrastructure_modules {
            merged_infrastructure_modules.insert(module);
        }

        let mut merged_internal_patterns = base.internal_patterns;
        merged_internal_patterns.extend(override_config.internal_patterns);

        ArchitecturalConfig {
            layer_mappings: merged_layer_mappings,
            infrastructure_modules: merged_infrastructure_modules,
            internal_patterns: merged_internal_patterns,
        }
    }
}
