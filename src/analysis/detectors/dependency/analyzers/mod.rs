pub mod circular_deps;
pub mod dependency_graph;
pub mod outdated_deps;
pub mod transitive_analyzer;
pub mod version_analyzer;

pub use circular_deps::{CircularDependencyAnalysis, CircularDependencyDetector};
pub use dependency_graph::{DependencyGraphAnalyzer, GraphAnalysisResult};
pub use outdated_deps::{OutdatedAnalysis, OutdatedDependencyChecker};
pub use transitive_analyzer::{TransitiveAnalysisResult, TransitiveAnalyzer};
pub use version_analyzer::{VersionAnalysis, VersionAnalyzer};

use crate::analysis::detectors::dependency::config::*;
use crate::analysis::detectors::dependency::types::*;

pub trait DependencyAnalyzer: Send + Sync {
    fn analyze(
        &self,
        dependencies: &[DependencyInfo],
        config: &DependencyDetectorConfig,
    ) -> Result<AnalysisOutput, DependencyError>;

    fn name(&self) -> &str;
    fn description(&self) -> &str;
}

#[derive(Debug, Clone)]
pub enum AnalysisOutput {
    Graph(GraphAnalysisResult),
    Version(VersionAnalysis),
    Circular(CircularDependencyAnalysis),
    Outdated(OutdatedAnalysis),
    Transitive(TransitiveAnalysisResult),
}

pub struct AnalyzerRegistry {
    analyzers: Vec<Box<dyn DependencyAnalyzer>>,
}

impl AnalyzerRegistry {
    pub fn new() -> Self {
        Self {
            analyzers: vec![
                Box::new(DependencyGraphAnalyzer::new()),
                Box::new(VersionAnalyzer::new()),
                Box::new(CircularDependencyDetector::new()),
                Box::new(OutdatedDependencyChecker::new()),
                Box::new(TransitiveAnalyzer::new()),
            ],
        }
    }

    pub fn run_all(
        &self,
        dependencies: &[DependencyInfo],
        config: &DependencyDetectorConfig,
    ) -> Vec<Result<AnalysisOutput, DependencyError>> {
        let mut results = Vec::new();
        for analyzer in &self.analyzers {
            results.push(analyzer.analyze(dependencies, config));
        }
        results
    }
}

impl Default for AnalyzerRegistry {
    fn default() -> Self {
        Self::new()
    }
}
