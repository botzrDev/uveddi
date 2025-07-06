//! # Uveddi - Architectural Analysis Tool
//!
//! Uveddi is a comprehensive architectural analysis tool that combines static code analysis
//! with AI-powered insights to help developers understand and improve their codebases.
//!
//! ## Features
//!
//! - **Multi-language AST parsing**: Support for analyzing various programming languages
//! - **AI-powered analysis**: Integration with AI providers for intelligent code insights
//! - **Dependency analysis**: Track and visualize code dependencies
//! - **Quality metrics**: Calculate maintainability and complexity scores
//! - **Report generation**: Create detailed analysis reports in multiple formats
//! - **Caching system**: Efficient caching for improved performance
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use uveddi::config::Config;
//! use uveddi::application::AnalysisOrchestrator;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Initialize the application with configuration
//! let config = Config::default();
//! let mut orchestrator = AnalysisOrchestrator::new()?;
//!
//! // Run analysis on a project
//! orchestrator.analyze_project("/path/to/project")?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Architecture
//!
//! The library is organized into several key modules:
//!
//! - [`ai`]: AI provider integrations for intelligent analysis
//! - [`analysis`]: Core analysis engines and algorithms
//! - [`ast`]: Abstract Syntax Tree parsing and manipulation
//! - [`database`]: Data persistence and querying capabilities
//! - [`report`]: Report generation and formatting utilities
//!
//! See individual module documentation for detailed usage information.

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

pub mod ai;
pub mod analysis;
pub mod application;
pub mod ast;
pub mod cache;
pub mod cli;
pub mod config;
pub mod database;
pub mod error;
pub mod ingestion;
pub mod models;
pub mod report;
pub mod semantic_search;
