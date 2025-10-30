use crate::error::UveddiError;
use anyhow::Result;
use async_trait::async_trait;

/// A trait for Large Language Model (LLM) providers, defining a common interface for generating text.
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Generates an explanation for a given prompt.
    async fn generate_explanation(&self, prompt: &str) -> Result<String, UveddiError>;
    /// Returns the name of the provider.
    fn get_provider_name(&self) -> &'static str;
}
