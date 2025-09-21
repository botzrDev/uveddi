//! Path tracking and confidence calculation for taint flows

use crate::analysis::detectors::security::taint_analysis::types::{DataFlowGraph, TaintLevel};
use std::collections::HashMap;

/// Tracker for taint flow paths and confidence calculation
pub struct PathTracker {
    confidence_weights: ConfidenceWeights,
    path_complexity_factors: HashMap<String, f64>,
}

/// Weights for different factors in confidence calculation
#[derive(Debug, Clone)]
struct ConfidenceWeights {
    path_length: f64,
    taint_strength: f64,
    sanitization_presence: f64,
    control_flow_complexity: f64,
    type_consistency: f64,
}

impl PathTracker {
    pub fn new() -> Self {
        Self {
            confidence_weights: ConfidenceWeights::default(),
            path_complexity_factors: HashMap::new(),
        }
    }

    /// Calculate confidence score for a taint flow
    pub fn calculate_confidence(&self, path: &[String], taint_level: &TaintLevel) -> f64 {
        let path_length_score = self.calculate_path_length_score(path);
        let taint_strength_score = taint_level.score();
        let complexity_score = self.calculate_complexity_score(path);

        // Weighted combination of factors
        let confidence = (path_length_score * self.confidence_weights.path_length
            + taint_strength_score * self.confidence_weights.taint_strength
            + complexity_score * self.confidence_weights.control_flow_complexity)
            / (self.confidence_weights.path_length
                + self.confidence_weights.taint_strength
                + self.confidence_weights.control_flow_complexity);

        confidence.clamp(0.0, 1.0)
    }

    /// Calculate confidence based on path length
    fn calculate_path_length_score(&self, path: &[String]) -> f64 {
        match path.len() {
            1..=2 => 0.95,   // Very short path, high confidence
            3..=4 => 0.85,   // Short path, good confidence
            5..=7 => 0.70,   // Medium path, moderate confidence
            8..=10 => 0.50,  // Long path, lower confidence
            11..=15 => 0.30, // Very long path, low confidence
            _ => 0.15,       // Extremely long path, very low confidence
        }
    }

    /// Calculate confidence based on control flow complexity
    fn calculate_complexity_score(&self, path: &[String]) -> f64 {
        let mut complexity_score = 1.0;

        for node_id in path {
            if let Some(factor) = self.path_complexity_factors.get(node_id) {
                complexity_score *= factor;
            }
        }

        complexity_score.clamp(0.0, 1.0)
    }

    /// Track execution path through control structures
    pub fn track_control_flow(&mut self, node_id: &str, control_type: ControlFlowType) {
        let complexity_factor = match control_type {
            ControlFlowType::Sequential => 1.0,
            ControlFlowType::Conditional => 0.8, // Some uncertainty in branches
            ControlFlowType::Loop => 0.7,        // Loops add complexity
            ControlFlowType::FunctionCall => 0.9, // Function calls are generally reliable
            ControlFlowType::Exception => 0.5,   // Exception handling is complex
            ControlFlowType::Async => 0.6,       // Async operations add uncertainty
        };

        self.path_complexity_factors
            .insert(node_id.to_string(), complexity_factor);
    }

    /// Calculate path divergence score for multiple paths
    pub fn calculate_path_divergence(&self, paths: &[Vec<String>]) -> f64 {
        if paths.len() < 2 {
            return 1.0; // No divergence
        }

        let mut common_prefix_length = 0;
        let min_length = paths.iter().map(|p| p.len()).min().unwrap_or(0);

        for i in 0..min_length {
            let first_node = &paths[0][i];
            if paths.iter().all(|path| path.get(i) == Some(first_node)) {
                common_prefix_length += 1;
            } else {
                break;
            }
        }

        if min_length == 0 {
            0.0
        } else {
            common_prefix_length as f64 / min_length as f64
        }
    }

    /// Track sanitization effectiveness along a path
    pub fn track_sanitization_along_path(
        &self,
        path: &[String],
        graph: &DataFlowGraph,
    ) -> SanitizationInfo {
        let mut sanitization_points = Vec::new();
        let mut total_effectiveness = 0.0;
        let mut sanitizer_count = 0;

        for node_id in path {
            if graph.sanitizers.contains(node_id) {
                sanitization_points.push(node_id.clone());
                // TODO: Get actual effectiveness from sanitizer metadata
                total_effectiveness += 0.8; // Placeholder
                sanitizer_count += 1;
            }
        }

        let average_effectiveness = if sanitizer_count > 0 {
            total_effectiveness / sanitizer_count as f64
        } else {
            0.0
        };

        SanitizationInfo {
            points: sanitization_points,
            average_effectiveness,
            total_count: sanitizer_count,
        }
    }

    /// Calculate confidence adjustment based on type consistency
    pub fn calculate_type_consistency_score(&self, path: &[String]) -> f64 {
        // This would track type information along the path
        // and reduce confidence for type inconsistencies
        // For now, return a default high score
        0.9
    }

    /// Track data transformations along the path
    pub fn track_data_transformations(&self, path: &[String]) -> Vec<DataTransformation> {
        let mut transformations = Vec::new();

        for node_id in path {
            // TODO: Analyze the node to determine what data transformations occur
            // This would involve looking at function calls, assignments, etc.
            transformations.push(DataTransformation {
                node_id: node_id.clone(),
                transformation_type: TransformationType::Unknown,
                confidence: 0.8,
            });
        }

        transformations
    }

    /// Estimate false positive likelihood
    pub fn estimate_false_positive_likelihood(
        &self,
        path: &[String],
        taint_level: &TaintLevel,
    ) -> f64 {
        let path_length_factor = match path.len() {
            1..=3 => 0.1,  // Short paths are less likely to be false positives
            4..=6 => 0.2,  // Medium paths have moderate false positive risk
            7..=10 => 0.4, // Long paths have higher false positive risk
            _ => 0.6,      // Very long paths are often false positives
        };

        let taint_strength_factor = 1.0 - taint_level.score();

        (path_length_factor + taint_strength_factor) / 2.0
    }
}

#[derive(Debug, Clone)]
pub enum ControlFlowType {
    Sequential,
    Conditional,
    Loop,
    FunctionCall,
    Exception,
    Async,
}

#[derive(Debug, Clone)]
pub struct SanitizationInfo {
    pub points: Vec<String>,
    pub average_effectiveness: f64,
    pub total_count: usize,
}

#[derive(Debug, Clone)]
pub struct DataTransformation {
    pub node_id: String,
    pub transformation_type: TransformationType,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub enum TransformationType {
    Encoding,    // HTML encoding, URL encoding, etc.
    Validation,  // Input validation
    Filtering,   // Data filtering
    Conversion,  // Type conversion
    Aggregation, // Data aggregation
    Unknown,     // Unknown transformation
}

impl Default for ConfidenceWeights {
    fn default() -> Self {
        Self {
            path_length: 0.3,
            taint_strength: 0.4,
            sanitization_presence: 0.2,
            control_flow_complexity: 0.1,
            type_consistency: 0.1,
        }
    }
}

impl Default for PathTracker {
    fn default() -> Self {
        Self::new()
    }
}
