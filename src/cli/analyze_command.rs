//! Analyze Command Implementation
//!
//! This module implements the `analyze` subcommand for the Uveddi CLI.
//! The analyze command performs comprehensive code analysis on a given path,
//! optionally integrating AI-powered insights and generating reports.
//!
//! # Features
//!
//! - Multi-format output (text, JSON, markdown, HTML)
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
use std::error::Error;
use std::path::PathBuf;
use sysinfo::System;
use tracing::{info, warn};

use crate::application::{AnalysisConfig, AnalysisOrchestrator};
use crate::error::UveddiError;
use crate::progress::{create_progress_reporter, ProgressTracker, AnalysisPhase};
use crate::report::DiagramMode;
use crate::security::{self, SecurityError, validate_cli_argument, CliArgumentType};

// Security functions are now available through the security module import above

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
    /// - `html`: Interactive HTML format with embedded diagrams and dark/light themes
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

    /// Disable memory optimization features
    ///
    /// By default, Uveddi uses memory optimization (object pooling, arena allocation,
    /// and zero-copy AST caching) for better performance. Use this flag to disable optimizations.
    #[arg(long)]
    pub disable_memory_optimization: bool,

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
    /// - `default`: Balanced settings for most projects (default)
    /// - `large`: Optimized for large codebases (> 10000 files)
    #[arg(long, value_name = "PROFILE", default_value = "default")]
    pub memory_profile: Option<String>,

    // === HYBRID RENDERING OPTIONS ===
    /// Enable image rendering (requires rendering service)
    ///
    /// When enabled, diagrams will be rendered as images using the rendering service.
    /// Falls back to Mermaid-only mode if service is unavailable (unless --no-fallback is used).
    #[arg(long)]
    pub enable_image_rendering: bool,

    /// Force Mermaid-only mode (no image rendering, zero hosting costs)
    ///
    /// Generate only Mermaid code with helpful rendering instructions.
    /// This is the default mode to eliminate hosting costs.
    #[arg(long)]
    pub mermaid_only: bool,

    /// Rendering service URL
    ///
    /// URL of the rendering service for image generation.
    /// Only used when --enable-image-rendering is specified.
    #[arg(long, default_value = "http://localhost:3001")]
    pub rendering_service_url: String,

    /// Disable fallback to Mermaid-only (fail if image rendering unavailable)
    ///
    /// When enabled, analysis will fail if image rendering is requested but unavailable.
    /// By default, the system gracefully falls back to Mermaid-only mode.
    #[arg(long)]
    pub no_fallback: bool,

    /// Check rendering service availability without running analysis
    ///
    /// Performs a health check on the rendering service and exits.
    /// Useful for verifying service configuration before running analysis.
    #[arg(long)]
    pub check_rendering_service: bool,

    /// Disable diagram generation completely
    ///
    /// Skip all diagram generation to speed up analysis when only textual output is needed.
    #[arg(long)]
    pub no_diagrams: bool,

    /// Maximum number of diagrams to generate per report
    ///
    /// Limits the total number of diagrams to prevent performance issues with large codebases.
    /// Set to 0 for no limit.
    #[arg(long, default_value = "20")]
    pub max_diagrams: usize,

    /// Output directory for diagram files
    ///
    /// When using image rendering modes, diagrams will be saved to this directory.
    /// If not specified, a subdirectory next to the report will be created.
    #[arg(long)]
    pub diagram_output_dir: Option<PathBuf>,

    /// Maximum analysis timeout in seconds
    ///
    /// Sets the maximum time to wait for analysis completion before aborting.
    /// Default is 300 seconds (5 minutes). Set to 0 to disable timeout.
    #[arg(long, default_value = "300")]
    pub timeout: u64,

    /// Show detailed error information and stack traces
    ///
    /// When enabled, errors will include additional debug information,
    /// stack traces, and context to help diagnose issues.
    #[arg(long)]
    pub verbose: bool,

    /// Progress reporting format
    ///
    /// Controls how progress is displayed during analysis:
    /// - `terminal`: Rich terminal output with progress bars and estimates (default)
    /// - `json`: JSON progress events for programmatic consumption
    /// - `silent`: No progress output (quiet mode)
    #[arg(long, default_value = "terminal")]
    pub progress_format: String,

    /// Show detailed progress information
    ///
    /// When enabled with terminal progress format, displays current file being processed,
    /// throughput metrics, and detailed timing information.
    #[arg(long)]
    pub progress_details: bool,

    /// Automatically open dashboard after analysis completes
    ///
    /// When enabled, the analysis dashboard will be launched in the default web browser
    /// after analysis is complete, showing the results in a visual interface.
    #[arg(long)]
    pub open_dashboard: bool,

    // === SECURITY ANALYSIS OPTIONS ===
    /// Enable security vulnerability analysis
    ///
    /// Performs comprehensive security analysis including OWASP Top 10 coverage,
    /// taint flow analysis, and vulnerability detection.
    #[arg(long)]
    pub security: bool,

    /// Run only security analysis (skip other detectors)
    ///
    /// When enabled, only security-related anti-pattern detection will be performed.
    /// This provides faster analysis when only security concerns are relevant.
    #[arg(long)]
    pub security_only: bool,

    /// Minimum confidence threshold for security findings (0.0 to 1.0)
    ///
    /// Only report security issues with confidence above this threshold.
    /// Higher values reduce false positives but may miss some vulnerabilities.
    #[arg(long, value_name = "THRESHOLD", default_value = "0.5")]
    pub min_security_confidence: Option<f64>,

    /// Export security findings in SARIF 2.1.0 format
    ///
    /// Generates SARIF-compliant output suitable for GitHub Security tab,
    /// CI/CD pipeline integration, and security toolchain interoperability.
    #[arg(long)]
    pub export_sarif: bool,

    /// SARIF output file path
    ///
    /// Specifies where to save the SARIF export file. If not provided,
    /// defaults to '<output-file-stem>.sarif' or 'security-findings.sarif'.
    #[arg(long)]
    pub sarif_output: Option<PathBuf>,

    /// Enable taint flow analysis for data flow vulnerabilities
    ///
    /// Performs source-to-sink taint analysis to identify potential
    /// injection vulnerabilities and unsafe data flows.
    #[arg(long)]
    pub enable_taint_analysis: bool,

    /// Maximum depth for taint flow analysis (1-20)
    ///
    /// Controls how deep the taint analysis traverses call chains.
    /// Higher values increase accuracy but also analysis time.
    #[arg(long, value_name = "DEPTH", default_value = "10")]
    pub taint_analysis_depth: Option<u8>,

    /// OWASP categories to focus on (comma-separated)
    ///
    /// Limit security analysis to specific OWASP Top 10 2021 categories.
    /// Examples: "A01,A03,A06" or "injection,broken_access_control"
    #[arg(long, value_delimiter = ',')]
    pub owasp_categories: Option<Vec<String>>,
}

