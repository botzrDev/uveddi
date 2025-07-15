//! Analyze Command Implementation
//!
//! This module implements the `analyze` subcommand for the Uveddi CLI.
//! The analyze command performs comprehensive code analysis on a given path,
//! optionally integrating AI-powered insights and generating reports.
//!
//! # Features
//!
//! - Multi-format output (text, JSON, markdown)
//! - AI-powered analysis with Ollama integration
//! - Configurable output destinations
//! - Environment variable support for configuration
//!
//! # Usage
//!
//! ```bash
//! # Basic analysis
//! uveddi analyze ./src
//!
//! # With AI analysis
//! uveddi analyze ./src --enable-ai
//!
//! # Custom output format and file
//! uveddi analyze ./src --output-format json --output analysis.json
//!
//! # Using environment variables for Ollama
//! OLLAMA_API_URL=http://localhost:11434 OLLAMA_MODEL=deepseek-coder uveddi analyze ./src --enable-ai
//! ```

// NOTE: UV-112, UV-115 - Layer boundary compliance confirmed July 2025. This module only interacts with the Application layer per architecture.

use anyhow::Context;
use clap::Args;
use log::info;
use std::path::PathBuf;

use crate::application::{AnalysisConfig, AnalysisOrchestrator};
use crate::error::UveddiError;

/// Command-line arguments for the analyze subcommand
///
/// This struct defines all the command-line options available for the
/// `analyze` command, including input paths, output configuration,
/// and AI integration settings.
#[derive(Args, Debug, Clone, PartialEq)]
pub struct AnalyzeCommand {
    /// Path to the project or directory to analyze
    ///
    /// Can be a file or directory. When analyzing a directory,
    /// all supported source files will be recursively processed.
    pub path: PathBuf,

    /// Output format for the analysis report
    ///
    /// Supported formats:
    /// - `text`: Plain text format for terminal output
    /// - `json`: Structured JSON format for programmatic consumption
    /// - `markdown`: Markdown format for documentation
    #[arg(long, default_value = "markdown")]
    pub output_format: String,

    /// Optional output file path
    ///
    /// If not specified, the report will be written to stdout.
    /// The file extension should match the chosen output format.
    #[arg(long)]
    pub output: Option<PathBuf>,

    /// Enable AI-powered analysis and explanations
    ///
    /// When enabled, the analysis will include AI-generated explanations
    /// for detected issues and architectural recommendations.
    /// Requires either an API key or a local Ollama instance.
    #[arg(long)]
    pub enable_ai: bool,

    /// Ollama API URL for local AI analysis
    ///
    /// Used when `--enable-ai` is specified and you want to use
    /// a local Ollama instance instead of external AI services.
    ///
    /// Can also be set via the `OLLAMA_API_URL` environment variable.
    #[arg(long, env = "OLLAMA_API_URL")]
    pub ollama_api_url: Option<String>,

    /// Ollama model name for local AI analysis
    ///
    /// Specifies which Ollama model to use for analysis.
    /// Common options include "deepseek-coder:6.7b-instruct-q4_0",
    /// "codellama:7b-instruct", etc.
    ///
    /// Can also be set via the `OLLAMA_MODEL` environment variable.
    #[arg(long, env = "OLLAMA_MODEL")]
    pub ollama_model: Option<String>,

    /// Confidence threshold for dead code detection (0.0 to 1.0)
    ///
    /// Only report dead code issues with confidence above this threshold.
    /// Higher values reduce false positives but may miss some issues.
    #[arg(long, value_name = "THRESHOLD")]
    pub dead_code_confidence: Option<f64>,

    /// Enable library mode for dead code detection
    ///
    /// In library mode, exported symbols are treated more conservatively
    /// to avoid false positives for public APIs.
    #[arg(long)]
    pub dead_code_library_mode: bool,

    /// Patterns to ignore during dead code detection
    ///
    /// Comma-separated list of patterns to exclude from analysis.
    /// Example: "test,spec,mock,generated"
    #[arg(long, value_delimiter = ',')]
    pub dead_code_ignore_patterns: Option<Vec<String>>,

    /// Symbols to always keep alive during dead code detection
    ///
    /// Comma-separated list of symbol patterns that should never be
    /// reported as dead code. Example: "main,init,setup,teardown"
    #[arg(long, value_delimiter = ',')]
    pub dead_code_keep_alive: Option<Vec<String>>,

    /// Maximum logical lines of code threshold for large classes
    ///
    /// Classes exceeding this threshold will be flagged as potentially too large.
    /// Default varies by language (Rust: 400, Python: 1000, JavaScript: 800)
    #[arg(long, value_name = "LINES")]
    pub large_classes_max_loc: Option<u32>,

    /// Maximum number of methods threshold for large classes
    ///
    /// Classes with more methods than this threshold will be flagged.
    /// Default varies by language (Rust: 20, Python: 20, JavaScript: 25)
    #[arg(long, value_name = "COUNT")]
    pub large_classes_max_methods: Option<u32>,

    /// Maximum number of fields threshold for large classes
    ///
    /// Classes with more fields than this threshold will be flagged.
    /// Default varies by language (Rust: 15, Python: 7, JavaScript: 12)
    #[arg(long, value_name = "COUNT")]
    pub large_classes_max_fields: Option<u32>,

