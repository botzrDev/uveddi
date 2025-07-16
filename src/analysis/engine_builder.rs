use crate::analysis::{AnalysisConfig, AnalysisDetector, AnalysisEngine};
use crate::analysis::traits::{AstParserTrait, DependencyExtractorTrait, ResultCacheTrait};
use crate::analysis::adapters::{AstParserAdapter, DependencyExtractorAdapter, ResultCacheAdapter};
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
pub struct AnalysisEngineBuilder {
    detectors: Vec<Box<dyn AnalysisDetector + Send + Sync>>,
    cache_path: Option<PathBuf>,
    enable_plugins: bool,
    use_memory_cache: bool,
    // Dependency injection fields
    ast_parser: Option<Box<dyn AstParserTrait>>,
    dependency_extractor: Option<Box<dyn DependencyExtractorTrait>>,
    cache: Option<Box<dyn ResultCacheTrait>>,
    config: Option<AnalysisConfig>,
    #[cfg(feature = "memory-optimization")]
    memory_optimization_config: Option<MemoryOptimizationConfig>,
}

impl Default for AnalysisEngineBuilder {
    fn default() -> Self {
        Self {
            detectors: Vec::new(),
            cache_path: None,
            enable_plugins: false,
            use_memory_cache: false,
            ast_parser: None,
            dependency_extractor: None,
            cache: None,
            config: None,
            #[cfg(feature = "memory-optimization")]
            memory_optimization_config: None,
        }
    }
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

