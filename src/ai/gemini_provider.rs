//! GeminiProvider: Google Gemini API integration
//
// Google Gemini API integration with LlmProvider trait implementation

use crate::ai::api::llm_provider::LlmProvider;
use anyhow::{anyhow, Result};
use async_trait::async_trait;

pub struct GeminiProvider {
    pub api_key: String,
}

impl GeminiProvider {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
        }
    }

    pub async fn infer(&self, _prompt: &str) -> Result<String, String> {
        // TODO: Implement HTTP call to Google Gemini API
        Ok("[Gemini stub]".to_string())
    }
}

#[async_trait]
impl LlmProvider for GeminiProvider {
    async fn generate_explanation(&self, prompt: &str) -> Result<String> {
        let response = self.infer(prompt).await.map_err(|e| anyhow!(e))?;
        Ok(response)
    }

    fn get_provider_name(&self) -> &'static str {
        "Gemini"
    }
}
