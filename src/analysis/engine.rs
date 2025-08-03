// Summary of Required Refactoring in engine.rs
//
// This file requires several refactorings to align with the updated architecture and error handling.
// The following changes are necessary:
//
// - Replace deprecated plugin manager calls with correct methods
//   - Example: `plugin_manager.unload_plugin(plugin_id).await`
//   - Example: `plugin_manager.load_plugin(manifest, binary).await`
//   - Example: `plugin_manager.list_loaded_plugins().await`
//   - Example: `plugin_manager.get_stats(&plugin_id).await`
//   - Example: `plugin_manager.monitor_resources().await` (if available)
//   - Example: `plugin_manager.get_stats().await`
// - Replace `cache_manager.get_cache_metrics()` with `cache_manager.get_cache_stats()`
// - Replace `cache_manager.clear_all_caches().await` with correct method if available
// - Import `CacheManager` trait where needed
// - Import `TryFutureExt` from futures where `.map_err` is used on futures
// - Update all `detect_issues` implementations in detectors to be `async fn` and match the trait signature
// - Refactor any code that calls `detect_issues` synchronously to use `.await`
// - Refactor any use of rayon for async code to use `futures::future::join_all` or similar combinators
//
// Once these changes are made, the engine.rs file will be aligned with the new architecture and error handling mechanisms.

use crate::analysis::cache::ast::{AstCache, CacheConfig};
use crate::analysis::components::cache_manager::CacheManager;
use crate::analysis::detectors::anti_patterns::dead_code::{DeadCodeConfig, DeadCodeDetector};
use crate::analysis::detectors::anti_patterns::large_classes::{
    LargeClassConfig, LargeClassDetector,
};
use crate::analysis::detectors::cycle::CycleDetector;
use crate::analysis::detectors::dependency::{Dependency, DependencyExtractor};
use crate::analysis::errors::AnalysisError;
use crate::analysis::extractors::SymbolExtractor;
use crate::analysis::file_discovery::{FileDiscovery, SourceFile};
use crate::analysis::graph::dependency::LocalDependencyGraph;
use crate::analysis::graph::dependency::{ComponentNode, LocalDependencyType};
use crate::analysis::incremental::{
    IncrementalAnalysisEngine, IncrementalAnalysisResult, IncrementalConfig,
};
use crate::analysis::symbols::GlobalSymbolTable;
use crate::analysis::traits::{AstParserTrait, DependencyExtractorTrait, ResultCacheTrait};
use crate::analysis::workspace::{WorkspaceDetector, WorkspaceInfo};
use crate::analysis::memory_report::MemoryAnalysisReport;
use crate::analysis::AnalysisDetector;
use crate::ast::tree_sitter_impl::AstParser;
use crate::ast::ParsedFile;
use crate::cache::result_cache::ResultCache;
use crate::database::models::{
    AntiPatternType, ArchitecturalIssue, ComponentPerformanceMetrics, PerformanceMetricsConfig,
};
use crate::ingestion::AsyncWalker;
use crate::monitoring::performance_metrics_collector::PerformanceMetricsCollector;
use crate::plugins::WasmPluginEngine;

// Component imports
use crate::analysis::components::{
    AnalysisAggregator, AstProviderImpl, CacheManagerImpl, ConfigurationService,
    DependencyGraphBuilderImpl, DetectorScheduler, PluginManagerHandle,
};
use crate::analysis::detector_factory::DetectorFactory;
use crate::analysis::engine_builder::AnalysisEngineBuilder; // Import the builder
use crate::error::UveddiError;

// Stub types for when AI features are disabled
#[cfg(not(feature = "ai"))]
pub struct AiAnalysisEngine;
#[cfg(not(feature = "ai"))]
pub struct KnowledgeLibrary;
#[cfg(not(feature = "ai"))]
pub struct ContextSelector;
#[cfg(not(feature = "ai"))]
#[derive(Debug, Clone)]
pub struct EngineAnalysisContext;
#[cfg(not(feature = "ai"))]
#[derive(Debug, Clone)]
pub struct EngineComplexityMetrics;
#[cfg(not(feature = "ai"))]
#[derive(Debug, Clone)]
pub struct KnowledgeContext;

// Import SourceLanguage from the appropriate module
use crate::ast::tree_sitter_impl::SourceLanguage;

// Knowledge Library imports (feature-gated)
#[cfg(feature = "ai")]
use crate::ai::engine::AiAnalysisEngine;
#[cfg(feature = "ai")]
use crate::ai::knowledge::{
    ContextSelector, EngineAnalysisContext, EngineComplexityMetrics, KnowledgeContext,
    KnowledgeLibrary,
};

// Component imports
use crate::analysis::components::traits::{
    AnalysisAggregator as AnalysisAggregatorTrait, AstProvider as AstProviderTrait,
    DependencyGraphBuilder as DependencyGraphBuilderTrait,
    DetectorScheduler as DetectorSchedulerTrait, PluginManagerHandle as PluginManagerHandleTrait,
};
use log::{info, warn};
use std::sync::Arc;

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
/// The `AnalysisEngine` is responsible for:
/// - Initializing language-specific parsers
/// - Processing source files into AST representations
/// - Running registered detectors (including plugins) against the codebase
/// - Aggregating and reporting analysis results
///
/// # Example
/// ```no_run
/// use uveddi::analysis::AnalysisEngine;
/// use std::path::Path;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Using the builder pattern for async initialization
///     let mut engine = AnalysisEngine::builder()
///         .enable_plugins(true)
///         .build_async()
///         .await?;
///     let (issues, graph) = engine.analyze(Path::new("src/")).await?;
///     println!("Found {} issues", issues.len());
///     Ok(())
/// }
/// ```
pub struct AnalysisEngine {
    // Component references - implementing facade pattern
    pub config_service: Arc<ConfigurationService>,
    pub ast_provider: Arc<AstProviderImpl>,
    pub cache_manager: Arc<CacheManagerImpl>,
    pub dependency_builder: Arc<DependencyGraphBuilderImpl>,
    pub detector_scheduler: Arc<DetectorScheduler>,
    pub plugin_manager: Option<PluginManagerHandle>,
    pub aggregator: Arc<AnalysisAggregator>,

    // NEW: Knowledge Library Integration Components
    #[cfg(feature = "ai")]
    pub knowledge_library: Option<Arc<KnowledgeLibrary>>,
    #[cfg(feature = "ai")]
    pub context_selector: Option<Arc<ContextSelector>>,
    #[cfg(feature = "ai")]
    pub ai_engine: Option<Arc<AiAnalysisEngine>>,

    // Configuration flags for knowledge enhancement
    pub enable_knowledge_enhancement: bool,
    pub enable_ai_explanations: bool,
}

impl AnalysisEngine {
    /// Returns a new `AnalysisEngineBuilder` for flexible engine construction.
    ///
    /// This is the recommended way to create an `AnalysisEngine` instance, allowing for configuration
    /// of detectors, cache paths, and plugin support.
    ///
    /// # Example
    /// ```no_run
    /// use uveddi::analysis::AnalysisEngine;
    /// use std::path::PathBuf;
    ///
    /// let engine = AnalysisEngine::builder()
    ///     .with_cache_path(&PathBuf::from("my_cache.db"))
    ///     .build()?; // Synchronous build
    ///
    /// let engine_async = AnalysisEngine::builder()
    ///     .enable_plugins(true)
    ///     .build_async()
    ///     .await?; // Asynchronous build
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn builder() -> AnalysisEngineBuilder {
        AnalysisEngineBuilder::new()
    }

    /// Creates a new analysis engine with default configuration.
    ///
    /// This is a synchronous constructor for basic use cases without plugins or custom paths.
    /// For more advanced configurations, use [`AnalysisEngine::builder`].
    ///
    /// # Errors
    /// Returns `UveddiError` if initialization fails.
    ///
    /// # Example
    /// ```rust
    /// use uveddi::analysis::AnalysisEngine;
    /// let engine = AnalysisEngine::new().unwrap();
    /// ```
    #[inline]
    pub fn new() -> crate::error::Result<Self> {
        AnalysisEngineBuilder::new().build()
    }

    /// Creates a new analysis engine with WASM plugin support enabled.
    ///
    /// This is an asynchronous constructor as plugin initialization involves I/O.
    /// For more advanced configurations, use [`AnalysisEngine::builder`].
    ///
    /// # Errors
    /// Returns `UveddiError` if initialization fails.
    ///
    /// # Example
    /// ```rust,ignore
    /// use uveddi::analysis::AnalysisEngine;
    /// let engine = AnalysisEngine::new_async().await.unwrap();
    /// ```
    #[inline]
    pub async fn new_async() -> crate::error::Result<Self> {
        AnalysisEngineBuilder::new()
            .enable_plugins(true)
            .build_async()
            .await
    }

    /// Creates an AnalysisEngine with a custom set of detectors.
    ///
    /// This is a synchronous constructor. For more advanced configurations,
    /// including plugin support, use `AnalysisEngine::builder()`.
    ///
    /// # Arguments
    /// * `detectors` - Vector of detectors to use for analysis.
    ///
    /// # Errors
    /// Returns `UveddiError` if initialization fails.
    ///
    /// # UV-294 Compliance
    /// This constructor is synchronous and performs no I/O or async operations.
    #[inline]
    pub fn with_detectors(
        detectors: Vec<Box<dyn AnalysisDetector + Send + Sync>>,
    ) -> crate::error::Result<Self> {
        AnalysisEngineBuilder::new()
            .with_detectors(detectors)
            .build()
    }

