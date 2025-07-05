//! Configuration Management for Uveddi
//!
//! This module provides centralized configuration management for the Uveddi application.
//! Configuration can be loaded from environment variables, TOML files, or provided
//! programmatically.
//!
//! # Configuration Sources
//!
//! 1. **Environment Variables**: Use `Config::from_env()` to load from env vars
//! 2. **TOML Files**: Use `Config::from_file()` to load from a TOML configuration file
//! 3. **Programmatic**: Create `Config` instances directly in code
//!
//! # Example Configuration File (config.toml)
//!
//! ```toml
//! ollama_model = "deepseek-coder:6.7b-instruct-q4_0"
//! ```
//!
//! # Environment Variables
//!
//! - `OLLAMA_MODEL`: The Ollama model to use for AI analysis
//!
//! # Usage
//!
//! ```rust,no_run
//! use uveddi::config::Config;
//!
//! // Load from environment
//! let config = Config::from_env()?;
//!
//! // Load from file
//! let config = Config::from_file("config.toml")?;
//!
//! // Create programmatically
//! let config = Config {
//!     ollama_model: Some("codellama:7b-instruct".to_string()),
//! };
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use serde::{Deserialize, Serialize};
use std::{env, fs};

/// Main configuration structure for Uveddi
///
/// Contains all configuration options that can be customized by users.
/// All fields are optional to allow partial configuration and fallback
/// to sensible defaults.
///
/// # Fields
///
/// * `ollama_model` - Optional model name for Ollama AI provider
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    /// The Ollama model to use for AI-powered analysis
    ///
    /// Examples: "deepseek-coder:6.7b-instruct-q4_0", "codellama:7b-instruct"
    pub ollama_model: Option<String>,
}

impl Config {
    /// Creates a new Config instance by loading values from environment variables.
    ///
    /// # Errors
    ///
    /// Returns an error if any of the required environment variables are not set.
    pub fn from_env() -> Result<Self, env::VarError> {
        let ollama_model = env::var("OLLAMA_MODEL").ok();
        Ok(Config { ollama_model })
    }

    /// Creates a new Config instance by loading values from a file.
    ///
    /// # Arguments
    ///
    /// * `path` - A string slice that represents the path to the configuration file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or if the contents cannot be parsed.
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}

/// Provides an abstraction for database operations, allowing for easier testing and swapping of implementations.
///
/// # Example
/// ```rust
/// use uveddi::config::DatabaseService;
/// struct MyDb;
/// impl DatabaseService for MyDb {
///     fn get_or_create_project_id(&self, project_path: &std::path::Path) -> rusqlite::Result<i64> { unimplemented!() }
///     fn create_analysis_run(&self, project_path: &std::path::Path) -> rusqlite::Result<uveddi::database::models::AnalysisRun> { unimplemented!() }
///     fn update_analysis_run(&self, run: &uveddi::database::models::AnalysisRun) -> rusqlite::Result<()> { unimplemented!() }
///     fn store_anti_pattern_type(&self, anti_pattern_type: &mut uveddi::database::models::AntiPatternType) -> rusqlite::Result<()> { unimplemented!() }
///     fn store_issues(&mut self, issues: &[uveddi::database::models::ArchitecturalIssue]) -> rusqlite::Result<()> { unimplemented!() }
/// }
/// ```
pub trait DatabaseService {
    /// Gets or creates a project ID for the given path.
    fn get_or_create_project_id(&self, project_path: &std::path::Path) -> rusqlite::Result<i64>;
    /// Creates a new analysis run for the given project.
    fn create_analysis_run(
        &self,
        project_path: &std::path::Path,
    ) -> rusqlite::Result<crate::database::models::AnalysisRun>;
    /// Updates an existing analysis run.
    fn update_analysis_run(
        &self,
        run: &crate::database::models::AnalysisRun,
    ) -> rusqlite::Result<()>;
    /// Stores a new anti-pattern type.
    fn store_anti_pattern_type(
        &self,
        anti_pattern_type: &mut crate::database::models::AntiPatternType,
    ) -> rusqlite::Result<()>;
    /// Stores architectural issues in the database.
    fn store_issues(
        &mut self,
        issues: &[crate::database::models::ArchitecturalIssue],
    ) -> rusqlite::Result<()>;
}

/// Provides an abstraction for AI-powered analysis engines.
///
/// This trait allows for different AI providers or mock engines to be used interchangeably.
pub trait AiEngineService {
    /// Analyzes the given codebase path.
    fn analyze(&self, codebase_path: &str) -> Result<(), crate::error::UveddiError>;
    /// Analyzes a single architectural issue asynchronously.
    fn analyze_issue(
        &self,
        issue: &mut crate::database::models::ArchitecturalIssue,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<(), crate::error::UveddiError>> + Send>,
    >;
}

/// Provides an abstraction for AST parsing services.
///
/// Enables the use of different AST backends or mocks for testing.
pub trait AstService {
    /// Parses a file and returns a parsed AST representation.
    fn parse_file(
        &self,
        file_path: &std::path::Path,
    ) -> Result<crate::ast::tree_sitter::ParsedFile, crate::error::UveddiError>;
}

/// Provides an abstraction for security checks and permission validation.
///
/// This trait allows for custom or mock security policies in tests and production.
pub trait SecurityService {
    /// Validates an API key for access control.
    fn validate_api_key(&self, api_key: &str) -> bool;
    /// Checks if the given action is permitted on a resource.
    fn check_permissions(&self, resource: &str, action: &str) -> bool;
}
