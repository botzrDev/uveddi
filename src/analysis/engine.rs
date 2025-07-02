use crate::analysis::detectors::anti_patterns::god_object::GodObjectDetector;
use crate::analysis::detectors::cycle::CycleDetector;
use crate::analysis::detectors::dependency::{Dependency, DependencyExtractor};
use crate::analysis::graph::dependency::LocalDependencyGraph;
use crate::analysis::graph::dependency::{ComponentNode, LocalDependencyType};
use crate::analysis::AnalysisDetector;
use crate::ast::tree_sitter::AstParser;
use crate::cache::result_cache::ResultCache;
use crate::database::models::{AntiPatternType, ArchitecturalIssue};
use crate::ingestion::AsyncWalker;
use log::{info, warn};
use std::path::{Path, PathBuf};
use tokio_stream::StreamExt;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct CachedAnalysisResult {
    issues: Vec<ArchitecturalIssue>,
    dependencies: Vec<Dependency>,
}

/// Core analysis engine that orchestrates the detection of anti-patterns and architectural issues
///
/// The `AnalysisEngine` is the main entry point for running code analysis. It manages
/// a collection of detectors, coordinates file parsing, builds dependency graphs,
/// and aggregates results from all analysis phases.
///
/// ## Key Features:
///
/// - **Multi-detector Support**: Runs multiple detectors in parallel
/// - **Caching**: Caches parsing and analysis results for performance
/// - **Dependency Analysis**: Builds and analyzes dependency graphs
/// - **Async Processing**: Processes files asynchronously for better performance
/// - **Error Recovery**: Continues analysis even if individual files fail
///
/// ## Architecture:
///
/// 1. **File Discovery**: Uses `AsyncWalker` to find source files
/// 2. **Parsing**: Leverages `AstParser` to create syntax trees
/// 3. **Detection**: Runs all registered detectors on each file
/// 4. **Graph Analysis**: Builds dependency graph and runs graph-based detectors
/// 5. **Result Aggregation**: Combines all detected issues into final report
///
/// ## Usage
///
/// ```no_run
/// use uveddi::analysis::AnalysisEngine;
/// use std::path::Path;
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
    detectors: Vec<Box<dyn AnalysisDetector>>,
    cycle_detector: CycleDetector,
    files_analyzed: i32,
    cache: ResultCache,
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
        Ok(Self {
            ast_parser: AstParser::new()?,
            dependency_extractor: DependencyExtractor::new()?,
            detectors: vec![Box::new(GodObjectDetector::new(15, 20))],
            cycle_detector: CycleDetector::new(),
            files_analyzed: 0,
            cache: ResultCache::new(&cache_path)?,
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
                path: dep.to_module.clone(),
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
                        all_issues.extend(cached_result.issues);
                        all_dependencies.extend(cached_result.dependencies);
                        self.files_analyzed += 1;
                        continue;
                    }

                    info!("CACHE MISS: Analyzing file: {}", file_path.display());

                    match self.ast_parser.parse_file(&file_path) {
                        Ok(parsed_file) => {
                            self.files_analyzed += 1;

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

                            let result_to_cache = CachedAnalysisResult {
                                issues: file_issues.clone(),
                                dependencies: file_dependencies.clone(),
                            };

                            if let Err(e) = self.cache.set(&file_path, &result_to_cache) {
                                warn!(
                                    "Failed to cache analysis for {}: {}",
                                    file_path.display(),
                                    e
                                );
                            }

                            all_issues.extend(file_issues);
                            all_dependencies.extend(file_dependencies);
                        }
                        Err(e) => warn!("Failed to parse file {}: {}", file_path.display(), e),
                    }
                }
                Err(e) => warn!("Error walking directory: {}", e),
            }
        }

        info!(
            "Analyzed {} files and extracted dependencies.",
            self.files_analyzed
        );
        Ok((all_issues, all_dependencies))
    }

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

    pub fn get_files_analyzed(&self) -> i32 {
        self.files_analyzed
    }
}
