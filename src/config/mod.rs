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

// TODO: Re-enable when monitoring dependencies are properly configured
// pub mod monitoring;

pub mod validation;

// use crate::analysis::components::ComponentConfig; // Unused import
use crate::error::UveddiError;
use crate::security::{self, SecurityError};
use serde::{Deserialize, Serialize};
use std::{env, fs};
use std::path::Path;

/// Placeholder documentation for public items
///
/// Configuration structure for Uveddi
///
/// Contains all configuration options that can be customized by users.
/// All fields are optional to allow partial configuration and fallback
/// to sensible defaults.
///
/// # Fields
///
/// * `ollama_model` - Optional model name for Ollama AI provider
/// * `dead_code` - Configuration for dead code detection
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    /// The Ollama model to use for AI-powered analysis
    ///
    /// Examples: "deepseek-coder:6.7b-instruct-q4_0", "codellama:7b-instruct"
    pub ollama_model: Option<String>,

    /// Dead code detection configuration
    pub dead_code: Option<DeadCodeConfig>,

    /// Large classes detection configuration
    pub large_classes: Option<LargeClassConfig>,
}

/// Configuration for dead code detection
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeadCodeConfig {
    /// Minimum confidence threshold for reporting (0.0 to 1.0)
    pub confidence_threshold: Option<f64>,
    /// Whether to analyze exported symbols in library mode
    pub library_mode: Option<bool>,
    /// Patterns to ignore (e.g., test files, generated code)
    pub ignore_patterns: Option<Vec<String>>,
    /// Symbols to always consider live
    pub keep_alive_patterns: Option<Vec<String>>,
}

/// Configuration for large classes detection
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LargeClassConfig {
    /// Maximum logical lines of code threshold
    pub max_logical_loc: Option<u32>,
    /// Maximum number of methods threshold
    pub max_methods: Option<u32>,
    /// Maximum number of fields threshold
    pub max_fields: Option<u32>,
    /// Maximum cyclomatic complexity threshold
    pub max_cyclomatic_complexity: Option<u32>,
    /// Maximum cognitive complexity threshold
    pub max_cognitive_complexity: Option<u32>,
    /// Maximum LCOM score threshold (0.0 to 1.0)
    pub max_lcom_score: Option<f64>,
    /// Maximum coupling count threshold
    pub max_coupling: Option<u32>,
    /// Patterns to ignore (e.g., test files, generated code)
    pub ignore_patterns: Option<Vec<String>>,
    /// Language-specific threshold overrides
    pub language_overrides: Option<std::collections::HashMap<String, LanguageThresholds>>,
}

/// Language-specific thresholds for large class detection
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LanguageThresholds {
    /// Maximum logical lines of code
    pub max_logical_loc: Option<u32>,
    /// Maximum number of methods
    pub max_methods: Option<u32>,
    /// Maximum number of fields
    pub max_fields: Option<u32>,
    /// Maximum cyclomatic complexity
    pub max_cyclomatic_complexity: Option<u32>,
    /// Maximum cognitive complexity
    pub max_cognitive_complexity: Option<u32>,
    /// Maximum LCOM score (higher = less cohesive)
    pub max_lcom_score: Option<f64>,
    /// Maximum coupling count
    pub max_coupling: Option<u32>,
}

