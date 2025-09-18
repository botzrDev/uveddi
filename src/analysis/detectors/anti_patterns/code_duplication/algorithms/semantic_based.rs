//! Semantic-based clone detection using control flow analysis

use super::CloneDetectionAlgorithm;
use crate::analysis::detectors::anti_patterns::code_duplication::types::{CodeBlock, ClonePair, CloneType};
use crate::analysis::AnalysisError;
use crate::ast::tree_sitter_impl::SourceLanguage;
use std::collections::{HashMap, HashSet};

/// Semantic clone detector using control flow graph analysis
pub struct SemanticDetector {
    /// Similarity threshold for semantic comparison
    similarity_threshold: f64,
    /// Weight for CFG similarity in overall score
    cfg_weight: f64,
    /// Number of iterations for graph kernel
    kernel_iterations: usize,
    /// Maximum nodes to process (performance limit)
    max_nodes: usize,
}

impl SemanticDetector {
    /// Creates a new semantic detector
    pub fn new(similarity_threshold: f64) -> Self {
        Self {
            similarity_threshold,
            cfg_weight: 0.6,
            kernel_iterations: 3,
            max_nodes: 500,
        }
    }

    /// Creates a detector with custom settings
    pub fn with_settings(
        similarity_threshold: f64,
        cfg_weight: f64,
        kernel_iterations: usize,
        max_nodes: usize,
    ) -> Self {
        Self {
            similarity_threshold,
            cfg_weight,
            kernel_iterations,
            max_nodes,
        }
    }

    /// Extracts control flow graph from source code
    pub fn extract_cfg(&self, source: &str, _language: &SourceLanguage) -> Result<ControlFlowGraph, AnalysisError> {
        // Simplified CFG extraction - real implementation would use proper parsing
        let statements = self.extract_statements(source);
        let mut cfg = ControlFlowGraph::new();

        let mut current_node = 0;
        for (i, statement) in statements.iter().enumerate() {
            let node_id = cfg.add_node(CfgNode {
                id: i,
                statement_type: self.classify_statement(statement),
                content: statement.clone(),
            });

            if i > 0 {
                cfg.add_edge(current_node, node_id);
            }
            current_node = node_id;

            // Handle control flow changes
            if self.is_branch_statement(statement) {
                // Add conditional edges (simplified)
                let branch_node = cfg.add_node(CfgNode {
                    id: i + 1000, // Offset to avoid conflicts
                    statement_type: StatementType::Block,
                    content: "branch_target".to_string(),
                });
                cfg.add_edge(node_id, branch_node);
            }
        }

        Ok(cfg)
    }

    /// Extracts statements from source code
    fn extract_statements(&self, source: &str) -> Vec<String> {
        source
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty() && !line.starts_with("//"))
            .collect()
    }

    /// Classifies a statement by type
    fn classify_statement(&self, statement: &str) -> StatementType {
        if statement.contains("if") || statement.contains("else") {
            StatementType::Conditional
        } else if statement.contains("while") || statement.contains("for") {
            StatementType::Loop
        } else if statement.contains("return") {
            StatementType::Return
        } else if statement.contains("=") && !statement.contains("==") {
            StatementType::Assignment
        } else if statement.contains("(") && statement.contains(")") {
            StatementType::FunctionCall
        } else {
            StatementType::Expression
        }
    }

    /// Checks if statement creates a branch
    fn is_branch_statement(&self, statement: &str) -> bool {
        statement.contains("if") || statement.contains("switch") || statement.contains("match")
    }

    /// Computes semantic similarity between two CFGs
    pub fn compute_semantic_similarity(&self, cfg1: &ControlFlowGraph, cfg2: &ControlFlowGraph) -> f64 {
        if cfg1.nodes.len() > self.max_nodes || cfg2.nodes.len() > self.max_nodes {
            return 0.5; // Default similarity for large graphs
        }

        // Compute structural similarity
        let structural_sim = self.compute_structural_similarity(cfg1, cfg2);

        // Compute flow similarity
        let flow_sim = self.compute_flow_similarity(cfg1, cfg2);

        // Compute node type similarity
        let type_sim = self.compute_node_type_similarity(cfg1, cfg2);

        // Weighted combination
        0.4 * structural_sim + 0.4 * flow_sim + 0.2 * type_sim
    }

    /// Computes structural similarity between CFGs
    fn compute_structural_similarity(&self, cfg1: &ControlFlowGraph, cfg2: &ControlFlowGraph) -> f64 {
        let size_diff = (cfg1.nodes.len() as f64 - cfg2.nodes.len() as f64).abs();
        let max_size = cfg1.nodes.len().max(cfg2.nodes.len()) as f64;

        if max_size == 0.0 {
            return 1.0;
        }

        1.0 - (size_diff / max_size)
    }

    /// Computes flow pattern similarity
    fn compute_flow_similarity(&self, cfg1: &ControlFlowGraph, cfg2: &ControlFlowGraph) -> f64 {
        let patterns1 = self.extract_flow_patterns(cfg1);
        let patterns2 = self.extract_flow_patterns(cfg2);

        self.compare_pattern_sets(&patterns1, &patterns2)
    }

    /// Computes node type distribution similarity
    fn compute_node_type_similarity(&self, cfg1: &ControlFlowGraph, cfg2: &ControlFlowGraph) -> f64 {
        let types1 = self.get_type_distribution(cfg1);
        let types2 = self.get_type_distribution(cfg2);

        self.compare_distributions(&types1, &types2)
    }

    /// Extracts flow patterns from CFG
    fn extract_flow_patterns(&self, cfg: &ControlFlowGraph) -> HashSet<FlowPattern> {
        let mut patterns = HashSet::new();

        for node_id in cfg.nodes.keys() {
            let successors: Vec<_> = cfg.edges.get(node_id).cloned().unwrap_or_default();

            match successors.len() {
                0 => patterns.insert(FlowPattern::Terminal),
                1 => patterns.insert(FlowPattern::Sequential),
                2 => patterns.insert(FlowPattern::Branch),
                _ => patterns.insert(FlowPattern::MultiBranch),
            };
        }

        patterns
    }

    /// Gets type distribution for nodes
    fn get_type_distribution(&self, cfg: &ControlFlowGraph) -> HashMap<StatementType, usize> {
        let mut distribution = HashMap::new();

        for node in cfg.nodes.values() {
            *distribution.entry(node.statement_type.clone()).or_insert(0) += 1;
        }

        distribution
    }

    /// Compares two pattern sets
    fn compare_pattern_sets(&self, patterns1: &HashSet<FlowPattern>, patterns2: &HashSet<FlowPattern>) -> f64 {
        if patterns1.is_empty() && patterns2.is_empty() {
            return 1.0;
        }

        let intersection = patterns1.intersection(patterns2).count();
        let union = patterns1.union(patterns2).count();

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }

    /// Compares two distributions
    fn compare_distributions(&self, dist1: &HashMap<StatementType, usize>, dist2: &HashMap<StatementType, usize>) -> f64 {
        let all_types: HashSet<_> = dist1.keys().chain(dist2.keys()).collect();

        if all_types.is_empty() {
            return 1.0;
        }

        let total1: usize = dist1.values().sum();
        let total2: usize = dist2.values().sum();

        if total1 == 0 && total2 == 0 {
            return 1.0;
        }

        let mut similarity = 0.0;
        let all_types_len = all_types.len();
        for stmt_type in &all_types {
            let freq1 = *dist1.get(stmt_type).unwrap_or(&0) as f64 / total1.max(1) as f64;
            let freq2 = *dist2.get(stmt_type).unwrap_or(&0) as f64 / total2.max(1) as f64;
            similarity += 1.0 - (freq1 - freq2).abs();
        }

        similarity / all_types_len as f64
    }
}