impl AnalyzeCommand {
    /// Auto-detect appropriate memory limit based on system RAM
    /// Returns memory limit in GB, using conservative estimates for stability
    fn auto_detect_memory_limit() -> Option<f64> {
        let mut system = System::new_all();
        system.refresh_memory();

        let total_memory_bytes = system.total_memory();
        let total_memory_gb = total_memory_bytes as f64 / (1024.0 * 1024.0 * 1024.0);

        // Use conservative limits based on available RAM
        if total_memory_gb >= 16.0 {
            Some(8.0) // Use up to 8GB on systems with 16GB+ RAM
        } else if total_memory_gb >= 8.0 {
            Some(4.0) // Use up to 4GB on systems with 8GB+ RAM
        } else if total_memory_gb >= 4.0 {
            Some(2.0) // Use up to 2GB on systems with 4GB+ RAM
        } else {
            Some(1.0) // Use up to 1GB on systems with less RAM
        }
    }

    /// Auto-detect appropriate memory profile based on project size and system resources
    fn auto_detect_memory_profile(&self) -> String {
        // If user specified a profile, respect it
        if let Some(ref profile) = self.memory_profile {
            return profile.clone();
        }

        let mut system = System::new_all();
        system.refresh_memory();
        let total_memory_gb = system.total_memory() as f64 / (1024.0 * 1024.0 * 1024.0);

        // Auto-detect based on system capabilities
        if total_memory_gb >= 16.0 {
            "large".to_string()
        } else if total_memory_gb >= 8.0 {
            "default".to_string()
        } else {
            "small".to_string()
        }
    }

    /// Determine the appropriate diagram mode based on CLI flags
    pub fn get_diagram_mode(&self) -> DiagramMode {
        if self.mermaid_only {
            DiagramMode::MermaidOnly
        } else if self.enable_image_rendering {
            if self.no_fallback {
                DiagramMode::ImageOnly
            } else {
                DiagramMode::ImageWithFallback
            }
        } else {
            // Default: Mermaid-only for zero hosting costs
            DiagramMode::MermaidOnly
        }
    }

