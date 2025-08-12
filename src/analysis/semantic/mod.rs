//! Semantic feature extraction and analysis
//!
//! This module provides advanced semantic analysis capabilities for Type-4 clone detection.
//! It extracts meaningful semantic features from code without relying on heavy ML frameworks,
//! using pure Rust implementations for high performance.
//!
//! ## Core Components:
//! - **SemanticFeatures**: Extracted semantic patterns and metrics
//! - **SemanticAnalyzer**: Main analyzer for feature extraction
//! - **WeisfeilerLehmanKernel**: Graph kernel for CFG similarity
//! - **HybridSimilarityScorer**: Multi-dimensional similarity scoring

use crate::analysis::cfg::{CfgEdge, CfgNode, CfgNodeType, ControlFlowGraph};
use crate::analysis::errors::AnalysisError;
use crate::ast::tree_sitter::Node;
use crate::ast::tree_sitter_impl::SourceLanguage;
use crate::core::logging::{debug, warn};
use ndarray::Array1;
use petgraph::{Direction, Graph};
use rayon::prelude::*;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};

/// Semantic features extracted from code without ML
#[derive(Debug, Clone)]
pub struct SemanticFeatures {
    /// Control flow complexity metric
    pub control_flow_complexity: f64,
    /// Cyclomatic complexity of the code
    pub cyclomatic_complexity: u32,
    /// Identified data flow patterns
    pub data_flow_patterns: Vec<DataFlowPattern>,
    /// API usage patterns found in the code
    pub api_usage_patterns: Vec<ApiPattern>,
    /// Structural complexity metrics
    pub structural_metrics: StructuralMetrics,
    /// Variable usage and lifecycle patterns
    pub variable_usage_patterns: Vec<VariablePattern>,
    /// Semantic feature vector for similarity calculations
    pub feature_vector: Array1<f64>,
}

/// Represents a data flow pattern in the code
#[derive(Debug, Clone)]
pub struct DataFlowPattern {
    /// The type of data flow pattern
    pub pattern_type: DataFlowType,
    /// Variables involved in the pattern
    pub variables: Vec<String>,
    /// Operations performed in the pattern
    pub operations: Vec<String>,
    /// Confidence score for pattern detection
    pub confidence: f64,
}

/// Types of data flow patterns
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DataFlowType {
    /// Simple variable assignment
    Assignment,
    /// Data transformation operations
    Transformation,
    /// Data aggregation (sum, collect, etc.)
    Aggregation,
    /// Filtering operations
    Filtering,
    /// Iteration patterns
    Iteration,
    /// Pipeline operations
    Pipeline,
}

/// API usage pattern
#[derive(Debug, Clone)]
pub struct ApiPattern {
    /// API function or method name
    pub api_name: String,
    /// Usage frequency in the code block
    pub usage_count: u32,
    /// Parameters patterns
    pub parameter_patterns: Vec<String>,
    /// Return type pattern
    pub return_pattern: Option<String>,
}

/// Structural complexity metrics
#[derive(Debug, Clone)]
pub struct StructuralMetrics {
    /// Number of decision points
    pub decision_points: u32,
    /// Maximum nesting depth
    pub max_nesting_depth: u32,
    /// Number of function calls
    pub function_calls: u32,
    /// Number of loops
    pub loop_count: u32,
    /// Number of conditional statements
    pub conditional_count: u32,
    /// Lines of code
    pub lines_of_code: u32,
}

/// Variable usage pattern
#[derive(Debug, Clone)]
pub struct VariablePattern {
    /// Variable name (normalized)
    pub variable_name: String,
    /// Usage type (read, write, modify)
    pub usage_type: VariableUsageType,
    /// Scope level
    pub scope_level: u32,
    /// Lifecycle stage
    pub lifecycle_stage: VariableLifecycleStage,
}

/// Types of variable usage
#[derive(Debug, Clone, PartialEq)]
pub enum VariableUsageType {
    /// Variable is read
    Read,
    /// Variable is written to
    Write,
    /// Variable is modified in place
    Modify,
    /// Variable is passed as parameter
    Parameter,
    /// Variable is returned
    Return,
}

/// Variable lifecycle stages
#[derive(Debug, Clone, PartialEq)]
pub enum VariableLifecycleStage {
    /// Variable declaration
    Declaration,
    /// Variable initialization
    Initialization,
    /// Variable usage
    Usage,
    /// Variable modification
    Modification,
    /// Variable cleanup/disposal
    Cleanup,
}

/// Main semantic analyzer for feature extraction
pub struct SemanticAnalyzer {
    /// Language-specific analysis configuration
    language_config: LanguageConfig,
    /// Feature extraction configuration
    feature_config: FeatureConfig,
}

/// Language-specific configuration for semantic analysis
#[derive(Debug, Clone)]
struct LanguageConfig {
    /// Programming language
    language: SourceLanguage,
    /// Common API patterns for this language
    common_apis: Vec<String>,
    /// Language-specific keywords
    keywords: Vec<String>,
}

