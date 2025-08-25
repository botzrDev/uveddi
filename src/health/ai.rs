//! AI integration health checks
//!
//! Validates AI provider connectivity, model availability, and
//! configuration for intelligent code analysis features.

use super::{HealthCheck, HealthStatus};
use crate::core::UveddiError;
use std::time::Duration;

/// Check AI integration health
pub async fn check_ai_health() -> Result<Vec<HealthCheck>, UveddiError> {
    let mut checks = Vec::new();

    // Check if AI features are enabled
    checks.push(check_ai_feature_availability().await);

    // Check Ollama connectivity if AI is enabled
    #[cfg(feature = "local-ai")]
    {
        checks.push(check_ollama_connectivity().await);
        checks.push(check_ollama_models().await);
    }

    Ok(checks)
}

/// Check if AI features are compiled in
async fn check_ai_feature_availability() -> HealthCheck {
    #[cfg(feature = "ai")]
    {
        HealthCheck::new(
            "ai_features",
            HealthStatus::Healthy,
            "AI features are available"
        )
    }

    #[cfg(not(feature = "ai"))]
    {
        HealthCheck::new(
            "ai_features",
            HealthStatus::Warning,
            "AI features are not enabled"
        ).with_details("Compile with --features ai to enable AI analysis")
    }
}

/// Check Ollama service connectivity
#[cfg(feature = "local-ai")]
async fn check_ollama_connectivity() -> HealthCheck {
    let ollama_url = std::env::var("OLLAMA_API_URL")
        .unwrap_or_else(|_| "http://localhost:11434".to_string());

    // Create a client with a short timeout for health checks
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build();

    match client {
        Ok(client) => {
            let health_url = format!("{}/api/tags", ollama_url);
            
            match client.get(&health_url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        HealthCheck::new(
                            "ollama_connectivity",
                            HealthStatus::Healthy,
                            format!("Ollama service is reachable at {}", ollama_url)
                        )
                    } else {
                        HealthCheck::new(
                            "ollama_connectivity",
                            HealthStatus::Warning,
                            format!("Ollama service responded with status: {}", response.status())
                        ).with_auto_fix(true)
                    }
                },
                Err(e) => {
                    if e.is_timeout() {
                        HealthCheck::new(
                            "ollama_connectivity",
                            HealthStatus::Warning,
                            format!("Ollama service at {} is not responding (timeout)", ollama_url)
                        ).with_details("Run 'ollama serve' to start the Ollama service")
                            .with_auto_fix(true)
                    } else if e.is_connect() {
                        HealthCheck::new(
                            "ollama_connectivity",
                            HealthStatus::Warning,
                            format!("Cannot connect to Ollama at {}", ollama_url)
                        ).with_details("Ensure Ollama is installed and running with 'ollama serve'")
                            .with_auto_fix(true)
                    } else {
                        HealthCheck::new(
                            "ollama_connectivity",
                            HealthStatus::Critical,
                            format!("Ollama connectivity check failed: {}", e)
                        )
                    }
                }
            }
        },
        Err(e) => HealthCheck::new(
            "ollama_connectivity",
            HealthStatus::Critical,
            format!("Failed to create HTTP client: {}", e)
        )
    }
}

#[cfg(not(feature = "local-ai"))]
async fn check_ollama_connectivity() -> HealthCheck {
    HealthCheck::new(
        "ollama_connectivity",
        HealthStatus::Warning,
        "Ollama connectivity check skipped - local-ai feature not enabled"
    ).with_details("Compile with --features local-ai to enable Ollama integration")
}

