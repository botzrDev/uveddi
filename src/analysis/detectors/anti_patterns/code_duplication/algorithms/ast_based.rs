//! AST-based clone detection using structural comparison

use super::{
    ast::{AstNode, AstParser, FeatureExtractor, Tokenizer},
    CloneDetectionAlgorithm,
};
use crate::analysis::detectors::anti_patterns::code_duplication::types::{
    ClonePair, CloneType, CodeBlock,
};
use crate::analysis::AnalysisError;
use crate::ast::tree_sitter_impl::SourceLanguage;

/// AST-based clone detector using structural tree comparison
pub struct AstBasedDetector {
    /// Minimum similarity threshold for structural comparison
    similarity_threshold: f64,
    /// Whether to ignore identifier names in comparison
    ignore_identifiers: bool,
    /// Whether to ignore literal values in comparison
    ignore_literals: bool,
    /// Maximum tree depth to analyze (performance limit)
    max_depth: usize,
    /// Tokenizer for parsing source code
    tokenizer: Tokenizer,
    /// Feature extractor for structural analysis
    feature_extractor: FeatureExtractor,
}

impl AstBasedDetector {
    /// Creates a new AST-based detector
    pub fn new(similarity_threshold: f64) -> Self {
        Self {
            similarity_threshold,
            ignore_identifiers: true,
            ignore_literals: true,
            max_depth: 50,
            tokenizer: Tokenizer::new(true, true),
            feature_extractor: FeatureExtractor::new(50),
        }
    }

    /// Creates a detector with custom settings
    pub fn with_settings(
        similarity_threshold: f64,
        ignore_identifiers: bool,
        ignore_literals: bool,
        max_depth: usize,
    ) -> Self {
        Self {
            similarity_threshold,
            ignore_identifiers,
            ignore_literals,
            max_depth,
            tokenizer: Tokenizer::new(ignore_identifiers, ignore_literals),
            feature_extractor: FeatureExtractor::new(max_depth),
        }
    }

    /// Parses source code and extracts AST structure
    pub fn parse_ast(
        &self,
        source: &str,
        _language: &SourceLanguage,
    ) -> Result<AstNode, AnalysisError> {
        let tokens = self.tokenizer.tokenize(source);
        let mut parser = AstParser::new(&tokens);
        parser.parse()
    }

    /// Computes structural similarity between two AST nodes
    pub fn compute_structural_similarity(&self, node1: &AstNode, node2: &AstNode) -> f64 {
        // Extract features from both nodes
        let features1 = self.feature_extractor.extract_features(node1);
        let features2 = self.feature_extractor.extract_features(node2);

        // Compare features
        features1.similarity(&features2)
    }

    /// Computes node-level similarity recursively
    fn compute_node_similarity(&self, node1: &AstNode, node2: &AstNode, depth: usize) -> f64 {
        if depth > self.max_depth {
            return 0.5; // Default similarity for deep nodes
        }

        // Compare node types
        if std::mem::discriminant(&node1.node_type) != std::mem::discriminant(&node2.node_type) {
            return 0.0;
        }

        // Compare node values based on settings
        let value_similarity =
            self.compute_value_similarity(&node1.value, &node2.value, &node1.node_type);

        if value_similarity == 0.0 {
            return 0.0;
        }

        // Compare children
        let child_similarity = if node1.children.is_empty() && node2.children.is_empty() {
            1.0
        } else {
            self.compute_children_similarity(&node1.children, &node2.children, depth + 1)
        };

        // Weighted average
        0.3 * value_similarity + 0.7 * child_similarity
    }

    /// Computes similarity between node values
    fn compute_value_similarity(
        &self,
        value1: &str,
        value2: &str,
        node_type: &super::ast::NodeType,
    ) -> f64 {
        use super::ast::NodeType;

        match node_type {
            NodeType::Keyword(_) => {
                if value1 == value2 {
                    1.0
                } else {
                    0.0
                }
            }
            NodeType::Identifier(_) => {
                if self.ignore_identifiers {
                    1.0
                } else {
                    if value1 == value2 {
                        1.0
                    } else {
                        0.0
                    }
                }
            }
            NodeType::Literal(_) => {
                if self.ignore_literals {
                    1.0
                } else {
                    if value1 == value2 {
                        1.0
                    } else {
                        0.0
                    }
                }
            }
            NodeType::Operator(_) => {
                if value1 == value2 {
                    1.0
                } else {
                    0.0
                }
            }
            _ => 1.0, // Other node types are considered similar by default
        }
    }

    /// Computes similarity between lists of child nodes
    fn compute_children_similarity(
        &self,
        children1: &[AstNode],
        children2: &[AstNode],
        depth: usize,
    ) -> f64 {
        if children1.is_empty() && children2.is_empty() {
            return 1.0;
        }

        let max_len = children1.len().max(children2.len());
        if max_len == 0 {
            return 1.0;
        }

        let mut total_similarity = 0.0;
        let min_len = children1.len().min(children2.len());

        // Compare aligned children
        for i in 0..min_len {
            total_similarity += self.compute_node_similarity(&children1[i], &children2[i], depth);
        }

        // Penalty for different lengths
        let length_penalty = (max_len - min_len) as f64 / max_len as f64;

        (total_similarity / max_len as f64) * (1.0 - length_penalty * 0.5)
    }
}

impl CloneDetectionAlgorithm for AstBasedDetector {
    fn name(&self) -> &'static str {
        "AST-based Structural"
    }

    fn detect_clones(
        &self,
        block1: &CodeBlock,
        block2: &CodeBlock,
    ) -> Result<Option<ClonePair>, AnalysisError> {
        // Skip if same file and overlapping regions
        if block1.file_path == block2.file_path
            && !(block1.end_line < block2.start_line || block2.end_line < block1.start_line)
        {
            return Ok(None);
        }

        // Parse ASTs
        let ast1 = self.parse_ast(&block1.source, &block1.language)?;
        let ast2 = self.parse_ast(&block2.source, &block2.language)?;

        // Compute structural similarity
        let similarity = self.compute_structural_similarity(&ast1, &ast2);

        if similarity < self.similarity_threshold {
            return Ok(None);
        }

        // Determine clone type based on similarity and settings
        let clone_type = if similarity >= 0.95 {
            if self.ignore_identifiers || self.ignore_literals {
                CloneType::Type2
            } else {
                CloneType::Type1
            }
        } else {
            CloneType::Type3
        };

        Ok(Some(ClonePair::new(
            block1.clone(),
            block2.clone(),
            similarity,
            clone_type,
            0, // No fingerprint concept for AST-based detection
        )))
    }

    fn supports_language(&self, _language: &SourceLanguage) -> bool {
        true // AST-based detection works for all languages with proper parsers
    }

    fn confidence_level(&self) -> f64 {
        0.9 // Very high confidence for structural comparison
    }
}
