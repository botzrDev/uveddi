#![cfg(feature = "ai")]

//! Ollama Provider Integration
//!
//! This module provides integration with Ollama, a local LLM runtime that allows
//! running large language models on local hardware. The Ollama provider enables
//! Uveddi to perform AI-powered code analysis without requiring external API calls.
//!
//! # Features
//!
//! - Local LLM inference through Ollama API
//! - Configurable model selection and parameters
//! - Timeout handling for robust operation
//! - Health checking and availability validation
//!
//! # Usage
//!
//! ```rust,no_run
//! use uveddi::ai::ollama_provider::{OllamaProvider, OllamaConfig};
//!
//! let config = OllamaConfig {
//!     model: "deepseek-coder:6.7b-instruct-q4_0".to_string(),
//!     api_url: "http://localhost:11434".to_string(),
//!     timeout_seconds: 60,
//!     temperature: 0.1,
//!     max_tokens: 2048,
//! };
//!
//! let provider = OllamaProvider::new(config);
//!
//! // Check if Ollama is available
//! if provider.check_availability().await? {
//!     // Run inference
//!     let response = provider.infer("Explain this code").await?;
//!     println!("AI Response: {}", response);
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

#[cfg(feature = "ai")]
use crate::security::{HttpSecurityConfig, SecureHttpClient};
use log::{debug, info, warn};
#[cfg(feature = "ai")]
use serde::{Deserialize, Serialize};
use std::time::Duration;
#[cfg(feature = "ai")]
use tokio::time::timeout;

/// Request payload structure for Ollama API calls
///
/// This structure represents the JSON payload sent to the Ollama API
/// for text generation requests.
#[cfg(feature = "ai")]
#[derive(Serialize, Debug)]
struct OllamaRequest<'a> {
    /// The model name to use for inference
    model: &'a str,
    /// The input prompt for text generation
    prompt: &'a str,
    /// Whether to stream the response (always false in our implementation)
    stream: bool,
    /// Optional generation parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
}

/// Generation options for controlling Ollama model behavior
///
/// These parameters control how the language model generates text,
/// affecting creativity, coherence, and output length.
#[cfg(feature = "ai")]
#[derive(Serialize, Debug)]
struct OllamaOptions {
    /// Controls randomness in generation (0.0 = deterministic, 1.0 = very random)
    temperature: f32,
    /// Nucleus sampling parameter - considers only top-p probability mass
    top_p: f32,
    /// Limits vocabulary to top-k most likely tokens
    top_k: i32,
    /// Maximum number of tokens to generate
    num_predict: i32,
    /// Stop sequences that terminate generation
    stop: Vec<String>,
}

/// Response structure from Ollama API
///
/// Represents the JSON response returned by the Ollama API after
/// a successful text generation request.
#[cfg(feature = "ai")]
#[derive(Deserialize, Debug)]
struct OllamaResponse {
    /// The generated text response
    response: String,
    /// Whether the generation is complete (always true for non-streaming)
    #[serde(default)]
    done: bool,
}

/// Error response structure from Ollama API
///
/// Used to parse error messages when the Ollama API returns an error status.
#[cfg(feature = "ai")]
#[derive(Deserialize, Debug)]
struct OllamaErrorResponse {
    /// The error message from the API
    error: String,
}

/// Configuration settings for the Ollama provider
///
/// This struct contains all the necessary configuration parameters
/// for connecting to and using an Ollama instance for AI inference.
///
/// # Examples
///
/// ```rust
/// use uveddi::ai::ollama_provider::OllamaConfig;
///
/// // Use default configuration
/// let config = OllamaConfig::default();
///
/// // Create custom configuration
/// let config = OllamaConfig {
///     model: "codellama:7b-instruct".to_string(),
///     api_url: "http://localhost:11434".to_string(),
///     timeout_seconds: 120,
///     temperature: 0.2,
///     max_tokens: 4096,
/// };
/// ```
#[cfg(feature = "ai")]
#[derive(Clone, Debug)]
pub struct OllamaConfig {
    /// The Ollama model to use for inference (e.g., "deepseek-coder:6.7b-instruct-q4_0")
    pub model: String,
    /// The base URL for the Ollama API (typically "http://localhost:11434")
    pub api_url: String,
    /// Maximum time to wait for API responses in seconds
    pub timeout_seconds: u64,
    /// Temperature parameter for controlling response randomness (0.0-1.0)
    pub temperature: f32,
    /// Maximum number of tokens to generate in responses
    pub max_tokens: i32,
}