    /// Validate CLI flag combinations
    pub fn validate(&self) -> Result<(), String> {
        if self.mermaid_only && self.enable_image_rendering {
            return Err("Cannot use both --mermaid-only and --enable-image-rendering".to_string());
        }

        if self.no_fallback && !self.enable_image_rendering {
            return Err("--no-fallback can only be used with --enable-image-rendering".to_string());
        }

        if self.no_diagrams && (self.mermaid_only || self.enable_image_rendering) {
            return Err(
                "--no-diagrams cannot be used with --mermaid-only or --enable-image-rendering"
                    .to_string(),
            );
        }

        if self.diagram_output_dir.is_some() && !self.enable_image_rendering {
            return Err(
                "--diagram-output-dir can only be used with --enable-image-rendering".to_string(),
            );
        }

        Ok(())
    }

    /// Validate the analysis path with specific, helpful error messages
    fn validate_analysis_path(&self) -> Result<(), SecurityError> {
        // Check if path exists
        if !self.path.exists() {
            return Err(SecurityError::InvalidInput {
                field: "path".to_string(),
                reason: format!(
                    "Path '{}' does not exist.\n💡 Suggestion: Check the path spelling and ensure the directory/file exists",
                    self.path.display()
                ),
            });
        }

        // Check if path is readable
        if let Err(e) = std::fs::metadata(&self.path) {
            return Err(SecurityError::InvalidInput {
                field: "path".to_string(),
                reason: format!(
                    "Cannot access path '{}': {}\n💡 Suggestion: Check file permissions and run with appropriate privileges",
                    self.path.display(),
                    e
                ),
            });
        }

        // Check for supported file types if it's a single file
        if self.path.is_file() {
            if let Some(extension) = self.path.extension() {
                let ext = extension.to_string_lossy().to_lowercase();
                let supported_extensions = [
                    "rs", "py", "js", "ts", "jsx", "tsx", "java", "cpp", "c", "h", "hpp",
                ];

                if !supported_extensions.contains(&ext.as_str()) {
                    return Err(SecurityError::InvalidInput {
                        field: "path".to_string(),
                        reason: format!(
                            "Unsupported file type '.{}' for file '{}'.\n✅ Supported file types: {}\n💡 Suggestion: Specify a directory containing supported files or use a supported file extension",
                            ext,
                            self.path.display(),
                            supported_extensions.join(", ")
                        ),
                    });
                }
            }
        }

        // Check if directory is empty or contains no supported files (using recursive discovery)
        if self.path.is_dir() {
            let has_supported_files =
                Self::discover_files_recursive(&self.path)?
                    .into_iter()
                    .any(|path| {
                        if let Some(extension) = path.extension() {
                            let ext = extension.to_string_lossy().to_lowercase();
                            [
                                "rs", "py", "js", "ts", "jsx", "tsx", "java", "cpp", "c", "h",
                                "hpp",
                            ]
                            .contains(&ext.as_str())
                        } else {
                            false
                        }
                    });

            if !has_supported_files {
                return Err(SecurityError::InvalidInput {
                    field: "path".to_string(),
                    reason: format!(
                        "Directory '{}' contains no supported source files.\n✅ Supported file types: rs, py, js, ts, jsx, tsx, java, cpp, c, h, hpp\n💡 Suggestion: Ensure the directory contains source code files with supported extensions",
                        self.path.display()
                    ),
                });
            }
        }

        Ok(())
    }

