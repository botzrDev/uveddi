//! Report Generation Module for Uveddi
//!
//! This module provides comprehensive report generation capabilities for analysis results.
//! It supports multiple output formats and includes advanced features like AI explanations,
//! code snippets, and visual diagrams.
//!
//! # Supported Formats
//!
//! ## Markdown Reports
//! Rich, human-readable reports with:
//! - Executive summary with key metrics
//! - Issues organized by severity level
//! - Detailed issue descriptions with context
//! - Optional code snippets and AI explanations
//! - Mermaid.js diagrams for architectural visualization
//!
//! ## JSON Reports  
//! Structured data format for:
//! - Integration with external tools
//! - Automated processing and analysis
//! - API consumption
//! - Custom toolchain integration
//!
//! # Key Features
//!
//! - **Multi-format Output**: Markdown for humans, JSON for machines
//! - **AI Integration**: Optional AI-generated explanations and recommendations
//! - **Visual Diagrams**: Mermaid.js integration for architectural visualization
//! - **Customizable Content**: Configure inclusion of code snippets, AI explanations, diagrams
//! - **Performance Optimized**: Efficient processing for large analysis results
//! - **ERD Compliant**: Follows Entity Relationship Diagram specifications (ER-F-011 to ER-F-014)
//!
//! # Usage Examples
//!
//! ## Basic Report Generation
//!
//! ```rust,no_run
//! use uveddi::report::ReportGenerator;
//! use uveddi::database::models::{AnalysisRun, ArchitecturalIssue, AntiPatternType};
//!
//! let generator = ReportGenerator::new()
//!     .with_ai_explanations(true)
//!     .with_code_snippets(true)
//!     .with_diagrams(true);
//!
//! let run = AnalysisRun { /* ... */ };
//! let issues = vec![/* ArchitecturalIssue instances */];
//! let anti_patterns = vec![/* AntiPatternType instances */];
//!
//! // Generate markdown report
//! let markdown = generator.generate_markdown_report(&run, &issues, &anti_patterns)?;
//!
//! // Generate JSON report
//! let json = generator.generate_json_report(&run, &issues, &anti_patterns)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Writing Reports to Files
//!
//! ```rust,no_run
//! # use uveddi::report::ReportGenerator;
//! # let generator = ReportGenerator::new();
//! # let markdown_content = String::new();
//! use std::path::Path;
//!
//! // Write markdown report
//! generator.write_report_to_file(&markdown_content, Path::new("analysis_report.md"))?;
//!
//! // Write JSON report  
//! generator.write_report_to_file(&json_content, Path::new("analysis_report.json"))?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Report Structure
//!
//! ## Markdown Report Sections
//!
//! 1. **Executive Summary**: High-level metrics and overview
//! 2. **Analysis Overview**: Configuration and execution details
//! 3. **Issues by Severity**: Grouped and prioritized issue lists
//! 4. **Detailed Analysis**: In-depth issue descriptions
//! 5. **Architectural Diagrams**: Visual representations (if enabled)
//! 6. **Recommendations**: AI-generated suggestions (if enabled)
//!
//! ## JSON Report Schema
//!
//! ```json
//! {
//!   "analysis_run": { /* AnalysisRun metadata */ },
//!   "summary": {
//!     "total_issues": 42,
//!     "by_severity": { "critical": 2, "high": 8, "medium": 15, "low": 17 },
//!     "files_analyzed": 156
//!   },
//!   "issues": [ /* Array of ArchitecturalIssue objects */ ],
//!   "anti_patterns": [ /* Array of AntiPatternType definitions */ ]
//! }
//! ```

// use crate::analysis::graph::ComponentNode;
use crate::analysis::mermaid_generator::{MermaidGenerationError, MermaidGenerator};
use crate::database::models::{AnalysisRun, AntiPatternType, ArchitecturalIssue};
use crate::models::visualization::{ArchitecturalComponent, DiagramMetadata, DiagramType};
use crate::report::svg_generator::{SvgConfig, SvgGenerator};
use serde::{Deserialize, Serialize};
use std::time::Duration;
pub mod errors;

#[cfg(feature = "image-rendering")]
pub mod image_renderer;

/// Diagram generation mode for hybrid rendering approach
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DiagramMode {
    /// Try image rendering, fallback to Mermaid-only if service unavailable (recommended)
    ImageWithFallback,
    /// Only generate Mermaid code (no image rendering attempted) - zero hosting costs
    MermaidOnly,
    /// Only generate images (fail if service unavailable) - requires hosting
    ImageOnly,
}

impl Default for DiagramMode {
    fn default() -> Self {
        DiagramMode::MermaidOnly // Default to zero-cost option
    }
}

impl DiagramMode {
    /// Returns true if this mode should attempt image rendering
    pub fn should_attempt_image_rendering(&self) -> bool {
        matches!(
            self,
            DiagramMode::ImageOnly | DiagramMode::ImageWithFallback
        )
    }

    /// Returns true if this mode allows fallback to Mermaid-only
    pub fn allows_fallback(&self) -> bool {
        matches!(
            self,
            DiagramMode::ImageWithFallback | DiagramMode::MermaidOnly
        )
    }
}
pub use crate::error::rendering::RenderingServiceError;
#[cfg(feature = "image-rendering")]
pub use image_renderer::{ImageFormat, ImageRenderer, RenderedImage};

// Export interactive report models
pub use interactive_models::{
    AiInsights, AnalysisSummary, DependencyGraph, DiagramDefinition, Finding, GraphEdge, GraphNode,
    InteractiveReport, ProjectMetadata, ReportMetadata, REPORT_SCHEMA_VERSION,
};

// Export interactive report generator
pub use interactive_generator::{InteractiveReportConfig, InteractiveReportGenerator};

// Export security utilities
pub use security::{ReportSecurityConfig, ReportSecurityError, ReportSecurityValidator};

pub mod data_transformer;
pub mod diagrams;
pub mod interactive_generator;
pub mod interactive_models;
pub mod markdown_generator;
pub mod metrics;
pub mod modern_generator;
pub mod security;
pub mod svg_generator;
use crate::core::logging::{debug, error, info, warn};
use crate::report::metrics::{compute_debt_score, compute_issues_by_severity, count_unique_files};
use chrono::{DateTime, Local};
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io::Write;

/// Logs the full causal chain of a tera::Error, providing maximum diagnostic visibility.
///
/// This function starts with the top-level error and iterates through its `source()`
/// method, printing each underlying cause. This transforms a generic error message
/// into a detailed, step-by-step report of the failure.
fn log_tera_error_chain(e: &crate::report::modern_generator::ModernReportError) {
    // Log the primary error message, which is the user-friendly summary.
    error!("Root Error: {}", e);

    // Start traversing the causal chain.
    let mut source = e.source();
    let mut level = 1;

    while let Some(cause) = source {
        // Log each subsequent cause, indented for clarity.
        error!("  -> Caused by [Layer {}]: {}", level, cause);
        source = cause.source();
        level += 1;
    }
}
use std::path::Path;
use uuid::Uuid;

/// Configurable report generator with multi-format support
///
/// The `ReportGenerator` provides a flexible interface for creating analysis reports
/// in multiple formats. It follows the builder pattern for configuration and supports
/// various output customization options.
///
/// # Configuration Options
///
/// - **AI Explanations**: Include AI-generated issue explanations and recommendations
/// - **Code Snippets**: Embed relevant code snippets in issue descriptions  
/// - **Diagrams**: Generate Mermaid.js diagrams for architectural visualization
///
/// # Thread Safety
///
/// This struct is designed to be thread-safe and can be used across multiple
/// async tasks for concurrent report generation.
pub struct ReportGenerator {
    /// Whether to include AI-generated explanations in reports
    include_ai_explanations: bool,
    /// Whether to include code snippets in issue descriptions
    include_code_snippets: bool,
    /// Whether to generate and include architectural diagrams
    include_diagrams: bool,
    include_severity_summary: bool,
    include_remediation_steps: bool,
    /// Mermaid generator for creating diagrams
    mermaid_generator: Option<MermaidGenerator>,
    /// SVG generator for static diagram rendering
    svg_generator: Option<SvgGenerator>,
    /// Diagram generation mode (hybrid rendering approach)
    diagram_mode: DiagramMode,
    /// Image renderer for generating images from Mermaid diagrams
    #[cfg(feature = "image-rendering")]
    image_renderer: Option<crate::report::ImageRenderer>,
    /// Modern template-based generator (feature flagged)
    modern_generator: Option<modern_generator::ModernReportGenerator>,
}

impl Default for ReportGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl ReportGenerator {
    /// Creates a new `ReportGenerator` with default configuration.
    ///
    /// # Returns
    ///
    /// * `ReportGenerator` - A new instance with all features enabled by default.
    pub fn new() -> Self {
        Self {
            include_ai_explanations: true,
            include_code_snippets: true,
            include_diagrams: true,
            include_severity_summary: true,
            include_remediation_steps: true,
            mermaid_generator: MermaidGenerator::new().ok(),
            svg_generator: SvgGenerator::new().ok(),
            diagram_mode: DiagramMode::default(), // MermaidOnly by default for zero hosting costs
            #[cfg(feature = "image-rendering")]
            image_renderer: None,
            modern_generator: modern_generator::ModernReportGenerator::new().ok(),
        }
    }

    /// Configures whether AI explanations should be included in the report.
    ///
    /// # Arguments
    ///
    /// * `include` - If true, include AI explanations in the report.
    ///
    /// # Returns
    ///
    /// * `Self` - The updated report generator.
    pub fn with_ai_explanations(mut self, include: bool) -> Self {
        self.include_ai_explanations = include;
        self
    }

    /// Configures whether code snippets should be included in the report.
    ///
    /// # Arguments
    ///
    /// * `include` - If true, include code snippets in the report.
    ///
    /// # Returns
    ///
    /// * `Self` - The updated report generator.
    pub fn with_code_snippets(mut self, include: bool) -> Self {
        self.include_code_snippets = include;
        self
    }

    /// Configures whether diagrams should be included in the report.
    ///
    /// # Arguments
    ///
    /// * `include` - If true, include diagrams in the report.
    ///
    /// # Returns
    ///
    /// * `Self` - The updated report generator.
    pub fn with_diagrams(mut self, include: bool) -> Self {
        self.include_diagrams = include;
        self
    }

    /// Configures whether a severity summary should be included in the report.
    ///
    /// # Arguments
    ///
    /// * `include` - If true, include a severity summary in the report.
    ///
    /// # Returns
    ///
    /// * `Self` - The updated report generator.
    pub fn with_severity_summary(mut self, include: bool) -> Self {
        self.include_severity_summary = include;
        self
    }

    /// Configures whether remediation steps should be included in the report.
    ///
    /// # Arguments
    ///
    /// * `include` - If true, include remediation steps in the report.
    ///
    /// # Returns
    ///
    /// * `Self` - The updated report generator.
    pub fn with_remediation_steps(mut self, include: bool) -> Self {
        self.include_remediation_steps = include;
        self
    }

    /// Set the diagram generation mode
    pub fn with_diagram_mode(mut self, mode: DiagramMode) -> Self {
        self.diagram_mode = mode;
        self
    }

    /// Enable image rendering with fallback (hybrid approach)
    pub fn with_image_rendering_fallback(mut self) -> Self {
        self.diagram_mode = DiagramMode::ImageWithFallback;
        #[cfg(feature = "image-rendering")]
        {
            match crate::report::ImageRenderer::new() {
                Ok(renderer) => self.image_renderer = Some(renderer),
                Err(e) => {
                    tracing::warn!("Failed to create image renderer: {}", e);
                    // Fall back to text-only mode
                    self.diagram_mode = DiagramMode::MermaidOnly;
                }
            }
        }
        self
    }

    /// Force Mermaid-only mode (zero hosting costs)
    pub fn with_mermaid_only(mut self) -> Self {
        self.diagram_mode = DiagramMode::MermaidOnly;
        self
    }

    /// Get current diagram mode
    pub fn diagram_mode(&self) -> &DiagramMode {
        &self.diagram_mode
    }

    /// Check if rendering service is available (quick check)
    pub async fn is_rendering_service_available(&self) -> bool {
        #[cfg(feature = "image-rendering")]
        {
            if let Some(ref renderer) = self.image_renderer {
                match tokio::time::timeout(
                    Duration::from_secs(2), // Quick 2-second timeout
                    renderer.health_check(),
                )
                .await
                {
                    Ok(Ok(_)) => {
                        tracing::debug!("Rendering service is available");
                        true
                    }
                    Ok(Err(e)) => {
                        tracing::debug!("Rendering service health check failed: {}", e);
                        false
                    }
                    Err(_) => {
                        tracing::debug!("Rendering service health check timed out");
                        false
                    }
                }
            } else {
                tracing::debug!("No image renderer configured");
                false
            }
        }
        #[cfg(not(feature = "image-rendering"))]
        {
            tracing::debug!("Image rendering feature not enabled");
            false
        }
    }

    /// Check if image rendering is available (feature + service)
    pub async fn is_image_rendering_available(&self) -> bool {
        #[cfg(feature = "image-rendering")]
        {
            self.diagram_mode.should_attempt_image_rendering()
                && self.is_rendering_service_available().await
        }
        #[cfg(not(feature = "image-rendering"))]
        {
            false
        }
    }

    /// Generates a Markdown report for the given analysis run and issues.
    ///
    /// # Arguments
    ///
    /// * `analysis_run` - The analysis run metadata.
    /// * `issues` - Slice of architectural issues to include in the report.
    /// * `anti_pattern_types` - Map of anti-pattern type IDs to definitions.
    /// * `output_path` - Optional path to write the report to disk.
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The generated Markdown report as a string.
    /// * `Err(std::io::Error)` - If writing to disk fails.
    pub fn generate_markdown_report(
        &self,
        analysis_run: &AnalysisRun,
        issues: &[ArchitecturalIssue],
        anti_pattern_types: &HashMap<i64, AntiPatternType>,
        output_path: Option<&Path>,
        codebase_path: Option<&str>,
    ) -> Result<String, String> {
        info!("Generating markdown report with {} issues", issues.len());

        let now: DateTime<Local> = Local::now();
        let report_title = format!(
            "# Uveddi Architectural Analysis Report\n\n_Generated on {}_\n\n",
            now.format("%Y-%m-%d %H:%M:%S")
        );

        // Build the report sections
        let mut report = String::new();
        report.push_str(&report_title);

        // Add executive summary
        report.push_str("## Executive Summary\n\n");
        report.push_str(&self.generate_executive_summary(analysis_run, issues, codebase_path));
        report.push_str("\n\n");

        // Add severity summary if enabled
        if self.include_severity_summary {
            report.push_str("## Issues by Severity\n\n");
            report.push_str(&self.generate_severity_summary(issues));
            report.push_str("\n\n");
        }

        // Add issues grouped by anti-pattern type
        report.push_str("## Detailed Analysis\n\n");
        report.push_str(&self.generate_detailed_analysis(issues, anti_pattern_types));

        // Add diagrams if enabled
        if self.include_diagrams {
            report.push_str("## Architecture Diagrams\n\n");
            report.push_str(&self.generate_diagrams_section(issues, anti_pattern_types));
        }

        // Write to file if output path is provided
        if let Some(path) = output_path {
            match fs::File::create(path) {
                Ok(mut file) => {
                    if let Err(e) = file.write_all(report.as_bytes()) {
                        error!("Failed to write report to file: {}", e);
                        return Err(format!("Failed to write report to file: {}", e));
                    }
                    info!("Report written to {}", path.display());
                }
                Err(e) => {
                    error!("Failed to create report file: {}", e);
                    return Err(format!("Failed to create report file: {}", e));
                }
            }
        }

        Ok(report)
    }

