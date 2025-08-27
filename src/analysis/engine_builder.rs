use crate::core::logging::{info, warn};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::analysis::adapters::ResultCacheAdapter;
use crate::analysis::cache::ast::{AstCache, CacheConfig};
use crate::analysis::components::{
    AnalysisAggregator, AstProviderImpl, CacheManagerImpl, ConfigurationService,
    DependencyGraphBuilderImpl, DetectorScheduler, PluginManager,
};
use crate::analysis::detector_factory::DetectorFactory;
use crate::analysis::traits::{AstParserTrait, DependencyExtractorTrait, ResultCacheTrait};
use crate::analysis::AnalysisDetector;
use crate::cache::result_cache::ResultCache;
use crate::error::UveddiError;
use crate::monitoring::performance_metrics_collector::PerformanceMetricsCollector;
use crate::plugins::WasmPluginEngine;

// Stub types for when AI features are disabled
#[cfg(not(feature = "ai"))]
pub struct AiAnalysisEngine;

#[cfg(not(feature = "ai"))]
impl AiAnalysisEngine {
    pub fn new() -> Self {
        Self
    }
}
#[cfg(not(feature = "ai"))]
pub struct KnowledgeLibrary;

#[cfg(not(feature = "ai"))]
impl KnowledgeLibrary {
    pub fn new() -> Self {
        Self
    }
}
#[cfg(not(feature = "ai"))]
pub struct ContextSelector;

#[cfg(not(feature = "ai"))]
impl ContextSelector {
    pub fn new(_knowledge_library: std::sync::Arc<KnowledgeLibrary>) -> Result<Self, UveddiError> {
        Ok(Self)
    }
}

// Knowledge Library imports (feature-gated)
#[cfg(feature = "ai")]
use crate::ai::engine::AiAnalysisEngine;
#[cfg(feature = "ai")]
use crate::ai::knowledge::{ContextSelector, KnowledgeLibrary};

/// Builder for `AnalysisEngine` to provide flexible and consistent construction.
///
/// This builder supports both synchronous and asynchronous construction paths,
/// allowing for configuration of detectors, cache paths, and plugin support.
///
/// # Examples
///
/// ## Synchronous build
/// ```no_run
/// use uveddi::analysis::AnalysisEngine;
/// use std::path::PathBuf;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let engine = AnalysisEngine::builder()
///         .with_cache_path(&PathBuf::from("my_cache.db"))
///         .build()?;
///     Ok(())
/// }
/// ```
///
/// ## Asynchronous build with plugins
/// ```no_run
/// use uveddi::analysis::AnalysisEngine;
/// use std::path::PathBuf;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let engine = AnalysisEngine::builder()
///         .enable_plugins(true)
///         .build_async()
///         .await?;
///     Ok(())
/// }
/// ```
pub struct AnalysisEngineBuilder {
    detectors: Option<Vec<Box<dyn AnalysisDetector + Send + Sync>>>,
    cache_path: Option<PathBuf>,
    enable_plugins: bool,
    in_memory_cache: bool,
    // Injected dependencies for advanced use cases
    injected_ast_parser: Option<Box<dyn AstParserTrait>>,
    injected_dependency_extractor: Option<Box<dyn DependencyExtractorTrait>>,
    injected_result_cache: Option<Box<dyn ResultCacheTrait>>,

    // Knowledge Library options
    enable_knowledge_enhancement: bool,
    enable_ai_explanations: bool,
    knowledge_library_path: Option<PathBuf>,
}

impl Default for AnalysisEngineBuilder {
    fn default() -> Self {
        Self {
            detectors: None,
            cache_path: None,
            enable_plugins: false,
            in_memory_cache: false,
            injected_ast_parser: None,
            injected_dependency_extractor: None,
            injected_result_cache: None,

            // Knowledge Library defaults
            enable_knowledge_enhancement: false,
            enable_ai_explanations: false,
            knowledge_library_path: None,
        }
    }
}

impl AnalysisEngineBuilder {
    /// Creates a new `AnalysisEngineBuilder` instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets custom detectors for the engine. If not set, default detectors will be used.
    pub fn with_detectors(
        mut self,
        detectors: Vec<Box<dyn AnalysisDetector + Send + Sync>>,
    ) -> Self {
        self.detectors = Some(detectors);
        self
    }

    /// Sets the path for the cache database. If not set, a default path will be used.
    pub fn with_cache_path(mut self, path: &Path) -> Self {
        self.cache_path = Some(path.to_path_buf());
        self
    }

    /// Configures the engine to use an in-memory cache. This overrides any `cache_path` set.
    pub fn with_in_memory_cache(mut self) -> Self {
        self.in_memory_cache = true;
        self
    }

