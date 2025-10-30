//! Core Types for AI Knowledge Library Integration
//!
//! This module defines the data structures and types needed for integrating
//! the AI Knowledge Library with the Analysis Engine.

use crate::ai::knowledge::schema::{EffortLevel, ImpactLevel, PatternKnowledge, SourceLanguage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Analysis context for intelligent pattern selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisContext {
    /// Programming language being analyzed
    pub language: SourceLanguage,
    /// Detected anti-patterns and issues
    pub detected_patterns: Vec<String>,
    /// Detected frameworks and libraries
    pub frameworks: Vec<String>,
    /// Complexity metrics for the code
    pub complexity_metrics: ComplexityMetrics,
    /// File context for the analysis
    pub file_context: Option<String>,
    /// Surrounding components from dependency graph
    pub surrounding_components: Vec<String>,
}

/// Complexity metrics for codebase analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityMetrics {
    /// Cyclomatic complexity average
    pub cyclomatic_complexity: usize,
    /// Cognitive complexity average
    pub cognitive_complexity: usize,
    /// Lines of code count
    pub lines_of_code: usize,
    /// Number of methods/functions
    pub number_of_methods: usize,
}

/// Selected knowledge context for AI prompt building
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeContext {
    /// Selected patterns with relevance information
    pub selected_patterns: Vec<PatternKnowledge>,
    /// Overall relevance score for the context
    pub relevance_score: f64,
    /// Time taken for selection in milliseconds
    pub selection_time_ms: u64,
    /// Knowledge library version used
    pub library_version: String,
    /// Total token count for the selected context
    pub token_count: usize,
}

/// Context selector for intelligent knowledge selection
pub struct ContextSelector {
    /// Reference to the knowledge library
    knowledge_library: std::sync::Arc<crate::ai::knowledge::KnowledgeLibrary>,
    /// Maximum patterns to select
    max_patterns: usize,
    /// Minimum relevance threshold
    relevance_threshold: f64,
}

impl ContextSelector {
    /// Create a new context selector
    pub fn new(
        knowledge_library: std::sync::Arc<crate::ai::knowledge::KnowledgeLibrary>,
    ) -> Result<Self, crate::error::UveddiError> {
        Ok(Self {
            knowledge_library,
            max_patterns: 5,
            relevance_threshold: 0.3,
        })
    }

    /// Select relevant knowledge context based on analysis context
    pub async fn select_context(
        &self,
        analysis_context: &AnalysisContext,
    ) -> Result<KnowledgeContext, crate::error::UveddiError> {
        let start_time = std::time::Instant::now();

        // Get all patterns for the detected language
        let available_patterns = self
            .knowledge_library
            .get_language_patterns(&analysis_context.language);

        // Score patterns based on relevance
        let mut scored_patterns = Vec::new();

        for pattern in available_patterns {
            let score = self.calculate_relevance_score(pattern, analysis_context);

            if score >= self.relevance_threshold {
                scored_patterns.push((pattern.clone(), score));
            }
        }

        // Sort by relevance score (descending)
        scored_patterns.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Take top patterns up to max_patterns limit
        let selected_patterns: Vec<PatternKnowledge> = scored_patterns
            .into_iter()
            .take(self.max_patterns)
            .map(|(pattern, _score)| pattern)
            .collect();

        let selection_time_ms = start_time.elapsed().as_millis() as u64;

        // Calculate overall relevance score
        let relevance_score = if !selected_patterns.is_empty() {
            selected_patterns.len() as f64 / self.max_patterns as f64
        } else {
            0.0
        };

        // Estimate token count (rough estimation)
        let token_count = selected_patterns
            .iter()
            .map(|p| self.estimate_pattern_tokens(p))
            .sum();

        Ok(KnowledgeContext {
            selected_patterns,
            relevance_score,
            selection_time_ms,
            library_version: self.knowledge_library.metadata.content_version.clone(),
            token_count,
        })
    }

    /// Calculate relevance score for a pattern given the analysis context
    fn calculate_relevance_score(
        &self,
        pattern: &PatternKnowledge,
        context: &AnalysisContext,
    ) -> f64 {
        let mut score = 0.0;
        let mut factors = 0;

        // Check for direct pattern matches
        for detected_pattern in &context.detected_patterns {
            if pattern.id == *detected_pattern
                || pattern
                    .name
                    .to_lowercase()
                    .contains(&detected_pattern.to_lowercase())
            {
                score += 1.0;
                factors += 1;
            }
        }

        // Check for related patterns
        for detected_pattern in &context.detected_patterns {
            if pattern.related_patterns.contains(detected_pattern) {
                score += 0.8;
                factors += 1;
            }
        }

        // Check for framework relevance
        if !context.frameworks.is_empty() {
            for framework in &context.frameworks {
                if pattern
                    .tags
                    .iter()
                    .any(|tag| tag.to_lowercase().contains(&framework.to_lowercase()))
                {
                    score += 0.6;
                    factors += 1;
                    break;
                }
            }
        }

        // Consider pattern frequency and detection confidence
        score += pattern.frequency_score as f64 * 0.5;
        score += pattern.detection_confidence as f64 * 0.3;
        factors += 2;

        // Complexity-based relevance
        match context.complexity_metrics.cognitive_complexity {
            0..=5 => {
                // Simple code - prefer simple solutions
                if pattern
                    .solutions
                    .iter()
                    .any(|s| matches!(s.effort_level, EffortLevel::Trivial | EffortLevel::Low))
                {
                    score += 0.4;
                    factors += 1;
                }
            }
            6..=15 => {
                // Medium complexity - balanced solutions
                if pattern
                    .solutions
                    .iter()
                    .any(|s| matches!(s.effort_level, EffortLevel::Medium))
                {
                    score += 0.5;
                    factors += 1;
                }
            }
            _ => {
                // High complexity - comprehensive solutions needed
                if pattern
                    .solutions
                    .iter()
                    .any(|s| matches!(s.effort_level, EffortLevel::High | EffortLevel::Significant))
                {
                    score += 0.6;
                    factors += 1;
                }
            }
        }

        // Normalize score by number of factors
        if factors > 0 {
            score / factors as f64
        } else {
            0.0
        }
    }

    /// Estimate token count for a pattern (rough approximation)
    fn estimate_pattern_tokens(&self, pattern: &PatternKnowledge) -> usize {
        let mut tokens = 0;

        // Pattern name and definition
        tokens += pattern.name.len() / 4; // ~4 chars per token
        tokens += pattern.definition.as_str().len() / 4;

        // Symptoms
        tokens += pattern
            .symptoms
            .iter()
            .map(|s| s.as_str().len() / 4)
            .sum::<usize>();

        // Solutions (limited to first 2 for token budget)
        tokens += pattern
            .solutions
            .iter()
            .take(2)
            .map(|s| s.title.len() / 4 + s.implementation.as_str().len() / 4)
            .sum::<usize>();

        tokens.max(50) // Minimum estimate
    }
}

impl Default for ComplexityMetrics {
    fn default() -> Self {
        Self {
            cyclomatic_complexity: 1,
            cognitive_complexity: 1,
            lines_of_code: 0,
            number_of_methods: 1,
        }
    }
}
