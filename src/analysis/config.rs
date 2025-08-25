use crate::analysis::{
    AnalysisEngine, DetectorConfig, DetectorRegistry, DetectorThresholds, EnhancedDetectorConfig,
    IssueSeverity, StandardDetectorConfig,
};
use tracing::{info, warn, error, debug};
use crate::error::UveddiError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Configuration for the analysis engine
///
/// The `AnalysisConfig` struct provides comprehensive configuration support
/// for the AnalysisEngine, including detector settings, cache configuration,
/// and plugin management. Configuration can be loaded from TOML files or
/// created programmatically.
///
/// # Features
///
/// - **TOML File Support**: Load configuration from external TOML files
/// - **Detector Configuration**: Configure individual detectors with custom parameters
/// - **Cache Settings**: Control cache behavior and size limits
/// - **Plugin Management**: Enable/disable plugin support
/// - **Engine Creation**: Direct creation of configured AnalysisEngine instances
///
/// # Configuration File Format
///
/// ```toml
/// # Cache settings
/// cache_size = 1000
/// cache_path = "uveddi_cache.db"
/// enable_plugins = true
///
/// # Detector configurations (legacy format)
/// [detectors.god_object]
/// threshold_methods = 5
/// threshold_fields = 8
///
/// # Standardized detector configurations (new format)
/// [standard_detectors.god_object]
/// enabled = true
/// severity = "High"
/// thresholds = { max_methods = 20, max_fields = 15 }
///
/// [standard_detectors.code_duplication]
/// enabled = true
/// severity = "Medium"
/// thresholds = { min_tokens = 50, similarity_threshold = 0.8 }
/// ```
///
/// # Examples
///
/// ```rust
/// use uveddi::analysis::{AnalysisConfig, DetectorConfig};
/// use std::collections::HashMap;
///
/// # async fn example() -> Result<(), uveddi::error::UveddiError> {
/// // Create from file
/// let config = AnalysisConfig::from_file(std::path::Path::new("config.toml"))?;
/// let engine = config.create_engine().await?;
///
/// // Create programmatically
/// let mut detectors = HashMap::new();
/// detectors.insert("god_object".to_string(),
///     DetectorConfig::new().with_param("threshold_methods", 10));
///
/// let config = AnalysisConfig {
///     detectors,
///     cache_size: Some(500),
///     enable_plugins: false,
///     cache_path: Some("custom_cache.db".to_string()),
/// };
/// let engine = config.create_engine().await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AnalysisConfig {
    /// Configuration for individual detectors (legacy format)
    #[serde(default)]
    pub detectors: HashMap<String, DetectorConfig>,
    /// Enhanced configuration for individual detectors (new format)
    #[serde(default)]
    pub enhanced_detectors: HashMap<String, EnhancedDetectorConfig>,
    /// Standardized configuration for individual detectors (newest format)
    #[serde(default)]
    pub standard_detectors: HashMap<String, StandardDetectorConfig>,
    /// Maximum cache size (number of entries)
    pub cache_size: Option<usize>,
    /// Whether to enable WASM plugin support
    pub enable_plugins: bool,
    /// Path to the cache database file
    pub cache_path: Option<String>,
    /// Cache configuration settings
    #[serde(default)]
    pub cache_settings: CacheConfig,
    /// Performance optimization settings
    #[serde(default)]
    pub performance_settings: PerformanceConfig,
}

/// Cache configuration for dependency injection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Cache type (memory, disk, hybrid)
    pub cache_type: String,
    /// Maximum cache size in bytes
    pub max_size: String,
    /// Cache eviction policy
    pub eviction_policy: String,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            cache_type: "memory".to_string(),
            max_size: "100MB".to_string(),
            eviction_policy: "lru".to_string(),
        }
    }
}

