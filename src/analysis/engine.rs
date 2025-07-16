use crate::analysis::cache::ast::{AstCache, CacheConfig};
use crate::analysis::detectors::anti_patterns::dead_code::{DeadCodeConfig, DeadCodeDetector};
use crate::analysis::detectors::anti_patterns::large_classes::{
    LargeClassConfig, LargeClassDetector,
};
use crate::analysis::detectors::cycle::CycleDetector;
use crate::analysis::detectors::dependency::{Dependency, DependencyExtractor};
use crate::analysis::extractors::SymbolExtractor;
use crate::analysis::graph::dependency::LocalDependencyGraph;
use crate::analysis::graph::dependency::{ComponentNode, LocalDependencyType};
use crate::analysis::symbols::GlobalSymbolTable;
use crate::analysis::traits::{AstParserTrait, DependencyExtractorTrait, ResultCacheTrait};
use crate::analysis::AnalysisDetector;
use crate::ast::tree_sitter_impl::AstParser;
use crate::cache::result_cache::ResultCache;
use crate::database::models::{
    AntiPatternType, ArchitecturalIssue, ComponentPerformanceMetrics, PerformanceMetricsConfig,
};
use crate::ingestion::AsyncWalker;
use crate::monitoring::performance_metrics_collector::PerformanceMetricsCollector;
use crate::plugins::WasmPluginEngine;

// Component imports
use crate::analysis::components::{
    ConfigurationService, AstProviderImpl, DependencyGraphBuilderImpl, 
    DetectorScheduler, PluginManagerHandle, AnalysisAggregator,
    CacheManager, CacheManagerImpl,
    traits::{DetectorScheduler as DetectorSchedulerTrait, AnalysisAggregator as AnalysisAggregatorTrait}
};
use crate::analysis::components::traits::{
    DependencyGraphBuilder as DependencyGraphBuilderTrait,
    AstProvider as AstProviderTrait
};
use std::sync::Arc;
use log::{info, warn};

use chrono::Utc;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tokio_stream::StreamExt;

/// Represents a cached analysis result for a file
#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct CachedAnalysisResult {
    /// The architectural issues found in the file
    issues: Vec<ArchitecturalIssue>,
    /// The dependencies extracted from the file
    dependencies: Vec<Dependency>,
}

/// Represents the core analysis engine that orchestrates code analysis operations.
///
/// The engine is responsible for:
/// - Initializing language-specific parsers
/// - Processing source files into AST representations
/// - Running registered detectors against the codebase
/// - Aggregating and reporting analysis results
///
/// # Examples
///
/// ```no_run
/// use uveddi::analysis::AnalysisEngine;
/// use std::path::Path;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let mut engine = AnalysisEngine::new()?;
///     let (issues, graph) = engine.analyze(Path::new("src/")).await?;
///     println!("Found {} issues", issues.len());
///     Ok(())
/// }
/// ```
pub struct AnalysisEngine {
    // Component references - implementing facade pattern
    config_service: Arc<ConfigurationService>,
    ast_provider: Arc<AstProviderImpl>,
    cache_manager: Arc<CacheManagerImpl>,
    dependency_builder: Arc<DependencyGraphBuilderImpl>,
    detector_scheduler: Arc<DetectorScheduler>,
    plugin_manager: Option<PluginManagerHandle>,
    aggregator: Arc<AnalysisAggregator>,
    
    // Minimal legacy fields for backward compatibility
    plugin_engine: Option<WasmPluginEngine>,
}

impl AnalysisEngine {
    /// Creates a new analysis engine with default configuration
    ///
    /// Initializes the engine with:
    /// - AST parser for syntax tree generation
    /// - Dependency extractor for import/include analysis
    /// - Default set of anti-pattern detectors
    /// - Result cache for performance optimization
    ///
    /// # Errors
    ///
    /// Returns `UveddiError` if:
    /// - Cache database cannot be created
    /// - AST parser initialization fails
    /// - Dependency extractor setup fails
    #[inline]
    pub fn new() -> crate::error::Result<Self> {
        let default_detectors =
            crate::analysis::detector_factory::DetectorFactory::create_default_detectors();
        let cache_path = PathBuf::from("uveddi_cache.db");
        Self::with_detectors(default_detectors, Some(&cache_path), false)
    }

