use crate::analysis::graph::dependency::{ComponentNode, LocalDependencyGraph};
use crate::analysis::{AnalysisDetector, AnalysisError};
use crate::ast::tree_sitter_impl::{ParsedFile, SourceLanguage};
use crate::database::models::{AntiPatternType, ArchitecturalIssue};
use async_trait::async_trait;
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, info};

use super::{
    analysis::DependencyAnalyzer,
    config::TightCouplingConfig,
    issue_evaluator::IssueEvaluator,
    language_support::{LanguageAnalyzer, PythonAnalyzer, RustAnalyzer, TypeScriptAnalyzer},
    report::{CouplingAnalysisReport, ReportGenerator},
    types::{CouplingThresholds, Dependency},
};

/// Detector for tight coupling anti-patterns
pub struct TightCouplingDetector {
    config: TightCouplingConfig,
    dependency_analyzer: DependencyAnalyzer,
    issue_evaluator: IssueEvaluator,
    report_generator: ReportGenerator,
    language_analyzers: HashMap<SourceLanguage, Box<dyn LanguageAnalyzer>>,
}

impl Default for TightCouplingDetector {
    fn default() -> Self {
        Self::new(TightCouplingConfig::default())
    }
}

impl std::fmt::Debug for TightCouplingDetector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TightCouplingDetector")
            .field("config", &self.config)
            .field("dependency_analyzer", &self.dependency_analyzer)
            .field("issue_evaluator", &self.issue_evaluator)
            .field("report_generator", &self.report_generator)
            .field("language_analyzers", &"[trait objects]")
            .finish()
    }
}

impl TightCouplingDetector {
    /// Creates a new tight coupling detector with custom configuration
    pub fn new(config: TightCouplingConfig) -> Self {
        let mut language_analyzers: HashMap<SourceLanguage, Box<dyn LanguageAnalyzer>> = HashMap::new();
        language_analyzers.insert(SourceLanguage::Rust, Box::new(RustAnalyzer::default()));
        language_analyzers.insert(SourceLanguage::Python, Box::new(PythonAnalyzer::default()));
        language_analyzers.insert(SourceLanguage::JavaScript, Box::new(TypeScriptAnalyzer::default()));
        language_analyzers.insert(SourceLanguage::TypeScript, Box::new(TypeScriptAnalyzer::default()));

        Self {
            config,
            dependency_analyzer: DependencyAnalyzer::new(),
            issue_evaluator: IssueEvaluator::new(),
            report_generator: ReportGenerator::new(),
            language_analyzers,
        }
    }

    /// Creates a new tight coupling detector with default configuration
    pub fn with_default_config() -> Self {
        Self::new(TightCouplingConfig::default())
    }

    /// Get thresholds for a specific language
    fn get_thresholds_for_language(&self, language: SourceLanguage) -> &CouplingThresholds {
        match language {
            SourceLanguage::Rust => &self.config.rust_thresholds,
            SourceLanguage::Python => &self.config.python_thresholds,
            SourceLanguage::JavaScript => &self.config.javascript_thresholds,
            SourceLanguage::TypeScript => &self.config.javascript_thresholds, // Reuse JS thresholds
        }
    }

    /// Get analyzer for a specific language
    fn get_analyzer_for_language(&self, language: SourceLanguage) -> Option<&dyn LanguageAnalyzer> {
        self.language_analyzers.get(&language).map(|analyzer| analyzer.as_ref())
    }

    /// Build dependency graph from multiple files with parallel processing
    pub fn build_project_dependency_graph(
        &self,
        files: &[(String, ParsedFile)],
    ) -> Result<LocalDependencyGraph, AnalysisError> {
        self.dependency_analyzer.build_project_dependency_graph(files, &self.language_analyzers)
    }

    /// Build dependency graph incrementally for changed files only
    pub fn build_incremental_dependency_graph(
        &self,
        changed_files: &[(String, ParsedFile)],
        existing_graph: &LocalDependencyGraph,
    ) -> Result<LocalDependencyGraph, AnalysisError> {
        self.dependency_analyzer.build_incremental_dependency_graph(
            changed_files,
            existing_graph,
            &self.language_analyzers,
        )
    }

    /// Analyze multiple files in parallel and return coupling issues
    pub fn analyze_files_parallel(
        &self,
        files: &[(String, ParsedFile)],
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        if !self.config.enable_cross_file_analysis {
            // Analyze each file independently in parallel
            let issues_result: Result<Vec<Vec<ArchitecturalIssue>>, AnalysisError> = files
                .par_iter()
                .map(|(_, parsed_file)| {
                    tokio::task::block_in_place(|| {
                        tokio::runtime::Handle::current().block_on(self.detect_issues(parsed_file))
                    })
                })
                .collect();

            let all_issues: Vec<ArchitecturalIssue> =
                issues_result?.into_iter().flatten().collect();
            Ok(all_issues)
        } else {
            // Cross-file analysis requires building full dependency graph
            let graph = self.build_project_dependency_graph(files)?;
            let graph_issues = self.detect_graph_issues(&graph, 0);
            Ok(graph_issues)
        }
    }

