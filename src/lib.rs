//! # Uveddi - CLI Architectural Analysis Tool
//!
//! Uveddi is a lightweight command-line architectural analysis tool that combines static code analysis
//! with optional AI-powered insights to help developers understand and improve their codebases.
//!
//! ## Features
//!
//! - **Multi-language AST parsing**: Support for analyzing Rust, Python, JavaScript, and TypeScript
//! - **Dependency analysis**: Track and visualize code dependencies with graph-based analysis
//! - **Quality metrics**: Calculate maintainability and complexity scores with anti-pattern detection
//! - **Report generation**: Create detailed analysis reports in multiple formats (JSON, HTML, Mermaid)
//! - **Caching system**: Efficient AST caching for improved performance on large codebases
//! - **AI-powered analysis** (optional): Integration with Ollama for intelligent code insights
//! - **Plugin system** (optional): WebAssembly-based plugin architecture for extensibility
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
//! let mut engine = AnalysisEngine::new()?;
//!
//! // Analyze a project directory
//! let (issues, dependency_graph) = engine.analyze(Path::new("src/")).await?;
//!
//! println!("Found {} architectural issues", issues.len());
//! println!("Analyzed {} files", dependency_graph.node_count());
//! # Ok(())
//! # }
//! ```
//!
//! ### Using Configuration
//!
//! ```rust,no_run
//! use uveddi::analysis::AnalysisEngine;
//! use uveddi::analysis::detectors::anti_patterns::GodObjectDetector;
//! use uveddi::analysis::detector_factory::DetectorFactory;
//! use std::path::Path;
//!
//! # async fn example() -> uveddi::Result<()> {
//! // Build engine with custom detectors
//! let mut detectors = DetectorFactory::create_default_detectors();
//! detectors.push(Box::new(GodObjectDetector::new(15, 10))); // Custom thresholds
//!
//! let mut engine = AnalysisEngine::builder()
//!     .with_detectors(detectors)
//!     .build()?;
//!
//! let (issues, graph) = engine.analyze(Path::new("src/")).await?;
//! println!("Found {} issues", issues.len());
//! # Ok(())
//! # }
//! ```
//!
//! ### AI-Powered Analysis
//!
//! ```ignore
//! // Requires "ai" feature flag to be enabled
//! use uveddi::ai::engine::AiEngine;
//! use uveddi::analysis::AnalysisEngine;
//! use std::path::Path;
//!
//! # async fn example() -> uveddi::Result<()> {
//! // Analyze with AI explanations (requires "ai" feature)
//! let analysis_engine = AnalysisEngine::new()?;
//! let (issues, graph) = analysis_engine.analyze(Path::new("src/")).await?;
//!
//! // AI-powered explanations available when the "ai" feature is enabled
//! println!("Found {} issues to analyze", issues.len());
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
//! - `cli-standard`: (default) Core CLI with all language support - recommended
//! - `cli-core`: Minimal CLI functionality without language parsing
//! - `cli-ai`: Standard CLI with AI support (requires Ollama)
//! - `cli-plugins`: Standard CLI with WASM plugin support
//! - `cli-full`: Everything for CLI (AI + plugins)
//! - `tree-sitter`: AST parsing for Rust, Python, JavaScript, TypeScript
//! - `ai`: Base AI functionality for analysis explanations
//! - `local-ai`: Ollama integration (includes `ai`)
//! - `wasm-plugins`: WebAssembly plugin system for extensibility
//!
//! ## Performance Characteristics
//!
//! - **Time Complexity**: O(n) where n is the number of files
//! - **Memory Usage**: O(m) where m is the number of parsed ASTs (with caching)
//! - **Parallelization**: Enabled by default for multi-core analysis
//! - **Cache Efficiency**: AST parsing results cached for repeated analysis
//!
//! For large codebases (>1000 files), use the builder with caching enabled:
//!
//! ```rust,no_run
//! use uveddi::analysis::AnalysisEngine;
//! use std::path::PathBuf;
//!
//! # async fn example() -> uveddi::Result<()> {
//! // Build engine with caching support
//! let engine = AnalysisEngine::builder()
//!     .with_cache_path(&PathBuf::from("analysis_cache.db"))
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
//! use uveddi::error::{UveddiError, ErrorCategory, ErrorSeverity};
//!
//! fn handle_error(error: UveddiError) {
//!     match error {
//!         UveddiError::ConfigError { message, location, suggestion } => {
//!             eprintln!("Configuration error at {}: {}", location, message);
//!             eprintln!("Suggestion: {}", suggestion);
//!         }
//!         UveddiError::IoError { operation, path, message, .. } => {
//!             eprintln!("File system error during {} on {}: {}", operation, path, message);
//!         }
//!         ref e => {
//!             eprintln!("Error: {}", e);
//!             eprintln!("Category: {:?}", e.category());
//!             eprintln!("Severity: {:?}", e.severity());
//!         }
//!     }
//! }
//! ```
//!
//! ## Architecture Overview
//!
//! The library is organized into several key modules:
//!
//! - [`analysis`]: Core analysis engines, detectors, and algorithms
//! - [`ast`]: Abstract Syntax Tree parsing and manipulation using tree-sitter
//! - [`cli`]: Command-line interface implementation
//! - [`database`]: Data persistence and querying capabilities with SQLite
//! - [`report`]: Report generation and formatting utilities
//! - [`cache`]: Caching system for improved performance
//! - [`error`]: Comprehensive error handling and reporting
//! - [`ai`]: (optional) AI provider integrations for intelligent analysis
//! - [`plugins`]: (optional) WebAssembly-based plugin system for extensibility
//!
//! ## Common Use Cases
//!
//! ### 1. Code Quality Assessment
//! ```rust,no_run
//! use uveddi::analysis::AnalysisEngine;
//! use uveddi::analysis::detector_factory::DetectorFactory;
//! use std::path::Path;
//!
//! # async fn example() -> uveddi::Result<()> {
//! // Use default detectors for comprehensive analysis
//! let detectors = DetectorFactory::create_default_detectors();
//! let mut engine = AnalysisEngine::builder()
//!     .with_detectors(detectors)
//!     .build()?;
//!
//! let (issues, graph) = engine.analyze(Path::new("src/")).await?;
//! println!("Found {} quality issues across {} nodes", issues.len(), graph.node_count());
//! # Ok(())
//! # }
//! ```
//!
//! ### 2. Dependency Analysis
//! ```rust,no_run
//! use uveddi::analysis::AnalysisEngine;
//! use std::path::Path;
//!
//! # async fn example() -> uveddi::Result<()> {
//! let mut engine = AnalysisEngine::new()?;
//! let (issues, graph) = engine.analyze(Path::new("src/")).await?;
//!
//! // Examine dependency graph
//! println!("Dependency graph has {} nodes", graph.node_count());
//! for node in graph.get_all_nodes() {
//!     println!("Component: {:?}", node);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ### 3. Plugin Development
//! ```ignore
//! // Requires "wasm-plugins" feature flag to be enabled
//! use uveddi::plugins::engine::PluginEngine;
//! use uveddi::plugins::types::PluginConfig;
//!
//! # async fn example() -> uveddi::Result<()> {
//! // Plugin system available when the "wasm-plugins" feature is enabled
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

