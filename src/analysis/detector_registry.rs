use crate::analysis::{
    AnalysisDetector, DetectorConfig, DetectorFactory, WasmPluginAdapterFactory,
};
use crate::error::UveddiError;
use std::collections::HashMap;

/// Registry for managing detector instances
///
/// The `DetectorRegistry` provides centralized management of detector instances,
/// supporting both manual registration and configuration-driven setup. This enables
/// dynamic detector management and simplifies bulk detector operations.
///
/// # Features
///
/// - **Manual Registration**: Register detectors with custom names
/// - **Configuration Loading**: Load detectors from configuration mappings
/// - **Bulk Operations**: Retrieve all registered detectors at once
/// - **Default Sets**: Load the standard detector set with predefined names
///
/// # Examples
///
/// ```rust
/// use uveddi::analysis::{DetectorRegistry, DetectorConfig, DetectorFactory};
/// use std::collections::HashMap;
///
/// # fn example() -> Result<(), uveddi::error::UveddiError> {
/// let mut registry = DetectorRegistry::new();
///
/// // Load default detectors
/// registry.load_defaults();
///
/// // Or load from configuration
/// let mut configs = HashMap::new();
/// configs.insert("god_object".to_string(),
///     DetectorConfig::new().with_param("threshold_methods", 10));
/// registry.load_from_config(&configs)?;
///
/// // Get all registered detectors
/// let detectors = registry.get_all_detectors();
/// # Ok(())
/// # }
/// ```
pub struct DetectorRegistry {
    detectors: HashMap<String, Box<dyn AnalysisDetector + Send + Sync>>,
    factory: DetectorFactory,
}

impl DetectorRegistry {
    /// Creates a new empty detector registry
    pub fn new() -> Self {
        Self {
            detectors: HashMap::new(),
            factory: DetectorFactory,
        }
    }

    /// Register a detector with a custom name
    ///
    /// # Arguments
    ///
    /// * `name` - Unique identifier for the detector
    /// * `detector` - The detector instance to register
    ///
    /// # Examples
    ///
    /// ```rust
    /// use uveddi::analysis::{DetectorRegistry, GodObjectDetector};
    ///
    /// let mut registry = DetectorRegistry::new();
    /// registry.register("custom_god_object".to_string(),
    ///     Box::new(GodObjectDetector::new(15, 20)));
    /// ```
    pub fn register(&mut self, name: String, detector: Box<dyn AnalysisDetector + Send + Sync>) {
        self.detectors.insert(name, detector);
    }

    /// Get all registered detectors
    ///
    /// Consumes the registry and returns all detectors that were registered.
    /// This is typically used when building an AnalysisEngine instance.
    ///
    /// # Returns
    ///
    /// A vector of all registered detector instances
    pub fn get_all_detectors(self) -> Vec<Box<dyn AnalysisDetector + Send + Sync>> {
        self.detectors.into_values().collect()
    }

    /// Get a list of all registered detector names
    ///
    /// # Returns
    ///
    /// A vector of detector names currently in the registry
    pub fn get_detector_names(&self) -> Vec<String> {
        self.detectors.keys().cloned().collect()
    }

    /// Check if a detector with the given name is registered
    ///
    /// # Arguments
    ///
    /// * `name` - The detector name to check
    ///
    /// # Returns
    ///
    /// True if a detector with this name exists, false otherwise
    pub fn has_detector(&self, name: &str) -> bool {
        self.detectors.contains_key(name)
    }

    /// Get the number of registered detectors
    pub fn count(&self) -> usize {
        self.detectors.len()
    }

