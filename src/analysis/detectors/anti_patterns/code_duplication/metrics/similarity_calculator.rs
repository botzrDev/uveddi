//! Similarity calculation utilities

use super::{AstMetrics, DuplicationMetrics, SemanticMetrics, TokenMetrics};
use crate::analysis::detectors::anti_patterns::code_duplication::types::CodeBlock;
use std::collections::HashSet;

/// Calculates various similarity metrics between code blocks
pub struct SimilarityCalculator {
    /// Weights for combining different similarity metrics
    pub weights: SimilarityWeights,
}

/// Weights for combining different types of similarity
#[derive(Debug, Clone)]
pub struct SimilarityWeights {
    pub token_weight: f64,
    pub ast_weight: f64,
    pub semantic_weight: f64,
}

impl Default for SimilarityWeights {
    fn default() -> Self {
        Self {
            token_weight: 0.4,
            ast_weight: 0.4,
            semantic_weight: 0.2,
        }
    }
}

impl SimilarityCalculator {
    /// Creates a new similarity calculator with default weights
    pub fn new() -> Self {
        Self {
            weights: SimilarityWeights::default(),
        }
    }

    /// Creates a calculator with custom weights
    pub fn with_weights(weights: SimilarityWeights) -> Self {
        Self { weights }
    }

    /// Calculates comprehensive similarity metrics between two code blocks
    pub fn calculate_similarity(
        &self,
        block1: &CodeBlock,
        block2: &CodeBlock,
    ) -> DuplicationMetrics {
        let token_metrics = self.calculate_token_similarity(block1, block2);
        let ast_metrics = self.calculate_ast_similarity(block1, block2);
        let semantic_metrics = self.calculate_semantic_similarity(block1, block2);

        let combined_score =
            self.combine_similarities(&token_metrics, &ast_metrics, semantic_metrics.as_ref());
        let confidence =
            self.calculate_confidence(&token_metrics, &ast_metrics, semantic_metrics.as_ref());

        DuplicationMetrics {
            token_metrics,
            ast_metrics,
            semantic_metrics,
            combined_score,
            confidence,
        }
    }

    /// Calculates token-level similarity metrics
    pub fn calculate_token_similarity(
        &self,
        block1: &CodeBlock,
        block2: &CodeBlock,
    ) -> TokenMetrics {
        let tokens1: HashSet<_> = block1.normalized_tokens.iter().collect();
        let tokens2: HashSet<_> = block2.normalized_tokens.iter().collect();

        // Jaccard similarity
        let intersection = tokens1.intersection(&tokens2).count();
        let union = tokens1.union(&tokens2).count();
        let jaccard_similarity = if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        };

        // Cosine similarity
        let cosine_similarity =
            self.calculate_cosine_similarity(&block1.normalized_tokens, &block2.normalized_tokens);

        // Edit distance
        let edit_distance =
            self.calculate_edit_distance(&block1.normalized_tokens, &block2.normalized_tokens);
        let max_length = block1
            .normalized_tokens
            .len()
            .max(block2.normalized_tokens.len());
        let normalized_edit_distance = if max_length == 0 {
            0.0
        } else {
            1.0 - (edit_distance as f64 / max_length as f64)
        };

        // Token ratio
        let min_length = block1
            .normalized_tokens
            .len()
            .min(block2.normalized_tokens.len());
        let token_ratio = if max_length == 0 {
            1.0
        } else {
            min_length as f64 / max_length as f64
        };

        // Matching sequences (simplified)
        let matching_sequences =
            self.count_matching_sequences(&block1.normalized_tokens, &block2.normalized_tokens);

