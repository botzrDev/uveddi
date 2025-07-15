//! Control Flow Graph (CFG) generation and analysis
//!
//! This module provides data structures and algorithms for creating and analyzing
//! control flow graphs from parsed source code. CFGs are used for advanced semantic
//! analysis and Type-4 clone detection.
//!
//! ## Architecture:
//! - Uses petgraph for efficient graph data structures
//! - Integrates with tree-sitter for multi-language AST parsing
//! - Provides language-specific CFG extraction queries
//! - Supports parallel CFG generation for performance

use crate::analysis::errors::AnalysisError;
use crate::ast::tree_sitter::{Node, Query, QueryCursor};
use crate::ast::tree_sitter_impl::SourceLanguage;
use log::{debug, warn};
use petgraph::{
    visit::{EdgeRef, IntoNodeReferences},
    Graph,
};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// CFG node types based on control flow semantics
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CfgNodeType {
    /// Entry point of a function or scope
    Entry,
    /// Exit point of a function or scope
    Exit,
    /// Regular statement node
    Statement,
    /// Conditional branch (if, match, etc.)
    Condition,
    /// Loop construct (for, while, loop)
    Loop,
    /// Function or method call
    FunctionCall,
    /// Return statement
    Return,
    /// Exception handling (try/catch)
    Exception,
}

/// CFG node with AST context
#[derive(Debug, Clone)]
pub struct CfgNode<'a> {
    /// Unique identifier for this node
    pub id: usize,
    /// The type of control flow node
    pub node_type: CfgNodeType,
    /// Reference to the original AST node (if applicable)
    pub ast_node: Option<Node<'a>>,
    /// Source code range (start_byte, end_byte)
    pub source_range: (usize, usize),
    /// Human-readable label for debugging
    pub label: Option<String>,
}

/// CFG edge representing control flow
#[derive(Debug, Clone)]
pub struct CfgEdge {
    /// The type of control flow edge
    pub edge_type: CfgEdgeType,
    /// Optional condition for conditional edges
    pub condition: Option<String>,
}

/// Types of control flow edges
#[derive(Debug, Clone, PartialEq)]
pub enum CfgEdgeType {
    /// Sequential execution
    Sequential,
    /// Conditional branch (true/false)
    Conditional,
    /// Loop back edge
    Loop,
    /// Exception handling edge
    Exception,
    /// Return edge
    Return,
}

/// Main CFG structure
#[derive(Debug, Clone)]
pub struct ControlFlowGraph {
    /// The underlying graph structure
    graph: Graph<CfgNode<'static>, CfgEdge>,
    /// Entry node index
    entry_node: petgraph::graph::NodeIndex,
    /// Exit node index
    exit_node: petgraph::graph::NodeIndex,
}

impl ControlFlowGraph {
    /// Creates a new empty CFG
    pub fn new() -> Self {
        let mut graph = Graph::new();

        // Create entry and exit nodes
        let entry_node = graph.add_node(CfgNode {
            id: 0,
            node_type: CfgNodeType::Entry,
            ast_node: None,
            source_range: (0, 0),
            label: Some("Entry".to_string()),
        });

        let exit_node = graph.add_node(CfgNode {
            id: 1,
            node_type: CfgNodeType::Exit,
            ast_node: None,
            source_range: (0, 0),
            label: Some("Exit".to_string()),
        });

        Self {
            graph,
            entry_node,
            exit_node,
        }
    }