    /// Load detectors from configuration
    ///
    /// Creates detectors based on a configuration mapping and registers them
    /// with their configuration keys as names.
    ///
    /// # Arguments
    ///
    /// * `configs` - A mapping of detector names to their configurations
    ///
    /// # Errors
    ///
    /// Returns `UveddiError` if any detector creation fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use uveddi::analysis::{DetectorRegistry, DetectorConfig};
    /// use std::collections::HashMap;
    ///
    /// # fn example() -> Result<(), uveddi::error::UveddiError> {
    /// let mut registry = DetectorRegistry::new();
    /// let mut configs = HashMap::new();
    ///
    /// configs.insert("god_object".to_string(),
    ///     DetectorConfig::new()
    ///         .with_param("threshold_methods", 15)
    ///         .with_param("threshold_fields", 20));
    /// configs.insert("code_duplication".to_string(), DetectorConfig::new());
    ///
    /// registry.load_from_config(&configs)?;
    /// assert_eq!(registry.count(), 2);
    /// # Ok(())
    /// # }
    /// ```
    pub fn load_from_config(
        &mut self,
        configs: &HashMap<String, DetectorConfig>,
    ) -> Result<(), UveddiError> {
        for (name, detector_config) in configs {
            let detector = DetectorFactory::create_detector(name, detector_config)?;
            self.register(name.clone(), detector);
        }
        Ok(())
    }

    /// Load default detector set
    ///
    /// Registers the standard set of detectors with predefined names.
    /// This provides the same detector set that was previously hardcoded
    /// in the AnalysisEngine.
    ///
    /// # Detector Names
    ///
    /// - `god_object`: GodObjectDetector with default thresholds (5 methods, 8 fields)
    /// - `code_duplication`: CodeDuplicationDetector
    /// - `dead_code`: DeadCodeDetector with default config
    /// - `large_classes`: LargeClassDetector with default config
    /// - `tight_coupling`: TightCouplingDetector
    /// - `long_methods`: LongMethodsDetector with default config
    /// - `magic_values`: MagicValuesDetector with default config
    pub fn load_defaults(&mut self) {
        let defaults = DetectorFactory::create_default_detectors();
        let detector_names = vec![
            "god_object",
            "code_duplication",
            "dead_code",
            "large_classes",
            "tight_coupling",
            "long_methods",
            "magic_values",
        ];

        for (i, detector) in defaults.into_iter().enumerate() {
            if let Some(&name) = detector_names.get(i) {
                self.register(name.to_string(), detector);
            } else {
                // Fallback name if we add more detectors than names
                self.register(format!("detector_{}", i), detector);
            }
        }
    }

    /// Remove a detector by name
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the detector to remove
    ///
    /// # Returns
    ///
    /// True if a detector was removed, false if no detector with that name existed
    pub fn remove_detector(&mut self, name: &str) -> bool {
        self.detectors.remove(name).is_some()
    }

    /// Clear all registered detectors
    pub fn clear(&mut self) {
        self.detectors.clear();
    }

    /// Load plugin detectors from a WASM plugin engine
    ///
    /// Creates detector adapters for all loaded plugins in the given plugin engine
    /// and registers them with the registry.
    ///
    /// # Arguments
    ///
    /// * `plugin_engine` - Shared reference to the WASM plugin engine
    ///
    /// # Errors
    ///
    /// Returns `UveddiError` if plugin adapter creation fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use uveddi::analysis::DetectorRegistry;
    /// use uveddi::plugins::WasmPluginEngine;
    /// use std::sync::Arc;
    /// use tokio::sync::RwLock;
    ///
    /// # async fn example() -> Result<(), uveddi::error::UveddiError> {
    /// let mut registry = DetectorRegistry::new();
    /// let plugin_engine = Arc::new(RwLock::new(WasmPluginEngine::new().await?));
    ///
    /// registry.load_plugin_detectors(plugin_engine).await?;
    /// println!("Loaded {} detectors including plugins", registry.count());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn load_plugin_detectors(
        &mut self,
        plugin_engine: std::sync::Arc<tokio::sync::RwLock<crate::plugins::WasmPluginEngine>>,
    ) -> Result<usize, UveddiError> {
        let adapter_factory = WasmPluginAdapterFactory::new(plugin_engine);
        let plugin_adapters = adapter_factory.create_all_adapters().await?;

        let mut count = 0;
        for (i, adapter) in plugin_adapters.into_iter().enumerate() {
            let plugin_name = format!("plugin_{}", i);
            self.register(plugin_name, adapter);
            count += 1;
        }

        tracing::info!("Loaded {} plugin detectors into registry", count);
        Ok(count)
    }

    /// Remove all plugin detectors from the registry
    ///
    /// Removes all detectors that were created from WASM plugins.
    ///
    /// # Returns
    ///
    /// The number of plugin detectors that were removed
    pub fn remove_plugin_detectors(&mut self) -> usize {
        let initial_count = self.detectors.len();

        // Remove detectors that are WASM plugin adapters
        self.detectors
            .retain(|_name, detector| detector.get_detector_name() != "wasm-plugin-detector");

        let removed_count = initial_count - self.detectors.len();

        if removed_count > 0 {
            tracing::info!("Removed {} plugin detectors from registry", removed_count);
        }

        removed_count
    }
}

