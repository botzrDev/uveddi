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
    analysis::{CouplingCalculator, DependencyAnalyzer},
    config::TightCouplingConfig,
    language_support::{LanguageAnalyzer, PythonAnalyzer, RustAnalyzer, TypeScriptAnalyzer},
    metrics::{AfferentCouplingCalculator, EfferentCouplingCalculator, InstabilityCalculator},
    types::{CouplingMetrics, CouplingThresholds, Dependency},
    visualization::{CouplingMatrixGenerator, DependencyGraphVisualizer},
};

/// Detector for tight coupling anti-patterns
pub struct TightCouplingDetector {
    config: TightCouplingConfig,
    dependency_analyzer: DependencyAnalyzer,
    coupling_calculator: CouplingCalculator,
    afferent_calculator: AfferentCouplingCalculator,
    efferent_calculator: EfferentCouplingCalculator,
    instability_calculator: InstabilityCalculator,
    graph_visualizer: DependencyGraphVisualizer,
    matrix_generator: CouplingMatrixGenerator,
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
            .field("coupling_calculator", &self.coupling_calculator)
            .field("afferent_calculator", &self.afferent_calculator)
            .field("efferent_calculator", &self.efferent_calculator)
            .field("instability_calculator", &self.instability_calculator)
            .field("graph_visualizer", &self.graph_visualizer)
            .field("matrix_generator", &self.matrix_generator)
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
            coupling_calculator: CouplingCalculator::new(),
            afferent_calculator: AfferentCouplingCalculator::new(),
            efferent_calculator: EfferentCouplingCalculator::new(),
            instability_calculator: InstabilityCalculator::new(),
            graph_visualizer: DependencyGraphVisualizer::new(),
            matrix_generator: CouplingMatrixGenerator::new(),
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

    /// Evaluate coupling issues based on thresholds
    fn evaluate_coupling_issues(
        &self,
        metrics: &HashMap<ComponentNode, CouplingMetrics>,
        language: SourceLanguage,
    ) -> Vec<ArchitecturalIssue> {
        let mut issues = Vec::new();
        let thresholds = self.get_thresholds_for_language(language);

        for (component, metric) in metrics {
            // Evaluate fan-out
            if metric.fan_out >= thresholds.fan_out_critical {
                issues.push(self.create_coupling_issue(
                    component,
                    "Critical",
                    format!(
                        "Fan-out {} exceeds critical threshold {}",
                        metric.fan_out, thresholds.fan_out_critical
                    ),
                    metric,
                ));
            } else if metric.fan_out >= thresholds.fan_out_warning {
                issues.push(self.create_coupling_issue(
                    component,
                    "Warning",
                    format!(
                        "Fan-out {} exceeds warning threshold {}",
                        metric.fan_out, thresholds.fan_out_warning
                    ),
                    metric,
                ));
            }

            // Evaluate CBO
            if metric.cbo >= thresholds.cbo_critical {
                issues.push(self.create_coupling_issue(
                    component,
                    "Critical",
                    format!(
                        "CBO {} exceeds critical threshold {}",
                        metric.cbo, thresholds.cbo_critical
                    ),
                    metric,
                ));
            } else if metric.cbo >= thresholds.cbo_warning {
                issues.push(self.create_coupling_issue(
                    component,
                    "Warning",
                    format!(
                        "CBO {} exceeds warning threshold {}",
                        metric.cbo, thresholds.cbo_warning
                    ),
                    metric,
                ));
            }

            // Evaluate RFC
            if metric.rfc >= thresholds.rfc_critical {
                issues.push(self.create_coupling_issue(
                    component,
                    "Critical",
                    format!(
                        "RFC {} exceeds critical threshold {}",
                        metric.rfc, thresholds.rfc_critical
                    ),
                    metric,
                ));
            } else if metric.rfc >= thresholds.rfc_warning {
                issues.push(self.create_coupling_issue(
                    component,
                    "Warning",
                    format!(
                        "RFC {} exceeds warning threshold {}",
                        metric.rfc, thresholds.rfc_warning
                    ),
                    metric,
                ));
            }
        }

        issues
    }

