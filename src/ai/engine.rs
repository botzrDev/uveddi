use crate::database::models::ArchitecturalIssue;
use crate::ai::api::llm_provider::LlmProvider;
use crate::ai::api::openai_provider::OpenAiProvider;
use crate::ai::anthropic_provider::AnthropicProvider;
use crate::ai::gemini_provider::GeminiProvider;
use crate::ai::ollama_provider::OllamaProvider;
use crate::ai::prompts::prompt_templates;
use anyhow::Result;

/// AI analysis engine with provider abstraction
pub struct AiAnalysisEngine {
    api_provider: Option<Box<dyn LlmProvider>>,
    local_provider: Option<Box<dyn LlmProvider>>,
    anthropic_provider: Option<Box<dyn LlmProvider>>,
    gemini_provider: Option<Box<dyn LlmProvider>>,
}

impl AiAnalysisEngine {
    pub fn new() -> Self {
        Self {
            api_provider: None,
            local_provider: None,
            anthropic_provider: None,
            gemini_provider: None,
        }
    }

    /// Configure OpenAI API provider
    pub fn with_openai_api(mut self, api_key: String) -> Self {
        self.api_provider = Some(Box::new(OpenAiProvider::new(api_key)));
        self
    }

    /// Configure local LLM provider (Ollama)
    pub fn with_ollama(mut self, model: &str, api_url: &str) -> Self {
        self.local_provider = Some(Box::new(OllamaProvider::new(model, api_url)));
        self
    }

    /// Configure Anthropic provider
    pub fn with_anthropic(mut self, api_key: String) -> Self {
        self.anthropic_provider = Some(Box::new(AnthropicProvider::new(&api_key)));
        self
    }

    /// Configure Gemini provider
    pub fn with_gemini(mut self, api_key: String) -> Self {
        self.gemini_provider = Some(Box::new(GeminiProvider::new(&api_key)));
        self
    }

    /// Analyze issue with contextual information
    pub async fn analyze_issue(
        &self,
        issue: &mut ArchitecturalIssue,
    ) -> Result<(), AiError> {
        // TODO: Implement prompt engineering and hallucination mitigation here
        // - Use structured prompt templates
        // - Add uncertainty handling instructions
        // - Add self-critique/verification step
        let prompt = prompt_templates::for_issue(issue);

        // Try providers in order: OpenAI, Anthropic, Gemini, Ollama
        if let Some(provider) = &self.api_provider {
            if let Ok(explanation) = provider.generate_explanation(&prompt).await {
                issue.ai_explanation = Some(explanation);
                return Ok(());
            }
        }
        if let Some(provider) = &self.anthropic_provider {
            if let Ok(explanation) = provider.generate_explanation(&prompt).await {
                issue.ai_explanation = Some(explanation);
                return Ok(());
            }
        }
        if let Some(provider) = &self.gemini_provider {
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