//! Enhanced AI Prompt Templates with Knowledge Library Integration
//!
//! This module extends the existing prompt template system with comprehensive
//! knowledge library integration, enabling contextually rich, precision AI guidance
//! through dynamic template generation and intelligent knowledge injection.

use crate::ai::knowledge::{
    context_selection::{DynamicContextSelector, SelectedContext, AnalysisContext},
    schema::{PatternKnowledge, LanguageContext, FrameworkKnowledge, SourceLanguage},
    language_integration::EnhancedPattern,
};
use crate::database::models::ArchitecturalIssue;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

/// Enhanced prompt template system with knowledge library integration
pub struct EnhancedPromptTemplateSystem {
    /// Dynamic context selector for intelligent knowledge selection
    context_selector: DynamicContextSelector,
    /// Enhanced template registry
    template_registry: EnhancedTemplateRegistry,
    /// Template optimization configuration
    optimization_config: TemplateOptimizationConfig,
    /// Performance metrics
    metrics: PromptTemplateMetrics,
}

/// Registry for enhanced prompt templates
#[derive(Debug, Clone)]
pub struct EnhancedTemplateRegistry {
    /// Base templates for different analysis types
    base_templates: HashMap<AnalysisType, BaseTemplate>,
    /// Knowledge injection templates
    knowledge_templates: HashMap<KnowledgeType, KnowledgeTemplate>,
    /// Language-specific template variations
    language_templates: HashMap<SourceLanguage, LanguageTemplateSet>,
    /// Framework-specific templates
    framework_templates: HashMap<String, FrameworkTemplate>,
}

/// Types of analysis that require different template approaches
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AnalysisType {
    GodObjectAnalysis,
    TightCouplingAnalysis,
    DeadCodeAnalysis,
    PerformanceAnalysis,
    SecurityAnalysis,
    GeneralAnalysis,
}

/// Types of knowledge that can be injected into templates
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum KnowledgeType {
    PatternDefinitions,
    LanguageSpecifics,
    FrameworkGuidance,
    SolutionPatterns,
    BestPractices,
}

/// Enhanced template for specific analysis types
#[derive(Debug, Clone)]
pub struct BaseTemplate {
    /// Template identifier
    pub id: String,
    /// Template content with knowledge injection points
    pub content: String,
    /// Knowledge injection points
    pub injection_points: Vec<InjectionPoint>,
    /// Token budget allocation
    pub token_allocation: TokenAllocation,
    /// Template metadata
    pub metadata: TemplateMetadata,
}

/// Knowledge injection point in template
#[derive(Debug, Clone)]
pub struct InjectionPoint {
    /// Injection point identifier
    pub id: String,
    /// Injection type
    pub injection_type: InjectionType,
    /// Token budget for this injection
    pub token_budget: usize,
    /// Priority level (1-10)
    pub priority: u8,
    /// Conditional injection rules
    pub conditions: Vec<InjectionCondition>,
}

/// Types of knowledge injection
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InjectionType {
    /// Pattern definition and explanation
    PatternDefinition,
    /// Language-specific symptoms and manifestations
    LanguageSpecificSymptoms,
    /// Framework-specific guidance
    FrameworkGuidance,
    /// Solution patterns and refactoring steps
    SolutionPatterns,
    /// Related patterns and cross-references
    RelatedPatterns,
    /// Code examples and demonstrations
    CodeExamples,
    /// Best practices and recommendations
    BestPractices,
}

/// Conditions that determine whether knowledge should be injected
#[derive(Debug, Clone)]
pub enum InjectionCondition {
    /// Framework was detected in the codebase
    FrameworkDetected,
    /// Language matches specific criteria
    LanguageMatch(SourceLanguage),
    /// Pattern confidence above threshold
    ConfidenceThreshold(f32),
    /// Token budget available
    TokenBudgetAvailable(usize),
}

/// Token allocation strategy for templates
#[derive(Debug, Clone)]
pub struct TokenAllocation {
    /// Total token budget
    pub total_budget: usize,
    /// Reserved tokens for base template
    pub base_template_tokens: usize,
    /// Available tokens for knowledge injection
    pub knowledge_injection_tokens: usize,
    /// Token allocation per injection type
    pub injection_allocations: HashMap<InjectionType, usize>,
}

/// Template metadata for management and versioning
#[derive(Debug, Clone)]
pub struct TemplateMetadata {
    /// Template version
    pub version: String,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last update timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// Template author
    pub author: String,
    /// Template description
    pub description: String,
}

/// Configuration for template optimization
#[derive(Debug, Clone)]
pub struct TemplateOptimizationConfig {
    /// Enable token compression
    pub enable_token_compression: bool,
    /// Maximum template size in tokens
    pub max_template_tokens: usize,
    /// Prioritize high-confidence knowledge
    pub prioritize_confidence: bool,
    /// Enable adaptive optimization
    pub enable_adaptive_optimization: bool,
}

/// Performance metrics for template system
#[derive(Debug, Clone)]
pub struct PromptTemplateMetrics {
    /// Total prompts generated
    pub prompts_generated: u64,
    /// Average generation time
    pub avg_generation_time: Duration,
    /// Token utilization efficiency
    pub token_efficiency: f32,
    /// Knowledge injection success rate
    pub injection_success_rate: f32,
    /// User satisfaction scores
    pub satisfaction_scores: Vec<f32>,
}

/// Enhanced prompt with knowledge integration
#[derive(Debug, Clone)]
pub struct EnhancedPrompt {
    /// Final prompt content
    pub content: String,
    /// Knowledge context used
    pub knowledge_context: SelectedContext,
    /// Token usage breakdown
    pub token_usage: DetailedTokenUsage,
    /// Prompt metadata
    pub metadata: PromptMetadata,
    /// Quality metrics
    pub quality_metrics: PromptQualityMetrics,
}

/// Detailed token usage tracking
#[derive(Debug, Clone)]
pub struct DetailedTokenUsage {
    /// Total tokens used
    pub total_tokens: usize,
    /// Base template tokens
    pub base_template_tokens: usize,
    /// Knowledge injection tokens
    pub knowledge_injection_tokens: usize,
    /// Token usage per injection type
    pub injection_breakdown: HashMap<InjectionType, usize>,
    /// Token efficiency score
    pub efficiency_score: f32,
}

