//! Centralized configuration module for Uveddi
//! Provides Config struct and loading from env or file
//! Extend this struct as needed for new configuration options

use serde::{Deserialize, Serialize};
use std::{env, fs};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
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