/// Configuration for feature extraction
#[derive(Debug, Clone)]
struct FeatureConfig {
    /// Maximum depth for nested structure analysis
    max_analysis_depth: u32,
    /// Minimum confidence threshold for pattern detection
    min_confidence_threshold: f64,
    /// Feature vector dimension
    feature_vector_size: usize,
}

impl Default for FeatureConfig {
    fn default() -> Self {
        Self {
            max_analysis_depth: 10,
            min_confidence_threshold: 0.7,
            feature_vector_size: 128,
        }
    }
}

impl SemanticAnalyzer {
    /// Creates a new semantic analyzer for the given language
    pub fn new(language: SourceLanguage) -> Self {
        let language_config = LanguageConfig {
            language: language.clone(),
            common_apis: get_common_apis_for_language(&language),
            keywords: get_keywords_for_language(&language),
        };

        let feature_config = FeatureConfig {
            max_analysis_depth: 10,
            min_confidence_threshold: 0.7,
            feature_vector_size: 64,
        };

        Self {
            language_config,
            feature_config,
        }
    }

    /// Extracts semantic features from CFG and AST
    pub fn extract_features(
        &self,
        cfg: &ControlFlowGraph,
        ast: &Node,
        source: &str,
    ) -> Result<SemanticFeatures, AnalysisError> {
        debug!(
            "Extracting semantic features from CFG with {} nodes",
            cfg.node_count()
        );

        // Extract different types of features in parallel
        let (control_flow_complexity, other_features) = rayon::join(
            || self.analyze_control_flow_complexity(cfg),
            || {
                let (cyclomatic_complexity, remaining_features) = rayon::join(
                    || self.compute_cyclomatic_complexity(cfg),
                    || {
                        let (data_flow_patterns, more_features) = rayon::join(
                            || self.extract_data_flow_patterns(ast, source),
                            || {
                                let (api_usage_patterns, final_features) = rayon::join(
                                    || self.extract_api_usage_patterns(ast, source),
                                    || {
                                        let (structural_metrics, variable_usage_patterns) =
                                            rayon::join(
                                                || self.compute_structural_metrics(cfg, ast),
                                                || {
                                                    self.extract_variable_usage_patterns(
                                                        ast, source,
                                                    )
                                                },
                                            );
                                        (structural_metrics, variable_usage_patterns)
                                    },
                                );
                                (api_usage_patterns, final_features)
                            },
                        );
                        (data_flow_patterns, more_features)
                    },
                );
                (cyclomatic_complexity, remaining_features)
            },
        );

        let (
            cyclomatic_complexity,
            (
                data_flow_patterns,
                (api_usage_patterns, (structural_metrics, variable_usage_patterns)),
            ),
        ) = other_features;

        // Compute feature vector from extracted features
        let feature_vector = self.compute_feature_vector(
            control_flow_complexity,
            cyclomatic_complexity,
            &data_flow_patterns,
            &api_usage_patterns,
            &structural_metrics,
            &variable_usage_patterns,
        );

        Ok(SemanticFeatures {
            control_flow_complexity,
            cyclomatic_complexity,
            data_flow_patterns: data_flow_patterns.to_vec(),
            api_usage_patterns: api_usage_patterns.to_vec(),
            structural_metrics,
            variable_usage_patterns: variable_usage_patterns.to_vec(),
            feature_vector,
        })
    }

    /// Analyzes control flow complexity
    fn analyze_control_flow_complexity(&self, cfg: &ControlFlowGraph) -> f64 {
        let node_count = cfg.node_count() as f64;
        let edge_count = cfg.edge_count() as f64;

        if node_count == 0.0 {
            return 0.0;
        }

        // Calculate complexity based on graph structure
        let basic_complexity = edge_count / node_count;

        // Add complexity for different node types
        let mut type_complexity = 0.0;
        for node in cfg.graph().node_weights() {
            match node.node_type {
                CfgNodeType::Loop => type_complexity += 2.0,
                CfgNodeType::Condition => type_complexity += 1.5,
                CfgNodeType::Exception => type_complexity += 1.5,
                CfgNodeType::FunctionCall => type_complexity += 0.5,
                _ => type_complexity += 0.1,
            }
        }

        basic_complexity + (type_complexity / node_count)
    }

    /// Computes cyclomatic complexity
    fn compute_cyclomatic_complexity(&self, cfg: &ControlFlowGraph) -> u32 {
        let edge_count = cfg.edge_count() as u32;
        let node_count = cfg.node_count() as u32;

        // Cyclomatic complexity = E - N + 2P
        // where E = edges, N = nodes, P = connected components (assume 1)
        if node_count == 0 {
            return 0;
        }

        let connected_components = 1u32;
        edge_count
            .saturating_sub(node_count)
            .saturating_add(2 * connected_components)
    }

    /// Extracts data flow patterns from AST
    fn extract_data_flow_patterns(&self, ast: &Node, source: &str) -> Vec<DataFlowPattern> {
        let mut patterns = Vec::new();

        // Extract assignment patterns
        patterns.extend(self.extract_assignment_patterns(ast, source));

        // Extract transformation patterns
        patterns.extend(self.extract_transformation_patterns(ast, source));

        // Extract aggregation patterns
        patterns.extend(self.extract_aggregation_patterns(ast, source));

        // Extract filtering patterns
        patterns.extend(self.extract_filtering_patterns(ast, source));

        // Extract iteration patterns
        patterns.extend(self.extract_iteration_patterns(ast, source));

        patterns
    }