        TokenMetrics {
            jaccard_similarity,
            cosine_similarity,
            normalized_edit_distance,
            matching_sequences,
            token_ratio,
        }
    }

    /// Calculates AST-level similarity metrics
    pub fn calculate_ast_similarity(&self, _block1: &CodeBlock, _block2: &CodeBlock) -> AstMetrics {
        // Simplified AST metrics - real implementation would use actual AST analysis
        AstMetrics {
            structural_similarity: 0.8,         // Placeholder
            tree_edit_distance: 0.2,            // Placeholder
            type_distribution_similarity: 0.85, // Placeholder
            depth_similarity: 0.9,              // Placeholder
        }
    }

    /// Calculates semantic similarity metrics
    pub fn calculate_semantic_similarity(
        &self,
        block1: &CodeBlock,
        block2: &CodeBlock,
    ) -> Option<SemanticMetrics> {
        // Only calculate if both blocks have semantic features
        if block1.cfg.is_some() && block2.cfg.is_some() {
            Some(SemanticMetrics {
                cfg_similarity: 0.75,      // Placeholder
                data_flow_similarity: 0.7, // Placeholder
                feature_similarity: 0.8,   // Placeholder
                behavioral_score: 0.65,    // Placeholder
            })
        } else {
            None
        }
    }

    /// Combines different similarity scores into a single metric
    fn combine_similarities(
        &self,
        token: &TokenMetrics,
        ast: &AstMetrics,
        semantic: Option<&SemanticMetrics>,
    ) -> f64 {
        let token_score =
            (token.jaccard_similarity + token.cosine_similarity + token.normalized_edit_distance)
                / 3.0;
        let ast_score =
            (ast.structural_similarity + ast.type_distribution_similarity + ast.depth_similarity)
                / 3.0;

        let semantic_score = semantic
            .map(|s| (s.cfg_similarity + s.feature_similarity + s.behavioral_score) / 3.0)
            .unwrap_or(0.0);

        let total_weight = self.weights.token_weight
            + self.weights.ast_weight
            + if semantic.is_some() {
                self.weights.semantic_weight
            } else {
                0.0
            };

        if total_weight == 0.0 {
            return 0.0;
        }

        let mut combined =
            token_score * self.weights.token_weight + ast_score * self.weights.ast_weight;

        if semantic.is_some() {
            combined += semantic_score * self.weights.semantic_weight;
        }

        combined / total_weight
    }

    /// Calculates confidence in the similarity measurement
    fn calculate_confidence(
        &self,
        token: &TokenMetrics,
        ast: &AstMetrics,
        semantic: Option<&SemanticMetrics>,
    ) -> f64 {
        let mut confidence_factors = vec![
            token.jaccard_similarity,
            token.cosine_similarity,
            ast.structural_similarity,
            ast.type_distribution_similarity,
        ];

        if let Some(sem) = semantic {
            confidence_factors.push(sem.cfg_similarity);
            confidence_factors.push(sem.feature_similarity);
        }

        // Calculate variance to determine confidence
        let mean = confidence_factors.iter().sum::<f64>() / confidence_factors.len() as f64;
        let variance = confidence_factors
            .iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>()
            / confidence_factors.len() as f64;

        // Lower variance = higher confidence
        let confidence = 1.0 - variance.min(1.0);

        // Boost confidence if multiple metrics agree
        if confidence_factors.iter().all(|&x| x > 0.7) {
            (confidence + 0.2).min(1.0)
        } else {
            confidence
        }
    }

    /// Calculates cosine similarity between two token vectors
    fn calculate_cosine_similarity(&self, tokens1: &[String], tokens2: &[String]) -> f64 {
        let vocab: HashSet<_> = tokens1.iter().chain(tokens2.iter()).collect();

        if vocab.is_empty() {
            return 1.0;
        }

        let mut vec1 = Vec::new();
        let mut vec2 = Vec::new();

        for token in &vocab {
            let count1 = tokens1.iter().filter(|t| t == token).count() as f64;
            let count2 = tokens2.iter().filter(|t| t == token).count() as f64;
            vec1.push(count1);
            vec2.push(count2);
        }

        let dot_product: f64 = vec1.iter().zip(&vec2).map(|(a, b)| a * b).sum();
        let norm1: f64 = vec1.iter().map(|x| x * x).sum::<f64>().sqrt();
        let norm2: f64 = vec2.iter().map(|x| x * x).sum::<f64>().sqrt();

        if norm1 == 0.0 || norm2 == 0.0 {
            0.0
        } else {
            dot_product / (norm1 * norm2)
        }
    }

    /// Calculates edit distance between two token sequences
    fn calculate_edit_distance(&self, tokens1: &[String], tokens2: &[String]) -> usize {
        let len1 = tokens1.len();
        let len2 = tokens2.len();

        if len1 == 0 {
            return len2;
        }
        if len2 == 0 {
            return len1;
        }

        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

        // Initialize first row and column
        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }

        // Fill the matrix
        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if tokens1[i - 1] == tokens2[j - 1] {
                    0
                } else {
                    1
                };
                matrix[i][j] = (matrix[i - 1][j] + 1)
                    .min(matrix[i][j - 1] + 1)
                    .min(matrix[i - 1][j - 1] + cost);
            }
        }

        matrix[len1][len2]
    }

    /// Counts matching subsequences between two token sequences
    fn count_matching_sequences(&self, tokens1: &[String], tokens2: &[String]) -> usize {
        let min_sequence_length = 3;
        let mut count = 0;

        for i in 0..=tokens1.len().saturating_sub(min_sequence_length) {
            for j in 0..=tokens2.len().saturating_sub(min_sequence_length) {
                let mut length = 0;
                while i + length < tokens1.len()
                    && j + length < tokens2.len()
                    && tokens1[i + length] == tokens2[j + length]
                {
                    length += 1;
                }
                if length >= min_sequence_length {
                    count += 1;
                }
            }
        }

        count
    }
}

impl Default for SimilarityCalculator {
    fn default() -> Self {
        Self::new()
    }
}
