use async_trait::async_trait;
use anyhow::Result;

/// We need to add the `async_trait` macro to make this dyn-compatible
#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn generate_explanation(&self, prompt: &str) -> Result<String>;
    fn get_provider_name(&self) -> &'static str;
}