/// Metadata for generated prompts
#[derive(Debug, Clone)]
pub struct PromptMetadata {
    /// Template used
    pub template_id: String,
    /// Template version
    pub template_version: String,
    /// Generation timestamp
    pub generated_at: chrono::DateTime<chrono::Utc>,
    /// Analysis context hash
    pub context_hash: String,
    /// Knowledge patterns included
    pub patterns_included: Vec<String>,
}

/// Quality metrics for prompt evaluation
#[derive(Debug, Clone)]
pub struct PromptQualityMetrics {
    /// Contextual relevance score (0.0-1.0)
    pub relevance_score: f32,
    /// Knowledge completeness score (0.0-1.0)
    pub completeness_score: f32,
    /// Token efficiency score (0.0-1.0)
    pub efficiency_score: f32,
    /// Estimated AI understanding score (0.0-1.0)
    pub understanding_score: f32,
}

/// Enhanced content with injected knowledge
#[derive(Debug, Clone)]
pub struct EnhancedContent {
    /// Base template content
    pub base_content: String,
    /// Injected knowledge by injection point
    pub injected_knowledge: HashMap<String, InjectedContent>,
    /// Token usage tracking
    pub token_usage: TokenUsage,
    /// Content metadata
    pub metadata: ContentMetadata,
}

/// Content injected at specific points
#[derive(Debug, Clone)]
pub struct InjectedContent {
    /// Injected content text
    pub content: String,
    /// Token count for this injection
    pub token_count: usize,
    /// Injection type
    pub injection_type: InjectionType,
    /// Confidence score for this injection
    pub confidence_score: f32,
    /// Injection metadata
    pub metadata: InjectionMetadata,
}

/// Token usage information
#[derive(Debug, Clone)]
pub struct TokenUsage {
    /// Total tokens used
    pub total: usize,
    /// Tokens by component
    pub breakdown: HashMap<String, usize>,
}

/// Metadata for content generation
#[derive(Debug, Clone)]
pub struct ContentMetadata {
    /// Generation timestamp
    pub generated_at: chrono::DateTime<chrono::Utc>,
    /// Source patterns used
    pub source_patterns: Vec<String>,
    /// Language context applied
    pub language_context: Option<String>,
    /// Framework guidance included
    pub framework_guidance: Vec<String>,
}

/// Metadata for individual injections
#[derive(Debug, Clone)]
pub struct InjectionMetadata {
    /// Source knowledge patterns
    pub source_patterns: Vec<String>,
    /// Confidence in injection relevance
    pub relevance_confidence: f32,
    /// Processing time for injection
    pub processing_time: Duration,
}

/// Template set for language-specific variations
#[derive(Debug, Clone)]
pub struct LanguageTemplateSet {
    /// Base language template
    pub base_template: String,
    /// Language-specific variations
    pub variations: HashMap<String, String>,
    /// Framework-specific overrides
    pub framework_overrides: HashMap<String, String>,
}

/// Framework-specific template
#[derive(Debug, Clone)]
pub struct FrameworkTemplate {
    /// Framework name
    pub framework_name: String,
    /// Template content
    pub content: String,
    /// Framework-specific injection points
    pub injection_points: Vec<InjectionPoint>,
}

/// Knowledge template for specific injection types
#[derive(Debug, Clone)]
pub struct KnowledgeTemplate {
    /// Template for formatting knowledge
    pub format_template: String,
    /// Token budget recommendations
    pub recommended_tokens: usize,
    /// Priority suggestions
    pub priority_level: u8,
}

/// Errors that can occur during prompt generation
#[derive(Debug, thiserror::Error)]
pub enum PromptGenerationError {
    #[error("Template not found for analysis type: {0:?}")]
    TemplateNotFound(AnalysisType),
    #[error("Knowledge injection failed: {0}")]
    InjectionFailed(String),
    #[error("Token budget exceeded: used {used}, budget {budget}")]
    TokenBudgetExceeded { used: usize, budget: usize },
    #[error("Context selection error: {0}")]
    ContextSelectionError(String),
    #[error("Template optimization failed: {0}")]
    OptimizationFailed(String),
}

impl EnhancedPromptTemplateSystem {
    /// Create new enhanced prompt template system
    pub fn new(
        context_selector: DynamicContextSelector,
        optimization_config: TemplateOptimizationConfig,
    ) -> Self {
        Self {
            context_selector,
            template_registry: EnhancedTemplateRegistry::create_default(),
            optimization_config,
            metrics: PromptTemplateMetrics::new(),
        }
    }

    /// Generate enhanced prompt with knowledge integration
    pub async fn generate_enhanced_prompt(
        &mut self,
        analysis_context: &AnalysisContext,
        analysis_type: AnalysisType,
    ) -> Result<EnhancedPrompt, PromptGenerationError> {
        let start_time = Instant::now();

        // 1. Select optimal knowledge context
        let selected_context = self.context_selector
            .select_context(analysis_context)
            .await
            .map_err(|e| PromptGenerationError::ContextSelectionError(e.to_string()))?;

        // 2. Get base template for analysis type
        let base_template = self.template_registry
            .get_base_template(&analysis_type)
            .ok_or(PromptGenerationError::TemplateNotFound(analysis_type))?;

        // 3. Generate knowledge-enhanced content
        let enhanced_content = self.generate_enhanced_content(
            &base_template,
            &selected_context,
            analysis_context,
        ).await?;

        // 4. Optimize for token budget and AI effectiveness
        let optimized_prompt = self.optimize_prompt(
            enhanced_content,
            &selected_context,
            analysis_context,
        ).await?;

        // 5. Update metrics and analytics
        self.metrics.record_generation_time(start_time.elapsed());
        self.update_template_analytics(&optimized_prompt, &selected_context);

        Ok(optimized_prompt)
    }

