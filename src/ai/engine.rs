use crate::database::models::ArchitecturalIssue;
use crate::ai::api::llm_provider::LlmProvider;
use crate::ai::api::openai_provider::OpenAiProvider;
use crate::ai::anthropic_provider::AnthropicProvider;
use crate::ai::gemini_provider::GeminiProvider;
use crate::ai::ollama_provider::OllamaProvider;
use anyhow::Result;
use crate::ai::types::AiSuggestion;

/// AI analysis engine with provider abstraction
pub struct AiAnalysisEngine {
    providers: Vec<Box<dyn LlmProvider>>,
}

impl AiAnalysisEngine {
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    /// Configure OpenAI API provider
    pub fn with_openai_api(mut self, api_key: String) -> Self {
        self.providers.push(Box::new(OpenAiProvider::new(api_key)));
        self
    }

    /// Configure local LLM provider (Ollama)
    pub fn with_ollama(mut self, model: &str, api_url: &str) -> Self {
        self.providers.push(Box::new(OllamaProvider::new(model, api_url)));
        self
    }

    /// Configure Anthropic provider
    pub fn with_anthropic(mut self, api_key: String) -> Self {
        self.providers.push(Box::new(AnthropicProvider::new(&api_key)));
        self
    }

    /// Configure Gemini provider
    pub fn with_gemini(mut self, api_key: String) -> Self {
        self.providers.push(Box::new(GeminiProvider::new(&api_key)));
        self
    }

    /// Analyze issue with contextual information
    pub async fn analyze_issue(
        &self,
        issue: &mut ArchitecturalIssue,
        ast: &crate::ast::CustomAst,
    ) -> Result<(), AiError> {
        use crate::ai::prompts::smart_prompting::{build_prompt_from_ast, add_hallucination_mitigation};
        // Build a smart prompt with AST/code context
        let base_prompt = build_prompt_from_ast(ast, &issue.description);
        let prompt = add_hallucination_mitigation(&base_prompt);

        for provider in &self.providers {
            if let Ok(response) = provider.generate_explanation(&prompt).await {
                if let Ok(suggestion) = crate::ai::engine::parse_ai_suggestion(&response) {
                    issue.ai_explanation = Some(suggestion.explanation.clone());
                    return Ok(());
                }
            }
        }

        log::warn!("No valid AI suggestion for issue analysis");
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

pub fn parse_ai_suggestion(response: &str) -> Result<AiSuggestion, String> {
    serde_json::from_str::<AiSuggestion>(response)
        .map_err(|e| format!("Failed to parse AI response: {}\nRaw: {}", e, response))
}