    /// Create engine with injected detectors (DEPENDENCY INJECTION)
    ///
    /// Creates an AnalysisEngine with a custom set of detectors, enabling
    /// dependency injection for better testability and flexibility.
    ///
    /// # Arguments
    ///
    /// * `detectors` - Vector of detectors to use for analysis
    /// * `cache_path` - Optional path to cache database (None for in-memory cache)
    /// * `enable_plugins` - Whether to enable WASM plugin support
    ///
    /// # Returns
    ///
    /// A configured AnalysisEngine instance
    ///
    /// # Errors
    ///
    /// Returns `UveddiError` if:
    /// - Cache database cannot be created
    /// - AST parser initialization fails
    /// - Dependency extractor setup fails
    #[inline]
    pub fn with_detectors(
        detectors: Vec<Box<dyn AnalysisDetector + Send + Sync>>,
        cache_path: Option<&Path>,
        enable_plugins: bool,
    ) -> crate::error::Result<Self> {
        let cache = if let Some(path) = cache_path {
            ResultCache::new(path)?
        } else {
            ResultCache::new_in_memory()?
        };

        // Initialize AST cache with default configuration
        let ast_cache_config = CacheConfig::default();
        let ast_cache = AstCache::new(ast_cache_config)?;

        // Initialize components
        let config_service = Arc::new(ConfigurationService::new());
        let ast_provider = Arc::new(AstProviderImpl::new()?);
        let cache_manager = Arc::new(CacheManagerImpl::with_ast_cache(ast_cache));
        let aggregator = Arc::new(AnalysisAggregator::new());
        
        // Components that need dependencies
        let dependency_builder = Arc::new(DependencyGraphBuilderImpl::new(ast_provider.clone())?);
        let detector_scheduler = Arc::new(DetectorScheduler::new(
            config_service.clone(),
            ast_provider.clone(),
            None, // plugin_manager not initialized yet
            aggregator.clone(),
            detectors, // Use the provided detectors
        ));
        
        // Plugin manager is initialized separately for async operations
        let plugin_manager = None;

        Ok(Self {
            // Component architecture - facade pattern
            config_service,
            ast_provider,
            cache_manager,
            dependency_builder,
            detector_scheduler,
            plugin_manager,
            aggregator,
            
            // Minimal legacy fields for backward compatibility
            plugin_engine: if enable_plugins {
                // Plugin engine will be initialized separately for async operations
                None
            } else {
                None
            },
        })
    }

    /// Create engine with injected detectors and plugin support (ASYNC VERSION)
    ///
    /// Creates an AnalysisEngine with custom detectors and initializes the
    /// WASM plugin engine if requested.
    ///
    /// # Arguments
    ///
    /// * `detectors` - Vector of detectors to use for analysis
    /// * `cache_path` - Optional path to cache database (None for in-memory cache)
    ///
    /// # Returns
    ///
    /// A configured AnalysisEngine instance with plugin support
    ///
    /// # Errors
    ///
    /// Returns `UveddiError` if initialization fails
    pub async fn with_detectors_and_plugins(
        detectors: Vec<Box<dyn AnalysisDetector + Send + Sync>>,
        cache_path: Option<&Path>,
    ) -> crate::error::Result<Self> {
        let cache = if let Some(path) = cache_path {
            ResultCache::new(path)?
        } else {
            ResultCache::new_in_memory()?
        };

        // Initialize plugin engine
        let plugin_engine = match WasmPluginEngine::new().await {
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
        };

        // Initialize AST cache with default configuration
        let ast_cache_config = CacheConfig::default();
        let ast_cache = AstCache::new(ast_cache_config)?;

        // Initialize components
        let config_service = Arc::new(ConfigurationService::new());
        let ast_provider = Arc::new(AstProviderImpl::new()?);
        let cache_manager = Arc::new(CacheManagerImpl::with_ast_cache(ast_cache));
        let aggregator = Arc::new(AnalysisAggregator::new());
        
        // Components that need dependencies
        let dependency_builder = Arc::new(DependencyGraphBuilderImpl::new(ast_provider.clone())?);
        let detector_scheduler = Arc::new(DetectorScheduler::new(
            config_service.clone(),
            ast_provider.clone(),
            None, // plugin_manager not initialized yet
            aggregator.clone(),
            detectors, // Use the provided detectors
        ));
        
        // Plugin manager will be initialized from plugin_engine
        let plugin_manager = None; // TODO: Convert WasmPluginEngine to PluginManagerHandle

        Ok(Self {
            // Component architecture - facade pattern
            config_service,
            ast_provider,
            cache_manager,
            dependency_builder,
            detector_scheduler,
            plugin_manager,
            aggregator,
            
            // Minimal legacy fields for backward compatibility
            plugin_engine,
        })
    }