    /// Generate the report header section
    fn generate_report_header(&self, analysis_run: &AnalysisRun) -> String {
        format!(
            "# Uveddi Architecture Analysis Report\n\n\
            **Analysis Date**: {}\n\
            **Configuration**: {}\n\
            **Run ID**: {}\n\n",
            analysis_run
                .start_time
                .format("%Y-%m-%d %H:%M:%S UTC")
                .to_string(),
            "default", // analysis_run doesn't have config_name field
            analysis_run
                .run_id
                .map(|id| id.to_string())
                .unwrap_or_else(|| "unknown".to_string())
        )
    }

    /// Generate the executive summary section
    fn generate_executive_summary(
        &self,
        analysis_run: &AnalysisRun,
        issues: &[ArchitecturalIssue],
        codebase_path: Option<&str>,
    ) -> String {
        let total_issues = issues.len();
        let high_severity = issues
            .iter()
            .filter(|i| {
                i.severity.to_lowercase() == "high" || i.severity.to_lowercase() == "critical"
            })
            .count();
        let medium_severity = issues
            .iter()
            .filter(|i| i.severity.to_lowercase() == "medium")
            .count();
        let low_severity = issues
            .iter()
            .filter(|i| i.severity.to_lowercase() == "low")
            .count();

        let unique_files = issues
            .iter()
            .map(|i| &i.file_path)
            .collect::<std::collections::HashSet<_>>()
            .len();

        format!(
            "This report analyzes the codebase at `{}` and identified **{} architectural issues** across **{} files**.\n\n\
            - **High Severity**: {} issues\n\
            - **Medium Severity**: {} issues\n\
            - **Low Severity**: {} issues\n\n\
            The analysis took {:.2} seconds to complete.",
            codebase_path.unwrap_or("Unknown"),
            total_issues,
            unique_files,
            high_severity,
            medium_severity,
            low_severity,
            analysis_run.end_time.and_then(|end|
                Some((end - analysis_run.start_time).num_seconds() as f64)
            ).unwrap_or(0.0)
        )
    }

    /// Generate the severity summary section with tables
    fn generate_severity_summary(&self, issues: &[ArchitecturalIssue]) -> String {
        let mut summary = String::new();

        // Group issues by severity
        let high_issues: Vec<_> = issues
            .iter()
            .filter(|i| {
                i.severity.to_lowercase() == "high" || i.severity.to_lowercase() == "critical"
            })
            .collect();
        let medium_issues: Vec<_> = issues
            .iter()
            .filter(|i| i.severity.to_lowercase() == "medium")
            .collect();
        let low_issues: Vec<_> = issues
            .iter()
            .filter(|i| i.severity.to_lowercase() == "low")
            .collect();

        // High severity table
        if !high_issues.is_empty() {
            summary.push_str("### 🔴 High Severity Issues\n\n");
            summary.push_str("| File | Issue |\n");
            summary.push_str("|------|-------|\n");

            for issue in high_issues {
                let file_path = Path::new(&issue.file_path);
                let file_name = file_path.file_name().unwrap_or_default().to_string_lossy();
                let lines = if let (Some(start), Some(end)) = (issue.start_line, issue.end_line) {
                    format!(":{}-{}", start, end)
                } else {
                    String::new()
                };

                summary.push_str(&format!(
                    "| `{}{}`| {} |\n",
                    file_name, lines, issue.description
                ));
            }

            summary.push_str("\n");
        }

        // Medium severity table
        if !medium_issues.is_empty() {
            summary.push_str("### 🟠 Medium Severity Issues\n\n");
            summary.push_str("| File | Issue |\n");
            summary.push_str("|------|-------|\n");

            for issue in medium_issues {
                let file_path = Path::new(&issue.file_path);
                let file_name = file_path.file_name().unwrap_or_default().to_string_lossy();
                let lines = if let (Some(start), Some(end)) = (issue.start_line, issue.end_line) {
                    format!(":{}-{}", start, end)
                } else {
                    String::new()
                };

                summary.push_str(&format!(
                    "| `{}{}`| {} |\n",
                    file_name, lines, issue.description
                ));
            }

            summary.push_str("\n");
        }

        // Low severity table
        if !low_issues.is_empty() {
            summary.push_str("### 🟡 Low Severity Issues\n\n");
            summary.push_str("| File | Issue |\n");
            summary.push_str("|------|-------|\n");

            for issue in low_issues {
                let file_path = Path::new(&issue.file_path);
                let file_name = file_path.file_name().unwrap_or_default().to_string_lossy();
                let lines = if let (Some(start), Some(end)) = (issue.start_line, issue.end_line) {
                    format!(":{}-{}", start, end)
                } else {
                    String::new()
                };

                summary.push_str(&format!(
                    "| `{}{}`| {} |\n",
                    file_name, lines, issue.description
                ));
            }
        }

        summary
    }

    /// Generate the detailed analysis section, grouped by anti-pattern type
    fn generate_detailed_analysis(
        &self,
        issues: &[ArchitecturalIssue],
        anti_pattern_types: &HashMap<i64, AntiPatternType>,
    ) -> String {
        let mut analysis = String::new();

        // Group issues by anti-pattern type
        let mut issues_by_type: HashMap<i64, Vec<&ArchitecturalIssue>> = HashMap::new();
        for issue in issues {
            issues_by_type
                .entry(issue.anti_pattern_type_id)
                .or_default()
                .push(issue);
        }

        // Sort anti-pattern types by name for consistent ordering
        let mut anti_pattern_ids: Vec<i64> = issues_by_type.keys().cloned().collect();
        anti_pattern_ids.sort_by_key(|id| {
            anti_pattern_types
                .get(id)
                .map(|ap| ap.name.clone())
                .unwrap_or_default()
        });

        // Generate a section for each anti-pattern type
        for type_id in anti_pattern_ids {
            let Some(issues) = issues_by_type.get(&type_id) else {
                continue;
            };
            let anti_pattern = anti_pattern_types.get(&type_id);

            let name = anti_pattern
                .map(|ap| ap.name.clone())
                .unwrap_or_else(|| format!("Unknown (ID: {})", type_id));

            let description = anti_pattern
                .map(|ap| ap.description.clone())
                .unwrap_or_else(|| "No description available".to_string());

            analysis.push_str(&format!("### {}\n\n", name));
            analysis.push_str(&format!("{}\n\n", description));

            // Add each issue of this type
            for (i, issue) in issues.iter().enumerate() {
                analysis.push_str(&format!("#### Issue #{}: {}\n\n", i + 1, issue.description));
                analysis.push_str(&format!("- **File**: `{}`\n", issue.file_path));
                analysis.push_str(&format!("- **Severity**: {}\n", issue.severity));

                if let (Some(start), Some(end)) = (issue.start_line, issue.end_line) {
                    analysis.push_str(&format!("- **Location**: Lines {}-{}\n", start, end));
                }

                // Add code snippet if available and enabled
                if self.include_code_snippets {
                    if let Some(ref snippet) = issue.code_snippet {
                        analysis.push_str("\n**Code Snippet**:\n\n");
                        analysis.push_str("```\n");
                        analysis.push_str(snippet);
                        analysis.push_str("\n```\n\n");
                    }
                }

                // Add AI explanation if available and enabled
                if self.include_ai_explanations {
                    if let Some(ref ai_explanation) = issue.ai_explanation {
                        analysis.push_str("**AI Analysis**:\n\n");

                        // Try to parse as JSON first (structured format)
                        if let Ok(json) = serde_json::from_str::<Value>(ai_explanation) {
                            if let Some(title) = json.get("title").and_then(|v| v.as_str()) {
                                analysis.push_str(&format!("*{}*\n\n", title));
                            }

                            if let Some(explanation) =
                                json.get("explanation").and_then(|v| v.as_str())
                            {
                                analysis.push_str(&format!("{}\n\n", explanation));
                            }

                            if self.include_remediation_steps {
                                if let Some(refactoring) =
                                    json.get("refactoring").and_then(|v| v.as_str())
                                {
                                    analysis.push_str("**Recommended Refactoring**:\n\n");
                                    analysis.push_str(&format!("{}\n\n", refactoring));
                                }
                            }
                        } else {
                            // Fall back to raw text if not valid JSON
                            analysis.push_str(&format!("{}\n\n", ai_explanation));
                        }
                    }
                }

                analysis.push_str("\n");
            }
        }

        analysis
    }

    /// Generate diagram with hybrid fallback logic
    async fn generate_diagram_with_fallback(
        &self,
        mermaid_code: &str,
        diagram_type: &str,
    ) -> Result<String, String> {
        match self.diagram_mode {
            DiagramMode::MermaidOnly => {
                Ok(self.generate_mermaid_only_with_instructions(mermaid_code, diagram_type))
            }
            DiagramMode::ImageOnly => {
                #[cfg(feature = "image-rendering")]
                {
                    self.generate_image_only(mermaid_code)
                        .await
                        .map_err(|e| format!("Image rendering failed: {}", e))
                }
                #[cfg(not(feature = "image-rendering"))]
                {
                    Err("Image rendering feature not enabled. Please rebuild with --features image-rendering".to_string())
                }
            }
            DiagramMode::ImageWithFallback => {
                #[cfg(feature = "image-rendering")]
                {
                    if self.is_rendering_service_available().await {
                        match self.generate_image_only(mermaid_code).await {
                            Ok(image_result) => {
                                tracing::info!(
                                    "Successfully generated image for {} diagram",
                                    diagram_type
                                );
                                Ok(image_result)
                            }
                            Err(e) => {
                                tracing::warn!(
                                    "Image rendering failed, falling back to Mermaid-only: {}",
                                    e
                                );
                                Ok(self.generate_mermaid_only_with_instructions(
                                    mermaid_code,
                                    diagram_type,
                                ))
                            }
                        }
                    } else {
                        tracing::info!("Rendering service not available, using Mermaid-only mode for {} diagram", diagram_type);
                        Ok(
                            self.generate_mermaid_only_with_instructions(
                                mermaid_code,
                                diagram_type,
                            ),
                        )
                    }
                }
                #[cfg(not(feature = "image-rendering"))]
                {
                    tracing::info!("Image rendering feature not enabled, using Mermaid-only mode");
                    Ok(self.generate_mermaid_only_with_instructions(mermaid_code, diagram_type))
                }
            }
        }
    }

    #[cfg(feature = "image-rendering")]
    async fn generate_image_only(
        &self,
        mermaid_code: &str,
    ) -> Result<String, crate::error::rendering::RenderingServiceError> {
        if let Some(ref renderer) = self.image_renderer {
            let request = crate::report::image_renderer::RenderRequest {
                mermaid_code: mermaid_code.to_string(),
                format: crate::report::image_renderer::ImageFormat::Png,
                width: Some(800),
                height: Some(600),
            };

            let result = renderer
                .render_diagram(
                    &request.mermaid_code,
                    request.format,
                    request.width.zip(request.height),
                )
                .await?;

            // Return markdown with embedded image
            Ok(format!(
                "## 📊 Architectural Diagram\n\n![Diagram](data:image/png;base64,{})\n\n",
                base64::encode(&result.data)
            ))
        } else {
            Err(crate::error::rendering::RenderingServiceError::ServiceUnavailable)
        }
    }