impl Config {
    /// Creates a new Config instance by loading values from environment variables.
    ///
    /// # Errors
    ///
    /// Returns an error if any of the required environment variables are not set.
    pub fn from_env() -> crate::error::Result<Self> {
        let ollama_model = env::var("OLLAMA_MODEL").ok();

        // Validate model name if provided
        if let Some(ref model) = ollama_model {
            security::validate_model_name(&model).map_err(crate::error::UveddiError::from)?;
        }

        // Dead code configuration from environment
        let dead_code = if env::var("DEAD_CODE_CONFIDENCE_THRESHOLD").is_ok()
            || env::var("DEAD_CODE_LIBRARY_MODE").is_ok()
            || env::var("DEAD_CODE_IGNORE_PATTERNS").is_ok()
            || env::var("DEAD_CODE_KEEP_ALIVE_PATTERNS").is_ok()
        {
            Some(DeadCodeConfig {
                confidence_threshold: env::var("DEAD_CODE_CONFIDENCE_THRESHOLD")
                    .ok()
                    .and_then(|s| s.parse().ok()),
                library_mode: env::var("DEAD_CODE_LIBRARY_MODE")
                    .ok()
                    .and_then(|s| s.parse().ok()),
                ignore_patterns: env::var("DEAD_CODE_IGNORE_PATTERNS")
                    .ok()
                    .map(|s| s.split(',').map(|p| p.trim().to_string()).collect()),
                keep_alive_patterns: env::var("DEAD_CODE_KEEP_ALIVE_PATTERNS")
                    .ok()
                    .map(|s| s.split(',').map(|p| p.trim().to_string()).collect()),
            })
        } else {
            None
        };

        // Large classes configuration from environment
        let large_classes = if env::var("LARGE_CLASSES_MAX_LOC").is_ok()
            || env::var("LARGE_CLASSES_MAX_METHODS").is_ok()
            || env::var("LARGE_CLASSES_MAX_FIELDS").is_ok()
            || env::var("LARGE_CLASSES_IGNORE_PATTERNS").is_ok()
        {
            Some(LargeClassConfig {
                max_logical_loc: env::var("LARGE_CLASSES_MAX_LOC")
                    .ok()
                    .and_then(|s| s.parse().ok()),
                max_methods: env::var("LARGE_CLASSES_MAX_METHODS")
                    .ok()
                    .and_then(|s| s.parse().ok()),
                max_fields: env::var("LARGE_CLASSES_MAX_FIELDS")
                    .ok()
                    .and_then(|s| s.parse().ok()),
                max_cyclomatic_complexity: env::var("LARGE_CLASSES_MAX_COMPLEXITY")
                    .ok()
                    .and_then(|s| s.parse().ok()),
                max_cognitive_complexity: env::var("LARGE_CLASSES_MAX_COGNITIVE_COMPLEXITY")
                    .ok()
                    .and_then(|s| s.parse().ok()),
                max_lcom_score: env::var("LARGE_CLASSES_MAX_LCOM")
                    .ok()
                    .and_then(|s| s.parse().ok()),
                max_coupling: env::var("LARGE_CLASSES_MAX_COUPLING")
                    .ok()
                    .and_then(|s| s.parse().ok()),
                ignore_patterns: env::var("LARGE_CLASSES_IGNORE_PATTERNS")
                    .ok()
                    .map(|s| s.split(',').map(|p| p.trim().to_string()).collect()),
                language_overrides: None, // Complex structure, not supported via env vars
            })
        } else {
            None
        };

        Ok(Config {
            ollama_model,
            dead_code,
            large_classes,
        })
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
    /// 
    /// # Security
    /// 
    /// This method validates the configuration file path to prevent directory traversal attacks
    /// and ensures the file meets security requirements (size limits, allowed extensions, etc.).
    pub fn from_file(path: &str) -> crate::error::Result<Self> {
        // Enhanced path validation to prevent directory traversal attacks
        let config_dir = std::env::var("UVEDDI_CONFIG_DIR").unwrap_or_else(|_| ".".to_string());
        let config_dir_path = Path::new(&config_dir);
        let allowed_config_dirs: Vec<&Path> = vec![
            Path::new("."), 
            Path::new("./config"), 
            Path::new("/etc/uveddi"), 
            Path::new("~/.config/uveddi"),
            config_dir_path
        ];
        
        let _validation = security::validate_config_file_path(Path::new(path), Some(&allowed_config_dirs))
            .map_err(|e| UveddiError::config_error(&format!("Configuration file path validation failed: {}", e), path))?;

        // Read and validate file content
        let content = fs::read_to_string(path)
            .map_err(|e| UveddiError::config_error(&format!("Failed to read configuration file: {}", e), path))?;
        
        // Validate content for security (basic input validation)
        security::validate_input(&content, "config_content")
            .map_err(|e| UveddiError::config_error(&format!("Configuration content validation failed: {}", e), path))?;
            
        // Parse TOML configuration
        let config: Config = toml::from_str(&content)
            .map_err(|e| crate::error::UveddiError::config_error(&e.to_string(), path))?;

        // Validate configuration values
        config.validate().map_err(|e| UveddiError::config_error(&format!("Configuration validation failed: {}", e), path))?;
        
        Ok(config)
    }

    /// Validates configuration values for security and correctness
    /// 
    /// # Returns
    /// * `Ok(())` - Configuration is valid
    /// * `Err(SecurityError)` - Configuration contains invalid values
    fn validate(&self) -> std::result::Result<(), SecurityError> {
        // Validate Ollama model name if provided
        if let Some(ref model) = self.ollama_model {
            security::validate_model_name(model)?;
        }

        // Validate dead code configuration
        if let Some(ref dc_config) = self.dead_code {
            if let Some(confidence) = dc_config.confidence_threshold {
                if !(0.0..=1.0).contains(&confidence) {
                    return Err(SecurityError::InvalidInput {
                        field: "dead_code.confidence_threshold".to_string(),
                        reason: "Confidence threshold must be between 0.0 and 1.0".to_string(),
                    });
                }
            }

            // Validate ignore patterns don't contain dangerous patterns
            if let Some(ref patterns) = dc_config.ignore_patterns {
                for pattern in patterns {
                    security::validate_input(pattern, "dead_code.ignore_patterns")?;
                }
            }

            // Validate keep alive patterns
            if let Some(ref patterns) = dc_config.keep_alive_patterns {
                for pattern in patterns {
                    security::validate_input(pattern, "dead_code.keep_alive_patterns")?;
                }
            }
        }

        // Validate large classes configuration
        if let Some(ref lc_config) = self.large_classes {
            // Validate numeric thresholds
            if let Some(max_loc) = lc_config.max_logical_loc {
                if max_loc > 100_000 {
                    return Err(SecurityError::InvalidInput {
                        field: "large_classes.max_logical_loc".to_string(),
                        reason: "Maximum LOC threshold exceeds reasonable limit of 100,000".to_string(),
                    });
                }
            }

            if let Some(max_methods) = lc_config.max_methods {
                if max_methods > 10_000 {
                    return Err(SecurityError::InvalidInput {
                        field: "large_classes.max_methods".to_string(),
                        reason: "Maximum methods threshold exceeds reasonable limit of 10,000".to_string(),
                    });
                }
            }

            if let Some(max_fields) = lc_config.max_fields {
                if max_fields > 10_000 {
                    return Err(SecurityError::InvalidInput {
                        field: "large_classes.max_fields".to_string(),
                        reason: "Maximum fields threshold exceeds reasonable limit of 10,000".to_string(),
                    });
                }
            }

            if let Some(lcom_score) = lc_config.max_lcom_score {
                if !(0.0..=1.0).contains(&lcom_score) {
                    return Err(SecurityError::InvalidInput {
                        field: "large_classes.max_lcom_score".to_string(),
                        reason: "LCOM score must be between 0.0 and 1.0".to_string(),
                    });
                }
            }

            // Validate ignore patterns
            if let Some(ref patterns) = lc_config.ignore_patterns {
                for pattern in patterns {
                    security::validate_input(pattern, "large_classes.ignore_patterns")?;
                }
            }

            // Validate language overrides
            if let Some(ref overrides) = lc_config.language_overrides {
                for (language, thresholds) in overrides {
                    security::validate_input(language, "large_classes.language_overrides.language")?;
                    
                    // Validate threshold values similar to main config
                    if let Some(max_loc) = thresholds.max_logical_loc {
                        if max_loc > 100_000 {
                            return Err(SecurityError::InvalidInput {
                                field: format!("large_classes.language_overrides.{}.max_logical_loc", language),
                                reason: "Maximum LOC threshold exceeds reasonable limit of 100,000".to_string(),
                            });
                        }
                    }

                    if let Some(lcom_score) = thresholds.max_lcom_score {
                        if !(0.0..=1.0).contains(&lcom_score) {
                            return Err(SecurityError::InvalidInput {
                                field: format!("large_classes.language_overrides.{}.max_lcom_score", language),
                                reason: "LCOM score must be between 0.0 and 1.0".to_string(),
                            });
                        }
                    }
                }
            }
        }

        Ok(())
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
    fn analyze(&self, codebase_path: &str) -> crate::error::Result<()>;
    /// Analyzes a single architectural issue asynchronously.
    fn analyze_issue(
        &self,
        issue: &mut crate::database::models::ArchitecturalIssue,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = crate::error::Result<()>> + Send>>;
}

/// Provides an abstraction for AST parsing services.
///
/// Enables the use of different AST backends or mocks for testing.
pub trait AstService {
    /// Parses a file and returns a parsed AST representation.
    fn parse_file(
        &self,
        file_path: &std::path::Path,
    ) -> crate::error::Result<crate::ast::tree_sitter::ParsedFile>;
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