    /// Generate knowledge-enhanced content for template
    async fn generate_enhanced_content(
        &self,
        base_template: &BaseTemplate,
        selected_context: &SelectedContext,
        analysis_context: &AnalysisContext,
    ) -> Result<EnhancedContent, PromptGenerationError> {
        let mut enhanced_content = EnhancedContent {
            base_content: base_template.content.clone(),
            injected_knowledge: HashMap::new(),
            token_usage: TokenUsage::new(),
            metadata: ContentMetadata::new(),
        };

        // Process each injection point
        for injection_point in &base_template.injection_points {
            if self.should_inject_knowledge(injection_point, selected_context, analysis_context) {
                let injected_content = self.generate_injection_content(
                    injection_point,
                    selected_context,
                    analysis_context,
                ).await?;

                enhanced_content.injected_knowledge.insert(
                    injection_point.id.clone(),
                    injected_content,
                );
            }
        }

        Ok(enhanced_content)
    }

    /// Generate content for specific injection point
    async fn generate_injection_content(
        &self,
        injection_point: &InjectionPoint,
        selected_context: &SelectedContext,
        analysis_context: &AnalysisContext,
    ) -> Result<InjectedContent, PromptGenerationError> {
        match injection_point.injection_type {
            InjectionType::PatternDefinition => {
                self.generate_pattern_definitions(selected_context, injection_point.token_budget)
            },
            InjectionType::LanguageSpecificSymptoms => {
                self.generate_language_symptoms(selected_context, analysis_context, injection_point.token_budget)
            },
            InjectionType::FrameworkGuidance => {
                self.generate_framework_guidance(selected_context, analysis_context, injection_point.token_budget)
            },
            InjectionType::SolutionPatterns => {
                self.generate_solution_patterns(selected_context, injection_point.token_budget)
            },
            InjectionType::RelatedPatterns => {
                self.generate_related_patterns(selected_context, injection_point.token_budget)
            },
            InjectionType::CodeExamples => {
                self.generate_code_examples(selected_context, analysis_context, injection_point.token_budget)
            },
            InjectionType::BestPractices => {
                self.generate_best_practices(selected_context, analysis_context, injection_point.token_budget)
            },
        }
    }

    /// Generate pattern definitions for injection
    fn generate_pattern_definitions(
        &self,
        selected_context: &SelectedContext,
        token_budget: usize,
    ) -> Result<InjectedContent, PromptGenerationError> {
        let mut content = String::new();
        let mut used_tokens = 0;

        for scored_pattern in &selected_context.patterns {
            let pattern = &scored_pattern.candidate.pattern.universal;

            let pattern_content = format!(
                "## {}\n\n**Definition**: {}\n\n**Impact**: {:?}\n\n",
                pattern.name,
                pattern.definition.content,
                pattern.impact
            );

            let pattern_tokens = self.estimate_tokens(&pattern_content);

            if used_tokens + pattern_tokens <= token_budget {
                content.push_str(&pattern_content);
                used_tokens += pattern_tokens;
            } else {
                break;
            }
        }

        Ok(InjectedContent {
            content,
            token_count: used_tokens,
            injection_type: InjectionType::PatternDefinition,
            confidence_score: self.calculate_injection_confidence(selected_context),
            metadata: InjectionMetadata::new(),
        })
    }

    /// Generate language-specific symptoms
    fn generate_language_symptoms(
        &self,
        selected_context: &SelectedContext,
        analysis_context: &AnalysisContext,
        token_budget: usize,
    ) -> Result<InjectedContent, PromptGenerationError> {
        let mut content = String::new();
        let mut used_tokens = 0;

        if let Some(language_context) = &selected_context.language_context {
            content.push_str(&format!(
                "### {}-Specific Symptoms:\n\n",
                format!("{:?}", analysis_context.language)
            ));

            for symptom in &language_context.specific_symptoms {
                let symptom_content = format!("- {}\n", symptom.content);
                let symptom_tokens = self.estimate_tokens(&symptom_content);

                if used_tokens + symptom_tokens <= token_budget {
                    content.push_str(&symptom_content);
                    used_tokens += symptom_tokens;
                } else {
                    break;
                }
            }
        }

        Ok(InjectedContent {
            content,
            token_count: used_tokens,
            injection_type: InjectionType::LanguageSpecificSymptoms,
            confidence_score: 0.9, // High confidence for language-specific content
            metadata: InjectionMetadata::new(),
        })
    }

    /// Generate framework-specific guidance
    fn generate_framework_guidance(
        &self,
        selected_context: &SelectedContext,
        analysis_context: &AnalysisContext,
        token_budget: usize,
    ) -> Result<InjectedContent, PromptGenerationError> {
        let mut content = String::new();
        let mut used_tokens = 0;

        for framework_knowledge in &selected_context.framework_guidance {
            let framework_content = format!(
                "### {} Framework Guidance:\n\n**Best Practices**:\n",
                framework_knowledge.name
            );

            let header_tokens = self.estimate_tokens(&framework_content);
            if used_tokens + header_tokens > token_budget {
                break;
            }

            content.push_str(&framework_content);
            used_tokens += header_tokens;

            for practice in &framework_knowledge.best_practices {
                let practice_content = format!("- {}\n", practice.content);
                let practice_tokens = self.estimate_tokens(&practice_content);

                if used_tokens + practice_tokens <= token_budget {
                    content.push_str(&practice_content);
                    used_tokens += practice_tokens;
                } else {
                    break;
                }
            }
        }

        Ok(InjectedContent {
            content,
            token_count: used_tokens,
            injection_type: InjectionType::FrameworkGuidance,
            confidence_score: 0.85,
            metadata: InjectionMetadata::new(),
        })
    }

    /// Generate solution patterns
    fn generate_solution_patterns(
        &self,
        selected_context: &SelectedContext,
        token_budget: usize,
    ) -> Result<InjectedContent, PromptGenerationError> {
        let mut content = String::new();
        let mut used_tokens = 0;

        for scored_pattern in &selected_context.patterns {
            let solutions = &scored_pattern.candidate.pattern.universal.solution_patterns;

            for solution in solutions {
                let solution_content = format!(
                    "### {}\n\n**Steps**:\n{}\n\n",
                    solution.name,
                    solution.steps.iter()
                        .enumerate()
                        .map(|(i, step)| format!("{}. {}", i + 1, step.content))
                        .collect::<Vec<_>>()
                        .join("\n")
                );

                let solution_tokens = self.estimate_tokens(&solution_content);

                if used_tokens + solution_tokens <= token_budget {
                    content.push_str(&solution_content);
                    used_tokens += solution_tokens;
                } else {
                    break;
                }
            }
        }

        Ok(InjectedContent {
            content,
            token_count: used_tokens,
            injection_type: InjectionType::SolutionPatterns,
            confidence_score: self.calculate_injection_confidence(selected_context),
            metadata: InjectionMetadata::new(),
        })
    }

