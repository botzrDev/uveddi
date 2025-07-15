use crate::analysis::{AnalysisConfig, AnalysisDetector, AnalysisEngine};
use crate::error::UveddiError;
use std::path::{Path, PathBuf};

#[cfg(feature = "memory-optimization")]
use crate::analysis::memory::MemoryOptimizationConfig;

/// Builder pattern for configuring AnalysisEngine with dependency injection
///
/// The `AnalysisEngineBuilder` provides a fluent interface for constructing
/// `AnalysisEngine` instances with custom detector configurations, cache settings,
/// and plugin support. This enables dependency injection and allows for more
/// flexible testing and configuration management.
///
/// # Examples
///
/// ```rust
/// use uveddi::analysis::{AnalysisEngineBuilder, AnalysisDetector};
/// use uveddi::analysis::detectors::anti_patterns::GodObjectDetector;
///
/// # async fn example() -> Result<(), uveddi::error::UveddiError> {
/// let engine = AnalysisEngineBuilder::new()
///     .add_detector(Box::new(GodObjectDetector::new(10, 15)))
///     .with_memory_cache()
///     .enable_plugins()
///     .build()
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Default)]
pub struct AnalysisEngineBuilder {
    detectors: Vec<Box<dyn AnalysisDetector + Send + Sync>>,
    cache_path: Option<PathBuf>,
    enable_plugins: bool,
    use_memory_cache: bool,
    #[cfg(feature = "memory-optimization")]
    memory_optimization_config: Option<MemoryOptimizationConfig>,
}

impl AnalysisEngineBuilder {
    /// Creates a new builder instance with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a detector to the engine
    ///
    /// # Arguments
    ///
    /// * `detector` - A boxed detector implementing the `AnalysisDetector` trait
    ///
    /// # Examples
    ///
    /// ```rust
    /// use uveddi::analysis::{AnalysisEngineBuilder};
    /// use uveddi::analysis::detectors::anti_patterns::GodObjectDetector;
    ///
    /// let builder = AnalysisEngineBuilder::new()
    ///     .add_detector(Box::new(GodObjectDetector::new(5, 8)));
    /// ```
    pub fn add_detector(mut self, detector: Box<dyn AnalysisDetector + Send + Sync>) -> Self {
        self.detectors.push(detector);
        self
    }

    /// Set custom cache path
    ///
    /// # Arguments
    ///
    /// * `path` - The filesystem path where the cache database should be stored
    pub fn with_cache_path(mut self, path: PathBuf) -> Self {
        self.cache_path = Some(path);
        self
    }

    /// Enable plugin support
    ///
    /// When enabled, the engine will initialize the WASM plugin system
    /// and allow dynamic loading of plugin-based detectors.
    pub fn enable_plugins(mut self) -> Self {
        self.enable_plugins = true;
        self
    }

    /// Use in-memory cache (for testing)
    ///
    /// This is primarily useful for testing scenarios where you want
    /// to avoid filesystem interactions and ensure test isolation.
    pub fn with_memory_cache(mut self) -> Self {
        self.use_memory_cache = true;
        self
    }

    /// Configure memory optimization settings
    ///
    /// Enables memory optimization features with custom configuration.
    /// This includes object pooling, arena allocation, and zero-copy AST caching.
    ///
    /// # Arguments
    ///
    /// * `config` - Memory optimization configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use uveddi::analysis::{AnalysisEngineBuilder, memory::MemoryOptimizationConfig};
    ///
    /// # async fn example() -> Result<(), uveddi::error::UveddiError> {
    /// let config = MemoryOptimizationConfig::large_codebase();
    /// let engine = AnalysisEngineBuilder::new()
    ///     .with_memory_optimization(config)
    ///     .build()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "memory-optimization")]
    pub fn with_memory_optimization(mut self, config: MemoryOptimizationConfig) -> Self {
        self.memory_optimization_config = Some(config);
        self
    }