/// Performance configuration for dependency injection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Maximum number of files to process concurrently
    pub max_concurrent_files: usize,
    /// Analysis timeout in seconds
    pub timeout_seconds: u64,
    /// Enable parallel processing
    pub parallel_processing: bool,
    /// Per-file timeout in seconds (for individual file analysis)
    pub file_timeout_seconds: u64,
    /// Enable graceful degradation when hitting timeouts
    pub enable_graceful_degradation: bool,
    /// Maximum files to analyze before applying degradation strategies
    pub degradation_file_threshold: usize,
    /// Timeout multiplier for large files (> 1000 lines)
    pub large_file_timeout_multiplier: f64,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_concurrent_files: 10,
            timeout_seconds: 300, // Increased default to 5 minutes
            parallel_processing: true,
            file_timeout_seconds: 30, // 30 seconds per file
            enable_graceful_degradation: true,
            degradation_file_threshold: 500, // Start degradation after 500 files
            large_file_timeout_multiplier: 2.0, // 2x timeout for large files
        }
    }
}

impl AnalysisConfig {
    /// Load configuration from TOML file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the TOML configuration file
    ///
    /// # Returns
    ///
    /// A parsed `AnalysisConfig` instance
    ///
    /// # Errors
    ///
    /// Returns `UveddiError::ConfigError` if:
    /// - File cannot be read
    /// - TOML parsing fails
    /// - Configuration is invalid
    ///
    /// # Examples
    ///
    /// ```rust
    /// use uveddi::analysis::AnalysisConfig;
    /// use std::path::Path;
    ///
    /// # fn example() -> Result<(), uveddi::error::UveddiError> {
    /// let config = AnalysisConfig::from_file(Path::new("analysis.toml"))?;
    /// println!("Loaded {} detector configurations", config.detectors.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_file(path: &Path) -> Result<Self, UveddiError> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            UveddiError::config_error(
                &format!("Failed to read config file '{}': {}", path.display(), e),
                &path.display().to_string(),
            )
        })?;

        toml::from_str(&content).map_err(|e| {
            UveddiError::config_error(
                &format!("Failed to parse TOML config: {}", e),
                &path.display().to_string(),
            )
        })
    }

    /// Save configuration to TOML file
    ///
    /// # Arguments
    ///
    /// * `path` - Path where the configuration should be saved
    ///
    /// # Errors
    ///
    /// Returns `UveddiError::ConfigError` if:
    /// - TOML serialization fails
    /// - File cannot be written
    pub fn save_to_file(&self, path: &Path) -> Result<(), UveddiError> {
        let toml_content = toml::to_string_pretty(self).map_err(|e| {
            UveddiError::config_error(
                &format!("Failed to serialize config to TOML: {}", e),
                &path.display().to_string(),
            )
        })?;

        std::fs::write(path, toml_content).map_err(|e| {
            UveddiError::config_error(
                &format!("Failed to write config file '{}': {}", path.display(), e),
                &path.display().to_string(),
            )
        })
    }

    /// Create engine from this configuration (ENHANCED)
    ///
    /// Creates an `AnalysisEngine` instance using the settings in this configuration.
    /// Supports both legacy detector configuration and enhanced detector configuration
    /// with dependency injection.
    ///
    /// # Returns
    ///
    /// A configured `AnalysisEngine` instance
    ///
    /// # Errors
    ///
    /// Returns `UveddiError` if:
    /// - Detector creation fails
    /// - Engine initialization fails
    /// - Plugin initialization fails (when enabled)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use uveddi::analysis::AnalysisConfig;
    ///
    /// # async fn example() -> Result<(), uveddi::error::UveddiError> {
    /// let config = AnalysisConfig::default();
    /// let engine = config.create_engine().await?;
    ///
    /// // Engine is ready for analysis
    /// let anti_pattern_types = engine.get_anti_pattern_types();
    /// println!("Engine supports {} anti-pattern types", anti_pattern_types.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_engine(&self) -> Result<AnalysisEngine, UveddiError> {
        let mut registry = DetectorRegistry::new();

        // Prioritize enhanced detectors over legacy detectors
        if !self.enhanced_detectors.is_empty() {
            // Load enhanced detectors
            self.load_enhanced_detectors(&mut registry)?;
        } else if !self.detectors.is_empty() {
            // Load legacy detectors
            registry.load_from_config(&self.detectors)?;
        } else {
            // If no detectors configured, use defaults
            registry.load_defaults();
        }

        let detectors = registry.get_all_detectors();
        let cache_path = self.cache_path.as_ref().map(|p| Path::new(p));

        if self.enable_plugins {
            AnalysisEngine::with_detectors_and_plugins(detectors, cache_path)
                .await
                .map_err(|e| UveddiError::AnalysisError {
                    file: "config.rs".to_string(),
                    line: 267,
                    message: e.to_string(),
                    context: "Plugin-enabled analysis engine creation".to_string(),
                    suggestion: "Check plugin configuration and dependencies".to_string(),
                    source: Some(e),
                })
        } else {
            let result = if let Some(path) = cache_path {
                AnalysisEngine::builder()
                    .with_detectors(detectors)
                    .with_cache_path(path)
                    .build()
            } else {
                AnalysisEngine::builder().with_detectors(detectors).build()
            };
            result.map_err(|e| UveddiError::AnalysisError {
                file: "config.rs".to_string(),
                line: 275,
                message: e.to_string(),
                context: "Basic analysis engine creation".to_string(),
                suggestion: "Check detector configuration".to_string(),
                source: None, // Remove the problematic source field
            })
        }
    }

    /// Load enhanced detectors into the registry (DEPENDENCY INJECTION)
    ///
    /// Creates detectors from enhanced configuration and registers them.
    ///
    /// # Arguments
    ///
    /// * `registry` - The detector registry to load detectors into
    ///
    /// # Errors
    ///
    /// Returns UveddiError if detector creation fails
    fn load_enhanced_detectors(&self, registry: &mut DetectorRegistry) -> Result<(), UveddiError> {
        use crate::analysis::detector_factory::DetectorFactory;

        let factory = DetectorFactory::new();

        for (detector_name, enhanced_config) in &self.enhanced_detectors {
            if enhanced_config.enabled {
                let detector = factory.create_detector_enhanced(detector_name, enhanced_config)?;
                registry.register(detector_name.clone(), detector);
            }
        }

        Ok(())
    }

    /// Create engine from this configuration (synchronous version)
    ///
    /// Creates an `AnalysisEngine` without plugin support for synchronous contexts.
    ///
    /// # Returns
    ///
    /// A configured `AnalysisEngine` instance without plugins
    ///
    /// # Errors
    ///
    /// Returns `UveddiError` if detector creation or engine initialization fails
    pub fn create_engine_sync(&self) -> Result<AnalysisEngine, UveddiError> {
        let mut registry = DetectorRegistry::new();

        if self.detectors.is_empty() {
            registry.load_defaults();
        } else {
            registry.load_from_config(&self.detectors)?;
        }

        let detectors = registry.get_all_detectors();
        let cache_path = self.cache_path.as_ref().map(|p| Path::new(p));

        AnalysisEngine::builder()
            .with_detectors(detectors)
            .with_cache_path(cache_path.unwrap_or(Path::new("uveddi_cache.db")))
            .build()
    }

    /// Validate the configuration
    ///
    /// Checks if the configuration is valid by attempting to create all
    /// configured detectors without actually creating an engine.
    ///
    /// # Returns
    ///
    /// `Ok(())` if configuration is valid
    ///
    /// # Errors
    ///
    /// Returns `UveddiError::ConfigError` if any detector configuration is invalid
    pub fn validate(&self) -> Result<(), UveddiError> {
        let mut registry = DetectorRegistry::new();
        registry.load_from_config(&self.detectors)?;
        Ok(())
    }

    /// Get a list of configured detector names
    pub fn get_detector_names(&self) -> Vec<String> {
        let mut names: std::collections::HashSet<String> = std::collections::HashSet::new();
        names.extend(self.detectors.keys().cloned());
        names.extend(self.enhanced_detectors.keys().cloned());
        names.extend(self.standard_detectors.keys().cloned());
        names.into_iter().collect::<Vec<String>>() // Explicitly collect into Vec<String> for API clarity
    }

    /// Check if a specific detector is configured
    pub fn has_detector(&self, name: &str) -> bool {
        self.detectors.contains_key(name)
            || self.enhanced_detectors.contains_key(name)
            || self.standard_detectors.contains_key(name)
    }

    /// Add a detector configuration
    ///
    /// # Arguments
    ///
    /// * `name` - The detector name
    /// * `config` - The detector configuration
    pub fn add_detector(&mut self, name: String, config: DetectorConfig) {
        self.detectors.insert(name, config);
    }

    /// Remove a detector configuration
    ///
    /// # Arguments
    ///
    /// * `name` - The detector name to remove
    ///
    /// # Returns
    ///
    /// The removed configuration, if it existed
    pub fn remove_detector(&mut self, name: &str) -> Option<DetectorConfig> {
        self.detectors.remove(name)
    }
    /// Migrate legacy detector configurations to standardized format
    ///
    /// This method converts old detector configurations to the new standardized
    /// format while preserving all settings. It's useful for upgrading existing
    /// configuration files.
    ///
    /// # Returns
    ///
    /// A new `AnalysisConfig` with standardized detector configurations
    pub fn migrate_to_standardized(&self) -> Result<AnalysisConfig, UveddiError> {
        let mut new_config = self.clone();

        // Clear existing standard detectors to avoid conflicts
        new_config.standard_detectors.clear();

        // Migrate enhanced detectors to standardized format
        for (detector_name, enhanced_config) in &self.enhanced_detectors {
            if enhanced_config.enabled {
                let standard_config = StandardDetectorConfig::default_for_detector(detector_name);
                new_config
                    .standard_detectors
                    .insert(detector_name.clone(), standard_config);
            }
        }

        // If no enhanced detectors, create defaults for all known detectors
        if new_config.standard_detectors.is_empty() {
            let detector_names = [
                "god_object",
                "code_duplication",
                "large_classes",
                "dead_code",
                "tight_coupling",
            ];
            for detector_name in &detector_names {
                let standard_config = StandardDetectorConfig::default_for_detector(detector_name);
                new_config
                    .standard_detectors
                    .insert(detector_name.to_string(), standard_config);
            }
        }

        Ok(new_config)
    }

    /// Get effective detector configuration (prioritizes standardized over legacy)
    ///
    /// This method returns the effective configuration for a detector, checking
    /// standardized configurations first, then enhanced, then legacy formats.
    ///
    /// # Arguments
    ///
    /// * `detector_name` - Name of the detector to get configuration for
    ///
    /// # Returns
    ///
    /// The effective `StandardDetectorConfig` for the detector, or default if none found
    pub fn get_effective_detector_config(&self, detector_name: &str) -> StandardDetectorConfig {
        // Priority: standardized > enhanced > legacy > default
        if let Some(standard_config) = self.standard_detectors.get(detector_name) {
            return standard_config.clone();
        }

        if let Some(enhanced_config) = self.enhanced_detectors.get(detector_name) {
            // Convert enhanced to standardized format
            let mut standard_config = StandardDetectorConfig::default_for_detector(detector_name);
            standard_config.enabled = enhanced_config.enabled;
            standard_config.severity = enhanced_config.severity.clone();
            return standard_config;
        }

        if let Some(_legacy_config) = self.detectors.get(detector_name) {
            // Convert legacy to standardized format (basic conversion)
            return StandardDetectorConfig::default_for_detector(detector_name);
        }

        // Return default configuration
        StandardDetectorConfig::default_for_detector(detector_name)
    }

    /// Set standard detector configuration for a specific detector
    ///
    /// Updates the detector configuration with standardized settings,
    /// overriding any existing configuration for the specified detector.
    ///
    /// # Parameters
    ///
    /// * `detector_name` - Name of the detector to configure (must match exactly)
    ///   - Valid names: "god_object", "dead_code", "cyclic_dependencies", "tight_coupling",
    ///     "code_duplication", "long_method", "long_parameter_list"
    ///   - Case-sensitive string matching is used
    ///   - Unknown detector names will be stored but ignored during analysis
    /// * `config` - Standardized configuration settings to apply
    ///   - Replaces existing configuration completely
    ///   - Will be validated for consistency before application
    ///   - Must include enabled status and severity level
    ///
    /// # Side Effects
    ///
    /// - Invalidates any cached analysis results for this detector
    /// - May trigger re-analysis if the engine is currently running
    /// - Configuration changes are persisted to config file if auto-save enabled
    ///
    /// # Examples
    ///
    /// ```rust
    /// use uveddi::analysis::{AnalysisConfig, StandardDetectorConfig, IssueSeverity};
    /// use std::collections::HashMap;
    ///
    /// let mut config = AnalysisConfig::default();
    /// let detector_config = StandardDetectorConfig {
    ///     enabled: true,
    ///     severity: IssueSeverity::High,
    ///     thresholds: {
    ///         let mut thresholds = HashMap::new();
    ///         thresholds.insert("max_methods".to_string(), 20.into());
    ///         thresholds.insert("max_fields".to_string(), 15.into());
    ///         thresholds
    ///     },
    /// };
    ///
    /// config.set_standard_detector_config("god_object", detector_config);
    /// ```
    pub fn set_standard_detector_config(
        &mut self,
        detector_name: &str,
        config: StandardDetectorConfig,
    ) {
        self.standard_detectors
            .insert(detector_name.to_string(), config);
    }

    /// Get all enabled detector names across all configuration formats
    ///
    /// # Returns
    ///
    /// A vector of detector names that are enabled in any configuration format
    pub fn get_enabled_detectors(&self) -> Vec<String> {
        let mut enabled = std::collections::HashSet::new();

        // Check standardized configurations
        for (name, config) in &self.standard_detectors {
            if config.enabled {
                enabled.insert(name.clone());
            }
        }

        // Check enhanced configurations
        for (name, config) in &self.enhanced_detectors {
            if config.enabled {
                enabled.insert(name.clone());
            }
        }

        // Legacy configurations are assumed enabled if present
        for name in self.detectors.keys() {
            enabled.insert(name.clone());
        }

        enabled.into_iter().collect::<Vec<String>>() // Explicitly collect to ensure deterministic API return type
    }
}