    /// Generate related patterns
    fn generate_related_patterns(
        &self,
        selected_context: &SelectedContext,
        token_budget: usize,
    ) -> Result<InjectedContent, PromptGenerationError> {
        let mut content = String::new();
        let mut used_tokens = 0;

        content.push_str("### Related Anti-Patterns:\n\n");

        for scored_pattern in &selected_context.patterns {
            let related = &scored_pattern.candidate.pattern.universal.related_patterns;

            for related_pattern in related {
                let related_content = format!(
                    "- **{}**: {} (Relationship: {})\n",
                    related_pattern.pattern_name,
                    related_pattern.description,
                    related_pattern.relationship_type
                );

                let related_tokens = self.estimate_tokens(&related_content);

                if used_tokens + related_tokens <= token_budget {
                    content.push_str(&related_content);
                    used_tokens += related_tokens;
                } else {
                    break;
                }
            }
        }

        Ok(InjectedContent {
            content,
            token_count: used_tokens,
            injection_type: InjectionType::RelatedPatterns,
            confidence_score: 0.8,
            metadata: InjectionMetadata::new(),
        })
    }

    /// Generate code examples
    fn generate_code_examples(
        &self,
        selected_context: &SelectedContext,
        analysis_context: &AnalysisContext,
        token_budget: usize,
    ) -> Result<InjectedContent, PromptGenerationError> {
        let mut content = String::new();
        let mut used_tokens = 0;

        if let Some(language_context) = &selected_context.language_context {
            content.push_str("### Code Examples:\n\n");

            for example in &language_context.code_examples {
                let example_content = format!(
                    "**{}**:\n```{}\n{}\n```\n\n",
                    example.title,
                    format!("{:?}", analysis_context.language).to_lowercase(),
                    example.code
                );

                let example_tokens = self.estimate_tokens(&example_content);

                if used_tokens + example_tokens <= token_budget {
                    content.push_str(&example_content);
                    used_tokens += example_tokens;
                } else {
                    break;
                }
            }
        }

        Ok(InjectedContent {
            content,
            token_count: used_tokens,
            injection_type: InjectionType::CodeExamples,
            confidence_score: 0.9,
            metadata: InjectionMetadata::new(),
        })
    }

    /// Generate best practices
    fn generate_best_practices(
        &self,
        selected_context: &SelectedContext,
        analysis_context: &AnalysisContext,
        token_budget: usize,
    ) -> Result<InjectedContent, PromptGenerationError> {
        let mut content = String::new();
        let mut used_tokens = 0;

        content.push_str("### Best Practices:\n\n");

        for scored_pattern in &selected_context.patterns {
            let practices = &scored_pattern.candidate.pattern.universal.best_practices;

            for practice in practices {
                let practice_content = format!("- {}\n", practice.content);
                let practice_tokens = self.estimate_tokens(&practice_content);

                if used_tokens + practice_tokens <= token_budget {
                    content.push_str(&practice_content);
                    used_tokens += practice_tokens;
                } else {
                    break;
                }
            }
        }

        Ok(InjectedContent {
            content,
            token_count: used_tokens,
            injection_type: InjectionType::BestPractices,
            confidence_score: 0.85,
            metadata: InjectionMetadata::new(),
        })
    }

    /// Determine if knowledge should be injected at this point
    fn should_inject_knowledge(
        &self,
        injection_point: &InjectionPoint,
        selected_context: &SelectedContext,
        analysis_context: &AnalysisContext,
    ) -> bool {
        // Check all conditions
        for condition in &injection_point.conditions {
            if !self.evaluate_condition(condition, selected_context, analysis_context) {
                return false;
            }
        }
        true
    }

    /// Evaluate injection condition
    fn evaluate_condition(
        &self,
        condition: &InjectionCondition,
        selected_context: &SelectedContext,
        analysis_context: &AnalysisContext,
    ) -> bool {
        match condition {
            InjectionCondition::FrameworkDetected => {
                !selected_context.framework_guidance.is_empty()
            },
            InjectionCondition::LanguageMatch(lang) => {
                analysis_context.language == *lang
            },
            InjectionCondition::ConfidenceThreshold(threshold) => {
                selected_context.patterns.iter()
                    .any(|p| p.score.total_score >= *threshold)
            },
            InjectionCondition::TokenBudgetAvailable(min_tokens) => {
                // Simple heuristic - assume budget is available
                *min_tokens <= 500
            },
        }
    }

    /// Estimate token count for text (simple approximation)
    fn estimate_tokens(&self, text: &str) -> usize {
        // Rough approximation: 1 token per 4 characters
        (text.len() + 3) / 4
    }

    /// Calculate confidence score for injection
    fn calculate_injection_confidence(&self, selected_context: &SelectedContext) -> f32 {
        if selected_context.patterns.is_empty() {
            return 0.0;
        }

        let avg_score = selected_context.patterns.iter()
            .map(|p| p.score.total_score)
            .sum::<f32>() / selected_context.patterns.len() as f32;

        avg_score
    }

