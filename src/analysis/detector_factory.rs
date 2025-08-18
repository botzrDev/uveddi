use crate::analysis::detectors::anti_patterns::code_duplication::CodeDuplicationDetector;
// use crate::analysis::detectors::anti_patterns::data_clumps::DataClumpsDetector;
use crate::analysis::detectors::anti_patterns::dead_code::DeadCodeDetector;
// use crate::analysis::detectors::anti_patterns::feature_envy::FeatureEnvyDetector;
use crate::analysis::detectors::anti_patterns::god_object::GodObjectDetector;
use crate::analysis::detectors::anti_patterns::large_classes::LargeClassDetector;
use crate::analysis::detectors::anti_patterns::long_methods::LongMethodsDetector;
use crate::analysis::detectors::anti_patterns::magic_values::MagicValuesDetector;
// use crate::analysis::detectors::anti_patterns::shotgun_surgery::ShotgunSurgeryDetector;
use crate::analysis::detectors::anti_patterns::tight_coupling::TightCouplingDetector;
use crate::analysis::AnalysisDetector;
use crate::error::UveddiError;
use std::collections::HashMap;

/// Factory for creating detector instances
///
/// The `DetectorFactory` provides methods for creating both default detector sets
/// and custom detectors based on configuration. This enables a centralized way
/// to manage detector creation and configuration.
pub struct DetectorFactory;

impl DetectorFactory {
    /// Create default detector set for backward compatibility
    ///
    /// Returns a vector containing the standard set of detectors that were
    /// previously hardcoded in the AnalysisEngine constructor. This ensures
    /// backward compatibility while enabling dependency injection.
    ///
    /// # Returns
    ///
    /// A vector of boxed detectors implementing the `AnalysisDetector` trait
    pub fn create_default_detectors() -> Vec<Box<dyn AnalysisDetector + Send + Sync>> {
        vec![
            // Original detectors
            Box::new(GodObjectDetector::new(5, 8)),
            Box::new(CodeDuplicationDetector::new()),
            Box::new(DeadCodeDetector::with_default_config()),
            Box::new(LargeClassDetector::with_default_config()),
            Box::new(TightCouplingDetector::default()),
            // Enhanced anti-pattern detectors
            // Box::new(ShotgunSurgeryDetector::new()),
            // Box::new(FeatureEnvyDetector::new()),
            // Box::new(DataClumpsDetector::new()),
            Box::new(LongMethodsDetector::default()),
            Box::new(MagicValuesDetector::default()),
        ]
    }

    /// Create a new factory instance with registry support
    ///
    /// Creates a factory instance that can be used to create detectors
    /// with enhanced configuration support.
    pub fn new() -> Self {
        Self
    }

    /// Create detector by name with configuration (ENHANCED)
    ///
    /// Creates a specific detector instance based on the provided name and
    /// configuration parameters. This enables dynamic detector creation
    /// from configuration files with enhanced parameter support.
    ///
    /// # Arguments
    ///
    /// * `name` - The detector type name (e.g., "god_object", "code_duplication")
    /// * `config` - Configuration parameters for the detector
    ///
    /// # Returns
    ///
    /// A boxed detector instance or an error if the detector type is unknown
    ///
    /// # Errors
    ///
    /// Returns `UveddiError::ConfigError` if the detector name is not recognized
    pub fn create_detector(
        name: &str,
        config: &DetectorConfig,
    ) -> Result<Box<dyn AnalysisDetector + Send + Sync>, UveddiError> {
        match name {
            "god_object" => Ok(Box::new(GodObjectDetector::new(
                config.get("threshold_methods").unwrap_or(5) as usize,
                config.get("threshold_fields").unwrap_or(8) as usize,
            ))),
            "code_duplication" => Ok(Box::new(CodeDuplicationDetector::new())),
            "dead_code" => Ok(Box::new(DeadCodeDetector::with_default_config())),
            "large_classes" => Ok(Box::new(LargeClassDetector::with_default_config())),
            "tight_coupling" => Ok(Box::new(TightCouplingDetector::default())),
            // "shotgun_surgery" => Ok(Box::new(ShotgunSurgeryDetector::new())),
            // "feature_envy" => Ok(Box::new(FeatureEnvyDetector::new())),
            // "data_clumps" => Ok(Box::new(DataClumpsDetector::new())),
            "long_methods" => Ok(Box::new(LongMethodsDetector::default())),
            "magic_values" => Ok(Box::new(MagicValuesDetector::default())),
            _ => Err(UveddiError::config_error(
                &format!("Unknown detector: {}", name),
                "detector factory",
            )),
        }
    }