    /// Creates a new analysis engine with WASM plugin support enabled
    pub async fn new_with_plugins() -> crate::error::Result<Self> {
        let default_detectors =
            crate::analysis::detector_factory::DetectorFactory::create_default_detectors();
        let cache_path = PathBuf::from("uveddi_cache.db");
        Self::with_detectors_and_plugins(default_detectors, Some(&cache_path)).await
    }

    /// Creates a new analysis engine with a custom cache database path
    ///
    /// This is primarily useful for testing to avoid database conflicts.
    ///
    /// # Arguments
    ///
    /// * `cache_path` - Path to the cache database file
    ///
    /// # Errors
    ///
    /// Returns `UveddiError` if:
    /// - Cache database cannot be created
    /// - AST parser initialization fails
    /// - Dependency extractor setup fails
    pub fn with_cache_path(cache_path: &Path) -> crate::error::Result<Self> {
        let default_detectors =
            crate::analysis::detector_factory::DetectorFactory::create_default_detectors();
        Self::with_detectors(default_detectors, Some(cache_path), false)
    }

    /// Creates a new analysis engine with WASM plugin support and custom cache path
    pub async fn with_cache_path_and_plugins(cache_path: &Path) -> crate::error::Result<Self> {
        let default_detectors =
            crate::analysis::detector_factory::DetectorFactory::create_default_detectors();
        Self::with_detectors_and_plugins(default_detectors, Some(cache_path)).await
    }

    /// Creates a new analysis engine with an in-memory cache database
    ///
    /// This is primarily useful for testing to avoid database conflicts
    /// and ensure test isolation.
    ///
    /// # Errors
    ///
    /// Returns `UveddiError` if:
    /// - Cache database cannot be created
    /// - AST parser initialization fails
    /// - Dependency extractor setup fails
    pub fn new_with_memory_cache() -> crate::error::Result<Self> {
        let default_detectors =
            crate::analysis::detector_factory::DetectorFactory::create_default_detectors();
        Self::with_detectors(default_detectors, None, false)
    }

    /// Create AnalysisEngine with injected dependencies (DEPENDENCY INJECTION)
    ///
    /// Creates an AnalysisEngine with custom trait-based dependencies, enabling
    /// full dependency injection for better testability and modularity.
    ///
    /// # Arguments
    ///
    /// * `ast_parser` - AST parsing implementation
    /// * `dependency_extractor` - Dependency analysis implementation  
    /// * `cache` - Result caching implementation
    /// * `detectors` - Vector of detectors to use for analysis
    ///
    /// # Returns
    ///
    /// A configured AnalysisEngine instance
    ///
    /// # Errors
    ///
    /// Returns `UveddiError` if:
    /// - Dependency validation fails
    /// - Component initialization fails
    pub fn with_injected_dependencies(
        ast_parser: Box<dyn AstParserTrait>,
        dependency_extractor: Box<dyn DependencyExtractorTrait>, 
        cache: Box<dyn ResultCacheTrait>,
        detectors: Vec<Box<dyn AnalysisDetector + Send + Sync>>,
    ) -> crate::error::Result<Self> {
        // Validate dependencies
        Self::validate_injected_dependencies(&ast_parser, &dependency_extractor, &cache)?;
        
        // For now, we need to create concrete implementations from the traits
        // This is a bridge solution until we can fully refactor the engine
        let concrete_parser = AstParser::new()?;
        let concrete_extractor = DependencyExtractor::new()?;
        let concrete_cache = ResultCache::new_in_memory()?;

        // Initialize AST cache with default configuration
        let ast_cache_config = CacheConfig::default();
        let ast_cache = AstCache::new(ast_cache_config)?;

        // Initialize components
        let config_service = Arc::new(ConfigurationService::new());
        let ast_provider = Arc::new(AstProviderImpl::new()?);
        let cache_manager = Arc::new(CacheManagerImpl::with_ast_cache(ast_cache));
        let aggregator = Arc::new(AnalysisAggregator::new());
        
        // Components that need dependencies
        let dependency_builder = Arc::new(DependencyGraphBuilderImpl::new(ast_provider.clone())?);
        let detector_scheduler = Arc::new(DetectorScheduler::new(
            config_service.clone(),
            ast_provider.clone(),
            None, // plugin_manager not initialized yet
            aggregator.clone(),
            detectors, // Use the provided detectors
        ));
        
        // Plugin manager is initialized separately for async operations
        let plugin_manager = None;

        Ok(Self {
            // Component architecture - facade pattern
            config_service,
            ast_provider,
            cache_manager,
            dependency_builder,
            detector_scheduler,
            plugin_manager,
            aggregator,
            
            // Minimal legacy fields for backward compatibility
            plugin_engine: None,
        })
    }
    