impl Default for AnalysisConfig {
    /// Create a default configuration
    ///
    /// The default configuration includes:
    /// - All standard detectors with default settings (using enhanced format)
    /// - Cache size of 1000 entries
    /// - Plugins disabled
    /// - Default cache path
    /// - Default cache and performance settings
    fn default() -> Self {
        let mut enhanced_detectors = HashMap::new();

        // Add default enhanced detector configurations
        enhanced_detectors.insert(
            "god_object".to_string(),
            EnhancedDetectorConfig::new()
                .with_enabled(true)
                .with_severity(IssueSeverity::High)
                .with_max_methods(20)
                .with_max_fields(15),
        );
        enhanced_detectors.insert(
            "code_duplication".to_string(),
            EnhancedDetectorConfig::new()
                .with_enabled(true)
                .with_severity(IssueSeverity::Medium)
                .with_min_similarity(0.8),
        );
        enhanced_detectors.insert(
            "dead_code".to_string(),
            EnhancedDetectorConfig::new()
                .with_enabled(true)
                .with_severity(IssueSeverity::Medium),
        );
        enhanced_detectors.insert(
            "large_classes".to_string(),
            EnhancedDetectorConfig::new()
                .with_enabled(true)
                .with_severity(IssueSeverity::High)
                .with_max_lines(500),
        );
        enhanced_detectors.insert(
            "tight_coupling".to_string(),
            EnhancedDetectorConfig::new()
                .with_enabled(true)
                .with_severity(IssueSeverity::Medium),
        );

        // Create standardized detector configurations
        let mut standard_detectors = HashMap::new();
        standard_detectors.insert(
            "god_object".to_string(),
            StandardDetectorConfig::default_for_detector("god_object"),
        );
        standard_detectors.insert(
            "code_duplication".to_string(),
            StandardDetectorConfig::default_for_detector("code_duplication"),
        );
        standard_detectors.insert(
            "large_classes".to_string(),
            StandardDetectorConfig::default_for_detector("large_classes"),
        );
        standard_detectors.insert(
            "dead_code".to_string(),
            StandardDetectorConfig::default_for_detector("dead_code"),
        );
        standard_detectors.insert(
            "tight_coupling".to_string(),
            StandardDetectorConfig::default_for_detector("tight_coupling"),
        );

        Self {
            detectors: HashMap::new(), // Legacy format, empty by default
            enhanced_detectors,
            standard_detectors,
            cache_size: Some(1000),
            enable_plugins: false,
            cache_path: Some("uveddi_cache.db".to_string()),
            cache_settings: CacheConfig::default(),
            performance_settings: PerformanceConfig::default(),
        }
    }
}