impl OllamaConfig {
    /// Automatically selects an appropriate model based on available system memory
    ///
    /// This function checks available system memory and selects a model that will
    /// run efficiently without causing memory pressure or OOM conditions.
    ///
    /// # Memory Thresholds
    ///
    /// - 32GB+: deepseek-coder:6.7b-instruct-q4_0 (high-performance model)
    /// - 16-32GB: llama3.2:3b (balanced performance/memory)
    /// - 8-16GB: phi3:mini (efficient small model)
    /// - 4-8GB: llama3.2:1b (ultra-lightweight)
    /// - <4GB: tinyllama:1.1b (minimal memory footprint)
    ///
    /// # Returns
    ///
    /// String containing the recommended model name for the current system
    pub fn get_memory_appropriate_model() -> String {
        let available_memory_gb = Self::get_system_memory_gb();
        
        let model = match available_memory_gb {
            mem if mem >= 32.0 => {
                info!("High memory system ({}GB) - using deepseek-coder:6.7b-instruct-q4_0", mem);
                "deepseek-coder:6.7b-instruct-q4_0".to_string()
            },
            mem if mem >= 16.0 => {
                info!("Medium-high memory system ({}GB) - using llama3.2:3b", mem);
                "llama3.2:3b".to_string()
            },
            mem if mem >= 8.0 => {
                info!("Medium memory system ({}GB) - using phi3:mini", mem);
                "phi3:mini".to_string()
            },
            mem if mem >= 4.0 => {
                info!("Low memory system ({}GB) - using llama3.2:1b", mem);
                "llama3.2:1b".to_string()
            },
            mem => {
                warn!("Very low memory system ({}GB) - using tinyllama:1.1b", mem);
                "tinyllama:1.1b".to_string()
            }
        };
        
        info!("Selected AI model: {} for {}GB system", model, available_memory_gb);
        model
    }

    /// Gets available system memory in gigabytes
    ///
    /// Attempts to read system memory information from /proc/meminfo on Linux,
    /// falls back to conservative estimates on other platforms or if reading fails.
    ///
    /// # Returns
    ///
    /// Available memory in gigabytes as f64
    fn get_system_memory_gb() -> f64 {
        #[cfg(target_os = "linux")]
        {
            use std::fs;
            
            if let Ok(meminfo) = fs::read_to_string("/proc/meminfo") {
                for line in meminfo.lines() {
                    if line.starts_with("MemAvailable:") {
                        if let Some(kb_str) = line.split_whitespace().nth(1) {
                            if let Ok(kb) = kb_str.parse::<u64>() {
                                let gb = kb as f64 / 1024.0 / 1024.0;
                                debug!("Detected {} GB available memory from /proc/meminfo", gb);
                                return gb;
                            }
                        }
                    }
                }
                
                // Fallback to MemTotal if MemAvailable not found
                for line in meminfo.lines() {
                    if line.starts_with("MemTotal:") {
                        if let Some(kb_str) = line.split_whitespace().nth(1) {
                            if let Ok(kb) = kb_str.parse::<u64>() {
                                let gb = (kb as f64 / 1024.0 / 1024.0) * 0.8; // Assume 80% available
                                debug!("Detected {} GB total memory, estimating {} GB available", 
                                       kb as f64 / 1024.0 / 1024.0, gb);
                                return gb;
                            }
                        }
                    }
                }
            }
        }
        
        // Conservative fallback for non-Linux or if reading fails
        warn!("Could not detect system memory, using conservative 8GB estimate");
        8.0
    }

    /// Creates a configuration with automatic memory-aware model selection
    ///
    /// # Arguments
    ///
    /// * `api_url` - Optional custom Ollama API URL (defaults to localhost:11434)
    ///
    /// # Returns
    ///
    /// OllamaConfig with automatically selected model based on system memory
    pub fn memory_aware(api_url: Option<String>) -> Self {
        Self {
            model: Self::get_memory_appropriate_model(),
            api_url: api_url.unwrap_or_else(|| "http://localhost:11434".to_string()),
            timeout_seconds: 60,
            temperature: 0.1,
            max_tokens: 2048,
        }
    }
}

