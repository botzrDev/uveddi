//! OllamaProvider: Local LLM integration for Uveddi

use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct OllamaRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

pub struct OllamaProvider {
    pub model: String,
    pub client: Client,
    pub api_url: String,
}

impl OllamaProvider {
    pub fn new(model: &str, api_url: &str) -> Self {
        Self {
            model: model.to_string(),
            client: Client::new(),
            api_url: api_url.to_string(),
        }
    }

    /// Run inference using the local Ollama model
    pub async fn infer(&self, prompt: &str) -> Result<String, String> {
        let request_body = OllamaRequest {
            model: &self.model,
            prompt,
            stream: false,
        };

        let response = self
            .client
            .post(format!("{}/api/generate", self.api_url))
            .json(&request_body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if response.status().is_success() {
            let ollama_response = response
                .json::<OllamaResponse>()
                .await
                .map_err(|e| e.to_string())?;
            Ok(ollama_response.response)
        } else {
            Err(format!(
                "Ollama API request failed with status: {}",
                response.status()
            ))
        }
    }

    /// Check if Ollama is running and the model is available
    pub async fn check_status(&self) -> Result<(), String> {
        let response = self
            .client
            .get(format!("{}/api/tags", self.api_url))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!(
                "Ollama API health check failed with status: {}",
                response.status()
            ))
        }
    }

    /// Download a model from Ollama's model registry
    pub async fn download_model(&self, model_name: &str) -> Result<(), String> {
        let url = format!("{}/api/pull", self.api_url);
        let response = self
            .client
            .post(&url)
            .json(&serde_json::json!({ "name": model_name }))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(format!("Failed to download model: {}", response.status()))
        }
    }
}

use crate::ai::api::llm_provider::LlmProvider;
use anyhow::{anyhow, Result};
use async_trait::async_trait;

#[async_trait]
impl LlmProvider for OllamaProvider {
    async fn generate_explanation(&self, prompt: &str) -> Result<String> {
        // Call the OllamaProvider's infer method
        let response = self.infer(prompt).await.map_err(|e| anyhow!(e))?;
        Ok(response)
    }

    fn get_provider_name(&self) -> &'static str {
        "Ollama"
    }
}
