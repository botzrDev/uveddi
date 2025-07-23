//! Carbon-Aware Ollama Integration
//!
//! This module provides carbon awareness capabilities for Ollama AI inference,
//! enabling energy-efficient AI operations and carbon footprint tracking.

use crate::ai::ollama_provider::{OllamaProvider, OllamaConfig};
use crate::monitoring::{CarbonAwarenessCollector, WorkloadType};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, warn, debug};

/// Carbon-aware wrapper for Ollama provider
pub struct CarbonAwareOllamaProvider {
    provider: OllamaProvider,
    carbon_collector: Option<Arc<CarbonAwarenessCollector>>,
    energy_budget_wh: Option<f64>,
    config: CarbonAwareOllamaConfig,
}

/// Configuration for carbon-aware Ollama operations
#[derive(Debug, Clone)]
pub struct CarbonAwareOllamaConfig {
    /// Enable carbon tracking for inference requests
    pub enable_carbon_tracking: bool,
    /// Maximum energy budget per inference request (Wh)
    pub max_energy_per_request_wh: Option<f64>,
    /// Preferred model for energy-efficient inference
    pub energy_efficient_model: Option<String>,
    /// Enable dynamic model selection based on energy budget
    pub dynamic_model_selection: bool,
    /// Context length limits for energy efficiency
    pub max_context_length: Option<usize>,
}

impl Default for CarbonAwareOllamaConfig {
    fn default() -> Self {
        Self {
            enable_carbon_tracking: true,
            max_energy_per_request_wh: Some(0.5), // 0.5 Wh per request
            energy_efficient_model: Some("deepseek-coder:1.3b-instruct-q4_0".to_string()),
            dynamic_model_selection: true,
            max_context_length: Some(4096),
        }
    }
}

impl CarbonAwareOllamaProvider {
    /// Create a new carbon-aware Ollama provider
    pub fn new(
        ollama_config: OllamaConfig,
        carbon_collector: Option<Arc<CarbonAwarenessCollector>>,
        carbon_config: CarbonAwareOllamaConfig,
    ) -> Self {
        let provider = OllamaProvider::new(ollama_config);
        
        Self {
            provider,
            carbon_collector,
            energy_budget_wh: carbon_config.max_energy_per_request_wh,
            config: carbon_config,
        }
    }

    /// Perform carbon-aware inference
    pub async fn infer_with_carbon_tracking(
        &self,
        prompt: &str,
        context: Option<HashMap<String, String>>,
    ) -> Result<InferenceResult> {
        if !self.config.enable_carbon_tracking {
            let response = self.provider.infer(prompt).await?;
            return Ok(InferenceResult {
                response,
                energy_consumed_wh: None,
                model_used: self.provider.get_model_name(),
                carbon_emissions_g: None,
                optimization_applied: false,
            });
        }

        // Start carbon tracking
        let workload_id = if let Some(collector) = &self.carbon_collector {
            let mut tracking_context = HashMap::new();
            tracking_context.insert("operation".to_string(), "ollama_inference".to_string());
            tracking_context.insert("model".to_string(), self.provider.get_model_name());
            tracking_context.insert("prompt_length".to_string(), prompt.len().to_string());
            
            if let Some(ctx) = context {
                tracking_context.extend(ctx);
            }
            
            Some(collector.start_tracking(WorkloadType::OllamaInference, Some(tracking_context)))
        } else {
            None
        };

        info!("Starting carbon-aware Ollama inference");

        // Optimize prompt if needed for energy efficiency
        let optimized_prompt = self.optimize_prompt_for_energy(prompt);
        
        // Select appropriate model based on energy constraints
        let selected_model = self.select_model_for_energy_budget().await?;
        
        // Perform inference with monitoring
        let inference_result = self.perform_monitored_inference(
            &optimized_prompt,
            &selected_model,
            workload_id.as_deref(),
        ).await?;

        // Stop carbon tracking and collect metrics
        let (energy_consumed, carbon_emissions) = if let (Some(collector), Some(id)) = (&self.carbon_collector, workload_id) {
            match collector.stop_tracking(&id) {
                Ok(metrics) => {
                    info!(
                        "Ollama inference consumed {:.3} Wh, emitted {:.3}g CO2",
                        metrics.energy_wh, metrics.co2_emissions_g
                    );
                    
                    // Check against energy budget
                    if let Some(budget) = self.energy_budget_wh {
                        if metrics.energy_wh > budget {
                            warn!(
                                "Ollama inference exceeded energy budget: {:.3} Wh > {:.3} Wh",
                                metrics.energy_wh, budget
                            );
                        }
                    }
                    
                    (Some(metrics.energy_wh), Some(metrics.co2_emissions_g))
                }
                Err(e) => {
                    warn!("Failed to stop carbon tracking: {}", e);
                    (None, None)
                }
            }
        } else {
            (None, None)
        };

        Ok(InferenceResult {
            response: inference_result,
            energy_consumed_wh: energy_consumed,
            model_used: selected_model,
            carbon_emissions_g: carbon_emissions,
            optimization_applied: selected_model != self.provider.get_model_name() || optimized_prompt != prompt,
        })
    }