impl Default for OllamaConfig {
    /// Provides sensible default configuration for Ollama
    ///
    /// Uses memory-appropriate model selection based on available system memory.
    fn default() -> Self {
        Self::memory_aware(None)
    }
}

/// Ollama provider for local LLM inference
///
/// This provider interfaces with a local Ollama instance to perform
/// AI-powered code analysis. It handles connection management, request
/// formatting, and response parsing.
///
/// # Thread Safety
///
/// This struct is designed to be used across async tasks. The internal
/// HTTP client is thread-safe and can be shared between requests.
#[cfg(feature = "ai")]
pub struct OllamaProvider {
    /// Configuration settings for this provider instance
    pub config: OllamaConfig,
    /// HTTP client for making API requests to Ollama (either secure or basic for localhost)
    pub client: OllamaHttpClient,
}

/// HTTP client wrapper for Ollama requests
/// 
/// This enum allows us to use either a secure HTTP client for external connections
/// or a basic reqwest client for localhost connections.
#[cfg(feature = "ai")]
pub enum OllamaHttpClient {
    /// Secure client for external connections
    Secure(SecureHttpClient),
    /// Basic client for localhost connections
    Basic(reqwest::Client),
}

#[cfg(feature = "ai")]
impl OllamaHttpClient {
    /// Make a GET request to the specified URL
    pub async fn get(&self, url: &str) -> Result<reqwest::Response, String> {
        match self {
            OllamaHttpClient::Secure(client) => {
                client.get(url).await.map_err(|e| format!("Secure GET request failed: {}", e))
            },
            OllamaHttpClient::Basic(client) => {
                client.get(url).send().await.map_err(|e| format!("Basic GET request failed: {}", e))
            }
        }
    }

    /// Make a POST request to the specified URL with JSON body
    pub async fn post(&self, url: &str, json_body: String) -> Result<reqwest::Response, String> {
        match self {
            OllamaHttpClient::Secure(client) => {
                client.post(url, json_body).await.map_err(|e| format!("Secure POST request failed: {}", e))
            },
            OllamaHttpClient::Basic(client) => {
                client
                    .post(url)
                    .header("Content-Type", "application/json")
                    .body(json_body)
                    .send()
                    .await
                    .map_err(|e| format!("Basic POST request failed: {}", e))
            }
        }
    }
}

impl OllamaProvider {
    /// Creates a new Ollama provider with the given configuration
    ///
    /// Initializes an HTTP client with appropriate timeouts based on
    /// the configuration settings.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration settings for the Ollama provider
    ///
    /// # Examples
    ///
    /// ```rust
    /// use uveddi::ai::ollama_provider::{OllamaProvider, OllamaConfig};
    ///
    /// let config = OllamaConfig::default();
    /// let provider = OllamaProvider::new(config);
    /// ```
    pub fn new(config: OllamaConfig) -> Result<Self, String> {
        // For localhost connections, use a basic reqwest client to bypass security restrictions
        // Ollama typically runs on localhost with HTTP, which is safe for local AI inference
        let client = if config.api_url.starts_with("http://localhost") || config.api_url.starts_with("http://127.0.0.1") {
            info!("Creating localhost HTTP client for Ollama (bypassing HTTPS enforcement)");
            
            // Create a basic reqwest client for localhost connections
            let reqwest_client = reqwest::Client::builder()
                .timeout(Duration::from_secs(config.timeout_seconds))
                .connect_timeout(Duration::from_secs(10))
                .build()
                .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
            
            OllamaHttpClient::Basic(reqwest_client)
        } else {
            // For external connections, use full security configuration
            let mut http_config = HttpSecurityConfig::default();
            
            #[cfg(debug_assertions)]
            {
                http_config.enforce_https = false; // Allow HTTP for debug builds
            }

            // Set timeouts based on Ollama config
            http_config.timeout_seconds = config.timeout_seconds + 5;
            http_config.connect_timeout_seconds = 10;
            http_config.read_timeout_seconds = config.timeout_seconds;

            let secure_client = SecureHttpClient::new(http_config)
                .map_err(|e| format!("Failed to create secure HTTP client: {}", e))?;
            
            OllamaHttpClient::Secure(secure_client)
        };

        Ok(Self { config, client })
    }

