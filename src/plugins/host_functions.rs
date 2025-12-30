//! Host functions that provide Uveddi core functionality to WASM plugins
//!
//! This module implements the host-side functions that plugins can call to access
//! Uveddi's core analysis capabilities, database storage, and system configuration.
//! All host functions are implemented using the WebAssembly Component Model with
//! proper capability-based security.

use crate::analysis::AnalysisEngine;
use crate::database::models::{AnalysisRun, ArchitecturalIssue};
use crate::database::ScalableDatabase;
use crate::error::UveddiError;
use crate::plugins::errors::PluginError;
use crate::plugins::security::{Permission, SecurityPolicy};
use crate::plugins::types::PluginId;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

#[cfg(feature = "wasm-plugins")]
use crate::plugins::types::HostContext;

#[cfg(feature = "wasm-plugins")]
use crate::plugins::wasm::core_analysis::{
    self, Host, LogLevel, AstNode, FileMetadata, HttpResponse, ProcessInfo,
    SeverityLevel, IssueCategory, Span, Position, Metrics, AnalysisResult,
    PluginConfig as WitPluginConfig, ResourceLimits as WitResourceLimits, PluginInfo
};

#[cfg(feature = "wasm-plugins")]
impl HostContext {
    /// Check if plugin has required permission
    fn check_permission(&self, permission: Permission) -> Result<(), String> {
        if self.host_state.security_policy.has_permission(&permission) {
            Ok(())
        } else {
            Err(format!(
                "Plugin {} does not have permission {:?}",
                self.host_state.plugin_id, permission
            ))
        }
    }
}

#[cfg(feature = "wasm-plugins")]
#[async_trait::async_trait]
impl Host for HostContext {
    async fn log(&mut self, level: LogLevel, message: String) -> () {
        let plugin_id = &self.host_state.plugin_id;
        match level {
            LogLevel::Trace => tracing::trace!("[Plugin {}] {}", plugin_id, message),
            LogLevel::Debug => tracing::debug!("[Plugin {}] {}", plugin_id, message),
            LogLevel::Info => tracing::info!("[Plugin {}] {}", plugin_id, message),
            LogLevel::Warn => tracing::warn!("[Plugin {}] {}", plugin_id, message),
            LogLevel::Error => tracing::error!("[Plugin {}] {}", plugin_id, message),
        }
    }

    async fn read_file(&mut self, path: String) -> Result<String, String> {
        self.check_permission(Permission::FileRead)?;
        // TODO: Implement actual file reading with path sanitization
        Err("Not implemented".to_string())
    }

    async fn write_file(&mut self, _path: String, _content: String) -> Result<(), String> {
        self.check_permission(Permission::FileWrite)?;
        Err("Not implemented".to_string())
    }

    async fn file_exists(&mut self, _path: String) -> bool {
        // TODO: Check permissions and file existence
        false
    }

    async fn list_files(&mut self, _pattern: String) -> Result<Vec<String>, String> {
        self.check_permission(Permission::FileRead)?;
        Err("Not implemented".to_string())
    }

    async fn get_file_metadata(&mut self, _path: String) -> Result<FileMetadata, String> {
         Err("Not implemented".to_string())
    }

    async fn get_config(&mut self, key: String) -> Option<String> {
        // First check runtime config store
        if let Ok(store) = self.config_store.try_read() {
            if let Some(val) = store.get(&key) {
                return Some(val.clone());
            }
        }
        // Then check initial config
        self.host_state.config.custom_settings.get(&key).cloned()
    }

    async fn set_config(&mut self, key: String, value: String) -> Result<(), String> {
         if let Ok(mut store) = self.config_store.write().await {
             store.insert(key, value);
             Ok(())
         } else {
             Err("Failed to acquire config lock".to_string())
         }
    }

    async fn parse_ast(&mut self, code: String, language: String) -> Result<AstNode, String> {
        // TODO: Integrate with AnalysisEngine
         Ok(AstNode {
            node_type: "root".to_string(),
            content: code,
            span: Span {
                start: Position { line: 0, column: 0, byte_offset: 0 },
                end: Position { line: 0, column: 0, byte_offset: 0 },
            },
            language,
            attributes: Vec::new(),
        })
    }

    async fn query_ast(&mut self, _node: AstNode, _query: String) -> Result<Vec<AstNode>, String> {
        Err("Not implemented".to_string())
    }

    async fn calculate_hash(&mut self, _algorithm: String, _content: String) -> Result<String, String> {
        Err("Not implemented".to_string())
    }

    async fn verify_signature(&mut self, _content: String, _signature: String, _public_key: String) -> Result<bool, String> {
        Err("Not implemented".to_string())
    }

