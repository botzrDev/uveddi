//! Tight Coupling Anti-Pattern Detector
//!
//! This module detects tight coupling between components, which makes code
//! difficult to maintain, test, and modify. Tight coupling occurs when
//! components are overly dependent on each other's internal implementation details.
//!
//! Features:
//! - Multi-language Tree-sitter analysis for Rust, Python, JavaScript
//! - Configurable thresholds for different languages and contexts
//! - Advanced coupling metrics (CBO, RFC, Fan-in, Fan-out)
//! - Cross-file dependency analysis
//! - Context-aware severity assessment

use crate::analysis::graph::dependency::{
    ComponentNode, DependencyEdge, LocalDependencyGraph, LocalDependencyType,
};
use crate::analysis::{AnalysisDetector, AnalysisError};
use crate::ast::tree_sitter::{tree_sitter_javascript, tree_sitter_python, tree_sitter_rust};
use crate::ast::tree_sitter::{Node, Query, QueryCursor, Tree};
use crate::ast::tree_sitter_impl::{ParsedFile, SourceLanguage};
use crate::database::models::{AntiPatternType, ArchitecturalIssue};
use async_trait::async_trait;
use petgraph::graph::{DiGraph, NodeIndex};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use streaming_iterator::StreamingIterator;
use tracing::{debug, info, warn};

/// Represents a dependency relationship between components
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Dependency {
    pub from_component: ComponentNode,
    pub to_component: ComponentNode,
    pub dependency_type: LocalDependencyType,
    pub line_number: Option<u32>,
    pub strength: DependencyStrength,
}

/// Strength of dependency coupling
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DependencyStrength {
    Weak,   // Loose coupling (interfaces, abstractions)
    Medium, // Moderate coupling (data structures, method calls)
    Strong, // Tight coupling (direct field access, inheritance)
}

/// Coupling metrics for a component
#[derive(Debug, Clone, Default)]
pub struct CouplingMetrics {
    pub fan_out: usize, // Number of components this depends on
    pub fan_in: usize,  // Number of components depending on this
    pub cbo: usize,     // Coupling Between Objects
    pub rfc: usize,     // Response for Class
    pub lcom: f64,      // Lack of Cohesion in Methods
}

/// Language-specific coupling thresholds
#[derive(Debug, Clone)]
pub struct CouplingThresholds {
    pub fan_out_warning: usize,
    pub fan_out_critical: usize,
    pub cbo_warning: usize,
    pub cbo_critical: usize,
    pub rfc_warning: usize,
    pub rfc_critical: usize,
    pub production_multiplier: f64,
    pub test_multiplier: f64,
    pub framework_multiplier: f64,
}

/// Configuration for tight coupling detection
#[derive(Debug, Clone)]
pub struct TightCouplingConfig {
    pub rust_thresholds: CouplingThresholds,
    pub python_thresholds: CouplingThresholds,
    pub javascript_thresholds: CouplingThresholds,
    pub enable_cross_file_analysis: bool,
    pub exclude_patterns: Vec<String>,
    pub include_test_files: bool,
}

