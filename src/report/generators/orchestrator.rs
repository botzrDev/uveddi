//! Report Generation Orchestrator
//!
//! This module contains the main ReportGenerator struct and its core orchestration logic.

use crate::analysis::mermaid_generator::{MermaidGenerationError, MermaidGenerator};
use crate::database::models::{AnalysisRun, AntiPatternType, ArchitecturalIssue};
use crate::models::visualization::{ArchitecturalComponent, DiagramMetadata, DiagramType};
use crate::report::modern_generator;
use crate::report::svg_generator::SvgGenerator;
use crate::report::DiagramMode;
use chrono;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::time::Duration;
use tracing::{error, info, warn};

/// Main report generator orchestrator
pub struct ReportGenerator {
    /// Whether to include AI-generated explanations in reports
    pub include_ai_explanations: bool,
    /// Whether to include code snippets in issue descriptions
    pub include_code_snippets: bool,
    /// Whether to generate and include architectural diagrams
    pub include_diagrams: bool,
    pub include_severity_summary: bool,
    pub include_remediation_steps: bool,
    /// Mermaid generator for creating diagrams
    pub mermaid_generator: Option<MermaidGenerator>,
    /// SVG generator for static diagram rendering
    pub svg_generator: Option<SvgGenerator>,
    /// Diagram generation mode (hybrid rendering approach)
    pub diagram_mode: DiagramMode,
    /// Image renderer for generating images from Mermaid diagrams
    #[cfg(feature = "image-rendering")]
    pub image_renderer: Option<crate::report::ImageRenderer>,
    /// Modern template-based generator (feature flagged)
    pub modern_generator: Option<modern_generator::ModernReportGenerator>,
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

    /// Generate HTML report - minimal delegation implementation
    pub async fn generate_html_report(
        &mut self,
        analysis_run: &AnalysisRun,
        issues: &[ArchitecturalIssue],
        anti_pattern_types: &HashMap<i64, AntiPatternType>,
        output_path: Option<&Path>,
        _codebase_path: Option<&str>,
    ) -> Result<String, String> {
        // Simple delegation to the html_generator module methods
        match self.generate_html_head() {
            html_head => {
                let report = format!(
                    "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n{}\n</head>\n<body>\n{}\n</body>\n</html>",
                    html_head,
                    self.generate_html_header(analysis_run)
                );

                // Write to file if output path is provided
                if let Some(path) = output_path {
                    match fs::File::create(path) {
                        Ok(mut file) => {
                            if let Err(e) = file.write_all(report.as_bytes()) {
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

                Ok(report)
            }
        }
    }

    /// Generate JSON report - minimal implementation
    pub fn generate_json_report(
        &self,
        analysis_run: &AnalysisRun,
        issues: &[ArchitecturalIssue],
        anti_pattern_types: &HashMap<i64, AntiPatternType>,
        output_path: Option<&Path>,
    ) -> Result<String, String> {
        // Simple JSON serialization of key data
        let report_data = serde_json::json!({
            "analysis_run": analysis_run,
            "issues": issues,
            "anti_pattern_types": anti_pattern_types,
            "metadata": {
                "generated_at": chrono::Utc::now().to_rfc3339(),
                "total_issues": issues.len()
            }
        });

        let json_string = serde_json::to_string_pretty(&report_data)
            .map_err(|e| format!("Failed to serialize JSON: {}", e))?;

        // Write to file if output path is provided
        if let Some(path) = output_path {
            match fs::File::create(path) {
                Ok(mut file) => {
                    if let Err(e) = file.write_all(json_string.as_bytes()) {
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

        Ok(json_string)
    }
}