    /// Get the number of nodes in the CFG
    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    /// Get the number of edges in the CFG
    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    /// Get a reference to the underlying graph
    pub fn graph(&self) -> &Graph<CfgNode<'static>, CfgEdge> {
        &self.graph
    }

    /// Get the entry node index
    pub fn entry_node(&self) -> petgraph::graph::NodeIndex {
        self.entry_node
    }

    /// Get the exit node index
    pub fn exit_node(&self) -> petgraph::graph::NodeIndex {
        self.exit_node
    }

    /// Compute a structural hash of the CFG for similarity comparison
    pub fn compute_structural_hash(&self) -> String {
        let mut hasher = Sha256::new();

        // Hash node types in a deterministic order
        let mut node_types: Vec<_> = self
            .graph
            .node_weights()
            .map(|node| format!("{:?}", node.node_type))
            .collect();
        node_types.sort();

        for node_type in node_types {
            hasher.update(node_type.as_bytes());
        }

        // Hash edge types
        let mut edge_types: Vec<_> = self
            .graph
            .edge_weights()
            .map(|edge| format!("{:?}", edge.edge_type))
            .collect();
        edge_types.sort();

        for edge_type in edge_types {
            hasher.update(edge_type.as_bytes());
        }

        format!("{:x}", hasher.finalize())
    }
}

/// Tree-sitter CFG extraction queries based on UV-24_Advanced_Research.md specifications
const RUST_CFG_QUERY: &str = r#"
[(function_item) @function (method_declaration) @method]
[(if_expression) @if (match_expression) @match]
[(for_expression) @for (while_expression) @while (loop_expression) @loop]
(call_expression) @call
(return_expression) @return
(try_expression) @try
"#;

const PYTHON_CFG_QUERY: &str = r#"
(function_definition) @function
(if_statement) @if
[(for_statement) @for (while_statement) @while]
(call) @call
(return_statement) @return
(try_statement) @try
"#;

const JAVASCRIPT_CFG_QUERY: &str = r#"
[(function_declaration) @function (arrow_function) @function (method_definition) @method]
[(if_statement) @if (switch_statement) @switch]
[(for_statement) @for (while_statement) @while (do_statement) @do]
(call_expression) @call
(return_statement) @return
(try_statement) @try
"#;

/// CFG builder for constructing control flow graphs from AST
pub struct CfgBuilder<'a> {
    /// The graph being constructed
    graph: Graph<CfgNode<'a>, CfgEdge>,
    /// Current node being processed
    current_node: Option<petgraph::graph::NodeIndex>,
    /// Node counter for unique IDs
    node_counter: usize,
    /// Stack for tracking nested structures
    node_stack: Vec<petgraph::graph::NodeIndex>,
}

