//! # Analysis Module
//!
//! The analysis module provides a comprehensive framework for detecting anti-patterns,
//! architectural issues, and code quality problems across multiple programming languages.
//!
//! ## Architecture Overview
//!
//! The analysis system is built around a trait-based architecture that allows for
//! pluggable detectors. Each detector implements the `AnalysisDetector` trait and
//! can operate on either individual files (via AST analysis) or the entire
//! dependency graph.
//!
//! ### Key Components:
//!
//! - **AnalysisEngine**: Core orchestrator that runs detectors and collects results
//! - **AnalysisDetector**: Trait that all detectors must implement
//! - **Anti-pattern Detectors**: Specialized detectors for various code quality issues
//! - **Dependency Graph**: Graph-based analysis for architectural patterns
//! - **AST Cache**: Caching layer for parsed syntax trees
//!
//! ### Pluggable Detector System
//!
//! The detector system is designed to be modular and extensible:
//!
//! 1. **File-level Analysis**: Detectors operate on individual `ParsedFile` instances
//! 2. **Graph-level Analysis**: Detectors can analyze the entire dependency graph
//! 3. **Language-specific Support**: Detectors can be specialized for specific languages
//! 4. **Multi-language Support**: Some detectors work across multiple languages
//!
//! ### Integration with Analysis Engine
//!
//! Detectors integrate with the engine through:
//!
//! - **Registration**: Detectors are registered with the engine
//! - **Execution**: Engine calls `detect_issues()` for each file
//! - **Result Collection**: Engine aggregates results from all detectors
//! - **Reporting**: Results are stored in the database for analysis
//!
/// ## Usage Examples
///
/// ### Basic Analysis
///
/// ```rust
/// use uveddi::analysis::AnalysisEngine;
/// use std::path::Path;
///
/// # async fn example() -> uveddi::Result<()> {
/// // Create engine with built-in detectors
/// let engine = AnalysisEngine::new()?;
///
/// // Analyze a project directory
/// let (issues, graph) = engine.analyze(Path::new("src/")).await?;
/// println!("Found {} issues across {} files", issues.len(), graph.nodes().count());
/// # Ok(())
/// # }
/// ```
///
/// ### Custom Detector Configuration
///
/// ```rust
/// use uveddi::analysis::{AnalysisEngine, detectors::anti_patterns::GodObjectDetector};
/// use uveddi::analysis::config::AnalysisConfig;
///
/// # async fn example() -> uveddi::Result<()> {
/// let config = AnalysisConfig {
///     max_file_size: 1024 * 1024, // 1MB limit
///     parallel_analysis: true,
///     cache_enabled: true,
///     ..Default::default()
/// };
///
/// let engine = AnalysisEngine::builder()
///     .with_config(config)
///     .with_detector(GodObjectDetector::default())
///     .build()?;
///
/// let (issues, graph) = engine.analyze("src/").await?;
/// # Ok(())
/// # }
/// ```
///
/// ### Performance Optimized Analysis
///
/// ```rust
/// use uveddi::analysis::{AnalysisEngine, cache::AstCache};
/// use std::sync::Arc;
///
/// # async fn example() -> uveddi::Result<()> {
/// // Shared cache across multiple analysis runs
/// let cache = Arc::new(AstCache::with_capacity(1000)?);
///
/// let engine = AnalysisEngine::builder()
///     .with_cache(cache.clone())
///     .with_parallel_processing(true)
///     .build()?;
///
/// // Multiple analysis runs will benefit from cache
/// let (issues1, _) = engine.analyze("src/").await?;
/// let (issues2, _) = engine.analyze("tests/").await?;
/// # Ok(())
/// # }
/// ```
pub mod adapters;
pub mod buffer;
pub mod cache;
pub mod cfg;
pub mod component_extractor;
pub mod components;
pub mod config;
pub mod config_migration;
pub mod detector_factory;
pub mod detector_registry;
pub mod detectors;
pub mod diagram_cache;
pub mod engine;
pub mod engine_builder;
pub mod errors;
pub mod extractors;
pub mod file_discovery;
pub mod graph;
pub mod incremental;
pub mod interactive_diagram_generator;
pub mod memory;
pub mod memory_report;
pub mod mermaid_generator;
pub mod orchestrator;
pub mod performance;
pub mod plugin_adapter;
pub mod robust_parser;
pub mod semantic;
pub mod services;
pub mod standardized_config;
pub mod symbols;
pub mod traits;
pub mod types;
pub mod workspace;