    /// Perform inference with continuous monitoring
    async fn perform_monitored_inference(
        &self,
        prompt: &str,
        model: &str,
        workload_id: Option<&str>,
    ) -> Result<String> {
        // Create a monitoring task if carbon tracking is enabled
        let monitoring_handle = if let (Some(collector), Some(id)) = (&self.carbon_collector, workload_id) {
            let collector_clone = collector.clone();
            let id_clone = id.to_string();
            
            Some(tokio::spawn(async move {
                let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
                
                for _ in 0..30 { // Monitor for up to 30 seconds
                    interval.tick().await;
                    
                    if let Ok((cpu_util, memory_mb)) = collector_clone.get_system_utilization() {
                        if let Err(e) = collector_clone.record_power_sample(&id_clone, cpu_util, memory_mb) {
                            debug!("Failed to record power sample during inference: {}", e);
                        }
                    }
                }
            }))
        } else {
            None
        };

        // Perform the actual inference
        let result = if model != self.provider.get_model_name() {
            // Would need to create a new provider with different model
            // For now, use the existing provider
            warn!("Model switching not fully implemented, using default model");
            self.provider.infer(prompt).await
        } else {
            self.provider.infer(prompt).await
        };

        // Stop monitoring
        if let Some(handle) = monitoring_handle {
            handle.abort();
        }

        result
    }

    /// Optimize prompt for energy efficiency
    fn optimize_prompt_for_energy(&self, prompt: &str) -> String {
        if let Some(max_length) = self.config.max_context_length {
            if prompt.len() > max_length {
                info!("Truncating prompt from {} to {} characters for energy efficiency", 
                      prompt.len(), max_length);
                return prompt.chars().take(max_length).collect();
            }
        }
        
        prompt.to_string()
    }

    /// Select the most appropriate model based on energy budget
    async fn select_model_for_energy_budget(&self) -> Result<String> {
        if !self.config.dynamic_model_selection {
            return Ok(self.provider.get_model_name());
        }

        // Check if we have a strict energy budget
        if let Some(budget) = self.energy_budget_wh {
            if budget < 0.1 {
                // Very low budget - use smallest model
                if let Some(efficient_model) = &self.config.energy_efficient_model {
                    info!("Selecting energy-efficient model {} due to low energy budget ({:.3} Wh)", 
                          efficient_model, budget);
                    return Ok(efficient_model.clone());
                }
            }
        }

        // Default to configured model
        Ok(self.provider.get_model_name())
    }

    /// Check Ollama availability with carbon awareness
    pub async fn check_availability_with_carbon_context(&self) -> Result<AvailabilityStatus> {
        let is_available = self.provider.check_availability().await?;
        
        if !is_available {
            return Ok(AvailabilityStatus {
                available: false,
                carbon_ready: false,
                recommended_action: "Ollama service is not available".to_string(),
            });
        }

        // Check carbon tracking readiness
        let carbon_ready = self.carbon_collector.is_some() && self.config.enable_carbon_tracking;
        
        let recommended_action = if carbon_ready {
            "Ready for carbon-aware inference".to_string()
        } else {
            "Ollama available but carbon tracking disabled".to_string()
        };

        Ok(AvailabilityStatus {
            available: true,
            carbon_ready,
            recommended_action,
        })
    }

    /// Get carbon efficiency recommendations
    pub fn get_efficiency_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if !self.config.enable_carbon_tracking {
            recommendations.push("Enable carbon tracking to monitor AI inference energy consumption".to_string());
        }
        
