//! Plugin detector adapter for integrating WASM plugins with the analysis pipeline
//!
//! This module provides the bridge between WASM plugins and Uveddi's core analysis
//! engine, allowing plugins to participate in the analysis workflow as first-class
//! detectors alongside built-in detectors.

use crate::analysis::{AnalysisDetector, AnalysisError};
use crate::ast::ParsedFile;
use crate::database::models::{AntiPatternType, ArchitecturalIssue};
use crate::error::UveddiError;
use crate::plugins::{
    host_functions::PluginIssue,
    types::{HostContext, PluginId},
    HostContextFactory,
    PluginAnalysisResult,
    PluginRuntime,
    WasmPluginEngine,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Adapter that wraps a WASM plugin to implement the AnalysisDetector trait
pub struct PluginDetectorAdapter {
    /// Plugin ID
    plugin_id: PluginId,
    /// Plugin name for display
    plugin_name: String,
    /// Plugin runtime for execution
    runtime: Arc<RwLock<PluginRuntime>>,
    /// Plugin binary for execution
    plugin_binary: Vec<u8>,
    /// Supported anti-pattern types
    supported_anti_patterns: Vec<AntiPatternType>,
    /// Plugin configuration
    config: PluginDetectorConfig,
}

/// Configuration for plugin detector behavior
#[derive(Debug, Clone)]
pub struct PluginDetectorConfig {
    /// Maximum execution time per file in milliseconds
    pub max_execution_time_ms: u64,
    /// Enable detailed logging
    pub enable_logging: bool,
    /// Retry failed executions
    pub retry_on_failure: bool,
    /// Maximum number of retries
    pub max_retries: u32,
    /// Timeout for plugin initialization
    pub init_timeout_ms: u64,
}

impl Default for PluginDetectorConfig {
    fn default() -> Self {
        Self {
            max_execution_time_ms: 30_000, // 30 seconds
            enable_logging: true,
            retry_on_failure: true,
            max_retries: 3,
            init_timeout_ms: 10_000, // 10 seconds
        }
    }
}

impl PluginDetectorAdapter {
    /// Create a new plugin detector adapter
    pub fn new(
        plugin_id: PluginId,
        plugin_name: String,
        runtime: Arc<RwLock<PluginRuntime>>,
        plugin_binary: Vec<u8>,
        supported_anti_patterns: Vec<AntiPatternType>,
    ) -> Self {
        Self {
            plugin_id,
            plugin_name,
            runtime,
            plugin_binary,
            supported_anti_patterns,
            config: PluginDetectorConfig::default(),
        }
    }

    /// Create adapter with custom configuration
    pub fn with_config(
        plugin_id: PluginId,
        plugin_name: String,
        runtime: Arc<RwLock<PluginRuntime>>,
        plugin_binary: Vec<u8>,
        supported_anti_patterns: Vec<AntiPatternType>,
        config: PluginDetectorConfig,
    ) -> Self {
        Self {
            plugin_id,
            plugin_name,
            runtime,
            plugin_binary,
            supported_anti_patterns,
            config,
        }
    }

    /// Execute the plugin for analysis
    async fn execute_plugin_analysis(
        &self,
        file: &ParsedFile,
    ) -> Result<Vec<PluginIssue>, UveddiError> {
        if self.config.enable_logging {
            debug!(
                "Executing plugin {} for file: {}",
                self.plugin_name,
                file.file_path.display()
            );
        }

        // Serialize file data for plugin
        let file_data = self.serialize_file_for_plugin(file)?;

        // Execute plugin with retries
        let mut attempt = 0;
        let max_attempts = if self.config.retry_on_failure {
            self.config.max_retries + 1
        } else {
            1
        };

        while attempt < max_attempts {
            match self.try_execute_plugin(&file_data).await {
                Ok(issues) => {
                    if self.config.enable_logging {
                        info!(
                            "Plugin {} found {} issues in {}",
                            self.plugin_name,
                            issues.len(),
                            file.file_path.display()
                        );
                    }
                    return Ok(issues);
                }
                Err(e) => {
                    attempt += 1;
                    if attempt < max_attempts {
                        warn!(
                            "Plugin {} execution failed (attempt {}): {}. Retrying...",
                            self.plugin_name, attempt, e
                        );
                        tokio::time::sleep(tokio::time::Duration::from_millis(
                            100 * attempt as u64,
                        ))
                        .await;
                    } else {
                        error!(
                            "Plugin {} execution failed after {} attempts: {}",
                            self.plugin_name, max_attempts, e
                        );
                        return Err(e);
                    }
                }
            }
        }

        // This should never be reached, but just in case
        Err(UveddiError::PluginError {
            plugin: self.plugin_name.clone(),
            plugin_type: "WASM".to_string(),
            message: "Plugin execution failed".to_string(),
            suggestion: "Check plugin configuration and logs".to_string(),
            source: None,
        })
    }

    /// Try to execute the plugin once
    async fn try_execute_plugin(&self, file_data: &[u8]) -> Result<Vec<PluginIssue>, UveddiError> {
        let runtime = self.runtime.read().await;

        // Execute the plugin's analyze function
        let result = runtime
            .execute_plugin(
                &self.plugin_id,
                &self.plugin_binary,
                "analyze_file",
                file_data,
            )
            .await?;

        // Deserialize plugin results
        self.deserialize_plugin_results(&result)
    }

    /// Serialize file data for plugin consumption
    fn serialize_file_for_plugin(&self, file: &ParsedFile) -> Result<Vec<u8>, UveddiError> {
        // Create a simplified file representation for the plugin
        let plugin_file_data = serde_json::json!({
            "file_path": file.file_path.to_string_lossy(),
            "language": format!("{:?}", file.language),
            "source": file.source.as_str(),
            "has_tree": file.tree.is_some(),
            "syntax_errors": 0, // TODO: Add syntax error tracking to ParsedFile
        });

        serde_json::to_vec(&plugin_file_data).map_err(|e| UveddiError::PluginError {
            plugin: self.plugin_name.clone(),
            plugin_type: "WASM".to_string(),
            message: format!("Failed to serialize file data: {}", e),
            suggestion: "Check file data format".to_string(),
            source: None,
        })
    }

    /// Deserialize plugin results
    fn deserialize_plugin_results(&self, data: &[u8]) -> Result<Vec<PluginIssue>, UveddiError> {
        if data.is_empty() {
            return Ok(Vec::new());
        }

        // Try to deserialize as JSON first
        match serde_json::from_slice::<Vec<PluginIssue>>(data) {
            Ok(issues) => Ok(issues),
            Err(e) => {
                error!("Failed to deserialize plugin results: {}", e);
                // Return empty results rather than failing completely
                Ok(Vec::new())
            }
        }
    }

    /// Convert plugin issues to architectural issues
    fn convert_to_architectural_issues(
        &self,
        plugin_issues: Vec<PluginIssue>,
        _file_path: &str,
    ) -> Vec<ArchitecturalIssue> {
        plugin_issues
            .into_iter()
            .map(|issue| ArchitecturalIssue {
                issue_id: None,     // Will be assigned by database
                analysis_run_id: 0, // Will be set by caller
                anti_pattern_type_id: issue.anti_pattern_type_id.unwrap_or(0),
                file_path: issue.file_path,
                start_line: issue.line_number,
                end_line: issue.line_number,
                line_number: issue.line_number,
                column_number: None, // TODO: Add column_number to PluginIssue
                message: format!("[{}] {}", self.plugin_name, issue.message),
                metadata: serde_json::to_string(&issue.metadata).unwrap_or_default(),
                detector_name: self.plugin_name.clone(),
                created_at: chrono::Utc::now(),
                severity: issue.severity,
                description: issue.message.clone(),
                code_snippet: None,
                ai_explanation: Some(issue.suggestion),
            })
            .collect()
    }
}

#[async_trait]
impl AnalysisDetector for PluginDetectorAdapter {
    async fn detect_issues(
        &self,
        file: &ParsedFile,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        // Execute plugin analysis
        let plugin_issues = self.execute_plugin_analysis(file).await.map_err(|e| {
            AnalysisError::PluginError(crate::plugins::errors::PluginError::Execution(format!(
                "Plugin '{}': {}",
                self.plugin_name, e
            )))
        })?;

        // Convert to architectural issues
        let architectural_issues =
            self.convert_to_architectural_issues(plugin_issues, &file.file_path.to_string_lossy());

        Ok(architectural_issues)
    }

    fn get_anti_pattern_types(&self) -> Vec<AntiPatternType> {
        self.supported_anti_patterns.clone()
    }

    fn get_detector_name(&self) -> &'static str {
        // Since we can't return a reference to self.plugin_name, we return a generic name
        // This could be improved by using a static string pool or Box::leak
        "PluginDetector"
    }
}

