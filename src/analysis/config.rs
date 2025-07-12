use crate::analysis::{DetectorConfig, DetectorRegistry, AnalysisEngine};
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
/// # Detector configurations
/// [detectors.god_object]
/// threshold_methods = 5
/// threshold_fields = 8
///
/// [detectors.code_duplication]
/// # Uses default settings
///
/// [detectors.large_classes]
/// max_lines = 300
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
    /// Configuration for individual detectors
    pub detectors: HashMap<String, DetectorConfig>,
    /// Maximum cache size (number of entries)
    pub cache_size: Option<usize>,
    /// Whether to enable WASM plugin support
    pub enable_plugins: bool,
    /// Path to the cache database file
    pub cache_path: Option<String>,
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
        let content = std::fs::read_to_string(path)
            .map_err(|e| UveddiError::ConfigError(format!("Failed to read config file '{}': {}", path.display(), e)))?;
        
        toml::from_str(&content)
            .map_err(|e| UveddiError::ConfigError(format!("Failed to parse TOML config: {}", e)))
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
        let toml_content = toml::to_string_pretty(self)
            .map_err(|e| UveddiError::ConfigError(format!("Failed to serialize config to TOML: {}", e)))?;
        
        std::fs::write(path, toml_content)
            .map_err(|e| UveddiError::ConfigError(format!("Failed to write config file '{}': {}", path.display(), e)))
    }
    
    /// Create engine from this configuration
    ///
    /// Creates an `AnalysisEngine` instance using the settings in this configuration.
    /// Detectors are loaded according to the configuration, and cache and plugin
    /// settings are applied.
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
        
        if self.detectors.is_empty() {
            // If no detectors configured, use defaults
            registry.load_defaults();
        } else {
            // Load configured detectors
            registry.load_from_config(&self.detectors)?;
        }
        
        let detectors = registry.get_all_detectors();
        let cache_path = self.cache_path.as_ref().map(|p| Path::new(p));
        
        if self.enable_plugins {
            AnalysisEngine::with_detectors_and_plugins(detectors, cache_path).await
        } else {
            AnalysisEngine::with_detectors(detectors, cache_path, false)
        }
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
        
        AnalysisEngine::with_detectors(detectors, cache_path, false)
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
        self.detectors.keys().cloned().collect()
    }
    
    /// Check if a specific detector is configured
    pub fn has_detector(&self, name: &str) -> bool {
        self.detectors.contains_key(name)
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
}

impl Default for AnalysisConfig {
    /// Create a default configuration
    ///
    /// The default configuration includes:
    /// - All standard detectors with default settings
    /// - Cache size of 1000 entries
    /// - Plugins disabled
    /// - Default cache path
    fn default() -> Self {
        let mut detectors = HashMap::new();
        
        // Add default detector configurations
        detectors.insert("god_object".to_string(), 
            DetectorConfig::new()
                .with_param("threshold_methods", 5)
                .with_param("threshold_fields", 8)
        );
        detectors.insert("code_duplication".to_string(), DetectorConfig::new());
        detectors.insert("dead_code".to_string(), DetectorConfig::new());
        detectors.insert("large_classes".to_string(), DetectorConfig::new());
        detectors.insert("tight_coupling".to_string(), DetectorConfig::new());
        
        Self {
            detectors,
            cache_size: Some(1000),
            enable_plugins: false,
            cache_path: Some("uveddi_cache.db".to_string()),
        }
    }
}

// Make DetectorConfig serializable for TOML support
impl Serialize for DetectorConfig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.params.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for DetectorConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let params = HashMap::deserialize(deserializer)?;
        Ok(DetectorConfig { params })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_default_config() {
        let config = AnalysisConfig::default();
        
        assert_eq!(config.detectors.len(), 5);
        assert!(config.has_detector("god_object"));
        assert!(config.has_detector("code_duplication"));
        assert_eq!(config.cache_size, Some(1000));
        assert!(!config.enable_plugins);
        assert_eq!(config.cache_path, Some("uveddi_cache.db".to_string()));
    }

    #[test]
    fn test_detector_management() {
        let mut config = AnalysisConfig::default();
        
        // Test adding detector
        config.add_detector("custom_detector".to_string(), 
            DetectorConfig::new().with_param("param1", 42));
        
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
        
        let engine = engine.unwrap();
        assert!(engine.get_anti_pattern_types().len() >= 5);
    }

    #[test]
    fn test_create_engine_sync() {
        let config = AnalysisConfig::default();
        let engine = config.create_engine_sync();
        assert!(engine.is_ok());
        
        let engine = engine.unwrap();
        assert!(engine.get_anti_pattern_types().len() >= 5);
    }

    #[test]
    fn test_toml_serialization() {
        let config = AnalysisConfig::default();
        let toml_string = toml::to_string(&config);
        assert!(toml_string.is_ok());
        
        let parsed_config: AnalysisConfig = toml::from_str(&toml_string.unwrap()).unwrap();
        assert_eq!(parsed_config.detectors.len(), config.detectors.len());
        assert_eq!(parsed_config.cache_size, config.cache_size);
    }

    #[test]
    fn test_file_operations() {
        let config = AnalysisConfig::default();
        
        // Test saving to file
        let mut temp_file = NamedTempFile::new().unwrap();
        let save_result = config.save_to_file(temp_file.path());
        assert!(save_result.is_ok());
        
        // Test loading from file
        let loaded_config = AnalysisConfig::from_file(temp_file.path());
        assert!(loaded_config.is_ok());
        
        let loaded_config = loaded_config.unwrap();
        assert_eq!(loaded_config.detectors.len(), config.detectors.len());
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
        
        let config: AnalysisConfig = toml::from_str(toml_content).unwrap();
        
        assert_eq!(config.cache_size, Some(500));
        assert!(config.enable_plugins);
        assert_eq!(config.cache_path, Some("custom_cache.db".to_string()));
        assert_eq!(config.detectors.len(), 2);
        
        let god_object_config = config.detectors.get("god_object").unwrap();
        assert_eq!(god_object_config.get("threshold_methods"), Some(10));
        assert_eq!(god_object_config.get("threshold_fields"), Some(15));
    }

    #[test]
    fn test_empty_detectors_uses_defaults() {
        let config = AnalysisConfig {
            detectors: HashMap::new(),
            cache_size: Some(100),
            enable_plugins: false,
            cache_path: None,
        };
        
        let engine = config.create_engine_sync();
        assert!(engine.is_ok());
        
        // Should have default detectors despite empty config
        let engine = engine.unwrap();
        assert!(engine.get_anti_pattern_types().len() >= 5);
    }
}