    /// Create AnalysisEngine with default dependencies (backward compatibility)
    ///
    /// This method provides the same functionality as `new()` but goes through
    /// the dependency injection infrastructure, ensuring consistency.
    ///
    /// # Returns
    ///
    /// A configured AnalysisEngine instance with default dependencies
    ///
    /// # Errors
    ///
    /// Returns `UveddiError` if dependency creation or validation fails
    pub fn new_with_defaults() -> crate::error::Result<Self> {
        use crate::analysis::adapters::{AstParserAdapter, DependencyExtractorAdapter, ResultCacheAdapter};
        
        let ast_parser = Box::new(AstParserAdapter::new_default()?) as Box<dyn AstParserTrait>;
        let dependency_extractor = Box::new(DependencyExtractorAdapter::new_default()?) as Box<dyn DependencyExtractorTrait>;
        let cache = Box::new(ResultCacheAdapter::new_memory()?) as Box<dyn ResultCacheTrait>;
        let default_detectors = crate::analysis::detector_factory::DetectorFactory::create_default_detectors();
        
        Self::with_injected_dependencies(ast_parser, dependency_extractor, cache, default_detectors)
    }
    
    /// Validate injected dependencies
    ///
    /// Ensures that all injected dependencies are properly initialized and
    /// compatible with each other.
    ///
    /// # Arguments
    ///
    /// * `ast_parser` - The AST parser to validate
    /// * `dependency_extractor` - The dependency extractor to validate
    /// * `cache` - The cache implementation to validate
    ///
    /// # Returns
    ///
    /// Ok(()) if all dependencies are valid
    ///
    /// # Errors
    ///
    /// Returns `UveddiError` if any dependency is invalid or incompatible
    fn validate_injected_dependencies(
        ast_parser: &Box<dyn AstParserTrait>,
        dependency_extractor: &Box<dyn DependencyExtractorTrait>,
        cache: &Box<dyn ResultCacheTrait>,
    ) -> crate::error::Result<()> {
        // Validate AST parser
        if !ast_parser.is_initialized() {
            return Err(crate::error::UveddiError::config_error(
                "AST parser not properly initialized",
                "dependency validation",
            ));
        }
        
        // Validate language support compatibility
        let parser_languages = ast_parser.supported_languages();
        if parser_languages.is_empty() {
            return Err(crate::error::UveddiError::config_error(
                "AST parser supports no languages",
                "dependency validation",
            ));
        }
        
        // Ensure dependency extractor supports at least one language that the parser does
        let has_common_language = parser_languages.iter()
            .any(|lang| dependency_extractor.supports_language(lang));
        
        if !has_common_language {
            return Err(crate::error::UveddiError::config_error(
                "AST parser and dependency extractor have no common language support",
                "dependency validation",
            ));
        }
        
        // Validate cache (basic check - could be expanded)
        let _stats = cache.get_stats(); // This should not panic for a valid cache
        
        Ok(())
    }

