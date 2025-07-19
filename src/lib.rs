#![allow(warnings)]
//! # Uveddi - Architectural Analysis Tool
//!
//! Uveddi is a comprehensive architectural analysis tool that combines static code analysis
//! with AI-powered insights to help developers understand and improve their codebases.
//!
//! ## Features
//!
//! - **Multi-language AST parsing**: Support for analyzing Rust, Python, JavaScript, and TypeScript
//! - **AI-powered analysis**: Integration with Ollama and other AI providers for intelligent code insights
//! - **Dependency analysis**: Track and visualize code dependencies with graph-based analysis
//! - **Quality metrics**: Calculate maintainability and complexity scores with anti-pattern detection
//! - **Report generation**: Create detailed analysis reports in multiple formats (JSON, HTML, Mermaid)
//! - **Caching system**: Efficient AST caching for improved performance on large codebases
//! - **Plugin system**: WebAssembly-based plugin architecture for extensibility
//! - **Terminal UI**: Interactive terminal interface for analysis and exploration
//!
//! ## Quick Start
//!
//! ### Basic Analysis
//!
//! ```rust,no_run
//! use uveddi::analysis::AnalysisEngine;
//! use std::path::Path;
//!
//! # async fn example() -> uveddi::Result<()> {
//! // Create an analysis engine with default detectors
//! let engine = AnalysisEngine::new()?;
//!
//! // Analyze a project directory
//! let (issues, dependency_graph) = engine.analyze(Path::new("src/")).await?;
//!
//! println!("Found {} architectural issues", issues.len());
//! println!("Analyzed {} files", dependency_graph.nodes().count());
//! # Ok(())
//! # }
//! ```
//!
//! ### Using Configuration
//!
//! ```rust,no_run
//! use uveddi::analysis::{AnalysisEngine, config::AnalysisConfig};
//! use uveddi::analysis::detectors::anti_patterns::GodObjectDetector;
//! use std::path::Path;
//!
//! # async fn example() -> uveddi::Result<()> {
//! // Create custom configuration
//! let config = AnalysisConfig {
//!     max_file_size: 1024 * 1024, // 1MB limit
//!     parallel_analysis: true,
//!     cache_enabled: true,
//!     ..Default::default()
//! };
//!
//! // Build engine with custom detector
//! let engine = AnalysisEngine::builder()
//!     .with_config(config)
//!     .with_detector(GodObjectDetector::default())
//!     .build()?;
//!
//! let (issues, graph) = engine.analyze(Path::new("src/")).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### AI-Powered Analysis
//!
//! ```rust,no_run
//! use uveddi::ai::engine::AiEngine;
//! use uveddi::ai::ollama_provider::OllamaProvider;
//! use uveddi::analysis::AnalysisEngine;
//!
//! # async fn example() -> uveddi::Result<()> {
//! // Set up AI provider (requires "local-ai" feature)
//! let ai_provider = OllamaProvider::new("http://localhost:11434")?;
//! let ai_engine = AiEngine::new(Box::new(ai_provider))?;
//!
//! // Analyze with AI explanations
//! let analysis_engine = AnalysisEngine::new()?;
//! let (issues, graph) = analysis_engine.analyze("src/").await?;
//!
//! // Get AI explanations for issues
//! for issue in issues {
//!     let explanation = ai_engine.explain_issue(&issue).await?;
//!     println!("AI Explanation: {}", explanation);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Feature Flags
//!
//! Uveddi uses feature flags for modular compilation. Enable only the features you need:
//!
//! ```toml
//! [dependencies]
//! # Full feature set (default)
//! uveddi = { version = "0.1", features = ["default"] }
//!
//! # Minimal analysis only
//! uveddi = { version = "0.1", features = ["analysis"] }
//!
//! # Analysis with AI support
//! uveddi = { version = "0.1", features = ["analysis", "local-ai"] }
//!
//! # Terminal UI only
//! uveddi = { version = "0.1", features = ["tui", "tree-sitter"] }
//! ```
//!
//! Available features:
//! - `default`: Full feature set including `local-ai`, `image-rendering`, `tree-sitter`
//! - `analysis`: Core analysis without tree-sitter parsing
//! - `tree-sitter`: AST parsing for Rust, Python, JavaScript, TypeScript
//! - `ai`: Base AI functionality for analysis explanations
//! - `local-ai`: Ollama integration (includes `ai`)
//! - `image-rendering`: Diagram generation service integration
//! - `wasm-plugins`: WebAssembly plugin system for extensibility
//! - `tui`: Terminal user interface for interactive analysis
//!
//! ## Performance Characteristics
//!
//! - **Time Complexity**: O(n) where n is the number of files
//! - **Memory Usage**: O(m) where m is the number of parsed ASTs (with caching)
//! - **Parallelization**: Enabled by default for multi-core analysis
//! - **Cache Efficiency**: AST parsing results cached for repeated analysis
//!
//! For large codebases (>1000 files), enable caching and parallel processing:
//!
//! ```rust,no_run
//! use uveddi::analysis::{AnalysisEngine, cache::AstCache};
//! use std::sync::Arc;
//!
//! # async fn example() -> uveddi::Result<()> {
//! let cache = Arc::new(AstCache::with_capacity(1000)?);
//! let engine = AnalysisEngine::builder()
//!     .with_cache(cache)
//!     .with_parallel_processing(true)
//!     .build()?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Error Handling
//!
//! All operations return `Result<T, UveddiError>` with comprehensive error context:
//!
//! ```rust,no_run
//! use uveddi::analysis::AnalysisEngine;
//! use uveddi::error::{UveddiError, ErrorCategory};
//!
//! # async fn example() {
//! match AnalysisEngine::new() {
//!     Ok(engine) => {
//!         // Use engine
//!     }
//!     Err(UveddiError::ConfigError(msg)) => {
//!         eprintln!("Configuration error: {}", msg);
//!     }
//!     Err(UveddiError::IoError(io_err)) => {
//!         eprintln!("File system error: {}", io_err);
//!     }
//!     Err(e) => {
//!         eprintln!("Analysis error: {}", e);
//!         eprintln!("Category: {:?}", e.category());
//!         eprintln!("Severity: {:?}", e.severity());
//!     }
//! }
//! # }
//! ```
//!
//! ## Architecture Overview
//!
//! The library is organized into several key modules:
//!
//! - [`ai`]: AI provider integrations for intelligent analysis and explanations
//! - [`analysis`]: Core analysis engines, detectors, and algorithms
//! - [`ast`]: Abstract Syntax Tree parsing and manipulation using tree-sitter
//! - [`database`]: Data persistence and querying capabilities with SQLite
//! - [`community`]: Community member management and analytics
//! - [`report`]: Report generation and formatting utilities
//! - [`plugins`]: WebAssembly-based plugin system for extensibility
//! - [`cache`]: Caching system for improved performance
//! - [`error`]: Comprehensive error handling and reporting
//! - [`tui`]: Terminal user interface for interactive analysis
//!
//! ## Common Use Cases
//!
//! ### 1. Code Quality Assessment
//! ```rust,no_run
//! use uveddi::analysis::AnalysisEngine;
//! use uveddi::analysis::detectors::anti_patterns::*;
//!
//! # async fn example() -> uveddi::Result<()> {
//! let engine = AnalysisEngine::builder()
//!     .with_detector(GodObjectDetector::default())
//!     .with_detector(DeadCodeDetector::default())
//!     .with_detector(CodeDuplicationDetector::default())
//!     .build()?;
//!
//! let (issues, _) = engine.analyze("src/").await?;
//! let quality_score = calculate_quality_score(&issues);
//! # Ok(())
//! # }
//! ```
//!
//! ### 2. Dependency Analysis
//! ```rust,no_run
//! use uveddi::analysis::AnalysisEngine;
//! use uveddi::analysis::graph::LocalDependencyGraph;
//!
//! # async fn example() -> uveddi::Result<()> {
//! let engine = AnalysisEngine::new()?;
//! let (_, graph) = engine.analyze("src/").await?;
//!
//! // Analyze dependency cycles
//! let cycles = graph.detect_cycles();
//! println!("Found {} dependency cycles", cycles.len());
//! # Ok(())
//! # }
//! ```
//!
//! ### 3. Plugin Development
//! ```rust,no_run
//! use uveddi::plugins::engine::PluginEngine;
//! use uveddi::plugins::types::PluginConfig;
//!
//! # async fn example() -> uveddi::Result<()> {
//! let plugin_engine = PluginEngine::new()?;
//! let config = PluginConfig {
//!     plugin_path: "path/to/plugin.wasm".into(),
//!     ..Default::default()
//! };
//!
//! plugin_engine.load_plugin(config).await?;
//! # Ok(())
//! # }
//! ```
//!
//! See individual module documentation for detailed usage information and examples.

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

pub mod ai;
pub mod analysis;
pub mod application;
pub mod ast;
pub mod cache;
pub mod cli;
pub mod community;
pub mod config;
pub mod constants;
pub mod database;
pub mod error;
pub mod ingestion;
pub mod models;
// TODO: Re-enable when monitoring dependencies are properly configured
pub mod monitoring;
pub mod observability;
pub mod plugins;
pub mod report;
pub mod resilience;
pub mod security;
pub mod semantic_search;
pub mod sla;
#[cfg(feature = "tui")]
pub mod tui;

// Re-export the unified Result type for convenience
pub use error::Result;
