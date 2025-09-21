//! Structural feature extraction from AST

use super::types::{AstNode, NodeType};
use std::collections::HashMap;

/// Structural features extracted from AST
#[derive(Debug, Clone)]
pub struct StructuralFeatures {
    pub node_count: usize,
    pub max_depth: usize,
    pub keyword_counts: HashMap<String, usize>,
    pub operator_counts: HashMap<String, usize>,
    pub children_counts: Vec<usize>,
    pub type_distribution: HashMap<String, usize>,
}

impl StructuralFeatures {
    pub fn new() -> Self {
        Self {
            node_count: 0,
            max_depth: 0,
            keyword_counts: HashMap::new(),
            operator_counts: HashMap::new(),
            children_counts: Vec::new(),
            type_distribution: HashMap::new(),
        }
    }

    /// Calculates similarity with another feature set
    pub fn similarity(&self, other: &StructuralFeatures) -> f64 {
        let size_similarity = self.size_similarity(other);
        let keyword_similarity = self.keyword_similarity(other);
        let operator_similarity = self.operator_similarity(other);
        let structure_similarity = self.structure_similarity(other);

        (size_similarity + keyword_similarity + operator_similarity + structure_similarity) / 4.0
    }

    fn size_similarity(&self, other: &StructuralFeatures) -> f64 {
        let max_nodes = self.node_count.max(other.node_count) as f64;
        if max_nodes == 0.0 {
            return 1.0;
        }
        1.0 - (self.node_count as f64 - other.node_count as f64).abs() / max_nodes
    }

    fn keyword_similarity(&self, other: &StructuralFeatures) -> f64 {
        self.distribution_similarity(&self.keyword_counts, &other.keyword_counts)
    }

    fn operator_similarity(&self, other: &StructuralFeatures) -> f64 {
        self.distribution_similarity(&self.operator_counts, &other.operator_counts)
    }

    fn structure_similarity(&self, other: &StructuralFeatures) -> f64 {
        let depth_sim = if self.max_depth == 0 && other.max_depth == 0 {
            1.0
        } else {
            let max_depth = self.max_depth.max(other.max_depth) as f64;
            1.0 - (self.max_depth as f64 - other.max_depth as f64).abs() / max_depth
        };

        // Compare children distribution
        let children_sim = self.children_distribution_similarity(other);

        (depth_sim + children_sim) / 2.0
    }

    fn distribution_similarity(&self, dist1: &HashMap<String, usize>, dist2: &HashMap<String, usize>) -> f64 {
        let all_keys: std::collections::HashSet<_> = dist1.keys().chain(dist2.keys()).collect();

        if all_keys.is_empty() {
            return 1.0;
        }

        let total1: usize = dist1.values().sum();
        let total2: usize = dist2.values().sum();

        if total1 == 0 && total2 == 0 {
            return 1.0;
        }

        let mut similarity = 0.0;
        let all_keys_len = all_keys.len();
        for key in &all_keys {
            let freq1 = *dist1.get(key).unwrap_or(&0) as f64 / total1.max(1) as f64;
            let freq2 = *dist2.get(key).unwrap_or(&0) as f64 / total2.max(1) as f64;
            similarity += 1.0 - (freq1 - freq2).abs();
        }

        similarity / all_keys_len as f64
    }

    fn children_distribution_similarity(&self, other: &StructuralFeatures) -> f64 {
        // Create histograms of children counts
        let mut hist1 = HashMap::new();
        let mut hist2 = HashMap::new();

        for &count in &self.children_counts {
            *hist1.entry(count.to_string()).or_insert(0) += 1;
        }

        for &count in &other.children_counts {
            *hist2.entry(count.to_string()).or_insert(0) += 1;
        }

        self.distribution_similarity(&hist1, &hist2)
    }
}

impl Default for StructuralFeatures {
    fn default() -> Self {
        Self::new()
    }
}

/// Extracts structural features from AST nodes
pub struct FeatureExtractor {
    max_depth: usize,
}

impl FeatureExtractor {
    pub fn new(max_depth: usize) -> Self {
        Self { max_depth }
    }

    pub fn extract_features(&self, node: &AstNode) -> StructuralFeatures {
        let mut features = StructuralFeatures::new();
        self.collect_features(node, &mut features, 0);
        features
    }

    fn collect_features(&self, node: &AstNode, features: &mut StructuralFeatures, depth: usize) {
        if depth > self.max_depth {
            return;
        }

        features.node_count += 1;
        features.max_depth = features.max_depth.max(depth);

        // Collect type-specific information
        match &node.node_type {
            NodeType::Keyword(k) => {
                *features.keyword_counts.entry(k.clone()).or_insert(0) += 1;
            }
            NodeType::Operator(o) => {
                *features.operator_counts.entry(o.clone()).or_insert(0) += 1;
            }
            node_type => {
                let type_name = format!("{:?}", node_type);
                *features.type_distribution.entry(type_name).or_insert(0) += 1;
            }
        }

        features.children_counts.push(node.children.len());

        // Recursively process children
        for child in &node.children {
            self.collect_features(child, features, depth + 1);
        }
    }
}

impl Default for FeatureExtractor {
    fn default() -> Self {
        Self::new(50) // Default max depth
    }
}