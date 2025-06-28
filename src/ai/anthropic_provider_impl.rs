use async_trait::async_trait;
use anyhow::{Result, anyhow};
use crate::ai::api::llm_provider::LlmProvider;
use crate::ai::anthropic_provider::AnthropicProvider;

#[async_trait]
impl LlmProvider for AnthropicProvider {
    async fn generate_explanation(&self, prompt: &str) -> Result<String> {
        let response = self.infer(prompt).await.map_err(|e| anyhow!(e))?;
        Ok(response)
    }

    fn get_provider_name(&self) -> &'static str {
        "Anthropic"
    }
}
