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

pub mod diagrams;
use chrono::{DateTime, Local};
use log::{error, info};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
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
    /// Diagram generation mode (hybrid rendering approach)
    diagram_mode: DiagramMode,
    /// Image renderer for generating images from Mermaid diagrams
    #[cfg(feature = "image-rendering")]
    image_renderer: Option<crate::report::ImageRenderer>,
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
            diagram_mode: DiagramMode::default(), // MermaidOnly by default for zero hosting costs
            #[cfg(feature = "image-rendering")]
            image_renderer: None,
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
                    log::warn!("Failed to create image renderer: {}", e);
                    // Fall back to text-only mode
                    self.diagram_mode = DiagramMode::TextOnly;
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
                        log::debug!("Rendering service is available");
                        true
                    }
                    Ok(Err(e)) => {
                        log::debug!("Rendering service health check failed: {}", e);
                        false
                    }
                    Err(_) => {
                        log::debug!("Rendering service health check timed out");
                        false
                    }
                }
            } else {
                log::debug!("No image renderer configured");
                false
            }
        }
        #[cfg(not(feature = "image-rendering"))]
        {
            log::debug!("Image rendering feature not enabled");
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
        report.push_str(&self.generate_executive_summary(analysis_run, issues));
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
    ) -> String {
        let total_issues = issues.len();
        let high_severity = issues.iter().filter(|i| i.severity == "high").count();
        let medium_severity = issues.iter().filter(|i| i.severity == "medium").count();
        let low_severity = issues.iter().filter(|i| i.severity == "low").count();

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
            "Unknown".to_string(), // TODO: Add codebase_path to AnalysisRun model
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
        let high_issues: Vec<_> = issues.iter().filter(|i| i.severity == "high").collect();
        let medium_issues: Vec<_> = issues.iter().filter(|i| i.severity == "medium").collect();
        let low_issues: Vec<_> = issues.iter().filter(|i| i.severity == "low").collect();

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
            let issues = issues_by_type.get(&type_id).unwrap();
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
                if self.include_code_snippets && issue.code_snippet.is_some() {
                    analysis.push_str("\n**Code Snippet**:\n\n");
                    analysis.push_str("```\n");
                    analysis.push_str(issue.code_snippet.as_ref().unwrap());
                    analysis.push_str("\n```\n\n");
                }

                // Add AI explanation if available and enabled
                if self.include_ai_explanations && issue.ai_explanation.is_some() {
                    analysis.push_str("**AI Analysis**:\n\n");

                    // Try to parse as JSON first (structured format)
                    if let Ok(json) =
                        serde_json::from_str::<Value>(issue.ai_explanation.as_ref().unwrap())
                    {
                        if let Some(title) = json.get("title").and_then(|v| v.as_str()) {
                            analysis.push_str(&format!("*{}*\n\n", title));
                        }

                        if let Some(explanation) = json.get("explanation").and_then(|v| v.as_str())
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
                        analysis
                            .push_str(&format!("{}\n\n", issue.ai_explanation.as_ref().unwrap()));
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
                                log::info!(
                                    "Successfully generated image for {} diagram",
                                    diagram_type
                                );
                                Ok(image_result)
                            }
                            Err(e) => {
                                log::warn!(
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
                        log::info!("Rendering service not available, using Mermaid-only mode for {} diagram", diagram_type);
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
                    log::info!("Image rendering feature not enabled, using Mermaid-only mode");
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

            let result = renderer.render_diagram(
                &request.mermaid_code, 
                request.format, 
                request.width.zip(request.height)
            ).await?;

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
                anti_pattern_types
                    .get(&i.anti_pattern_type_id)
                    .map(|apt| apt.name.contains("God Object") || apt.name.contains("god object"))
                    .unwrap_or(false)
            })
            .collect();

        if !god_object_issues.is_empty() {
            diagrams.push_str("### God Objects\n\n");

            for issue in god_object_issues {
                // Extract the god object name from the description
                let god_object_name = issue
                    .description
                    .split_whitespace()
                    .next()
                    .unwrap_or("Unknown");

                diagrams.push_str(&format!("#### {}\n\n", god_object_name));
                diagrams.push_str("```mermaid\nclassDiagram\n");
                diagrams.push_str(&format!("    class {} {{\n", god_object_name));

                // If we have a code snippet, try to extract methods
                if let Some(code) = &issue.code_snippet {
                    let lines: Vec<&str> = code.lines().collect();
                    for line in lines {
                        let trimmed = line.trim();
                        // Very simple heuristic for method declarations
                        if (trimmed.starts_with("fn ")
                            || trimmed.starts_with("pub fn ")
                            || trimmed.starts_with("def ")
                            || trimmed.starts_with("function "))
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

                            diagrams.push_str(&format!("        +{}\n", method_name));
                        }
                    }
                }

                diagrams.push_str("    }\n```\n\n");
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
    ) -> Result<String, String> {
        // Build the report structure
        let mut report = serde_json::Map::new();

        // Add metadata
        let mut metadata = serde_json::Map::new();
        metadata.insert(
            "codebasePath".to_string(),
            Value::String("Unknown".to_string()),
        ); // TODO: Add codebase_path to AnalysisRun
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
                json_issue.insert(
                    "antiPatternType".to_string(),
                    Value::String(anti_pattern.name.clone()),
                );
                json_issue.insert(
                    "antiPatternDescription".to_string(),
                    Value::String(anti_pattern.description.clone()),
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
    ) -> Result<EnhancedReportData, ReportGenerationError> {
        let mut report = String::new();
        let mut diagrams = Vec::new();

        // Generate standard report sections
        report.push_str(&self.generate_report_header(analysis_run));
        report.push_str(&self.generate_executive_summary(analysis_run, issues));

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