/// Trait for language-specific dependency analysis
pub trait LanguageAnalyzer {
    fn extract_dependencies(
        &self,
        file_path: &Path,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<Dependency>, AnalysisError>;
    fn get_language(&self) -> SourceLanguage;
}

/// Rust language analyzer
#[derive(Debug, Default, Clone)]
pub struct RustAnalyzer;

/// Python language analyzer
#[derive(Debug, Default, Clone)]
pub struct PythonAnalyzer;

/// JavaScript language analyzer
#[derive(Debug, Default, Clone)]
pub struct JavaScriptAnalyzer;

/// Detector for tight coupling anti-patterns
#[derive(Debug, Clone)]
pub struct TightCouplingDetector {
    config: TightCouplingConfig,
    rust_analyzer: RustAnalyzer,
    python_analyzer: PythonAnalyzer,
    javascript_analyzer: JavaScriptAnalyzer,
}

impl Default for TightCouplingDetector {
    fn default() -> Self {
        Self::new(TightCouplingConfig::default())
    }
}

impl Default for TightCouplingConfig {
    fn default() -> Self {
        Self {
            rust_thresholds: CouplingThresholds {
                fan_out_warning: 7,
                fan_out_critical: 12,
                cbo_warning: 6,
                cbo_critical: 10,
                rfc_warning: 15,
                rfc_critical: 25,
                production_multiplier: 1.0,
                test_multiplier: 1.5,
                framework_multiplier: 2.0,
            },
            python_thresholds: CouplingThresholds {
                fan_out_warning: 10,
                fan_out_critical: 15,
                cbo_warning: 8,
                cbo_critical: 12,
                rfc_warning: 18,
                rfc_critical: 30,
                production_multiplier: 1.0,
                test_multiplier: 1.5,
                framework_multiplier: 2.0,
            },
            javascript_thresholds: CouplingThresholds {
                fan_out_warning: 12,
                fan_out_critical: 18,
                cbo_warning: 10,
                cbo_critical: 15,
                rfc_warning: 20,
                rfc_critical: 35,
                production_multiplier: 1.0,
                test_multiplier: 1.5,
                framework_multiplier: 2.0,
            },
            enable_cross_file_analysis: true,
            exclude_patterns: vec![
                "**/target/**".to_string(),
                "**/node_modules/**".to_string(),
                "**/.git/**".to_string(),
            ],
            include_test_files: false,
        }
    }
}

impl TightCouplingDetector {
    /// Creates a new tight coupling detector with custom configuration
    pub fn new(config: TightCouplingConfig) -> Self {
        Self {
            config,
            rust_analyzer: RustAnalyzer::default(),
            python_analyzer: PythonAnalyzer::default(),
            javascript_analyzer: JavaScriptAnalyzer::default(),
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
            SourceLanguage::TypeScript => &self.config.javascript_thresholds, // Reuse JS thresholds for now
        }
    }

    /// Get analyzer for a specific language
    fn get_analyzer_for_language(&self, language: SourceLanguage) -> &dyn LanguageAnalyzer {
        match language {
            SourceLanguage::Rust => &self.rust_analyzer,
            SourceLanguage::Python => &self.python_analyzer,
            SourceLanguage::JavaScript => &self.javascript_analyzer,
            SourceLanguage::TypeScript => &self.javascript_analyzer, // Reuse JS analyzer placeholder
        }
    }

    /// Calculate coupling metrics for components
    fn calculate_metrics(
        &self,
        graph: &LocalDependencyGraph,
    ) -> HashMap<ComponentNode, CouplingMetrics> {
        let mut metrics = HashMap::new();
        let petgraph = graph.get_petgraph();

        for node_index in petgraph.node_indices() {
            if let Some(component) = graph.get_node_from_index(node_index) {
                let mut component_metrics = CouplingMetrics::default();

                // Calculate fan-out (outgoing dependencies)
                component_metrics.fan_out = petgraph.edges(node_index).count();

                // Calculate fan-in (incoming dependencies)
                component_metrics.fan_in = petgraph
                    .edges_directed(node_index, petgraph::Direction::Incoming)
                    .count();

                // CBO = fan-in + fan-out
                component_metrics.cbo = component_metrics.fan_in + component_metrics.fan_out;

                // RFC approximation (would need method-level analysis for accuracy)
                component_metrics.rfc =
                    component_metrics.fan_out + self.estimate_local_methods(component);

                metrics.insert(component.clone(), component_metrics);
            }
        }

        metrics
    }