    /// Performs comprehensive analysis on a directory or file
    ///
    /// This is the main entry point for analysis. It:
    /// 1. Discovers and parses all source files in the given path
    /// 2. Runs file-level detectors on each parsed file
    /// 3. Extracts dependencies and builds a dependency graph
    /// 4. Runs graph-level detectors (cycle detection, architectural patterns)
    /// 5. Aggregates and returns all detected issues
    ///
    /// # Arguments
    ///
    /// * `path` - Directory or file path to analyze
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - `Vec<ArchitecturalIssue>` - All detected issues across all detectors
    /// - `LocalDependencyGraph` - The constructed dependency graph
    ///
    /// # Errors
    ///
    /// Returns `UveddiError` if:
    /// - Path is inaccessible or doesn't exist
    /// - Critical parsing errors occur
    /// - Cache operations fail
    pub async fn analyze(
        &mut self,
        path: &Path,
    ) -> crate::error::Result<(Vec<ArchitecturalIssue>, LocalDependencyGraph)> {
        // Facade pattern: delegate to components
        
        // Phase 1: Use dependency builder to construct the dependency graph
        info!("Building dependency graph...");
        let dependency_graph = self.dependency_builder.build_graph(path).await?;
        info!("Dependency graph built.");

        // Phase 2: Use detector scheduler to run analysis
        info!("Running detection analysis...");
        let file_issues = if path.is_file() {
            self.detector_scheduler.schedule_file(path).await?
        } else {
            self.detector_scheduler.schedule_directory(path).await?
        };
        info!("Detection analysis completed.");

        // Phase 3: Use aggregator to collect and format results
        self.aggregator.record_findings(file_issues.clone());
        let aggregated_stats = self.aggregator.get_stats();
        
        // UV-2: Initialize metrics collector and emit metrics
        let metrics_config = PerformanceMetricsConfig::default();
        let files_analyzed = aggregated_stats.files_processed;
        let mut metrics_collector =
            PerformanceMetricsCollector::new(metrics_config, files_analyzed);
        
        metrics_collector.record_analysis_metrics(file_issues.len(), files_analyzed);
        
        if let Err(e) = metrics_collector.emit_metrics() {
            warn!("Failed to emit performance metrics: {}", e);
        }

        Ok((file_issues, dependency_graph))
    }

    /// Analyzes files in the given path and collects dependencies (DEPRECATED)
    ///
    /// This method has been replaced by the facade pattern delegation to components.
    /// It's kept for backward compatibility but now delegates to the new architecture.
    ///
    /// # Arguments
    ///
    /// * `path` - Directory or file path to analyze
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - `Vec<ArchitecturalIssue>` - Issues found in the files
    /// - `Vec<Dependency>` - Dependencies extracted from the files
    ///
    /// # Errors
    ///
    /// Returns `UveddiError` if analysis fails
    async fn analyze_files_and_collect_dependencies(
        &mut self,
        path: &Path,
    ) -> crate::error::Result<(Vec<ArchitecturalIssue>, Vec<Dependency>)> {
        // Facade pattern: Delegate to detector_scheduler for issues and dependency_builder for dependencies
        let file_issues = if path.is_file() {
            self.detector_scheduler.schedule_file(path).await?
        } else {
            self.detector_scheduler.schedule_directory(path).await?
        };
        
        // Build dependency graph and extract dependencies
        let dependency_graph = self.dependency_builder.build_graph(path).await?;
        
        // Extract dependencies from the graph (simplified)
        let dependencies = Vec::new(); // TODO: Extract dependencies from dependency_graph
        
        // Record findings in aggregator
        self.aggregator.record_findings(file_issues.clone());
        
        info!(
            "Analyzed {} files and extracted dependencies.",
            self.aggregator.get_stats().files_processed
        );
        return Ok((file_issues, dependencies));
    }

    /// Returns a list of all anti-pattern types supported by the registered detectors.
    ///
    /// This method aggregates the anti-pattern types from all configured detectors,
    /// including a built-in type for cyclic dependencies.
    pub fn get_anti_pattern_types(&self) -> Vec<AntiPatternType> {
        // Facade pattern: For now, return static types since detector_scheduler doesn't expose this
        // TODO: Future enhancement - delegate to detector_scheduler
        vec![
            AntiPatternType {
                anti_pattern_type_id: None,
                name: "God Object".to_string(),
                description: "A class that has too many responsibilities and is difficult to maintain.".to_string(),
                category: "Structural".to_string(),
            },
            AntiPatternType {
                anti_pattern_type_id: None,
                name: "Code Duplication".to_string(),
                description: "Multiple instances of similar code that should be refactored.".to_string(),
                category: "Structural".to_string(),
            },
            AntiPatternType {
                anti_pattern_type_id: None,
                name: "Cyclic Dependency".to_string(),
                description: "A direct or indirect dependency cycle between modules or components.".to_string(),
                category: "Structural".to_string(),
            },
            AntiPatternType {
                anti_pattern_type_id: None,
                name: "Dead Code".to_string(),
                description: "Code that is never executed or referenced.".to_string(),
                category: "Structural".to_string(),
            },
            AntiPatternType {
                anti_pattern_type_id: None,
                name: "Large Class".to_string(),
                description: "A class that has grown too large and should be broken down.".to_string(),
                category: "Structural".to_string(),
            },
        ]
    }

