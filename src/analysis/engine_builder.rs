use std::path::{Path, PathBuf};
use std::sync::Arc;
use log::{info, warn};

use crate::analysis::cache::ast::{AstCache, CacheConfig};
use crate::analysis::components::{
    AnalysisAggregator, AstProviderImpl, CacheManagerImpl, ConfigurationService,
    DependencyGraphBuilderImpl, DetectorScheduler, PluginManagerHandle, PluginManager,
};
use crate::analysis::adapters::ResultCacheAdapter;
use crate::analysis::detector_factory::DetectorFactory;
use crate::analysis::traits::{AstParserTrait, DependencyExtractorTrait, ResultCacheTrait};
use crate::analysis::AnalysisDetector;
use crate::ast::tree_sitter_impl::AstParser;
use crate::cache::result_cache::ResultCache;
use crate::error::UveddiError;
use crate::plugins::WasmPluginEngine;

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

        let detectors = self
            .detectors
            .unwrap_or_else(DetectorFactory::create_default_detectors);

        let cache = if let Some(injected_cache) = self.injected_result_cache {
            injected_cache
        } else if self.in_memory_cache {
            Box::new(ResultCacheAdapter::new(ResultCache::new_in_memory()?))
        } else if let Some(path) = self.cache_path {
            Box::new(ResultCacheAdapter::new(ResultCache::new(&path)?))
        } else {
            Box::new(ResultCacheAdapter::new(ResultCache::new(&PathBuf::from("uveddi_cache.db"))?))
        };

        // Initialize AST cache with default configuration
        let ast_cache_config = CacheConfig::default();
        let ast_cache = AstCache::new(ast_cache_config)?;

        // Initialize components
        let config_service = Arc::new(ConfigurationService::new());
        let ast_provider = if let Some(parser) = self.injected_ast_parser {
            Arc::new(AstProviderImpl::new()?)
        } else {
            Arc::new(AstProviderImpl::new()?)
        };
        let cache_manager = Arc::new(CacheManagerImpl::with_ast_cache(ast_cache));
        let aggregator = Arc::new(AnalysisAggregator::new());

        // Components that need dependencies
        let dependency_builder = if let Some(extractor) = self.injected_dependency_extractor {
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

        Ok(crate::analysis::AnalysisEngine {
            config_service,
            ast_provider,
            cache_manager,
            dependency_builder,
            detector_scheduler,
            plugin_manager: None, // No plugin manager in sync build
            aggregator,
            // No plugin engine field needed - removed per UV-294
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
        let detectors = self
            .detectors
            .unwrap_or_else(DetectorFactory::create_default_detectors);

        let cache = if let Some(injected_cache) = self.injected_result_cache {
            injected_cache
        } else if self.in_memory_cache {
            Box::new(ResultCacheAdapter::new(ResultCache::new_in_memory()?))
        } else if let Some(path) = self.cache_path {
            Box::new(ResultCacheAdapter::new(ResultCache::new(&path)?))
        } else {
            Box::new(ResultCacheAdapter::new(ResultCache::new(&PathBuf::from("uveddi_cache.db"))?))
        };

        // Initialize plugin engine if enabled
        let plugin_engine = if self.enable_plugins {
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
        let ast_provider = if let Some(parser) = self.injected_ast_parser {
            Arc::new(AstProviderImpl::new()?)
        } else {
            Arc::new(AstProviderImpl::new()?)
        };
        let cache_manager = Arc::new(CacheManagerImpl::with_ast_cache(ast_cache));
        let aggregator = Arc::new(AnalysisAggregator::new());

        // PluginManagerHandle needs to be created from WasmPluginEngine
        let plugin_manager = if let Some(ref engine) = plugin_engine {
            Some(PluginManager::spawn(config_service.clone()))
        } else {
            None
        };

        // Components that need dependencies
        let dependency_builder = if let Some(extractor) = self.injected_dependency_extractor {
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

        Ok(crate::analysis::AnalysisEngine {
            config_service,
            ast_provider,
            cache_manager,
            dependency_builder,
            detector_scheduler,
            plugin_manager,
            aggregator,
        })
    }
}
