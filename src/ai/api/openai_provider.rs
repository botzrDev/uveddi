use super::llm_provider::LlmProvider;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

pub struct OpenAiProvider {
    api_key: String,
    client: Client,
}

impl OpenAiProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    async fn generate_explanation(&self, prompt: &str) -> Result<String> {
        let response = self.client.post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&json!({
                "model": "gpt-4o-mini", // Using a cost-effective model for explanations
                "messages": [
                    {"role": "system", "content": "You are a helpful assistant that provides concise and actionable explanations for architectural code issues."},
                    {"role": "user", "content": prompt}
                ],
                "max_tokens": 500,
                "temperature": 0.7,
            }))
            .send()
            .await?;

        let json_response: serde_json::Value = response.json().await?;

        if let Some(error) = json_response["error"].as_object() {
            return Err(anyhow!(
                "OpenAI API error: {}",
                error["message"].as_str().unwrap_or("unknown error")
            ));
        }

        let explanation = json_response["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow!("Failed to parse OpenAI response"))?;

        Ok(explanation.to_string())
    }

    fn get_provider_name(&self) -> &'static str {
        "openai"
    }
}