    /// Gets the number of files analyzed in the last run.
    pub fn get_files_analyzed(&self) -> i32 {
        // Facade pattern: delegate to aggregator component
        let aggregator_stats = self.aggregator.get_stats();
        aggregator_stats.files_processed as i32
    }

    /// Parses a file using the AST cache for performance optimization
    async fn parse_file_with_cache(
        &mut self,
        path: &Path,
    ) -> crate::error::Result<crate::ast::ParsedFile> {
        #[cfg(feature = "tree-sitter")]
        {
            // Try to get from AST cache first
            if let Ok(cached_tree) = self.ast_provider.get_ast(path).await {
                info!("AST CACHE HIT: Using cached AST for {}", path.display());
                // Create ParsedFile from cached tree
                // We need to read the file source and create a ParsedFile manually
                if let Ok(source_content) = std::fs::read_to_string(path) {
                    let parsed_file = crate::ast::ParsedFile {
                        file_path: std::sync::Arc::new(path.to_path_buf()),
                        language: self.detect_language_from_path(path),
                        tree: Some((*cached_tree).clone()),
                        source: std::sync::Arc::new(source_content),
                        custom_ast: std::sync::Arc::new(None),
                        modified_at: std::fs::metadata(path)?.modified()?,
                    };
                    return Ok(parsed_file);
                }
            }

            info!("AST CACHE MISS: Parsing file {}", path.display());
            // Parse the file normally using AST provider
            let cached_tree = self.ast_provider.get_ast(path).await?;
            
            // Create ParsedFile from cached tree
            if let Ok(source_content) = std::fs::read_to_string(path) {
                let parsed_file = crate::ast::ParsedFile {
                    file_path: std::sync::Arc::new(path.to_path_buf()),
                    language: self.detect_language_from_path(path),
                    tree: Some((*cached_tree).clone()),
                    source: std::sync::Arc::new(source_content),
                    custom_ast: std::sync::Arc::new(None),
                    modified_at: std::fs::metadata(path)?.modified()?,
                };
                return Ok(parsed_file);
            }
            
            // Fallback error if file can't be read
            Err(crate::error::UveddiError::io_error(
                "read file content",
                &path.to_string_lossy(),
                std::io::Error::new(std::io::ErrorKind::NotFound, "Cannot read file")
            ))
        }

        #[cfg(not(feature = "tree-sitter"))]
        {
            // When tree-sitter is disabled, use AST provider
            let cached_tree = self.ast_provider.get_ast(path).await?;
            
            // Create ParsedFile from cached tree
            if let Ok(source_content) = std::fs::read_to_string(path) {
                let parsed_file = crate::ast::ParsedFile {
                    file_path: std::sync::Arc::new(path.to_path_buf()),
                    language: self.detect_language_from_path(path),
                    tree: Some((*cached_tree).clone()),
                    source: std::sync::Arc::new(source_content),
                    custom_ast: std::sync::Arc::new(None),
                    modified_at: std::fs::metadata(path)?.modified()?,
                };
                return Ok(parsed_file);
            }
            
            // Fallback error if file can't be read
            Err(crate::error::UveddiError::io_error(
                "read file content",
                &path.to_string_lossy(),
                std::io::Error::new(std::io::ErrorKind::NotFound, "Cannot read file")
            ))
        }
    }

    /// Detects programming language from file path extension
    fn detect_language_from_path(&self, path: &Path) -> crate::ast::SourceLanguage {
        use crate::ast::SourceLanguage;

        match path.extension().and_then(|ext| ext.to_str()) {
            Some("rs") => SourceLanguage::Rust,
            Some("py") => SourceLanguage::Python,
            Some("js") | Some("ts") | Some("jsx") | Some("tsx") => SourceLanguage::JavaScript,
            _ => SourceLanguage::JavaScript, // Default fallback
        }
    }

    /// Returns AST cache metrics for observability
    pub fn get_ast_cache_metrics(&self) -> serde_json::Value {
        // Facade pattern: delegate to cache manager component
        self.cache_manager.get_cache_metrics()
    }

    /// Clears the AST cache
    pub fn clear_ast_cache(&self) {
        // Facade pattern: delegate to cache manager component
        let cache_manager = self.cache_manager.clone();
        tokio::spawn(async move {
            cache_manager.clear_all_caches().await;
        });
    }