    /// Optimize prompt for token budget and effectiveness
    async fn optimize_prompt(
        &self,
        enhanced_content: EnhancedContent,
        selected_context: &SelectedContext,
        analysis_context: &AnalysisContext,
    ) -> Result<EnhancedPrompt, PromptGenerationError> {
        // Build final prompt by replacing injection points
        let mut final_content = enhanced_content.base_content.clone();
        let mut total_tokens = self.estimate_tokens(&final_content);
        let mut injection_breakdown = HashMap::new();

        // Replace injection points with generated content
        for (injection_id, injected_content) in &enhanced_content.injected_knowledge {
            let placeholder = format!("{{{}}}", injection_id);
            if final_content.contains(&placeholder) {
                final_content = final_content.replace(&placeholder, &injected_content.content);
                total_tokens += injected_content.token_count;
                injection_breakdown.insert(injected_content.injection_type.clone(), injected_content.token_count);
            }
        }

        // Clean up any remaining placeholders
        final_content = self.clean_remaining_placeholders(final_content);

        let token_usage = DetailedTokenUsage {
            total_tokens,
            base_template_tokens: self.estimate_tokens(&enhanced_content.base_content),
            knowledge_injection_tokens: total_tokens - self.estimate_tokens(&enhanced_content.base_content),
            injection_breakdown,
            efficiency_score: self.calculate_efficiency_score(total_tokens, &enhanced_content),
        };

        Ok(EnhancedPrompt {
            content: final_content,
            knowledge_context: selected_context.clone(),
            token_usage,
            metadata: PromptMetadata {
                template_id: "enhanced_template".to_string(),
                template_version: "1.0.0".to_string(),
                generated_at: chrono::Utc::now(),
                context_hash: "placeholder".to_string(), // TODO: implement proper hashing
                patterns_included: selected_context.patterns.iter()
                    .map(|p| p.candidate.pattern.universal.name.clone())
                    .collect(),
            },
            quality_metrics: PromptQualityMetrics {
                relevance_score: 0.85,
                completeness_score: 0.80,
                efficiency_score: 0.90,
                understanding_score: 0.85,
            },
        })
    }

    /// Clean up remaining placeholder injection points
    fn clean_remaining_placeholders(&self, mut content: String) -> String {
        // Remove any remaining injection point placeholders
        let placeholders = vec![
            "{pattern_definitions}",
            "{language_specific_symptoms}",
            "{framework_guidance}",
            "{solution_patterns}",
            "{related_patterns}",
            "{code_examples}",
            "{best_practices}",
        ];

        for placeholder in placeholders {
            content = content.replace(placeholder, "");
        }

        // Clean up extra whitespace
        content = content.replace("\n\n\n", "\n\n");
        content.trim().to_string()
    }

    /// Calculate efficiency score
    fn calculate_efficiency_score(&self, total_tokens: usize, enhanced_content: &EnhancedContent) -> f32 {
        let knowledge_tokens = enhanced_content.injected_knowledge.values()
            .map(|ic| ic.token_count)
            .sum::<usize>();

        if total_tokens == 0 {
            return 0.0;
        }

        knowledge_tokens as f32 / total_tokens as f32
    }

    /// Update template analytics
    fn update_template_analytics(&mut self, prompt: &EnhancedPrompt, context: &SelectedContext) {
        // Implementation would track usage patterns for adaptive learning
        // For now, just increment counters
        self.metrics.prompts_generated += 1;
    }
}

impl PromptTemplateMetrics {
    fn new() -> Self {
        Self {
            prompts_generated: 0,
            avg_generation_time: Duration::from_millis(0),
            token_efficiency: 0.0,
            injection_success_rate: 0.0,
            satisfaction_scores: Vec::new(),
        }
    }

    fn record_generation_time(&mut self, duration: Duration) {
        // Simple moving average calculation
        let current_avg_ms = self.avg_generation_time.as_millis() as f64;
        let new_duration_ms = duration.as_millis() as f64;
        let count = self.prompts_generated as f64;
        
        let new_avg_ms = if count == 0.0 {
            new_duration_ms
        } else {
            (current_avg_ms * count + new_duration_ms) / (count + 1.0)
        };
        
        self.avg_generation_time = Duration::from_millis(new_avg_ms as u64);
    }
}

impl TokenUsage {
    fn new() -> Self {
        Self {
            total: 0,
            breakdown: HashMap::new(),
        }
    }
}

impl ContentMetadata {
    fn new() -> Self {
        Self {
            generated_at: chrono::Utc::now(),
            source_patterns: Vec::new(),
            language_context: None,
            framework_guidance: Vec::new(),
        }
    }
}

impl InjectionMetadata {
    fn new() -> Self {
        Self {
            source_patterns: Vec::new(),
            relevance_confidence: 0.0,
            processing_time: Duration::from_millis(0),
        }
    }
}

impl EnhancedTemplateRegistry {
    /// Create default template registry with knowledge-enhanced templates
    pub fn create_default() -> Self {
        let mut registry = Self {
            base_templates: HashMap::new(),
            knowledge_templates: HashMap::new(),
            language_templates: HashMap::new(),
            framework_templates: HashMap::new(),
        };

        // Register base templates for different analysis types
        registry.register_base_templates();
        registry.register_knowledge_templates();
        registry.register_language_templates();
        registry.register_framework_templates();

        registry
    }

    /// Get base template for analysis type
    pub fn get_base_template(&self, analysis_type: &AnalysisType) -> Option<&BaseTemplate> {
        self.base_templates.get(analysis_type)
    }