    /// Maximum cyclomatic complexity threshold for large classes
    ///
    /// Classes with higher complexity will be flagged as potentially too complex.
    /// Default varies by language (Rust: 50, Python: 60, JavaScript: 55)
    #[arg(long, value_name = "COMPLEXITY")]
    pub large_classes_max_complexity: Option<u32>,

    /// Maximum LCOM (Lack of Cohesion in Methods) score threshold
    ///
    /// Higher values indicate lower cohesion. Range: 0.0 to 1.0
    /// Default: 0.8 for all languages
    #[arg(long, value_name = "SCORE")]
    pub large_classes_max_lcom: Option<f64>,

    /// Patterns to ignore during large classes detection
    ///
    /// Comma-separated list of patterns to exclude from analysis.
    /// Example: "test,spec,mock,generated,fixture"
    #[arg(long, value_delimiter = ',')]
    pub large_classes_ignore_patterns: Option<Vec<String>>,

    /// Minimum severity score for large classes reporting (0-100)
    ///
    /// Only report issues with severity above this threshold.
    /// 0-25: Info, 26-50: Low, 51-75: Medium, 76-90: High, 91-100: Critical
    #[arg(long, value_name = "SCORE", default_value = "25")]
    pub large_classes_min_severity: Option<u32>,

    /// Enable memory optimization features
    ///
    /// Enables object pooling, arena allocation, and zero-copy AST caching
    /// for improved performance on large codebases.
    #[arg(long)]
    pub enable_memory_optimization: bool,

    /// Memory limit in gigabytes for analysis
    ///
    /// Sets a soft limit on memory usage. The analysis will attempt to
    /// stay within this limit by using more aggressive memory management.
    #[arg(long, value_name = "GB")]
    pub memory_limit_gb: Option<f64>,

    /// Memory profile for optimization settings
    ///
    /// Selects pre-configured memory optimization settings:
    /// - `small`: Optimized for small projects (< 1000 files)
    /// - `default`: Balanced settings for most projects
    /// - `large`: Optimized for large codebases (> 10000 files)
    #[arg(long, value_name = "PROFILE")]
    pub memory_profile: Option<String>,
}

impl AnalyzeCommand {
    /// Execute the analyze command with the provided arguments
    ///
    /// This method orchestrates the complete analysis workflow:
    /// 1. Initialize the analysis orchestrator
    /// 2. Configure analysis parameters
    /// 3. Run the analysis
    /// 4. Generate and output the report
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Analysis completed successfully
    /// * `Err(UveddiError)` - Analysis failed with specific error details
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The target path doesn't exist or isn't readable
    /// - The analysis orchestrator cannot be initialized
    /// - AI services are enabled but unavailable
    /// - The output file cannot be written
    /// - Internal analysis errors occur
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use uveddi::cli::analyze_command::AnalyzeCommand;
    /// use std::path::PathBuf;
    ///
    /// let command = AnalyzeCommand {
    ///     path: PathBuf::from("./src"),
    ///     output_format: "markdown".to_string(),
    ///     output: None,
    ///     enable_ai: false,
    ///     ollama_api_url: None,
    ///     ollama_model: None,
    /// };
    ///
    /// # tokio_test::block_on(async {
    /// command.execute().await?;
    /// # Ok::<(), uveddi::error::UveddiError>(())
    /// # });
    /// ```
    pub async fn execute(&self) -> Result<(), UveddiError> {
        info!("Starting analysis of: {}", self.path.display());

        // Create application layer orchestrator
        let mut orchestrator =
            AnalysisOrchestrator::new().context("Failed to initialize analysis orchestrator")?;

        // Configure analysis parameters
        let config = AnalysisConfig {
            target_path: self.path.clone(),
            output_format: self.output_format.clone(),
            output_file: self.output.clone(),
            enable_ai: self.enable_ai,
            ollama_api_url: self.ollama_api_url.clone(),
            ollama_model: self.ollama_model.clone(),
            dead_code_confidence: self.dead_code_confidence,
            dead_code_library_mode: self.dead_code_library_mode,
            dead_code_ignore_patterns: self.dead_code_ignore_patterns.clone(),
            dead_code_keep_alive: self.dead_code_keep_alive.clone(),
            large_classes_max_loc: self.large_classes_max_loc,
            large_classes_max_methods: self.large_classes_max_methods,
            large_classes_max_fields: self.large_classes_max_fields,
            large_classes_max_complexity: self.large_classes_max_complexity,
            large_classes_max_lcom: self.large_classes_max_lcom,
            large_classes_ignore_patterns: self.large_classes_ignore_patterns.clone(),
            large_classes_min_severity: self.large_classes_min_severity,

            // Memory optimization fields
            #[cfg(feature = "memory-optimization")]
            memory_optimization: None, // Will be created based on profile/limits
            enable_memory_optimization: self.enable_memory_optimization,
            memory_limit_gb: self.memory_limit_gb,
            memory_profile: self.memory_profile.clone(),
        };

        // Execute analysis through application layer
        let report = orchestrator
            .execute_analysis(config)
            .await
            .context("Analysis execution failed")?;

        // Output results
        if self.output.is_none() {
            println!("{}", report.content);
        }

        // Log summary
        info!(
            "Analysis completed: {} files analyzed, {} issues found, AI enhanced: {}",
            report.metadata.files_analyzed,
            report.metadata.issues_found,
            report.metadata.ai_enhanced
        );

        Ok(())
    }
}
