use crate::ai::api::llm_provider::LlmProvider;
use crate::ai::knowledge::KnowledgeContext;
use crate::ai::ollama_provider::{OllamaConfig, OllamaProvider};
use crate::ai::prompts::smart_prompting::SmartPromptBuilder;
use crate::database::models::ArchitecturalIssue;
use crate::error::UveddiError;
use log::{info, warn};
use std::env;
use std::time::Instant;

/// AiAnalysisEngine is responsible for performing AI-powered architectural analysis.
/// It integrates with different AI providers to analyze codebases and detect architectural issues.
pub struct AiAnalysisEngine {
    provider: Option<Box<dyn LlmProvider + Send + Sync>>,
    prompt_builder: SmartPromptBuilder,
}

impl AiAnalysisEngine {
    /// Creates a new instance of AiAnalysisEngine with default configurations.
    pub fn new() -> Self {
        // Try to initialize with Ollama provider if available
        let provider = if let Ok(api_url) = env::var("OLLAMA_API_URL") {
            let model = env::var("OLLAMA_MODEL")
                .unwrap_or_else(|_| "deepseek-coder:6.7b-instruct-q4_0".to_string());

            let config = OllamaConfig {
                model,
                api_url,
                ..Default::default()
            };

            match OllamaProvider::new(config) {
                Ok(ollama_provider) => {
                    Some(Box::new(ollama_provider) as Box<dyn LlmProvider + Send + Sync>)
                }
                Err(e) => {
                    warn!("Failed to create Ollama provider: {}", e);
                    None
                }
            }
        } else {
            None
        };

        AiAnalysisEngine {
            provider,
            prompt_builder: SmartPromptBuilder::new(),
        }
    }

    /// Performs architectural analysis on the provided codebase.
    ///
    /// # Arguments
    ///
    /// * `codebase_path` - The path to the codebase to analyze.
    ///
    /// # Returns
    ///
    /// * `Result<(), UveddiError>` - Ok on success, or an error on failure.
    pub fn analyze(&self, _codebase_path: &str) -> Result<(), UveddiError> {
        // Implementation of the analysis logic
        Ok(())
    }

    /// Analyzes a single architectural issue to provide an explanation and recommended solution.
    pub async fn analyze_issue(&self, issue: &mut ArchitecturalIssue) -> Result<(), UveddiError> {
        info!("AI Engine analyzing issue: {}", issue.description);

        // If no provider is configured, skip AI analysis
        let provider = match &self.provider {
            Some(provider) => provider,
            None => {
                info!("No AI provider configured, skipping AI analysis");
                return Ok(());
            }
        };

        // Build a smart prompt for the issue
        let prompt = self.prompt_builder.build_prompt_for_issue(issue);

        // Generate AI explanation
        match provider.generate_explanation(&prompt).await {
            Ok(explanation) => {
                info!("Generated AI explanation for issue");
                issue.ai_explanation = Some(explanation);
            }
            Err(e) => {
                warn!("Failed to generate AI explanation: {e}");
                // Don't fail the entire analysis if AI fails
            }
        }

        Ok(())
    }

    /// Analyze issue with knowledge library context
    ///
    /// This method enhances AI analysis by incorporating relevant knowledge patterns
    /// from the knowledge library to provide more accurate and comprehensive explanations.
    ///
    /// # Arguments
    ///
    /// * `issue` - The architectural issue to analyze and enhance
    /// * `knowledge_context` - Selected knowledge patterns relevant to the issue
    ///
    /// # Returns
    ///
    /// * `Result<(), UveddiError>` - Ok on success, or an error on failure
    pub async fn analyze_issue_with_knowledge(
        &self,
        issue: &mut ArchitecturalIssue,
        knowledge_context: &KnowledgeContext,
    ) -> Result<(), UveddiError> {
        let start_time = Instant::now();
        info!(
            "AI Engine analyzing issue with knowledge context: {}",
            issue.description
        );

        // Check if AI provider is available
        let provider = match &self.provider {
            Some(provider) => provider,
            None => {
                info!("No AI provider configured, using knowledge-only enhancement");
                return self
                    .apply_knowledge_only_enhancement(issue, knowledge_context)
                    .await;
            }
        };

        // Build knowledge-enhanced prompt
        let enhanced_prompt = self
            .prompt_builder
            .build_knowledge_enhanced_prompt(issue, knowledge_context)
            .map_err(|e| {
                UveddiError::config_error(
                    &format!("Failed to build enhanced prompt: {}", e),
                    "AI analysis",
                )
            })?;

        // Generate AI explanation with knowledge context
        match provider.generate_explanation(&enhanced_prompt).await {
            Ok(explanation) => {
                let analysis_time = start_time.elapsed();
                info!(
                    "Generated knowledge-enhanced AI explanation in {:?}",
                    analysis_time
                );

                // Parse and validate AI response
                let validated_explanation =
                    self.validate_ai_explanation(&explanation, knowledge_context)?;
                issue.ai_explanation = Some(validated_explanation);

                // Extract and log solution recommendations (DB model doesn't support this field yet)
                if let Some(solution) = self.extract_solution_from_explanation(&explanation) {
                    info!("AI recommended solution: {}", solution);
                }

                // Record quality metrics
                self.record_ai_quality_metrics(&explanation, knowledge_context, analysis_time);
            }
            Err(e) => {
                warn!("Failed to generate AI explanation: {e}");
                // Fallback to knowledge-only enhancement
                return self
                    .apply_knowledge_only_enhancement(issue, knowledge_context)
                    .await;
            }
        }

        Ok(())
    }

