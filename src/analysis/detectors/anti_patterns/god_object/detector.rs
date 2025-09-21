//! Core detector trait and pattern types for God Object detection

use crate::analysis::AnalysisError;
use crate::ast::tree_sitter::{Node, QueryCursor};
use crate::ast::tree_sitter_impl::{ParsedFile, SourceLanguage};
use crate::database::models::ArchitecturalIssue;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Pattern detection results for exclusion from God Object detection
#[derive(Debug, Clone)]
pub enum DetectedPattern {
    Builder {
        builder_methods: Vec<String>,
        build_method: Option<String>,
    },
    Factory {
        factory_methods: Vec<String>,
        product_types: Vec<String>,
    },
    Dto {
        framework: String,
        field_ratio: f64,
    },
    FrameworkController {
        framework: String,
        base_class: Option<String>,
    },
    GeneratedCode {
        generator: String,
        markers: Vec<String>,
    },
}

/// Complexity metrics for behavioral analysis
#[derive(Debug, Clone, Default)]
pub struct ComplexityMetrics {
    pub method_count: usize,
    pub field_count: usize,
    pub trivial_methods: usize,
    pub complex_methods: usize,
    pub lcom4_score: Option<u32>,
    pub dependency_count: usize,
}

/// Core trait for God Object detection implementations
pub trait GodObjectDetector {
    /// Detect God Object violations in a parsed file
    fn detect(&self, parsed_file: &ParsedFile) -> Result<Vec<ArchitecturalIssue>, AnalysisError>;

    /// Calculate complexity metrics for a given node
    fn calculate_metrics(
        &self,
        parsed_file: &ParsedFile,
        node: Node,
    ) -> Result<ComplexityMetrics, AnalysisError>;

    /// Check if this detector applies to the given language
    fn applies_to_language(&self, language: &SourceLanguage) -> bool;

    /// Detect design patterns that should exclude from God Object detection
    fn detect_patterns(
        &self,
        parsed_file: &ParsedFile,
        node: Node,
        class_name: &str,
    ) -> Option<DetectedPattern>;

    /// Score the severity of a detected God Object
    fn score_severity(
        &self,
        metrics: &ComplexityMetrics,
        language: SourceLanguage,
    ) -> Option<String>;
}
