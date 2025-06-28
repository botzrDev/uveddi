use crate::database::models::ArchitecturalIssue;
use crate::ai::api::llm_provider::LlmProvider;
use crate::ai::api::openai_provider::OpenAiProvider;
use crate::ai::prompts::prompt_templates;
use anyhow::Result;

/// AI analysis engine with provider abstraction
pub struct AiAnalysisEngine {
    api_provider: Option<Box<dyn LlmProvider>>,
    local_provider: Option<Box<dyn LlmProvider>>,
}

impl AiAnalysisEngine {
    pub fn new() -> Self {
        Self {
            api_provider: None,
            local_provider: None,
        }
    }

    /// Configure OpenAI API provider
    pub fn with_openai_api(mut self, api_key: String) -> Self {
        self.api_provider = Some(Box::new(OpenAiProvider::new(api_key)));
        self
    }

    /// Configure local LLM provider (e.g., Ollama)
    pub fn with_local_provider(mut self, provider: Box<dyn LlmProvider>) -> Self {
        self.local_provider = Some(provider);
        self
    }

    /// Analyze issue with contextual information
    pub async fn analyze_issue(
        &self,
        issue: &mut ArchitecturalIssue,
    ) -> Result<(), AiError> {
        let prompt = prompt_templates::for_issue(issue);

        // Try API provider first, then local, then skip
        if let Some(provider) = &self.api_provider {
            if let Ok(explanation) = provider.generate_explanation(&prompt).await {
                issue.ai_explanation = Some(explanation);
                return Ok(());
            }
        }

        if let Some(provider) = &self.local_provider {
            if let Ok(explanation) = provider.generate_explanation(&prompt).await {
                issue.ai_explanation = Some(explanation);
                return Ok(());
            }
        }

        // Continue without AI explanation
        log::warn!("No AI provider available for issue analysis");
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AiError {
    #[error("AI API error: {0}")]
    Api(String),
    #[error("Context building error: {0}")]
    Context(String),
    #[error("Other AI error: {0}")]
    Other(String),
}