/// Manager for plugin detector adapters
pub struct PluginDetectorManager {
    /// Active plugin detectors
    detectors: Arc<RwLock<HashMap<PluginId, Arc<PluginDetectorAdapter>>>>,
    /// Plugin engine for managing plugins
    plugin_engine: Arc<RwLock<WasmPluginEngine>>,
    /// Plugin runtime
    runtime: Arc<RwLock<PluginRuntime>>,
}

impl PluginDetectorManager {
    /// Create a new plugin detector manager
    pub async fn new() -> Result<Self, UveddiError> {
        let plugin_engine = WasmPluginEngine::new().await?;
        let mut runtime = PluginRuntime::new();
        runtime.initialize().await?;

        Ok(Self {
            detectors: Arc::new(RwLock::new(HashMap::new())),
            plugin_engine: Arc::new(RwLock::new(plugin_engine)),
            runtime: Arc::new(RwLock::new(runtime)),
        })
    }

    /// Register a plugin as a detector
    pub async fn register_plugin_detector(
        &self,
        plugin_id: PluginId,
        plugin_name: String,
        host_context: HostContext,
        plugin_binary: Vec<u8>,
        supported_anti_patterns: Vec<AntiPatternType>,
    ) -> Result<(), UveddiError> {
        info!("Registering plugin detector: {}", plugin_name);

        // Register plugin with runtime
        {
            let factory = HostContextFactory::new(
                host_context.database.clone(),
                host_context.analysis_engine.clone(),
            );
            let runtime = self.runtime.read().await;
            runtime
                .register_plugin(
                    plugin_id.clone(),
                    host_context.host_state.security_policy.clone(),
                    factory,
                )
                .await?;
        }

        // Create detector adapter
        let adapter = Arc::new(PluginDetectorAdapter::new(
            plugin_id.clone(),
            plugin_name.clone(),
            self.runtime.clone(),
            plugin_binary,
            supported_anti_patterns,
        ));

        // Store detector
        let mut detectors = self.detectors.write().await;
        detectors.insert(plugin_id.clone(), adapter);

        info!("Plugin detector registered: {}", plugin_name);
        Ok(())
    }