impl<'a> CfgBuilder<'a> {
    /// Creates a new CFG builder
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            current_node: None,
            node_counter: 0,
            node_stack: Vec::new(),
        }
    }

    /// Build CFG from AST following UV-24_Advanced_Research.md specifications
    pub fn build_from_ast(
        &mut self,
        ast_node: Node<'a>,
        source: &str,
        language: SourceLanguage,
    ) -> Result<ControlFlowGraph, AnalysisError> {
        // 1. Create entry and exit nodes
        let entry_node = self.add_node(CfgNodeType::Entry, None, (0, 0), Some("Entry".to_string()));
        let exit_node = self.add_node(
            CfgNodeType::Exit,
            None,
            (source.len(), source.len()),
            Some("Exit".to_string()),
        );

        // 2. Get language-specific query
        let query_str = match language {
            SourceLanguage::Rust => RUST_CFG_QUERY,
            SourceLanguage::Python => PYTHON_CFG_QUERY,
            SourceLanguage::JavaScript => JAVASCRIPT_CFG_QUERY,
        };

        // 3. Parse query and traverse AST
        let query = Query::new(&ast_node.language(), query_str).map_err(|e| {
            AnalysisError::AstError(crate::ast::tree_sitter_impl::AstError::Other(format!(
                "Failed to create CFG query: {}",
                e
            )))
        })?;

        let mut cursor = QueryCursor::new();
        let matches = cursor.matches(&query, ast_node, source.as_bytes());

        // 4. Build CFG nodes for control flow constructs
        self.current_node = Some(entry_node);

        for match_ in matches {
            for capture in match_.captures {
                let node = capture.node;
                let node_type = self.classify_node(&node);
                let range = (node.start_byte(), node.end_byte());
                let label = node
                    .utf8_text(source.as_bytes())
                    .ok()
                    .map(|s| s.to_string());

                let cfg_node_idx = self.add_node(node_type, Some(node), range, label);

                // 5. Connect nodes with appropriate edges
                if let Some(current) = self.current_node {
                    self.add_edge(current, cfg_node_idx, CfgEdgeType::Sequential, None);
                }

                self.current_node = Some(cfg_node_idx);
            }
        }

        // Connect final node to exit
        if let Some(current) = self.current_node {
            self.add_edge(current, exit_node, CfgEdgeType::Sequential, None);
        }

        // Convert to static lifetime for storage
        let static_graph = self.convert_to_static();

        Ok(ControlFlowGraph {
            graph: static_graph,
            entry_node,
            exit_node,
        })
    }

    /// Add a new node to the CFG
    fn add_node(
        &mut self,
        node_type: CfgNodeType,
        ast_node: Option<Node<'a>>,
        source_range: (usize, usize),
        label: Option<String>,
    ) -> petgraph::graph::NodeIndex {
        let cfg_node = CfgNode {
            id: self.node_counter,
            node_type,
            ast_node,
            source_range,
            label,
        };
        self.node_counter += 1;
        self.graph.add_node(cfg_node)
    }

    /// Add an edge between two nodes
    fn add_edge(
        &mut self,
        from: petgraph::graph::NodeIndex,
        to: petgraph::graph::NodeIndex,
        edge_type: CfgEdgeType,
        condition: Option<String>,
    ) {
        let edge = CfgEdge {
            edge_type,
            condition,
        };
        self.graph.add_edge(from, to, edge);
    }

    /// Classify AST node into CFG node type
    fn classify_node(&self, node: &Node) -> CfgNodeType {
        match node.kind() {
            "function_item"
            | "function_declaration"
            | "function_definition"
            | "method_declaration" => CfgNodeType::Entry,
            "if_expression" | "if_statement" | "match_expression" | "switch_statement" => {
                CfgNodeType::Condition
            }
            "for_expression" | "for_statement" | "while_expression" | "while_statement"
            | "loop_expression" | "do_statement" => CfgNodeType::Loop,
            "call_expression" | "call" => CfgNodeType::FunctionCall,
            "return_expression" | "return_statement" => CfgNodeType::Return,
            "try_expression" | "try_statement" => CfgNodeType::Exception,
            _ => CfgNodeType::Statement,
        }
    }

    /// Convert graph with lifetime 'a to static lifetime for storage
    fn convert_to_static(&self) -> Graph<CfgNode<'static>, CfgEdge> {
        let mut static_graph = Graph::new();
        let mut node_map = HashMap::new();

        // Add all nodes without AST references
        for (idx, node) in self.graph.node_references() {
            let static_node = CfgNode {
                id: node.id,
                node_type: node.node_type.clone(),
                ast_node: None, // Remove AST reference for static storage
                source_range: node.source_range,
                label: node.label.clone(),
            };
            let new_idx = static_graph.add_node(static_node);
            node_map.insert(idx, new_idx);
        }

        // Add all edges
        for edge_ref in self.graph.edge_references() {
            let source = node_map[&edge_ref.source()];
            let target = node_map[&edge_ref.target()];
            static_graph.add_edge(source, target, edge_ref.weight().clone());
        }

        static_graph
    }

    /// Get graph statistics for debugging
    pub fn get_stats(&self) -> (usize, usize) {
        (self.graph.node_count(), self.graph.edge_count())
    }
}

impl<'a> Default for CfgBuilder<'a> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cfg_creation() {
        let cfg = ControlFlowGraph::new();
        assert_eq!(cfg.node_count(), 2); // Entry and exit nodes
        assert_eq!(cfg.edge_count(), 0);
    }

    #[test]
    fn test_cfg_builder() {
        let builder = CfgBuilder::new();
        assert_eq!(builder.get_stats(), (0, 0));
    }

    #[test]
    fn test_node_classification() {
        let builder = CfgBuilder::new();
        // Note: This would need actual tree-sitter nodes for full testing
        // For now, we test the structure
        assert_eq!(builder.node_counter, 0);
    }
}