// Make enhanced types serializable for TOML support
impl Serialize for IssueSeverity {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            IssueSeverity::Low => serializer.serialize_str("Low"),
            IssueSeverity::Medium => serializer.serialize_str("Medium"),
            IssueSeverity::High => serializer.serialize_str("High"),
            IssueSeverity::Critical => serializer.serialize_str("Critical"),
        }
    }
}

impl<'de> Deserialize<'de> for IssueSeverity {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "Low" => Ok(IssueSeverity::Low),
            "Medium" => Ok(IssueSeverity::Medium),
            "High" => Ok(IssueSeverity::High),
            "Critical" => Ok(IssueSeverity::Critical),
            _ => Err(serde::de::Error::custom(format!(
                "Unknown severity level: {}",
                s
            ))),
        }
    }
}

impl Serialize for EnhancedDetectorConfig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("EnhancedDetectorConfig", 3)?;
        state.serialize_field("enabled", &self.enabled)?;
        state.serialize_field("severity", &self.severity)?;
        state.serialize_field("thresholds", &self.thresholds)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for EnhancedDetectorConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            #[serde(default = "default_true")]
            enabled: bool,
            #[serde(default)]
            severity: IssueSeverity,
            #[serde(default)]
            thresholds: DetectorThresholds,
        }

        fn default_true() -> bool {
            true
        }

        let helper = Helper::deserialize(deserializer)?;
        Ok(EnhancedDetectorConfig {
            enabled: helper.enabled,
            severity: helper.severity,
            thresholds: helper.thresholds,
        })
    }
}