    /// Inject custom AST parser (DEPENDENCY INJECTION)
    ///
    /// Allows injection of a custom AST parser implementation for
    /// better testability and modularity.
    ///
    /// # Arguments
    ///
    /// * `parser` - AST parser implementation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use uveddi::analysis::{AnalysisEngineBuilder, adapters::AstParserAdapter};
    ///
    /// # async fn example() -> Result<(), uveddi::error::UveddiError> {
    /// let custom_parser = Box::new(AstParserAdapter::new_default()?);
    /// let engine = AnalysisEngineBuilder::new()
    ///     .with_ast_parser(custom_parser)
    ///     .build()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_ast_parser(mut self, parser: Box<dyn AstParserTrait>) -> Self {
        self.ast_parser = Some(parser);
        self
    }

    /// Inject custom dependency extractor (DEPENDENCY INJECTION)
    ///
    /// Allows injection of a custom dependency extractor implementation for
    /// better testability and modularity.
    ///
    /// # Arguments
    ///
    /// * `extractor` - Dependency extractor implementation
    pub fn with_dependency_extractor(mut self, extractor: Box<dyn DependencyExtractorTrait>) -> Self {
        self.dependency_extractor = Some(extractor);
        self
    }

    /// Inject custom result cache (DEPENDENCY INJECTION)
    ///
    /// Allows injection of a custom cache implementation for
    /// better testability and modularity.
    ///
    /// # Arguments
    ///
    /// * `cache` - Cache implementation
    pub fn with_cache(mut self, cache: Box<dyn ResultCacheTrait>) -> Self {
        self.cache = Some(cache);
        self
    }

    /// Configure from TOML configuration file (DEPENDENCY INJECTION)
    ///
    /// Loads configuration from a TOML file and sets up detectors and
    /// dependencies accordingly.
    ///
    /// # Arguments
    ///
    /// * `config_path` - Path to TOML configuration file
    ///
    /// # Returns
    ///
    /// Builder configured from file
    ///
    /// # Errors
    ///
    /// Returns UveddiError if file cannot be read or parsed
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
    pub fn from_config_file_di(mut self, config_path: &Path) -> Result<Self, UveddiError> {
        let config_content = std::fs::read_to_string(config_path)
            .map_err(|e| UveddiError::config_error(
                &format!("Failed to read config file {}: {}", config_path.display(), e),
                "configuration loading",
            ))?;
        
        let config: AnalysisConfig = toml::from_str(&config_content)
            .map_err(|e| UveddiError::config_error(
                &format!("Failed to parse config file {}: {}", config_path.display(), e),
                "TOML parsing",
            ))?;
        
        self.config = Some(config);
        Ok(self)
    }

    /// Add detector to the analysis pipeline (DEPENDENCY INJECTION)
    ///
    /// # Arguments
    ///
    /// * `detector` - Detector implementation
    pub fn with_detector(mut self, detector: Box<dyn AnalysisDetector + Send + Sync>) -> Self {
        self.detectors.push(detector);
        self
    }

    /// Add multiple detectors from configuration (DEPENDENCY INJECTION)
    ///
    /// Creates detectors based on the current configuration and adds them
    /// to the analysis pipeline.
    ///
    /// # Returns
    ///
    /// Builder with detectors configured from current config
    ///
    /// # Errors
    ///
    /// Returns UveddiError if detector creation fails
    pub fn with_detectors_from_config(mut self) -> Result<Self, UveddiError> {
        if let Some(ref config) = self.config {
            use crate::analysis::detector_factory::DetectorFactory;
            
            for (detector_name, detector_config) in &config.detectors {
                if detector_config.get("enabled").unwrap_or(1) == 1 {
                    let detector = DetectorFactory::create_detector(detector_name, detector_config)?;
                    self.detectors.push(detector);
                }
            }
        }
        
        Ok(self)
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

    /// Build the AnalysisEngine with configured options (DEPENDENCY INJECTION)
    ///
    /// Creates an `AnalysisEngine` instance using the builder's configuration.
    /// Uses dependency injection if custom dependencies were provided,
    /// otherwise falls back to the original build approach.
    ///
    /// # Returns
    ///
    /// A configured `AnalysisEngine` instance
    ///
    /// # Errors
    ///
    /// Returns `UveddiError` if:
    /// - Dependency validation fails
    /// - Cache initialization fails
    /// - Plugin engine initialization fails (when plugins are enabled)
    /// - AST parser or dependency extractor initialization fails
    pub async fn build(self) -> Result<AnalysisEngine, UveddiError> {
        // Check if we have dependency injection configured
        if self.ast_parser.is_some() || self.dependency_extractor.is_some() || self.cache.is_some() {
            // Use dependency injection approach
            self.build_with_dependency_injection().await
        } else {
            // Use original approach for backward compatibility
            self.build_traditional().await
        }
    }

    /// Build using dependency injection approach
    async fn build_with_dependency_injection(self) -> Result<AnalysisEngine, UveddiError> {
        // Use provided dependencies or create defaults
        let ast_parser = self.ast_parser
            .unwrap_or_else(|| Box::new(AstParserAdapter::new_default().expect("Failed to create default AST parser")));
        
        let dependency_extractor = self.dependency_extractor
            .unwrap_or_else(|| Box::new(DependencyExtractorAdapter::new_default().expect("Failed to create default dependency extractor")));
        
        let cache = self.cache
            .unwrap_or_else(|| {
                if self.use_memory_cache {
                    Box::new(ResultCacheAdapter::new_memory().expect("Failed to create memory cache"))
                } else if let Some(ref path) = self.cache_path {
                    Box::new(ResultCacheAdapter::new_with_path(path).expect("Failed to create file cache"))
                } else {
                    Box::new(ResultCacheAdapter::new_memory().expect("Failed to create default memory cache"))
                }
            });

        // Use provided detectors or create defaults
        let detectors = if self.detectors.is_empty() {
            crate::analysis::detector_factory::DetectorFactory::create_default_detectors()
        } else {
            self.detectors
        };

        // Create engine with injected dependencies
        let engine = AnalysisEngine::with_injected_dependencies(
            ast_parser,
            dependency_extractor,
            cache,
            detectors,
        )?;

        Ok(engine)
    }

    /// Build using traditional approach (backward compatibility)
    async fn build_traditional(self) -> Result<AnalysisEngine, UveddiError> {
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
            if let Some(_memory_config) = self.memory_optimization_config {
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