    /// Generate Mermaid-only output with helpful instructions
    fn generate_mermaid_only_with_instructions(
        &self,
        mermaid_code: &str,
        diagram_type: &str,
    ) -> String {
        format!(
            r#"## 📊 {} Diagram

```mermaid
{}
```

> **💡 Want to see this as an image?**
> 
> **Option 1: Online Rendering (Fastest)**
> - Copy the Mermaid code above
> - Visit [mermaid.live](https://mermaid.live)
> - Paste and generate your image instantly
> 
> **Option 2: Local Rendering Service (Full Control)**
> ```bash
> # Run this in your project directory
> docker-compose up rendering-service
> 
> # Then re-run analysis with image rendering
> uveddi analyze --enable-image-rendering
> ```
> 
> **Option 3: VS Code Extension (Developer Friendly)**
> - Install "Mermaid Markdown Syntax Highlighting"
> - View diagrams directly in your editor
> 
> **Option 4: GitHub/GitLab (Documentation)**
> - Both platforms render Mermaid diagrams natively
> - Perfect for README files and documentation

"#,
            diagram_type, mermaid_code
        )
    }

    /// Generate diagrams section with Mermaid.js syntax
    fn generate_diagrams_section(
        &self,
        issues: &[ArchitecturalIssue],
        anti_pattern_types: &HashMap<i64, AntiPatternType>,
    ) -> String {
        let mut diagrams = String::new();

        // Add dependency cycles diagram if there are cyclic dependency issues
        let cycle_issues: Vec<_> = issues
            .iter()
            .filter(|i| {
                anti_pattern_types
                    .get(&i.anti_pattern_type_id)
                    .map(|apt| apt.name.contains("Cyclic") || apt.name.contains("cycle"))
                    .unwrap_or(false)
            })
            .collect();

        if !cycle_issues.is_empty() {
            diagrams.push_str("### Dependency Cycles\n\n");
            diagrams.push_str("```mermaid\ngraph TD;\n");

            // Extract component names from cycle descriptions
            let mut components = std::collections::HashSet::new();
            let mut dependencies = std::collections::HashSet::new();

            for issue in cycle_issues {
                // Simple parsing of cycle components from description
                // Example: "Cyclic dependency detected between components: A → B → C"
                if let Some(components_str) = issue.description.split(": ").nth(1) {
                    let parts: Vec<&str> = components_str.split(" → ").collect();

                    for part in &parts {
                        components.insert(part.trim().to_string());
                    }

                    // Add dependencies
                    for i in 0..parts.len() - 1 {
                        dependencies
                            .insert((parts[i].trim().to_string(), parts[i + 1].trim().to_string()));
                    }

                    // Add the last to first dependency to complete the cycle
                    if parts.len() > 1 {
                        dependencies.insert((
                            parts[parts.len() - 1].trim().to_string(),
                            parts[0].trim().to_string(),
                        ));
                    }
                }
            }

            // Add components to diagram
            for component in &components {
                diagrams.push_str(&format!("    {}[{}];\n", component, component));
            }

            // Add dependencies to diagram
            for (from, to) in &dependencies {
                diagrams.push_str(&format!("    {} --> {};\n", from, to));
            }

            diagrams.push_str("```\n\n");
        }

        // Add god object diagrams if there are god object issues
        let god_object_issues: Vec<_> = issues
            .iter()
            .filter(|i| {
                // Check both the anti-pattern type name and the description
                let by_type = anti_pattern_types
                    .get(&i.anti_pattern_type_id)
                    .map(|apt| {
                        apt.name.to_lowercase().contains("god object")
                            || apt.name.to_lowercase().contains("large class")
                    })
                    .unwrap_or(false);
                let by_description = i.description.to_lowercase().contains("god object");

                by_type || by_description
            })
            .collect();

        if !god_object_issues.is_empty() {
            diagrams.push_str("### God Objects\n\n");

            for issue in god_object_issues {
                // Extract the god object name from the description
                // Description format: "God Object detected: 'ClassName' has X methods..."
                let god_object_name = if let Some(start) = issue.description.find("'") {
                    if let Some(end) = issue.description[start + 1..].find("'") {
                        &issue.description[start + 1..start + 1 + end]
                    } else {
                        "Unknown"
                    }
                } else {
                    // Fallback: try to extract from code snippet if available
                    if let Some(code) = &issue.code_snippet {
                        if let Some(line) = code.lines().next() {
                            if line.contains("struct") {
                                line.split_whitespace()
                                    .skip_while(|&word| word != "struct")
                                    .nth(1)
                                    .unwrap_or("Unknown")
                            } else {
                                "Unknown"
                            }
                        } else {
                            "Unknown"
                        }
                    } else {
                        "Unknown"
                    }
                };

                diagrams.push_str(&format!("#### {}\n\n", god_object_name));
                diagrams.push_str("```mermaid\nclassDiagram\n");
                diagrams.push_str(&format!("    class {} {{\n", god_object_name));

                // If we have a code snippet, try to extract fields and methods
                if let Some(code) = &issue.code_snippet {
                    let lines: Vec<&str> = code.lines().collect();
                    let mut in_struct = false;
                    let mut in_impl = false;

                    for line in lines {
                        let trimmed = line.trim();

                        // Track if we're inside a struct definition
                        if trimmed.starts_with("struct") || trimmed.starts_with("pub struct") {
                            in_struct = true;
                            continue;
                        }
                        if trimmed.starts_with("impl") {
                            in_impl = true;
                            in_struct = false;
                            continue;
                        }
                        if trimmed == "}" {
                            in_struct = false;
                            in_impl = false;
                            continue;
                        }

                        // Extract struct fields
                        if in_struct && trimmed.contains(":") && !trimmed.starts_with("//") {
                            let field_name = trimmed.split(":").next().unwrap_or("").trim();
                            if !field_name.is_empty() {
                                diagrams.push_str(&format!("        {}: Type\n", field_name));
                            }
                        }

                        // Extract methods from impl block or direct function declarations
                        if (in_impl || !in_struct)
                            && (trimmed.starts_with("fn ") || trimmed.starts_with("pub fn "))
                            && trimmed.contains("(")
                        {
                            // Extract method name
                            let method_name = trimmed
                                .split("(")
                                .next()
                                .unwrap_or("")
                                .trim()
                                .split_whitespace()
                                .last()
                                .unwrap_or("");

                            if !method_name.is_empty() {
                                diagrams.push_str(&format!("        +{}()\n", method_name));
                            }
                        }
                    }
                }

                diagrams.push_str("    }\n```\n\n");
            }
        }

        // Add dead code diagram if there are dead code issues
        let dead_code_issues: Vec<_> = issues
            .iter()
            .filter(|i| {
                i.description.to_lowercase().contains("dead code")
                    || i.description.to_lowercase().contains("unused")
            })
            .collect();

        if !dead_code_issues.is_empty() && dead_code_issues.len() <= 20 {
            diagrams.push_str("### Dead Code Analysis\n\n");
            diagrams.push_str("```mermaid\nflowchart TD\n");
            diagrams.push_str("    A[Project] --> B[Live Code]\n");
            diagrams.push_str("    A --> C[Dead Code]\n");
            diagrams.push_str("    style C fill:#ff9999,stroke:#ff0000,stroke-width:2px\n");

            for (idx, issue) in dead_code_issues.iter().enumerate().take(15) {
                let node_id = format!("D{}", idx + 1);
                let item_name = if let Some(snippet) = &issue.code_snippet {
                    snippet.chars().take(20).collect::<String>()
                } else {
                    format!("Item {}", idx + 1)
                };
                diagrams.push_str(&format!("    C --> {}[{}]\n", node_id, item_name));
                diagrams.push_str(&format!("    style {} fill:#ffcccc\n", node_id));
            }

            diagrams.push_str("```\n\n");
            diagrams.push_str(&format!(
                "*Found {} dead code items. ",
                dead_code_issues.len()
            ));
            if dead_code_issues.len() > 15 {
                diagrams.push_str(&format!("Showing first 15 items.*\n\n"));
            } else {
                diagrams.push_str("*\n\n");
            }
        }

        diagrams
    }

    /// Generates a JSON report for the given analysis run and issues.
    ///
    /// # Arguments
    ///
    /// * `analysis_run` - The analysis run metadata.
    /// * `issues` - Slice of architectural issues to include in the report.
    /// * `anti_pattern_types` - Map of anti-pattern type IDs to definitions.
    /// * `output_path` - Optional path to write the report to disk.
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The generated JSON report as a string.
    /// * `Err(std::io::Error)` - If writing to disk fails.
    pub fn generate_json_report(
        &self,
        analysis_run: &AnalysisRun,
        issues: &[ArchitecturalIssue],
        anti_pattern_types: &HashMap<i64, AntiPatternType>,
        output_path: Option<&Path>,
        codebase_path: Option<&str>,
    ) -> Result<String, String> {
        // Build the report structure
        let mut report = serde_json::Map::new();

        // Add metadata
        let mut metadata = serde_json::Map::new();
        metadata.insert(
            "codebasePath".to_string(),
            Value::String(codebase_path.unwrap_or("Unknown").to_string()),
        );
        metadata.insert(
            "timestamp".to_string(),
            Value::String(Local::now().to_rfc3339()),
        );
        metadata.insert(
            "durationSeconds".to_string(),
            Value::Number(
                serde_json::Number::from_f64(
                    analysis_run
                        .end_time
                        .and_then(|end| Some((end - analysis_run.start_time).num_seconds() as f64))
                        .unwrap_or(0.0),
                )
                .unwrap_or(serde_json::Number::from(0)),
            ),
        );
        report.insert("metadata".to_string(), Value::Object(metadata));

        // Add summary block
        let sev_map = compute_issues_by_severity(issues);
        let mut by_sev_obj = serde_json::Map::new();
        for (k, v) in &sev_map {
            by_sev_obj.insert(k.clone(), Value::Number((*v as u64).into()));
        }
        let summary = serde_json::json!({
            "issuesTotal": issues.len(),
            "issuesBySeverity": Value::Object(by_sev_obj),
            "filesAnalyzed": analysis_run.total_files_analyzed.unwrap_or(0),
            "debtScore": compute_debt_score(issues),
        });
        report.insert("summary".to_string(), summary);

        // Add issues
        let mut json_issues = Vec::new();
        for issue in issues {
            let mut json_issue = serde_json::Map::new();

            json_issue.insert(
                "description".to_string(),
                Value::String(issue.description.clone()),
            );
            json_issue.insert(
                "filePath".to_string(),
                Value::String(issue.file_path.clone()),
            );
            json_issue.insert(
                "severity".to_string(),
                Value::String(issue.severity.clone()),
            );

            if let Some(start_line) = issue.start_line {
                json_issue.insert("startLine".to_string(), Value::Number(start_line.into()));
            }

            if let Some(end_line) = issue.end_line {
                json_issue.insert("endLine".to_string(), Value::Number(end_line.into()));
            }

            if self.include_code_snippets {
                if let Some(snippet) = &issue.code_snippet {
                    json_issue.insert("codeSnippet".to_string(), Value::String(snippet.clone()));
                }
            }

            if self.include_ai_explanations {
                if let Some(explanation) = &issue.ai_explanation {
                    json_issue.insert(
                        "aiExplanation".to_string(),
                        Value::String(explanation.clone()),
                    );
                }
            }

            // Add anti-pattern type information
            if let Some(anti_pattern) = anti_pattern_types.get(&issue.anti_pattern_type_id) {
                debug!(
                    "✅ Issue '{}' (ID: {}) correctly mapped to '{}'",
                    issue.description.chars().take(50).collect::<String>(),
                    issue.anti_pattern_type_id,
                    anti_pattern.name
                );

                json_issue.insert(
                    "antiPatternType".to_string(),
                    Value::String(anti_pattern.name.clone()),
                );
                json_issue.insert(
                    "antiPatternDescription".to_string(),
                    Value::String(anti_pattern.description.clone()),
                );
            } else {
                error!(
                    "❌ Issue '{}' (ID: {}) has no mapping! Available IDs: {:?}",
                    issue.description.chars().take(50).collect::<String>(),
                    issue.anti_pattern_type_id,
                    anti_pattern_types.keys().collect::<Vec<_>>()
                );

                // Fallback: Use issue description or default value
                json_issue.insert(
                    "antiPatternType".to_string(),
                    Value::String(format!("Unknown (ID: {})", issue.anti_pattern_type_id)),
                );
                json_issue.insert(
                    "antiPatternDescription".to_string(),
                    Value::String(
                        "Unknown anti-pattern type - check detector configuration".to_string(),
                    ),
                );
            }

            json_issues.push(Value::Object(json_issue));
        }

        report.insert("issues".to_string(), Value::Array(json_issues));

        // Convert to string
        let json = serde_json::to_string_pretty(&Value::Object(report))
            .map_err(|e| format!("Failed to serialize JSON: {}", e))?;

        // Write to file if output path is provided
        if let Some(path) = output_path {
            match fs::File::create(path) {
                Ok(mut file) => {
                    if let Err(e) = file.write_all(json.as_bytes()) {
                        error!("Failed to write JSON report to file: {}", e);
                        return Err(format!("Failed to write JSON report to file: {}", e));
                    }
                    info!("JSON report written to {}", path.display());
                }
                Err(e) => {
                    error!("Failed to create JSON report file: {}", e);
                    return Err(format!("Failed to create JSON report file: {}", e));
                }
            }
        }

        Ok(json)
    }

    /// Generates an enhanced Markdown report with architectural component diagrams.
    ///
    /// This method generates a comprehensive report that includes traditional issue
    /// analysis enhanced with architectural diagrams generated from extracted components.
    ///
    /// # Arguments
    ///
    /// * `analysis_run` - The analysis run metadata
    /// * `issues` - Slice of architectural issues to include
    /// * `anti_pattern_types` - Map of anti-pattern type definitions
    /// * `components` - Optional architectural components for diagram generation
    ///
    /// # Returns
    ///
    /// * `Ok(EnhancedReportData)` - The generated report with metadata
    /// * `Err(ReportGenerationError)` - If generation fails
    pub fn generate_enhanced_markdown_report(
        &self,
        analysis_run: &AnalysisRun,
        issues: &[ArchitecturalIssue],
        anti_pattern_types: &HashMap<i64, AntiPatternType>,
        components: Option<&[ArchitecturalComponent]>,
        codebase_path: Option<&str>,
    ) -> Result<EnhancedReportData, ReportGenerationError> {
        let mut report = String::new();
        let mut diagrams = Vec::new();

        // Generate standard report sections
        report.push_str(&self.generate_report_header(analysis_run));
        report.push_str(&self.generate_executive_summary(analysis_run, issues, codebase_path));

        if self.include_severity_summary {
            report.push_str(&self.generate_severity_summary(issues));
        }

        // Generate enhanced diagrams section
        if self.include_diagrams && components.is_some() {
            let (diagrams_section, generated_diagrams) = self.generate_enhanced_diagrams_section(
                issues,
                anti_pattern_types,
                components.unwrap(),
            )?;
            report.push_str(&diagrams_section);
            diagrams.extend(generated_diagrams);
        }

        report.push_str(&self.generate_detailed_analysis(issues, anti_pattern_types));

        Ok(EnhancedReportData {
            markdown_content: report,
            diagrams,
            components_analyzed: components.map(|c| c.len()).unwrap_or(0),
            generation_timestamp: chrono::Utc::now(),
        })
    }

    /// Generate comprehensive diagrams section using architectural components
    fn generate_enhanced_diagrams_section(
        &self,
        issues: &[ArchitecturalIssue],
        anti_pattern_types: &HashMap<i64, AntiPatternType>,
        components: &[ArchitecturalComponent],
    ) -> Result<(String, Vec<DiagramMetadata>), ReportGenerationError> {
        let mut section = String::from("## 📊 Architectural Diagrams\n\n");
        let mut generated_diagrams = Vec::new();

        if let Some(ref generator) = self.mermaid_generator {
            // Group issues by anti-pattern type for targeted diagram generation
            let issues_by_type = self.group_issues_by_anti_pattern(issues);

            for (anti_pattern_id, pattern_issues) in issues_by_type {
                if let Some(anti_pattern) = anti_pattern_types.get(&anti_pattern_id) {
                    section.push_str(&format!("### {} Analysis\n\n", anti_pattern.name));

                    // Generate appropriate diagram based on anti-pattern type
                    let diagram_result = self.generate_diagram_for_anti_pattern(
                        generator,
                        components,
                        &pattern_issues,
                        anti_pattern_id,
                    );

                    match diagram_result {
                        Ok(diagram) => {
                            section.push_str("```mermaid\n");
                            section.push_str(&diagram.mermaid_src);
                            section.push_str("\n```\n\n");
                            generated_diagrams.push(diagram);
                        }
                        Err(e) => {
                            error!(
                                "Failed to generate diagram for anti-pattern {}: {}",
                                anti_pattern_id, e
                            );
                            section
                                .push_str("*Diagram generation failed for this anti-pattern.*\n\n");
                        }
                    }
                }
            }

            // Generate overview component diagram
            section.push_str("### System Overview\n\n");
            // No generic generate_diagram method exists; handle as not supported for now
            section.push_str("*Overview diagram generation not implemented.*\n\n");
            // TODO: Implement overview diagram generation if/when supported by MermaidGenerator
        } else {
            section.push_str(
                "*Diagram generation not available - MermaidGenerator not initialized.*\n\n",
            );
        }

        Ok((section, generated_diagrams))
    }

    /// Generate diagram for a specific anti-pattern type
    fn generate_diagram_for_anti_pattern(
        &self,
        generator: &MermaidGenerator,
        components: &[ArchitecturalComponent],
        issues: &[ArchitecturalIssue],
        anti_pattern_id: i64,
    ) -> Result<DiagramMetadata, MermaidGenerationError> {
        // Create severity mapping from issues
        let mut severity_data = HashMap::new();
        for issue in issues {
            // Extract component IDs from file paths (simplified)
            if let Some(component_id) =
                self.find_component_by_file_path(components, &issue.file_path)
            {
                severity_data.insert(component_id, issue.severity.clone());
            }
        }

        // Determine diagram type based on anti-pattern
        let diagram_type = match anti_pattern_id {
            1 => DiagramType::Dependency, // Assuming 1 is cyclic dependencies
            2 => DiagramType::Class,      // Assuming 2 is god objects
            _ => DiagramType::Component,
        };

        match diagram_type {
            DiagramType::Class => {
                // God Object diagram: requires god_object_components and member_counts
                // TODO: Replace with actual logic to extract these from issues/components
                let god_object_components = components
                    .iter()
                    .map(|c| c.component_id)
                    .collect::<Vec<_>>();
                let member_counts = HashMap::new();
                generator.generate_god_object_diagram(
                    components,
                    &god_object_components,
                    &member_counts,
                )
            }
            DiagramType::Dependency => {
                // Cyclic Dependencies diagram: requires cycles and cycle_edges
                // TODO: Replace with actual logic to extract these from issues/components
                let cycles = Vec::new();
                let cycle_edges = Vec::new();
                generator.generate_cyclic_dependencies_diagram(components, &cycles, &cycle_edges)
            }
            _ => Err(MermaidGenerationError::InvalidSpecError(
                "Diagram type not supported in report generator".to_string(),
            )),
        }
    }

    /// Find component by file path
    fn find_component_by_file_path(
        &self,
        components: &[ArchitecturalComponent],
        file_path: &str,
    ) -> Option<Uuid> {
        components
            .iter()
            .find(|c| c.file_path.to_string_lossy() == file_path)
            .map(|c| c.component_id)
    }

    /// Group issues by anti-pattern type
    fn group_issues_by_anti_pattern(
        &self,
        issues: &[ArchitecturalIssue],
    ) -> HashMap<i64, Vec<ArchitecturalIssue>> {
        let mut grouped = HashMap::new();
        for issue in issues {
            grouped
                .entry(issue.anti_pattern_type_id)
                .or_insert_with(Vec::new)
                .push(issue.clone());
        }
        grouped
    }

    /// Generate JSON report with diagram metadata
    pub fn generate_enhanced_json_report(
        &self,
        analysis_run: &AnalysisRun,
        issues: &[ArchitecturalIssue],
        anti_pattern_types: &[AntiPatternType],
        components: Option<&[ArchitecturalComponent]>,
        diagrams: &[DiagramMetadata],
    ) -> Result<String, serde_json::Error> {
        let anti_pattern_map: HashMap<i64, &AntiPatternType> = anti_pattern_types
            .iter()
            .filter_map(|apt| apt.anti_pattern_type_id.map(|id| (id, apt)))
            .collect();

        let summary = self.create_enhanced_summary(issues, components, diagrams);

        let report_data = serde_json::json!({
            "analysis_run": analysis_run,
            "summary": summary,
            "issues": issues,
            "anti_patterns": anti_pattern_types,
            "components": components.unwrap_or(&[]),
            "diagrams": diagrams.iter().map(|d| serde_json::json!({
                "type": d.diagram_type,
                "mermaid_src": d.mermaid_src,
                "image_path": d.image_path,
                "generated_at": d.generated_at,
                "component_count": d.components.len(),
                "validation_metrics": d.validation_metrics
            })).collect::<Vec<_>>(),
            "metadata": {
                "report_version": "2.0",
                "enhanced_features": {
                    "architectural_components": components.is_some(),
                    "diagram_generation": !diagrams.is_empty(),
                    "component_extraction": true
                }
            }
        });

        serde_json::to_string_pretty(&report_data)
    }

    /// Generate a stunning interactive HTML report with embedded diagrams
    ///
    /// # Arguments
    ///
    /// * `analysis_run` - The analysis run metadata
    /// * `issues` - The architectural issues found
    /// * `anti_pattern_types` - The anti-pattern type definitions
    /// * `output_path` - Optional path to write the HTML file
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The generated HTML content
    /// * `Err(String)` - Error message if generation fails
    pub async fn generate_html_report(
        &mut self,
        analysis_run: &AnalysisRun,
        issues: &[ArchitecturalIssue],
        anti_pattern_types: &HashMap<i64, AntiPatternType>,
        output_path: Option<&Path>,
        codebase_path: Option<&str>,
    ) -> Result<String, String> {
        info!(
            "Generating interactive HTML report with {} issues",
            issues.len()
        );

        let now: DateTime<Local> = Local::now();

        // Extract output directory from path
        let output_dir = output_path.and_then(|p| p.parent());
        if let Some(path) = output_path {
            info!("Output path: {}", path.display());
            if let Some(dir) = output_dir {
                info!("Output directory: {}", dir.display());
            } else {
                warn!(
                    "Failed to extract output directory from path: {}",
                    path.display()
                );
            }
        }

        // Generate the HTML content
        let html_content = self
            .generate_html_content(analysis_run, issues, anti_pattern_types, &now, output_dir)
            .await?;

        // Write to file if output path is provided
        if let Some(path) = output_path {
            match fs::File::create(path) {
                Ok(mut file) => {
                    if let Err(e) = file.write_all(html_content.as_bytes()) {
                        error!("Failed to write HTML report to file: {}", e);
                        return Err(format!("Failed to write HTML report to file: {}", e));
                    }
                    info!("HTML report written to {}", path.display());
                }
                Err(e) => {
                    error!("Failed to create HTML report file: {}", e);
                    return Err(format!("Failed to create HTML report file: {}", e));
                }
            }
        }

        Ok(html_content)
    }

    /// Generate the complete HTML content
    async fn generate_html_content(
        &mut self,
        analysis_run: &AnalysisRun,
        issues: &[ArchitecturalIssue],
        anti_pattern_types: &HashMap<i64, AntiPatternType>,
        timestamp: &DateTime<Local>,
        output_dir: Option<&Path>,
    ) -> Result<String, String> {
        // Feature flag: Use modern template-based generator if available
        if let Some(ref mut modern_gen) = self.modern_generator {
            info!("Using modern template-based report generator");
            match modern_gen
                .generate_html_report(analysis_run, issues, anti_pattern_types, output_dir)
                .await
            {
                Ok(html) => {
                    info!(
                        "Modern report generated successfully ({} bytes)",
                        html.len()
                    );
                    return Ok(html);
                }
                Err(e) => {
                    warn!("Modern generator failed, falling back to legacy. Full error details below:");
                    log_tera_error_chain(&e);
                    // Fall through to legacy generator
                }
            }
        }
        let mut html = String::new();

        // HTML document structure
        html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
        html.push_str(&self.generate_html_head());
        html.push_str("</head>\n<body>\n");

        // Main container
        html.push_str("<div class=\"container\">\n");

        // Header
        html.push_str(&self.generate_html_header(analysis_run, timestamp));

        // Navigation
        html.push_str(&self.generate_html_navigation());

        // Executive summary
        html.push_str(&self.generate_html_executive_summary(analysis_run, issues));

        // Severity dashboard
        html.push_str(&self.generate_html_severity_dashboard(issues));

        // Architecture diagrams section (with links to separate files)
        html.push_str(
            &self
                .generate_html_diagrams_section(issues, anti_pattern_types, output_dir)
                .await,
        );

        // Detailed issues with inline diagrams
        html.push_str(&self.generate_html_detailed_issues(issues, anti_pattern_types));

        // Footer
        html.push_str(&self.generate_html_footer());

        html.push_str("</div>\n");

        // JavaScript
        html.push_str(&self.generate_html_scripts());

        html.push_str("</body>\n</html>");

        Ok(html)
    }

    /// Generate HTML head with styles and meta tags
    fn generate_html_head(&self) -> String {
        format!(
            r#"
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Uveddi Architectural Analysis Report</title>
    
    <!-- Mermaid.js for diagram rendering -->
    <script src="https://cdnjs.cloudflare.com/ajax/libs/mermaid/10.6.1/mermaid.min.js"></script>
    
    <!-- Font Awesome for icons -->
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.4.0/css/all.min.css">
    
    <style>
        /* Modern CSS Variables for theming */
        :root {{
            --primary-color: #2563eb;
            --secondary-color: #64748b;
            --success-color: #059669;
            --warning-color: #d97706;
            --error-color: #dc2626;
            --bg-primary: #ffffff;
            --bg-secondary: #f8fafc;
            --bg-tertiary: #e2e8f0;
            --text-primary: #0f172a;
            --text-secondary: #475569;
            --border-color: #e2e8f0;
            --shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
            --radius: 8px;
        }}

        [data-theme="dark"] {{
            --primary-color: #3b82f6;
            --secondary-color: #94a3b8;
            --success-color: #10b981;
            --warning-color: #f59e0b;
            --error-color: #ef4444;
            --bg-primary: #0f172a;
            --bg-secondary: #1e293b;
            --bg-tertiary: #334155;
            --text-primary: #f1f5f9;
            --text-secondary: #cbd5e1;
            --border-color: #475569;
            --shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.3);
        }}

        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}

        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background-color: var(--bg-secondary);
            color: var(--text-primary);
            line-height: 1.6;
            transition: background-color 0.3s ease, color 0.3s ease;
        }}

        .container {{
            max-width: 1200px;
            margin: 0 auto;
            padding: 20px;
        }}

        /* Header Styles */
        .header {{
            background: linear-gradient(135deg, var(--primary-color), var(--secondary-color));
            color: white;
            padding: 2rem;
            border-radius: var(--radius);
            margin-bottom: 2rem;
            position: relative;
            overflow: hidden;
        }}

        .header::before {{
            content: '';
            position: absolute;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background: linear-gradient(45deg, transparent 49%, rgba(255,255,255,0.1) 50%, transparent 51%);
            animation: shimmer 3s infinite;
        }}

        @keyframes shimmer {{
            0% {{ transform: translateX(-100%); }}
            100% {{ transform: translateX(100%); }}
        }}

        .header h1 {{
            font-size: 2.5rem;
            font-weight: 700;
            margin-bottom: 0.5rem;
            position: relative;
            z-index: 1;
        }}

        .header-meta {{
            position: relative;
            z-index: 1;
            opacity: 0.9;
        }}

        /* Navigation */
        .nav {{
            background: var(--bg-primary);
            border-radius: var(--radius);
            padding: 1rem;
            margin-bottom: 2rem;
            box-shadow: var(--shadow);
            display: flex;
            gap: 1rem;
            flex-wrap: wrap;
            align-items: center;
            justify-content: space-between;
        }}

        .nav-links {{
            display: flex;
            gap: 1rem;
            flex-wrap: wrap;
        }}

        .nav-link {{
            color: var(--text-secondary);
            text-decoration: none;
            padding: 0.5rem 1rem;
            border-radius: var(--radius);
            transition: all 0.3s ease;
            display: flex;
            align-items: center;
            gap: 0.5rem;
        }}

        .nav-link:hover {{
            background: var(--bg-tertiary);
            color: var(--primary-color);
            transform: translateY(-2px);
        }}

        .theme-toggle {{
            background: none;
            border: 2px solid var(--border-color);
            color: var(--text-primary);
            padding: 0.5rem 1rem;
            border-radius: var(--radius);
            cursor: pointer;
            transition: all 0.3s ease;
            display: flex;
            align-items: center;
            gap: 0.5rem;
        }}

        .theme-toggle:hover {{
            border-color: var(--primary-color);
            color: var(--primary-color);
        }}

        /* Card Styles */
        .card {{
            background: var(--bg-primary);
            border-radius: var(--radius);
            padding: 1.5rem;
            margin-bottom: 2rem;
            box-shadow: var(--shadow);
            border: 1px solid var(--border-color);
            transition: all 0.3s ease;
        }}

        .card:hover {{
            transform: translateY(-2px);
            box-shadow: 0 8px 15px -3px rgba(0, 0, 0, 0.1);
        }}

        .card h2 {{
            color: var(--primary-color);
            margin-bottom: 1rem;
            display: flex;
            align-items: center;
            gap: 0.5rem;
        }}

        /* Dashboard Grid */
        .dashboard {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 1.5rem;
            margin-bottom: 2rem;
        }}

        .stat-card {{
            background: var(--bg-primary);
            border-radius: var(--radius);
            padding: 1.5rem;
            text-align: center;
            box-shadow: var(--shadow);
            border: 1px solid var(--border-color);
            transition: all 0.3s ease;
        }}

        .stat-card:hover {{
            transform: translateY(-4px);
            box-shadow: 0 12px 20px -5px rgba(0, 0, 0, 0.15);
        }}

        .stat-number {{
            font-size: 2.5rem;
            font-weight: 700;
            margin-bottom: 0.5rem;
        }}

        .stat-label {{
            color: var(--text-secondary);
            text-transform: uppercase;
            font-size: 0.875rem;
            font-weight: 600;
            letter-spacing: 0.05em;
        }}

        .stat-critical {{ color: var(--error-color); }}
        .stat-high {{ color: var(--warning-color); }}
        .stat-medium {{ color: var(--primary-color); }}
        .stat-low {{ color: var(--success-color); }}

        /* Severity Badges */
        .severity-badge {{
            display: inline-flex;
            align-items: center;
            gap: 0.25rem;
            padding: 0.25rem 0.75rem;
            border-radius: 9999px;
            font-size: 0.75rem;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: 0.05em;
        }}

        .severity-critical {{
            background: rgba(220, 38, 38, 0.1);
            color: var(--error-color);
            border: 1px solid rgba(220, 38, 38, 0.2);
        }}

        .severity-high {{
            background: rgba(217, 119, 6, 0.1);
            color: var(--warning-color);
            border: 1px solid rgba(217, 119, 6, 0.2);
        }}

        .severity-medium {{
            background: rgba(37, 99, 235, 0.1);
            color: var(--primary-color);
            border: 1px solid rgba(37, 99, 235, 0.2);
        }}

        .severity-low {{
            background: rgba(5, 150, 105, 0.1);
            color: var(--success-color);  
            border: 1px solid rgba(5, 150, 105, 0.2);
        }}

        /* Issue Cards */
        .issue-card {{
            background: var(--bg-primary);
            border-radius: var(--radius);
            padding: 1.5rem;
            margin-bottom: 1rem;
            border-left: 4px solid var(--primary-color);
            box-shadow: var(--shadow);
            transition: all 0.3s ease;
        }}

        .issue-card:hover {{
            transform: translateX(4px);
            box-shadow: 0 8px 15px -3px rgba(0, 0, 0, 0.1);
        }}

        .issue-header {{
            display: flex;
            justify-content: space-between;
            align-items: flex-start;
            margin-bottom: 1rem;
            flex-wrap: wrap;
            gap: 1rem;
        }}

        .issue-title {{
            font-size: 1.125rem;
            font-weight: 600;
            color: var(--text-primary);
            flex: 1;
        }}

        .code-snippet {{
            background: var(--bg-tertiary);
            border-radius: var(--radius);
            padding: 1rem;
            margin: 1rem 0;
            font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
            font-size: 0.875rem;
            overflow-x: auto;
            border: 1px solid var(--border-color);
        }}

        /* Diagram Styles */
        .diagram-container {{
            background: var(--bg-primary);
            border-radius: var(--radius);
            padding: 1.5rem;
            margin: 1rem 0;
            border: 1px solid var(--border-color);
            text-align: center;
            position: relative;
        }}

        .diagram-title {{
            font-size: 1.125rem;
            font-weight: 600;
            margin-bottom: 1rem;
            color: var(--primary-color);
        }}

        /* Diagram Link Cards */
        .diagram-intro {{
            margin-bottom: 1.5rem;
            padding: 1rem;
            background: var(--bg-secondary);
            border-radius: var(--radius);
            color: var(--text-secondary);
            font-size: 0.95rem;
        }}

        .diagram-links-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 1rem;
            margin-top: 1rem;
        }}

        .diagram-link-card {{
            background: var(--bg-primary);
            border: 1px solid var(--border-color);
            border-radius: var(--radius);
            padding: 1.5rem;
            transition: all 0.2s ease;
            position: relative;
        }}

        .diagram-link-card:hover {{
            transform: translateY(-2px);
            box-shadow: var(--shadow);
            border-color: var(--primary-color);
        }}

        .diagram-card-header {{
            display: flex;
            align-items: center;
            gap: 0.5rem;
            margin-bottom: 1rem;
        }}

        .diagram-card-header h3 {{
            margin: 0;
            font-size: 1.1rem;
            font-weight: 600;
            color: var(--text-primary);
        }}

        .diagram-card-header i {{
            color: var(--primary-color);
        }}

        .diagram-card-body p {{
            color: var(--text-secondary);
            margin-bottom: 1rem;
            font-size: 0.9rem;
        }}

        .diagram-button {{
            display: inline-flex;
            align-items: center;
            gap: 0.5rem;
            background: var(--primary-color);
            color: white;
            text-decoration: none;
            padding: 0.75rem 1.5rem;
            border-radius: var(--radius);
            font-weight: 500;
            font-size: 0.9rem;
            transition: all 0.2s ease;
        }}

        .diagram-button:hover {{
            background: var(--secondary-color);
            transform: translateY(-1px);
            box-shadow: var(--shadow);
        }}

        /* Diagram Toggle Interface Styles */
        .diagram-buttons {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 1rem;
            margin-bottom: 2rem;
        }}

        .diagram-button-card {{
            background: var(--bg-primary);
            border: 1px solid var(--border-color);
            border-radius: var(--radius);
            padding: 1.5rem;
            transition: all 0.2s ease;
            position: relative;
        }}

        .diagram-button-card:hover {{
            transform: translateY(-2px);
            box-shadow: var(--shadow);
            border-color: var(--primary-color);
        }}

        .diagram-toggle-btn {{
            display: inline-flex;
            align-items: center;
            gap: 0.5rem;
            background: var(--primary-color);
            color: white;
            border: none;
            padding: 0.75rem 1.5rem;
            border-radius: var(--radius);
            font-weight: 500;
            font-size: 0.9rem;
            cursor: pointer;
            transition: all 0.2s ease;
        }}

        .diagram-toggle-btn:hover {{
            background: var(--secondary-color);
            transform: translateY(-1px);
            box-shadow: var(--shadow);
        }}

        .diagram-content {{
            margin-top: 2rem;
        }}

        #active-diagram-header {{
            display: none;
            justify-content: space-between;
            align-items: center;
            padding: 1rem;
            background: var(--bg-secondary);
            border: 1px solid var(--border-color);
            border-radius: var(--radius);
            margin-bottom: 1rem;
        }}

        #active-diagram-header h3 {{
            margin: 0;
            color: var(--text-primary);
            font-size: 1.2rem;
        }}

        .hide-diagram-btn {{
            display: inline-flex;
            align-items: center;
            gap: 0.5rem;
            background: var(--secondary-color);
            color: white;
            border: none;
            padding: 0.5rem 1rem;
            border-radius: calc(var(--radius) / 2);
            font-size: 0.85rem;
            cursor: pointer;
            transition: all 0.2s ease;
        }}

        .hide-diagram-btn:hover {{
            background: #ef4444;
            transform: translateY(-1px);
        }}

        .embedded-diagram {{
            opacity: 0;
            transform: translateY(-10px);
            transition: all 0.3s ease;
            background: var(--bg-primary);
            border: 1px solid var(--border-color);
            border-radius: var(--radius);
            padding: 1.5rem;
            margin-bottom: 1rem;
        }}

        .embedded-diagram.visible {{
            opacity: 1;
            transform: translateY(0);
        }}
        
        /* SVG Diagram Styles - Critical for rendering */
        .svg-diagram {{
            width: 100%;
            min-height: 400px;
            display: block;
            overflow: visible;
            text-align: center;
        }}
        
        .svg-diagram svg {{
            width: 100% !important;
            height: auto !important;
            max-width: none !important;
            display: block;
            margin: 0 auto;
        }}

        .mermaid {{
            background: var(--bg-primary);
            border-radius: var(--radius);
        }}

        /* Collapsible sections */
        .collapsible {{
            cursor: pointer;
            display: flex;
            align-items: center;
            gap: 0.5rem;
            user-select: none;
        }}

        .collapsible-icon {{
            transition: transform 0.3s ease;
        }}

        .collapsible.active .collapsible-icon {{
            transform: rotate(90deg);
        }}

        .collapsible-content {{
            max-height: 0;
            overflow: hidden;
            transition: max-height 0.3s ease;
        }}

        .collapsible-content.active {{
            max-height: 1000px;
        }}

        /* Search and Filter */
        .search-filter {{
            background: var(--bg-primary);
            border-radius: var(--radius);
            padding: 1.5rem;
            margin-bottom: 2rem;
            box-shadow: var(--shadow);
        }}

        .search-input {{
            width: 100%;
            padding: 0.75rem;
            border: 2px solid var(--border-color);
            border-radius: var(--radius);
            background: var(--bg-secondary);
            color: var(--text-primary);
            font-size: 1rem;
            transition: border-color 0.3s ease;
        }}

        .search-input:focus {{
            outline: none;
            border-color: var(--primary-color);
        }}

        .filter-buttons {{
            display: flex;
            gap: 0.5rem;
            margin-top: 1rem;
            flex-wrap: wrap;
        }}

        .filter-btn {{
            padding: 0.5rem 1rem;
            border: 2px solid var(--border-color);
            background: var(--bg-secondary);
            color: var(--text-primary);
            border-radius: var(--radius);
            cursor: pointer;
            transition: all 0.3s ease;
            font-size: 0.875rem;
        }}

        .filter-btn:hover, .filter-btn.active {{
            border-color: var(--primary-color);
            background: var(--primary-color);
            color: white;
        }}

        /* Footer */
        .footer {{
            text-align: center;
            padding: 2rem;
            color: var(--text-secondary);
            border-top: 1px solid var(--border-color);
            margin-top: 3rem;
        }}

        /* Responsive Design */
        @media (max-width: 768px) {{
            .container {{
                padding: 10px;
            }}

            .header h1 {{
                font-size: 2rem;
            }}

            .nav {{
                flex-direction: column;
                align-items: stretch;
            }}

            .nav-links {{
                justify-content: center;
            }}

            .dashboard {{
                grid-template-columns: 1fr;
            }}

            .issue-header {{
                flex-direction: column;
                align-items: stretch;
            }}
        }}

        /* Loading Animation */
        .loading {{
            display: inline-block;
            width: 20px;
            height: 20px;
            border: 3px solid var(--border-color);
            border-radius: 50%;
            border-top-color: var(--primary-color);
            animation: spin 1s ease-in-out infinite;
        }}

        @keyframes spin {{
            to {{ transform: rotate(360deg); }}
        }}

        /* Hidden class for filtering */
        .hidden {{
            display: none !important;
        }}

        /* Anti-pattern section styles */
        .anti-pattern-section {{
            margin-bottom: 2.5rem;
            border: 1px solid var(--border-color);
            border-radius: var(--radius);
            background: var(--bg-secondary);
            padding: 1.5rem;
        }}

        .anti-pattern-header {{
            color: var(--primary-color);
            font-size: 1.5rem;
            font-weight: 600;
            margin-bottom: 1rem;
            display: flex;
            align-items: center;
            gap: 0.75rem;
            border-bottom: 2px solid var(--border-color);
            padding-bottom: 0.5rem;
        }}

        .diagram-explanation {{
            background: var(--bg-tertiary);
            border-radius: var(--radius);
            padding: 1rem;
            margin: 0.5rem 0 1rem 0;
            border-left: 4px solid var(--primary-color);
            font-style: italic;
            color: var(--text-secondary);
        }}

        .diagram-explanation p {{
            margin: 0;
        }}

        /* Enhanced diagram containers */
        .diagram-isolator {{
            background: var(--bg-primary);
            border: 2px solid var(--border-color);
            border-radius: var(--radius);
            margin: 1.5rem 0;
            transition: all 0.3s ease;
        }}

        .diagram-isolator:hover {{
            border-color: var(--primary-color);
            box-shadow: var(--shadow);
        }}

        /* Issue numbering within sections */
        .anti-pattern-section .issue-card .issue-title {{
            font-size: 1rem;
            font-weight: 500;
        }}

        /* Responsive diagram containers */
        @media (max-width: 768px) {{
            .anti-pattern-section {{
                padding: 1rem;
            }}
            
            .anti-pattern-header {{
                font-size: 1.25rem;
                flex-direction: column;
                align-items: flex-start;
                gap: 0.5rem;
            }}
            
            .diagram-explanation {{
                font-size: 0.875rem;
                padding: 0.75rem;
            }}
        }}
    </style>