    /// Create a coupling issue
    fn create_coupling_issue(
        &self,
        component: &ComponentNode,
        severity: &str,
        description: String,
        metrics: &CouplingMetrics,
    ) -> ArchitecturalIssue {
        let (file_path, component_name) = match component {
            ComponentNode::Class { name, file_path } => (file_path.clone(), name.clone()),
            ComponentNode::Function { name, file_path } => (file_path.clone(), name.clone()),
            ComponentNode::Module { path } => (path.clone(), "module".to_string()),
        };

        let mut issue = ArchitecturalIssue::new(
            0, // analysis_run_id will be set by caller
            3, // anti_pattern_type_id for tight coupling
            file_path,
            None, // line_number
            description.clone(),
            "TightCouplingDetector".to_string(),
            severity.to_string(),
            description,
        );
        issue.start_line = None;
        issue.end_line = None;
        issue.code_snippet = Some(component_name);
        issue.ai_explanation = Some(format!(
            "Reduce coupling by: 1) Using dependency injection, 2) Applying interfaces/traits, 3) Reducing direct dependencies. Current metrics: Fan-out={}, Fan-in={}, CBO={}, RFC={}",
            metrics.fan_out, metrics.fan_in, metrics.cbo, metrics.rfc
        ));
        issue
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

    /// Infer language from component file path
    fn infer_language_from_component(&self, component: &ComponentNode) -> SourceLanguage {
        let file_path = match component {
            ComponentNode::Class { file_path, .. } => file_path,
            ComponentNode::Function { file_path, .. } => file_path,
            ComponentNode::Module { path } => path,
        };

        SourceLanguage::from_path(Path::new(file_path)).unwrap_or(SourceLanguage::Rust)
    }

    /// Generate comprehensive coupling analysis report
    pub fn generate_analysis_report(
        &self,
        graph: &LocalDependencyGraph,
        dependencies: &[Dependency],
    ) -> CouplingAnalysisReport {
        info!("Generating comprehensive coupling analysis report");

        let metrics = self.coupling_calculator.calculate_metrics(graph);
        let stability_analysis = self.instability_calculator.analyze_system_stability(graph);
        let hotspots = self.coupling_calculator.identify_coupling_hotspots(&metrics, 0.9);

        let visualization_data = if self.config.visualization_enabled {
            Some(self.graph_visualizer.generate_graph_data(graph, &metrics, dependencies))
        } else {
            None
        };

        let coupling_matrix = if self.config.generate_coupling_matrix {
            Some(self.matrix_generator.generate_matrix(graph, &metrics, dependencies))
        } else {
            None
        };

        CouplingAnalysisReport {
            metrics,
            stability_analysis,
            hotspots,
            visualization_data,
            coupling_matrix,
        }
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
        let metrics = self.coupling_calculator.calculate_metrics(graph);

        // Group components by language for threshold evaluation
        let mut issues_by_language: HashMap<SourceLanguage, Vec<ArchitecturalIssue>> =
            HashMap::new();

        for (component, metric) in &metrics {
            let language = self.infer_language_from_component(component);
            let component_issues = self.evaluate_coupling_issues(
                &[(component.clone(), metric.clone())].into_iter().collect(),
                language,
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

/// Comprehensive coupling analysis report
#[derive(Debug)]
pub struct CouplingAnalysisReport {
    pub metrics: HashMap<ComponentNode, CouplingMetrics>,
    pub stability_analysis: super::metrics::instability_calculator::StabilityAnalysis,
    pub hotspots: Vec<(ComponentNode, CouplingMetrics)>,
    pub visualization_data: Option<super::visualization::dependency_graph::DependencyGraphData>,
    pub coupling_matrix: Option<super::visualization::coupling_matrix::CouplingMatrix>,
}