/// Check available Ollama models
#[cfg(feature = "local-ai")]
async fn check_ollama_models() -> HealthCheck {
    let ollama_url = std::env::var("OLLAMA_API_URL")
        .unwrap_or_else(|_| "http://localhost:11434".to_string());
    let expected_model = std::env::var("OLLAMA_MODEL")
        .unwrap_or_else(|_| "deepseek-coder:6.7b".to_string());

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build();

    match client {
        Ok(client) => {
            let models_url = format!("{}/api/tags", ollama_url);
            
            match client.get(&models_url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.json::<serde_json::Value>().await {
                            Ok(json) => {
                                if let Some(models) = json.get("models").and_then(|m| m.as_array()) {
                                    let model_names: Vec<String> = models
                                        .iter()
                                        .filter_map(|model| {
                                            model.get("name")
                                                .and_then(|n| n.as_str())
                                                .map(|s| s.to_string())
                                        })
                                        .collect();

                                    if model_names.is_empty() {
                                        HealthCheck::new(
                                            "ollama_models",
                                            HealthStatus::Warning,
                                            "No models are installed in Ollama"
                                        ).with_details(&format!("Run 'ollama pull {}' to install the recommended model", expected_model))
                                            .with_auto_fix(true)
                                    } else if model_names.iter().any(|name| name.contains(&expected_model.split(':').next().unwrap_or(&expected_model))) {
                                        HealthCheck::new(
                                            "ollama_models",
                                            HealthStatus::Healthy,
                                            format!("Required model '{}' is available", expected_model)
                                        ).with_details(&format!("Available models: {}", model_names.join(", ")))
                                    } else {
                                        HealthCheck::new(
                                            "ollama_models",
                                            HealthStatus::Warning,
                                            format!("Expected model '{}' not found", expected_model)
                                        ).with_details(&format!("Available models: {}. Run 'ollama pull {}' to install the expected model.", 
                                                              model_names.join(", "), expected_model))
                                            .with_auto_fix(true)
                                    }
                                } else {
                                    HealthCheck::new(
                                        "ollama_models",
                                        HealthStatus::Warning,
                                        "Could not parse models list from Ollama response"
                                    )
                                }
                            },
                            Err(e) => HealthCheck::new(
                                "ollama_models",
                                HealthStatus::Warning,
                                format!("Failed to parse Ollama models response: {}", e)
                            )
                        }
                    } else {
                        HealthCheck::new(
                            "ollama_models",
                            HealthStatus::Warning,
                            format!("Ollama models check failed with status: {}", response.status())
                        )
                    }
                },
                Err(e) => HealthCheck::new(
                    "ollama_models",
                    HealthStatus::Warning,
                    format!("Failed to fetch Ollama models: {}", e)
                ).with_details("Ensure Ollama service is running and accessible")
            }
        },
        Err(e) => HealthCheck::new(
            "ollama_models",
            HealthStatus::Critical,
            format!("Failed to create HTTP client for models check: {}", e)
        )
    }
}

#[cfg(not(feature = "local-ai"))]
async fn check_ollama_models() -> HealthCheck {
    HealthCheck::new(
        "ollama_models",
        HealthStatus::Warning,
        "Ollama models check skipped - local-ai feature not enabled"
    ).with_details("Compile with --features local-ai to enable Ollama integration")
}

/// Test AI functionality with a simple request
#[cfg(feature = "local-ai")]
pub async fn test_ai_functionality() -> Result<HealthCheck, UveddiError> {
    let ollama_url = std::env::var("OLLAMA_API_URL")
        .unwrap_or_else(|_| "http://localhost:11434".to_string());
    let model = std::env::var("OLLAMA_MODEL")
        .unwrap_or_else(|_| "deepseek-coder:6.7b".to_string());

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build();

    match client {
        Ok(client) => {
            let generate_url = format!("{}/api/generate", ollama_url);
            let request_body = serde_json::json!({
                "model": model,
                "prompt": "Hello, world!",
                "stream": false,
                "options": {
                    "temperature": 0.1,
                    "num_predict": 10
                }
            });

            match client.post(&generate_url).json(&request_body).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.json::<serde_json::Value>().await {
                            Ok(json) => {
                                if json.get("response").is_some() {
                                    Ok(HealthCheck::new(
                                        "ai_functionality_test",
                                        HealthStatus::Healthy,
                                        format!("AI functionality test passed with model '{}'", model)
                                    ))
                                } else {
                                    Ok(HealthCheck::new(
                                        "ai_functionality_test",
                                        HealthStatus::Warning,
                                        "AI responded but without expected 'response' field"
                                    ))
                                }
                            },
                            Err(e) => Ok(HealthCheck::new(
                                "ai_functionality_test",
                                HealthStatus::Warning,
                                format!("Failed to parse AI response: {}", e)
                            ))
                        }
                    } else {
                        Ok(HealthCheck::new(
                            "ai_functionality_test",
                            HealthStatus::Warning,
                            format!("AI test failed with status: {}", response.status())
                        ))
                    }
                },
                Err(e) => Ok(HealthCheck::new(
                    "ai_functionality_test",
                    HealthStatus::Critical,
                    format!("AI functionality test failed: {}", e)
                ))
            }
        },
        Err(e) => Ok(HealthCheck::new(
            "ai_functionality_test",
            HealthStatus::Critical,
            format!("Failed to create HTTP client for AI test: {}", e)
        ))
    }
}

#[cfg(not(feature = "local-ai"))]
pub async fn test_ai_functionality() -> Result<HealthCheck, UveddiError> {
    Ok(HealthCheck::new(
        "ai_functionality_test",
        HealthStatus::Warning,
        "AI functionality test skipped - local-ai feature not enabled"
    ))
}