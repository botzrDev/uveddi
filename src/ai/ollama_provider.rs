//! OllamaProvider: Local LLM integration for Uveddi

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use log::{info, warn, error, debug};
use tokio::time::timeout;

#[derive(Serialize, Debug)]
struct OllamaRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
}

#[derive(Serialize, Debug)]
struct OllamaOptions {
    temperature: f32,
    top_p: f32,
    top_k: i32,
    num_predict: i32,
    stop: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct OllamaResponse {
    response: String,
    #[serde(default)]
    done: bool,
}

#[derive(Deserialize, Debug)]
struct OllamaErrorResponse {
    error: String,
}

/// Configuration for Ollama provider
#[derive(Clone, Debug)]
pub struct OllamaConfig {
    pub model: String,
    pub api_url: String,
    pub timeout_seconds: u64,
    pub temperature: f32,
    pub max_tokens: i32,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            model: "deepseek-coder:6.7b-instruct-q4_0".to_string(),
            api_url: "http://localhost:11434".to_string(),
            timeout_seconds: 60,
            temperature: 0.1,
            max_tokens: 2048,
        }
    }
}

pub struct OllamaProvider {
    pub config: OllamaConfig,
    pub client: Client,
}

impl OllamaProvider {
    pub fn new(config: OllamaConfig) -> Self {
        // Create HTTP client with appropriate timeouts
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds + 5))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self { config, client }
    }

    /// Check if Ollama service is available and model is loaded
    pub async fn check_availability(&self) -> Result<bool, String> {
        info!("Checking Ollama availability with model: {}", self.config.model);
        
        // First check if service is running
        match timeout(
            Duration::from_secs(5),
            self.client.get(format!("{}/api/tags", self.config.api_url)).send()
        ).await {
            Ok(result) => match result {
                Ok(response) => {
                    if !response.status().is_success() {
                        warn!("Ollama service responded with error status: {}", response.status());
                        return Ok(false);
                    }
                },
                Err(e) => {
                    warn!("Failed to connect to Ollama service: {}", e);
                    return Ok(false);
                }
            },
            Err(_) => {
                warn!("Timeout connecting to Ollama service");
                return Ok(false);
            }
        }

        // Then try a minimal inference to see if model works
        let test_prompt = "Say 'hello'";
        match self.infer(test_prompt).await {
            Ok(_) => {
                info!("Ollama model {} is available and working", self.config.model);
                Ok(true)
            },
            Err(e) => {
                warn!("Ollama model check failed: {}", e);
                Ok(false)
            }
        }
    }

    /// Run inference using the local Ollama model
    pub async fn infer(&self, prompt: &str) -> Result<String, String> {
        debug!("Running inference with prompt of length {}", prompt.len());
        
        let options = OllamaOptions {
            temperature: self.config.temperature,
            top_p: 0.9,
            top_k: 40,
            num_predict: self.config.max_tokens,
            stop: vec!["\n```".to_string(), "</answer>".to_string()],
        };

        let request_body = OllamaRequest {
            model: &self.config.model,
            prompt,
            stream: false,
            options: Some(options),
        };

        // Use timeout to prevent hanging
        let result = timeout(
            Duration::from_secs(self.config.timeout_seconds),
            self.client
                .post(format!("{}/api/generate", self.config.api_url))
                .json(&request_body)
                .send()
        ).await;
        
        // Handle timeout
        let response = match result {
            Ok(res) => match res {
                Ok(response) => response,
                Err(e) => return Err(format!("HTTP request error: {}", e)),
            },
            Err(_) => return Err(format!("Request timed out after {} seconds", self.config.timeout_seconds)),
        };

        let status = response.status();
        if status.is_success() {
            match response.json::<OllamaResponse>().await {
                Ok(ollama_response) => {
                    debug!("Successfully received response of length {}", ollama_response.response.len());
                    Ok(ollama_response.response)
                }
                Err(e) => Err(format!("Failed to parse response: {}", e)),
            }
        } else {
            // Try to get error message
            match response.json::<OllamaErrorResponse>().await {
                Ok(error_response) => Err(format!("Ollama API error: {}", error_response.error)),
                Err(_) => Err(format!("Ollama API error: HTTP {}", status)),
            }
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
