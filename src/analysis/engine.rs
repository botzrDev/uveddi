use crate::analysis::detectors::anti_patterns::code_duplication::CodeDuplicationDetector;
use crate::analysis::detectors::anti_patterns::dead_code::{DeadCodeConfig, DeadCodeDetector};
use crate::analysis::detectors::anti_patterns::god_object::GodObjectDetector;
use crate::analysis::detectors::anti_patterns::large_classes::{
    LargeClassConfig, LargeClassDetector,
};
use crate::analysis::detectors::anti_patterns::tight_coupling::TightCouplingDetector;
use crate::analysis::detectors::cycle::CycleDetector;
use crate::analysis::detectors::dependency::{Dependency, DependencyExtractor};
use crate::analysis::extractors::SymbolExtractor;
use crate::analysis::graph::dependency::LocalDependencyGraph;
use crate::analysis::graph::dependency::{ComponentNode, LocalDependencyType};
use crate::analysis::symbols::GlobalSymbolTable;
use crate::analysis::AnalysisDetector;
use crate::ast::AstParser;
use crate::cache::result_cache::ResultCache;
use crate::database::models::{AntiPatternType, ArchitecturalIssue};
use crate::ingestion::AsyncWalker;
use crate::plugins::WasmPluginEngine;
use log::{info, warn};