impl CloneDetectionAlgorithm for SemanticDetector {
    fn name(&self) -> &'static str {
        "Semantic CFG-based"
    }

    fn detect_clones(&self, block1: &CodeBlock, block2: &CodeBlock) -> Result<Option<ClonePair>, AnalysisError> {
        // Skip if same file and overlapping regions
        if block1.file_path == block2.file_path &&
           !(block1.end_line < block2.start_line || block2.end_line < block1.start_line) {
            return Ok(None);
        }

        // Extract CFGs
        let cfg1 = self.extract_cfg(&block1.source, &block1.language)?;
        let cfg2 = self.extract_cfg(&block2.source, &block2.language)?;

        // Compute semantic similarity
        let similarity = self.compute_semantic_similarity(&cfg1, &cfg2);

        if similarity < self.similarity_threshold {
            return Ok(None);
        }

        // Semantic clones are typically Type-4
        let clone_type = CloneType::Type4;

        Ok(Some(ClonePair::new(
            block1.clone(),
            block2.clone(),
            similarity,
            clone_type,
            0, // No fingerprint concept for semantic detection
        )))
    }

    fn supports_language(&self, language: &SourceLanguage) -> bool {
        // Semantic analysis requires language-specific knowledge
        matches!(
            language,
            SourceLanguage::Rust | SourceLanguage::Python | SourceLanguage::JavaScript | SourceLanguage::TypeScript
        )
    }

    fn confidence_level(&self) -> f64 {
        0.75 // Moderate confidence for semantic detection
    }
}

/// Control Flow Graph representation
#[derive(Debug, Clone)]
pub struct ControlFlowGraph {
    nodes: HashMap<usize, CfgNode>,
    edges: HashMap<usize, Vec<usize>>,
    next_id: usize,
}

impl ControlFlowGraph {
    fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            next_id: 0,
        }
    }

    fn add_node(&mut self, node: CfgNode) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        self.nodes.insert(id, node);
        self.edges.insert(id, Vec::new());
        id
    }

    fn add_edge(&mut self, from: usize, to: usize) {
        self.edges.entry(from).or_default().push(to);
    }
}

/// Control Flow Graph node
#[derive(Debug, Clone)]
pub struct CfgNode {
    id: usize,
    statement_type: StatementType,
    content: String,
}

/// Types of statements in control flow
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum StatementType {
    Assignment,
    Conditional,
    Loop,
    FunctionCall,
    Return,
    Expression,
    Block,
}

/// Flow patterns in control flow graphs
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum FlowPattern {
    Sequential,
    Branch,
    MultiBranch,
    Terminal,
}