// Targeted warning suppressions - only suppress specific warnings, not all warnings
#![allow(unused_variables, unused_imports, dead_code, unused_mut)]
// TODO: Re-enable documentation warnings after v1.0 release
// #![warn(missing_docs)]
// #![warn(rustdoc::missing_crate_level_docs)]

// Core CLI modules
#[cfg(feature = "ai")]
pub mod ai;
pub mod analysis;
pub mod application;
pub mod ast;
pub mod cache;
pub mod cli;
pub mod config;
pub mod constants;
pub mod core;
pub mod database;
#[cfg(feature = "engine-integration")]
pub mod engine;
pub mod error;
pub mod health;
pub mod hooks;
pub mod models;
#[cfg(feature = "wasm-plugins")]
pub mod plugins;
pub mod progress;
pub mod report;

// REMOVED: Enterprise and service modules for CLI-only release
// pub mod api;
// pub mod community;
// pub mod deployment;
// pub mod infrastructure;
// pub mod ingestion;
// pub mod monitoring;
// pub mod observability;
// pub mod performance;
// pub mod resilience;
// pub mod resource_management;
// pub mod security;
// pub mod security_stub;
// pub mod semantic_search;
// pub mod service_orchestration;
// pub mod sla;
// pub mod tui;

// Re-export the unified Result type for convenience
pub use error::Result;

// Re-export tree-sitter language modules when tree-sitter is disabled
#[cfg(not(feature = "tree-sitter"))]
pub use ast::tree_sitter::{
    tree_sitter_javascript, tree_sitter_python, tree_sitter_rust, tree_sitter_typescript,
};
