use async_trait::async_trait;
use anyhow::Result;
use crate::ai::api::llm_provider::LlmProvider;
use crate::ai::ollama_provider::OllamaProvider;

#[async_trait]
impl LlmProvider for OllamaProvider {
    async fn generate_explanation(&self, prompt: &str) -> Result<String> {
        // Call the OllamaProvider's infer method
        let response = self.infer(prompt).await?;
        Ok(response)
    }

    fn get_provider_name(&self) -> &'static str {
        "Ollama"
    }
}
