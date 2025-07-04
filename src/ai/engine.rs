use crate::ai::api::llm_provider::LlmProvider;
use crate::ai::ollama_provider::OllamaProvider;
use crate::ai::prompts::smart_prompting::SmartPromptBuilder;
use crate::database::models::ArchitecturalIssue;
use crate::error::UveddiError;
use log::{info, warn};
use std::env;

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
            let model = env::var("OLLAMA_MODEL").unwrap_or_else(|_| "deepseek-coder:6.7b-instruct-q4_0".to_string());
            let ollama_provider = OllamaProvider::new(&model, &api_url);
            Some(Box::new(ollama_provider) as Box<dyn LlmProvider + Send + Sync>)
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
            },
            Err(e) => {
                warn!("Failed to generate AI explanation: {e}");
                // Don't fail the entire analysis if AI fails
            }
        }
        
        Ok(())
    }

    // Add more methods as needed for additional functionalities
}

impl Default for AiAnalysisEngine {
    fn default() -> Self {
        Self::new()
    }
}