    /// Validate all command inputs before processing
    pub fn validate_inputs(&self) -> Result<(), SecurityError> {
        // Enhanced CLI argument validation using new security framework
        let path_str = self.path.to_string_lossy();
        validate_cli_argument(&path_str, "path", CliArgumentType::FilePath)?;
        
        // Validate output format using enhanced CLI validation
        validate_cli_argument(&self.output_format, "output_format", CliArgumentType::Generic)?;
        
        // Comprehensive path validation with specific error messages
        self.validate_analysis_path()?;

        // Additional general input validation
        security::validate_input(&path_str, "path")?;
        security::validate_input(&self.output_format, "output_format")?;

        // Validate output file if specified
        if let Some(ref output) = self.output {
            let output_str = output.to_string_lossy();
            security::validate_input(&output_str, "output")?;
        }

        // Validate Ollama API URL if specified
        if let Some(ref url) = self.ollama_api_url {
            security::validate_url(url)?;
        }

        // Validate Ollama model name if specified
        if let Some(ref model) = self.ollama_model {
            security::validate_model_name(model)?;
        }

        // Validate numeric parameters
        if let Some(confidence) = self.dead_code_confidence {
            let confidence_int = (confidence * 100.0) as i32;
            security::validate_numeric_range(
                confidence_int.into(),
                0,
                100,
                "dead_code_confidence",
            )?;
        }

        if let Some(max_loc) = self.large_classes_max_loc {
            security::validate_numeric_range(
                (max_loc as i32).into(),
                1,
                100_000,
                "large_classes_max_loc",
            )?;
        }

        if let Some(max_methods) = self.large_classes_max_methods {
            security::validate_numeric_range(
                (max_methods as i32).into(),
                1,
                10_000,
                "large_classes_max_methods",
            )?;
        }

        if let Some(max_fields) = self.large_classes_max_fields {
            security::validate_numeric_range(
                (max_fields as i32).into(),
                1,
                10_000,
                "large_classes_max_fields",
            )?;
        }

        if let Some(max_complexity) = self.large_classes_max_complexity {
            security::validate_numeric_range(
                (max_complexity as i32).into(),
                1,
                10_000,
                "large_classes_max_complexity",
            )?;
        }

        if let Some(max_lcom) = self.large_classes_max_lcom {
            let lcom_int = (max_lcom * 100.0) as i32;
            security::validate_numeric_range(lcom_int.into(), 0, 100, "large_classes_max_lcom")?;
        }

        if let Some(min_severity) = self.large_classes_min_severity {
            security::validate_numeric_range(
                (min_severity as i32).into(),
                0,
                100,
                "large_classes_min_severity",
            )?;
        }

        if let Some(memory_limit) = self.memory_limit_gb {
            let memory_int = (memory_limit * 10.0) as i32; // Convert to decidigabytes for int validation
            security::validate_numeric_range(memory_int.into(), 1, 1000, "memory_limit_gb")?;
            // 0.1 GB to 100 GB
        }

        // Validate memory profile if specified
        if let Some(ref profile) = self.memory_profile {
            let allowed_profiles = ["small", "default", "large"];
            if !allowed_profiles.contains(&profile.as_str()) {
                return Err(SecurityError::InvalidInput {
                    field: "memory_profile".to_string(),
                    reason: format!("Must be one of: {}", allowed_profiles.join(", ")),
                });
            }
        }

        // Validate ignore patterns if specified
        if let Some(ref patterns) = self.dead_code_ignore_patterns {
            for pattern in patterns {
                security::validate_input(pattern, "dead_code_ignore_pattern")?;
            }
        }

        if let Some(ref patterns) = self.dead_code_keep_alive {
            for pattern in patterns {
                security::validate_input(pattern, "dead_code_keep_alive_pattern")?;
            }
        }

        if let Some(ref patterns) = self.large_classes_ignore_patterns {
            for pattern in patterns {
                security::validate_input(pattern, "large_classes_ignore_pattern")?;
            }
        }

        // Validate security-specific options
        if let Some(confidence) = self.min_security_confidence {
            let confidence_int = (confidence * 100.0) as i32;
            security::validate_numeric_range(
                confidence_int.into(),
                0,
                100,
                "min_security_confidence",
            )?;
        }

        if let Some(depth) = self.taint_analysis_depth {
            security::validate_numeric_range((depth as i32).into(), 1, 20, "taint_analysis_depth")?;
        }

        if let Some(ref output) = self.sarif_output {
            let output_str = output.to_string_lossy();
            security::validate_input(&output_str, "sarif_output")?;
        }

        if let Some(ref categories) = self.owasp_categories {
            for category in categories {
                security::validate_input(category, "owasp_category")?;
            }
        }

        // Validate security flag combinations
        if self.security_only && !self.security {
            return Err(SecurityError::InvalidInput {
                field: "security_only".to_string(),
                reason: "--security-only requires --security to be enabled".to_string(),
            });
        }

        if self.export_sarif && !self.security {
            return Err(SecurityError::InvalidInput {
                field: "export_sarif".to_string(),
                reason: "--export-sarif requires --security to be enabled".to_string(),
            });
        }

        if self.enable_taint_analysis && !self.security {
            return Err(SecurityError::InvalidInput {
                field: "enable_taint_analysis".to_string(),
                reason: "--enable-taint-analysis requires --security to be enabled".to_string(),
            });
        }

        Ok(())
    }

    /// Execute analysis with input validation
    pub async fn execute_validated(&self) -> Result<(), UveddiError> {
        // Validate inputs first
        self.validate_inputs()?;

        // Proceed with existing execution logic
        self.execute().await
    }

    /// Print diagram mode information
    pub fn print_diagram_mode_info(&self) {
        let mode = self.get_diagram_mode();
        match mode {
            DiagramMode::MermaidOnly => {
                info!("📊 Diagram Mode: Mermaid-only (zero hosting costs)");
                info!(
                    "   Diagrams will be generated as Mermaid code with rendering instructions"
                );
            }
            DiagramMode::ImageOnly => {
                info!("📊 Diagram Mode: Image-only (requires rendering service)");
                info!("   Analysis will fail if rendering service is unavailable");
            }
            DiagramMode::ImageWithFallback => {
                info!("📊 Diagram Mode: Image with fallback (hybrid approach)");
                info!(
                    "   Will attempt image rendering, fallback to Mermaid-only if unavailable"
                );
            }
        }
    }