    /// Create detector by name with enhanced configuration (DEPENDENCY INJECTION)
    ///
    /// Creates a detector instance with enhanced configuration support including
    /// severity levels, thresholds, and other advanced settings.
    ///
    /// # Arguments
    ///
    /// * `name` - The detector type name
    /// * `config` - Enhanced configuration with support for severity, thresholds, etc.
    ///
    /// # Returns
    ///
    /// A configured detector instance
    ///
    /// # Errors
    ///
    /// Returns UveddiError if detector creation fails
    pub fn create_detector_enhanced(
        &self,
        name: &str,
        config: &EnhancedDetectorConfig,
    ) -> Result<Box<dyn AnalysisDetector + Send + Sync>, UveddiError> {
        match name {
            "god_object" => {
                let detector = GodObjectDetector::new(
                    config.thresholds.max_methods.unwrap_or(20) as usize,
                    config.thresholds.max_fields.unwrap_or(15) as usize,
                );
                Ok(Box::new(detector))
            }
            "code_duplication" => Ok(Box::new(CodeDuplicationDetector::new())),
            "dead_code" => Ok(Box::new(DeadCodeDetector::with_default_config())),
            "large_classes" => Ok(Box::new(LargeClassDetector::with_default_config())),
            "tight_coupling" => Ok(Box::new(TightCouplingDetector::default())),
            _ => Err(UveddiError::config_error(
                &format!("Unknown detector: {}", name),
                "detector factory",
            )),
        }
    }

    /// Get list of available detector names
    ///
    /// Returns all detector types that can be created by this factory.
    ///
    /// # Returns
    ///
    /// Vector of detector names
    pub fn available_detectors(&self) -> Vec<String> {
        vec![
            "god_object".to_string(),
            "code_duplication".to_string(),
            "dead_code".to_string(),
            "large_classes".to_string(),
            "tight_coupling".to_string(),
        ]
    }

    /// Create detectors from a configuration map
    ///
    /// Creates multiple detectors based on a configuration mapping. This is
    /// useful for loading detector configurations from files.
    ///
    /// # Arguments
    ///
    /// * `configs` - A mapping of detector names to their configurations
    ///
    /// # Returns
    ///
    /// A vector of created detectors or an error if any detector creation fails
    pub fn create_detectors_from_config(
        configs: &HashMap<String, DetectorConfig>,
    ) -> Result<Vec<Box<dyn AnalysisDetector + Send + Sync>>, UveddiError> {
        let mut detectors = Vec::new();

        for (name, config) in configs {
            let detector = Self::create_detector(name, config)?;
            detectors.push(detector);
        }

        Ok(detectors)
    }
}

/// Configuration structure for detectors
///
/// Holds configuration parameters for detector instances. Parameters are
/// stored as key-value pairs where keys are parameter names and values
/// are integer configuration values.
#[derive(Debug, Clone, Default)]
pub struct DetectorConfig {
    params: HashMap<String, i32>,
}

impl DetectorConfig {
    /// Create a new empty configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a parameter to the configuration
    ///
    /// # Arguments
    ///
    /// * `key` - The parameter name
    /// * `value` - The parameter value
    ///
    /// # Returns
    ///
    /// The modified configuration for method chaining
    pub fn with_param(mut self, key: &str, value: i32) -> Self {
        self.params.insert(key.to_string(), value);
        self
    }

    /// Get a parameter value
    ///
    /// # Arguments
    ///
    /// * `key` - The parameter name to retrieve
    ///
    /// # Returns
    ///
    /// The parameter value if it exists, otherwise None
    pub fn get(&self, key: &str) -> Option<i32> {
        self.params.get(key).copied()
    }

    /// Set a parameter value
    ///
    /// # Arguments
    ///
    /// * `key` - The parameter name
    /// * `value` - The parameter value
    pub fn set(&mut self, key: &str, value: i32) {
        self.params.insert(key.to_string(), value);
    }

    /// Check if a parameter exists
    ///
    /// # Arguments
    ///
    /// * `key` - The parameter name to check
    ///
    /// # Returns
    ///
    /// True if the parameter exists, false otherwise
    pub fn has_param(&self, key: &str) -> bool {
        self.params.contains_key(key)
    }