    /// Unregister a plugin detector
    pub async fn unregister_plugin_detector(
        &self,
        plugin_id: &PluginId,
    ) -> Result<(), UveddiError> {
        info!("Unregistering plugin detector: {}", plugin_id);

        // Remove from detectors
        let mut detectors = self.detectors.write().await;
        detectors.remove(plugin_id);

        // Unregister from runtime
        {
            let runtime = self.runtime.read().await;
            runtime.unregister_plugin(plugin_id).await?;
        }

        info!("Plugin detector unregistered: {}", plugin_id);
        Ok(())
    }

    /// Get all registered plugin detectors
    pub async fn get_all_detectors(&self) -> Vec<Arc<PluginDetectorAdapter>> {
        let detectors = self.detectors.read().await;
        detectors.values().cloned().collect()
    }

    /// Get a specific plugin detector
    pub async fn get_detector(&self, plugin_id: &PluginId) -> Option<Arc<PluginDetectorAdapter>> {
        let detectors = self.detectors.read().await;
        detectors.get(plugin_id).cloned()
    }

    /// Get statistics for all plugin detectors
    pub async fn get_detector_stats(
        &self,
    ) -> HashMap<PluginId, crate::plugins::PluginRuntimeStats> {
        let mut stats = HashMap::new();
        let runtime = self.runtime.read().await;

        let detectors = self.detectors.read().await;
        for plugin_id in detectors.keys() {
            if let Some(plugin_stats) = runtime.get_plugin_stats(plugin_id).await {
                stats.insert(plugin_id.clone(), plugin_stats);
            }
        }

        stats
    }
}

impl Default for PluginDetectorManager {
    fn default() -> Self {
        // Use blocking version for default constructor
        // In real usage, prefer the async new() method
        let plugin_engine = futures::executor::block_on(async {
            WasmPluginEngine::new().await.unwrap_or_else(|_| {
                // Return a placeholder if engine creation fails
                panic!("Failed to create plugin engine for default constructor")
            })
        });

        let mut runtime = PluginRuntime::new();
        futures::executor::block_on(async {
            runtime
                .initialize()
                .await
                .expect("Failed to initialize runtime");
        });

        Self {
            detectors: Arc::new(RwLock::new(HashMap::new())),
            plugin_engine: Arc::new(RwLock::new(plugin_engine)),
            runtime: Arc::new(RwLock::new(runtime)),
        }
    }
}