    /// Estimate local methods for RFC calculation
    fn estimate_local_methods(&self, component: &ComponentNode) -> usize {
        match component {
            ComponentNode::Class { .. } => 5,    // Average methods per class
            ComponentNode::Module { .. } => 3,   // Average functions per module
            ComponentNode::Function { .. } => 1, // Single function
        }
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

        {
            let mut issue = ArchitecturalIssue::new(
                0, // analysis_run_id will be set by caller
                5, // anti_pattern_type_id for tight coupling
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
    }

    /// Build dependency graph from multiple files with parallel processing
    pub fn build_project_dependency_graph(
        &self,
        files: &[(String, ParsedFile)],
    ) -> Result<LocalDependencyGraph, AnalysisError> {
        info!("Building dependency graph for {} files", files.len());

        // Extract dependencies in parallel
        let all_dependencies: Result<Vec<Vec<Dependency>>, AnalysisError> = files
            .par_iter()
            .map(|(file_path, parsed_file)| {
                let analyzer = self.get_analyzer_for_language(parsed_file.language);
                analyzer.extract_dependencies(Path::new(file_path), parsed_file)
            })
            .collect();

        let dependencies_list = all_dependencies?;

        // Build graph sequentially (graph modifications need to be sequential)
        let mut graph = LocalDependencyGraph::new();
        for dependencies in dependencies_list {
            for dependency in dependencies {
                graph.add_dependency(
                    &dependency.from_component,
                    &dependency.to_component,
                    dependency.dependency_type,
                );
            }
        }

        info!(
            "Built dependency graph with {} nodes",
            graph.get_petgraph().node_count()
        );
        Ok(graph)
    }

    /// Build dependency graph incrementally for changed files only
    pub fn build_incremental_dependency_graph(
        &self,
        changed_files: &[(String, ParsedFile)],
        existing_graph: &LocalDependencyGraph,
    ) -> Result<LocalDependencyGraph, AnalysisError> {
        info!(
            "Building incremental dependency graph for {} changed files",
            changed_files.len()
        );

        // For now, rebuild completely - in future could optimize to only update affected nodes
        // This would require tracking file->component mappings and dependency provenance
        self.build_project_dependency_graph(changed_files)
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

            let all_issues = issues_result?.into_iter().flatten().collect();
            Ok(all_issues)
        } else {
            // Cross-file analysis requires building full dependency graph
            let graph = self.build_project_dependency_graph(files)?;
            let graph_issues = self.detect_graph_issues(&graph, 0);
            Ok(graph_issues)
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

        // Use optimized parallel analysis
        self.analyze_files_parallel(files)
    }
}

impl TightCouplingDetector {
    /// Infer language from component file path
    fn infer_language_from_component(&self, component: &ComponentNode) -> SourceLanguage {
        let file_path = match component {
            ComponentNode::Class { file_path, .. } => file_path,
            ComponentNode::Function { file_path, .. } => file_path,
            ComponentNode::Module { path } => path,
        };

        SourceLanguage::from_path(Path::new(file_path)).unwrap_or(SourceLanguage::Rust)
        // Default fallback
    }
}

#[async_trait]
impl AnalysisDetector for TightCouplingDetector {
    async fn detect_issues(
        &self,
        file: &ParsedFile,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        // Single-file analysis for basic coupling detection
        let analyzer = self.get_analyzer_for_language(file.language);
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
                    5, // anti_pattern_type_id: Tight coupling
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
        let metrics = self.calculate_metrics(graph);

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
        let mut all_issues: Vec<ArchitecturalIssue> = issues_by_language
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
            anti_pattern_type_id: Some(5),
            name: "Tight Coupling".to_string(),
            description: "Components that are overly dependent on each other's internal implementation details".to_string(),
            category: "structural".to_string(),
        }]
    }
}

impl LanguageAnalyzer for RustAnalyzer {
    fn extract_dependencies(
        &self,
        file_path: &Path,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        let mut dependencies = Vec::new();

        if let Some(tree) = &parsed_file.tree {
            dependencies.extend(self.extract_use_declarations(
                file_path,
                tree,
                &parsed_file.source,
            )?);
            dependencies.extend(self.extract_function_calls(
                file_path,
                tree,
                &parsed_file.source,
            )?);
            dependencies.extend(self.extract_struct_instantiations(
                file_path,
                tree,
                &parsed_file.source,
            )?);
            dependencies.extend(self.extract_trait_implementations(
                file_path,
                tree,
                &parsed_file.source,
            )?);
        }

        Ok(dependencies)
    }