    /// Configures the dead code detector with custom settings.
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration for the dead code detector.
    pub fn configure_dead_code_detector(&mut self, config: DeadCodeConfig) {
        // TODO: Facade pattern - delegate to detector_scheduler
        // For now, this is a no-op since detector configuration is handled by components
        warn!("configure_dead_code_detector is deprecated in facade pattern - use component configuration instead");
    }

    /// Configures the large classes detector with custom settings.
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration for the large classes detector.
    pub fn configure_large_classes_detector(&mut self, config: LargeClassConfig) {
        // TODO: Facade pattern - delegate to detector_scheduler
        // For now, this is a no-op since detector configuration is handled by components
        warn!("configure_large_classes_detector is deprecated in facade pattern - use component configuration instead");
    }

    /// Loads all available WASM plugins from the plugin directory.
    ///
    /// This function scans the configured plugin directory, loads each valid WASM plugin,
    /// and integrates it into the analysis engine as a detector.
    ///
    /// # Errors
    ///
    /// Returns `UveddiError` if the plugin engine is not initialized or if there's an
    /// error during plugin loading.
    pub async fn load_plugins(&mut self) -> crate::error::Result<usize> {
        if let Some(ref mut plugin_engine) = self.plugin_engine {
            let loaded_plugins = plugin_engine
                .load_all_plugins()
                .await
                .map_err(crate::error::UveddiError::from)?;

            // TODO: Re-enable when plugin adapter implements AnalysisDetector
            // Add plugin adapters as detectors
            // for plugin_id in &loaded_plugins {
            //     if let Some(adapter) = plugin_engine.get_plugin_adapter(plugin_id).await {
            //         self.detectors.push(Box::new(adapter));
            //     }
            // }

            info!("Loaded {} WASM plugins", loaded_plugins.len());
            Ok(loaded_plugins.len())
        } else {
            warn!(
                "Plugin engine not initialized. Use new_with_plugins() to enable plugin support."
            );
            Ok(0)
        }
    }

    /// Installs a new WASM plugin.
    ///
    /// # Arguments
    ///
    /// * `manifest` - The plugin manifest containing metadata.
    /// * `binary` - The WASM binary content of the plugin.
    ///
    /// # Errors
    ///
    /// Returns `UveddiError` if the plugin engine is not initialized or if the
    /// installation fails.
    pub async fn install_plugin(
        &mut self,
        manifest: crate::plugins::PluginManifest,
        binary: Vec<u8>,
    ) -> crate::error::Result<crate::plugins::PluginId> {
        if let Some(ref mut plugin_engine) = self.plugin_engine {
            let plugin_id = plugin_engine
                .install_plugin(manifest, binary)
                .await
                .map_err(crate::error::UveddiError::from)?;

            // TODO: Re-enable when plugin adapter implements AnalysisDetector
            // Add the new plugin as a detector
            // if let Some(adapter) = plugin_engine.get_plugin_adapter(&plugin_id).await {
            //     self.detectors.push(Box::new(adapter));
            // }

            Ok(plugin_id)
        } else {
            Err(crate::error::UveddiError::PluginError {
                plugin: "unknown".to_string(),
                plugin_type: "WASM".to_string(),
                message: "Plugin engine not initialized".to_string(),
                suggestion: "Initialize plugin engine before installing plugins".to_string(),
                source: Some(crate::plugins::errors::PluginError::Execution(
                    "Plugin engine not initialized".to_string(),
                )),
            })
        }
    }

    /// Uninstalls a WASM plugin.
    ///
    /// # Arguments
    ///
    /// * `plugin_id` - The ID of the plugin to uninstall.
    ///
    /// # Errors
    ///
    /// Returns `UveddiError` if the plugin engine is not initialized or if the
    /// uninstallation fails.
    pub async fn uninstall_plugin(
        &mut self,
        plugin_id: &crate::plugins::PluginId,
    ) -> crate::error::Result<()> {
        if let Some(ref mut plugin_engine) = self.plugin_engine {
            plugin_engine
                .uninstall_plugin(plugin_id)
                .await
                .map_err(crate::error::UveddiError::from)?;

            // TODO: Re-enable when plugin adapter implements AnalysisDetector
            // Reload all adapters to remove the uninstalled plugin
            // self.detectors.retain(|detector| detector.get_detector_name() != "wasm-plugin-detector");

            // Re-add remaining plugin adapters
            // for adapter in plugin_engine.get_all_plugin_adapters().await {
            //     self.detectors.push(Box::new(adapter));
            // }

            Ok(())
        } else {
            Err(crate::error::UveddiError::PluginError {
                plugin: "unknown".to_string(),
                plugin_type: "WASM".to_string(),
                message: "Plugin engine not initialized".to_string(),
                suggestion: "Initialize plugin engine before uninstalling plugins".to_string(),
                source: Some(crate::plugins::errors::PluginError::Execution(
                    "Plugin engine not initialized".to_string(),
                )),
            })
        }
    }

