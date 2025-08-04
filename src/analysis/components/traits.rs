//! Component trait definitions for the AnalysisEngine refactoring
//!
//! These traits define the public interfaces for each component in the
//! decomposed AnalysisEngine architecture.

use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::analysis::detectors::dependency::Dependency;
use crate::analysis::graph::dependency::LocalDependencyGraph;
use crate::database::models::ArchitecturalIssue;
use crate::error::UveddiError;

/// Provides access to cached Abstract Syntax Trees, parsing on-demand.
#[async_trait]
pub trait AstProvider: Send + Sync {
    /// Retrieves the AST for a given file path.
    /// If the AST is not in the cache, it parses the file and caches the result.
    async fn get_ast(&self, file_path: &Path) -> Result<Arc<crate::ast::tree_sitter::Tree>, UveddiError>;

    /// Parses a file and returns a ParsedFile structure
    async fn parse_file(&self, file_path: &Path) -> Result<Arc<crate::analysis::components::ast_provider::ParsedFile>, UveddiError>;

    /// Clears the AST cache
    fn clear_cache(&self);

    /// Returns cache statistics for observability
    fn get_cache_metrics(&self) -> serde_json::Value;
}

/// Constructs the project's dependency graph from source files.
#[async_trait]
pub trait DependencyGraphBuilder: Send + Sync {
    /// Builds or updates the full dependency graph for the project, starting from a root path.
    async fn build_graph(&self, root_path: &Path) -> Result<LocalDependencyGraph, UveddiError>;

    /// Builds a dependency graph from a collection of dependencies
    fn build_from_dependencies(&self, dependencies: Vec<Dependency>) -> LocalDependencyGraph;
}

/// Central configuration service for all tool settings.
pub trait ConfigurationService: Send + Sync {
    /// Retrieves a configuration value by key
    fn get_config_value(&self, key: &str) -> Option<String>;

    /// Checks if a detector is enabled
    fn is_detector_enabled(&self, detector_name: &str) -> bool;

    /// Gets plugin configuration
    fn get_plugin_config(&self) -> HashMap<String, serde_json::Value>;

    /// Gets cache configuration
    fn get_cache_path(&self) -> Option<PathBuf>;

    /// Checks if plugins are enabled globally
    fn are_plugins_enabled(&self) -> bool;
}

/// Schedules and orchestrates analysis detectors.
#[async_trait]
pub trait DetectorScheduler: Send + Sync {
    /// Schedules analysis on a single file
    async fn schedule_file(&self, file_path: &Path)
        -> Result<Vec<ArchitecturalIssue>, UveddiError>;

    /// Schedules analysis on a directory tree
    async fn schedule_directory(
        &self,
        dir_path: &Path,
    ) -> Result<Vec<ArchitecturalIssue>, UveddiError>;

    /// Schedules graph-level analysis
    async fn schedule_graph_analysis(
        &self,
        graph: &LocalDependencyGraph,
    ) -> Result<Vec<ArchitecturalIssue>, UveddiError>;

    /// Schedules analysis for a SourceFile (wrapper around schedule_file)
    async fn schedule_file_analysis(
        &self,
        source_file: &crate::analysis::file_discovery::SourceFile,
    ) -> Result<Vec<ArchitecturalIssue>, UveddiError>;
}

/// Aggregates analysis results from multiple sources.
pub trait AnalysisAggregator: Send + Sync {
    /// Records a finding from analysis
    fn record_finding(&self, issue: ArchitecturalIssue);

    /// Records multiple findings
    fn record_findings(&self, issues: Vec<ArchitecturalIssue>);

    /// Retrieves all recorded findings
    fn get_findings(&self) -> Vec<ArchitecturalIssue>;

    /// Clears all recorded findings
    fn clear_findings(&self);

    /// Records that a file has been processed
    fn record_file_processed(&self);

    /// Gets aggregation statistics
    fn get_stats(&self) -> AggregationStats;
}

/// Statistics about the aggregation process
#[derive(Debug, Clone)]
pub struct AggregationStats {
    pub total_findings: usize,
    pub findings_by_type: HashMap<String, usize>,
    pub files_processed: usize,
}

/// Plugin manager actor commands for asynchronous communication
#[derive(Debug)]
pub enum PluginCommand {
    /// Execute a plugin on a file
    Execute {
        plugin_id: String,
        source_file_path: PathBuf,
        ast: Arc<crate::ast::tree_sitter::Tree>,
        responder: tokio::sync::oneshot::Sender<Result<Vec<ArchitecturalIssue>, UveddiError>>,
    },
    /// Load a plugin
    LoadPlugin {
        plugin_path: PathBuf,
        responder: tokio::sync::oneshot::Sender<Result<String, UveddiError>>,
    },
    /// Unload a plugin
    UnloadPlugin {
        plugin_id: String,
        responder: tokio::sync::oneshot::Sender<Result<(), UveddiError>>,
    },
    /// Get plugin statistics
    GetStats {
        responder: tokio::sync::oneshot::Sender<Result<PluginStats, UveddiError>>,
    },
    /// List loaded plugins
    ListLoadedPlugins {
        responder: tokio::sync::oneshot::Sender<Result<Vec<String>, UveddiError>>,
    },
    /// Get plugin adapter
    GetPluginAdapter {
        plugin_id: String,
        responder: tokio::sync::oneshot::Sender<
            Result<Option<crate::analysis::WasmPluginDetectorAdapter>, UveddiError>,
        >,
    },
}

/// Plugin statistics for monitoring
#[derive(Debug, Clone)]
pub struct PluginStats {
    pub loaded_plugins: usize,
    pub total_executions: u64,
    pub average_execution_time_ms: f64,
    pub memory_usage_bytes: u64,
}

/// Handle for communicating with the plugin manager actor
pub trait PluginManagerHandle: Send + Sync {
    /// Execute a plugin on a file
    async fn execute_plugin(
        &self,
        plugin_id: String,
        source_file_path: PathBuf,
        ast: Arc<crate::ast::tree_sitter::Tree>,
    ) -> Result<Vec<ArchitecturalIssue>, UveddiError>;

    /// Load a plugin from a file path
    async fn load_plugin(&self, plugin_path: PathBuf) -> Result<String, UveddiError>;

    /// Unload a plugin by ID
    async fn unload_plugin(&self, plugin_id: String) -> Result<(), UveddiError>;

    /// Get plugin statistics
    async fn get_stats(&self) -> Result<PluginStats, UveddiError>;

    /// List loaded plugins
    async fn list_loaded_plugins(&self) -> Result<Vec<String>, UveddiError>;

    /// Get plugin adapter for a specific plugin
    async fn get_plugin_adapter(
        &self,
        plugin_id: &str,
    ) -> Result<Option<crate::analysis::WasmPluginDetectorAdapter>, UveddiError>;
}