#[cfg(test)]
pub mod tests;

pub use adapters::{AstParserAdapter, DependencyExtractorAdapter, ResultCacheAdapter};
/// Placeholder documentation for public items
// Re-exports for convenience
pub use cache::AstCache;
pub use config::{AnalysisConfig, CacheConfig, PerformanceConfig};
pub use config_migration::{ConfigMigration, ConfigMigrationUtils};
pub use detector_factory::{
    DetectorConfig, DetectorFactory, DetectorThresholds, EnhancedDetectorConfig, IssueSeverity,
};
pub use detector_registry::DetectorRegistry;
pub use detectors::anti_patterns::GodObjectDetector;
pub use detectors::{CycleDetector, Dependency, DependencyExtractor};
pub use diagram_cache::{
    CachedDiagram, CompressionEngine, DiagramCacheEngine, DiagramDependencyTracker, DiagramType,
    InvalidationManager,
};
pub use engine::AnalysisEngine;
pub use engine_builder::AnalysisEngineBuilder;
pub use errors::AnalysisError;
pub use graph::{ComponentNode, LocalDependencyGraph, LocalDependencyType};
pub use incremental::{
    ChangeDetector, ChangeImpact, ChangeSet, DependencyTracker, IncrementalAnalysisConfig,
    IncrementalAnalysisEngine, IncrementalConfig, IncrementalStateManager,
};
pub use interactive_diagram_generator::{
    ComponentMetrics as InteractiveComponentMetrics, InteractionConfig,
    InteractiveDiagramGenerator, InteractiveDiagramResult, InteractiveNodeMetadata, NodePosition,
};
pub use memory::{get_optimization_status, MemoryOptimizationConfig};
pub use memory_report::{MemoryAnalysisReport, PhaseMemoryBreakdown};
pub use orchestrator::{AnalysisOrchestrator, AnalysisOrchestratorBuilder, AnalysisOptions, EnhancedAnalysisResult, OrchestratorStatus};
pub use plugin_adapter::{WasmPluginAdapterFactory, WasmPluginDetectorAdapter};
pub use services::{AnalysisService, DependencyAnalysisService, PerformanceAnalysisService};
pub use standardized_config::{
    constants, AdvancedConfig, ConfigValue, DetectorMetadata, ExclusionConfig,
    StandardConfigBuilder, StandardDetectorConfig,
};
pub use traits::{AstParserTrait, CacheStats, DependencyExtractorTrait, ResultCacheTrait};
pub use workspace::{CrateInfo, WorkspaceDetector, WorkspaceInfo};

use crate::ast::ParsedFile;
use crate::database::models::{AntiPatternType, ArchitecturalIssue};
use async_trait::async_trait;