"#
        )
    }

    /// Generate HTML header section
    fn generate_html_header(
        &self,
        analysis_run: &AnalysisRun,
        timestamp: &DateTime<Local>,
    ) -> String {
        format!(
            r#"
    <div class="header">
        <h1><i class="fas fa-chart-line"></i> Uveddi Architectural Analysis</h1>
        <div class="header-meta">
            <p><i class="fas fa-calendar"></i> Generated on {}</p>
            <p><i class="fas fa-cog"></i> Run ID: {}</p>
        </div>
    </div>
"#,
            timestamp.format("%Y-%m-%d %H:%M:%S"),
            analysis_run
                .run_id
                .map(|id| id.to_string())
                .unwrap_or_else(|| "unknown".to_string())
        )
    }

    /// Generate HTML navigation
    fn generate_html_navigation(&self) -> String {
        let mut nav = String::new();
        nav.push_str("\n    <nav class=\"nav\">\n");
        nav.push_str("        <div class=\"nav-links\">\n");
        nav.push_str("            <a href=\"#summary\" class=\"nav-link\"><i class=\"fas fa-chart-pie\"></i> Summary</a>\n");
        nav.push_str("            <a href=\"#issues\" class=\"nav-link\"><i class=\"fas fa-exclamation-triangle\"></i> Issues & Diagrams</a>\n");
        nav.push_str("        </div>\n");
        nav.push_str("        <button class=\"theme-toggle\" onclick=\"toggleTheme()\">\n");
        nav.push_str("            <i class=\"fas fa-moon\"></i> Dark Mode\n");
        nav.push_str("        </button>\n");
        nav.push_str("    </nav>\n");
        nav
    }

    /// Generate HTML executive summary section
    fn generate_html_executive_summary(
        &self,
        analysis_run: &AnalysisRun,
        issues: &[ArchitecturalIssue],
    ) -> String {
        let total_issues = issues.len();
        let critical_issues = issues
            .iter()
            .filter(|i| i.severity.to_lowercase() == "critical")
            .count();
        let high_issues = issues
            .iter()
            .filter(|i| i.severity.to_lowercase() == "high")
            .count();
        let medium_issues = issues
            .iter()
            .filter(|i| i.severity.to_lowercase() == "medium")
            .count();
        let low_issues = issues
            .iter()
            .filter(|i| i.severity.to_lowercase() == "low")
            .count();

        let unique_files = issues
            .iter()
            .map(|i| &i.file_path)
            .collect::<std::collections::HashSet<_>>()
            .len();

        format!(
            r#"
    <section id="summary" class="card">
        <h2><i class="fas fa-chart-pie"></i> Executive Summary</h2>
        <p>This analysis identified <strong>{} architectural issues</strong> across <strong>{} files</strong> in your codebase.</p>
        
        <div class="dashboard">
            <div class="stat-card">
                <div class="stat-number stat-critical">{}</div>
                <div class="stat-label">Critical Issues</div>
            </div>
            <div class="stat-card">
                <div class="stat-number stat-high">{}</div>
                <div class="stat-label">High Issues</div>
            </div>
            <div class="stat-card">
                <div class="stat-number stat-medium">{}</div>
                <div class="stat-label">Medium Issues</div>
            </div>
            <div class="stat-card">
                <div class="stat-number stat-low">{}</div>
                <div class="stat-label">Low Issues</div>
            </div>
        </div>
    </section>
"#,
            total_issues, unique_files, critical_issues, high_issues, medium_issues, low_issues
        )
    }

    /// Generate HTML severity dashboard
    fn generate_html_severity_dashboard(&self, issues: &[ArchitecturalIssue]) -> String {
        let mut severity_stats = HashMap::new();
        for issue in issues {
            *severity_stats.entry(&issue.severity).or_insert(0) += 1;
        }

        let mut dashboard_html = String::from(
            r#"
    <section class="card">
        <h2><i class="fas fa-tachometer-alt"></i> Severity Dashboard</h2>
        <div class="dashboard">
"#,
        );

        for (severity, count) in &severity_stats {
            let icon = match severity.as_str() {
                "critical" => "fas fa-times-circle",
                "high" => "fas fa-exclamation-triangle",
                "medium" => "fas fa-exclamation-circle",
                "low" => "fas fa-info-circle",
                _ => "fas fa-circle",
            };

            dashboard_html.push_str(&format!(
                r#"
            <div class="stat-card">
                <div class="stat-number stat-{}">
                    <i class="{}"></i> {}
                </div>
                <div class="stat-label">{} Severity</div>
            </div>
"#,
                severity,
                icon,
                count,
                severity.to_uppercase()
            ));
        }

        dashboard_html.push_str("        </div>\n    </section>\n");
        dashboard_html
    }

    /// Generate HTML diagrams section with embedded diagrams that can be toggled
    async fn generate_html_diagrams_section(
        &self,
        issues: &[ArchitecturalIssue],
        anti_pattern_types: &HashMap<i64, AntiPatternType>,
        _output_dir: Option<&Path>,
    ) -> String {
        let diagrams_content = self.generate_diagrams_section(issues, anti_pattern_types);

        if diagrams_content.trim().is_empty() {
            return String::new();
        }

        let mut html = String::from(
            r#"
    <section id="diagrams" class="card">
        <h2 class="collapsible" onclick="toggleSection('diagrams-content')">
            <i class="fas fa-chevron-right collapsible-icon"></i>
            <i class="fas fa-project-diagram"></i> 
            Architecture Diagrams
        </h2>
        <div id="diagrams-content" class="collapsible-content">
            <p class="diagram-intro">
                <i class="fas fa-info-circle"></i> 
                The following diagrams provide visual representations of the architectural issues found in your codebase. Click any diagram button to view it inline.
            </p>
            
            <div class="diagram-buttons">
"#,
        );

        // Parse Mermaid content to extract diagrams
        let lines: Vec<&str> = diagrams_content.lines().collect();
        let mut in_mermaid_block = false;
        let mut mermaid_content = String::new();
        let mut diagram_title = String::new();
        let mut diagrams_to_render = Vec::new();

        // Collect all diagrams first
        for line in lines {
            if line.starts_with("### ") {
                // If we were in a mermaid block, save it for rendering
                if in_mermaid_block && !mermaid_content.trim().is_empty() {
                    diagrams_to_render.push((diagram_title.clone(), mermaid_content.clone()));
                    mermaid_content.clear();
                    in_mermaid_block = false;
                }
                diagram_title = line[4..].to_string();
            } else if line.starts_with("```mermaid") {
                in_mermaid_block = true;
                mermaid_content.clear();
            } else if line == "```" && in_mermaid_block {
                if !mermaid_content.trim().is_empty() {
                    diagrams_to_render.push((diagram_title.clone(), mermaid_content.clone()));
                }
                mermaid_content.clear();
                in_mermaid_block = false;
            } else if in_mermaid_block {
                mermaid_content.push_str(line);
                mermaid_content.push('\n');
            }
        }

        // Handle any remaining mermaid block
        if in_mermaid_block && !mermaid_content.trim().is_empty() {
            diagrams_to_render.push((diagram_title, mermaid_content));
        }

        // Generate diagram buttons
        for (i, (title, _)) in diagrams_to_render.iter().enumerate() {
            let diagram_id = format!("diagram-{}", i);
            let icon = match title {
                title if title.contains("Cycle") => "fas fa-sync-alt",
                title if title.contains("God Object") => "fas fa-cube",
                title if title.contains("Dead Code") => "fas fa-skull-crossbones",
                _ => "fas fa-chart-bar",
            };

            html.push_str(&format!(
                r#"
                <div class="diagram-button-card">
                    <div class="diagram-card-header">
                        <i class="{}"></i>
                        <h3>{}</h3>
                    </div>
                    <div class="diagram-card-body">
                        <p>Interactive architectural diagram showing {}.</p>
                        <button onclick="toggleDiagram('{}')" class="diagram-toggle-btn">
                            <i class="fas fa-eye"></i> Show Diagram
                        </button>
                    </div>
                </div>
"#,
                icon,
                title,
                title.to_lowercase(),
                diagram_id
            ));
        }

        html.push_str(
            r#"
            </div>
            
            <div class="diagram-content">
                <div id="active-diagram-header" style="display: none;">
                    <h3 id="active-diagram-title"></h3>
                    <button onclick="hideAllDiagrams()" class="hide-diagram-btn">
                        <i class="fas fa-eye-slash"></i> Hide Diagram
                    </button>
                </div>
"#,
        );

        // Generate embedded hidden diagrams
        for (i, (title, code)) in diagrams_to_render.iter().enumerate() {
            let diagram_id = format!("diagram-{}", i);
            let unique_mermaid_id = format!(
                "mermaid-{}",
                uuid::Uuid::new_v4().to_string().replace('-', "")[..8].to_string()
            );

            html.push_str(&format!(
                r#"
                <div id="{}" class="embedded-diagram" style="display: none;">
                    <div class="diagram-container">
                        <div class="mermaid" id="{}">{}</div>
                    </div>
                </div>
"#,
                diagram_id,
                unique_mermaid_id,
                code.trim()
            ));
        }

        html.push_str(
            r#"
            </div>
        </div>
    </section>
"#,
        );
        html
    }

    /// Generate a separate HTML file for a single diagram
    async fn generate_separate_diagram_file(
        &self,
        output_path: &Path,
        title: &str,
        mermaid_code: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let diagram_id = format!(
            "diagram-{}",
            uuid::Uuid::new_v4().to_string().replace('-', "")[..8].to_string()
        );
        let diagram_content = self.render_diagram_content(mermaid_code, &diagram_id).await;

        let html_content = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{} - Uveddi Diagram</title>
    
    <!-- Mermaid.js for diagram rendering -->
    <script src="https://cdnjs.cloudflare.com/ajax/libs/mermaid/10.6.1/mermaid.min.js"></script>
    
    <!-- Font Awesome for icons -->
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.4.0/css/all.min.css">
    
    <style>
        :root {{
            --primary-color: #2563eb;
            --secondary-color: #64748b;
            --bg-primary: #ffffff;
            --bg-secondary: #f8fafc;
            --text-primary: #0f172a;
            --text-secondary: #475569;
            --border-color: #e2e8f0;
            --shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
            --radius: 8px;
        }}

        [data-theme="dark"] {{
            --primary-color: #3b82f6;
            --secondary-color: #94a3b8;
            --bg-primary: #0f172a;
            --bg-secondary: #1e293b;
            --text-primary: #f1f5f9;
            --text-secondary: #cbd5e1;
            --border-color: #475569;
            --shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.3);
        }}

        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}

        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Noto Sans', Helvetica, Arial, sans-serif;
            line-height: 1.6;
            color: var(--text-primary);
            background-color: var(--bg-primary);
            padding: 20px;
        }}

        .diagram-container {{
            max-width: 100%;
            margin: 0 auto;
            background: var(--bg-primary);
            border: 1px solid var(--border-color);
            border-radius: var(--radius);
            box-shadow: var(--shadow);
            padding: 2rem;
            overflow-x: auto;
        }}

        .diagram-header {{
            text-align: center;
            margin-bottom: 2rem;
            padding-bottom: 1rem;
            border-bottom: 1px solid var(--border-color);
        }}

        .diagram-title {{
            font-size: 1.75rem;
            font-weight: 600;
            color: var(--primary-color);
            margin-bottom: 0.5rem;
            display: flex;
            align-items: center;
            justify-content: center;
            gap: 0.5rem;
        }}

        .theme-toggle {{
            position: absolute;
            top: 20px;
            right: 20px;
            background: var(--primary-color);
            color: white;
            border: none;
            border-radius: var(--radius);
            padding: 0.5rem 1rem;
            cursor: pointer;
            font-size: 0.9rem;
            transition: all 0.2s ease;
        }}

        .theme-toggle:hover {{
            transform: translateY(-1px);
            box-shadow: var(--shadow);
        }}

        .mermaid {{
            display: flex;
            justify-content: center;
            min-height: 400px;
            background: var(--bg-secondary);
            border-radius: var(--radius);
            padding: 1rem;
        }}

        .svg-diagram {{
            display: flex;
            justify-content: center;
            width: 100%;
            overflow-x: auto;
        }}

        .svg-diagram svg {{
            max-width: 100%;
            height: auto;
            background: white;
            border-radius: var(--radius);
        }}

        .back-link {{
            display: inline-flex;
            align-items: center;
            gap: 0.5rem;
            color: var(--primary-color);
            text-decoration: none;
            font-weight: 500;
            margin-bottom: 1rem;
            padding: 0.5rem 1rem;
            border: 1px solid var(--primary-color);
            border-radius: var(--radius);
            transition: all 0.2s ease;
        }}

        .back-link:hover {{
            background: var(--primary-color);
            color: white;
            transform: translateY(-1px);
        }}

        @media (max-width: 768px) {{
            body {{
                padding: 10px;
            }}
            
            .diagram-container {{
                padding: 1rem;
            }}
            
            .diagram-title {{
                font-size: 1.25rem;
            }}
        }}
    </style>