use std::path::{Path, PathBuf};
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
    ast_parser: AstParser,
    dependency_extractor: DependencyExtractor,
    symbol_extractor: SymbolExtractor,
    detectors: Vec<Box<dyn AnalysisDetector + Send + Sync>>,
    cycle_detector: CycleDetector,
    files_analyzed: i32,
    cache: ResultCache,
    symbol_table: GlobalSymbolTable,
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
    pub fn new() -> Result<Self, crate::error::UveddiError> {
        let cache_path = PathBuf::from("uveddi_cache.db");
        Self::with_cache_path(&cache_path)
    }

    /// Creates a new analysis engine with WASM plugin support enabled
    pub async fn new_with_plugins() -> Result<Self, crate::error::UveddiError> {
        let cache_path = PathBuf::from("uveddi_cache.db");
        Self::with_cache_path_and_plugins(&cache_path).await
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
    pub fn with_cache_path(cache_path: &Path) -> Result<Self, crate::error::UveddiError> {
        Ok(Self {
            ast_parser: AstParser::new()?,
            dependency_extractor: DependencyExtractor::new()?,
            symbol_extractor: SymbolExtractor::new(),
            detectors: vec![
                Box::new(GodObjectDetector::new(5, 8)), // More sensitive thresholds
                Box::new(CodeDuplicationDetector::new()),
                Box::new(DeadCodeDetector::with_default_config()),
                Box::new(LargeClassDetector::with_default_config()),
                Box::new(TightCouplingDetector::default()),
            ],
            cycle_detector: CycleDetector::new(),
            files_analyzed: 0,
            cache: ResultCache::new(cache_path)?,
            symbol_table: GlobalSymbolTable::new(),
            plugin_engine: None,
        })
    }

    /// Creates a new analysis engine with WASM plugin support and custom cache path
    pub async fn with_cache_path_and_plugins(
        cache_path: &Path,
    ) -> Result<Self, crate::error::UveddiError> {
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

        Ok(Self {
            ast_parser: AstParser::new()?,
            dependency_extractor: DependencyExtractor::new()?,
            symbol_extractor: SymbolExtractor::new(),
            detectors: vec![
                Box::new(GodObjectDetector::new(5, 8)), // More sensitive thresholds
                Box::new(CodeDuplicationDetector::new()),
                Box::new(DeadCodeDetector::with_default_config()),
                Box::new(LargeClassDetector::with_default_config()),
                Box::new(TightCouplingDetector::default()),
            ],
            cycle_detector: CycleDetector::new(),
            files_analyzed: 0,
            cache: ResultCache::new(cache_path)?,
            symbol_table: GlobalSymbolTable::new(),
            plugin_engine,
        })
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
    pub fn new_with_memory_cache() -> Result<Self, crate::error::UveddiError> {
        Ok(Self {
            ast_parser: AstParser::new()?,
            dependency_extractor: DependencyExtractor::new()?,
            symbol_extractor: SymbolExtractor::new(),
            detectors: vec![
                Box::new(GodObjectDetector::new(5, 8)), // More sensitive thresholds
                Box::new(CodeDuplicationDetector::new()),
                Box::new(DeadCodeDetector::with_default_config()),
                Box::new(LargeClassDetector::with_default_config()),
                Box::new(TightCouplingDetector::default()),
            ],
            cycle_detector: CycleDetector::new(),
            files_analyzed: 0,
            cache: ResultCache::new_in_memory()?,
            symbol_table: GlobalSymbolTable::new(),
            plugin_engine: None,
        })
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
    ) -> Result<(Vec<ArchitecturalIssue>, LocalDependencyGraph), crate::error::UveddiError> {
        let (mut file_issues, all_dependencies) =
            self.analyze_files_and_collect_dependencies(path).await?;

        info!("Building dependency graph...");
        let mut dependency_graph = LocalDependencyGraph::new();
        for dep in all_dependencies {
            let from_node = ComponentNode::Module {
                path: dep.from_file.to_string_lossy().into_owned(),
            };
            let to_node = ComponentNode::Module {
                path: dep.to_module, // UV-153: Move instead of clone
            };
            dependency_graph.add_dependency(&from_node, &to_node, LocalDependencyType::Import);
        }
        info!("Dependency graph built.");

        info!("Detecting cycles...");
        // Assuming analysis_run_id is 0 for now. This will be managed by a higher-level process.
        let cycle_issues = self.cycle_detector.detect_cycles(&dependency_graph, 0);
        info!("Found {} cycles.", cycle_issues.len());
        file_issues.extend(cycle_issues);

        // Run graph-based anti-pattern detectors
        for detector in &self.detectors {
            let issues = detector.detect(&dependency_graph);
            file_issues.extend(issues);
        }

        Ok((file_issues, dependency_graph))
    }

    /// Analyzes files in the given path and collects dependencies
    ///
    /// This internal method:
    /// 1. Walks the directory tree to find source files
    /// 2. Checks the cache for existing analysis results
    /// 3. Parses and analyzes files not found in cache
    /// 4. Extracts symbols and dependencies
    /// 5. Caches the results for future use
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
    /// Returns `UveddiError` if:
    /// - File walking fails
    /// - Parsing errors occur
    /// - Cache operations fail
    async fn analyze_files_and_collect_dependencies(
        &mut self,
        path: &Path,
    ) -> Result<(Vec<ArchitecturalIssue>, Vec<Dependency>), crate::error::UveddiError> {
        let mut all_issues = Vec::new();
        let mut all_dependencies = Vec::new();
        self.files_analyzed = 0;

        let walker = AsyncWalker::for_source_code();
        let mut file_stream = walker.walk(path);

        while let Some(file_result) = file_stream.next().await {
            match file_result {
                Ok(file_path) => {
                    if let Some(cached_result) =
                        self.cache.get::<_, CachedAnalysisResult>(&file_path)?
                    {
                        info!(
                            "CACHE HIT: Using cached analysis for {}",
                            file_path.display()
                        );
                        // UV-220: Use move semantics for cached result aggregation
                        all_issues.extend(cached_result.issues);
                        all_dependencies.extend(cached_result.dependencies);
                        self.files_analyzed += 1;
                        continue;
                    }

                    info!("CACHE MISS: Analyzing file: {}", file_path.display());

                    match self.ast_parser.parse_file(&file_path) {
                        Ok(parsed_file) => {
                            self.files_analyzed += 1;

                            // Populate symbol table
                            if let Err(e) = self
                                .symbol_extractor
                                .extract_declarations(&parsed_file, &mut self.symbol_table)
                            {
                                warn!(
                                    "Could not extract symbols from {}: {}",
                                    file_path.display(),
                                    e
                                );
                            }

                            let mut file_issues = Vec::new();
                            let mut file_dependencies = Vec::new();

                            // Run file-level detectors
                            for detector in &self.detectors {
                                match detector.detect_issues(&parsed_file) {
                                    Ok(mut issues) => file_issues.append(&mut issues),
                                    Err(e) => warn!(
                                        "Error running detector {} on {}: {}",
                                        detector.get_detector_name(),
                                        file_path.display(),
                                        e
                                    ),
                                }
                            }

                            // Extract dependencies
                            match self.dependency_extractor.extract_from_ast(&parsed_file) {
                                Ok(mut dependencies) => file_dependencies.append(&mut dependencies),
                                Err(e) => warn!(
                                    "Error extracting dependencies from {}: {}",
                                    file_path.display(),
                                    e
                                ),
                            }

                            // UV-220: Streaming aggregation - process results efficiently
                            let result_to_cache = CachedAnalysisResult {
                                issues: file_issues.clone(), // Required for cache storage
                                dependencies: file_dependencies.clone(), // Required for cache storage
                            };

                            if let Err(e) = self.cache.set(&file_path, &result_to_cache) {
                                warn!(
                                    "Failed to cache analysis for {}: {}",
                                    file_path.display(),
                                    e
                                );
                            }

                            // UV-220: Move semantics for efficient aggregation
                            all_issues.extend(file_issues);
                            all_dependencies.extend(file_dependencies);
                        }
                        Err(e) => warn!("Failed to parse file {}: {}", file_path.display(), e),
                    }
                }
                Err(e) => warn!("Error walking directory: {e}"),
            }
        }

        info!(
            "Analyzed {} files and extracted dependencies.",
            self.files_analyzed
        );
        Ok((all_issues, all_dependencies))
    }

    /// Returns a list of all anti-pattern types supported by the registered detectors.
    ///
    /// This method aggregates the anti-pattern types from all configured detectors,
    /// including a built-in type for cyclic dependencies.
    pub fn get_anti_pattern_types(&self) -> Vec<AntiPatternType> {
        let mut types = Vec::new();
        for detector in &self.detectors {
            types.extend(detector.get_anti_pattern_types());
        }
        // Add cycle dependency type
        types.push(AntiPatternType {
            anti_pattern_type_id: None, // Will be assigned by DB
            name: "Cyclic Dependency".to_string(),
            description: "A direct or indirect dependency cycle between modules or components."
                .to_string(),
            category: "Structural".to_string(),
        });
        types
    }

    /// Gets the number of files analyzed in the last run.
    pub fn get_files_analyzed(&self) -> i32 {
        self.files_analyzed
    }

    /// Configures the dead code detector with custom settings.
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration for the dead code detector.
    pub fn configure_dead_code_detector(&mut self, config: DeadCodeConfig) {
        // Find and replace the dead code detector
        for detector in &mut self.detectors {
            if detector.get_detector_name() == "DeadCodeDetector" {
                // We need to replace the detector since we can't modify it in place
                break;
            }
        }

        // Remove the old detector and add the new one
        self.detectors
            .retain(|d| d.get_detector_name() != "DeadCodeDetector");
        self.detectors.push(Box::new(DeadCodeDetector::new(config)));
    }

    /// Configures the large classes detector with custom settings.
    ///
    /// # Arguments
    ///
    /// * `config` - The configuration for the large classes detector.
    pub fn configure_large_classes_detector(&mut self, config: LargeClassConfig) {
        // Remove the old detector and add the new one
        self.detectors
            .retain(|d| d.get_detector_name() != "LargeClassDetector");
        self.detectors
            .push(Box::new(LargeClassDetector::new(config)));
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
    pub async fn load_plugins(&mut self) -> Result<usize, crate::error::UveddiError> {
        if let Some(ref mut plugin_engine) = self.plugin_engine {
            let loaded_plugins = plugin_engine
                .load_all_plugins()
                .await
                .map_err(|e| crate::error::UveddiError::PluginError(e.to_string()))?;

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
    ) -> Result<crate::plugins::PluginId, crate::error::UveddiError> {
        if let Some(ref mut plugin_engine) = self.plugin_engine {
            let plugin_id = plugin_engine
                .install_plugin(manifest, binary)
                .await
                .map_err(|e| crate::error::UveddiError::PluginError(e.to_string()))?;

            // TODO: Re-enable when plugin adapter implements AnalysisDetector
            // Add the new plugin as a detector
            // if let Some(adapter) = plugin_engine.get_plugin_adapter(&plugin_id).await {
            //     self.detectors.push(Box::new(adapter));
            // }

            Ok(plugin_id)
        } else {
            Err(crate::error::UveddiError::PluginError(
                "Plugin engine not initialized".to_string(),
            ))
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
    ) -> Result<(), crate::error::UveddiError> {
        if let Some(ref mut plugin_engine) = self.plugin_engine {
            plugin_engine
                .uninstall_plugin(plugin_id)
                .await
                .map_err(|e| crate::error::UveddiError::PluginError(e.to_string()))?;

            // TODO: Re-enable when plugin adapter implements AnalysisDetector
            // Reload all adapters to remove the uninstalled plugin
            // self.detectors.retain(|detector| detector.get_detector_name() != "wasm-plugin-detector");

            // Re-add remaining plugin adapters
            // for adapter in plugin_engine.get_all_plugin_adapters().await {
            //     self.detectors.push(Box::new(adapter));
            // }

            Ok(())
        } else {
            Err(crate::error::UveddiError::PluginError(
                "Plugin engine not initialized".to_string(),
            ))
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
    ) -> Result<crate::plugins::ResourceReport, crate::error::UveddiError> {
        if let Some(ref mut plugin_engine) = self.plugin_engine {
            plugin_engine
                .monitor_resources()
                .await
                .map_err(|e| crate::error::UveddiError::PluginError(e.to_string()))
        } else {
            Err(crate::error::UveddiError::PluginError(
                "Plugin engine not initialized".to_string(),
            ))
        }
    }

    /// Check if plugin engine is available
    pub fn has_plugin_support(&self) -> bool {
        self.plugin_engine.is_some()
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