    /// Generate comprehensive coupling analysis report
    pub fn generate_analysis_report(
        &self,
        graph: &LocalDependencyGraph,
        dependencies: &[Dependency],
    ) -> CouplingAnalysisReport {
        self.report_generator.generate_report(graph, dependencies, &self.config)
    }

    /// Infer language from component file path
    fn infer_language_from_component(&self, component: &ComponentNode) -> SourceLanguage {
        let file_path = match component {
            ComponentNode::Class { file_path, .. } => file_path,
            ComponentNode::Function { file_path, .. } => file_path,
            ComponentNode::Module { path } => path,
        };

        SourceLanguage::from_path(Path::new(file_path)).unwrap_or(SourceLanguage::Rust)
    }
}

/// Cross-file analysis trait for project-wide coupling detection
pub trait CrossFileAnalysisDetector {
    fn detect_cross_file_issues(
        &self,
        files: &[(String, ParsedFile)],
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError>;
}

impl CrossFileAnalysisDetector for TightCouplingDetector {
    fn detect_cross_file_issues(
        &self,
        files: &[(String, ParsedFile)],
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        info!(
            "Starting cross-file coupling analysis for {} files",
            files.len()
        );

        self.analyze_files_parallel(files)
    }
}

#[async_trait]
impl AnalysisDetector for TightCouplingDetector {
    async fn detect_issues(
        &self,
        file: &ParsedFile,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        // Single-file analysis for basic coupling detection
        if let Some(analyzer) = self.get_analyzer_for_language(file.language) {
            let dependencies = analyzer.extract_dependencies(file.path(), file)?;

            // Create simple metrics based on file-level dependencies
            let mut component_deps: HashMap<String, usize> = HashMap::new();
            for dep in dependencies {
                let component_key = match &dep.from_component {
                    ComponentNode::Class { name, .. } => name.clone(),
                    ComponentNode::Function { name, .. } => name.clone(),
                    ComponentNode::Module { path } => path.clone(),
                };
                *component_deps.entry(component_key).or_insert(0) += 1;
            }

            let mut issues = Vec::new();
            let thresholds = self.get_thresholds_for_language(file.language);

            for (component_name, dep_count) in component_deps {
                if dep_count >= thresholds.fan_out_critical {
                    issues.push(ArchitecturalIssue::new(
                        0, // analysis_run_id
                        3, // anti_pattern_type_id: Tight coupling
                        file.path().to_string_lossy().to_string(),
                        None, // line_number
                        format!("Component '{}' has {} dependencies (critical threshold: {})",
                            component_name, dep_count, thresholds.fan_out_critical),
                        "TightCouplingDetector".to_string(),
                        "Critical".to_string(),
                        format!(
                            "Component '{}' has {} dependencies, which exceeds the critical threshold of {}. \
                             This indicates tight coupling and makes the code harder to maintain.",
                            component_name, dep_count, thresholds.fan_out_critical
                        ),
                    ));
                }
            }

            Ok(issues)
        } else {
            Ok(Vec::new())
        }
    }

    fn detect_graph_issues(
        &self,
        graph: &LocalDependencyGraph,
        analysis_run_id: i64,
    ) -> Vec<ArchitecturalIssue> {
        debug!(
            "Starting graph-level tight coupling analysis with {} nodes",
            graph.get_petgraph().node_count()
        );

        // Calculate coupling metrics for all components
        let report = self.generate_analysis_report(graph, &[]);
        let metrics = report.metrics;

        // Group components by language for threshold evaluation
        let mut issues_by_language: HashMap<SourceLanguage, Vec<ArchitecturalIssue>> =
            HashMap::new();

        for (component, metric) in &metrics {
            let language = self.infer_language_from_component(component);
            let thresholds = self.get_thresholds_for_language(language);
            let component_issues = self.issue_evaluator.evaluate_coupling_issues(
                &[(component.clone(), metric.clone())].into_iter().collect(),
                language,
                thresholds,
            );

            issues_by_language
                .entry(language)
                .or_insert_with(Vec::new)
                .extend(component_issues);
        }

        // Flatten all issues and set analysis_run_id
        let all_issues: Vec<ArchitecturalIssue> = issues_by_language
            .into_values()
            .flatten()
            .map(|mut issue| {
                issue.analysis_run_id = analysis_run_id;
                issue
            })
            .collect();

        info!(
            "Found {} tight coupling issues in dependency graph",
            all_issues.len()
        );

        all_issues
    }

    fn get_detector_name(&self) -> &'static str {
        "TightCouplingDetector"
    }

    fn get_anti_pattern_types(&self) -> Vec<AntiPatternType> {
        vec![AntiPatternType {
            anti_pattern_type_id: Some(3),
            name: "Tight Coupling".to_string(),
            description: "Components that are overly dependent on each other's internal implementation details".to_string(),
            category: "structural".to_string(),
        }]
    }
}