    /// Enables or disables WASM plugin support.
    pub fn enable_plugins(mut self, enable: bool) -> Self {
        self.enable_plugins = enable;
        self
    }

    /// Injects a custom AST parser implementation.
    pub fn with_injected_ast_parser(mut self, parser: Box<dyn AstParserTrait>) -> Self {
        self.injected_ast_parser = Some(parser);
        self
    }

    /// Injects a custom dependency extractor implementation.
    pub fn with_injected_dependency_extractor(
        mut self,
        extractor: Box<dyn DependencyExtractorTrait>,
    ) -> Self {
        self.injected_dependency_extractor = Some(extractor);
        self
    }

    /// Injects a custom result cache implementation.
    pub fn with_injected_result_cache(mut self, cache: Box<dyn ResultCacheTrait>) -> Self {
        self.injected_result_cache = Some(cache);
        self
    }

    /// Enable knowledge library integration for enhanced analysis
    pub fn with_knowledge_library(mut self, enable: bool) -> Self {
        self.enable_knowledge_enhancement = enable;
        self
    }

    /// Enable AI explanations (requires knowledge library to be enabled)
    pub fn with_ai_explanations(mut self, enable: bool) -> Self {
        self.enable_ai_explanations = enable;
        self
    }

    /// Set a custom path for the knowledge library
    pub fn with_knowledge_library_path(mut self, path: &Path) -> Self {
        self.knowledge_library_path = Some(path.to_path_buf());
        self
    }

