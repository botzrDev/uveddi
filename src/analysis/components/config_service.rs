//! Configuration service implementation
//!
//! Provides centralized access to all analysis configuration settings.

use super::traits::ConfigurationService as ConfigurationServiceTrait;
use std::collections::HashMap;
use std::path::PathBuf;

/// Configuration service implementation
pub struct ConfigurationService {
    config_values: HashMap<String, String>,
    enabled_detectors: HashMap<String, bool>,
    plugin_config: HashMap<String, serde_json::Value>,
    cache_path: Option<PathBuf>,
    plugins_enabled: bool,
}

impl ConfigurationService {
    /// Creates a new configuration service with default settings
    pub fn new() -> Self {
        let mut config = Self {
            config_values: HashMap::new(),
            enabled_detectors: HashMap::new(),
            plugin_config: HashMap::new(),
            cache_path: Some(PathBuf::from("uveddi_cache.db")),
            plugins_enabled: false,
        };

        // Initialize with default detector settings
        config.set_default_detector_config();
        config
    }

    /// Creates a configuration service from an AnalysisConfig
    pub fn from_analysis_config(config: &crate::analysis::AnalysisConfig) -> Self {
        let mut service = Self::new();

        // Apply config settings
        service.cache_path = config.cache_path.as_ref().map(PathBuf::from);
        service.plugins_enabled = config.enable_plugins;

        // Configure detectors
        for (detector_name, detector_config) in &config.detectors {
            let enabled = detector_config.get("enabled").unwrap_or(1) == 1;
            service
                .enabled_detectors
                .insert(detector_name.clone(), enabled);

            // Store detector-specific configuration
            if let Ok(json_value) = serde_json::to_value(detector_config) {
                service
                    .plugin_config
                    .insert(detector_name.clone(), json_value);
            }
        }

        service
    }

    /// Sets default detector configuration
    fn set_default_detector_config(&mut self) {
        // Enable all default detectors
        let default_detectors = vec![
            "god_object",
            "large_class",
            "dead_code",
            "cyclic_dependency",
            "tight_coupling",
            "long_method",
            "magic_values",
        ];

        for detector in default_detectors {
            self.enabled_detectors.insert(detector.to_string(), true);
        }
    }

    /// Sets a configuration value
    pub fn set_config_value(&mut self, key: String, value: String) {
        self.config_values.insert(key, value);
    }

    /// Enables or disables a detector
    pub fn set_detector_enabled(&mut self, detector_name: String, enabled: bool) {
        self.enabled_detectors.insert(detector_name, enabled);
    }

    /// Sets plugin configuration
    pub fn set_plugin_config(&mut self, plugin_name: String, config: serde_json::Value) {
        self.plugin_config.insert(plugin_name, config);
    }

    /// Sets the cache path
    pub fn set_cache_path(&mut self, path: Option<PathBuf>) {
        self.cache_path = path;
    }

    /// Enables or disables plugins globally
    pub fn set_plugins_enabled(&mut self, enabled: bool) {
        self.plugins_enabled = enabled;
    }
}

impl ConfigurationServiceTrait for ConfigurationService {
    fn get_config_value(&self, key: &str) -> Option<String> {
        self.config_values.get(key).cloned()
    }

    fn is_detector_enabled(&self, detector_name: &str) -> bool {
        self.enabled_detectors
            .get(detector_name)
            .copied()
            .unwrap_or(false)
    }

    fn get_plugin_config(&self) -> HashMap<String, serde_json::Value> {
        self.plugin_config.clone()
    }

    fn get_cache_path(&self) -> Option<PathBuf> {
        self.cache_path.clone()
    }

    fn are_plugins_enabled(&self) -> bool {
        self.plugins_enabled
    }
}

impl Default for ConfigurationService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_configuration() {
        let config_service = ConfigurationService::new();

        // Test default detector settings
        assert!(config_service.is_detector_enabled("god_object"));
        assert!(config_service.is_detector_enabled("dead_code"));
        assert!(!config_service.are_plugins_enabled());
        assert!(config_service.get_cache_path().is_some());
    }

    #[test]
    fn test_configuration_mutations() {
        let mut config_service = ConfigurationService::new();

        // Test setting configuration values
        config_service.set_config_value("test_key".to_string(), "test_value".to_string());
        assert_eq!(
            config_service.get_config_value("test_key"),
            Some("test_value".to_string())
        );

        // Test detector configuration
        config_service.set_detector_enabled("custom_detector".to_string(), true);
        assert!(config_service.is_detector_enabled("custom_detector"));

        // Test plugin settings
        config_service.set_plugins_enabled(true);
        assert!(config_service.are_plugins_enabled());
    }

    #[test]
    fn test_from_analysis_config() {
        use crate::analysis::{AnalysisConfig, DetectorConfig};
        use std::collections::HashMap;

        let mut analysis_config = AnalysisConfig::default();
        analysis_config.enable_plugins = true;
        analysis_config.cache_path = Some("custom_cache.db".to_string());

        // Add a detector configuration
        let detector_config = DetectorConfig::new()
            .with_param("enabled", 1)
            .with_param("threshold", 10);
        analysis_config
            .detectors
            .insert("test_detector".to_string(), detector_config);

        let config_service = ConfigurationService::from_analysis_config(&analysis_config);

        assert!(config_service.are_plugins_enabled());
        assert_eq!(
            config_service.get_cache_path(),
            Some(PathBuf::from("custom_cache.db"))
        );
        assert!(config_service.is_detector_enabled("test_detector"));
    }
}
