use crate::ast::tree_sitter::AstParser;
use crate::analysis::dependency_graph::{ComponentNode, LocalDependencyType};
use crate::analysis::AnalysisDetector;
use crate::analysis::anti_patterns::god_object_detector::GodObjectDetector;
use crate::analysis::anti_patterns::unstable_interface_detector::UnstableInterfaceDetector;
use crate::analysis::anti_patterns::modularity_violation_detector::ModularityViolationDetector;
use crate::analysis::cycle_detector::CycleDetector;
use crate::analysis::dependency_extractor::{Dependency, DependencyExtractor};
use crate::analysis::dependency_graph::LocalDependencyGraph;
use crate::database::models::{ArchitecturalIssue, AntiPatternType};
use crate::ingestion::AsyncWalker;
use crate::cache::result_cache::ResultCache;
use std::path::{Path, PathBuf};
use log::{info, warn};
use tokio_stream::StreamExt;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct CachedAnalysisResult {
    issues: Vec<ArchitecturalIssue>,
    dependencies: Vec<Dependency>,
}

pub struct AnalysisEngine {
    ast_parser: AstParser,
    dependency_extractor: DependencyExtractor,
    detectors: Vec<Box<dyn AnalysisDetector>>,
    cycle_detector: CycleDetector,
    files_analyzed: i32,
    cache: ResultCache,
}

impl AnalysisEngine {
    pub fn new() -> Result<Self, crate::error::UveddiError> {
        let cache_path = PathBuf::from("uveddi_cache.db");
        Ok(Self {
            ast_parser: AstParser::new()?,
            dependency_extractor: DependencyExtractor::new()?,
            detectors: vec![
                Box::new(GodObjectDetector::new(15, 20)),
                Box::new(UnstableInterfaceDetector::new()),
                Box::new(ModularityViolationDetector::new()),
            ],
            cycle_detector: CycleDetector::new(),
            files_analyzed: 0,
            cache: ResultCache::new(&cache_path)?,
        })
    }

    pub async fn analyze(&mut self, path: &Path) -> Result<(Vec<ArchitecturalIssue>, LocalDependencyGraph), crate::error::UveddiError> {
        let (mut file_issues, all_dependencies) = self.analyze_files_and_collect_dependencies(path).await?;

        info!("Building dependency graph...");
        let mut dependency_graph = LocalDependencyGraph::new();
        for dep in all_dependencies {
            let from_node = ComponentNode::Module { path: dep.from_file.to_string_lossy().into_owned() };
            let to_node = ComponentNode::Module { path: dep.to_module.clone() };
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

    async fn analyze_files_and_collect_dependencies(&mut self, path: &Path) -> Result<(Vec<ArchitecturalIssue>, Vec<Dependency>), crate::error::UveddiError> {
        let mut all_issues = Vec::new();
        let mut all_dependencies = Vec::new();
        self.files_analyzed = 0;

        let walker = AsyncWalker::for_source_code();
        let mut file_stream = walker.walk(path);

        while let Some(file_result) = file_stream.next().await {
            match file_result {
                Ok(file_path) => {
                    if let Some(cached_result) = self.cache.get::<_, CachedAnalysisResult>(&file_path)? {
                        info!("CACHE HIT: Using cached analysis for {}", file_path.display());
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
                                    Err(e) => warn!("Error running detector {} on {}: {}", detector.get_detector_name(), file_path.display(), e),
                                }
                            }

                            // Extract dependencies
                            match self.dependency_extractor.extract_from_ast(&parsed_file) {
                                Ok(mut dependencies) => file_dependencies.append(&mut dependencies),
                                Err(e) => warn!("Error extracting dependencies from {}: {}", file_path.display(), e),
                            }

                            let result_to_cache = CachedAnalysisResult {
                                issues: file_issues.clone(),
                                dependencies: file_dependencies.clone(),
                            };

                            if let Err(e) = self.cache.set(&file_path, &result_to_cache) {
                                warn!("Failed to cache analysis for {}: {}", file_path.display(), e);
                            }

                            all_issues.extend(file_issues);
                            all_dependencies.extend(file_dependencies);
                        },
                        Err(e) => warn!("Failed to parse file {}: {}", file_path.display(), e),
                    }
                },
                Err(e) => warn!("Error walking directory: {}", e),
            }
        }
        
        info!("Analyzed {} files and extracted dependencies.", self.files_analyzed);
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
            description: "A direct or indirect dependency cycle between modules or components.".to_string(),
            category: "Structural".to_string(),
        });
        types
    }

    pub fn get_files_analyzed(&self) -> i32 {
        self.files_analyzed
    }
}