        if self.config.max_energy_per_request_wh.is_none() {
            recommendations.push("Set energy budget limits for inference requests".to_string());
        }
        
        if !self.config.dynamic_model_selection {
            recommendations.push("Enable dynamic model selection for energy-efficient inference".to_string());
        }
        
        if self.config.energy_efficient_model.is_none() {
            recommendations.push("Configure a smaller, energy-efficient model for low-priority tasks".to_string());
        }
        
        recommendations
    }
}

/// Result of carbon-aware inference
#[derive(Debug, Clone)]
pub struct InferenceResult {
    pub response: String,
    pub energy_consumed_wh: Option<f64>,
    pub model_used: String,
    pub carbon_emissions_g: Option<f64>,
    pub optimization_applied: bool,
}

/// Ollama availability status with carbon context
#[derive(Debug, Clone)]
pub struct AvailabilityStatus {
    pub available: bool,
    pub carbon_ready: bool,
    pub recommended_action: String,
}

/// Inference optimization strategy based on energy constraints
#[derive(Debug, Clone)]
pub enum OptimizationStrategy {
    /// No optimization, use full model capabilities
    None,
    /// Reduce context length to save energy
    ReduceContext(usize),
    /// Switch to more efficient model
    SwitchModel(String),
    /// Both context reduction and model switching
    Aggressive(usize, String),
}

impl CarbonAwareOllamaProvider {
    /// Determine optimization strategy based on energy budget
    pub fn determine_optimization_strategy(&self, estimated_energy_wh: f64) -> OptimizationStrategy {
        if let Some(budget) = self.energy_budget_wh {
            if estimated_energy_wh > budget * 1.5 {
                // Aggressive optimization needed
                if let Some(efficient_model) = &self.config.energy_efficient_model {
                    return OptimizationStrategy::Aggressive(
                        self.config.max_context_length.unwrap_or(2048),
                        efficient_model.clone(),
                    );
                }
            } else if estimated_energy_wh > budget {
                // Moderate optimization
                if self.config.dynamic_model_selection {
                    if let Some(efficient_model) = &self.config.energy_efficient_model {
                        return OptimizationStrategy::SwitchModel(efficient_model.clone());
                    }
                }
                
                if let Some(max_context) = self.config.max_context_length {
                    return OptimizationStrategy::ReduceContext(max_context);
                }
            }
        }
        
        OptimizationStrategy::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monitoring::CarbonAwarenessConfig;

    #[tokio::test]
    async fn test_carbon_aware_inference() {
        let ollama_config = OllamaConfig::default();
        let carbon_config = CarbonAwarenessConfig::default();
        let carbon_collector = Arc::new(CarbonAwarenessCollector::new(carbon_config, None).unwrap());
        
        let provider = CarbonAwareOllamaProvider::new(
            ollama_config,
            Some(carbon_collector),
            CarbonAwareOllamaConfig::default(),
        );
        
        // Test availability check
        let status = provider.check_availability_with_carbon_context().await;
        // Would need actual Ollama service running to test fully
        
        // Test optimization strategy
        let strategy = provider.determine_optimization_strategy(1.0);
        assert!(matches!(strategy, OptimizationStrategy::SwitchModel(_)));
    }

    #[test]
    fn test_prompt_optimization() {
        let ollama_config = OllamaConfig::default();
        let mut carbon_config = CarbonAwareOllamaConfig::default();
        carbon_config.max_context_length = Some(100);
        
        let provider = CarbonAwareOllamaProvider::new(
            ollama_config,
            None,
            carbon_config,
        );
        
        let long_prompt = "a".repeat(200);
        let optimized = provider.optimize_prompt_for_energy(&long_prompt);
        assert_eq!(optimized.len(), 100);
    }

    #[test]
    fn test_efficiency_recommendations() {
        let ollama_config = OllamaConfig::default();
        let mut carbon_config = CarbonAwareOllamaConfig::default();
        carbon_config.enable_carbon_tracking = false;
        
        let provider = CarbonAwareOllamaProvider::new(
            ollama_config,
            None,
            carbon_config,
        );
        
        let recommendations = provider.get_efficiency_recommendations();
        assert!(!recommendations.is_empty());
        assert!(recommendations.iter().any(|r| r.contains("carbon tracking")));
    }
}