/// Core analysis trait for all detectors
///
/// The `AnalysisDetector` trait defines the interface that all analysis detectors
/// must implement. This trait enables a pluggable architecture where different
/// detectors can be easily added, removed, or replaced without affecting the
/// core analysis engine.
///
/// ## Required Methods
///
/// - `detect_issues()`: Analyze a single parsed file for issues
/// - `get_anti_pattern_types()`: Return the types of anti-patterns this detector finds
/// - `get_detector_name()`: Return a unique identifier for this detector
///
/// ## Optional Methods
///
/// - `detect_graph_issues()`: Analyze the entire dependency graph (default: no-op)
/// - `detect()`: Unified detection method for backward compatibility
///
/// ## Implementation Guidelines
///
/// 1. **Performance**: Detectors should be efficient as they run on every file
/// 2. **Language Support**: Use `ParsedFile.language` to provide language-specific logic
/// 3. **Error Handling**: Return `AnalysisError` for serious issues, empty Vec for no findings
/// 4. **Caching**: Leverage the AST cache for expensive parsing operations
/// 5. **Thread Safety**: All detectors must be `Send + Sync` for parallel analysis
/// 6. **Memory Efficiency**: Avoid holding large data structures in detector state
///
/// ## Example Implementation
///
/// ```rust
/// use uveddi::analysis::{AnalysisDetector, AnalysisError};
/// use uveddi::ast::ParsedFile;
/// use uveddi::database::models::{AntiPatternType, ArchitecturalIssue};
///
/// pub struct CustomDetector {
///     threshold: usize,
/// }
///
/// impl AnalysisDetector for CustomDetector {
///     fn detect_issues(&self, file: &ParsedFile) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
///         let mut issues = Vec::new();
///         
///         // Analyze the file based on language
///         match file.language {
///             crate::ast::SourceLanguage::Rust => {
///                 // Rust-specific analysis
///                 if let Some(issue) = self.analyze_rust_file(file)? {
///                     issues.push(issue);
///                 }
///             }
///             crate::ast::SourceLanguage::Python => {
///                 // Python-specific analysis
///                 if let Some(issue) = self.analyze_python_file(file)? {
///                     issues.push(issue);
///                 }
///             }
///             _ => {
///                 // Generic analysis for other languages
///             }
///         }
///         
///         Ok(issues)
///     }
///
///     fn get_anti_pattern_types(&self) -> Vec<AntiPatternType> {
///         vec![AntiPatternType {
///             anti_pattern_type_id: Some(99),
///             name: "Custom Pattern".to_string(),
///             description: "Custom anti-pattern detection".to_string(),
///             category: "Custom".to_string(),
///         }]
///     }
///
///     fn get_detector_name(&self) -> &'static str {
///         "CustomDetector"
///     }
/// }
///
/// impl CustomDetector {
///     pub fn new(threshold: usize) -> Self {
///         Self { threshold }
///     }
///     
///     fn analyze_rust_file(&self, file: &ParsedFile) -> Result<Option<ArchitecturalIssue>, AnalysisError> {
///         // Implementation specific to Rust files
///         Ok(None)
///     }
///     
///     fn analyze_python_file(&self, file: &ParsedFile) -> Result<Option<ArchitecturalIssue>, AnalysisError> {
///         // Implementation specific to Python files  
///         Ok(None)
///     }
/// }
/// ```
///
/// ## Usage in Analysis Engine
///
/// ```rust
/// use uveddi::analysis::AnalysisEngine;
///
/// # async fn example() -> uveddi::Result<()> {
/// let engine = AnalysisEngine::builder()
///     .with_detector(CustomDetector::new(10))
///     .build()?;
///
/// let (issues, graph) = engine.analyze("src/").await?;
/// # Ok(())
/// # }
/// ```
#[async_trait]
pub trait AnalysisDetector: Send + Sync {
    async fn detect_issues(
        &self,
        file: &ParsedFile,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError>;
    fn detect_graph_issues(
        &self,
        _graph: &LocalDependencyGraph,
        _analysis_run_id: i64,
    ) -> Vec<ArchitecturalIssue> {
        // Default implementation for detectors that don't analyze the graph
        vec![]
    }
    fn detect(&self, graph: &LocalDependencyGraph) -> Vec<ArchitecturalIssue> {
        // Unified detection method for pluggable system
        self.detect_graph_issues(graph, 0i64)
    }
    fn get_anti_pattern_types(&self) -> Vec<AntiPatternType>;
    fn get_detector_name(&self) -> &'static str;
}