</head>
<body>
    <button class="theme-toggle" onclick="toggleTheme()" id="themeToggle">
        <i class="fas fa-moon"></i> Dark Mode
    </button>

    <div class="diagram-container">
        <div class="diagram-header">
            <a href="javascript:history.back()" class="back-link">
                <i class="fas fa-arrow-left"></i> Back to Report
            </a>
            <div class="diagram-title">
                <i class="fas fa-project-diagram"></i> {}
            </div>
        </div>

        <div class="svg-diagram" id="{}">
{}
        </div>
    </div>

    <script>
        // Initialize Mermaid
        mermaid.initialize({{
            theme: document.documentElement.getAttribute('data-theme') === 'dark' ? 'dark' : 'default',
            startOnLoad: true,
            flowchart: {{
                useMaxWidth: true,
                htmlLabels: true
            }},
            sequence: {{
                useMaxWidth: true
            }},
            journey: {{
                useMaxWidth: true
            }}
        }});

        // Theme toggle functionality
        function toggleTheme() {{
            const html = document.documentElement;
            const themeToggle = document.getElementById('themeToggle');
            const currentTheme = html.getAttribute('data-theme');
            
            if (currentTheme === 'dark') {{
                html.removeAttribute('data-theme');
                themeToggle.innerHTML = '<i class="fas fa-moon"></i> Dark Mode';
                localStorage.setItem('theme', 'light');
                mermaid.initialize({{ theme: 'default' }});
            }} else {{
                html.setAttribute('data-theme', 'dark');
                themeToggle.innerHTML = '<i class="fas fa-sun"></i> Light Mode';
                localStorage.setItem('theme', 'dark');
                mermaid.initialize({{ theme: 'dark' }});
            }}
            
            // Re-render mermaid diagrams with new theme
            location.reload();
        }}

        // Load saved theme
        const savedTheme = localStorage.getItem('theme');
        if (savedTheme === 'dark') {{
            document.documentElement.setAttribute('data-theme', 'dark');
            document.getElementById('themeToggle').innerHTML = '<i class="fas fa-sun"></i> Light Mode';
        }}
    </script>
