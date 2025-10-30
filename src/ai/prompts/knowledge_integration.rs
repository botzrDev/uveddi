//! Integration layer between enhanced templates and existing AI system
//!
//! This module provides seamless integration between the enhanced prompt template system
//! and Uveddi's existing AI explanation system, enabling knowledge-enhanced AI responses
//! while maintaining backward compatibility.

use crate::ai::engine::AiAnalysisEngine;
use crate::ai::prompts::enhanced_templates::{
    EnhancedPromptTemplateSystem, AnalysisType, PromptGenerationError
};
use crate::ai::knowledge::context_selection::{AnalysisContext, SelectedContext};
use crate::database::models::ArchitecturalIssue;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

/// Integration layer for knowledge-enhanced AI prompts
pub struct KnowledgeEnhancedAiAnalysisEngine {
    /// Base AI engine
    ai_engine: AiAnalysisEngine,
    /// Enhanced prompt template system
    template_system: EnhancedPromptTemplateSystem,
    /// Integration configuration
    config: IntegrationConfig,
    /// Performance metrics
    metrics: IntegrationMetrics,
}

/// Configuration for AI-template integration
#[derive(Debug, Clone)]
pub struct IntegrationConfig {
    /// Enable knowledge enhancement
    pub enable_enhancement: bool,
    /// Fallback to basic templates on error
    pub fallback_on_error: bool,
    /// Maximum enhancement processing time
    pub max_enhancement_time: Duration,
    /// Enable response quality tracking
    pub enable_quality_tracking: bool,
}

/// Metrics for integration performance
#[derive(Debug, Clone)]
pub struct IntegrationMetrics {
    /// Total enhanced explanations generated
    pub enhanced_explanations: u64,
    /// Total fallback explanations used
    pub fallback_explanations: u64,
    /// Average enhancement processing time
    pub avg_enhancement_time: Duration,
    /// Success rate for enhancements
    pub enhancement_success_rate: f32,
    /// Quality improvement scores
    pub quality_improvements: Vec<f32>,
}

/// Enhanced AI response with knowledge context
#[derive(Debug, Clone, Serialize)]
pub struct EnhancedAIResponse {
    /// AI-generated explanation
    pub explanation: String,
    /// Knowledge context used for enhancement
    pub knowledge_context: SelectedContext,
    /// Confidence score from AI
    pub confidence_score: f32,
    /// Token usage breakdown
    pub token_usage: crate::ai::prompts::enhanced_templates::DetailedTokenUsage,
    /// Enhancement metadata
    pub enhancement_metadata: ResponseEnhancementMetadata,
    /// Response quality metrics
    pub quality_metrics: ResponseQualityMetrics,
}

/// Metadata about the enhancement process
#[derive(Debug, Clone, Serialize)]
pub struct ResponseEnhancementMetadata {
    /// Number of patterns used in enhancement
    pub patterns_used: usize,
    /// Whether language-specific context was applied
    pub language_specific: bool,
    /// Whether framework guidance was included
    pub framework_guidance: bool,
    /// Template version used
    pub template_version: String,
    /// Enhancement processing time
    pub processing_time: Duration,
    /// Whether enhancement was successful
    pub enhancement_successful: bool,
}

/// Quality metrics for response evaluation
#[derive(Debug, Clone, Serialize)]
pub struct ResponseQualityMetrics {
    /// Estimated comprehensiveness (0.0-1.0)
    pub comprehensiveness: f32,
    /// Estimated actionability (0.0-1.0)
    pub actionability: f32,
    /// Context relevance score (0.0-1.0)
    pub context_relevance: f32,
    /// Technical accuracy estimate (0.0-1.0)
    pub technical_accuracy: f32,
}

/// Error types for integration operations
#[derive(Debug, thiserror::Error)]
pub enum IntegrationError {
    #[error("AI engine error: {0}")]
    AiAnalysisEngineError(String),
    #[error("Template generation error: {0}")]
    TemplateGenerationError(#[from] PromptGenerationError),
    #[error("Enhancement timeout: exceeded {0:?}")]
    EnhancementTimeout(Duration),
    #[error("Context preparation error: {0}")]
    ContextPreparationError(String),
    #[error("Integration configuration error: {0}")]
    ConfigurationError(String),
}

impl KnowledgeEnhancedAiAnalysisEngine {
    /// Create new knowledge-enhanced AI engine
    pub fn new(
        ai_engine: AiAnalysisEngine,
        template_system: EnhancedPromptTemplateSystem,
        config: IntegrationConfig,
    ) -> Self {
        Self {
            ai_engine,
            template_system,
            config,
            metrics: IntegrationMetrics::new(),
        }
    }

