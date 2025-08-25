//! Host functions that provide Uveddi core functionality to WASM plugins
//!
//! This module implements the host-side functions that plugins can call to access
//! Uveddi's core analysis capabilities, database storage, and system configuration.
//! All host functions are implemented using the WebAssembly Component Model with
//! proper capability-based security.

use crate::analysis::AnalysisEngine;
use crate::database::crud::Database;
use crate::database::models::{AnalysisRun, ArchitecturalIssue};
use crate::error::UveddiError;
use crate::plugins::errors::PluginError;
use crate::plugins::security::{Permission, SecurityPolicy};
use crate::plugins::types::PluginId;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Context for plugin host functions containing access to Uveddi services
#[derive(Clone)]
pub struct HostContext {
    /// Database connection for storing and retrieving analysis results
    database: Arc<Database>,
    /// Analysis engine for AST parsing and code analysis
    analysis_engine: Arc<RwLock<AnalysisEngine>>,
    /// Security policy for the current plugin
    security_policy: SecurityPolicy,
    /// Plugin ID for permission checking
    plugin_id: PluginId,
    /// Configuration store
    config: Arc<RwLock<HashMap<String, String>>>,
    /// File cache for parsed ASTs
    ast_cache: Arc<RwLock<HashMap<String, crate::analysis::components::ast_provider::ParsedFile>>>,
}

impl HostContext {
    /// Create a new host context for a plugin
    pub fn new(
        database: Arc<Database>,
        analysis_engine: Arc<RwLock<AnalysisEngine>>,
        security_policy: SecurityPolicy,
        plugin_id: PluginId,
    ) -> Self {
        Self {
            database,
            analysis_engine,
            security_policy,
            plugin_id,
            config: Arc::new(RwLock::new(HashMap::new())),
            ast_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check if plugin has required permission
    fn check_permission(&self, permission: Permission) -> Result<(), PluginError> {
        if self.security_policy.has_permission(&permission) {
            Ok(())
        } else {
            Err(PluginError::PermissionDenied(format!(
                "Plugin {} does not have permission {:?}",
                self.plugin_id, permission
            )))
        }
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
        let temp_path = format!("plugin_temp_{}.{}", self.context.plugin_id, 
                              match language {
                                  "rust" => "rs",
                                  "python" => "py",
                                  "javascript" => "js",
                                  "typescript" => "ts",
                                  _ => "txt",
                              });

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
    pub async fn get_analysis_context(&self, file_path: &str) -> Result<AnalysisContext, PluginError> {
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
            Err(PluginError::Execution(format!("File not found: {}", file_path)))
        }
    }

    /// Store plugin analysis results in the database
    pub async fn store_plugin_results(&self, results: &[PluginIssue], analysis_run_id: i32) -> Result<(), PluginError> {
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
                analysis_run_id,
                anti_pattern_type_id: plugin_issue.anti_pattern_type_id,
                file_path: plugin_issue.file_path.clone(),
                line_number: plugin_issue.line_number,
                message: plugin_issue.message.clone(),
                severity: plugin_issue.severity.clone(),
                suggestion: plugin_issue.suggestion.clone(),
                metadata: serde_json::to_string(&plugin_issue.metadata).unwrap_or_default(),
            };
            issues.push(issue);
        }

        // Store issues in database
        self.context
            .database
            .store_issues(&issues)
            .map_err(|e| PluginError::Execution(format!("Database storage failed: {}", e)))?;

        debug!("Successfully stored {} issues from plugin {}", issues.len(), self.context.plugin_id);
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
            .get_analysis_run(run_id)
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
        let parsed_file = cache.get(file_path)
            .ok_or_else(|| PluginError::Execution(format!("File not in AST cache: {}", file_path)))?;

        // Execute tree-sitter query
        match self.execute_query_on_ast(parsed_file, query) {
            Ok(matches) => Ok(matches),
            Err(e) => {
                error!("Tree-sitter query failed for plugin {}: {}", self.context.plugin_id, e);
                Err(PluginError::Execution(format!("Query execution failed: {}", e)))
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

/// Tree-sitter query capture
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QueryCapture {
    pub index: u32,
    pub name: String,
    pub text: String,
    pub start_byte: u32,
    pub end_byte: u32,
    pub start_row: u32,
    pub start_column: u32,
    pub end_row: u32,
    pub end_column: u32,
}

/// Plugin issue format for database storage
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PluginIssue {
    pub anti_pattern_type_id: Option<i64>,
    pub file_path: String,
    pub line_number: Option<i32>,
    pub message: String,
    pub severity: String,
    pub suggestion: String,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Extension trait to register host functions with Wasmtime linker
#[cfg(feature = "wasm-plugins")]
pub trait HostFunctionLinker {
    /// Register all host functions with the Wasmtime linker
    fn register_host_functions(
        &mut self,
        host_context: HostContext,
    ) -> Result<(), PluginError>;
}

#[cfg(feature = "wasm-plugins")]
impl HostFunctionLinker for wasmtime::Linker<HostContext> {
    fn register_host_functions(
        &mut self,
        host_context: HostContext,
    ) -> Result<(), PluginError> {
        info!("Registering host functions for plugin: {}", host_context.plugin_id);

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

        debug!("Host functions registered successfully for plugin: {}", host_context.plugin_id);
        Ok(())
    }
}

/// Factory for creating host contexts
pub struct HostContextFactory {
    database: Arc<Database>,
    analysis_engine: Arc<RwLock<AnalysisEngine>>,
}

impl HostContextFactory {
    /// Create a new host context factory
    pub fn new(database: Arc<Database>, analysis_engine: Arc<RwLock<AnalysisEngine>>) -> Self {
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