    /// Extracts assignment patterns
    fn extract_assignment_patterns(&self, ast: &Node, source: &str) -> Vec<DataFlowPattern> {
        let mut patterns = Vec::new();

        // Simple assignment detection
        if let Ok(text) = ast.utf8_text(source.as_bytes()) {
            if text.contains("=") && !text.contains("==") && !text.contains("!=") {
                patterns.push(DataFlowPattern {
                    pattern_type: DataFlowType::Assignment,
                    variables: vec!["_var_".to_string()],
                    operations: vec!["assign".to_string()],
                    confidence: 0.8,
                });
            }
        }

        patterns
    }

    /// Extracts transformation patterns
    fn extract_transformation_patterns(&self, ast: &Node, source: &str) -> Vec<DataFlowPattern> {
        let mut patterns = Vec::new();

        // Look for transformation operations
        if let Ok(text) = ast.utf8_text(source.as_bytes()) {
            let transformation_keywords = ["map", "transform", "convert", "parse", "format"];

            for keyword in &transformation_keywords {
                if text.contains(keyword) {
                    patterns.push(DataFlowPattern {
                        pattern_type: DataFlowType::Transformation,
                        variables: vec!["_data_".to_string()],
                        operations: vec![keyword.to_string()],
                        confidence: 0.7,
                    });
                }
            }
        }

        patterns
    }

    /// Extracts aggregation patterns
    fn extract_aggregation_patterns(&self, ast: &Node, source: &str) -> Vec<DataFlowPattern> {
        let mut patterns = Vec::new();

        // Look for aggregation operations
        if let Ok(text) = ast.utf8_text(source.as_bytes()) {
            let aggregation_keywords = ["sum", "collect", "fold", "reduce", "count", "max", "min"];

            for keyword in &aggregation_keywords {
                if text.contains(keyword) {
                    patterns.push(DataFlowPattern {
                        pattern_type: DataFlowType::Aggregation,
                        variables: vec!["_collection_".to_string()],
                        operations: vec![keyword.to_string()],
                        confidence: 0.75,
                    });
                }
            }
        }

        patterns
    }

    /// Extracts filtering patterns
    fn extract_filtering_patterns(&self, ast: &Node, source: &str) -> Vec<DataFlowPattern> {
        let mut patterns = Vec::new();

        // Look for filtering operations
        if let Ok(text) = ast.utf8_text(source.as_bytes()) {
            let filtering_keywords = ["filter", "where", "select", "find", "search"];

            for keyword in &filtering_keywords {
                if text.contains(keyword) {
                    patterns.push(DataFlowPattern {
                        pattern_type: DataFlowType::Filtering,
                        variables: vec!["_items_".to_string()],
                        operations: vec![keyword.to_string()],
                        confidence: 0.8,
                    });
                }
            }
        }

        patterns
    }

    /// Extracts iteration patterns
    fn extract_iteration_patterns(&self, ast: &Node, source: &str) -> Vec<DataFlowPattern> {
        let mut patterns = Vec::new();

        // Look for iteration patterns
        if let Ok(text) = ast.utf8_text(source.as_bytes()) {
            let iteration_keywords = ["for", "while", "loop", "iter", "each"];

            for keyword in &iteration_keywords {
                if text.contains(keyword) {
                    patterns.push(DataFlowPattern {
                        pattern_type: DataFlowType::Iteration,
                        variables: vec!["_iterator_".to_string()],
                        operations: vec![keyword.to_string()],
                        confidence: 0.85,
                    });
                }
            }
        }

        patterns
    }

    /// Extracts API usage patterns
    fn extract_api_usage_patterns(&self, ast: &Node, source: &str) -> Vec<ApiPattern> {
        let mut patterns = Vec::new();

        // Count API usage
        let mut api_counts = HashMap::new();

        if let Ok(text) = ast.utf8_text(source.as_bytes()) {
            for api in &self.language_config.common_apis {
                let count = text.matches(api).count() as u32;
                if count > 0 {
                    api_counts.insert(api.clone(), count);
                }
            }
        }

        // Convert to patterns
        for (api_name, count) in api_counts {
            patterns.push(ApiPattern {
                api_name,
                usage_count: count,
                parameter_patterns: vec!["_param_".to_string()],
                return_pattern: Some("_return_".to_string()),
            });
        }

        patterns
    }

    /// Computes structural metrics
    fn compute_structural_metrics(&self, cfg: &ControlFlowGraph, ast: &Node) -> StructuralMetrics {
        let mut decision_points = 0;
        let mut loop_count = 0;
        let mut conditional_count = 0;
        let mut function_calls = 0;

        // Count node types in CFG
        for node in cfg.graph().node_weights() {
            match node.node_type {
                CfgNodeType::Condition => {
                    decision_points += 1;
                    conditional_count += 1;
                }
                CfgNodeType::Loop => {
                    decision_points += 1;
                    loop_count += 1;
                }
                CfgNodeType::FunctionCall => function_calls += 1,
                _ => {}
            }
        }

        let lines_of_code = ast.end_position().row as u32 - ast.start_position().row as u32 + 1;
        let max_nesting_depth = self.calculate_max_nesting_depth(ast);

        StructuralMetrics {
            decision_points,
            max_nesting_depth,
            function_calls,
            loop_count,
            conditional_count,
            lines_of_code,
        }
    }