    /// Generate enhanced AI explanation with knowledge integration
    pub async fn generate_enhanced_explanation(
        &mut self,
        issue: &ArchitecturalIssue,
        context: &AnalysisContext,
    ) -> Result<EnhancedAIResponse, IntegrationError> {
        let start_time = Instant::now();

        // Early return if enhancement is disabled
        if !self.config.enable_enhancement {
            return self.generate_fallback_explanation(issue, context).await;
        }

        // Try enhanced generation with timeout
        let enhancement_result = tokio::time::timeout(
            self.config.max_enhancement_time,
            self.try_enhanced_generation(issue, context)
        ).await;

        match enhancement_result {
            Ok(Ok(response)) => {
                self.metrics.record_successful_enhancement(start_time.elapsed());
                Ok(response)
            },
            Ok(Err(e)) if self.config.fallback_on_error => {
                tracing::warn!("Enhancement failed, falling back to basic template: {}", e);
                self.metrics.record_fallback_used();
                self.generate_fallback_explanation(issue, context).await
            },
            Ok(Err(e)) => Err(e),
            Err(_) => {
                tracing::warn!("Enhancement timed out, falling back to basic template");
                if self.config.fallback_on_error {
                    self.metrics.record_fallback_used();
                    self.generate_fallback_explanation(issue, context).await
                } else {
                    Err(IntegrationError::EnhancementTimeout(self.config.max_enhancement_time))
                }
            }
        }
    }

    /// Try enhanced generation with knowledge integration
    async fn try_enhanced_generation(
        &mut self,
        issue: &ArchitecturalIssue,
        context: &AnalysisContext,
    ) -> Result<EnhancedAIResponse, IntegrationError> {
        // 1. Determine analysis type from issue
        let analysis_type = self.determine_analysis_type(issue);

        // 2. Generate enhanced prompt with knowledge
        let enhanced_prompt = self.template_system
            .generate_enhanced_prompt(context, analysis_type)
            .await?;

        // 3. Prepare final prompt with issue-specific context
        let final_prompt = self.prepare_final_prompt(&enhanced_prompt.content, issue, context)?;

        // 4. Call AI engine with enhanced prompt
        // TODO: For alpha release, return a placeholder response since analyze_issue expects ArchitecturalIssue
        let ai_response_text = "Knowledge-enhanced AI response (placeholder for alpha release)".to_string();

        // 5. Build enhanced response with metadata
        let knowledge_context = enhanced_prompt.knowledge_context.clone();
        let response = EnhancedAIResponse {
            explanation: ai_response_text.clone(),
            knowledge_context: knowledge_context.clone(),
            confidence_score: 0.85, // Placeholder confidence score
            token_usage: enhanced_prompt.token_usage,
            enhancement_metadata: ResponseEnhancementMetadata {
                patterns_used: knowledge_context.patterns.len(),
                language_specific: knowledge_context.language_context.is_some(),
                framework_guidance: !knowledge_context.framework_guidance.is_empty(),
                template_version: enhanced_prompt.metadata.template_version,
                processing_time: Duration::from_millis(0), // TODO: measure actual time
                enhancement_successful: true,
            },
            quality_metrics: self.estimate_response_quality(&ai_response_text, &knowledge_context),
        };

        Ok(response)
    }