impl Default for DetectorRegistry {
    /// Creates a registry with the default detector set loaded
    fn default() -> Self {
        let mut registry = Self::new();
        registry.load_defaults();
        registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::detectors::anti_patterns::god_object::GodObjectDetector;

    #[test]
    fn test_new_registry() {
        let registry = DetectorRegistry::new();
        assert_eq!(registry.count(), 0);
        assert!(registry.get_detector_names().is_empty());
    }

    #[test]
    fn test_register_detector() {
        let mut registry = DetectorRegistry::new();
        let detector = Box::new(GodObjectDetector::new(5, 8));

        registry.register("test_detector".to_string(), detector);

        assert_eq!(registry.count(), 1);
        assert!(registry.has_detector("test_detector"));
        assert!(!registry.has_detector("nonexistent"));
    }

    #[test]
    fn test_load_defaults() {
        let mut registry = DetectorRegistry::new();
        registry.load_defaults();

        assert_eq!(registry.count(), 7);
        assert!(registry.has_detector("god_object"));
        assert!(registry.has_detector("code_duplication"));
        assert!(registry.has_detector("dead_code"));
        assert!(registry.has_detector("large_classes"));
        assert!(registry.has_detector("tight_coupling"));
        assert!(registry.has_detector("long_methods"));
        assert!(registry.has_detector("magic_values"));
    }

    #[test]
    fn test_load_from_config() {
        let mut registry = DetectorRegistry::new();
        let mut configs = HashMap::new();

        configs.insert(
            "god_object".to_string(),
            DetectorConfig::new().with_param("threshold_methods", 10),
        );
        configs.insert("code_duplication".to_string(), DetectorConfig::new());

        let result = registry.load_from_config(&configs);
        assert!(result.is_ok());
        assert_eq!(registry.count(), 2);
    }

    #[test]
    fn test_load_from_config_invalid_detector() {
        let mut registry = DetectorRegistry::new();
        let mut configs = HashMap::new();

        configs.insert("invalid_detector".to_string(), DetectorConfig::new());

        let result = registry.load_from_config(&configs);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_all_detectors() {
        let mut registry = DetectorRegistry::new();
        registry.load_defaults();

        let detectors = registry.get_all_detectors();
        assert_eq!(detectors.len(), 7);

        // Verify detector types
        let detector_names: Vec<&str> = detectors.iter().map(|d| d.get_detector_name()).collect();
        assert!(detector_names.contains(&"GodObjectDetector"));
        assert!(detector_names.contains(&"CodeDuplicationDetector"));
    }

    #[test]
    fn test_remove_detector() {
        let mut registry = DetectorRegistry::new();
        registry.load_defaults();

        assert!(registry.has_detector("god_object"));
        assert!(registry.remove_detector("god_object"));
        assert!(!registry.has_detector("god_object"));
        assert_eq!(registry.count(), 4);

        // Removing non-existent detector should return false
        assert!(!registry.remove_detector("nonexistent"));
    }

    #[test]
    fn test_clear() {
        let mut registry = DetectorRegistry::new();
        registry.load_defaults();

        assert_eq!(registry.count(), 7);
        registry.clear();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_default_registry() {
        let registry = DetectorRegistry::default();
        assert_eq!(registry.count(), 7);
        assert!(registry.has_detector("god_object"));
    }
}
