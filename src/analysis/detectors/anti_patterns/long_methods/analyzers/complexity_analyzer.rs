//! Complexity analysis for methods

use crate::analysis::AnalysisError;
#[cfg(feature = "tree-sitter")]
use crate::ast::tree_sitter::Node;

/// Metrics for different complexity measurements
#[derive(Debug, Clone, Default)]
pub struct ComplexityMetrics {
    pub cyclomatic: u32,
    pub cognitive: u32,
    pub essential: u32,
}

/// Analyzer for calculating various complexity metrics
pub struct ComplexityAnalyzer;

impl ComplexityAnalyzer {
    /// Calculate cyclomatic complexity
    #[cfg(feature = "tree-sitter")]
    pub fn calculate_cyclomatic(node: &Node, _source: &[u8]) -> Result<u32, AnalysisError> {
        let mut complexity = 1; // Base complexity
        Self::traverse_cyclomatic(node, &mut complexity);
        Ok(complexity)
    }

    #[cfg(feature = "tree-sitter")]
    fn traverse_cyclomatic(node: &Node, complexity: &mut u32) {
        let mut cursor = node.walk();

        if cursor.goto_first_child() {
            loop {
                let child_node = cursor.node();
                let node_type = child_node.kind();

                // Increment for decision points
                if Self::is_decision_point(node_type) {
                    *complexity += 1;
                }

                // Recursive traversal
                Self::traverse_cyclomatic(&child_node, complexity);

                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
    }

    /// Check if a node represents a decision point
    fn is_decision_point(node_type: &str) -> bool {
        matches!(
            node_type,
            "if_statement"
                | "if_expression"
                | "while_statement"
                | "while_expression"
                | "for_statement"
                | "for_expression"
                | "loop_expression"
                | "match_expression"
                | "conditional_expression"
                | "logical_and"
                | "logical_or"
                | "match_arm"
                | "case_statement"
                | "catch_clause"
                | "elif_clause"
                | "else_clause"
        )
    }

    /// Calculate cognitive complexity (more nuanced than cyclomatic)
    #[cfg(feature = "tree-sitter")]
    pub fn calculate_cognitive(node: &Node, _source: &[u8]) -> Result<u32, AnalysisError> {
        let mut complexity = 0;
        Self::traverse_cognitive(node, &mut complexity, 0);
        Ok(complexity)
    }

    #[cfg(feature = "tree-sitter")]
    fn traverse_cognitive(node: &Node, complexity: &mut u32, nesting_level: u32) {
        let mut cursor = node.walk();

        if cursor.goto_first_child() {
            loop {
                let child_node = cursor.node();
                let node_type = child_node.kind();

                let (increment, increases_nesting) = Self::cognitive_weight(node_type);

                if increment > 0 {
                    *complexity += increment + nesting_level;
                }

                let new_nesting = if increases_nesting {
                    nesting_level + 1
                } else {
                    nesting_level
                };

                Self::traverse_cognitive(&child_node, complexity, new_nesting);

                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
    }

    /// Get cognitive complexity weight for a node type
    fn cognitive_weight(node_type: &str) -> (u32, bool) {
        match node_type {
            // Control flow with nesting
            "if_statement" | "if_expression" => (1, true),
            "while_statement" | "while_expression" => (1, true),
            "for_statement" | "for_expression" => (1, true),
            "match_expression" | "switch_statement" => (1, true),
            "conditional_expression" => (1, true),
            "catch_clause" | "except_clause" => (1, true),

            // Binary logical operators
            "logical_and" | "logical_or" => (1, false),

            // Jump statements
            "break_statement" | "continue_statement" => (1, false),
            "return_statement" | "return_expression" => (0, false),

            // Recursion (higher weight)
            "recursive_call" => (3, false),

            _ => (0, false),
        }
    }

    /// Calculate essential complexity (minimal complexity after refactoring)
    #[cfg(feature = "tree-sitter")]
    pub fn calculate_essential(node: &Node, _source: &[u8]) -> Result<u32, AnalysisError> {
        let cyclomatic = Self::calculate_cyclomatic(node, _source)?;

        // Essential complexity is approximately cyclomatic complexity
        // minus the complexity that could be removed through structured programming
        let reducible = Self::count_reducible_complexity(node);
        Ok(cyclomatic.saturating_sub(reducible))
    }

    #[cfg(feature = "tree-sitter")]
    fn count_reducible_complexity(node: &Node) -> u32 {
        let mut reducible = 0;

        let mut cursor = node.walk();
        if cursor.goto_first_child() {
            loop {
                let child_node = cursor.node();
                let node_type = child_node.kind();

                // These patterns can often be simplified
                if matches!(
                    node_type,
                    "else_clause" | "elif_clause" | "nested_if" | "nested_loop"
                ) {
                    reducible += 1;
                }

                reducible += Self::count_reducible_complexity(&child_node);

                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }

        reducible
    }

    /// Calculate all complexity metrics at once
    #[cfg(feature = "tree-sitter")]
    pub fn calculate_all_metrics(
        node: &Node,
        source: &[u8],
    ) -> Result<ComplexityMetrics, AnalysisError> {
        Ok(ComplexityMetrics {
            cyclomatic: Self::calculate_cyclomatic(node, source)?,
            cognitive: Self::calculate_cognitive(node, source)?,
            essential: Self::calculate_essential(node, source)?,
        })
    }
}