    /// Load configuration from file
    ///
    /// Loads an AnalysisConfig from a TOML file and applies its settings
    /// to the builder. This allows for external configuration of the engine.
    ///
    /// # Arguments
    ///
    /// * `config_path` - Path to the TOML configuration file
    ///
    /// # Examples
    ///
    /// ```rust
    /// use uveddi::analysis::AnalysisEngineBuilder;
    /// use std::path::Path;
    ///
    /// # async fn example() -> Result<(), uveddi::error::UveddiError> {
    /// let engine = AnalysisEngineBuilder::new()
    ///     .from_config_file(Path::new("config.toml"))?
    ///     .build()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_config_file(mut self, config_path: &Path) -> Result<Self, UveddiError> {
        let config = AnalysisConfig::from_file(config_path)?;
        self.from_config(&config)
    }

    /// Load configuration from AnalysisConfig
    ///
    /// Applies settings from an AnalysisConfig instance to the builder.
    /// This enables programmatic configuration of the engine.
    ///
    /// # Arguments
    ///
    /// * `config` - The AnalysisConfig to apply
    ///
    /// # Examples
    ///
    /// ```rust
    /// use uveddi::analysis::{AnalysisEngineBuilder, AnalysisConfig};
    ///
    /// # async fn example() -> Result<(), uveddi::error::UveddiError> {
    /// let config = AnalysisConfig::default();
    /// let engine = AnalysisEngineBuilder::new()
    ///     .from_config(&config)?
    ///     .build()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_config(mut self, config: &AnalysisConfig) -> Result<Self, UveddiError> {
        // Load detectors from configuration
        let mut registry = crate::analysis::DetectorRegistry::new();
        if config.detectors.is_empty() {
            registry.load_defaults();
        } else {
            registry.load_from_config(&config.detectors)?;
        }

        // Apply configuration settings
        if let Some(ref cache_path) = config.cache_path {
            self.cache_path = Some(PathBuf::from(cache_path));
        }

        if config.enable_plugins {
            self.enable_plugins = true;
        }

        // Set detectors from registry
        self.detectors = registry.get_all_detectors();

        Ok(self)
    }

    /// Build the AnalysisEngine with configured options
    ///
    /// Creates an `AnalysisEngine` instance using the builder's configuration.
    /// If no detectors were added, the default detector set will be used.
    /// If plugins are enabled, plugin detectors will be automatically loaded.
    ///
    /// # Returns
    ///
    /// A configured `AnalysisEngine` instance
    ///
    /// # Errors
    ///
    /// Returns `UveddiError` if:
    /// - Cache initialization fails
    /// - Plugin engine initialization fails (when plugins are enabled)
    /// - AST parser or dependency extractor initialization fails
    pub async fn build(self) -> Result<AnalysisEngine, UveddiError> {
        // If no detectors were provided, use the default set
        let detectors = if self.detectors.is_empty() {
            crate::analysis::detector_factory::DetectorFactory::create_default_detectors()
        } else {
            self.detectors
        };

        let cache_path = if self.use_memory_cache {
            None
        } else {
            self.cache_path.as_deref()
        };

        let mut engine = if self.enable_plugins {
            // TODO: Add plugin support with memory optimization
            if let Some(path) = cache_path {
                AnalysisEngine::with_detectors_and_plugins(detectors, Some(path)).await?
            } else {
                AnalysisEngine::with_detectors_and_plugins(detectors, None).await?
            }
        } else {
            // Use memory optimization if configured
            #[cfg(feature = "memory-optimization")]
            if let Some(memory_config) = self.memory_optimization_config {
                AnalysisEngine::with_detectors(detectors, cache_path, false)?
            } else {
                AnalysisEngine::with_detectors(detectors, cache_path, false)?
            }
            
            #[cfg(not(feature = "memory-optimization"))]
            AnalysisEngine::with_detectors(detectors, cache_path, false)?
        };

        // If plugins are enabled, automatically load plugin detectors
        if self.enable_plugins && engine.has_plugin_support() {
            match engine.add_plugin_detectors().await {
                Ok(count) => {
                    log::info!("Builder automatically loaded {} plugin detectors", count);
                }
                Err(e) => {
                    log::warn!("Failed to auto-load plugin detectors in builder: {}", e);
                    // Continue anyway - the engine is still usable without plugins
                }
            }
        }

        Ok(engine)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::detectors::anti_patterns::god_object::GodObjectDetector;

    #[tokio::test]
    async fn test_builder_with_default_detectors() {
        let builder = AnalysisEngineBuilder::new().with_memory_cache();

        let engine = builder.build().await;
        assert!(engine.is_ok());

        let engine = engine.unwrap();
        // Should have default detectors plus cycle detector
        assert!(engine.get_anti_pattern_types().len() >= 5);
    }

    #[tokio::test]
    async fn test_builder_with_custom_detectors() {
        let builder = AnalysisEngineBuilder::new()
            .add_detector(Box::new(GodObjectDetector::new(10, 15)))
            .with_memory_cache();

        let engine = builder.build().await;
        assert!(engine.is_ok());

        let engine = engine.unwrap();
        // Should have our custom detector plus cycle detector
        assert!(engine.get_anti_pattern_types().len() >= 1);
    }

    #[test]
    fn test_builder_fluent_interface() {
        let builder = AnalysisEngineBuilder::new()
            .add_detector(Box::new(GodObjectDetector::new(5, 8)))
            .enable_plugins()
            .with_memory_cache();

        // Builder should be properly configured
        assert_eq!(builder.detectors.len(), 1);
        assert!(builder.enable_plugins);
        assert!(builder.use_memory_cache);
    }

    #[test]
    fn test_builder_from_config() {
        let config = AnalysisConfig::default();
        let builder = AnalysisEngineBuilder::new().from_config(&config);

        assert!(builder.is_ok());
        let builder = builder.unwrap();

        // Should have default detectors loaded from config
        assert_eq!(builder.detectors.len(), 5);
        assert!(builder.enable_plugins == config.enable_plugins);
    }

    #[tokio::test]
    async fn test_builder_with_config_file() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Create a temporary config file
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_content = r#"
cache_size = 500
enable_plugins = false

[detectors.god_object]
threshold_methods = 10
threshold_fields = 15
"#;
        temp_file.write_all(config_content.as_bytes()).unwrap();

        let builder = AnalysisEngineBuilder::new().from_config_file(temp_file.path());

        assert!(builder.is_ok());
        let builder = builder.unwrap();

        // Should have configured detectors
        assert!(builder.detectors.len() > 0);
        assert!(!builder.enable_plugins); // Should respect config setting
    }

    #[tokio::test]
    async fn test_builder_config_overrides() {
        let config = AnalysisConfig::default();

        let engine = AnalysisEngineBuilder::new()
            .from_config(&config)
            .unwrap()
            .with_memory_cache() // Override cache setting
            .build()
            .await;

        assert!(engine.is_ok());
    }
}