    /// Builds the `AnalysisEngine` synchronously.
    ///
    /// This method should be used when no asynchronous operations (like plugin loading)
    /// are required during engine initialization.
    ///
    /// # Errors
    /// Returns `UveddiError` if initialization fails.
    pub fn build(self) -> crate::error::Result<crate::analysis::AnalysisEngine> {
        if self.enable_plugins {
            warn!("Plugins enabled, but building synchronously. Use `build_async().await` for proper plugin initialization.");
        }

        // Extract all values before using self methods to avoid partial moves
        let enable_knowledge = self.enable_knowledge_enhancement;
        let enable_ai = self.enable_ai_explanations;
        let injected_result_cache = self.injected_result_cache;
        let in_memory_cache = self.in_memory_cache;
        let cache_path = self.cache_path;
        let injected_ast_parser = self.injected_ast_parser;
        let injected_dependency_extractor = self.injected_dependency_extractor;

        let detectors = self
            .detectors
            .unwrap_or_else(DetectorFactory::create_default_detectors);

        let _cache = if let Some(injected_cache) = injected_result_cache {
            injected_cache
        } else if in_memory_cache {
            Box::new(ResultCacheAdapter::new(ResultCache::new_in_memory()?))
        } else if let Some(path) = cache_path {
            Box::new(ResultCacheAdapter::new(ResultCache::new(&path)?))
        } else {
            Box::new(ResultCacheAdapter::new(ResultCache::new(&PathBuf::from(
                "uveddi_cache.db",
            ))?))
        };

        // Initialize AST cache with default configuration
        let ast_cache_config = CacheConfig::default();
        let ast_cache = AstCache::new(ast_cache_config)?;

        // Initialize components
        let config_service = Arc::new(ConfigurationService::new());
        let ast_provider = if let Some(_parser) = injected_ast_parser {
            Arc::new(AstProviderImpl::new()?)
        } else {
            Arc::new(AstProviderImpl::new()?)
        };
        let cache_manager = Arc::new(futures::executor::block_on(
            CacheManagerImpl::with_ast_cache(ast_cache),
        )?);
        let aggregator = Arc::new(AnalysisAggregator::new());

        // Components that need dependencies
        let dependency_builder = if let Some(_extractor) = injected_dependency_extractor {
            Arc::new(DependencyGraphBuilderImpl::new(ast_provider.clone())?)
        } else {
            Arc::new(DependencyGraphBuilderImpl::new(ast_provider.clone())?)
        };

        let detector_scheduler = Arc::new(DetectorScheduler::new(
            config_service.clone(),
            ast_provider.clone(),
            None, // plugin_manager not initialized in sync build
            aggregator.clone(),
            detectors,
        ));

        // Initialize knowledge library components if enabled
        let knowledge_library_result = if enable_knowledge {
            match self.knowledge_library_path {
                Some(ref custom_path) => {
                    info!(
                        "Loading knowledge library from custom path: {:?}",
                        custom_path
                    );
                    // TODO: Implement custom path loading
                    Ok(KnowledgeLibrary::new())
                }
                None => {
                    info!("Loading default knowledge library");
                    // For now, create an empty knowledge library
                    // TODO: Load from bundled knowledge base
                    Ok(KnowledgeLibrary::new())
                }
            }
        } else {
            Err(UveddiError::config_error(
                "Knowledge library disabled",
                "builder",
            ))
        };

        let (_knowledge_library, _context_selector, _ai_engine) = if enable_knowledge {
            let knowledge_lib = match knowledge_library_result {
                Ok(lib) => Some(Arc::new(lib)),
                Err(e) => {
                    warn!("Failed to initialize knowledge library: {}. Disabling knowledge enhancement.", e);
                    None
                }
            };

            let context_sel = if let Some(ref lib) = knowledge_lib {
                match ContextSelector::new(Arc::clone(lib)) {
                    Ok(selector) => Some(Arc::new(selector)),
                    Err(e) => {
                        warn!("Failed to create context selector: {}. Disabling knowledge enhancement.", e);
                        None
                    }
                }
            } else {
                None
            };

            let ai_eng = if enable_ai {
                Some(Arc::new(AiAnalysisEngine::new()))
            } else {
                None
            };

            (knowledge_lib, context_sel, ai_eng)
        } else {
            (None, None, None)
        };

        // Create services for the orchestrator
        let detector_factory = Arc::new(crate::analysis::detector_factory::DetectorFactory::new());

        let analysis_service = Arc::new(crate::analysis::services::AnalysisService::new(
            Arc::clone(&config_service),
            Arc::clone(&detector_scheduler),
            Arc::clone(&aggregator),
            None, // plugin_manager: Option<Arc<PluginManagerHandle>>
            Arc::clone(&detector_factory),
        ));

        let dependency_service =
            Arc::new(crate::analysis::services::DependencyAnalysisService::new(
                Arc::clone(&ast_provider),
                Arc::clone(&dependency_builder),
                Arc::clone(&cache_manager),
            ));

        let performance_service = Arc::new(crate::analysis::services::PerformanceAnalysisService::new(
            Arc::new(crate::monitoring::performance_metrics_collector::PerformanceMetricsCollector::new(
                crate::database::models::PerformanceMetricsConfig::default(),
                10, // total_components estimate
            )),
            crate::analysis::services::performance_service::MemoryConfig::default(),
        ));

        // Create the orchestrator
        let orchestrator = crate::analysis::orchestrator::AnalysisOrchestrator::new(
            analysis_service,
            dependency_service,
            performance_service,
        );

        Ok(crate::analysis::AnalysisEngine {
            orchestrator,
            config_service,
            ast_provider,
            cache_manager,
            dependency_builder,
            detector_scheduler,
            plugin_manager: None, // No plugin manager in sync build
            aggregator,

            // Knowledge Library components
            #[cfg(feature = "ai")]
            knowledge_library,
            #[cfg(feature = "ai")]
            context_selector,
            #[cfg(feature = "ai")]
            ai_engine,
            enable_knowledge_enhancement: enable_knowledge,
            enable_ai_explanations: enable_ai,
        })
    }