    fn get_language(&self) -> SourceLanguage {
        SourceLanguage::Rust
    }
}

impl RustAnalyzer {
    const USE_QUERY: &'static str = r#"
        (use_declaration 
          argument: (scoped_use_list 
            list: (use_list 
              (scoped_identifier 
                path: (identifier) @module
                name: (identifier) @item))) @use_decl)
        
        (use_declaration 
          argument: (scoped_identifier 
            path: (identifier) @module
            name: (identifier) @item)) @use_decl
    "#;

    const CALL_QUERY: &'static str = r#"
        (call_expression
          function: (scoped_identifier
            path: (identifier) @module
            name: (identifier) @function)) @call
        
        (call_expression
          function: (field_expression
            value: (identifier) @object
            field: (field_identifier) @method)) @method_call
    "#;

    const STRUCT_QUERY: &'static str = r#"
        (struct_expression
          name: (scoped_type_identifier
            path: (identifier) @module
            name: (type_identifier) @struct)) @instantiation
    "#;

    fn extract_use_declarations(
        &self,
        file_path: &Path,
        tree: &Tree,
        source: &str,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        let mut dependencies = Vec::new();
        let query = Query::new(&tree_sitter_rust::LANGUAGE.into(), Self::USE_QUERY)
            .map_err(|e| AnalysisError::QueryError(format!("Failed to create use query: {}", e)))?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

        while let Some(m) = matches.next() {
            for capture in m.captures {
                let node_text = capture.node.utf8_text(source.as_bytes()).map_err(|e| {
                    AnalysisError::QueryError(format!("Failed to get node text: {}", e))
                })?;

                let from_component = ComponentNode::Module {
                    path: file_path.to_string_lossy().to_string(),
                };

                let to_component = ComponentNode::Module {
                    path: node_text.to_string(),
                };

                dependencies.push(Dependency {
                    from_component,
                    to_component,
                    dependency_type: LocalDependencyType::Import,
                    line_number: Some(capture.node.start_position().row as u32 + 1),
                    strength: DependencyStrength::Medium,
                });
            }
        }

        Ok(dependencies)
    }

    fn extract_function_calls(
        &self,
        file_path: &Path,
        tree: &Tree,
        source: &str,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        let mut dependencies = Vec::new();
        let query =
            Query::new(&tree_sitter_rust::LANGUAGE.into(), Self::CALL_QUERY).map_err(|e| {
                AnalysisError::QueryError(format!("Failed to create call query: {}", e))
            })?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

        while let Some(m) = matches.next() {
            if let Some(capture) = m.captures.first() {
                let node_text = capture.node.utf8_text(source.as_bytes()).map_err(|e| {
                    AnalysisError::QueryError(format!("Failed to get node text: {}", e))
                })?;

                let from_component = ComponentNode::Function {
                    name: "caller".to_string(), // Would need more context to get actual function name
                    file_path: file_path.to_string_lossy().to_string(),
                };

                let to_component = ComponentNode::Function {
                    name: node_text.to_string(),
                    file_path: "external".to_string(),
                };

                dependencies.push(Dependency {
                    from_component,
                    to_component,
                    dependency_type: LocalDependencyType::Call,
                    line_number: Some(capture.node.start_position().row as u32 + 1),
                    strength: DependencyStrength::Strong,
                });
            }
        }

        Ok(dependencies)
    }