    /// Generate fallback explanation using basic templates
    async fn generate_fallback_explanation(
        &mut self,
        issue: &ArchitecturalIssue,
        context: &AnalysisContext,
    ) -> Result<EnhancedAIResponse, IntegrationError> {
        // Use the existing basic prompt template
        let basic_prompt = crate::ai::prompts::prompt_templates::for_issue(issue);

        // TODO: For alpha release, return a placeholder response
        let ai_response_text = "Basic AI response (placeholder for alpha release)".to_string();

        // Create minimal enhanced response structure
        Ok(EnhancedAIResponse {
            explanation: ai_response_text.clone(),
            knowledge_context: SelectedContext::empty(), // Empty context for fallback
            confidence_score: 0.75, // Placeholder confidence score for fallback
            token_usage: crate::ai::prompts::enhanced_templates::DetailedTokenUsage {
                total_tokens: self.estimate_tokens(&basic_prompt),
                base_template_tokens: self.estimate_tokens(&basic_prompt),
                knowledge_injection_tokens: 0,
                injection_breakdown: HashMap::new(),
                efficiency_score: 0.0,
            },
            enhancement_metadata: ResponseEnhancementMetadata {
                patterns_used: 0,
                language_specific: false,
                framework_guidance: false,
                template_version: "fallback-1.0.0".to_string(),
                processing_time: Duration::from_millis(0),
                enhancement_successful: false,
            },
            quality_metrics: ResponseQualityMetrics {
                comprehensiveness: 0.6, // Estimated lower quality for basic template
                actionability: 0.7,
                context_relevance: 0.5,
                technical_accuracy: 0.8,
            },
        })
    }

    /// Determine analysis type from architectural issue
    fn determine_analysis_type(&self, issue: &ArchitecturalIssue) -> AnalysisType {
        match issue.anti_pattern_type_id {
            1 => AnalysisType::GodObjectAnalysis,
            2 => AnalysisType::TightCouplingAnalysis,
            3 => AnalysisType::DeadCodeAnalysis,
            4 => AnalysisType::PerformanceAnalysis,
            5 => AnalysisType::SecurityAnalysis,
            _ => AnalysisType::GeneralAnalysis,
        }
    }

    /// Prepare final prompt by injecting issue-specific context
    fn prepare_final_prompt(
        &self,
        enhanced_template: &str,
        issue: &ArchitecturalIssue,
        context: &AnalysisContext,
    ) -> Result<String, IntegrationError> {
        let mut final_prompt = enhanced_template.to_string();

        // Replace context placeholders with actual values
        final_prompt = final_prompt.replace(
            "{language}",
            &format!("{:?}", context.language)
        );

        final_prompt = final_prompt.replace(
            "{detected_issues}",
            &issue.description
        );

        // Add code snippet if available
        if let Some(code_snippet) = &issue.code_snippet {
            final_prompt.push_str(&format!(
                "\n\n## Code Under Analysis\n```\n{}\n```\n",
                code_snippet
            ));
        }

        // Add file and location context
        final_prompt.push_str(&format!(
            "\n\n## Location Context\n**File:** {}\n**Line:** {}\n",
            issue.file_path,
            issue.start_line.unwrap_or(0)
        ));

        Ok(final_prompt)
    }

    /// Estimate response quality based on knowledge context
    fn estimate_response_quality(
        &self,
        response: &str,
        knowledge_context: &SelectedContext,
    ) -> ResponseQualityMetrics {
        // Simple heuristic-based quality estimation
        let has_patterns = !knowledge_context.patterns.is_empty();
        let has_language_context = knowledge_context.language_context.is_some();
        let has_framework_guidance = !knowledge_context.framework_guidance.is_empty();
        let response_length = response.len();

        let comprehensiveness = if has_patterns && response_length > 500 {
            0.9
        } else if has_patterns {
            0.7
        } else {
            0.5
        };

        let actionability = if has_patterns && response_length > 300 {
            0.85
        } else {
            0.6
        };

        let context_relevance = if has_language_context && has_patterns {
            0.9
        } else if has_patterns {
            0.75
        } else {
            0.5
        };

        let technical_accuracy = if has_framework_guidance {
            0.9
        } else if has_patterns {
            0.8
        } else {
            0.7
        };

        ResponseQualityMetrics {
            comprehensiveness,
            actionability,
            context_relevance,
            technical_accuracy,
        }
    }

    /// Estimate token count (simple approximation)
    fn estimate_tokens(&self, text: &str) -> usize {
        // Rough approximation: 1 token per 4 characters
        (text.len() + 3) / 4
    }

    /// Get integration metrics
    pub fn get_metrics(&self) -> &IntegrationMetrics {
        &self.metrics
    }

    /// Update integration configuration
    pub fn update_config(&mut self, config: IntegrationConfig) {
        self.config = config;
    }
}

impl IntegrationConfig {
    /// Create default integration configuration
    pub fn default() -> Self {
        Self {
            enable_enhancement: true,
            fallback_on_error: true,
            max_enhancement_time: Duration::from_millis(5000), // 5 second timeout
            enable_quality_tracking: true,
        }
    }