    /// Check if Ollama service is available and the configured model is working
    ///
    /// This method performs a two-step validation:
    /// 1. Checks if the Ollama service is running and responsive
    /// 2. Attempts a test inference to verify the model is loaded and functional
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - Service and model are available
    /// * `Ok(false)` - Service is unavailable or model failed
    /// * `Err(String)` - Unexpected error during availability check
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use uveddi::ai::ollama_provider::{OllamaProvider, OllamaConfig};
    /// # let provider = OllamaProvider::new(OllamaConfig::default());
    /// if provider.check_availability().await? {
    ///     println!("Ollama is ready for use");
    /// } else {
    ///     println!("Ollama is not available");
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub async fn check_availability(&self) -> Result<bool, String> {
        info!(
            "Checking Ollama availability with model: {}",
            self.config.model
        );

        // First check if service is running
        match timeout(
            Duration::from_secs(5),
            self.client
                .get(&format!("{}/api/tags", self.config.api_url)),
        )
        .await
        {
            Ok(result) => match result {
                Ok(response) => {
                    if !response.status().is_success() {
                        warn!(
                            "Ollama service responded with error status: {}",
                            response.status()
                        );
                        return Ok(false);
                    }
                }
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
                info!(
                    "Ollama model {} is available and working",
                    self.config.model
                );
                Ok(true)
            }
            Err(e) => {
                warn!("Ollama model check failed: {}", e);
                Ok(false)
            }
        }
    }

    /// Run inference using the configured Ollama model
    ///
    /// Sends a prompt to the Ollama API and returns the generated response.
    /// This method handles request formatting, timeout management, and error parsing.
    ///
    /// # Arguments
    ///
    /// * `prompt` - The input text prompt for the language model
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The generated response text
    /// * `Err(String)` - Error message describing what went wrong
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The HTTP request fails or times out
    /// - The Ollama API returns an error status
    /// - The response cannot be parsed as valid JSON
    /// - The configured timeout is exceeded
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use uveddi::ai::ollama_provider::{OllamaProvider, OllamaConfig};
    /// # let provider = OllamaProvider::new(OllamaConfig::default());
    /// let prompt = "Explain what this function does: fn add(a: i32, b: i32) -> i32 { a + b }";
    /// let response = provider.infer(prompt).await?;
    /// println!("AI explanation: {}", response);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
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

        // Serialize the request body to JSON
        let json_body = serde_json::to_string(&request_body)
            .map_err(|e| format!("Failed to serialize request: {}", e))?;

        // Use timeout to prevent hanging
        let result = timeout(
            Duration::from_secs(self.config.timeout_seconds),
            self.client
                .post(&format!("{}/api/generate", self.config.api_url), json_body),
        )
        .await;

        // Handle timeout
        let response = match result {
            Ok(res) => match res {
                Ok(response) => response,
                Err(e) => return Err(format!("HTTP request error: {}", e)),
            },
            Err(_) => {
                return Err(format!(
                    "Request timed out after {} seconds",
                    self.config.timeout_seconds
                ))
            }
        };

        let status = response.status();
        if status.is_success() {
            match response.json::<OllamaResponse>().await {
                Ok(ollama_response) => {
                    debug!(
                        "Successfully received response of length {}",
                        ollama_response.response.len()
                    );
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

#[cfg(feature = "ai")]
use crate::ai::api::llm_provider::LlmProvider;
#[cfg(feature = "ai")]
use crate::error::UveddiError;
#[cfg(feature = "ai")]
use async_trait::async_trait;

#[async_trait]
impl LlmProvider for OllamaProvider {
    async fn generate_explanation(&self, prompt: &str) -> Result<String, UveddiError> {
        // Call the OllamaProvider's infer method
        let response = self
            .infer(prompt)
            .await
            .map_err(|e| UveddiError::GenericError {
                message: e.to_string(),
                context: "Ollama request failed".to_string(),
                suggestion: "Check Ollama service status and connectivity".to_string(),
                source: Some(anyhow::anyhow!(e)),
            })?;
        Ok(response)
    }

    fn get_provider_name(&self) -> &'static str {
        "Ollama"
    }
}