    /// Get all parameters for serialization
    pub fn params(&self) -> &HashMap<String, i32> {
        &self.params
    }
}

/// Enhanced detector configuration for dependency injection
///
/// Provides more sophisticated configuration options including severity levels,
/// threshold configurations, and feature flags.
#[derive(Debug, Clone)]
pub struct EnhancedDetectorConfig {
    pub enabled: bool,
    pub severity: IssueSeverity,
    pub thresholds: DetectorThresholds,
}

/// Issue severity levels for detector configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl Default for IssueSeverity {
    fn default() -> Self {
        Self::Medium
    }
}

/// Detector threshold configuration
///
/// Centralized configuration for various detector thresholds and limits.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct DetectorThresholds {
    pub max_methods: Option<u32>,
    pub max_fields: Option<u32>,
    pub max_lines: Option<u32>,
    pub min_similarity: Option<f64>,
    pub complexity_threshold: Option<u32>,
    pub coupling_threshold: Option<u32>,
}

impl EnhancedDetectorConfig {
    /// Create a new enhanced configuration with defaults
    pub fn new() -> Self {
        Self {
            enabled: true,
            severity: IssueSeverity::Medium,
            thresholds: DetectorThresholds::default(),
        }
    }

    /// Set enabled status
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set severity level
    pub fn with_severity(mut self, severity: IssueSeverity) -> Self {
        self.severity = severity;
        self
    }

    /// Set maximum methods threshold
    pub fn with_max_methods(mut self, max_methods: u32) -> Self {
        self.thresholds.max_methods = Some(max_methods);
        self
    }

    /// Set maximum fields threshold
    pub fn with_max_fields(mut self, max_fields: u32) -> Self {
        self.thresholds.max_fields = Some(max_fields);
        self
    }

    /// Set maximum lines threshold
    pub fn with_max_lines(mut self, max_lines: u32) -> Self {
        self.thresholds.max_lines = Some(max_lines);
        self
    }

    /// Set minimum similarity threshold
    pub fn with_min_similarity(mut self, min_similarity: f64) -> Self {
        self.thresholds.min_similarity = Some(min_similarity);
        self
    }
}

impl Default for EnhancedDetectorConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_default_detectors() {
        let detectors = DetectorFactory::create_default_detectors();
        assert_eq!(detectors.len(), 5);

        // Verify each detector type is present
        let detector_names: Vec<&str> = detectors.iter().map(|d| d.get_detector_name()).collect();

        assert!(detector_names.contains(&"GodObjectDetector"));
        assert!(detector_names.contains(&"CodeDuplicationDetector"));
        assert!(detector_names.contains(&"DeadCodeDetector"));
        assert!(detector_names.contains(&"LargeClassDetector"));
        assert!(detector_names.contains(&"TightCouplingDetector"));
    }

    #[test]
    fn test_create_detector_by_name() {
        let config = DetectorConfig::new()
            .with_param("threshold_methods", 10)
            .with_param("threshold_fields", 15);

        let detector = DetectorFactory::create_detector("god_object", &config);
        assert!(detector.is_ok());
        assert_eq!(detector.unwrap().get_detector_name(), "GodObjectDetector");
    }

    #[test]
    fn test_create_detector_unknown_name() {
        let config = DetectorConfig::new();
        let detector = DetectorFactory::create_detector("unknown_detector", &config);
        assert!(detector.is_err());
    }

    #[test]
    fn test_detector_config() {
        let mut config = DetectorConfig::new();

        assert!(!config.has_param("test_param"));
        assert_eq!(config.get("test_param"), None);

        config.set("test_param", 42);
        assert!(config.has_param("test_param"));
        assert_eq!(config.get("test_param"), Some(42));

        let config2 = DetectorConfig::new()
            .with_param("param1", 1)
            .with_param("param2", 2);

        assert_eq!(config2.get("param1"), Some(1));
        assert_eq!(config2.get("param2"), Some(2));
    }

    #[test]
    fn test_create_detectors_from_config() {
        let mut configs = HashMap::new();
        configs.insert(
            "god_object".to_string(),
            DetectorConfig::new().with_param("threshold_methods", 10),
        );
        configs.insert("code_duplication".to_string(), DetectorConfig::new());

        let detectors = DetectorFactory::create_detectors_from_config(&configs);
        assert!(detectors.is_ok());
        assert_eq!(detectors.unwrap().len(), 2);
    }
}
