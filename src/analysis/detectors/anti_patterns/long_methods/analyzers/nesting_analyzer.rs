//! Nesting depth analysis for methods

#[cfg(feature = "tree-sitter")]
use crate::ast::tree_sitter::Node;

/// Analyzer for calculating nesting depth metrics
pub struct NestingAnalyzer;

impl NestingAnalyzer {
    /// Calculate maximum nesting depth in a method
    #[cfg(feature = "tree-sitter")]
    pub fn calculate_max_nesting_depth(node: &Node, _source: &[u8]) -> u32 {
        Self::calculate_nesting_recursive(node, 0)
    }

    #[cfg(feature = "tree-sitter")]
    fn calculate_nesting_recursive(node: &Node, current_depth: u32) -> u32 {
        let node_kind = node.kind();
        let increases_nesting = Self::increases_nesting(node_kind);

        let new_depth = if increases_nesting {
            current_depth + 1
        } else {
            current_depth
        };

        let mut max_depth = new_depth;

        // Check all children
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                let child_depth = Self::calculate_nesting_recursive(&child, new_depth);
                max_depth = max_depth.max(child_depth);
            }
        }

        max_depth
    }

    /// Check if a node type increases nesting depth
    fn increases_nesting(node_kind: &str) -> bool {
        matches!(
            node_kind,
            "block"
                | "statement_block"
                | "if_statement"
                | "if_expression"
                | "else_clause"
                | "elif_clause"
                | "while_statement"
                | "while_expression"
                | "for_statement"
                | "for_expression"
                | "loop_expression"
                | "match_expression"
                | "match_arm"
                | "switch_statement"
                | "case_statement"
                | "try_statement"
                | "try_expression"
                | "catch_clause"
                | "except_clause"
                | "finally_clause"
                | "with_statement"
                | "closure_expression"
                | "lambda_expression"
                | "arrow_function"
        )
    }

    /// Calculate average nesting depth
    #[cfg(feature = "tree-sitter")]
    pub fn calculate_average_nesting(node: &Node, source: &[u8]) -> f32 {
        let (total_depth, node_count) = Self::sum_nesting_depths(node, source, 0);

        if node_count == 0 {
            0.0
        } else {
            total_depth as f32 / node_count as f32
        }
    }

    #[cfg(feature = "tree-sitter")]
    fn sum_nesting_depths(node: &Node, _source: &[u8], current_depth: u32) -> (u32, u32) {
        let node_kind = node.kind();
        let increases_nesting = Self::increases_nesting(node_kind);

        let new_depth = if increases_nesting {
            current_depth + 1
        } else {
            current_depth
        };

        let mut total_depth = if increases_nesting { new_depth } else { 0 };
        let mut count = if increases_nesting { 1 } else { 0 };

        // Process children
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                let (child_total, child_count) =
                    Self::sum_nesting_depths(&child, _source, new_depth);
                total_depth += child_total;
                count += child_count;
            }
        }

        (total_depth, count)
    }

    /// Count nodes at each nesting level
    #[cfg(feature = "tree-sitter")]
    pub fn count_by_nesting_level(node: &Node, source: &[u8]) -> Vec<u32> {
        let mut counts = vec![0; 10]; // Support up to 10 levels
        Self::count_nesting_levels(node, source, 0, &mut counts);
        counts
    }

    #[cfg(feature = "tree-sitter")]
    fn count_nesting_levels(
        node: &Node,
        _source: &[u8],
        current_depth: u32,
        counts: &mut Vec<u32>,
    ) {
        let node_kind = node.kind();
        let increases_nesting = Self::increases_nesting(node_kind);

        let new_depth = if increases_nesting {
            current_depth + 1
        } else {
            current_depth
        };

        if increases_nesting && (new_depth as usize) < counts.len() {
            counts[new_depth as usize] += 1;
        }

        // Process children
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                Self::count_nesting_levels(&child, _source, new_depth, counts);
            }
        }
    }

    /// Identify deeply nested blocks that should be refactored
    #[cfg(feature = "tree-sitter")]
    pub fn find_deep_nesting_locations(
        node: &Node,
        source: &[u8],
        threshold: u32,
    ) -> Vec<(u32, u32)> {
        let mut locations = Vec::new();
        Self::find_deep_nesting_recursive(node, source, 0, threshold, &mut locations);
        locations
    }

    #[cfg(feature = "tree-sitter")]
    fn find_deep_nesting_recursive(
        node: &Node,
        _source: &[u8],
        current_depth: u32,
        threshold: u32,
        locations: &mut Vec<(u32, u32)>,
    ) {
        let node_kind = node.kind();
        let increases_nesting = Self::increases_nesting(node_kind);

        let new_depth = if increases_nesting {
            current_depth + 1
        } else {
            current_depth
        };

        if new_depth > threshold {
            let start = node.start_position();
            locations.push((start.row as u32 + 1, start.column as u32 + 1));
        }

        // Process children
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                Self::find_deep_nesting_recursive(&child, _source, new_depth, threshold, locations);
            }
        }
    }
}