    /// Calculates maximum nesting depth
    fn calculate_max_nesting_depth(&self, ast: &Node) -> u32 {
        let mut max_depth = 0;
        let mut current_depth = 0;

        self.traverse_for_depth(ast, &mut current_depth, &mut max_depth);

        max_depth
    }

    /// Traverses AST to calculate nesting depth
    fn traverse_for_depth(&self, node: &Node, current_depth: &mut u32, max_depth: &mut u32) {
        let node_kind = node.kind();

        // Increase depth for nesting constructs
        let increases_depth = matches!(
            node_kind,
            "if_expression"
                | "if_statement"
                | "for_statement"
                | "for_expression"
                | "while_statement"
                | "while_expression"
                | "loop_expression"
                | "block"
                | "function_item"
                | "function_declaration"
                | "function_definition"
        );

        if increases_depth {
            *current_depth += 1;
            *max_depth = (*max_depth).max(*current_depth);
        }

        // Traverse children
        let mut cursor = node.walk();
        if cursor.goto_first_child() {
            loop {
                self.traverse_for_depth(&cursor.node(), current_depth, max_depth);
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }

        if increases_depth {
            *current_depth -= 1;
        }
    }

    /// Extracts variable usage patterns
    fn extract_variable_usage_patterns(&self, ast: &Node, source: &str) -> Vec<VariablePattern> {
        let mut patterns = Vec::new();

        // Simple variable pattern detection
        if let Ok(text) = ast.utf8_text(source.as_bytes()) {
            // Look for common variable patterns
            let variable_keywords = ["let", "var", "const", "mut"];

            for keyword in &variable_keywords {
                if text.contains(keyword) {
                    patterns.push(VariablePattern {
                        variable_name: "_var_".to_string(),
                        usage_type: VariableUsageType::Write,
                        scope_level: 1,
                        lifecycle_stage: VariableLifecycleStage::Declaration,
                    });
                }
            }
        }

        patterns
    }

    /// Computes feature vector from extracted features
    fn compute_feature_vector(
        &self,
        control_flow_complexity: f64,
        cyclomatic_complexity: u32,
        data_flow_patterns: &[DataFlowPattern],
        api_usage_patterns: &[ApiPattern],
        structural_metrics: &StructuralMetrics,
        variable_usage_patterns: &[VariablePattern],
    ) -> Array1<f64> {
        let mut features = vec![0.0; self.feature_config.feature_vector_size];

        // Basic complexity features
        features[0] = control_flow_complexity;
        features[1] = cyclomatic_complexity as f64;

        // Structural features
        features[2] = structural_metrics.decision_points as f64;
        features[3] = structural_metrics.max_nesting_depth as f64;
        features[4] = structural_metrics.function_calls as f64;
        features[5] = structural_metrics.loop_count as f64;
        features[6] = structural_metrics.conditional_count as f64;
        features[7] = structural_metrics.lines_of_code as f64;

        // Data flow pattern features
        let mut pattern_counts = HashMap::new();
        for pattern in data_flow_patterns {
            *pattern_counts
                .entry(pattern.pattern_type.clone())
                .or_insert(0.0) += pattern.confidence;
        }

        features[8] = *pattern_counts
            .get(&DataFlowType::Assignment)
            .unwrap_or(&0.0);
        features[9] = *pattern_counts
            .get(&DataFlowType::Transformation)
            .unwrap_or(&0.0);
        features[10] = *pattern_counts
            .get(&DataFlowType::Aggregation)
            .unwrap_or(&0.0);
        features[11] = *pattern_counts.get(&DataFlowType::Filtering).unwrap_or(&0.0);
        features[12] = *pattern_counts.get(&DataFlowType::Iteration).unwrap_or(&0.0);

        // API usage features
        features[13] = api_usage_patterns.len() as f64;
        features[14] = api_usage_patterns
            .iter()
            .map(|p| p.usage_count as f64)
            .sum::<f64>();

        // Variable usage features
        features[15] = variable_usage_patterns.len() as f64;

        // Normalize features
        self.normalize_feature_vector(&mut features);

        Array1::from_vec(features)
    }

    /// Normalizes feature vector values
    fn normalize_feature_vector(&self, features: &mut [f64]) {
        let max_val = features.iter().fold(0.0f64, |acc, &x| acc.max(x));
        if max_val > 0.0 {
            for feature in features.iter_mut() {
                *feature /= max_val;
            }
        }
    }
}

/// Returns common APIs for the given language
fn get_common_apis_for_language(language: &SourceLanguage) -> Vec<String> {
    match language {
        SourceLanguage::Rust => vec![
            "println!".to_string(),
            "vec!".to_string(),
            "String::new".to_string(),
            "HashMap::new".to_string(),
            "Vec::new".to_string(),
            "collect".to_string(),
            "iter".to_string(),
            "map".to_string(),
            "filter".to_string(),
            "fold".to_string(),
            "unwrap".to_string(),
            "expect".to_string(),
            "match".to_string(),
        ],
        SourceLanguage::Python => vec![
            "print".to_string(),
            "len".to_string(),
            "range".to_string(),
            "list".to_string(),
            "dict".to_string(),
            "str".to_string(),
            "int".to_string(),
            "float".to_string(),
            "map".to_string(),
            "filter".to_string(),
            "reduce".to_string(),
            "enumerate".to_string(),
            "zip".to_string(),
        ],
        SourceLanguage::JavaScript => vec![
            "console.log".to_string(),
            "document.getElementById".to_string(),
            "Array.from".to_string(),
            "Object.keys".to_string(),
            "JSON.parse".to_string(),
            "JSON.stringify".to_string(),
            "map".to_string(),
            "filter".to_string(),
            "reduce".to_string(),
            "forEach".to_string(),
            "addEventListener".to_string(),
            "setTimeout".to_string(),
            "Promise".to_string(),
        ],
        SourceLanguage::TypeScript => vec![
            // Reuse JS common APIs for placeholder
            "console.log".to_string(),
            "Array.from".to_string(),
            "Object.keys".to_string(),
            "JSON.parse".to_string(),
            "JSON.stringify".to_string(),
            "map".to_string(),
            "filter".to_string(),
            "reduce".to_string(),
            "forEach".to_string(),
            "Promise".to_string(),
        ],
    }
}

/// Returns keywords for the given language
fn get_keywords_for_language(language: &SourceLanguage) -> Vec<String> {
    match language {
        SourceLanguage::Rust => vec![
            "fn".to_string(),
            "let".to_string(),
            "mut".to_string(),
            "if".to_string(),
            "else".to_string(),
            "for".to_string(),
            "while".to_string(),
            "loop".to_string(),
            "match".to_string(),
            "return".to_string(),
            "struct".to_string(),
            "enum".to_string(),
            "impl".to_string(),
            "trait".to_string(),
        ],
        SourceLanguage::Python => vec![
            "def".to_string(),
            "class".to_string(),
            "if".to_string(),
            "else".to_string(),
            "elif".to_string(),
            "for".to_string(),
            "while".to_string(),
            "return".to_string(),
            "yield".to_string(),
            "with".to_string(),
            "try".to_string(),
            "except".to_string(),
            "finally".to_string(),
            "import".to_string(),
            "from".to_string(),
        ],
        SourceLanguage::JavaScript => vec![
            "function".to_string(),
            "var".to_string(),
            "let".to_string(),
            "const".to_string(),
            "if".to_string(),
            "else".to_string(),
            "for".to_string(),
            "while".to_string(),
            "return".to_string(),
            "class".to_string(),
            "extends".to_string(),
            "import".to_string(),
            "export".to_string(),
            "async".to_string(),
            "await".to_string(),
        ],
        SourceLanguage::TypeScript => vec![
            // Reuse JS keywords plus TS specific placeholders
            "function".to_string(),
            "let".to_string(),
            "const".to_string(),
            "class".to_string(),
            "interface".to_string(),
            "type".to_string(),
            "enum".to_string(),
            "implements".to_string(),
            "extends".to_string(),
            "import".to_string(),
            "export".to_string(),
            "from".to_string(),
            "return".to_string(),
            "if".to_string(),
            "else".to_string(),
            "for".to_string(),
            "while".to_string(),
        ],
    }
}

/// Weisfeiler-Lehman kernel for CFG similarity computation
pub struct WeisfeilerLehmanKernel {
    /// Number of iterations for the WL algorithm
    iterations: usize,
    /// Hash function for node label updates
    hasher_seed: u64,
}

impl WeisfeilerLehmanKernel {
    /// Creates a new WL kernel with specified iterations
    pub fn new(iterations: usize) -> Self {
        Self {
            iterations,
            hasher_seed: 0,
        }
    }