    /// Register base templates for analysis types
    fn register_base_templates(&mut self) {
        // God Object Analysis Template
        self.base_templates.insert(
            AnalysisType::GodObjectAnalysis,
            BaseTemplate {
                id: "god_object_analysis".to_string(),
                content: r#"
You are analyzing code for the God Object anti-pattern. This pattern occurs when a class takes on too many 
responsibilities and becomes difficult to maintain.

## Analysis Context
Language: {language}
Detected Issues: {detected_issues}

{pattern_definitions}

## Language-Specific Considerations
{language_specific_symptoms}

{framework_guidance}

## Solution Approaches
{solution_patterns}

## Related Patterns to Consider
{related_patterns}

## Code Examples
{code_examples}

## Recommendations
{best_practices}

Please provide a comprehensive analysis of the detected God Object pattern, including:
1. Specific symptoms observed in this code
2. Impact on maintainability and testing
3. Step-by-step refactoring recommendations
4. Language and framework-specific considerations
5. Prevention strategies for the future

Focus on actionable, practical guidance that the development team can implement immediately.
"#.to_string(),
                injection_points: vec![
                    InjectionPoint {
                        id: "pattern_definitions".to_string(),
                        injection_type: InjectionType::PatternDefinition,
                        token_budget: 200,
                        priority: 10,
                        conditions: vec![],
                    },
                    InjectionPoint {
                        id: "language_specific_symptoms".to_string(),
                        injection_type: InjectionType::LanguageSpecificSymptoms,
                        token_budget: 150,
                        priority: 9,
                        conditions: vec![],
                    },
                    InjectionPoint {
                        id: "framework_guidance".to_string(),
                        injection_type: InjectionType::FrameworkGuidance,
                        token_budget: 100,
                        priority: 8,
                        conditions: vec![InjectionCondition::FrameworkDetected],
                    },
                    InjectionPoint {
                        id: "solution_patterns".to_string(),
                        injection_type: InjectionType::SolutionPatterns,
                        token_budget: 250,
                        priority: 10,
                        conditions: vec![],
                    },
                    InjectionPoint {
                        id: "related_patterns".to_string(),
                        injection_type: InjectionType::RelatedPatterns,
                        token_budget: 100,
                        priority: 7,
                        conditions: vec![],
                    },
                    InjectionPoint {
                        id: "code_examples".to_string(),
                        injection_type: InjectionType::CodeExamples,
                        token_budget: 200,
                        priority: 8,
                        conditions: vec![],
                    },
                    InjectionPoint {
                        id: "best_practices".to_string(),
                        injection_type: InjectionType::BestPractices,
                        token_budget: 150,
                        priority: 9,
                        conditions: vec![],
                    },
                ],
                token_allocation: TokenAllocation {
                    total_budget: 2000,
                    base_template_tokens: 350,
                    knowledge_injection_tokens: 1650,
                    injection_allocations: HashMap::new(),
                },
                metadata: TemplateMetadata {
                    version: "1.0.0".to_string(),
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    author: "UV-336 Enhanced Templates".to_string(),
                    description: "Knowledge-enhanced God Object analysis template".to_string(),
                },
            }
        );

        // Add templates for other analysis types
        self.register_tight_coupling_template();
        self.register_dead_code_template();
        self.register_performance_analysis_template();
        self.register_general_analysis_template();
    }

    /// Register tight coupling analysis template
    fn register_tight_coupling_template(&mut self) {
        self.base_templates.insert(
            AnalysisType::TightCouplingAnalysis,
            BaseTemplate {
                id: "tight_coupling_analysis".to_string(),
                content: r#"
You are analyzing code for Tight Coupling anti-patterns. This occurs when classes or modules are overly dependent 
on each other, making the system brittle and difficult to modify.

## Analysis Context
Language: {language}
Architectural Patterns: {architectural_patterns}
Detected Dependencies: {detected_dependencies}

{pattern_definitions}

## Language-Specific Coupling Indicators
{language_specific_symptoms}

{framework_guidance}

## Decoupling Strategies
{solution_patterns}

## Related Architectural Concerns
{related_patterns}

## Refactoring Examples
{code_examples}

## Architecture Best Practices
{best_practices}

Please analyze the tight coupling issues and provide:
1. Specific coupling problems identified
2. Impact on system flexibility and maintainability
3. Prioritized decoupling strategies
4. Language and framework-specific solutions
5. Long-term architectural recommendations

Focus on practical steps to reduce coupling while maintaining system functionality.
"#.to_string(),
                injection_points: vec![
                    InjectionPoint {
                        id: "pattern_definitions".to_string(),
                        injection_type: InjectionType::PatternDefinition,
                        token_budget: 180,
                        priority: 10,
                        conditions: vec![],
                    },
                    InjectionPoint {
                        id: "language_specific_symptoms".to_string(),
                        injection_type: InjectionType::LanguageSpecificSymptoms,
                        token_budget: 140,
                        priority: 9,
                        conditions: vec![],
                    },
                    InjectionPoint {
                        id: "framework_guidance".to_string(),
                        injection_type: InjectionType::FrameworkGuidance,
                        token_budget: 120,
                        priority: 8,
                        conditions: vec![InjectionCondition::FrameworkDetected],
                    },
                    InjectionPoint {
                        id: "solution_patterns".to_string(),
                        injection_type: InjectionType::SolutionPatterns,
                        token_budget: 280,
                        priority: 10,
                        conditions: vec![],
                    },
                    InjectionPoint {
                        id: "related_patterns".to_string(),
                        injection_type: InjectionType::RelatedPatterns,
                        token_budget: 120,
                        priority: 7,
                        conditions: vec![],
                    },
                    InjectionPoint {
                        id: "code_examples".to_string(),
                        injection_type: InjectionType::CodeExamples,
                        token_budget: 220,
                        priority: 8,
                        conditions: vec![],
                    },
                    InjectionPoint {
                        id: "best_practices".to_string(),
                        injection_type: InjectionType::BestPractices,
                        token_budget: 160,
                        priority: 9,
                        conditions: vec![],
                    },
                ],
                token_allocation: TokenAllocation {
                    total_budget: 2000,
                    base_template_tokens: 400,
                    knowledge_injection_tokens: 1600,
                    injection_allocations: HashMap::new(),
                },
                metadata: TemplateMetadata {
                    version: "1.0.0".to_string(),
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    author: "UV-336 Enhanced Templates".to_string(),
                    description: "Knowledge-enhanced Tight Coupling analysis template".to_string(),
                },
            }
        );
    }