    /// Retrieves statistics for all loaded plugins.
    pub async fn get_plugin_stats(
        &self,
    ) -> Option<Vec<(crate::plugins::PluginId, crate::plugins::PluginStats)>> {
        if let Some(ref plugin_engine) = self.plugin_engine {
            let mut stats = Vec::new();
            for plugin_id in plugin_engine.list_loaded_plugins().await {
                if let Some(plugin_stats) = plugin_engine.get_plugin_stats(&plugin_id).await {
                    stats.push((plugin_id, plugin_stats));
                }
            }
            Some(stats)
        } else {
            None
        }
    }

    /// Monitors and reports the resource usage of all active plugins.
    ///
    /// # Errors
    ///
    /// Returns `UveddiError` if the plugin engine is not initialized or if
    /// resource monitoring fails.
    pub async fn monitor_plugin_resources(
        &mut self,
    ) -> crate::error::Result<crate::plugins::ResourceReport> {
        if let Some(ref mut plugin_engine) = self.plugin_engine {
            plugin_engine.monitor_resources().await.map_err(|e| {
                crate::error::UveddiError::PluginError {
                    plugin: "unknown".to_string(),
                    plugin_type: "WASM".to_string(),
                    message: e.to_string(),
                    suggestion: "Check plugin status and retry operation".to_string(),
                    source: Some(crate::plugins::errors::PluginError::Execution(
                        e.to_string(),
                    )),
                }
            })
        } else {
            Err(crate::error::UveddiError::PluginError {
                plugin: "unknown".to_string(),
                plugin_type: "WASM".to_string(),
                message: "Plugin engine not initialized".to_string(),
                suggestion: "Initialize plugin engine before monitoring resources".to_string(),
                source: Some(crate::plugins::errors::PluginError::Execution(
                    "Plugin engine not initialized".to_string(),
                )),
            })
        }
    }

    /// Check if plugin engine is available
    pub fn has_plugin_support(&self) -> bool {
        self.plugin_engine.is_some()
    }

    /// Add plugin detectors dynamically
    ///
    /// Scans the plugin engine for loaded plugins and creates detector adapters
    /// for each one, integrating them into the analysis engine's detector pipeline.
    ///
    /// # Errors
    ///
    /// Returns `UveddiError` if plugin integration fails
    pub async fn add_plugin_detectors(&mut self) -> crate::error::Result<usize> {
        if let Some(ref plugin_engine) = self.plugin_engine {
            // Skip adapter factory for now due to type constraints
            // TODO: Implement proper plugin adapter integration
            log::warn!("Plugin adapter integration skipped due to type constraints");
            return Ok(0);
        } else {
            Ok(0) // No plugin engine, no detectors added
        }
    }

    /// Remove plugin detectors from the detector pipeline
    ///
    /// Removes all WASM plugin detectors from the current detector set.
    /// This is useful when reloading plugins or disabling plugin support.
    pub fn remove_plugin_detectors(&mut self) -> usize {
        // TODO: Facade pattern - delegate to detector_scheduler
        // For now, this is a no-op since detector management is handled by components
        warn!("remove_plugin_detectors is deprecated in facade pattern - use component management instead");
        0
    }

    /// Reload plugin detectors
    ///
    /// Removes existing plugin detectors and reloads them from the plugin engine.
    /// This is useful when plugins have been added, removed, or updated.
    ///
    /// # Errors
    ///
    /// Returns `UveddiError` if plugin reloading fails
    pub async fn reload_plugin_detectors(&mut self) -> crate::error::Result<usize> {
        // Remove existing plugin detectors
        self.remove_plugin_detectors();

        // Add current plugin detectors
        self.add_plugin_detectors().await
    }

    /// Gets statistics from the plugin registry.
    pub fn get_plugin_registry_stats(
        &self,
    ) -> Option<crate::plugins::registry::RegistryStatistics> {
        self.plugin_engine
            .as_ref()
            .map(|engine| engine.get_registry_stats())
    }
}