    /// Creates a new WL kernel with custom hash seed
    pub fn with_seed(iterations: usize, seed: u64) -> Self {
        Self {
            iterations,
            hasher_seed: seed,
        }
    }

    /// Computes similarity between two CFGs using WL kernel
    pub fn compute_similarity(
        &self,
        cfg1: &Graph<CfgNode, CfgEdge>,
        cfg2: &Graph<CfgNode, CfgEdge>,
    ) -> f64 {
        debug!(
            "Computing WL similarity between CFGs with {} and {} nodes",
            cfg1.node_count(),
            cfg2.node_count()
        );

        // Handle empty graphs
        if cfg1.node_count() == 0 && cfg2.node_count() == 0 {
            return 1.0;
        }
        if cfg1.node_count() == 0 || cfg2.node_count() == 0 {
            return 0.0;
        }

        // Compute WL histograms for both graphs
        let histogram1 = self.compute_wl_histogram(cfg1);
        let histogram2 = self.compute_wl_histogram(cfg2);

        // Calculate similarity using histogram intersection
        self.compute_histogram_similarity(&histogram1, &histogram2)
    }

    /// Computes WL histogram for a single CFG
    fn compute_wl_histogram(&self, cfg: &Graph<CfgNode, CfgEdge>) -> HashMap<String, u32> {
        let mut node_labels = HashMap::new();
        let mut histogram = HashMap::new();

        // Initialize node labels with node types
        for node_index in cfg.node_indices() {
            if let Some(node) = cfg.node_weight(node_index) {
                let initial_label = format!("{:?}", node.node_type);
                node_labels.insert(node_index, initial_label);
            }
        }

        // Perform WL iterations
        for iteration in 0..self.iterations {
            let mut new_labels = HashMap::new();

            for node_index in cfg.node_indices() {
                // Get current node label
                let current_label = node_labels.get(&node_index).cloned().unwrap_or_default();

                // Collect neighbor labels
                let mut neighbor_labels = Vec::new();
                for neighbor_index in cfg.neighbors_directed(node_index, Direction::Outgoing) {
                    if let Some(neighbor_label) = node_labels.get(&neighbor_index) {
                        neighbor_labels.push(neighbor_label.clone());
                    }
                }
                for neighbor_index in cfg.neighbors_directed(node_index, Direction::Incoming) {
                    if let Some(neighbor_label) = node_labels.get(&neighbor_index) {
                        neighbor_labels.push(format!("rev_{}", neighbor_label));
                    }
                }

                // Sort neighbor labels for consistency
                neighbor_labels.sort();

                // Create new label by hashing current label + neighbor labels
                let new_label = self.hash_label_with_neighbors(&current_label, &neighbor_labels);
                new_labels.insert(node_index, new_label);
            }

            // Update labels for next iteration
            node_labels = new_labels;

            debug!("WL iteration {} completed", iteration);
        }

        // Build histogram from final labels
        for (_, label) in node_labels {
            *histogram.entry(label).or_insert(0) += 1;
        }

        histogram
    }

