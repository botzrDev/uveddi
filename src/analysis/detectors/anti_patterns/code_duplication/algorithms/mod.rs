//! Detection algorithms for code duplication
//!
//! This module contains different algorithms for detecting code clones:
//! - Token-based detection using rolling hashes
//! - AST-based structural comparison
//! - Semantic analysis using control flow graphs

pub mod ast;
pub mod ast_based;
pub mod semantic_based;
pub mod token_based;

pub use ast_based::AstBasedDetector;
pub use semantic_based::SemanticDetector;
pub use token_based::TokenBasedDetector;

use crate::analysis::AnalysisError;
use super::types::{CodeBlock, ClonePair};

/// Trait for clone detection algorithms
pub trait CloneDetectionAlgorithm: Send + Sync {
    /// Returns the name of this algorithm
    fn name(&self) -> &'static str;

    /// Detects clones between two code blocks
    fn detect_clones(&self, block1: &CodeBlock, block2: &CodeBlock) -> Result<Option<ClonePair>, AnalysisError>;

    /// Detects clones within a collection of code blocks
    fn detect_all_clones(&self, blocks: &[CodeBlock]) -> Result<Vec<ClonePair>, AnalysisError> {
        let mut clone_pairs = Vec::new();

        for i in 0..blocks.len() {
            for j in (i + 1)..blocks.len() {
                if let Some(pair) = self.detect_clones(&blocks[i], &blocks[j])? {
                    clone_pairs.push(pair);
                }
            }
        }

        Ok(clone_pairs)
    }

    /// Returns whether this algorithm supports the given language
    fn supports_language(&self, language: &crate::ast::tree_sitter_impl::SourceLanguage) -> bool;

    /// Returns the confidence level of this algorithm (0.0 to 1.0)
    fn confidence_level(&self) -> f64;
}