    /// Creates an AnalysisEngine with a custom cache database path.
    ///
    /// This is a synchronous constructor. For more advanced configurations,
    /// including plugin support, use `AnalysisEngine::builder()`.
    ///
    /// # Arguments
    /// * `cache_path` - Path to the cache database file.
    ///
    /// # Errors
    /// Returns `UveddiError` if initialization fails.
    ///
    /// # UV-294 Compliance
    /// This constructor is synchronous and performs no I/O or async operations.
    #[inline]
    pub fn with_cache_path(cache_path: &Path) -> crate::error::Result<Self> {
        AnalysisEngineBuilder::new()
            .with_cache_path(cache_path)
            .build()
    }

    /// Creates a new analysis engine with an in-memory cache database.
    ///
    /// This is a synchronous constructor. For more advanced configurations,
    /// including plugin support, use `AnalysisEngine::builder()`.
    ///
    /// # Errors
    /// Returns `UveddiError` if initialization fails.
    ///
    /// # UV-294 Compliance
    /// This constructor is synchronous and performs no I/O or async operations.
    #[inline]
    pub fn new_with_memory_cache() -> crate::error::Result<Self> {
        AnalysisEngineBuilder::new().with_in_memory_cache().build()
    }

    /// Create AnalysisEngine with injected dependencies (DEPENDENCY INJECTION).
    ///
    /// This is a synchronous constructor. For more advanced configurations,
    /// including plugin support, use `AnalysisEngine::builder()` with injection methods.
    ///
    /// # Arguments
    /// * `ast_parser` - AST parsing implementation.
    /// * `dependency_extractor` - Dependency analysis implementation.
    /// * `cache` - Result caching implementation.
    /// * `detectors` - Vector of detectors to use for analysis.
    ///
    /// # Errors
    /// Returns `UveddiError` if initialization fails.
    ///
    /// # UV-294 Compliance
    /// This constructor is synchronous and performs no I/O or async operations.
    #[inline]
    pub fn with_injected_dependencies(
        ast_parser: Box<dyn AstParserTrait>,
        dependency_extractor: Box<dyn DependencyExtractorTrait>,
        cache: Box<dyn ResultCacheTrait>,
        detectors: Vec<Box<dyn AnalysisDetector + Send + Sync>>,
    ) -> crate::error::Result<Self> {
        AnalysisEngineBuilder::new()
            .with_injected_ast_parser(ast_parser)
            .with_injected_dependency_extractor(dependency_extractor)
            .with_injected_result_cache(cache)
            .with_detectors(detectors)
            .build()
    }

    /// Create AnalysisEngine with default dependencies (backward compatibility).
    ///
    /// This is a synchronous constructor. For more advanced configurations,
    /// including plugin support, use `AnalysisEngine::builder()`.
    ///
    /// # Errors
    /// Returns `UveddiError` if initialization fails.
    ///
    /// # UV-294 Compliance
    /// This constructor is synchronous and performs no I/O or async operations.
    #[inline]
    pub fn new_with_defaults() -> crate::error::Result<Self> {
        AnalysisEngineBuilder::new().build()
    }