    /// Check rendering service availability and provide user feedback
    #[cfg(feature = "image-rendering")]
    pub async fn check_rendering_service_availability(
        &self,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        use crate::report::ImageRenderer;

        info!(
            "🔍 Checking rendering service at {}...",
            self.rendering_service_url
        );

        let renderer = ImageRenderer::new()?; // Use default config for now

        match tokio::time::timeout(std::time::Duration::from_secs(5), renderer.health_check()).await
        {
            Ok(Ok(_)) => {
                info!("✅ Rendering service is available!");
                info!("   URL: {}", self.rendering_service_url);
                Ok(true)
            }
            Ok(Err(e)) => {
                warn!("❌ Rendering service is not available:");
                warn!("   Error: {}", e);
                warn!("   URL: {}", self.rendering_service_url);
                self.print_rendering_service_setup_help();
                Ok(false)
            }
            Err(_) => {
                warn!("⏰ Rendering service check timed out");
                warn!("   URL: {}", self.rendering_service_url);
                self.print_rendering_service_setup_help();
                Ok(false)
            }
        }
    }

    /// Check rendering service availability (no-op when feature disabled)
    #[cfg(not(feature = "image-rendering"))]
    pub async fn check_rendering_service_availability(
        &self,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        info!("❌ Image rendering feature not enabled");
        info!("   To enable image rendering, rebuild with:");
        info!("   cargo build --features image-rendering");
        Ok(false)
    }

    /// Print helpful setup instructions for the rendering service
    fn print_rendering_service_setup_help(&self) {
        info!("\n💡 To set up the rendering service:");
        info!("   1. Local Docker setup:");
        info!("      docker-compose up rendering-service");
        info!("   2. Custom URL:");
        info!("      uveddi analyze --rendering-service-url http://your-service:3001");
        info!("   3. Zero-cost alternative:");
        info!("      uveddi analyze --mermaid-only");
        info!("\n📖 For more help, visit: https://github.com/botzrDev/uveddi#image-rendering");
    }
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

        // Set up enhanced progress reporting
        let progress_reporter = create_progress_reporter(&self.progress_format, self.progress_details);
        let mut progress_tracker = ProgressTracker::new(progress_reporter);
        
        // Start discovery phase
        progress_tracker.start_phase(AnalysisPhase::Discovery, None);
        info!("Starting analysis with {} progress reporting", self.progress_format);

        // Create application layer orchestrator with persistent database
        // Use the same database path as the dashboard server for consistency
        let database_path = std::path::Path::new("./.uveddi/database.db");