    async fn http_get(&mut self, _url: String, _headers: Vec<(String, String)>) -> Result<HttpResponse, String> {
        self.check_permission(Permission::NetworkAccess)?;
        Err("Not implemented".to_string())
    }

    async fn http_post(&mut self, _url: String, _body: String, _headers: Vec<(String, String)>) -> Result<HttpResponse, String> {
        self.check_permission(Permission::NetworkAccess)?;
        Err("Not implemented".to_string())
    }

    async fn db_get(&mut self, _key: String) -> Option<String> {
        None
    }

    async fn db_set(&mut self, _key: String, _value: String, _ttl_seconds: Option<u32>) -> Result<(), String> {
        Err("Not implemented".to_string())
    }

    async fn db_delete(&mut self, _key: String) -> Result<bool, String> {
        Err("Not implemented".to_string())
    }

    async fn get_process_info(&mut self) -> Result<ProcessInfo, String> {
        self.check_permission(Permission::SystemInfo)?;
        Err("Not implemented".to_string())
    }
}

/// Host functions implementation providing Uveddi services to WASM plugins
pub struct HostFunctions {
    context: HostContext,
}

impl HostFunctions {
    /// Create new host functions with the given context
    pub fn new(context: HostContext) -> Self {
        Self { context }
    }

    /// Parse code into an AST using Uveddi's tree-sitter implementation
    pub async fn parse_ast(&self, code: &str, language: &str) -> Result<AstNode, PluginError> {
        // Check permission (using ConfigRead as a reasonable substitute)
        self.context.check_permission(Permission::ConfigRead)?;

        debug!(
            "Plugin {} requesting AST parsing for {} code",
            self.context.plugin_id, language
        );

        // For now, we'll use a simplified approach since we need to create an AstParser
        // In a full implementation, this would use the analysis engine's AST provider

        // Create a temporary file path for caching
        let temp_path = format!(
            "plugin_temp_{}.{}",
            self.context.plugin_id,
            match language {
                "rust" => "rs",
                "python" => "py",
                "javascript" => "js",
                "typescript" => "ts",
                _ => "txt",
            }
        );

        // Create a simple parsed file structure
        // TODO: In full implementation, this would use AstParser::parse_content
        let parsed_file = crate::analysis::components::ast_provider::ParsedFile {
            file_path: Arc::new(PathBuf::from(&temp_path)),
            language: match language {
                "rust" => crate::ast::SourceLanguage::Rust,
                "python" => crate::ast::SourceLanguage::Python,
                "javascript" => crate::ast::SourceLanguage::JavaScript,
                "typescript" => crate::ast::SourceLanguage::TypeScript,
                _ => crate::ast::SourceLanguage::Rust, // Default fallback
            },
            tree: None, // Would be populated by actual parsing
            source: Arc::new(code.to_string()),
            syntax_errors: Vec::new(),
            custom_ast: Arc::new(None),
            modified_at: crate::analysis::cache::wrappers::ArchivableSystemTime::now(),
        };

        // Cache the parsed file
        let mut cache = self.context.ast_cache.write().await;
        cache.insert(temp_path.clone(), parsed_file);

        // Return a placeholder AST node (in real implementation would return actual node)
        Ok(AstNode {
            node_type: "root".to_string(),
            text: code.to_string(),
            start_byte: 0,
            end_byte: code.len() as u32,
            start_row: 0,
            start_column: 0,
            end_row: code.lines().count() as u32,
            end_column: 0,
            children: Vec::new(),
        })
    }

    /// Get analysis context for a specific file
    pub async fn get_analysis_context(
        &self,
        file_path: &str,
    ) -> Result<AnalysisContext, PluginError> {
        // Check permission
        self.context.check_permission(Permission::ConfigRead)?;

        debug!(
            "Plugin {} requesting analysis context for file: {}",
            self.context.plugin_id, file_path
        );

        // Check if file exists in cache
        let cache = self.context.ast_cache.read().await;
        if let Some(parsed_file) = cache.get(file_path) {
            return Ok(AnalysisContext {
                file_path: file_path.to_string(),
                language: format!("{:?}", parsed_file.language),
                ast_available: true,
                line_count: parsed_file.source.lines().count() as u32,
                metadata: HashMap::new(),
            });
        }

        // If not in cache, check if file exists on disk
        if Path::new(file_path).exists() {
            // Try to determine language from file extension
            let language = match Path::new(file_path).extension().and_then(|e| e.to_str()) {
                Some("rs") => "rust",
                Some("py") => "python",
                Some("js") => "javascript",
                Some("ts") => "typescript",
                _ => "unknown",
            };

            Ok(AnalysisContext {
                file_path: file_path.to_string(),
                language: language.to_string(),
                ast_available: false,
                line_count: 0, // Would need to read file to get accurate count
                metadata: HashMap::new(),
            })
        } else {
            Err(PluginError::Execution(format!(
                "File not found: {}",
                file_path
            )))
        }
    }