</body>
</html>"#,
            title, title, diagram_id, diagram_content
        );

        let mut file = fs::File::create(output_path)?;
        file.write_all(html_content.as_bytes())?;
        info!("Generated separate diagram file: {}", output_path.display());

        Ok(())
    }

    /// Render diagram content (SVG or Mermaid fallback)
    async fn render_diagram_content(&self, mermaid_code: &str, diagram_id: &str) -> String {
        // Try to generate static SVG first
        if let Some(ref svg_generator) = self.svg_generator {
            let svg_content = svg_generator
                .generate_svg_with_fallback(mermaid_code, diagram_id)
                .await;
            return svg_content;
        }

        // Fallback to client-side Mermaid
        format!(
            r#"<div class="mermaid" id="{}">{}</div>"#,
            diagram_id,
            mermaid_code.trim()
        )
    }

    /// Render a single Mermaid diagram as HTML with static SVG generation
    async fn render_mermaid_diagram_async(&self, title: &str, mermaid_code: &str) -> String {
        let diagram_id = format!(
            "diagram-{}",
            uuid::Uuid::new_v4().to_string().replace('-', "")[..8].to_string()
        );

        // Try to generate static SVG first
        if let Some(ref svg_generator) = self.svg_generator {
            let svg_content = svg_generator
                .generate_svg_with_fallback(mermaid_code, &diagram_id)
                .await;
            return format!(
                r#"
            <div class="diagram-isolator" style="display: block; width: 100%; margin: 1rem 0; isolation: isolate;">
                <div class="diagram-container">
                    <div class="diagram-title">{}</div>
                    <div class="svg-diagram" id="{}">
                        {}
                    </div>
                </div>
            </div>
"#,
                title, diagram_id, svg_content
            );
        }

        // Fallback to client-side Mermaid (existing behavior)
        format!(
            r#"
            <div class="diagram-isolator" style="display: block; width: 100%; margin: 1rem 0; isolation: isolate;">
                <div class="diagram-container">
                    <div class="diagram-title">{}</div>
                    <div class="mermaid" id="{}">{}</div>
                </div>
            </div>
"#,
            title,
            diagram_id,
            mermaid_code.trim()
        )
    }

    /// Synchronous wrapper for backward compatibility
    fn render_mermaid_diagram(&self, title: &str, mermaid_code: &str) -> String {
        // For now, use the fallback approach - we'll update the calling code to be async
        let diagram_id = format!(
            "diagram-{}",
            uuid::Uuid::new_v4().to_string().replace('-', "")[..8].to_string()
        );
        format!(
            r#"
            <div class="diagram-isolator" style="display: block; width: 100%; margin: 1rem 0; isolation: isolate;">
                <div class="diagram-container">
                    <div class="diagram-title">{}</div>
                    <div class="mermaid" id="{}">{}</div>
                </div>
            </div>
"#,
            title,
            diagram_id,
            mermaid_code.trim()
        )
    }

    /// Generate HTML detailed issues section with inline diagrams
    fn generate_html_detailed_issues(
        &self,
        issues: &[ArchitecturalIssue],
        anti_pattern_types: &HashMap<i64, AntiPatternType>,
    ) -> String {
        let mut html = String::from(
            r#"
    <section id="issues" class="card">
        <h2><i class="fas fa-exclamation-triangle"></i> Detailed Issues</h2>
        
        <div class="search-filter">
            <input type="text" id="searchInput" class="search-input" placeholder="Search issues..." onkeyup="filterIssues()">
            <div class="filter-buttons">
                <button class="filter-btn active" onclick="filterBySeverity('all')">All</button>
                <button class="filter-btn" onclick="filterBySeverity('critical')">Critical</button>
                <button class="filter-btn" onclick="filterBySeverity('high')">High</button>
                <button class="filter-btn" onclick="filterBySeverity('medium')">Medium</button>
                <button class="filter-btn" onclick="filterBySeverity('low')">Low</button>
            </div>
        </div>
        
        <div id="issues-container">
"#,
        );

        // Group issues by anti-pattern type for better organization and diagram generation
        let mut issues_by_type: HashMap<i64, Vec<&ArchitecturalIssue>> = HashMap::new();
        for issue in issues {
            issues_by_type
                .entry(issue.anti_pattern_type_id)
                .or_default()
                .push(issue);
        }

        // Sort anti-pattern types by name for consistent ordering
        let mut anti_pattern_ids: Vec<i64> = issues_by_type.keys().cloned().collect();
        anti_pattern_ids.sort_by_key(|id| {
            anti_pattern_types
                .get(id)
                .map(|ap| ap.name.clone())
                .unwrap_or_default()
        });

        for type_id in anti_pattern_ids {
            let type_issues = issues_by_type.get(&type_id).unwrap();
            let anti_pattern = anti_pattern_types.get(&type_id);
            let pattern_name = anti_pattern.map(|ap| ap.name.as_str()).unwrap_or("Unknown");

            // Infer actual pattern name from issues
            let inferred_pattern = self.infer_anti_pattern_from_issues(type_issues);
            let display_name = if inferred_pattern != "Unknown" {
                &inferred_pattern
            } else {
                pattern_name
            };

            // Add section header for this anti-pattern type
            html.push_str(&format!(
                r#"
            <div class="anti-pattern-section">
                <h3 class="anti-pattern-header">
                    <i class="fas fa-layer-group"></i> {} Issues ({} found)
                </h3>
"#,
                display_name,
                type_issues.len()
            ));

            // Generate diagram for this specific anti-pattern type if diagrams are enabled
            if self.include_diagrams {
                let diagram_html = self.generate_inline_diagram_for_anti_pattern(
                    type_id,
                    type_issues,
                    anti_pattern_types,
                );
                if !diagram_html.is_empty() {
                    html.push_str(&diagram_html);
                }
            }

            // Add individual issues for this type
            for (i, issue) in type_issues.iter().enumerate() {
                html.push_str(&format!(r#"
                <div class="issue-card" data-severity="{}" data-pattern="{}">
                    <div class="issue-header">
                        <div class="issue-title">
                            <i class="fas fa-bug"></i> Issue #{}: {}
                        </div>
                        <span class="severity-badge severity-{}">
                            <i class="fas fa-circle"></i> {}
                        </span>
                    </div>
                    
                    <div class="issue-details">
                        <p><strong>File:</strong> <code>{}</code></p>
                        {}<p><strong>Description:</strong> {}</p>
                        
                        {}
                        
                        {}
                    </div>
                </div>
    "#,
                    issue.severity,
                    pattern_name.to_lowercase().replace(' ', "-"),
                    i + 1,
                    pattern_name,
                    issue.severity,
                    issue.severity.to_uppercase(),
                    issue.file_path,
                    if let (Some(start), Some(end)) = (issue.start_line, issue.end_line) {
                        format!("<p><strong>Lines:</strong> {}-{}</p>", start, end)
                    } else if let Some(start) = issue.start_line {
                        format!("<p><strong>Line:</strong> {}</p>", start)
                    } else {
                        String::new()
                    },
                    issue.description,
                    if self.include_code_snippets && issue.code_snippet.is_some() {
                        format!("<div class=\"code-snippet\"><pre><code>{}</code></pre></div>", 
                            issue.code_snippet.as_ref().unwrap())
                    } else {
                        String::new()
                    },
                    if self.include_ai_explanations && issue.ai_explanation.is_some() {
                        format!("<div class=\"collapsible-section\">
                            <h4 class=\"collapsible\" onclick=\"toggleSection('ai-explanation-{}-{}')\">
                                <i class=\"fas fa-chevron-right collapsible-icon\"></i>
                                <i class=\"fas fa-robot\"></i> AI Explanation
                            </h4>
                            <div id=\"ai-explanation-{}-{}\" class=\"collapsible-content\">
                                <p>{}</p>
                            </div>
                        </div>", type_id, i, type_id, i, issue.ai_explanation.as_ref().unwrap())
                    } else {
                        String::new()
                    }
                ));
            }

            html.push_str("            </div>\n"); // Close anti-pattern-section
        }

        html.push_str("        </div>\n    </section>\n");
        html
    }

    /// Generate inline diagram for a specific anti-pattern type
    fn generate_inline_diagram_for_anti_pattern(
        &self,
        type_id: i64,
        issues: &[&ArchitecturalIssue],
        anti_pattern_types: &HashMap<i64, AntiPatternType>,
    ) -> String {
        let anti_pattern = anti_pattern_types.get(&type_id);
        let pattern_name = anti_pattern.map(|ap| ap.name.as_str()).unwrap_or("Unknown");

        // Determine anti-pattern type from actual issue descriptions
        let inferred_pattern = self.infer_anti_pattern_from_issues(issues);
        let actual_pattern_name = if inferred_pattern != "Unknown" {
            &inferred_pattern
        } else {
            pattern_name
        };

        // Generate appropriate diagram based on inferred or provided anti-pattern type
        let mermaid_code = match actual_pattern_name.to_lowercase().as_str() {
            name if name.contains("god object") || name.contains("large class") => {
                self.generate_god_object_diagram_for_issues(issues)
            }
            name if name.contains("cyclic") || name.contains("cycle") => {
                self.generate_cyclic_dependency_diagram_for_issues(issues)
            }
            name if name.contains("dead code") || name.contains("unused") => {
                self.generate_dead_code_diagram_for_issues(issues)
            }
            _ => {
                // Use actual issue data for meaningful diagrams
                self.generate_meaningful_diagram_for_issues(actual_pattern_name, issues)
            }
        };

        if mermaid_code.is_empty() {
            return String::new();
        }

        let diagram_id = format!("inline-diagram-{}", type_id);

        // Generate static SVG if available
        if let Some(ref svg_generator) = self.svg_generator {
            // Actually generate SVG using the SVG generator
            let svg_result = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    svg_generator
                        .generate_svg_with_fallback(&mermaid_code, &diagram_id)
                        .await
                })
            });

            format!(
                r#"
                <div class="diagram-isolator" style="display: block; width: 100%; margin: 1rem 0; isolation: isolate;">
                    <div class="diagram-container">
                        <div class="diagram-title">
                            <i class="fas fa-project-diagram"></i> {} Analysis
                        </div>
                        <div id="diagram-explanation-{}" class="diagram-explanation">
                            <p><em>This diagram shows the architectural issues found in this anti-pattern category. Each node represents a component or relationship affected by the identified problems.</em></p>
                        </div>
                        <div class="svg-diagram" id="{}">
                            {}
                        </div>
                    </div>
                </div>
"#,
                pattern_name, type_id, diagram_id, svg_result
            )
        } else {
            // Fallback to client-side Mermaid
            format!(
                r#"
                <div class="diagram-isolator" style="display: block; width: 100%; margin: 1rem 0; isolation: isolate;">
                    <div class="diagram-container">
                        <div class="diagram-title">
                            <i class="fas fa-project-diagram"></i> {} Analysis
                        </div>
                        <div id="diagram-explanation-{}" class="diagram-explanation">
                            <p><em>This diagram shows the architectural issues found in this anti-pattern category.</em></p>
                        </div>
                        <div class="mermaid" id="{}">{}</div>
                    </div>
                </div>
"#,
                pattern_name,
                type_id,
                diagram_id,
                mermaid_code.trim()
            )
        }
    }

    /// Generate God Object diagram for specific issues
    fn generate_god_object_diagram_for_issues(&self, issues: &[&ArchitecturalIssue]) -> String {
        let mut diagram = String::from("classDiagram\n");

        for (i, issue) in issues.iter().enumerate() {
            // Extract class name from description or file path
            let class_name = if let Some(start) = issue.description.find("'") {
                if let Some(end) = issue.description[start + 1..].find("'") {
                    &issue.description[start + 1..start + 1 + end]
                } else {
                    &format!("GodObject{}", i + 1)
                }
            } else {
                &format!("GodObject{}", i + 1)
            };

            diagram.push_str(&format!("    class {} {{\n", class_name));

            // Add some representative methods/fields
            if let Some(code) = &issue.code_snippet {
                let lines: Vec<&str> = code.lines().take(8).collect(); // Limit to avoid huge diagrams
                for line in lines {
                    let trimmed = line.trim();
                    if trimmed.starts_with("fn ") || trimmed.starts_with("pub fn ") {
                        if let Some(method_name) = trimmed.split('(').next() {
                            let method = method_name.split_whitespace().last().unwrap_or("method");
                            diagram.push_str(&format!("        +{}()\n", method));
                        }
                    } else if trimmed.contains(":") && !trimmed.starts_with("//") {
                        if let Some(field_name) = trimmed.split(':').next() {
                            let field = field_name.trim().replace("pub ", "");
                            if !field.is_empty() && field.len() < 20 {
                                diagram.push_str(&format!("        {}: Type\n", field));
                            }
                        }
                    }
                }
            } else {
                // Generic large class representation
                diagram.push_str("        +method1()\n");
                diagram.push_str("        +method2()\n");
                diagram.push_str("        +method3()\n");
                diagram.push_str("        ...\n");
                diagram.push_str("        +methodN()\n");
            }

            diagram.push_str("    }\n");

            // Add style to highlight the god object
            diagram.push_str(&format!(
                "    style {} fill:#ffcccc,stroke:#ff0000,stroke-width:3px\n",
                class_name
            ));
        }

        diagram
    }

    /// Generate Cyclic Dependency diagram for specific issues
    fn generate_cyclic_dependency_diagram_for_issues(
        &self,
        issues: &[&ArchitecturalIssue],
    ) -> String {
        let mut diagram = String::from("graph TD\n");
        let mut components = std::collections::HashSet::new();
        let mut dependencies = std::collections::HashSet::new();

        for issue in issues {
            // Extract component names from cycle descriptions
            if let Some(components_str) = issue.description.split(": ").nth(1) {
                let parts: Vec<&str> = components_str.split(" → ").collect();

                for part in &parts {
                    let clean_name = part.trim().replace(" ", "_").replace(".", "_");
                    components.insert(clean_name);
                }

                // Add dependencies
                for i in 0..parts.len() - 1 {
                    let from = parts[i].trim().replace(" ", "_").replace(".", "_");
                    let to = parts[i + 1].trim().replace(" ", "_").replace(".", "_");
                    dependencies.insert((from, to));
                }

                // Add the last to first dependency to complete the cycle
                if parts.len() > 1 {
                    let from = parts[parts.len() - 1]
                        .trim()
                        .replace(" ", "_")
                        .replace(".", "_");
                    let to = parts[0].trim().replace(" ", "_").replace(".", "_");
                    dependencies.insert((from, to));
                }
            }
        }

        // Add components to diagram
        for component in &components {
            diagram.push_str(&format!(
                "    {}[{}]\n",
                component,
                component.replace("_", " ")
            ));
        }

        // Add dependencies with cycle highlighting
        for (from, to) in &dependencies {
            diagram.push_str(&format!("    {} -->|depends on| {}\n", from, to));
            diagram.push_str(&format!("    style {} fill:#ffcccc,stroke:#ff0000\n", from));
            diagram.push_str(&format!("    style {} fill:#ffcccc,stroke:#ff0000\n", to));
        }

        diagram
    }

    /// Generate Dead Code diagram for specific issues
    fn generate_dead_code_diagram_for_issues(&self, issues: &[&ArchitecturalIssue]) -> String {
        let mut diagram = String::from("flowchart TD\n");
        diagram.push_str("    Live[Live Code]\n");
        diagram.push_str("    Dead[Dead Code]\n");
        diagram.push_str("    style Dead fill:#ffcccc,stroke:#ff0000,stroke-width:2px\n");

        for (i, issue) in issues.iter().enumerate().take(10) {
            // Limit to prevent huge diagrams
            let item_id = format!("DeadItem{}", i + 1);
            let item_name = if let Some(snippet) = &issue.code_snippet {
                snippet
                    .lines()
                    .next()
                    .unwrap_or("Dead Code")
                    .chars()
                    .take(15)
                    .collect::<String>()
            } else {
                format!("Unused Item {}", i + 1)
            };

            diagram.push_str(&format!("    Dead --> {}[{}]\n", item_id, item_name));
            diagram.push_str(&format!("    style {} fill:#ffe6e6\n", item_id));
        }

        if issues.len() > 10 {
            diagram.push_str("    Dead --> More[... and more]\n");
            diagram.push_str("    style More fill:#ffe6e6\n");
        }

        diagram
    }

    /// Infer anti-pattern type from issue descriptions
    fn infer_anti_pattern_from_issues(&self, issues: &[&ArchitecturalIssue]) -> String {
        // Count different types of issues
        let mut god_object_count = 0;
        let mut dead_code_count = 0;
        let mut cyclic_dep_count = 0;

        for issue in issues {
            let desc = issue.description.to_lowercase();
            if desc.contains("god object")
                || desc.contains("has") && desc.contains("methods") && desc.contains("fields")
            {
                god_object_count += 1;
            } else if desc.contains("dead code")
                || desc.contains("not used")
                || desc.contains("unused")
            {
                dead_code_count += 1;
            } else if desc.contains("cyclic") || desc.contains("circular") {
                cyclic_dep_count += 1;
            }
        }

        // Return the most common pattern type
        if god_object_count > 0 {
            "God Objects".to_string()
        } else if dead_code_count > 0 {
            "Dead Code".to_string()
        } else if cyclic_dep_count > 0 {
            "Cyclic Dependencies".to_string()
        } else {
            "Unknown".to_string()
        }
    }

    /// Generate meaningful diagram using actual issue data
    fn generate_meaningful_diagram_for_issues(
        &self,
        pattern_name: &str,
        issues: &[&ArchitecturalIssue],
    ) -> String {
        let mut diagram = String::from("mindmap\n");
        diagram.push_str(&format!("  root(({}))\n", pattern_name));

        for (i, issue) in issues.iter().enumerate().take(8) {
            let file_name = std::path::Path::new(&issue.file_path)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .replace(".rs", "");

            // Extract meaningful info from description
            let short_desc = if issue.description.len() > 50 {
                format!("{}...", &issue.description[..47])
            } else {
                issue.description.clone()
            };

            diagram.push_str(&format!("    Issue{}\n", i + 1));
            diagram.push_str(&format!("      {}\n", file_name));
            diagram.push_str(&format!("      ({})\n", issue.severity.to_uppercase()));

            // Add description as sub-node if it's meaningful
            if !short_desc.contains("Unknown") && short_desc.len() > 10 {
                diagram.push_str(&format!("        \"{}\"\n", short_desc.replace('"', "'")));
            }
        }

        if issues.len() > 8 {
            diagram.push_str("    More\n");
            diagram.push_str(&format!("      {} more issues\n", issues.len() - 8));
        }

        diagram
    }

    /// Generate generic diagram for other anti-pattern types
    fn generate_generic_diagram_for_issues(
        &self,
        pattern_name: &str,
        issues: &[&ArchitecturalIssue],
    ) -> String {
        let mut diagram = String::from("mindmap\n");
        diagram.push_str(&format!("  root(({}))\n", pattern_name));

        for (i, issue) in issues.iter().enumerate().take(8) {
            // Limit for readability
            let file_name = std::path::Path::new(&issue.file_path)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy();

            diagram.push_str(&format!("    Issue{}\n", i + 1));
            diagram.push_str(&format!("      {}\n", file_name));
            diagram.push_str(&format!("      ({})\n", issue.severity.to_uppercase()));
        }

        if issues.len() > 8 {
            diagram.push_str("    More\n");
            diagram.push_str(&format!("      {} more issues\n", issues.len() - 8));
        }

        diagram
    }

    /// Generate HTML footer
    fn generate_html_footer(&self) -> String {
        let mut footer = String::new();
        footer.push_str("\n    <footer class=\"footer\">\n");
        footer.push_str("        <p><i class=\"fas fa-code\"></i> Generated by <strong>Uveddi</strong> - AI-Powered Code Analysis</p>\n");
        footer.push_str("        <p>For more information, visit <a href=\"https://github.com/botzrDev/uveddi\" target=\"_blank\">github.com/botzrDev/uveddi</a></p>\n");
        footer.push_str("    </footer>\n");
        footer
    }

    /// Generate JavaScript for interactivity
    fn generate_html_scripts(&self) -> String {
        format!(
            r#"
    <script>
        // Initialize Mermaid.js for diagram rendering with unique IDs
        mermaid.initialize({{
            startOnLoad: false,
            theme: 'default',
            deterministicIds: true,
            themeVariables: {{
                primaryColor: '#2563eb',
                primaryTextColor: '#0f172a',
                primaryBorderColor: '#64748b',
                lineColor: '#64748b',
                background: '#ffffff',
                secondaryColor: '#f8fafc',
                tertiaryColor: '#e2e8f0'
            }}
        }});

        // Generate unique seeds for each diagram to prevent ID conflicts
        document.addEventListener('DOMContentLoaded', function() {{
            const mermaidElements = document.querySelectorAll('.mermaid');
            mermaidElements.forEach((element, index) => {{
                const uniqueId = `mermaid-diagram-${{index}}-${{Date.now()}}`;
                const graphDefinition = element.textContent.trim();
                
                const config = {{
                    deterministicIds: true,
                    deterministicIDSeed: `seed-${{index}}-${{Date.now()}}`,
                    theme: document.body.dataset.theme === 'dark' ? 'dark' : 'default'
                }};
                
                mermaid.render(uniqueId + '-svg', graphDefinition, config).then((result) => {{
                    element.innerHTML = result.svg;
                }}).catch((error) => {{
                    console.error('Mermaid rendering error for diagram', index, ':', error);
                    element.innerHTML = '<div class="diagram-error" style="color: red; padding: 1rem; border: 1px solid red; border-radius: 4px;">Diagram rendering failed: ' + error.message + '</div>';
                }});
            }});
        }});

        // Re-render Mermaid diagrams when theme changes
        function rerenderMermaidDiagrams() {{
            const isDark = document.body.dataset.theme === 'dark';
            const theme = isDark ? 'dark' : 'default';
            const themeVariables = isDark ? {{
                primaryColor: '#3b82f6',
                primaryTextColor: '#f1f5f9',
                primaryBorderColor: '#94a3b8',
                lineColor: '#94a3b8',
                background: '#0f172a',
                secondaryColor: '#1e293b',
                tertiaryColor: '#334155'
            }} : {{
                primaryColor: '#2563eb',
                primaryTextColor: '#0f172a',
                primaryBorderColor: '#64748b',
                lineColor: '#64748b',
                background: '#ffffff',
                secondaryColor: '#f8fafc',
                tertiaryColor: '#e2e8f0'
            }};

            mermaid.initialize({{
                startOnLoad: false,
                theme: theme,
                themeVariables: themeVariables
            }});

            // Re-render all mermaid diagrams with unique seeds
            const mermaidElements = document.querySelectorAll('.mermaid');
            mermaidElements.forEach((element, index) => {{
                const graphDefinition = element.textContent;
                const diagramId = element.id || `mermaid-diagram-${{index}}`;
                element.innerHTML = '';
                element.removeAttribute('data-processed');
                
                // Configure unique seed for each diagram to prevent ID conflicts
                mermaid.initialize({{
                    startOnLoad: false,
                    theme: theme,
                    deterministicIds: true,
                    deterministicIDSeed: diagramId,
                    themeVariables: themeVariables
                }});
                
                mermaid.render(diagramId, graphDefinition, (svgCode) => {{
                    element.innerHTML = svgCode;
                }});
            }});
        }}

        // Theme toggle functionality
        function toggleTheme() {{
            const body = document.body;
            const themeToggle = document.querySelector('.theme-toggle');
            
            if (body.dataset.theme === 'dark') {{
                body.dataset.theme = 'light';
                themeToggle.innerHTML = '<i class="fas fa-moon"></i> Dark Mode';
                localStorage.setItem('theme', 'light');
            }} else {{
                body.dataset.theme = 'dark';
                themeToggle.innerHTML = '<i class="fas fa-sun"></i> Light Mode';
                localStorage.setItem('theme', 'dark');
            }}
            
            // Re-render Mermaid diagrams with new theme
            setTimeout(rerenderMermaidDiagrams, 100);
        }}

        // Load saved theme
        const savedTheme = localStorage.getItem('theme') || 'light';
        document.body.dataset.theme = savedTheme;
        if (savedTheme === 'dark') {{
            document.querySelector('.theme-toggle').innerHTML = '<i class="fas fa-sun"></i> Light Mode';
        }}

        // Collapsible sections
        function toggleSection(sectionId) {{
            const content = document.getElementById(sectionId);
            const icon = content.previousElementSibling.querySelector('.collapsible-icon');
            
            content.classList.toggle('active');
            icon.style.transform = content.classList.contains('active') ? 'rotate(90deg)' : 'rotate(0deg)';
        }}

        // Diagram toggle functionality
        function toggleDiagram(diagramId) {{
            // Hide all other diagrams first
            hideAllDiagrams();
            
            // Show the selected diagram
            const diagram = document.getElementById(diagramId);
            const header = document.getElementById('active-diagram-header');
            const title = document.getElementById('active-diagram-title');
            
            if (diagram && header && title) {{
                // Update title
                const diagramTitle = document.querySelector(`button[onclick="toggleDiagram('${{diagramId}}')"]`)
                    .closest('.diagram-button-card')
                    .querySelector('h3').textContent;
                title.textContent = diagramTitle;
                
                // Show diagram with smooth transition
                diagram.style.display = 'block';
                header.style.display = 'flex';
                
                // Trigger reflow for animation
                diagram.offsetHeight;
                diagram.style.opacity = '1';
                diagram.style.transform = 'translateY(0)';
                
                // Update button text
                const button = document.querySelector(`button[onclick="toggleDiagram('${{diagramId}}')"]`);
                button.innerHTML = '<i class="fas fa-eye-slash"></i> Hide Diagram';
                button.onclick = () => hideAllDiagrams();
                
                // Re-render the specific Mermaid diagram
                setTimeout(() => {{
                    const mermaidElement = diagram.querySelector('.mermaid');
                    if (mermaidElement && !mermaidElement.getAttribute('data-processed')) {{
                        const graphDefinition = mermaidElement.textContent;
                        const diagramElementId = mermaidElement.id;
                        mermaidElement.innerHTML = '';
                        mermaidElement.removeAttribute('data-processed');
                        
                        mermaid.render(diagramElementId, graphDefinition, (svgCode) => {{
                            mermaidElement.innerHTML = svgCode;
                        }});
                    }}
                }}, 50);
            }}
        }}

        function hideAllDiagrams() {{
            // Hide all diagrams
            document.querySelectorAll('.embedded-diagram').forEach(diagram => {{
                diagram.style.opacity = '0';
                diagram.style.transform = 'translateY(-10px)';
                setTimeout(() => {{
                    diagram.style.display = 'none';
                }}, 200);
            }});
            
            // Hide header
            const header = document.getElementById('active-diagram-header');
            if (header) {{
                header.style.display = 'none';
            }}
            
            // Reset all button texts
            document.querySelectorAll('.diagram-toggle-btn').forEach(button => {{
                const diagramId = button.getAttribute('onclick').match(/'([^']+)'/)[1];
                button.innerHTML = '<i class="fas fa-eye"></i> Show Diagram';
                button.onclick = () => toggleDiagram(diagramId);
            }});
        }}

        // Issue filtering
        let currentSeverityFilter = 'all';
        let currentSearchQuery = '';

        function filterBySeverity(severity) {{
            currentSeverityFilter = severity;
            
            // Update active button
            document.querySelectorAll('.filter-btn').forEach(btn => btn.classList.remove('active'));
            event.target.classList.add('active');
            
            applyFilters();
        }}

        function filterIssues() {{
            currentSearchQuery = document.getElementById('searchInput').value.toLowerCase();
            applyFilters();
        }}

        function applyFilters() {{
            const issues = document.querySelectorAll('.issue-card');
            
            issues.forEach(issue => {{
                const severity = issue.dataset.severity;
                const text = issue.textContent.toLowerCase();
                
                const matchesSeverity = currentSeverityFilter === 'all' || severity === currentSeverityFilter;
                const matchesSearch = currentSearchQuery === '' || text.includes(currentSearchQuery);
                
                if (matchesSeverity && matchesSearch) {{
                    issue.classList.remove('hidden');
                }} else {{
                    issue.classList.add('hidden');
                }}
            }});
        }}

        // Initialize page
        document.addEventListener('DOMContentLoaded', function() {{
            // Auto-expand diagrams section
            const diagramsContent = document.getElementById('diagrams-content');
            if (diagramsContent) {{
                diagramsContent.classList.add('active');
                const icon = diagramsContent.previousElementSibling.querySelector('.collapsible-icon');
                if (icon) {{
                    icon.style.transform = 'rotate(90deg)';
                }}
            }}
            
            // Initial render of all Mermaid diagrams with unique IDs
            setTimeout(() => {{
                const mermaidElements = document.querySelectorAll('.mermaid');
                mermaidElements.forEach((element, index) => {{
                    const graphDefinition = element.textContent;
                    const diagramId = element.id || `mermaid-diagram-${{index}}`;
                    element.innerHTML = '';
                    element.removeAttribute('data-processed');
                    
                    // Configure unique seed for each diagram to prevent ID conflicts
                    mermaid.initialize({{
                        startOnLoad: false,
                        theme: document.body.dataset.theme === 'dark' ? 'dark' : 'default',
                        deterministicIds: true,
                        deterministicIDSeed: diagramId,
                        themeVariables: document.body.dataset.theme === 'dark' ? {{
                            primaryColor: '#3b82f6',
                            primaryTextColor: '#f1f5f9',
                            primaryBorderColor: '#64748b',
                            lineColor: '#64748b',
                            background: '#0f172a',
                            secondaryColor: '#1e293b',
                            tertiaryColor: '#334155'
                        }} : {{
                            primaryColor: '#2563eb',
                            primaryTextColor: '#0f172a',
                            primaryBorderColor: '#64748b',
                            lineColor: '#64748b',
                            background: '#ffffff',
                            secondaryColor: '#f8fafc',
                            tertiaryColor: '#e2e8f0'
                        }}
                    }});
                    
                    mermaid.render(diagramId, graphDefinition, (svgCode) => {{
                        element.innerHTML = svgCode;
                    }});
                }});
            }}, 100);
        }});
    </script>
"#
        )
    }

    /// Create enhanced summary with component and diagram information
    fn create_enhanced_summary(
        &self,
        issues: &[ArchitecturalIssue],
        components: Option<&[ArchitecturalComponent]>,
        diagrams: &[DiagramMetadata],
    ) -> serde_json::Value {
        let mut severity_counts = HashMap::new();
        for issue in issues {
            *severity_counts.entry(&issue.severity).or_insert(0) += 1;
        }

        let component_summary = if let Some(comps) = components {
            let mut type_counts = HashMap::new();
            for comp in comps {
                *type_counts.entry(&comp.component_type).or_insert(0) += 1;
            }

            serde_json::json!({
                "total_components": comps.len(),
                "by_type": type_counts,
                "avg_dependencies": comps.iter()
                    .map(|c| c.dependencies.len())
                    .sum::<usize>() as f64 / comps.len().max(1) as f64
            })
        } else {
            serde_json::json!(null)
        };

        serde_json::json!({
            "total_issues": issues.len(),
            "by_severity": severity_counts,
            "components": component_summary,
            "diagrams_generated": diagrams.len(),
            "analysis_completeness": if components.is_some() { "enhanced" } else { "standard" }
        })
    }
}

/// Enhanced report data with architectural components and diagrams
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedReportData {
    /// Generated markdown content
    pub markdown_content: String,
    /// Generated diagram metadata
    pub diagrams: Vec<DiagramMetadata>,
    /// Number of architectural components analyzed
    pub components_analyzed: usize,
    /// Timestamp when the report was generated
    pub generation_timestamp: chrono::DateTime<chrono::Utc>,
}

/// Errors that can occur during report generation
#[derive(Debug, thiserror::Error)]
pub enum ReportGenerationError {
    #[error("Diagram generation failed: {0}")]
    DiagramGenerationError(#[from] MermaidGenerationError),

    #[error("Template processing failed: {0}")]
    TemplateError(String),

    #[error("Component analysis failed: {0}")]
    ComponentAnalysisError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}