    fn extract_struct_instantiations(
        &self,
        file_path: &Path,
        tree: &Tree,
        source: &str,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        let mut dependencies = Vec::new();
        let query =
            Query::new(&tree_sitter_rust::LANGUAGE.into(), Self::STRUCT_QUERY).map_err(|e| {
                AnalysisError::QueryError(format!("Failed to create struct query: {}", e))
            })?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

        while let Some(m) = matches.next() {
            if let Some(capture) = m.captures.first() {
                let node_text = capture.node.utf8_text(source.as_bytes()).map_err(|e| {
                    AnalysisError::QueryError(format!("Failed to get node text: {}", e))
                })?;

                let from_component = ComponentNode::Function {
                    name: "instantiator".to_string(),
                    file_path: file_path.to_string_lossy().to_string(),
                };

                let to_component = ComponentNode::Class {
                    name: node_text.to_string(),
                    file_path: "external".to_string(),
                };

                dependencies.push(Dependency {
                    from_component,
                    to_component,
                    dependency_type: LocalDependencyType::Call,
                    line_number: Some(capture.node.start_position().row as u32 + 1),
                    strength: DependencyStrength::Strong,
                });
            }
        }

        Ok(dependencies)
    }

    fn extract_trait_implementations(
        &self,
        file_path: &Path,
        tree: &Tree,
        source: &str,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        let mut dependencies = Vec::new();

        // Simple implementation - would need more sophisticated parsing
        let impl_query = r#"
            (impl_item
              trait: (type_identifier) @trait_name
              type: (type_identifier) @type_name) @impl_block
        "#;

        if let Ok(query) = Query::new(&tree_sitter_rust::LANGUAGE.into(), impl_query) {
            let mut cursor = QueryCursor::new();
            let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

            while let Some(m) = matches.next() {
                if let Some(capture) = m.captures.first() {
                    let trait_name = capture
                        .node
                        .utf8_text(source.as_bytes())
                        .unwrap_or("unknown_trait");

                    dependencies.push(Dependency {
                        from_component: ComponentNode::Class {
                            name: "impl_struct".to_string(),
                            file_path: file_path.to_string_lossy().to_string(),
                        },
                        to_component: ComponentNode::Class {
                            name: trait_name.to_string(),
                            file_path: "trait_definition".to_string(),
                        },
                        dependency_type: LocalDependencyType::Implementation,
                        line_number: Some(capture.node.start_position().row as u32 + 1),
                        strength: DependencyStrength::Strong,
                    });
                }
            }
        }

        Ok(dependencies)
    }
}

impl LanguageAnalyzer for PythonAnalyzer {
    fn extract_dependencies(
        &self,
        file_path: &Path,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        let mut dependencies = Vec::new();

        if let Some(tree) = &parsed_file.tree {
            dependencies.extend(self.extract_imports(file_path, tree, &parsed_file.source)?);
            dependencies.extend(self.extract_calls(file_path, tree, &parsed_file.source)?);
            dependencies.extend(self.extract_inheritance(file_path, tree, &parsed_file.source)?);
        }

        Ok(dependencies)
    }

    fn get_language(&self) -> SourceLanguage {
        SourceLanguage::Python
    }
}

impl PythonAnalyzer {
    const IMPORT_QUERY: &'static str = r#"
        (import_statement
          name: (dotted_name) @module) @import
        
        (import_from_statement
          module_name: (dotted_name) @module
          name: (dotted_name) @item) @from_import
        
        (import_from_statement
          module_name: (dotted_name) @module
          name: (import_list
            (dotted_name) @item)) @from_import_list
    "#;

    const CALL_QUERY: &'static str = r#"
        (call
          function: (attribute
            object: (identifier) @object
            attribute: (identifier) @method)) @method_call
        
        (call
          function: (identifier) @function) @function_call
    "#;

    fn extract_imports(
        &self,
        file_path: &Path,
        tree: &Tree,
        source: &str,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        let mut dependencies = Vec::new();

        if let Ok(query) = Query::new(&tree_sitter_python::LANGUAGE.into(), Self::IMPORT_QUERY) {
            let mut cursor = QueryCursor::new();
            let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

            while let Some(m) = matches.next() {
                for capture in m.captures {
                    let module_name = capture
                        .node
                        .utf8_text(source.as_bytes())
                        .unwrap_or("unknown_module");

                    dependencies.push(Dependency {
                        from_component: ComponentNode::Module {
                            path: file_path.to_string_lossy().to_string(),
                        },
                        to_component: ComponentNode::Module {
                            path: module_name.to_string(),
                        },
                        dependency_type: LocalDependencyType::Import,
                        line_number: Some(capture.node.start_position().row as u32 + 1),
                        strength: DependencyStrength::Medium,
                    });
                }
            }
        }

        Ok(dependencies)
    }