    /// Create production-optimized configuration
    pub fn production() -> Self {
        Self {
            enable_enhancement: true,
            fallback_on_error: true,
            max_enhancement_time: Duration::from_millis(3000), // Shorter timeout for prod
            enable_quality_tracking: false, // Disable for performance
        }
    }

    /// Create development configuration with extended timeouts
    pub fn development() -> Self {
        Self {
            enable_enhancement: true,
            fallback_on_error: false, // Fail fast in development
            max_enhancement_time: Duration::from_millis(10000), // Longer timeout for debugging
            enable_quality_tracking: true,
        }
    }
}

impl IntegrationMetrics {
    fn new() -> Self {
        Self {
            enhanced_explanations: 0,
            fallback_explanations: 0,
            avg_enhancement_time: Duration::from_millis(0),
            enhancement_success_rate: 0.0,
            quality_improvements: Vec::new(),
        }
    }

    fn record_successful_enhancement(&mut self, processing_time: Duration) {
        self.enhanced_explanations += 1;
        self.update_average_time(processing_time);
        self.update_success_rate();
    }

    fn record_fallback_used(&mut self) {
        self.fallback_explanations += 1;
        self.update_success_rate();
    }

    fn update_average_time(&mut self, new_time: Duration) {
        let current_avg_ms = self.avg_enhancement_time.as_millis() as f64;
        let new_time_ms = new_time.as_millis() as f64;
        let count = self.enhanced_explanations as f64;
        
        let new_avg_ms = if count <= 1.0 {
            new_time_ms
        } else {
            (current_avg_ms * (count - 1.0) + new_time_ms) / count
        };
        
        self.avg_enhancement_time = Duration::from_millis(new_avg_ms as u64);
    }

    fn update_success_rate(&mut self) {
        let total = self.enhanced_explanations + self.fallback_explanations;
        if total > 0 {
            self.enhancement_success_rate = self.enhanced_explanations as f32 / total as f32;
        }
    }

    /// Get success rate percentage
    pub fn success_rate_percentage(&self) -> f32 {
        self.enhancement_success_rate * 100.0
    }

    /// Get total explanations generated
    pub fn total_explanations(&self) -> u64 {
        self.enhanced_explanations + self.fallback_explanations
    }
}

impl SelectedContext {
    /// Create empty context for fallback scenarios
    fn empty() -> Self {
        // This is a placeholder - actual implementation would depend on SelectedContext structure
        // For now, we'll need to check the actual definition in context_selection.rs
        Self {
            patterns: Vec::new(),
            language_context: None,
            framework_guidance: Vec::new(),
            metadata: crate::ai::knowledge::context_selection::ContextMetadata::default(),
        }
    }
}

// Placeholder for AI response structure - this should match the actual AiAnalysisEngine response
#[derive(Debug)]
pub struct AIResponse {
    pub explanation: String,
    pub confidence_score: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration_config_defaults() {
        let config = IntegrationConfig::default();
        assert!(config.enable_enhancement);
        assert!(config.fallback_on_error);
        assert!(config.enable_quality_tracking);
        assert_eq!(config.max_enhancement_time, Duration::from_millis(5000));
    }

    #[test]
    fn test_metrics_initialization() {
        let metrics = IntegrationMetrics::new();
        assert_eq!(metrics.enhanced_explanations, 0);
        assert_eq!(metrics.fallback_explanations, 0);
        assert_eq!(metrics.enhancement_success_rate, 0.0);
    }

    #[test]
    fn test_success_rate_calculation() {
        let mut metrics = IntegrationMetrics::new();
        
        metrics.record_successful_enhancement(Duration::from_millis(100));
        assert_eq!(metrics.success_rate_percentage(), 100.0);
        
        metrics.record_fallback_used();
        assert_eq!(metrics.success_rate_percentage(), 50.0);
        
        metrics.record_successful_enhancement(Duration::from_millis(150));
        assert!((metrics.success_rate_percentage() - 66.67).abs() < 0.1);
    }

    #[test]
    fn test_analysis_type_determination() {
        // This test would need actual ArchitecturalIssue instances
        // Placeholder for when the integration is complete
    }
}