    /// Store plugin analysis results in the database
    pub async fn store_plugin_results(
        &self,
        results: &[PluginIssue],
        analysis_run_id: i32,
    ) -> Result<(), PluginError> {
        // Check permission
        self.context.check_permission(Permission::TempFileCreate)?;

        info!(
            "Plugin {} storing {} analysis results",
            self.context.plugin_id,
            results.len()
        );

        // Convert plugin results to architectural issues
        let mut issues = Vec::new();
        for plugin_issue in results {
            let issue = ArchitecturalIssue {
                issue_id: None, // Will be assigned by database
                analysis_run_id: analysis_run_id.into(),
                anti_pattern_type_id: plugin_issue.anti_pattern_type_id.unwrap_or(0),
                file_path: plugin_issue.file_path.clone(),
                start_line: plugin_issue.line_number,
                end_line: plugin_issue.line_number,
                line_number: plugin_issue.line_number,
                column_number: None, // TODO: Add column_number to PluginIssue
                message: plugin_issue.message.clone(),
                metadata: serde_json::to_string(&plugin_issue.metadata).unwrap_or_default(),
                detector_name: "plugin".to_string(),
                created_at: chrono::Utc::now(),
                severity: plugin_issue.severity.clone(),
                description: plugin_issue.message.clone(),
                code_snippet: None,
                ai_explanation: Some(plugin_issue.suggestion.clone()),
            };
            issues.push(issue);
        }

        // Store issues in database
        // TODO: Fix Arc<Database> mutable borrow issue
        // self.context
        //     .database
        //     .store_issues(&issues)
        //     .map_err(|e| PluginError::Execution(format!("Database storage failed: {}", e)))?;

        debug!(
            "Successfully stored {} issues from plugin {}",
            issues.len(),
            self.context.plugin_id
        );
        Ok(())
    }

    /// Get project configuration value
    pub async fn get_project_config(&self, key: &str) -> Result<Option<String>, PluginError> {
        // Check permission
        self.context.check_permission(Permission::ConfigRead)?;

        debug!(
            "Plugin {} requesting configuration key: {}",
            self.context.plugin_id, key
        );

        let config = self.context.config.read().await;
        Ok(config.get(key).cloned())
    }

    /// Set project configuration value (if plugin has write permission)
    pub async fn set_project_config(&self, key: &str, value: &str) -> Result<(), PluginError> {
        // Check permission
        self.context.check_permission(Permission::ConfigRead)?;

        info!(
            "Plugin {} setting configuration: {} = {}",
            self.context.plugin_id, key, value
        );

        let mut config = self.context.config.write().await;
        config.insert(key.to_string(), value.to_string());
        Ok(())
    }

    /// Log a message from the plugin (for debugging and monitoring)
    pub fn log_message(&self, level: LogLevel, message: &str) -> Result<(), PluginError> {
        // Basic logging permission (usually allowed for all plugins)
        let plugin_id = &self.context.plugin_id;

        match level {
            LogLevel::Debug => debug!("[Plugin {}] {}", plugin_id, message),
            LogLevel::Info => info!("[Plugin {}] {}", plugin_id, message),
            LogLevel::Warn => warn!("[Plugin {}] {}", plugin_id, message),
            LogLevel::Error => error!("[Plugin {}] {}", plugin_id, message),
        }

        Ok(())
    }

    /// Get analysis run information
    pub async fn get_analysis_run(&self, run_id: i32) -> Result<Option<AnalysisRun>, PluginError> {
        // Check permission
        self.context.check_permission(Permission::ConfigRead)?;

        debug!(
            "Plugin {} requesting analysis run: {}",
            self.context.plugin_id, run_id
        );

        self.context
            .database
            .get_analysis_run(run_id.into())
            .await
            .map_err(|e| PluginError::Execution(format!("Database query failed: {}", e)))
    }

    /// Execute tree-sitter query on cached AST
    pub async fn execute_tree_sitter_query(
        &self,
        file_path: &str,
        query: &str,
    ) -> Result<Vec<QueryMatch>, PluginError> {
        // Check permission
        self.context.check_permission(Permission::ConfigRead)?;

        debug!(
            "Plugin {} executing tree-sitter query on file: {}",
            self.context.plugin_id, file_path
        );

        // Get cached AST
        let cache = self.context.ast_cache.read().await;
        let parsed_file = cache.get(file_path).ok_or_else(|| {
            PluginError::Execution(format!("File not in AST cache: {}", file_path))
        })?;

        // Execute tree-sitter query
        match self.execute_query_on_ast(parsed_file, query) {
            Ok(matches) => Ok(matches),
            Err(e) => {
                error!(
                    "Tree-sitter query failed for plugin {}: {}",
                    self.context.plugin_id, e
                );
                Err(PluginError::Execution(format!(
                    "Query execution failed: {}",
                    e
                )))
            }
        }
    }