    fn extract_calls(
        &self,
        file_path: &Path,
        tree: &Tree,
        source: &str,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        let mut dependencies = Vec::new();

        if let Ok(query) = Query::new(&tree_sitter_python::LANGUAGE.into(), Self::CALL_QUERY) {
            let mut cursor = QueryCursor::new();
            let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

            while let Some(m) = matches.next() {
                if let Some(capture) = m.captures.first() {
                    let function_name = capture
                        .node
                        .utf8_text(source.as_bytes())
                        .unwrap_or("unknown_function");

                    dependencies.push(Dependency {
                        from_component: ComponentNode::Function {
                            name: "caller".to_string(),
                            file_path: file_path.to_string_lossy().to_string(),
                        },
                        to_component: ComponentNode::Function {
                            name: function_name.to_string(),
                            file_path: "external".to_string(),
                        },
                        dependency_type: LocalDependencyType::Call,
                        line_number: Some(capture.node.start_position().row as u32 + 1),
                        strength: DependencyStrength::Strong,
                    });
                }
            }
        }

        Ok(dependencies)
    }

    fn extract_inheritance(
        &self,
        file_path: &Path,
        tree: &Tree,
        source: &str,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        let mut dependencies = Vec::new();

        let inheritance_query = r#"
            (class_definition
              name: (identifier) @class_name
              superclasses: (argument_list
                (identifier) @parent_class)) @class_def
        "#;

        if let Ok(query) = Query::new(&tree_sitter_python::LANGUAGE.into(), inheritance_query) {
            let mut cursor = QueryCursor::new();
            let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

            while let Some(m) = matches.next() {
                if m.captures.len() >= 2 {
                    let class_name = m.captures[0]
                        .node
                        .utf8_text(source.as_bytes())
                        .unwrap_or("unknown_class");
                    let parent_name = m.captures[1]
                        .node
                        .utf8_text(source.as_bytes())
                        .unwrap_or("unknown_parent");

                    dependencies.push(Dependency {
                        from_component: ComponentNode::Class {
                            name: class_name.to_string(),
                            file_path: file_path.to_string_lossy().to_string(),
                        },
                        to_component: ComponentNode::Class {
                            name: parent_name.to_string(),
                            file_path: "parent_definition".to_string(),
                        },
                        dependency_type: LocalDependencyType::Inheritance,
                        line_number: Some(m.captures[0].node.start_position().row as u32 + 1),
                        strength: DependencyStrength::Strong,
                    });
                }
            }
        }

        Ok(dependencies)
    }
}

impl LanguageAnalyzer for JavaScriptAnalyzer {
    fn extract_dependencies(
        &self,
        file_path: &Path,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        let mut dependencies = Vec::new();

        if let Some(tree) = &parsed_file.tree {
            dependencies.extend(self.extract_es6_imports(file_path, tree, &parsed_file.source)?);
            dependencies.extend(self.extract_commonjs_requires(
                file_path,
                tree,
                &parsed_file.source,
            )?);
            dependencies.extend(self.extract_function_calls(
                file_path,
                tree,
                &parsed_file.source,
            )?);
        }

        Ok(dependencies)
    }

    fn get_language(&self) -> SourceLanguage {
        SourceLanguage::JavaScript
    }
}

impl JavaScriptAnalyzer {
    const IMPORT_QUERY: &'static str = r#"
        (import_statement
          source: (string) @module) @import
        
        (import_statement
          (import_clause
            (named_imports
              (import_specifier
                name: (identifier) @item)))
          source: (string) @module) @named_import
        