    /// Hashes a node label with its neighbors
    fn hash_label_with_neighbors(&self, current_label: &str, neighbor_labels: &[String]) -> String {
        let mut hasher = Sha256::new();

        // Add seed for reproducibility
        hasher.update(self.hasher_seed.to_be_bytes());

        // Add current label
        hasher.update(current_label.as_bytes());

        // Add neighbor labels
        for neighbor_label in neighbor_labels {
            hasher.update(neighbor_label.as_bytes());
        }

        format!("{:x}", hasher.finalize())
    }

    /// Computes similarity between two histograms using intersection
    fn compute_histogram_similarity(
        &self,
        hist1: &HashMap<String, u32>,
        hist2: &HashMap<String, u32>,
    ) -> f64 {
        let mut intersection = 0u32;
        let mut union = 0u32;

        // Get all unique labels
        let all_labels: HashSet<_> = hist1.keys().chain(hist2.keys()).collect();

        for label in all_labels {
            let count1 = hist1.get(label).copied().unwrap_or(0);
            let count2 = hist2.get(label).copied().unwrap_or(0);

            intersection += count1.min(count2);
            union += count1.max(count2);
        }

        if union == 0 {
            1.0
        } else {
            intersection as f64 / union as f64
        }
    }
}

impl Default for WeisfeilerLehmanKernel {
    fn default() -> Self {
        Self::new(3)
    }
}

/// Hybrid similarity scorer combining multiple approaches
pub struct HybridSimilarityScorer {
    /// WL kernel for CFG similarity
    wl_kernel: WeisfeilerLehmanKernel,
    /// Semantic analyzer for feature extraction
    semantic_analyzer: SemanticAnalyzer,
    /// Weights for different similarity components
    weights: SimilarityWeights,
}

/// Weights for different similarity components
#[derive(Debug, Clone)]
pub struct SimilarityWeights {
    /// Weight for structural similarity
    pub structural: f64,
    /// Weight for CFG similarity
    pub cfg: f64,
    /// Weight for semantic feature similarity
    pub semantic: f64,
    /// Weight for AST pattern similarity
    pub ast_pattern: f64,
}

impl Default for SimilarityWeights {
    fn default() -> Self {
        Self {
            structural: 0.3,
            cfg: 0.3,
            semantic: 0.25,
            ast_pattern: 0.15,
        }
    }
}

impl HybridSimilarityScorer {
    /// Creates a new hybrid similarity scorer
    pub fn new(language: SourceLanguage) -> Self {
        Self {
            wl_kernel: WeisfeilerLehmanKernel::new(3),
            semantic_analyzer: SemanticAnalyzer::new(language),
            weights: SimilarityWeights::default(),
        }
    }

    /// Creates a scorer with custom weights
    pub fn with_weights(language: SourceLanguage, weights: SimilarityWeights) -> Self {
        Self {
            wl_kernel: WeisfeilerLehmanKernel::new(3),
            semantic_analyzer: SemanticAnalyzer::new(language),
            weights,
        }
    }