    /// Register dead code analysis template
    fn register_dead_code_template(&mut self) {
        self.base_templates.insert(
            AnalysisType::DeadCodeAnalysis,
            BaseTemplate {
                id: "dead_code_analysis".to_string(),
                content: r#"
You are analyzing code for Dead Code anti-patterns. This occurs when code is present but never executed,
creating maintenance overhead and reducing code clarity.

## Analysis Context
Language: {language}
Code Coverage: {code_coverage}
Detected Unreachable Code: {unreachable_code}

{pattern_definitions}

## Language-Specific Dead Code Indicators
{language_specific_symptoms}

{framework_guidance}

## Dead Code Elimination Strategies
{solution_patterns}

## Related Maintenance Issues
{related_patterns}

## Detection and Removal Examples
{code_examples}

## Code Hygiene Best Practices
{best_practices}

Please analyze the dead code issues and provide:
1. Specific dead code patterns identified
2. Impact on codebase maintainability and performance
3. Safe removal strategies and verification methods
4. Language and tooling-specific approaches
5. Prevention techniques for the future

Focus on safe, systematic approaches to code cleanup and maintenance.
"#.to_string(),
                injection_points: vec![
                    InjectionPoint {
                        id: "pattern_definitions".to_string(),
                        injection_type: InjectionType::PatternDefinition,
                        token_budget: 160,
                        priority: 10,
                        conditions: vec![],
                    },
                    InjectionPoint {
                        id: "language_specific_symptoms".to_string(),
                        injection_type: InjectionType::LanguageSpecificSymptoms,
                        token_budget: 130,
                        priority: 9,
                        conditions: vec![],
                    },
                    InjectionPoint {
                        id: "framework_guidance".to_string(),
                        injection_type: InjectionType::FrameworkGuidance,
                        token_budget: 100,
                        priority: 7,
                        conditions: vec![InjectionCondition::FrameworkDetected],
                    },
                    InjectionPoint {
                        id: "solution_patterns".to_string(),
                        injection_type: InjectionType::SolutionPatterns,
                        token_budget: 240,
                        priority: 10,
                        conditions: vec![],
                    },
                    InjectionPoint {
                        id: "related_patterns".to_string(),
                        injection_type: InjectionType::RelatedPatterns,
                        token_budget: 100,
                        priority: 6,
                        conditions: vec![],
                    },
                    InjectionPoint {
                        id: "code_examples".to_string(),
                        injection_type: InjectionType::CodeExamples,
                        token_budget: 200,
                        priority: 8,
                        conditions: vec![],
                    },
                    InjectionPoint {
                        id: "best_practices".to_string(),
                        injection_type: InjectionType::BestPractices,
                        token_budget: 140,
                        priority: 8,
                        conditions: vec![],
                    },
                ],
                token_allocation: TokenAllocation {
                    total_budget: 1800,
                    base_template_tokens: 330,
                    knowledge_injection_tokens: 1470,
                    injection_allocations: HashMap::new(),
                },
                metadata: TemplateMetadata {
                    version: "1.0.0".to_string(),
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    author: "UV-336 Enhanced Templates".to_string(),
                    description: "Knowledge-enhanced Dead Code analysis template".to_string(),
                },
            }
        );
    }

    /// Register performance analysis template
    fn register_performance_analysis_template(&mut self) {
        self.base_templates.insert(
            AnalysisType::PerformanceAnalysis,
            BaseTemplate {
                id: "performance_analysis".to_string(),
                content: r#"
You are analyzing code for Performance anti-patterns. These are code structures that negatively impact
system performance, scalability, or resource utilization.

## Analysis Context
Language: {language}
Performance Metrics: {performance_metrics}
Detected Bottlenecks: {detected_bottlenecks}

{pattern_definitions}

## Language-Specific Performance Issues
{language_specific_symptoms}

{framework_guidance}

## Performance Optimization Strategies
{solution_patterns}

## Related Performance Concerns
{related_patterns}

## Optimization Examples
{code_examples}

## Performance Best Practices
{best_practices}

Please analyze the performance issues and provide:
1. Specific performance anti-patterns identified
2. Impact on system performance and scalability
3. Prioritized optimization strategies
4. Language and runtime-specific solutions
5. Performance monitoring and testing recommendations

Focus on measurable improvements with clear before/after impact assessment.
"#.to_string(),
                injection_points: vec![
                    InjectionPoint {
                        id: "pattern_definitions".to_string(),
                        injection_type: InjectionType::PatternDefinition,
                        token_budget: 180,
                        priority: 10,
                        conditions: vec![],
                    },
                    InjectionPoint {
                        id: "language_specific_symptoms".to_string(),
                        injection_type: InjectionType::LanguageSpecificSymptoms,
                        token_budget: 160,
                        priority: 9,
                        conditions: vec![],
                    },
                    InjectionPoint {
                        id: "framework_guidance".to_string(),
                        injection_type: InjectionType::FrameworkGuidance,
                        token_budget: 140,
                        priority: 8,
                        conditions: vec![InjectionCondition::FrameworkDetected],
                    },
                    InjectionPoint {
                        id: "solution_patterns".to_string(),
                        injection_type: InjectionType::SolutionPatterns,
                        token_budget: 300,
                        priority: 10,
                        conditions: vec![],
                    },
                    InjectionPoint {
                        id: "related_patterns".to_string(),
                        injection_type: InjectionType::RelatedPatterns,
                        token_budget: 120,
                        priority: 7,
                        conditions: vec![],
                    },
                    InjectionPoint {
                        id: "code_examples".to_string(),
                        injection_type: InjectionType::CodeExamples,
                        token_budget: 250,
                        priority: 9,
                        conditions: vec![],
                    },
                    InjectionPoint {
                        id: "best_practices".to_string(),
                        injection_type: InjectionType::BestPractices,
                        token_budget: 180,
                        priority: 9,
                        conditions: vec![],
                    },
                ],
                token_allocation: TokenAllocation {
                    total_budget: 2200,
                    base_template_tokens: 370,
                    knowledge_injection_tokens: 1830,
                    injection_allocations: HashMap::new(),
                },
                metadata: TemplateMetadata {
                    version: "1.0.0".to_string(),
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    author: "UV-336 Enhanced Templates".to_string(),
                    description: "Knowledge-enhanced Performance analysis template".to_string(),
                },
            }
        );
    }