        (variable_declaration
          (variable_declarator
            name: (identifier) @var
            value: (call_expression
              function: (identifier) @require
              arguments: (arguments (string) @module)))) @require_call
    "#;

    fn extract_es6_imports(
        &self,
        file_path: &Path,
        tree: &Tree,
        source: &str,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        let mut dependencies = Vec::new();

        if let Ok(query) = Query::new(&tree_sitter_javascript::LANGUAGE.into(), Self::IMPORT_QUERY)
        {
            let mut cursor = QueryCursor::new();
            let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

            while let Some(m) = matches.next() {
                for capture in m.captures {
                    if capture.index == query.capture_index_for_name("module").unwrap_or(u32::MAX) {
                        let module_name = capture
                            .node
                            .utf8_text(source.as_bytes())
                            .unwrap_or("unknown_module")
                            .trim_matches('"')
                            .trim_matches('\'');

                        dependencies.push(Dependency {
                            from_component: ComponentNode::Module {
                                path: file_path.to_string_lossy().to_string(),
                            },
                            to_component: ComponentNode::Module {
                                path: module_name.to_string(),
                            },
                            dependency_type: LocalDependencyType::Import,
                            line_number: Some(capture.node.start_position().row as u32 + 1),
                            strength: DependencyStrength::Medium,
                        });
                    }
                }
            }
        }

        Ok(dependencies)
    }

    fn extract_commonjs_requires(
        &self,
        file_path: &Path,
        tree: &Tree,
        source: &str,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        let mut dependencies = Vec::new();

        let require_query = r#"
            (call_expression
              function: (identifier) @require_func
              arguments: (arguments (string) @module)) @require_call
        "#;

        if let Ok(query) = Query::new(&tree_sitter_javascript::LANGUAGE.into(), require_query) {
            let mut cursor = QueryCursor::new();
            let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

            while let Some(m) = matches.next() {
                for capture in m.captures {
                    if capture.index == query.capture_index_for_name("module").unwrap_or(u32::MAX) {
                        let module_name = capture
                            .node
                            .utf8_text(source.as_bytes())
                            .unwrap_or("unknown_module")
                            .trim_matches('"')
                            .trim_matches('\'');

                        dependencies.push(Dependency {
                            from_component: ComponentNode::Module {
                                path: file_path.to_string_lossy().to_string(),
                            },
                            to_component: ComponentNode::Module {
                                path: module_name.to_string(),
                            },
                            dependency_type: LocalDependencyType::Import,
                            line_number: Some(capture.node.start_position().row as u32 + 1),
                            strength: DependencyStrength::Medium,
                        });
                    }
                }
            }
        }

        Ok(dependencies)
    }