    /// Builds the `AnalysisEngine` asynchronously.
    ///
    /// This method should be used when asynchronous operations (like plugin loading)
    /// are required during engine initialization.
    ///
    /// # Errors
    /// Returns `UveddiError` if initialization fails.
    pub async fn build_async(self) -> crate::error::Result<crate::analysis::AnalysisEngine> {
        // Extract all values before using self methods to avoid partial moves
        let enable_knowledge = self.enable_knowledge_enhancement;
        let enable_ai = self.enable_ai_explanations;
        let enable_plugins = self.enable_plugins;
        let injected_result_cache = self.injected_result_cache;
        let in_memory_cache = self.in_memory_cache;
        let cache_path = self.cache_path;
        let injected_ast_parser = self.injected_ast_parser;
        let injected_dependency_extractor = self.injected_dependency_extractor;

        let detectors = self
            .detectors
            .unwrap_or_else(DetectorFactory::create_default_detectors);

        let _cache = if let Some(injected_cache) = injected_result_cache {
            injected_cache
        } else if in_memory_cache {
            Box::new(ResultCacheAdapter::new(ResultCache::new_in_memory()?))
        } else if let Some(path) = cache_path {
            Box::new(ResultCacheAdapter::new(ResultCache::new(&path)?))
        } else {
            Box::new(ResultCacheAdapter::new(ResultCache::new(&PathBuf::from(
                "uveddi_cache.db",
            ))?))
        };

        // Initialize plugin engine if enabled
        let plugin_engine = if enable_plugins {
            match WasmPluginEngine::new().await {
                Ok(engine) => {
                    info!("WASM plugin engine initialized successfully");
                    Some(engine)
                }
                Err(e) => {
                    warn!(
                        "Failed to initialize WASM plugin engine: {}. Continuing without plugins.",
                        e
                    );
                    None
                }
            }
        } else {
            None
        };

        // Initialize AST cache with default configuration
        let ast_cache_config = CacheConfig::default();
        let ast_cache = AstCache::new(ast_cache_config)?;

        // Initialize components
        let config_service = Arc::new(ConfigurationService::new());
        let ast_provider = if let Some(_parser) = injected_ast_parser {
            Arc::new(AstProviderImpl::new()?)
        } else {
            Arc::new(AstProviderImpl::new()?)
        };
        let cache_manager = Arc::new(futures::executor::block_on(
            CacheManagerImpl::with_ast_cache(ast_cache),
        )?);
        let aggregator = Arc::new(AnalysisAggregator::new());

        // PluginManagerHandle needs to be created from WasmPluginEngine
        let plugin_manager = if let Some(ref _engine) = plugin_engine {
            Some(PluginManager::spawn(config_service.clone()))
        } else {
            None
        };

        // Components that need dependencies
        let dependency_builder = if let Some(_extractor) = injected_dependency_extractor {
            Arc::new(DependencyGraphBuilderImpl::new(ast_provider.clone())?)
        } else {
            Arc::new(DependencyGraphBuilderImpl::new(ast_provider.clone())?)
        };

        let detector_scheduler = Arc::new(DetectorScheduler::new(
            config_service.clone(),
            ast_provider.clone(),
            plugin_manager.clone(), // Pass plugin_manager to detector_scheduler
            aggregator.clone(),
            detectors,
        ));

        // Create detector factory and performance metrics collector
        let detector_factory = DetectorFactory;
        let performance_metrics_collector = Arc::new(PerformanceMetricsCollector::new(
            crate::database::models::PerformanceMetricsConfig::default(),
            100, // Default total components
        ));

        // Initialize knowledge library components if enabled
        let knowledge_library_result = if enable_knowledge {
            match self.knowledge_library_path {
                Some(ref custom_path) => {
                    info!(
                        "Loading knowledge library from custom path: {:?}",
                        custom_path
                    );
                    // TODO: Implement custom path loading
                    Ok(KnowledgeLibrary::new())
                }
                None => {
                    info!("Loading default knowledge library");
                    // For now, create an empty knowledge library
                    // TODO: Load from bundled knowledge base
                    Ok(KnowledgeLibrary::new())
                }
            }
        } else {
            Err(UveddiError::config_error(
                "Knowledge library disabled",
                "builder",
            ))
        };

        let (_knowledge_library, _context_selector, _ai_engine) = if enable_knowledge {
            let knowledge_lib = match knowledge_library_result {
                Ok(lib) => Some(Arc::new(lib)),
                Err(e) => {
                    warn!("Failed to initialize knowledge library: {}. Disabling knowledge enhancement.", e);
                    None
                }
            };

            let context_sel = if let Some(ref lib) = knowledge_lib {
                match ContextSelector::new(Arc::clone(lib)) {
                    Ok(selector) => Some(Arc::new(selector)),
                    Err(e) => {
                        warn!("Failed to create context selector: {}. Disabling knowledge enhancement.", e);
                        None
                    }
                }
            } else {
                None
            };

            let ai_eng = if enable_ai {
                Some(Arc::new(AiAnalysisEngine::new()))
            } else {
                None
            };

            (knowledge_lib, context_sel, ai_eng)
        } else {
            (None, None, None)
        };

        info!(
            "Knowledge library integration: enabled={}, AI explanations: enabled={}",
            enable_knowledge, enable_ai
        );

        crate::analysis::AnalysisEngine::from_components(
            config_service,
            ast_provider,
            cache_manager,
            dependency_builder,
            detector_scheduler,
            plugin_manager,
            aggregator,
            Arc::new(detector_factory),
            performance_metrics_collector,
            enable_knowledge,
            enable_ai,
            #[cfg(feature = "ai")]
            knowledge_library,
            #[cfg(feature = "ai")]
            context_selector,
            #[cfg(feature = "ai")]
            ai_engine,
        )
        .map_err(|e| crate::error::UveddiError::AnalysisError {
            file: "engine_builder.rs".to_string(),
            line: 503,
            message: e.to_string(),
            context: "Building analysis engine from components".to_string(),
            suggestion: "Check component configuration and dependencies".to_string(),
            source: None,
        })
    }
}