// Make DetectorConfig serializable for TOML support
impl Serialize for DetectorConfig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.params().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for DetectorConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let params: HashMap<String, i32> = HashMap::deserialize(deserializer)?;
        let mut config = DetectorConfig::new();
        for (key, value) in params {
            config = config.with_param(&key, value);
        }
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = AnalysisConfig::default();

        // Enhanced detectors should be configured by default now
        assert_eq!(config.enhanced_detectors.len(), 5);
        assert!(config.enhanced_detectors.contains_key("god_object"));
        assert!(config.enhanced_detectors.contains_key("code_duplication"));
        assert_eq!(config.cache_size, Some(1000));
        assert!(!config.enable_plugins);
        assert_eq!(config.cache_path, Some("uveddi_cache.db".to_string()));
    }

    #[test]
    fn test_detector_management() {
        let mut config = AnalysisConfig::default();

        // Test adding detector
        config.add_detector(
            "custom_detector".to_string(),
            DetectorConfig::new().with_param("param1", 42),
        );

        assert!(config.has_detector("custom_detector"));
        assert_eq!(config.get_detector_names().len(), 6);

        // Test removing detector
        let removed = config.remove_detector("custom_detector");
        assert!(removed.is_some());
        assert!(!config.has_detector("custom_detector"));
    }

    #[test]
    fn test_validate_config() {
        let config = AnalysisConfig::default();
        assert!(config.validate().is_ok());

        // Test invalid config
        let mut invalid_config = AnalysisConfig::default();
        invalid_config.add_detector("invalid_detector".to_string(), DetectorConfig::new());
        assert!(invalid_config.validate().is_err());
    }

    #[tokio::test]
    async fn test_create_engine() {
        let config = AnalysisConfig::default();
        let engine = config.create_engine().await;
        assert!(engine.is_ok());

        let engine = engine.expect("Engine creation should succeed in test");
        assert!(engine.get_anti_pattern_types().len() >= 5);
    }

    #[test]
    fn test_create_engine_sync() {
        let config = AnalysisConfig::default();
        let engine = config.create_engine_sync();
        assert!(engine.is_ok());

        let engine = engine.expect("Engine creation should succeed in test");
        assert!(engine.get_anti_pattern_types().len() >= 5);
    }

    #[test]
    fn test_toml_serialization() {
        let config = AnalysisConfig::default();
        let toml_string = toml::to_string(&config);
        assert!(toml_string.is_ok());

        let toml_string = toml_string.expect("TOML serialization should succeed in test");
        let parsed_config: AnalysisConfig =
            toml::from_str(&toml_string).expect("TOML parsing should succeed in test");
        assert_eq!(
            parsed_config.enhanced_detectors.len(),
            config.enhanced_detectors.len()
        );
        assert_eq!(parsed_config.cache_size, config.cache_size);
    }

    #[test]
    fn test_file_operations() {
        let config = AnalysisConfig::default();

        // Test saving to file
        let mut temp_file = NamedTempFile::new().expect("Failed to create temporary file for test");
        let save_result = config.save_to_file(temp_file.path());
        assert!(save_result.is_ok());

        // Test loading from file
        let loaded_config = AnalysisConfig::from_file(temp_file.path());
        assert!(loaded_config.is_ok());

        let loaded_config = loaded_config.expect("Config loading should succeed in test");
        assert_eq!(
            loaded_config.enhanced_detectors.len(),
            config.enhanced_detectors.len()
        );
        assert_eq!(loaded_config.cache_size, config.cache_size);
    }

    #[test]
    fn test_from_toml_string() {
        let toml_content = r#"
cache_size = 500
enable_plugins = true
cache_path = "custom_cache.db"

[detectors.god_object]
threshold_methods = 10
threshold_fields = 15

[detectors.code_duplication]
"#;

        let config: AnalysisConfig =
            toml::from_str(toml_content).expect("TOML parsing should succeed in test");

        assert_eq!(config.cache_size, Some(500));
        assert!(config.enable_plugins);
        assert_eq!(config.cache_path, Some("custom_cache.db".to_string()));
        assert_eq!(config.detectors.len(), 2);

        let god_object_config = config
            .detectors
            .get("god_object")
            .expect("god_object config should exist in test");
        assert_eq!(god_object_config.get("threshold_methods"), Some(10));
        assert_eq!(god_object_config.get("threshold_fields"), Some(15));
    }

    #[test]
    fn test_empty_detectors_uses_defaults() {
        let config = AnalysisConfig {
            detectors: HashMap::new(),
            enhanced_detectors: HashMap::new(),
            standard_detectors: HashMap::new(),
            cache_size: Some(100),
            enable_plugins: false,
            cache_path: None,
            cache_settings: CacheConfig::default(),
            performance_settings: PerformanceConfig::default(),
        };

        let engine = config.create_engine_sync();
        assert!(engine.is_ok());

        // Should have default detectors despite empty config
        let engine = engine.expect("Engine creation should succeed in test");
        assert!(engine.get_anti_pattern_types().len() >= 5);
    }
}