    /// Register general analysis template
    fn register_general_analysis_template(&mut self) {
        self.base_templates.insert(
            AnalysisType::GeneralAnalysis,
            BaseTemplate {
                id: "general_analysis".to_string(),
                content: r#"
You are performing a comprehensive architectural analysis of the provided code. This analysis covers
multiple dimensions of code quality, maintainability, and architectural soundness.

## Analysis Context
Language: {language}
Analysis Scope: {analysis_scope}
Detected Issues: {detected_issues}

{pattern_definitions}

## Language-Specific Considerations
{language_specific_symptoms}

{framework_guidance}

## Recommended Solutions
{solution_patterns}

## Related Architectural Patterns
{related_patterns}

## Implementation Examples
{code_examples}

## General Best Practices
{best_practices}

Please provide a comprehensive analysis that includes:
1. Overview of architectural issues identified
2. Priority assessment for addressing issues
3. Systematic refactoring recommendations
4. Language and framework-specific guidance
5. Long-term architectural improvement strategies

Focus on creating a maintainable, scalable, and robust codebase.
"#.to_string(),
                injection_points: vec![
                    InjectionPoint {
                        id: "pattern_definitions".to_string(),
                        injection_type: InjectionType::PatternDefinition,
                        token_budget: 220,
                        priority: 10,
                        conditions: vec![],
                    },
                    InjectionPoint {
                        id: "language_specific_symptoms".to_string(),
                        injection_type: InjectionType::LanguageSpecificSymptoms,
                        token_budget: 170,
                        priority: 8,
                        conditions: vec![],
                    },
                    InjectionPoint {
                        id: "framework_guidance".to_string(),
                        injection_type: InjectionType::FrameworkGuidance,
                        token_budget: 130,
                        priority: 7,
                        conditions: vec![InjectionCondition::FrameworkDetected],
                    },
                    InjectionPoint {
                        id: "solution_patterns".to_string(),
                        injection_type: InjectionType::SolutionPatterns,
                        token_budget: 280,
                        priority: 10,
                        conditions: vec![],
                    },
                    InjectionPoint {
                        id: "related_patterns".to_string(),
                        injection_type: InjectionType::RelatedPatterns,
                        token_budget: 140,
                        priority: 6,
                        conditions: vec![],
                    },
                    InjectionPoint {
                        id: "code_examples".to_string(),
                        injection_type: InjectionType::CodeExamples,
                        token_budget: 230,
                        priority: 8,
                        conditions: vec![],
                    },
                    InjectionPoint {
                        id: "best_practices".to_string(),
                        injection_type: InjectionType::BestPractices,
                        token_budget: 200,
                        priority: 9,
                        conditions: vec![],
                    },
                ],
                token_allocation: TokenAllocation {
                    total_budget: 2100,
                    base_template_tokens: 330,
                    knowledge_injection_tokens: 1770,
                    injection_allocations: HashMap::new(),
                },
                metadata: TemplateMetadata {
                    version: "1.0.0".to_string(),
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    author: "UV-336 Enhanced Templates".to_string(),
                    description: "Knowledge-enhanced General analysis template".to_string(),
                },
            }
        );
    }

    /// Register knowledge templates for different injection types
    fn register_knowledge_templates(&mut self) {
        self.knowledge_templates.insert(
            KnowledgeType::PatternDefinitions,
            KnowledgeTemplate {
                format_template: "## {pattern_name}\n\n**Definition**: {definition}\n\n**Impact**: {impact}\n\n".to_string(),
                recommended_tokens: 150,
                priority_level: 10,
            }
        );

        self.knowledge_templates.insert(
            KnowledgeType::LanguageSpecifics,
            KnowledgeTemplate {
                format_template: "### {language}-Specific Symptoms:\n\n{symptoms}\n\n".to_string(),
                recommended_tokens: 120,
                priority_level: 9,
            }
        );

        self.knowledge_templates.insert(
            KnowledgeType::FrameworkGuidance,
            KnowledgeTemplate {
                format_template: "### {framework} Framework Guidance:\n\n{guidance}\n\n".to_string(),
                recommended_tokens: 100,
                priority_level: 8,
            }
        );

        self.knowledge_templates.insert(
            KnowledgeType::SolutionPatterns,
            KnowledgeTemplate {
                format_template: "### {solution_name}\n\n**Steps**:\n{steps}\n\n".to_string(),
                recommended_tokens: 200,
                priority_level: 10,
            }
        );

        self.knowledge_templates.insert(
            KnowledgeType::BestPractices,
            KnowledgeTemplate {
                format_template: "### Best Practices:\n\n{practices}\n\n".to_string(),
                recommended_tokens: 130,
                priority_level: 8,
            }
        );
    }

    /// Register language-specific template variations
    fn register_language_templates(&mut self) {
        // Rust-specific templates
        self.language_templates.insert(
            SourceLanguage::Rust,
            LanguageTemplateSet {
                base_template: "Focus on ownership, borrowing, and lifetime issues specific to Rust.".to_string(),
                variations: HashMap::new(),
                framework_overrides: HashMap::new(),
            }
        );

        // Python-specific templates
        self.language_templates.insert(
            SourceLanguage::Python,
            LanguageTemplateSet {
                base_template: "Consider Python-specific patterns like duck typing, generators, and GIL implications.".to_string(),
                variations: HashMap::new(),
                framework_overrides: HashMap::new(),
            }
        );

        // JavaScript/TypeScript-specific templates
        self.language_templates.insert(
            SourceLanguage::JavaScript,
            LanguageTemplateSet {
                base_template: "Address JavaScript-specific concerns like prototypal inheritance, closures, and async patterns.".to_string(),
                variations: HashMap::new(),
                framework_overrides: HashMap::new(),
            }
        );

        self.language_templates.insert(
            SourceLanguage::TypeScript,
            LanguageTemplateSet {
                base_template: "Focus on TypeScript type system benefits and proper typing strategies.".to_string(),
                variations: HashMap::new(),
                framework_overrides: HashMap::new(),
            }
        );
    }

    /// Register framework-specific templates
    fn register_framework_templates(&mut self) {
        // React framework template
        self.framework_templates.insert(
            "React".to_string(),
            FrameworkTemplate {
                framework_name: "React".to_string(),
                content: "Consider React-specific patterns like component lifecycle, hooks, and state management.".to_string(),
                injection_points: vec![],
            }
        );

        // Spring framework template
        self.framework_templates.insert(
            "Spring".to_string(),
            FrameworkTemplate {
                framework_name: "Spring".to_string(),
                content: "Focus on Spring dependency injection, AOP, and configuration patterns.".to_string(),
                injection_points: vec![],
            }
        );

        // Django framework template
        self.framework_templates.insert(
            "Django".to_string(),
            FrameworkTemplate {
                framework_name: "Django".to_string(),
                content: "Address Django-specific patterns like models, views, and ORM usage.".to_string(),
                injection_points: vec![],
            }
        );
    }
}