        // Ensure the database directory exists
        if let Some(parent) = database_path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create database directory")?;
        }

        let mut orchestrator = AnalysisOrchestrator::with_db_path(database_path)
            .context("Failed to initialize analysis orchestrator with persistent database")?;

        // Memory optimization is enabled by default, unless explicitly disabled
        let enable_memory_optimization = !self.disable_memory_optimization;

        // Auto-detect memory settings if not specified
        let memory_limit_gb = self.memory_limit_gb.or_else(|| {
            if enable_memory_optimization {
                Self::auto_detect_memory_limit()
            } else {
                None
            }
        });

        let memory_profile = if enable_memory_optimization {
            Some(self.auto_detect_memory_profile())
        } else {
            self.memory_profile.clone()
        };

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
            enable_memory_optimization,
            memory_limit_gb,
            memory_profile,
            timeout_seconds: self.timeout,
        };

        // Start parsing phase
        progress_tracker.start_phase(AnalysisPhase::Parsing, None);
        
        // Execute analysis through application layer with timeout
        let analysis_future = orchestrator.execute_analysis(config);

        let report = if self.timeout > 0 {
            // Execute with timeout and graceful degradation
            info!(
                "Analysis timeout set to {} seconds with graceful degradation enabled",
                self.timeout
            );
            match tokio::time::timeout(
                std::time::Duration::from_secs(self.timeout),
                analysis_future,
            )
            .await
            {
                Ok(result) => result.map_err(|e| {
                    let specific_error = match e {
                        ref err if err.to_string().contains("database") => {
                            "Database storage failed - check schema compatibility and disk space"
                        },
                        ref err if err.to_string().contains("parse") || err.to_string().contains("AST") => {
                            "Code parsing failed - verify file syntax and language support"
                        },
                        ref err if err.to_string().contains("memory") => {
                            "Memory limit exceeded - reduce analysis scope or increase memory limits"
                        },
                        ref err if err.to_string().contains("permission") || err.to_string().contains("access") => {
                            "File access denied - check file permissions and access rights"
                        },
                        _ => "Analysis failed"
                    };

                    progress_tracker.error(specific_error);
                    if self.verbose {
                        tracing::error!("🔍 {}: {:#}", specific_error, e);
                        if let Some(backtrace) = e.source() {
                            tracing::error!("🔧 Stack trace: {:?}", backtrace);
                        }
                    } else {
                        tracing::error!("{}. Use --verbose for detailed error information.", specific_error);
                    }
                    e
                })?,
                Err(_) => {
                    // Implement graceful degradation on timeout
                    progress_tracker.error("Analysis timed out, attempting graceful degradation");
                    warn!("Analysis timed out after {} seconds. Attempting graceful degradation...", self.timeout);

                    // Try with reduced scope and timeouts
                    let degraded_config = AnalysisConfig {
                        target_path: self.path.clone(),
                        output_format: self.output_format.clone(),
                        output_file: self.output.clone(),
                        enable_ai: false,  // Disable AI for faster analysis
                        ollama_api_url: None,
                        ollama_model: None,

                        // Dead code detection configs (reduced scope)
                        dead_code_confidence: Some(0.9),  // Higher confidence for faster processing
                        dead_code_library_mode: false,
                        dead_code_ignore_patterns: None,
                        dead_code_keep_alive: None,

                        // Large classes configs (more restrictive)
                        large_classes_max_loc: Some(500),  // Reduced from default
                        large_classes_max_methods: Some(self.large_classes_max_loc.unwrap_or(20)),
                        large_classes_max_fields: Some(15),
                        large_classes_max_complexity: Some(10),
                        large_classes_max_lcom: Some(0.8),
                        large_classes_ignore_patterns: None,
                        large_classes_min_severity: self.large_classes_min_severity,

                        #[cfg(feature = "memory-optimization")]
                        memory_optimization: None,
                        enable_memory_optimization: false,  // Disable for faster analysis
                        memory_limit_gb: Some(1.0),  // Strict memory limit
                        memory_profile: Some("small".to_string()),
                        timeout_seconds: 60,  // Reduced timeout for degraded analysis
                    };

                    info!("🔄 Retrying analysis with degraded settings: max 100 files, 15s per detector");
                    let degraded_future = orchestrator.execute_analysis(degraded_config);

                    match tokio::time::timeout(
                        std::time::Duration::from_secs(60),
                        degraded_future,
                    )
                    .await
                    {
                        Ok(result) => {
                            warn!("⚠️ Analysis completed with reduced scope due to timeout");
                            result?
                        },
                        Err(_) => {
                            return Err(UveddiError::analysis_error(
                                &self.path.display().to_string(),
                                0,
                                &format!(
                                    "Analysis timed out after {} seconds even with graceful degradation. Codebase may be too large. Consider using --timeout with a much larger value (e.g., 1800 for 30 minutes) or analyze a smaller subset.",
                                    self.timeout
                                ),
                                "timeout with degradation failure",
                            ));
                        }
                    }
                }
            }
        } else {
            // Execute without timeout
            info!("Analysis running without timeout");
            analysis_future.await.map_err(|e| {
                let specific_error = match e {
                    ref err if err.to_string().contains("database") => {
                        "Database storage failed - check schema compatibility and disk space"
                    }
                    ref err
                        if err.to_string().contains("parse") || err.to_string().contains("AST") =>
                    {
                        "Code parsing failed - verify file syntax and language support"
                    }
                    ref err if err.to_string().contains("memory") => {
                        "Memory limit exceeded - reduce analysis scope or increase memory limits"
                    }
                    ref err
                        if err.to_string().contains("permission")
                            || err.to_string().contains("access") =>
                    {
                        "File access denied - check file permissions and access rights"
                    }
                    _ => "Analysis failed",
                };

                progress_tracker.error(specific_error);
                if self.verbose {
                    tracing::error!("🔍 {}: {:#}", specific_error, e);
                    if let Some(backtrace) = e.source() {
                        tracing::error!("🔧 Stack trace: {:?}", backtrace);
                    }
                } else {
                    tracing::error!(
                        "{}. Use --verbose for detailed error information.",
                        specific_error
                    );
                }
                e
            })?
        };

        // Complete progress tracking
        progress_tracker.complete();

        // Output results
        if self.output.is_none() {
            // Print to stdout for user - this is intentional user output, not logging
            println!("{}", report.content);
        }

        // Log summary
        info!(
            "Analysis completed: {} files analyzed, {} issues found, AI enhanced: {}",
            report.metadata.files_analyzed,
            report.metadata.issues_found,
            report.metadata.ai_enhanced
        );

        // TODO: Notify dashboard of new analysis results
        // self.notify_dashboard_of_new_results(&report).await;

        // Print colorful summary to stdout for user
        let output_info = self
            .output
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "stdout".to_string());

        if report.metadata.issues_found > 0 {
            info!("\n📊 Analysis Summary:");
            info!("  • Files analyzed: {}", report.metadata.files_analyzed);
            info!("  • Issues found: {}", report.metadata.issues_found);
            if report.metadata.ai_enhanced {
                info!("  • AI enhanced: ✅");
            }

            // Add security-specific summary if security analysis was enabled
            if self.security {
                self.print_security_summary(&report).await;
            }

            info!("\n💡 Report generated: {}", output_info);
        } else {
            info!("\n✅ Analysis complete: No issues found! 🎉");
            info!("📊 Files analyzed: {}", report.metadata.files_analyzed);
            if self.security {
                info!("🔒 Security analysis: No vulnerabilities detected");
            }
            info!("💡 Report generated: {}", output_info);
        }

        // Auto-launch dashboard if requested
        if self.open_dashboard {
            info!("\n🚀 Dashboard launch requested but temporarily disabled during development");
            // self.launch_dashboard();
        } else if report.metadata.issues_found > 0 {
            // Suggest dashboard for a better experience when issues are found
            info!(
                "\n💡 Tip: Run with --open-dashboard to visualize results in the web interface"
            );
        }

        Ok(())
    }

    /// Recursively discover files in a directory, respecting .gitignore patterns
    /// and handling symlinks safely.
    pub fn discover_files_recursive(
        dir: &std::path::Path,
    ) -> Result<Vec<std::path::PathBuf>, SecurityError> {
        use ignore::WalkBuilder;
        use std::collections::HashSet;
        use walkdir::WalkDir;

        let mut discovered_files = Vec::new();
        let mut visited_inodes = HashSet::new();

        // Use ignore crate for proper gitignore support
        let walker = WalkBuilder::new(dir)
            .standard_filters(true) // Enable .gitignore, .ignore, etc.
            .hidden(false) // Include hidden files/dirs (let gitignore decide)
            .follow_links(false) // Don't follow symlinks to prevent infinite loops
            .max_depth(Some(100)) // Reasonable depth limit to prevent runaway traversal
            .build();

        for result in walker {
            match result {
                Ok(entry) => {
                    let path = entry.path();

                    // Skip directories
                    if !path.is_file() {
                        continue;
                    }

                    // Prevent infinite loops with symlinks by tracking inodes
                    if let Ok(metadata) = entry.metadata() {
                        #[cfg(unix)]
                        {
                            use std::os::unix::fs::MetadataExt;
                            let inode = metadata.ino();
                            if !visited_inodes.insert(inode) {
                                continue; // Already visited this inode
                            }
                        }
                    }

                    // Check file size to prevent analyzing extremely large files
                    if let Ok(metadata) = entry.metadata() {
                        const MAX_FILE_SIZE: u64 = 100 * 1024 * 1024; // 100MB limit
                        if metadata.len() > MAX_FILE_SIZE {
                            continue;
                        }
                    }

                    // Add to discovered files
                    discovered_files.push(path.to_path_buf());
                }
                Err(err) => {
                    // Log but don't fail on individual file access errors
                    warn!("Failed to access path during discovery: {}", err);
                }
            }
        }

        // If no files found with ignore patterns, fall back to basic walkdir
        // This handles cases where .gitignore might be too restrictive
        if discovered_files.is_empty() {
            warn!("No files found with gitignore filtering, falling back to basic discovery");

            for entry in WalkDir::new(dir)
                .follow_links(false)
                .max_depth(50)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.file_type().is_file() {
                    // Basic filtering - skip common non-source directories
                    let path_str = entry.path().to_string_lossy();
                    if path_str.contains("/.git/")
                        || path_str.contains("/target/")
                        || path_str.contains("/node_modules/")
                        || path_str.contains("/__pycache__/")
                        || path_str.contains("/build/")
                        || path_str.contains("/dist/")
                    {
                        continue;
                    }

                    discovered_files.push(entry.path().to_path_buf());
                }
            }
        }

        Ok(discovered_files)
    }

    /// Print security analysis summary with color-coded output
    async fn print_security_summary(&self, _report: &crate::application::AnalysisReport) {
        // Note: This is a placeholder implementation until we have the security data
        // properly flowing through the AnalysisReport structure

        info!("  🔒 Security Analysis:");

        if self.export_sarif {
            let sarif_path = self
                .sarif_output
                .as_ref()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| {
                    if let Some(output) = &self.output {
                        let mut sarif_path = output.clone();
                        sarif_path.set_extension("sarif");
                        sarif_path.to_string_lossy().to_string()
                    } else {
                        "security-findings.sarif".to_string()
                    }
                });

            info!("     • SARIF export: {}", sarif_path);
        }

        if self.enable_taint_analysis {
            let depth = self.taint_analysis_depth.unwrap_or(10);
            info!("     • Taint analysis depth: {}", depth);
        }

        if let Some(ref categories) = self.owasp_categories {
            info!("     • OWASP categories: {}", categories.join(", "));
        } else {
            info!("     • OWASP coverage: All Top 10 2021 categories");
        }

        let confidence = self.min_security_confidence.unwrap_or(0.5);
        info!("     • Min confidence: {:.0}%", confidence * 100.0);
    }

    // TODO: Dashboard integration methods temporarily disabled
    /*
    /// Notify the dashboard of new analysis results
    async fn notify_dashboard_of_new_results(&self, report: &crate::application::AnalysisReport) {
        use std::path::Path;
        use tokio::fs;
        use tokio::io::AsyncWriteExt;

        // Create notification file that indicates new results are available
        let notification_dir = Path::new("./.uveddi/notifications");
        let _ = fs::create_dir_all(&notification_dir).await;

        // Generate a unique notification ID based on timestamp
        let notification_id = chrono::Utc::now().timestamp();
        let notification_path = notification_dir.join(format!("analysis_complete_{}.json", notification_id));

        // Create notification payload with basic metadata
        let notification_content = serde_json::json!({
            "type": "analysis_complete",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "metadata": {
                "files_analyzed": report.metadata.files_analyzed,
                "issues_found": report.metadata.issues_found,
                "analysis_duration_ms": report.metadata.analysis_duration.as_millis(),
                "ai_enhanced": report.metadata.ai_enhanced,
                "format": self.output_format
            }
        });

        // Write notification file asynchronously
        if let Ok(mut file) = fs::File::create(&notification_path).await {
            if let Ok(content) = serde_json::to_string(&notification_content) {
                let _ = file.write_all(content.as_bytes()).await;
                info!("Dashboard notification created: {}", notification_path.display());
            }
        }

        // Try to notify any running dashboard service via HTTP
        let client = reqwest::Client::new();
        let dashboard_url = "http://localhost:8080/api/notifications/new-analysis";

        let _ = client.post(dashboard_url)
            .json(&notification_content)
            .timeout(std::time::Duration::from_secs(1))
            .send()
            .await;

        // Note: We intentionally ignore errors here to prevent analysis failures
        // if the dashboard isn't running
    }

    /// Launch the dashboard in the default web browser
    fn launch_dashboard(&self) {
        use std::process::Command;
        use std::thread;

        info!("\n🚀 Launching dashboard in your browser...");

        // Start the dashboard server in the background if not already running
        thread::spawn(|| {
            // Use a separate tokio runtime for this background process
            let rt = match tokio::runtime::Runtime::new() {
                Ok(rt) => rt,
                Err(e) => {
                    tracing::error!("Failed to create tokio runtime for dashboard: {}", e);
                    return;
                }
            };
            rt.block_on(async {
                // Configure the dashboard server
                let config = crate::service_orchestration::OrchestratorConfig {
                    api_port: 8080,
                    rendering_port: 3001,
                    frontend_port: 3000,
                    auto_start_services: true,
                    database_path: std::path::PathBuf::from("./.uveddi/database.db"),
                    frontend_assets_path: None,
                    development_mode: false,
                };

                // Start the services
                let mut orchestrator = crate::service_orchestration::ServiceOrchestrator::new();
                if let Err(e) = orchestrator.start_services(config).await {
                    tracing::error!("Failed to start dashboard services: {}", e);
                    return;
                }

                // Wait for Ctrl+C or program termination
                let _ = tokio::signal::ctrl_c().await;
            });
        });

        // Give services a moment to start
        thread::sleep(std::time::Duration::from_secs(2));

        // Open the browser to the dashboard URL
        #[cfg(target_os = "windows")]
        let open_cmd = "start";
        #[cfg(target_os = "macos")]
        let open_cmd = "open";
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        let open_cmd = "xdg-open";

        let dashboard_url = "http://localhost:8080/dashboard";
        if let Err(e) = Command::new(open_cmd).arg(dashboard_url).spawn() {
            warn!("Could not open browser automatically: {}", e);
            info!("Please open the dashboard manually at: {}", dashboard_url);
            info!("Dashboard URL: {}", dashboard_url);
        }
    }
    */
}