    /// Computes semantic similarity between two code blocks
    pub fn compute_semantic_similarity(
        &self,
        cfg1: &ControlFlowGraph,
        cfg2: &ControlFlowGraph,
        features1: &SemanticFeatures,
        features2: &SemanticFeatures,
    ) -> f64 {
        // Compute individual similarity components
        let structural_score = self.compute_structural_similarity(features1, features2);
        let cfg_score = self
            .wl_kernel
            .compute_similarity(cfg1.graph(), cfg2.graph());
        let semantic_score = self.compute_feature_similarity(features1, features2);
        let ast_score = self.compute_ast_pattern_similarity(features1, features2);

        // Weighted combination for Type-4 detection
        self.weights.structural * structural_score
            + self.weights.cfg * cfg_score
            + self.weights.semantic * semantic_score
            + self.weights.ast_pattern * ast_score
    }

    /// Computes structural similarity between two feature sets
    fn compute_structural_similarity(
        &self,
        features1: &SemanticFeatures,
        features2: &SemanticFeatures,
    ) -> f64 {
        let metrics1 = &features1.structural_metrics;
        let metrics2 = &features2.structural_metrics;

        // Compute normalized differences
        let complexity_diff = self.normalize_difference(
            features1.control_flow_complexity,
            features2.control_flow_complexity,
            10.0, // max expected complexity
        );

        let cyclomatic_diff = self.normalize_difference(
            features1.cyclomatic_complexity as f64,
            features2.cyclomatic_complexity as f64,
            50.0, // max expected cyclomatic complexity
        );

        let decision_diff = self.normalize_difference(
            metrics1.decision_points as f64,
            metrics2.decision_points as f64,
            20.0, // max expected decision points
        );

        let nesting_diff = self.normalize_difference(
            metrics1.max_nesting_depth as f64,
            metrics2.max_nesting_depth as f64,
            10.0, // max expected nesting depth
        );

        // Average similarity
        (complexity_diff + cyclomatic_diff + decision_diff + nesting_diff) / 4.0
    }

    /// Computes feature similarity using cosine similarity
    fn compute_feature_similarity(
        &self,
        features1: &SemanticFeatures,
        features2: &SemanticFeatures,
    ) -> f64 {
        let vec1 = &features1.feature_vector;
        let vec2 = &features2.feature_vector;

        // Compute cosine similarity
        let dot_product = vec1.dot(vec2);
        let norm1 = vec1.dot(vec1).sqrt();
        let norm2 = vec2.dot(vec2).sqrt();

        if norm1 == 0.0 || norm2 == 0.0 {
            0.0
        } else {
            dot_product / (norm1 * norm2)
        }
    }

    /// Computes AST pattern similarity
    fn compute_ast_pattern_similarity(
        &self,
        features1: &SemanticFeatures,
        features2: &SemanticFeatures,
    ) -> f64 {
        // Compare data flow patterns
        let pattern_similarity = self.compute_pattern_similarity(
            &features1.data_flow_patterns,
            &features2.data_flow_patterns,
        );

        // Compare API usage patterns
        let api_similarity = self
            .compute_api_similarity(&features1.api_usage_patterns, &features2.api_usage_patterns);

        // Average the similarities
        (pattern_similarity + api_similarity) / 2.0
    }

    /// Computes similarity between data flow patterns
    fn compute_pattern_similarity(
        &self,
        patterns1: &[DataFlowPattern],
        patterns2: &[DataFlowPattern],
    ) -> f64 {
        if patterns1.is_empty() && patterns2.is_empty() {
            return 1.0;
        }
        if patterns1.is_empty() || patterns2.is_empty() {
            return 0.0;
        }

        // Count patterns by type
        let mut counts1 = HashMap::new();
        let mut counts2 = HashMap::new();

        for pattern in patterns1 {
            *counts1.entry(pattern.pattern_type.clone()).or_insert(0.0) += pattern.confidence;
        }

        for pattern in patterns2 {
            *counts2.entry(pattern.pattern_type.clone()).or_insert(0.0) += pattern.confidence;
        }

        // Compute Jaccard similarity
        let all_types: HashSet<_> = counts1.keys().chain(counts2.keys()).collect();
        let mut intersection = 0.0;
        let mut union = 0.0;

        for pattern_type in all_types {
            let count1 = counts1.get(pattern_type).copied().unwrap_or(0.0);
            let count2 = counts2.get(pattern_type).copied().unwrap_or(0.0);

            intersection += count1.min(count2);
            union += count1.max(count2);
        }

        if union == 0.0 {
            1.0
        } else {
            intersection / union
        }
    }

    /// Computes similarity between API usage patterns
    fn compute_api_similarity(&self, apis1: &[ApiPattern], apis2: &[ApiPattern]) -> f64 {
        if apis1.is_empty() && apis2.is_empty() {
            return 1.0;
        }
        if apis1.is_empty() || apis2.is_empty() {
            return 0.0;
        }

        // Count API usage
        let mut counts1 = HashMap::new();
        let mut counts2 = HashMap::new();

        for api in apis1 {
            *counts1.entry(api.api_name.clone()).or_insert(0u32) += api.usage_count;
        }

        for api in apis2 {
            *counts2.entry(api.api_name.clone()).or_insert(0u32) += api.usage_count;
        }

        // Compute Jaccard similarity
        let all_apis: HashSet<_> = counts1.keys().chain(counts2.keys()).collect();
        let mut intersection = 0u32;
        let mut union = 0u32;

        for api_name in all_apis {
            let count1 = counts1.get(api_name).copied().unwrap_or(0);
            let count2 = counts2.get(api_name).copied().unwrap_or(0);

            intersection += count1.min(count2);
            union += count1.max(count2);
        }

        if union == 0 {
            1.0
        } else {
            intersection as f64 / union as f64
        }
    }