    /// Apply knowledge-only enhancement when AI provider is unavailable
    async fn apply_knowledge_only_enhancement(
        &self,
        issue: &mut ArchitecturalIssue,
        knowledge_context: &KnowledgeContext,
    ) -> Result<(), UveddiError> {
        if let Some(pattern_knowledge) = knowledge_context.selected_patterns.first() {
            let explanation = format!(
                "**Knowledge-Based Analysis**\n\n\
                **Pattern**: {}\n\n\
                **Description**: {}\n\n\
                **Impact**: {:?}\n\n\
                **Common Symptoms**:\n{}\n\n\
                **Recommended Solutions**:\n{}\n\n\
                **Detection Confidence**: {:.1}%\n\n\
                *Note: This analysis is based on architectural knowledge patterns. \
                For AI-enhanced explanations, configure an AI provider.*",
                pattern_knowledge.name,
                pattern_knowledge.definition.as_str(),
                pattern_knowledge.impact,
                pattern_knowledge
                    .symptoms
                    .iter()
                    .enumerate()
                    .map(|(i, symptom)| format!("{}. {}", i + 1, symptom.as_str()))
                    .collect::<Vec<_>>()
                    .join("\n"),
                pattern_knowledge
                    .solutions
                    .iter()
                    .enumerate()
                    .map(|(i, solution)| format!(
                        "{}. **{}** (Effort: {:?}, Impact: {:?})\n   {}",
                        i + 1,
                        solution.title,
                        solution.effort_level,
                        solution.expected_impact,
                        solution.implementation.as_str()
                    ))
                    .collect::<Vec<_>>()
                    .join("\n\n"),
                knowledge_context.relevance_score * 100.0
            );

            issue.ai_explanation = Some(explanation);

            // Log recommended solution (DB model doesn't support this field yet)
            if let Some(best_solution) = pattern_knowledge.solutions.first() {
                info!(
                    "Knowledge-based recommended solution: {}",
                    best_solution.implementation.as_str()
                );
            }

            info!(
                "Applied knowledge-only enhancement for pattern: {}",
                pattern_knowledge.name
            );
        } else {
            warn!("No pattern knowledge available for enhancement");
        }

        Ok(())
    }

    /// Validate AI explanation quality and content
    fn validate_ai_explanation(
        &self,
        explanation: &str,
        knowledge_context: &KnowledgeContext,
    ) -> Result<String, UveddiError> {
        // Basic validation checks
        if explanation.trim().is_empty() {
            return Err(UveddiError::config_error(
                "Empty AI explanation",
                "AI validation",
            ));
        }

        if explanation.len() < 50 {
            return Err(UveddiError::config_error(
                "AI explanation too short",
                "AI validation",
            ));
        }

        // Check for hallucination indicators
        if explanation.contains("I don't have enough information")
            || explanation.contains("I cannot determine")
        {
            warn!("AI explanation indicates uncertainty, may need knowledge fallback");
        }

        // Add knowledge context validation footer
        let validated_explanation = format!(
            "{}\n\n---\n*Analysis enhanced with knowledge library (relevance: {:.1}%, patterns: {})*",
            explanation,
            knowledge_context.relevance_score * 100.0,
            knowledge_context.selected_patterns.len()
        );

        Ok(validated_explanation)
    }

    /// Extract solution recommendation from AI explanation
    fn extract_solution_from_explanation(&self, explanation: &str) -> Option<String> {
        // Simple extraction logic - look for solution patterns
        let lines: Vec<&str> = explanation.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            if line.to_lowercase().contains("solution")
                || line.to_lowercase().contains("recommendation")
                || line.to_lowercase().contains("refactor")
            {
                // Take the next few lines as the solution
                let solution_lines: Vec<&str> = lines.iter().skip(i).take(3).cloned().collect();
                return Some(solution_lines.join(" ").trim().to_string());
            }
        }

        None
    }

    /// Record AI quality metrics for monitoring
    fn record_ai_quality_metrics(
        &self,
        explanation: &str,
        knowledge_context: &KnowledgeContext,
        analysis_time: std::time::Duration,
    ) {
        // Calculate basic quality metrics
        let explanation_length = explanation.len();
        let word_count = explanation.split_whitespace().count();
        let relevance_score = knowledge_context.relevance_score;

        info!(
            "AI quality metrics: length={}, words={}, relevance={:.2}, time={:?}",
            explanation_length, word_count, relevance_score, analysis_time
        );

        // TODO: Integrate with your metrics collection system
        // self.metrics.record_ai_explanation_quality(explanation_length, word_count, relevance_score, analysis_time);
    }

    // Add more methods as needed for additional functionalities
}

impl Default for AiAnalysisEngine {
    fn default() -> Self {
        Self::new()
    }
}
