use crate::ast::tree_sitter::AstParser;
use crate::analysis::AnalysisDetector;
use crate::analysis::anti_patterns::god_object_detector::GodObjectDetector;
use crate::analysis::anti_patterns::unstable_interface_detector::UnstableInterfaceDetector;
use crate::analysis::anti_patterns::modularity_violation_detector::ModularityViolationDetector;
use crate::analysis::cycle_detector::CycleDetector;
use crate::analysis::dependency_extractor::{Dependency, DependencyExtractor};
use crate::analysis::dependency_graph::DependencyGraph;
use crate::database::models::{ArchitecturalIssue, AntiPatternType};
use std::path::Path;
use log::{info, warn};

pub struct AnalysisEngine {
    ast_parser: AstParser,
    dependency_extractor: DependencyExtractor,
    detectors: Vec<Box<dyn AnalysisDetector>>,
    cycle_detector: CycleDetector,
    files_analyzed: i32,
}

impl AnalysisEngine {
    pub fn new() -> Result<Self, crate::error::UveddiError> {
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
        })
    }

    pub async fn analyze(&mut self, path: &Path) -> Result<Vec<ArchitecturalIssue>, crate::error::UveddiError> {
        let (mut file_issues, all_dependencies) = self.analyze_files_and_collect_dependencies(path).await?;

        info!("Building dependency graph...");
        let mut dependency_graph = DependencyGraph::new();
        dependency_graph.build_from_dependencies(all_dependencies);
        info!("Dependency graph built with {} modules and {} dependencies.", 
              dependency_graph.get_modules().len(), 
              dependency_graph.get_all_dependencies().len());

        info!("Detecting cycles...");
        let cycle_results = self.cycle_detector.detect_cycles(&dependency_graph);
        info!("Found {} cycles.", cycle_results.cycles.len());

        for cycle in cycle_results.cycles {
            file_issues.push(ArchitecturalIssue::from_cycle(cycle, &dependency_graph));
        }

        // Run graph-based anti-pattern detectors
        for detector in &self.detectors {
            let issues = detector.detect(&dependency_graph);
            file_issues.extend(issues);
        }

        Ok(file_issues)
    }

    async fn analyze_files_and_collect_dependencies(&mut self, path: &Path) -> Result<(Vec<ArchitecturalIssue>, Vec<Dependency>), crate::error::UveddiError> {
        let mut all_issues = Vec::new();
        let mut all_dependencies = Vec::new();
        self.files_analyzed = 0;

        for entry in walkdir::WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file()) {
            
            let file_path = entry.path();
            info!("Analyzing file: {}", file_path.display());

            match self.ast_parser.parse_file(file_path) {
                Ok(parsed_file) => {
                    self.files_analyzed += 1;
                    
                    // Run file-level detectors
                    for detector in &self.detectors {
                        match detector.detect_issues(&parsed_file) {
                            Ok(mut issues) => all_issues.append(&mut issues),
                            Err(e) => warn!("Error running detector {} on {}: {}", detector.get_detector_name(), file_path.display(), e),
                        }
                    }

                    // Extract dependencies
                    match self.dependency_extractor.extract_from_ast(&parsed_file) {
                        Ok(mut dependencies) => all_dependencies.append(&mut dependencies),
                        Err(e) => warn!("Error extracting dependencies from {}: {}", file_path.display(), e),
                    }
                },
                Err(e) => warn!("Failed to parse file {}: {}", file_path.display(), e),
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