    /// Normalizes difference between two values
    fn normalize_difference(&self, val1: f64, val2: f64, max_val: f64) -> f64 {
        let diff = (val1 - val2).abs();
        let normalized_diff = diff / max_val;
        1.0 - normalized_diff.min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::cfg::ControlFlowGraph;

    #[test]
    fn test_semantic_analyzer_creation() {
        let analyzer = SemanticAnalyzer::new(SourceLanguage::Rust);
        assert_eq!(analyzer.language_config.language, SourceLanguage::Rust);
        assert!(analyzer
            .language_config
            .common_apis
            .contains(&"println!".to_string()));
    }

    #[test]
    fn test_control_flow_complexity() {
        let analyzer = SemanticAnalyzer::new(SourceLanguage::Rust);
        let cfg = ControlFlowGraph::new();

        let complexity = analyzer.analyze_control_flow_complexity(&cfg);
        assert!(complexity >= 0.0);
    }

    #[test]
    fn test_cyclomatic_complexity() {
        let analyzer = SemanticAnalyzer::new(SourceLanguage::Rust);
        let cfg = ControlFlowGraph::new();

        let complexity = analyzer.compute_cyclomatic_complexity(&cfg);
        // Complexity should be valid (no need to check >= 0 for u32)
        let _complexity_value = complexity;
    }

    #[test]
    fn test_feature_vector_normalization() {
        let analyzer = SemanticAnalyzer::new(SourceLanguage::Rust);
        let mut features = vec![10.0, 20.0, 30.0];

        analyzer.normalize_feature_vector(&mut features);

        // All values should be normalized to [0, 1]
        for &feature in &features {
            assert!(feature >= 0.0 && feature <= 1.0);
        }
    }

    #[test]
    fn test_data_flow_pattern_detection() {
        let analyzer = SemanticAnalyzer::new(SourceLanguage::Rust);
        // This is a simplified test - in practice, we'd need real AST nodes
        // For now, just verify the pattern types are correctly defined
        let pattern = DataFlowPattern {
            pattern_type: DataFlowType::Assignment,
            variables: vec!["test".to_string()],
            operations: vec!["=".to_string()],
            confidence: 0.8,
        };

        assert_eq!(pattern.pattern_type, DataFlowType::Assignment);
        assert_eq!(pattern.confidence, 0.8);
    }

    #[test]
    fn test_wl_kernel_creation() {
        let kernel = WeisfeilerLehmanKernel::new(5);
        assert_eq!(kernel.iterations, 5);
        assert_eq!(kernel.hasher_seed, 0);

        let kernel_with_seed = WeisfeilerLehmanKernel::with_seed(3, 12345);
        assert_eq!(kernel_with_seed.iterations, 3);
        assert_eq!(kernel_with_seed.hasher_seed, 12345);
    }

    #[test]
    fn test_wl_kernel_empty_graphs() {
        let kernel = WeisfeilerLehmanKernel::new(3);
        let empty_graph1 = Graph::new();
        let empty_graph2 = Graph::new();

        let similarity = kernel.compute_similarity(&empty_graph1, &empty_graph2);
        assert_eq!(similarity, 1.0);
    }

    #[test]
    fn test_hybrid_scorer_creation() {
        let scorer = HybridSimilarityScorer::new(SourceLanguage::Rust);
        assert_eq!(scorer.weights.structural, 0.3);
        assert_eq!(scorer.weights.cfg, 0.3);
        assert_eq!(scorer.weights.semantic, 0.25);
        assert_eq!(scorer.weights.ast_pattern, 0.15);
    }

    #[test]
    fn test_hybrid_scorer_with_custom_weights() {
        let custom_weights = SimilarityWeights {
            structural: 0.4,
            cfg: 0.4,
            semantic: 0.1,
            ast_pattern: 0.1,
        };

        let scorer = HybridSimilarityScorer::with_weights(SourceLanguage::Python, custom_weights);
        assert_eq!(scorer.weights.structural, 0.4);
        assert_eq!(scorer.weights.cfg, 0.4);
    }

    #[test]
    fn test_normalize_difference() {
        let scorer = HybridSimilarityScorer::new(SourceLanguage::Rust);

        // Same values should give similarity of 1.0
        let similarity = scorer.normalize_difference(5.0, 5.0, 10.0);
        assert_eq!(similarity, 1.0);

        // Maximum difference should give similarity of 0.0
        let similarity = scorer.normalize_difference(0.0, 10.0, 10.0);
        assert_eq!(similarity, 0.0);

        // Half difference should give similarity of 0.5
        let similarity = scorer.normalize_difference(2.5, 7.5, 10.0);
        assert_eq!(similarity, 0.5);
    }
}