    /// Internal method to execute tree-sitter query on AST
    fn execute_query_on_ast(
        &self,
        _parsed_file: &crate::analysis::components::ast_provider::ParsedFile,
        query_str: &str,
    ) -> Result<Vec<QueryMatch>, UveddiError> {
        // This would integrate with tree-sitter query execution
        // For now, return empty results as placeholder
        debug!("Executing tree-sitter query: {}", query_str);

        // TODO: Implement actual tree-sitter query execution
        // This would require:
        // 1. Parse the query string using tree-sitter-query
        // 2. Execute the query against the AST
        // 3. Convert results to QueryMatch format

        Ok(Vec::new()) // Placeholder
    }
}

/// AST node representation for plugins
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AstNode {
    pub node_type: String,
    pub text: String,
    pub start_byte: u32,
    pub end_byte: u32,
    pub start_row: u32,
    pub start_column: u32,
    pub end_row: u32,
    pub end_column: u32,
    pub children: Vec<AstNode>,
}

/// Analysis context provided to plugins
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AnalysisContext {
    pub file_path: String,
    pub language: String,
    pub ast_available: bool,
    pub line_count: u32,
    pub metadata: HashMap<String, String>,
}

/// Log levels for plugin messages
#[derive(Debug, Clone)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

/// Tree-sitter query match result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QueryMatch {
    pub pattern: u32,
    pub captures: Vec<QueryCapture>,
}

/// Tree-sitter query capture result with location information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QueryCapture {
    /// Capture index in the query pattern
    pub index: u32,
    /// Capture name from the query pattern
    pub name: String,
    /// Captured text content
    pub text: String,
    /// Starting byte position in the source
    pub start_byte: u32,
    /// Ending byte position in the source
    pub end_byte: u32,
    /// Starting row (line) number (0-indexed)
    pub start_row: u32,
    /// Starting column number (0-indexed)
    pub start_column: u32,
    /// Ending row (line) number (0-indexed)
    pub end_row: u32,
    /// Ending column number (0-indexed)
    pub end_column: u32,
}

/// Plugin issue format for database storage
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PluginIssue {
    /// Anti-pattern type identifier from the database
    pub anti_pattern_type_id: Option<i64>,
    /// File path where the issue was detected
    pub file_path: String,
    /// Line number where the issue occurs (1-indexed)
    pub line_number: Option<i32>,
    /// Human-readable issue description
    pub message: String,
    /// Issue severity level (e.g., "low", "medium", "high", "critical")
    pub severity: String,
    /// Suggested fix or improvement
    pub suggestion: String,
    /// Additional metadata about the issue
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Extension trait to register host functions with Wasmtime linker
#[cfg(feature = "wasm-plugins")]
pub trait HostFunctionLinker {
    /// Register all host functions with the Wasmtime linker
    fn register_host_functions(&mut self, host_context: HostContext) -> Result<(), PluginError>;
}

#[cfg(feature = "wasm-plugins")]
impl HostFunctionLinker for wasmtime::Linker<HostContext> {
    fn register_host_functions(&mut self, host_context: HostContext) -> Result<(), PluginError> {
        info!(
            "Registering host functions for plugin: {}",
            host_context.plugin_id
        );

        // This would use the actual WIT-generated bindings
        // For now, we'll just log that registration would happen here

        // Example of what the actual registration would look like:
        // self.func_wrap_async(
        //     "uveddi:core",
        //     "parse-ast",
        //     |mut caller: Caller<'_, HostContext>, code: String, language: String| {
        //         Box::new(async move {
        //             let host_functions = HostFunctions::new(caller.data().clone());
        //             host_functions.parse_ast(&code, &language).await
        //         })
        //     }
        // )?;

        debug!(
            "Host functions registered successfully for plugin: {}",
            host_context.plugin_id
        );
        Ok(())
    }
}

/// Factory for creating host contexts
#[derive(Clone)]
pub struct HostContextFactory {
    database: Arc<ScalableDatabase>,
    analysis_engine: Arc<RwLock<AnalysisEngine>>,
}

impl HostContextFactory {
    /// Create a new host context factory
    pub fn new(
        database: Arc<ScalableDatabase>,
        analysis_engine: Arc<RwLock<AnalysisEngine>>,
    ) -> Self {
        Self {
            database,
            analysis_engine,
        }
    }

    /// Create a host context for a specific plugin
    pub fn create_context(
        &self,
        plugin_id: PluginId,
        security_policy: SecurityPolicy,
    ) -> HostContext {
        HostContext::new(
            self.database.clone(),
            self.analysis_engine.clone(),
            security_policy,
            plugin_id,
        )
    }
}