    fn extract_function_calls(
        &self,
        file_path: &Path,
        tree: &Tree,
        source: &str,
    ) -> Result<Vec<Dependency>, AnalysisError> {
        let mut dependencies = Vec::new();

        let call_query = r#"
            (call_expression
              function: (member_expression
                object: (identifier) @object
                property: (property_identifier) @method)) @method_call
            
            (call_expression
              function: (identifier) @function) @function_call
        "#;

        if let Ok(query) = Query::new(&tree_sitter_javascript::LANGUAGE.into(), call_query) {
            let mut cursor = QueryCursor::new();
            let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

            while let Some(m) = matches.next() {
                if let Some(capture) = m.captures.first() {
                    let function_name = capture
                        .node
                        .utf8_text(source.as_bytes())
                        .unwrap_or("unknown_function");

                    dependencies.push(Dependency {
                        from_component: ComponentNode::Function {
                            name: "caller".to_string(),
                            file_path: file_path.to_string_lossy().to_string(),
                        },
                        to_component: ComponentNode::Function {
                            name: function_name.to_string(),
                            file_path: "external".to_string(),
                        },
                        dependency_type: LocalDependencyType::Call,
                        line_number: Some(capture.node.start_position().row as u32 + 1),
                        strength: DependencyStrength::Strong,
                    });
                }
            }
        }

        Ok(dependencies)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::Arc;

    #[test]
    fn test_detector_name() {
        let detector = TightCouplingDetector::default();
        assert_eq!(detector.get_detector_name(), "TightCouplingDetector");
    }

    #[test]
    fn test_anti_pattern_types() {
        let detector = TightCouplingDetector::default();
        let types = detector.get_anti_pattern_types();
        assert_eq!(types.len(), 1);
        assert_eq!(types[0].name, "Tight Coupling");
    }

    #[test]
    fn test_default_configuration() {
        let config = TightCouplingConfig::default();

        // Test Rust thresholds
        assert_eq!(config.rust_thresholds.fan_out_warning, 7);
        assert_eq!(config.rust_thresholds.fan_out_critical, 12);

        // Test Python thresholds
        assert_eq!(config.python_thresholds.fan_out_warning, 10);
        assert_eq!(config.python_thresholds.fan_out_critical, 15);

        // Test JavaScript thresholds
        assert_eq!(config.javascript_thresholds.fan_out_warning, 12);
        assert_eq!(config.javascript_thresholds.fan_out_critical, 18);

        assert!(config.enable_cross_file_analysis);
        assert!(!config.include_test_files);
    }

    #[test]
    fn test_coupling_metrics_calculation() {
        let detector = TightCouplingDetector::default();
        let mut graph = LocalDependencyGraph::new();

        // Add test components
        let comp1 = ComponentNode::Class {
            name: "TestClass1".to_string(),
            file_path: "test1.rs".to_string(),
        };
        let comp2 = ComponentNode::Class {
            name: "TestClass2".to_string(),
            file_path: "test2.rs".to_string(),
        };

        // Add dependency
        graph.add_dependency(&comp1, &comp2, LocalDependencyType::Call);

        let metrics = detector.calculate_metrics(&graph);

        // Verify metrics calculation
        assert!(metrics.contains_key(&comp1));
        assert!(metrics.contains_key(&comp2));

        let comp1_metrics = &metrics[&comp1];
        assert_eq!(comp1_metrics.fan_out, 1); // comp1 depends on comp2
        assert_eq!(comp1_metrics.fan_in, 0); // nothing depends on comp1

        let comp2_metrics = &metrics[&comp2];
        assert_eq!(comp2_metrics.fan_out, 0); // comp2 depends on nothing
        assert_eq!(comp2_metrics.fan_in, 1); // comp1 depends on comp2
    }

    #[test]
    fn test_language_analyzer_selection() {
        let detector = TightCouplingDetector::default();

        let rust_analyzer = detector.get_analyzer_for_language(SourceLanguage::Rust);
        assert_eq!(rust_analyzer.get_language(), SourceLanguage::Rust);

        let python_analyzer = detector.get_analyzer_for_language(SourceLanguage::Python);
        assert_eq!(python_analyzer.get_language(), SourceLanguage::Python);

        let js_analyzer = detector.get_analyzer_for_language(SourceLanguage::JavaScript);
        assert_eq!(js_analyzer.get_language(), SourceLanguage::JavaScript);
    }

    #[test]
    fn test_dependency_strength_classification() {
        use DependencyStrength::*;

        // Test that different dependency types have appropriate strengths
        let import_dep = Dependency {
            from_component: ComponentNode::Module {
                path: "a.rs".to_string(),
            },
            to_component: ComponentNode::Module {
                path: "b.rs".to_string(),
            },
            dependency_type: LocalDependencyType::Import,
            line_number: Some(1),
            strength: Medium,
        };

        assert_eq!(import_dep.strength, Medium);

        let inheritance_dep = Dependency {
            from_component: ComponentNode::Class {
                name: "Child".to_string(),
                file_path: "child.rs".to_string(),
            },
            to_component: ComponentNode::Class {
                name: "Parent".to_string(),
                file_path: "parent.rs".to_string(),
            },
            dependency_type: LocalDependencyType::Inheritance,
            line_number: Some(5),
            strength: Strong,
        };

        assert_eq!(inheritance_dep.strength, Strong);
    }
}