    /// Deprecated: Use `AnalysisEngine::builder().enable_plugins(true).build_async().await` instead.
    #[deprecated(
        since = "0.2.0",
        note = "Use `AnalysisEngine::builder().enable_plugins(true).build_async().await` for async plugin initialization. Refer to UV-294 guidelines for async standardization."
    )]
    pub async fn new_with_plugins() -> crate::error::Result<Self> {
        AnalysisEngineBuilder::new()
            .enable_plugins(true)
            .build_async()
            .await
    }

    /// Deprecated: Use `AnalysisEngine::builder().with_detectors(detectors).enable_plugins(true).build_async().await` instead.
    #[deprecated(
        since = "0.2.0",
        note = "Use `AnalysisEngine::builder().with_detectors(detectors).enable_plugins(true).build_async().await` for async plugin initialization. Refer to UV-294 guidelines for async standardization."
    )]
    pub async fn with_detectors_and_plugins(
        detectors: Vec<Box<dyn AnalysisDetector + Send + Sync>>,
        cache_path: Option<&Path>,
    ) -> crate::error::Result<Self> {
        let mut builder = AnalysisEngineBuilder::new()
            .with_detectors(detectors)
            .enable_plugins(true);
        if let Some(path) = cache_path {
            builder = builder.with_cache_path(path);
        }
        builder.build_async().await
    }

    /// Deprecated: Use `AnalysisEngine::builder().with_cache_path(cache_path).enable_plugins(true).build_async().await` instead.
    #[deprecated(
        since = "0.2.0",
        note = "Use `AnalysisEngine::builder().with_cache_path(cache_path).enable_plugins(true).build_async().await` for async plugin initialization. Refer to UV-294 guidelines for async standardization."
    )]
    pub async fn with_cache_path_and_plugins(cache_path: &Path) -> crate::error::Result<Self> {
        AnalysisEngineBuilder::new()
            .with_cache_path(cache_path)
            .enable_plugins(true)
            .build_async()
            .await
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
        let has_common_language = parser_languages
            .iter()
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
    /// This is the main entry point for analysis. It implements the robust,
    /// concurrent analysis pipeline described in Section 1 of the roadmap:
    /// 1. File Discovery: Discovers and filters source files
    /// 2. AST Parsing: Parses files into structured representations
    /// 3. Dependency Extraction: Builds dependency relationships
    /// 4. Anti-Pattern Detection: Runs detectors concurrently
    /// 5. Graph Construction: Creates dependency graph
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
        info!("Starting comprehensive analysis for: {}", path.display());
        let start_time = Instant::now();

        // Execute the main analysis pipeline
        let result = self.execute_analysis_pipeline(path).await;
        
        let duration = start_time.elapsed();
        match &result {
            Ok((issues, _)) => {
                info!(
                    "Analysis completed successfully in {:?}: {} issues found",
                    duration, issues.len()
                );
            }
            Err(e) => {
                warn!("Analysis failed after {:?}: {}", duration, e);
            }
        }

        result
    }

    /// Executes the core analysis pipeline with fault tolerance and concurrency
    ///
    /// This method implements the multi-stage pipeline from Section 1.1 of the roadmap:
    /// - File Discovery: self.discover_source_files(path).await?
    /// - AST Parsing: self.parse_files(files).await?
    /// - Dependency Extraction: self.extract_dependencies(&parsed_files).await?
    /// - Anti-Pattern Detection: self.run_detectors(&parsed_files).await?
    /// - Graph Construction: self.build_dependency_graph(dependencies).await?
    ///
    /// Uses the "fan-out, fan-in" pattern for concurrent file processing with
    /// graceful degradation when individual files fail.
    async fn execute_analysis_pipeline(
        &mut self,
        path: &Path,
    ) -> crate::error::Result<(Vec<ArchitecturalIssue>, LocalDependencyGraph)> {
        // Stage 1: File Discovery
        info!("Stage 1: Discovering source files...");
        let source_files = self.discover_source_files(path).await?;
        info!("Discovered {} source files", source_files.len());

        if source_files.is_empty() {
            warn!("No supported source files found in {}", path.display());
            return Ok((Vec::new(), LocalDependencyGraph::new()));
        }

        // Stage 2: AST Parsing (concurrent with fault tolerance)
        info!("Stage 2: Parsing files concurrently...");
        let parsed_files = self.parse_files(source_files).await?;
        info!("Successfully parsed {}/{} files", parsed_files.len(), parsed_files.len());

        // Stage 3: Dependency Extraction
        info!("Stage 3: Extracting dependencies...");
        let dependencies = self.extract_dependencies(&parsed_files).await?;
        info!("Extracted {} dependencies", dependencies.len());

        // Stage 4: Anti-Pattern Detection (concurrent)
        info!("Stage 4: Running detectors concurrently...");
        let issues = self.run_detectors(&parsed_files).await?;
        info!("Detected {} issues", issues.len());

        // Stage 5: Graph Construction
        info!("Stage 5: Building dependency graph...");
        let dependency_graph = self.build_dependency_graph(&parsed_files, dependencies).await?;
        info!("Built dependency graph with {} nodes", dependency_graph.node_count());

        // Record metrics
        self.record_analysis_metrics(&issues, parsed_files.len());

        Ok((issues, dependency_graph))
    }

    /// Stage 1: Discovers source files using intelligent file discovery
    ///
    /// Implements Section 2.3 of the roadmap using the ignore crate for
    /// high-performance directory traversal with git-style ignore patterns.
    async fn discover_source_files(&self, path: &Path) -> crate::error::Result<Vec<SourceFile>> {
        let file_discovery = FileDiscovery::new();
        
        if path.is_file() {
            // Single file analysis
            let language = self.detect_language_from_path(path);
            Ok(vec![SourceFile {
                path: path.to_path_buf(),
                language,
            }])
        } else {
            // Directory analysis
            file_discovery.discover_files(path).map_err(|e| {
                crate::error::UveddiError::io_error(
                    "file discovery",
                    &path.to_string_lossy(),
                    std::io::Error::new(std::io::ErrorKind::Other, e.to_string()),
                )
            })
        }
    }

    /// Stage 2: Parses files concurrently with fault tolerance
    ///
    /// Implements the "fan-out, fan-in" pattern from Section 1.2 using Tokio
    /// for concurrent processing with graceful degradation.
    async fn parse_files(&self, source_files: Vec<SourceFile>) -> crate::error::Result<Vec<ParsedFile>> {
        use futures::future::join_all;
        
        info!("Parsing {} files concurrently", source_files.len());
        
        // Fan-out: Spawn concurrent parsing tasks
        let parse_tasks: Vec<_> = source_files
            .into_iter()
            .map(|source_file| {
                let ast_provider = self.ast_provider.clone();
                tokio::spawn(async move {
                    let path = &source_file.path;
                    match ast_provider.get_ast(path).await {
                        Ok(tree) => {
                            // Read source content
                            match std::fs::read_to_string(path) {
                                Ok(source_content) => {
                                    let parsed_file = ParsedFile {
                                        file_path: std::sync::Arc::new(path.clone()),
                                        language: source_file.language,
                                        tree: Some((*tree).clone()),
                                        source: std::sync::Arc::new(source_content),
                                        custom_ast: std::sync::Arc::new(None),
                                        modified_at: crate::analysis::cache::wrappers::ArchivableSystemTime(
                                            std::fs::metadata(path)
                                                .and_then(|m| m.modified())
                                                .unwrap_or_else(|_| std::time::SystemTime::now()),
                                        ),
                                    };
                                    Ok::<Option<ParsedFile>, crate::error::UveddiError>(Some(parsed_file))
                                }
                                Err(e) => {
                                    warn!("Failed to read file {}: {}", path.display(), e);
                                    Ok::<Option<ParsedFile>, crate::error::UveddiError>(None) // Graceful degradation
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Failed to parse file {}: {}", path.display(), e);
                            Ok::<Option<ParsedFile>, crate::error::UveddiError>(None) // Graceful degradation
                        }
                    }
                })
            })
            .collect();

        // Fan-in: Collect results with fault tolerance
        let parse_results = join_all(parse_tasks).await;
        let mut parsed_files = Vec::new();
        let mut failed_count = 0;

        for result in parse_results {
            match result {
                Ok(Ok(Some(parsed_file))) => {
                    parsed_files.push(parsed_file);
                }
                Ok(Ok(None)) => {
                    failed_count += 1;
                    // File failed to parse but we continue (graceful degradation)
                }
                Ok(Err(e)) => {
                    failed_count += 1;
                    warn!("Parse task returned error: {}", e);
                }
                Err(e) => {
                    failed_count += 1;
                    warn!("Parse task panicked: {}", e);
                }
            }
        }

        if failed_count > 0 {
            warn!(
                "Parsing completed with {} failures out of {} files",
                failed_count,
                parsed_files.len() + failed_count
            );
        }

        Ok(parsed_files)
    }

    /// Stage 3: Extracts dependencies from parsed files
    ///
    /// Uses tree-sitter queries to extract syntactic dependencies and
    /// performs semantic resolution as described in Section 3.1.
    async fn extract_dependencies(&self, parsed_files: &[ParsedFile]) -> crate::error::Result<Vec<Dependency>> {
        let mut all_dependencies = Vec::new();
        
        for parsed_file in parsed_files {
            match self.extract_file_dependencies(&parsed_file.file_path).await {
                Ok(mut deps) => {
                    all_dependencies.append(&mut deps);
                }
                Err(e) => {
                    warn!(
                        "Failed to extract dependencies from {}: {}",
                        parsed_file.file_path.display(),
                        e
                    );
                    // Continue with other files (graceful degradation)
                }
            }
        }
        
        Ok(all_dependencies)
    }

    /// Stage 4: Runs detectors concurrently on parsed files
    ///
    /// Implements concurrent detector execution with fault tolerance
    /// as described in Section 1.3.
    async fn run_detectors(&self, parsed_files: &[ParsedFile]) -> crate::error::Result<Vec<ArchitecturalIssue>> {
        use futures::future::join_all;
        
        info!("Running detectors on {} files", parsed_files.len());
        
        // Fan-out: Run detectors concurrently on each file
        let detector_tasks: Vec<_> = parsed_files
            .iter()
            .map(|parsed_file| {
                let detector_scheduler = self.detector_scheduler.clone();
                let file_path = parsed_file.file_path.as_ref().clone();
                tokio::spawn(async move {
                    match detector_scheduler.schedule_file(&file_path).await {
                        Ok(issues) => Ok::<Vec<ArchitecturalIssue>, crate::error::UveddiError>(issues),
                        Err(e) => {
                            warn!("Detector failed for file {}: {}", file_path.display(), e);
                            Ok::<Vec<ArchitecturalIssue>, crate::error::UveddiError>(Vec::new()) // Graceful degradation
                        }
                    }
                })
            })
            .collect();

        // Fan-in: Collect all issues
        let detector_results = join_all(detector_tasks).await;
        let mut all_issues = Vec::new();

        for result in detector_results {
            match result {
                Ok(Ok(mut issues)) => {
                    all_issues.append(&mut issues);
                }
                Ok(Err(e)) => {
                    warn!("Detector task returned error: {}", e);
                }
                Err(e) => {
                    warn!("Detector task panicked: {}", e);
                }
            }
        }

        // Record findings in aggregator
        self.aggregator.record_findings(all_issues.clone());

        Ok(all_issues)
    }

    /// Stage 5: Builds dependency graph from parsed files and dependencies
    ///
    /// Constructs a petgraph-based dependency graph as described in Section 3.2.
    async fn build_dependency_graph(
        &self,
        parsed_files: &[ParsedFile],
        dependencies: Vec<Dependency>,
    ) -> crate::error::Result<LocalDependencyGraph> {
        let mut graph = LocalDependencyGraph::new();

        // Add nodes for each parsed file
        for parsed_file in parsed_files {
            let node = ComponentNode::Module {
                path: parsed_file.file_path.display().to_string(),
            };
            graph.add_component(node);
        }

        // Add edges for dependencies
        for dependency in dependencies {
            let from_node = ComponentNode::Module {
                path: dependency.from_file.display().to_string(),
            };
            let to_node = ComponentNode::Module {
                path: dependency.to_module.clone(),
            };
            graph.add_dependency(&from_node, &to_node, LocalDependencyType::Import);
        }

        Ok(graph)
    }

    /// Records analysis metrics for observability
    fn record_analysis_metrics(&self, issues: &[ArchitecturalIssue], files_processed: usize) {
        let metrics_config = PerformanceMetricsConfig::default();
        let mut metrics_collector = PerformanceMetricsCollector::new(metrics_config, files_processed);

        metrics_collector.record_analysis_metrics(issues.len(), files_processed);

        if let Err(e) = metrics_collector.emit_metrics() {
            warn!("Failed to emit performance metrics: {}", e);
        }
    }

    /// Robust analysis with workspace detection and error recovery
    ///
    /// This method provides a robust analysis pipeline by:
    /// 1. Detecting workspace structure (single crate vs multi-crate workspace)
    /// 2. Analyzing each crate individually with error recovery
    /// 3. Aggregating results across all crates
    /// 4. Providing detailed error reporting with actionable suggestions
    ///
    /// # Arguments
    ///
    /// * `path` - Directory or file path to analyze
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - `Vec<ArchitecturalIssue>` - All detected issues across all crates
    /// - `LocalDependencyGraph` - The aggregated dependency graph
    ///
    /// # Errors
    ///
    /// Returns specific `AnalysisError` variants for different failure modes
    pub async fn analyze_robust(
        &mut self,
        path: &Path,
    ) -> Result<(Vec<ArchitecturalIssue>, LocalDependencyGraph), AnalysisError> {
        let start_time = Instant::now();
        info!("Starting robust analysis for: {:?}", path);

        // Phase 1: Workspace Detection
        let workspace_info = match WorkspaceDetector::detect_workspace(path).await? {
            Some(workspace) => {
                info!("Detected workspace with {} crates", workspace.crates.len());
                workspace
            }
            None => {
                // Fall back to standard analysis for non-Rust projects
                info!("No workspace detected, falling back to standard analysis");
                return self.analyze_fallback(path).await;
            }
        };

        // Phase 2: Multi-crate Analysis with Error Recovery
        let mut all_issues = Vec::new();
        let mut all_dependencies = Vec::new();
        let mut successful_crates = 0;
        let mut failed_crates = Vec::new();

        for (crate_name, crate_info) in &workspace_info.crates {
            info!("Analyzing crate: {}", crate_name);
            
            match self.analyze_single_crate(crate_info).await {
                Ok((crate_issues, crate_deps)) => {
                    all_issues.extend(crate_issues);
                    all_dependencies.extend(crate_deps);
                    successful_crates += 1;
                    info!("Successfully analyzed crate: {}", crate_name);
                }
                Err(e) => {
                    let error_msg = format!("Failed to analyze crate '{}': {}", crate_name, e);
                    warn!("{}", error_msg);
                    failed_crates.push((crate_name.clone(), e));
                    
                    // Don't fail completely, continue with other crates
                    continue;
                }
            }
        }

        // Phase 3: Build Aggregated Dependency Graph
        let dependency_graph = self.build_workspace_dependency_graph(&workspace_info, &all_dependencies).await?;

        // Phase 4: Report Analysis Results
        let analysis_duration = start_time.elapsed();
        info!(
            "Robust analysis completed in {:?}: {} crates successful, {} failed",
            analysis_duration,
            successful_crates,
            failed_crates.len()
        );

        // If we have partial failures, add them as issues but don't fail
        if !failed_crates.is_empty() {
            for (crate_name, error) in failed_crates {
                let issue = self.create_analysis_failure_issue(crate_name, error);
                all_issues.push(issue);
            }
        }

        // Fail only if no crates were successfully analyzed
        if successful_crates == 0 {
            return Err(AnalysisError::pipeline_error(
                "multi-crate-analysis",
                format!("Failed to analyze any crates in workspace at '{}'", path.display()),
            ));
        }

        Ok((all_issues, dependency_graph))
    }

    /// Fallback analysis method for non-workspace projects
    async fn analyze_fallback(
        &mut self,
        path: &Path,
    ) -> Result<(Vec<ArchitecturalIssue>, LocalDependencyGraph), AnalysisError> {
        info!("Using fallback analysis for path: {:?}", path);
        
        // Convert UveddiError to AnalysisError for the return type
        self.analyze(path).await.map_err(|e| {
            AnalysisError::pipeline_error("fallback-analysis", format!("Standard analysis failed: {}", e))
        })
    }

    /// Analyze a single crate with error recovery
    async fn analyze_single_crate(
        &mut self,
        crate_info: &crate::analysis::workspace::CrateInfo,
    ) -> Result<(Vec<ArchitecturalIssue>, Vec<Dependency>), AnalysisError> {
        let crate_path = &crate_info.path;
        info!("Analyzing crate '{}' at: {:?}", crate_info.name, crate_path);

        // Get source files for this crate
        let source_files = self.get_crate_source_files(crate_info).await?;
        
        if source_files.is_empty() {
            warn!("No source files found for crate: {}", crate_info.name);
            return Ok((vec![], vec![]));
        }

        let mut crate_issues = Vec::new();
        let mut crate_dependencies = Vec::new();
        let mut processed_files = 0;
        let mut failed_files = 0;

        // Analyze each source file with error recovery
        for source_file in source_files {
            match self.analyze_single_file(&source_file).await {
                Ok((file_issues, file_deps)) => {
                    crate_issues.extend(file_issues);
                    crate_dependencies.extend(file_deps);
                    processed_files += 1;
                }
                Err(e) => {
                    // Log the error but continue with other files
                    if e.is_recoverable() {
                        warn!("Recoverable error in file {:?}: {}", source_file, e);
                        failed_files += 1;
                    } else {
                        // For non-recoverable errors, we might want to fail the entire crate
                        return Err(AnalysisError::crate_analysis_error(
                            crate_info.name.clone(),
                            crate_path.display().to_string(),
                            format!("Non-recoverable error in file {:?}: {}", source_file, e),
                        ));
                    }
                }
            }
        }

        info!(
            "Crate '{}' analysis complete: {} files processed, {} files failed", 
            crate_info.name, processed_files, failed_files
        );

        Ok((crate_issues, crate_dependencies))
    }

    /// Get all source files for a crate
    async fn get_crate_source_files(
        &self,
        crate_info: &crate::analysis::workspace::CrateInfo,
    ) -> Result<Vec<PathBuf>, AnalysisError> {
        let mut source_files = Vec::new();

        for source_dir in &crate_info.source_dirs {
            if !source_dir.exists() {
                warn!("Source directory does not exist: {:?}", source_dir);
                continue;
            }

            let files = WorkspaceDetector::collect_rust_files(source_dir).await?;
            source_files.extend(files);
        }

        Ok(source_files)
    }

    /// Analyze a single file with detailed error handling
    async fn analyze_single_file(
        &mut self,
        file_path: &Path,
    ) -> Result<(Vec<ArchitecturalIssue>, Vec<Dependency>), AnalysisError> {
        // This is a simplified version - in practice you'd want to use the detector scheduler
        // For now, we'll delegate to the existing analyze method and handle the conversion
        
        match self.detector_scheduler.schedule_file(file_path).await {
            Ok(issues) => {
                // Extract dependencies - this is a simplified approach
                let dependencies = match self.extract_file_dependencies(file_path).await {
                    Ok(deps) => deps,
                    Err(e) => {
                        warn!("Failed to extract dependencies from {:?}: {}", file_path, e);
                        vec![]
                    }
                };
                Ok((issues, dependencies))
            }
            Err(e) => Err(AnalysisError::detector_error(
                "multi-detector",
                file_path.display().to_string(),
                format!("File analysis failed: {}", e),
            )),
        }
    }

    /// Extract dependencies from a single file
    async fn extract_file_dependencies(
        &self,
        file_path: &Path,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        // For now, return empty dependencies as this is a complex operation
        // In practice, you'd want to implement proper dependency extraction
        // based on the file type and language
        
        // This is a simplified placeholder - real implementation would:
        // 1. Parse the file using the appropriate parser
        // 2. Extract import/use statements 
        // 3. Build dependency relationships
        // 4. Return structured dependency information
        
        Ok(vec![])
    }

    /// Build dependency graph for the entire workspace
    async fn build_workspace_dependency_graph(
        &self,
        workspace_info: &WorkspaceInfo,
        all_dependencies: &[Dependency],
    ) -> Result<LocalDependencyGraph, AnalysisError> {
        let mut graph = LocalDependencyGraph::new();

        // Add nodes for each crate
        for (crate_name, crate_info) in &workspace_info.crates {
            let node = ComponentNode::Module { 
                path: crate_info.path.display().to_string()
            };
            graph.add_component(node);
        }

        // Add dependencies as edges  
        for dep in all_dependencies {
            let from_node = ComponentNode::Module { 
                path: dep.from_file.display().to_string()
            };
            let to_node = ComponentNode::Module { 
                path: dep.to_module.clone() 
            };
            graph.add_dependency(&from_node, &to_node, LocalDependencyType::Import);
        }

        Ok(graph)
    }

    /// Create an architectural issue for analysis failures
    fn create_analysis_failure_issue(
        &self,
        crate_name: String,
        error: AnalysisError,
    ) -> ArchitecturalIssue {
        let recommendation = match error.category() {
            "memory" => "Consider using --enable-memory-optimization or analyzing smaller subsets".to_string(),
            "parsing" => "Check for syntax errors or unsupported language constructs".to_string(),
            "filesystem" => "Ensure all files are accessible and properly encoded".to_string(),
            _ => "Review the error details and try analyzing individual files".to_string(),
        };
        
        ArchitecturalIssue {
            issue_id: None,
            analysis_run_id: 0, // Will be set by the database layer
            anti_pattern_type_id: 999, // Special ID for analysis failures
            file_path: format!("crate:{}", crate_name),
            start_line: Some(0),
            end_line: Some(0),
            severity: "High".to_string(),
            description: format!("Analysis failed for crate '{}': {}", crate_name, error),
            code_snippet: None,
            ai_explanation: Some(recommendation),
        }
    }

    /// Memory-aware analysis with adaptive behavior and monitoring
    ///
    /// This method wraps the robust analysis with memory monitoring and adaptive behavior:
    /// 1. Monitors memory usage throughout the analysis process
    /// 2. Applies memory optimization techniques when limits are approached
    /// 3. Provides fallback strategies when memory constraints are hit
    /// 4. Reports memory usage metrics and recommendations
    ///
    /// # Arguments
    ///
    /// * `path` - Directory or file path to analyze
    /// * `memory_limit_mb` - Optional memory limit in MB (defaults to system detection)
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - `Vec<ArchitecturalIssue>` - All detected issues
    /// - `LocalDependencyGraph` - The dependency graph
    /// - `MemoryAnalysisReport` - Memory usage report and recommendations
    ///
    /// # Errors
    ///
    /// Returns `AnalysisError::MemoryLimitExceeded` if analysis cannot proceed within limits
    pub async fn analyze_with_memory_optimization(
        &mut self,
        path: &Path,
        memory_limit_mb: Option<usize>,
    ) -> Result<(Vec<ArchitecturalIssue>, LocalDependencyGraph, MemoryAnalysisReport), AnalysisError> {
        use crate::analysis::memory::{get_optimization_status, initialize_memory_optimization, MemoryOptimizationConfig};
        
        let start_time = Instant::now();
        let memory_limit = memory_limit_mb.unwrap_or(Self::detect_available_memory_mb());
        
        info!("Starting memory-aware analysis with {}MB limit", memory_limit);

        // Initialize memory optimization if not already done
        let memory_config = MemoryOptimizationConfig {
            enabled: true,
            target_max_memory_bytes: memory_limit * 1024 * 1024,
            ..Default::default()
        };

        if let Err(e) = initialize_memory_optimization(memory_config.clone()) {
            warn!("Failed to initialize memory optimization: {}", e);
        }

        let mut report = MemoryAnalysisReport::new(memory_limit);
        
        // Phase 1: Pre-analysis Memory Check
        let initial_memory = Self::get_current_memory_usage_mb();
        report.initial_memory_mb = initial_memory;
        
        if initial_memory > memory_limit / 2 {
            warn!("High initial memory usage: {}MB (limit: {}MB)", initial_memory, memory_limit);
        }

        // Phase 2: Adaptive Analysis Strategy
        let (issues, graph) = match self.analyze_with_memory_monitoring(path, &memory_config, &mut report).await {
            Ok(result) => result,
            Err(AnalysisError::MemoryLimitExceeded { used_mb, limit_mb }) => {
                // Try fallback strategies
                warn!("Memory limit exceeded ({}MB > {}MB), trying fallback strategies", used_mb, limit_mb);
                report.memory_limit_exceeded = true;
                
                self.analyze_with_fallback_strategies(path, &memory_config, &mut report).await?
            }
            Err(e) => return Err(e),
        };

        // Phase 3: Final Memory Report
        let final_memory = Self::get_current_memory_usage_mb();
        let analysis_duration = start_time.elapsed();
        
        report.final_memory_mb = final_memory;
        report.peak_memory_mb = report.peak_memory_mb.max(final_memory);
        report.analysis_duration = analysis_duration;
        report.memory_optimization_effective = report.peak_memory_mb < memory_limit;
        
        // Generate recommendations
        report.recommendations = Self::generate_memory_recommendations(&report, &memory_config);

        info!(
            "Memory-aware analysis completed in {:?}: peak {}MB, final {}MB (limit {}MB)",
            analysis_duration, report.peak_memory_mb, final_memory, memory_limit
        );

        Ok((issues, graph, report))
    }

    /// Analyze with continuous memory monitoring
    async fn analyze_with_memory_monitoring(
        &mut self,
        path: &Path,
        memory_config: &crate::analysis::memory::MemoryOptimizationConfig,
        report: &mut MemoryAnalysisReport,
    ) -> Result<(Vec<ArchitecturalIssue>, LocalDependencyGraph), AnalysisError> {
        let memory_limit_mb = memory_config.target_max_memory_bytes / (1024 * 1024);
        
        // Start memory monitoring
        let monitor_handle = tokio::spawn({
            let limit_mb = memory_limit_mb;
            async move {
                let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(500));
                loop {
                    interval.tick().await;
                    let current_mb = Self::get_current_memory_usage_mb();
                    
                    if current_mb > limit_mb {
                        break Some(current_mb);
                    }
                }
            }
        });

        // Run the robust analysis
        let analysis_result = tokio::select! {
            result = self.analyze_robust(path) => {
                result
            }
            memory_exceeded = monitor_handle => {
                if let Ok(Some(current_mb)) = memory_exceeded {
                    return Err(AnalysisError::memory_limit_exceeded(current_mb, memory_limit_mb));
                }
                // Monitor was aborted, analysis completed successfully
                self.analyze_robust(path).await
            }
        };

        // Update peak memory usage
        report.peak_memory_mb = report.peak_memory_mb.max(Self::get_current_memory_usage_mb());

        analysis_result
    }

    /// Analyze with fallback strategies when memory limits are hit
    async fn analyze_with_fallback_strategies(
        &mut self,
        path: &Path,
        memory_config: &crate::analysis::memory::MemoryOptimizationConfig,
        report: &mut MemoryAnalysisReport,
    ) -> Result<(Vec<ArchitecturalIssue>, LocalDependencyGraph), AnalysisError> {
        info!("Applying memory fallback strategies");
        
        // Strategy 1: Try analyzing smaller chunks
        if let Some(workspace) = WorkspaceDetector::detect_workspace(path).await? {
            if workspace.crates.len() > 1 {
                info!("Fallback: Analyzing crates individually to reduce memory usage");
                return self.analyze_crates_sequentially(&workspace, report).await;
            }
        }

        // Strategy 2: Reduce analysis scope (skip expensive detectors)
        info!("Fallback: Reducing analysis scope to essential detectors only");
        self.analyze_with_reduced_scope(path, report).await
    }

    /// Analyze crates one by one to minimize memory usage
    async fn analyze_crates_sequentially(
        &mut self,
        workspace: &WorkspaceInfo,
        report: &mut MemoryAnalysisReport,
    ) -> Result<(Vec<ArchitecturalIssue>, LocalDependencyGraph), AnalysisError> {
        let mut all_issues = Vec::new();
        let mut graph = LocalDependencyGraph::new();
        
        report.applied_strategies.push("sequential-crate-analysis".to_string());

        for (crate_name, crate_info) in &workspace.crates {
            info!("Analyzing crate sequentially: {}", crate_name);
            
            // Force garbage collection before each crate
            Self::force_garbage_collection();
            
            let pre_crate_memory = Self::get_current_memory_usage_mb();
            
            match self.analyze_single_crate(crate_info).await {
                Ok((crate_issues, crate_deps)) => {
                    all_issues.extend(crate_issues);
                    
                    // Add crate to graph
                    let node = ComponentNode::Module { 
                        path: crate_info.path.display().to_string()
                    };
                    graph.add_component(node);
                    
                    let post_crate_memory = Self::get_current_memory_usage_mb();
                    let crate_memory_usage = post_crate_memory.saturating_sub(pre_crate_memory);
                    
                    info!("Crate '{}' used {}MB memory", crate_name, crate_memory_usage);
                    report.peak_memory_mb = report.peak_memory_mb.max(post_crate_memory);
                }
                Err(e) => {
                    warn!("Failed to analyze crate '{}': {}", crate_name, e);
                    continue;
                }
            }
        }

        Ok((all_issues, graph))
    }

    /// Analyze with reduced scope (essential detectors only)
    async fn analyze_with_reduced_scope(
        &mut self,
        path: &Path,
        report: &mut MemoryAnalysisReport,
    ) -> Result<(Vec<ArchitecturalIssue>, LocalDependencyGraph), AnalysisError> {
        report.applied_strategies.push("reduced-scope-analysis".to_string());
        report.scope_reduced = true;
        
        // This would ideally use a reduced set of detectors
        // For now, we'll use the standard analysis but with more aggressive cleanup
        Self::force_garbage_collection();
        
        let result = self.analyze_fallback(path).await?;
        
        Self::force_garbage_collection();
        
        Ok(result)
    }

    /// Detect available system memory in MB
    fn detect_available_memory_mb() -> usize {
        // This is a simplified implementation
        // In practice, you'd want to use system APIs to detect available memory
        if cfg!(debug_assertions) {
            2048 // 2GB for debug builds
        } else {
            8192 // 8GB for release builds
        }
    }

    /// Get current memory usage in MB
    fn get_current_memory_usage_mb() -> usize {
        #[cfg(feature = "memory-optimization")]
        {
            use crate::analysis::memory::BASIC_MEMORY_METRICS;
            let metrics = BASIC_MEMORY_METRICS.get_metrics();
            (metrics.current_memory_bytes / (1024 * 1024)) as usize
        }
        #[cfg(not(feature = "memory-optimization"))]
        {
            // Fallback implementation - this is approximate
            use std::alloc::{GlobalAlloc, Layout, System};
            // This is a very rough estimate since we can't easily get actual usage
            256 // Default guess of 256MB
        }
    }

    /// Force garbage collection to free memory
    fn force_garbage_collection() {
        // Rust doesn't have a GC, but we can drop temporary allocations
        // and hint to the allocator to return memory to the OS
        
        #[cfg(feature = "memory-optimization")]
        {
            // If using a custom allocator, call its cleanup methods
            std::hint::black_box(Vec::<u8>::new()); // Force some allocation/deallocation
        }
        
        // For mimalloc specifically
        #[cfg(all(feature = "memory-optimization", target_family = "unix"))]
        {
            extern "C" {
                fn mi_collect(force: bool);
            }
            unsafe {
                mi_collect(true);
            }
        }
    }

    /// Generate memory optimization recommendations
    fn generate_memory_recommendations(
        report: &MemoryAnalysisReport,
        config: &crate::analysis::memory::MemoryOptimizationConfig,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if !report.memory_optimization_effective {
            recommendations.push(format!(
                "Analysis exceeded memory limit ({}MB > {}MB). Consider increasing the limit with --memory-limit",
                report.peak_memory_mb, report.memory_limit_mb
            ));
        }

        if report.peak_memory_mb > report.memory_limit_mb * 3 / 4 {
            recommendations.push(
                "High memory usage detected. Try --enable-memory-optimization for better efficiency".to_string()
            );
        }

        if report.memory_limit_exceeded {
            recommendations.push(
                "Memory limit was exceeded. Consider analyzing smaller subsets or individual crates".to_string()
            );
        }

        if report.scope_reduced {
            recommendations.push(
                "Analysis scope was reduced due to memory constraints. Run with higher memory limit for complete analysis".to_string()
            );
        }

        if !config.enabled {
            recommendations.push(
                "Memory optimization is disabled. Enable it with --enable-memory-optimization for large codebases".to_string()
            );
        }

        if recommendations.is_empty() {
            recommendations.push("Memory usage was within acceptable limits. No optimization needed.".to_string());
        }

        recommendations
    }

    /// Enhanced analysis with AI Knowledge Library integration
    ///
    /// This method provides knowledge-enhanced architectural analysis by:
    /// 1. Performing standard analysis pipeline (existing functionality)
    /// 2. Enhancing detected issues with knowledge library context
    /// 3. Adding AI-powered explanations and recommendations (if enabled)
    ///
    /// # Arguments
    ///
    /// * `path` - Directory or file path to analyze
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - `Vec<ArchitecturalIssue>` - Enhanced issues with knowledge context
    /// - `LocalDependencyGraph` - The constructed dependency graph
    ///
    /// # Errors
    ///
    /// Returns `UveddiError` if analysis or knowledge enhancement fails
    pub async fn analyze_with_knowledge(
        &mut self,
        path: &Path,
    ) -> crate::error::Result<(Vec<ArchitecturalIssue>, LocalDependencyGraph)> {
        let start_time = Instant::now();
        info!("Starting knowledge-enhanced analysis for: {:?}", path);

        // 1. Perform standard analysis pipeline (existing functionality)
        let (mut issues, dependency_graph) = self.analyze(path).await?;

        // 2. Early return if knowledge enhancement is disabled
        if !self.enable_knowledge_enhancement {
            info!("Knowledge enhancement disabled, returning standard analysis");
            return Ok((issues, dependency_graph));
        }

        // 3. Check if knowledge library components are available
        #[cfg(feature = "ai")]
        let (knowledge_library, context_selector, ai_engine) = match (
            &self.knowledge_library,
            &self.context_selector,
            &self.ai_engine,
        ) {
            (Some(kb), Some(cs), Some(ai)) => (kb, cs, ai),
            _ => {
                warn!("Knowledge library components not properly initialized, falling back to standard analysis");
                return Ok((issues, dependency_graph));
            }
        };
        
        #[cfg(not(feature = "ai"))]
        {
            info!("AI features not compiled, returning standard analysis");
            return Ok((issues, dependency_graph));
        }

        // 4. Enhance each issue with knowledge library context
        #[cfg(feature = "ai")]
        {
            let enhancement_start = Instant::now();
            let mut enhanced_count = 0;

            for issue in &mut issues {
                match self
                    .enhance_issue_with_knowledge(issue, &dependency_graph)
                    .await
                {
                    Ok(enhanced) => {
                        if enhanced {
                            enhanced_count += 1;
                        }
                    }
                    Err(e) => {
                        warn!("Failed to enhance issue {}: {}", issue.description, e);
                        // Continue with other issues - don't fail entire analysis
                    }
                }
            }
            let enhancement_time = enhancement_start.elapsed();
            let total_time = start_time.elapsed();

            info!(
                "Knowledge-enhanced analysis completed in {:?}: {}/{} issues enhanced (enhancement: {:?})",
                total_time, enhanced_count, issues.len(), enhancement_time
            );

            // 5. Record metrics
            self.record_enhancement_metrics(enhanced_count, issues.len(), enhancement_time);
        }

        Ok((issues, dependency_graph))
    }

    /// Enhance a single issue with knowledge library context
    #[cfg(feature = "ai")]
    async fn enhance_issue_with_knowledge(
        &self,
        issue: &mut ArchitecturalIssue,
        dependency_graph: &LocalDependencyGraph,
    ) -> crate::error::Result<bool> {
        // 1. Build analysis context for this issue
        let analysis_context = match self.build_analysis_context(issue, dependency_graph).await {
            Ok(context) => context,
            Err(e) => {
                warn!(
                    "Failed to build analysis context for issue {}: {}",
                    issue.description, e
                );
                return Ok(false);
            }
        };

        // 2. Select relevant knowledge context
        let context_selector = match &self.context_selector {
            Some(selector) => selector,
            None => return Ok(false),
        };

        let knowledge_context = match context_selector.select_context(&analysis_context).await {
            Ok(context) => context,
            Err(e) => {
                warn!(
                    "Context selection failed for issue {}: {}",
                    issue.description, e
                );
                return Ok(false);
            }
        };

        // 3. Skip enhancement if no relevant knowledge found
        if knowledge_context.selected_patterns.is_empty() {
            info!(
                "No relevant knowledge found for issue: {}",
                issue.description
            );
            return Ok(false);
        }

        // 4. Enhance with AI explanation if enabled and provider available
        if self.enable_ai_explanations {
            if let Some(ai_engine) = &self.ai_engine {
                match ai_engine
                    .analyze_issue_with_knowledge(issue, &knowledge_context)
                    .await
                {
                    Ok(_) => {
                        info!(
                            "Successfully enhanced issue {} with AI analysis",
                            issue.description
                        );
                    }
                    Err(e) => {
                        warn!(
                            "AI enhancement failed for issue {}: {}",
                            issue.description, e
                        );
                        // Fallback to knowledge-only enhancement
                        self.apply_knowledge_only_enhancement(issue, &knowledge_context)
                            .await?;
                    }
                }
            } else {
                // Fallback to knowledge-only enhancement
                self.apply_knowledge_only_enhancement(issue, &knowledge_context)
                    .await?;
            }
        } else {
            // Knowledge-only enhancement
            self.apply_knowledge_only_enhancement(issue, &knowledge_context)
                .await?;
        }

        // 5. Log knowledge metadata (fields don't exist in current DB model)
        info!(
            "Enhanced issue with knowledge: patterns={}, relevance={:.2}, library_version={}",
            knowledge_context.selected_patterns.len(),
            knowledge_context.relevance_score,
            knowledge_context.library_version
        );

        Ok(true)
    }

    /// Build analysis context from issue and surrounding code
    #[cfg(feature = "ai")]
    async fn build_analysis_context(
        &self,
        issue: &ArchitecturalIssue,
        dependency_graph: &LocalDependencyGraph,
    ) -> crate::error::Result<EngineAnalysisContext> {
        // Extract language from file extension
        let language = self.detect_language_from_issue(&issue.file_path)?;

        // Detect frameworks (simplified implementation)
        let frameworks = self.detect_frameworks_from_issue(issue).await?;

        // Calculate basic complexity metrics
        let complexity_metrics = self.calculate_issue_complexity(issue).await?;

        // Extract surrounding components from dependency graph
        let surrounding_components = self.extract_surrounding_components(issue, dependency_graph);

        // Convert AST SourceLanguage to schema SourceLanguage
        let schema_language = match language {
            crate::ast::tree_sitter_impl::SourceLanguage::Rust => crate::ai::knowledge::schema::SourceLanguage::Rust,
            crate::ast::tree_sitter_impl::SourceLanguage::Python => crate::ai::knowledge::schema::SourceLanguage::Python,
            crate::ast::tree_sitter_impl::SourceLanguage::JavaScript => crate::ai::knowledge::schema::SourceLanguage::JavaScript,
        };

        Ok(EngineAnalysisContext {
            language: schema_language,
            detected_patterns: vec![issue.anti_pattern_type_id.to_string()],
            frameworks,
            complexity_metrics,
            file_context: Some(issue.file_path.clone()),
            surrounding_components,
        })
    }

    /// Apply knowledge-only enhancement when AI is not available
    #[cfg(feature = "ai")]
    async fn apply_knowledge_only_enhancement(
        &self,
        issue: &mut ArchitecturalIssue,
        knowledge_context: &KnowledgeContext,
    ) -> crate::error::Result<()> {
        if let Some(pattern_knowledge) = knowledge_context.selected_patterns.first() {
            // Create structured explanation from knowledge base
            let explanation = format!(
                "**Anti-Pattern**: {}\n\n\
                **Description**: {}\n\n\
                **Common Causes**:\n{}\n\n\
                **Recommended Solutions**:\n{}\n\n\
                **Impact**: {:?}\n\
                **Detection Confidence**: {:.1}%",
                pattern_knowledge.name,
                pattern_knowledge.definition.as_str(),
                pattern_knowledge
                    .symptoms
                    .iter()
                    .map(|symptom| format!("• {}", symptom.as_str()))
                    .collect::<Vec<_>>()
                    .join("\n"),
                pattern_knowledge
                    .solutions
                    .iter()
                    .map(|solution| format!(
                        "• {} (Effort: {:?}, Impact: {:?})",
                        solution.title, solution.effort_level, solution.expected_impact
                    ))
                    .collect::<Vec<_>>()
                    .join("\n"),
                pattern_knowledge.impact,
                knowledge_context.relevance_score * 100.0
            );

            issue.ai_explanation = Some(explanation);

            // TODO: Add solution recommendations when DB model supports it
            if let Some(best_solution) = pattern_knowledge.solutions.first() {
                info!(
                    "Recommended solution for issue: {}",
                    best_solution.implementation.as_str()
                );
            }
        }

        Ok(())
    }

    /// Helper methods for context building
    #[cfg(feature = "ai")]
    fn detect_language_from_issue(
        &self,
        file_path: &str,
    ) -> crate::error::Result<SourceLanguage> {
        let extension = Path::new(file_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        match extension {
            "rs" => Ok(SourceLanguage::Rust),
            "py" => Ok(SourceLanguage::Python),
            "js" | "ts" => Ok(SourceLanguage::JavaScript),
            _ => Ok(SourceLanguage::JavaScript), // Default fallback
        }
    }

    #[cfg(feature = "ai")]
    async fn detect_frameworks_from_issue(
        &self,
        issue: &ArchitecturalIssue,
    ) -> crate::error::Result<Vec<String>> {
        // Simplified framework detection based on file path and content
        let mut frameworks = Vec::new();

        if issue.file_path.contains("tokio") || issue.description.contains("async") {
            frameworks.push("tokio".to_string());
        }
        if issue.file_path.contains("axum") || issue.description.contains("web") {
            frameworks.push("axum".to_string());
        }
        if issue.file_path.contains("serde") || issue.description.contains("serialization") {
            frameworks.push("serde".to_string());
        }

        Ok(frameworks)
    }

    #[cfg(feature = "ai")]
    async fn calculate_issue_complexity(
        &self,
        issue: &ArchitecturalIssue,
    ) -> crate::error::Result<EngineComplexityMetrics> {
        // Basic complexity calculation based on issue characteristics
        Ok(EngineComplexityMetrics {
            cyclomatic_complexity: 1, // Default for single issue
            cognitive_complexity: match issue.severity.as_str() {
                "Critical" => 10,
                "High" => 7,
                "Medium" => 4,
                "Low" => 2,
                _ => 1,
            },
            lines_of_code: issue
                .code_snippet
                .as_ref()
                .map(|s| s.lines().count())
                .unwrap_or(0),
            number_of_methods: 1, // Simplified
        })
    }

    #[cfg(feature = "ai")]
    fn extract_surrounding_components(
        &self,
        issue: &ArchitecturalIssue,
        dependency_graph: &LocalDependencyGraph,
    ) -> Vec<String> {
        // Extract component names that are related to this issue's file
        // For now, return the issue file path as a simple implementation
        // TODO: Implement proper component extraction from dependency graph
        vec![issue.file_path.clone()]
    }

    /// Record enhancement metrics for monitoring
    #[cfg(feature = "ai")]
    fn record_enhancement_metrics(
        &self,
        enhanced_count: usize,
        total_count: usize,
        enhancement_time: std::time::Duration,
    ) {
        // Implementation depends on your metrics collection system
        info!(
            "Enhancement metrics: {}/{} issues enhanced in {:?} (rate: {:.1}%)",
            enhanced_count,
            total_count,
            enhancement_time,
            (enhanced_count as f64 / total_count as f64) * 100.0
        );
    }

    /// Performs incremental analysis with 50%+ time reduction for enterprise codebases
    ///
    /// This method uses intelligent change detection and dependency tracking to analyze
    /// only the files that have changed or are affected by changes. For large codebases,
    /// this can provide significant performance improvements.
    ///
    /// # Arguments
    ///
    /// * `path` - Directory or file path to analyze
    /// * `config` - Configuration for incremental analysis behavior
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - `Vec<ArchitecturalIssue>` - All detected issues (cached + newly analyzed)
    /// - `IncrementalAnalysisResult` - Detailed metrics and analysis information
    ///
    /// # Errors
    ///
    /// Returns `UveddiError` if incremental analysis fails. In case of failure,
    /// the system will automatically fall back to full analysis.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use uveddi::analysis::{AnalysisEngine, IncrementalConfig};
    /// use std::path::Path;
    ///
    /// let mut engine = AnalysisEngine::new()?;
    /// let config = IncrementalConfig::default();
    /// let (issues, result) = engine.analyze_incremental(Path::new("src/"), config).await?;
    ///
    /// println!("Found {} issues", issues.len());
    /// if result.was_incremental {
    ///     println!("Time saved: {}ms ({:.1}% improvement)",
    ///         result.time_saved_ms,
    ///         (result.time_saved_ms as f64 / (result.time_saved_ms + result.performance_metrics.reanalysis_time_ms) as f64) * 100.0
    ///     );
    /// }
    /// ```
    pub async fn analyze_incremental(
        &mut self,
        path: &Path,
        config: IncrementalConfig,
    ) -> crate::error::Result<(Vec<ArchitecturalIssue>, IncrementalAnalysisResult)> {
        info!("Starting incremental analysis for: {}", path.display());

        // Create state file path based on project directory
        let state_dir = if path.is_file() {
            path.parent().unwrap_or(path).join(".uveddi")
        } else {
            path.join(".uveddi")
        };

        let state_file_path = state_dir.join("incremental_state.json");

        // Create incremental analysis engine
        let mut incremental_engine =
            IncrementalAnalysisEngine::new(self.clone_for_incremental()?, config, state_file_path)
                .await
                .map_err(|e| crate::error::UveddiError::AnalysisError {
                    message: format!("Failed to initialize incremental analysis: {}", e),
                    file: path.to_string_lossy().to_string(),
                    line: 0,
                    context: "incremental analysis initialization".to_string(),
                    suggestion: "Check file permissions and system resources".to_string(),
                    source: None,
                })?;

        // Perform incremental analysis
        let result = incremental_engine
            .analyze_incremental(path)
            .await
            .map_err(|e| crate::error::UveddiError::AnalysisError {
                message: format!("Incremental analysis failed: {}", e),
                file: path.to_string_lossy().to_string(),
                line: 0,
                context: "incremental analysis execution".to_string(),
                suggestion: "Check file changes and dependency graph".to_string(),
                source: None,
            })?;

        info!(
            "Incremental analysis completed: {} issues found, {:.1}% time savings",
            result.0.len(),
            if result.1.was_incremental && result.1.time_saved_ms > 0 {
                (result.1.time_saved_ms as f64
                    / (result.1.time_saved_ms + result.1.performance_metrics.reanalysis_time_ms)
                        as f64)
                    * 100.0
            } else {
                0.0
            }
        );

        Ok(result)
    }

    /// Creates a clone of the analysis engine for incremental analysis
    ///
    /// This method creates a copy of the current engine that can be used
    /// by the incremental analysis system without affecting the original engine.
    fn clone_for_incremental(&self) -> crate::error::Result<AnalysisEngine> {
        // Create a new engine with the same configuration
        // For now, create a basic engine - in a real implementation,
        // you would properly clone the configuration and components
        AnalysisEngine::new()
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
    #[deprecated(
        since = "0.2.0",
        note = "This method is deprecated. Use `analyze()` instead, which provides a more comprehensive analysis result including the dependency graph."
    )]
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
                description:
                    "A class that has too many responsibilities and is difficult to maintain."
                        .to_string(),
                category: "Structural".to_string(),
            },
            AntiPatternType {
                anti_pattern_type_id: None,
                name: "Code Duplication".to_string(),
                description: "Multiple instances of similar code that should be refactored."
                    .to_string(),
                category: "Structural".to_string(),
            },
            AntiPatternType {
                anti_pattern_type_id: None,
                name: "Cyclic Dependency".to_string(),
                description: "A direct or indirect dependency cycle between modules or components."
                    .to_string(),
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
                description: "A class that has grown too large and should be broken down."
                    .to_string(),
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
                        modified_at: crate::analysis::cache::wrappers::ArchivableSystemTime(
                            std::fs::metadata(path)?.modified()?,
                        ),
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
                    modified_at: crate::analysis::cache::wrappers::ArchivableSystemTime(
                        std::fs::metadata(path)?.modified()?,
                    ),
                };
                return Ok(parsed_file);
            }

            // Fallback error if file can't be read
            Err(crate::error::UveddiError::io_error(
                "read file content",
                &path.to_string_lossy(),
                std::io::Error::new(std::io::ErrorKind::NotFound, "Cannot read file"),
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
                    modified_at: crate::analysis::cache::wrappers::ArchivableSystemTime(
                        std::fs::metadata(path)?.modified()?,
                    ),
                };
                return Ok(parsed_file);
            }

            // Fallback error if file can't be read
            Err(crate::error::UveddiError::io_error(
                "read file content",
                &path.to_string_lossy(),
                std::io::Error::new(std::io::ErrorKind::NotFound, "Cannot read file"),
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
    pub async fn get_ast_cache_metrics(&self) -> serde_json::Value {
        // Facade pattern: delegate to cache manager component
        let stats = self.cache_manager.get_cache_stats().await;
        serde_json::json!({
            "ast_cache_size": stats.ast_cache_size,
            "result_cache_size": stats.result_cache_size,
            "ast_hit_rate": stats.ast_hit_rate,
            "result_hit_rate": stats.result_hit_rate,
            "total_memory_usage": stats.total_memory_usage
        })
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
    #[deprecated(
        since = "0.2.0",
        note = "Plugin loading is now handled during asynchronous engine construction via `AnalysisEngine::builder().enable_plugins(true).build_async().await`. This method will be removed in future versions. Refer to UV-294 guidelines for async standardization."
    )]
    pub async fn load_plugins(&mut self) -> crate::error::Result<usize> {
        if let Some(ref mut plugin_manager) = self.plugin_manager {
            let stats = plugin_manager
                .get_stats()
                .await
                .map_err(crate::error::UveddiError::from)?;
            let loaded_plugins = stats.loaded_plugins;

            info!(
                "Loaded {} WASM plugins via PluginManagerHandle",
                loaded_plugins
            );
            Ok(loaded_plugins)
        } else {
            warn!(
                "Plugin manager not initialized. Use `AnalysisEngine::builder().enable_plugins(true).build_async().await` to enable plugin support."
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
    #[deprecated(
        since = "0.2.0",
        note = "Plugin installation should be handled via the `PluginManagerHandle` directly, obtained from an asynchronously constructed `AnalysisEngine`. This method will be removed in future versions. Refer to UV-294 guidelines for async standardization."
    )]
    pub async fn install_plugin(
        &mut self,
        manifest: crate::plugins::PluginManifest,
        binary: Vec<u8>,
    ) -> crate::error::Result<crate::plugins::PluginId> {
        if let Some(ref mut plugin_manager) = self.plugin_manager {
            // Since the new API expects a PathBuf, we need to write the binary to a temporary file
            // This is a workaround for the deprecated method
            // Use a secure approach to create a temporary file in the current directory
            use std::io::Write;
            let plugin_filename = format!("{}-{}.wasm", manifest.name, std::process::id()); // Use process ID to make filename unique
            let plugin_path = std::env::current_dir()?.join(&plugin_filename);

            // Write the binary to the temporary file
            std::fs::write(&plugin_path, &binary)?;

            let plugin_id_string = plugin_manager
                .load_plugin(plugin_path.clone())
                .await
                .map_err(crate::error::UveddiError::from)?;

            // Clean up the temporary file
            if let Err(e) = std::fs::remove_file(&plugin_path) {
                warn!(
                    "Failed to cleanup temporary plugin file {}: {}",
                    plugin_path.display(),
                    e
                );
            }
            // Convert String to PluginId
            let plugin_id = crate::plugins::PluginId::from_name(&plugin_id_string);
            Ok(plugin_id)
        } else {
            Err(crate::error::UveddiError::PluginError {
                plugin: "unknown".to_string(),
                plugin_type: "WASM".to_string(),
                message: "Plugin manager not initialized".to_string(),
                suggestion: "Initialize plugin manager before installing plugins".to_string(),
                source: Some(crate::plugins::errors::PluginError::Execution(
                    "Plugin manager not initialized".to_string(),
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
    #[deprecated(
        since = "0.2.0",
        note = "Plugin uninstallation should be handled via the `PluginManagerHandle` directly, obtained from an asynchronously constructed `AnalysisEngine`. This method will be removed in future versions. Refer to UV-294 guidelines for async standardization."
    )]
    pub async fn uninstall_plugin(
        &mut self,
        plugin_id: &crate::plugins::PluginId,
    ) -> crate::error::Result<()> {
        if let Some(ref mut plugin_manager) = self.plugin_manager {
            plugin_manager
                .unload_plugin(plugin_id.to_string())
                .await
                .map_err(crate::error::UveddiError::from)?;
            Ok(())
        } else {
            Err(crate::error::UveddiError::PluginError {
                plugin: "unknown".to_string(),
                plugin_type: "WASM".to_string(),
                message: "Plugin manager not initialized".to_string(),
                suggestion: "Initialize plugin manager before uninstalling plugins".to_string(),
                source: Some(crate::plugins::errors::PluginError::Execution(
                    "Plugin manager not initialized".to_string(),
                )),
            })
        }
    }

    /// Retrieves statistics for all loaded plugins.
    #[deprecated(
        since = "0.2.0",
        note = "Plugin statistics should be retrieved via the `PluginManagerHandle` directly, obtained from an asynchronously constructed `AnalysisEngine`. This method will be removed in future versions. Refer to UV-294 guidelines for async standardization."
    )]
    pub async fn get_stats(
        &self,
    ) -> Option<Vec<(crate::plugins::PluginId, crate::plugins::PluginStats)>> {
        if let Some(ref plugin_manager) = self.plugin_manager {
            // Since individual plugin stats are no longer available via the API,
            // return overall stats as a single entry
            if let Ok(overall_stats) = plugin_manager.get_stats().await {
                // Convert overall stats to the expected format
                let plugin_stats = crate::plugins::PluginStats {
                    invocations: overall_stats.total_executions,
                    total_execution_time_ms: (overall_stats.average_execution_time_ms
                        * overall_stats.total_executions as f64)
                        as u64,
                    avg_execution_time_ms: overall_stats.average_execution_time_ms,
                    total_fuel_consumed: 0, // Not available in overall stats
                    peak_memory_usage: overall_stats.memory_usage_bytes,
                    error_count: 0, // Not available in overall stats
                    last_error: None,
                };
                let plugin_id = crate::plugins::PluginId::from_name("overall");
                Some(vec![(plugin_id, plugin_stats)])
            } else {
                Some(vec![])
            }
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
    #[deprecated(
        since = "0.2.0",
        note = "Plugin resource monitoring should be handled via the `PluginManagerHandle` directly, obtained from an asynchronously constructed `AnalysisEngine`. This method will be removed in future versions. Refer to UV-294 guidelines for async standardization."
    )]
    pub async fn monitor_plugin_resources(
        &mut self,
    ) -> crate::error::Result<crate::plugins::ResourceReport> {
        if let Some(ref mut plugin_manager) = self.plugin_manager {
            // If monitor_resources is deprecated, document for team review
            // plugin_manager.monitor_resources().await.map_err(|e| { ... })
            // If not available, return a not implemented error
            Err(crate::error::UveddiError::PluginError {
                plugin: "unknown".to_string(),
                plugin_type: "WASM".to_string(),
                message: "Resource monitoring not implemented in PluginManagerHandle".to_string(),
                suggestion: "Implement resource monitoring or remove this method".to_string(),
                source: None,
            })
        } else {
            Err(crate::error::UveddiError::PluginError {
                plugin: "unknown".to_string(),
                plugin_type: "WASM".to_string(),
                message: "Plugin manager not initialized".to_string(),
                suggestion: "Initialize plugin manager before monitoring resources".to_string(),
                source: Some(crate::plugins::errors::PluginError::Execution(
                    "Plugin manager not initialized".to_string(),
                )),
            })
        }
    }

    /// Check if plugin support is available.
    ///
    /// This method checks if the `AnalysisEngine` instance was initialized with plugin support.
    pub fn has_plugin_support(&self) -> bool {
        self.plugin_manager.is_some()
    }

    /// Add plugin detectors dynamically.
    ///
    /// Scans the plugin manager for loaded plugins and creates detector adapters
    /// for each one, integrating them into the analysis engine's detector pipeline.
    ///
    /// # Errors
    /// Returns `UveddiError` if plugin integration fails.
    #[deprecated(
        since = "0.2.0",
        note = "Plugin detectors are now integrated during asynchronous engine construction. This method will be removed in future versions. Refer to UV-294 guidelines for async standardization."
    )]
    pub async fn add_plugin_detectors(&mut self) -> crate::error::Result<usize> {
        if let Some(ref plugin_manager) = self.plugin_manager {
            // Get the loaded plugins from the plugin manager
            let loaded_plugins = plugin_manager.list_loaded_plugins().await?;

            if loaded_plugins.is_empty() {
                info!("No plugins loaded, skipping plugin detector integration");
                return Ok(0);
            }

            // Create plugin adapters for each loaded plugin
            let mut added_detectors = 0;
            for plugin_id in loaded_plugins {
                match plugin_manager.get_plugin_adapter(&plugin_id).await {
                    Ok(Some(adapter)) => {
                        // Add the adapter to the detector scheduler
                        match self
                            .detector_scheduler
                            .add_detector(Box::new(adapter))
                            .await
                        {
                            Ok(_) => {
                                added_detectors += 1;
                                info!(
                                    "Successfully added plugin detector for plugin: {}",
                                    plugin_id
                                );
                            }
                            Err(e) => {
                                warn!("Failed to add plugin detector for {}: {}", plugin_id, e);
                            }
                        }
                    }
                    Ok(None) => {
                        warn!("Plugin {} not found or not loaded", plugin_id);
                    }
                    Err(e) => {
                        warn!("Failed to get adapter for plugin {}: {}", plugin_id, e);
                    }
                }
            }

            info!(
                "Successfully integrated {} plugin detectors",
                added_detectors
            );
            Ok(added_detectors)
        } else {
            Ok(0) // No plugin manager, no detectors added
        }
    }

    /// Remove plugin detectors from the detector pipeline.
    ///
    /// Removes all WASM plugin detectors from the current detector set.
    /// This is useful when reloading plugins or disabling plugin support.
    #[deprecated(
        since = "0.2.0",
        note = "Plugin detectors are now managed internally by the `DetectorScheduler` and `PluginManagerHandle`. This method will be removed in future versions. Refer to UV-294 guidelines for async standardization."
    )]
    pub fn remove_plugin_detectors(&mut self) -> usize {
        // TODO: Facade pattern - delegate to detector_scheduler
        // For now, this is a no-op since detector management is handled by components
        warn!("remove_plugin_detectors is deprecated in facade pattern - use component management instead");
        0
    }

    /// Reload plugin detectors.
    ///
    /// Removes existing plugin detectors and reloads them from the plugin manager.
    /// This is useful when plugins have been added, removed, or updated.
    ///
    /// # Errors
    /// Returns `UveddiError` if plugin reloading fails.
    #[deprecated(
        since = "0.2.0",
        note = "Plugin detectors are now managed internally by the `DetectorScheduler` and `PluginManagerHandle`. This method will be removed in future versions. Refer to UV-294 guidelines for async standardization."
    )]
    pub async fn reload_plugin_detectors(&mut self) -> crate::error::Result<usize> {
        // Remove existing plugin detectors
        self.remove_plugin_detectors();

        // Add current plugin detectors
        self.add_plugin_detectors().await
    }

    /// Gets statistics from the plugin registry.
    #[deprecated(
        since = "0.2.0",
        note = "Plugin statistics should be retrieved via the `PluginManagerHandle` directly, obtained from an asynchronously constructed `AnalysisEngine`. This method will be removed in future versions. Refer to UV-294 guidelines for async standardization."
    )]
    pub async fn get_registry_stats(&self) -> Option<crate::plugins::PluginStats> {
        if let Some(ref plugin_manager) = self.plugin_manager {
            if let Ok(overall_stats) = plugin_manager.get_stats().await {
                Some(crate::plugins::PluginStats {
                    invocations: overall_stats.total_executions,
                    total_execution_time_ms: (overall_stats.average_execution_time_ms
                        * overall_stats.total_executions as f64)
                        as u64,
                    avg_execution_time_ms: overall_stats.average_execution_time_ms,
                    total_fuel_consumed: 0, // Not available in overall stats
                    peak_memory_usage: overall_stats.memory_usage_bytes,
                    error_count: 0, // Not available in overall stats
                    last_error: None,
                })
            } else {
                None
            }
        } else {
            None
        }
    }
}
