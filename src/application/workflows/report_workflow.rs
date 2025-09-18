//! Report generation workflow for creating analysis reports
//!
//! This module provides workflow coordination for generating reports
//! in multiple formats with support for templating, custom styling,
//! and output file management.

use crate::application::configuration::OutputConfig;
use crate::application::orchestrator::AnalysisResult;
use crate::core::logging::{debug, error, info, warn};
use crate::core::mocks::ai_mocks::AiInsight;
use crate::database::models::{AnalysisRun, ArchitecturalIssue, AntiPatternType};
use crate::error::UveddiError;
use crate::report::{markdown_generator::MarkdownReportGenerator, ReportGenerator};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

use super::traits::{Workflow, WorkflowStatus};

/// Report generation workflow coordinator
pub struct ReportWorkflow {
    /// Report generator instance
    report_generator: ReportGenerator,
    /// Markdown report generator
    markdown_generator: Option<MarkdownReportGenerator>,
    /// Workflow configuration
    config: ReportWorkflowConfig,
    /// Current workflow status
    status: WorkflowStatus,
}

/// Configuration for report workflow
#[derive(Debug, Clone)]
pub struct ReportWorkflowConfig {
    /// Enable template processing
    pub enable_template_processing: bool,
    /// Enable syntax highlighting for code snippets
    pub enable_syntax_highlighting: bool,
    /// Maximum file size for generated reports (in bytes)
    pub max_report_size: usize,
    /// Enable report compression
    pub enable_compression: bool,
    /// Custom CSS for HTML reports
    pub custom_css: Option<String>,
    /// Include detailed metrics in reports
    pub include_detailed_metrics: bool,
    /// Enable interactive features for HTML reports
    pub enable_interactive_features: bool,
}

/// Input for report workflow
#[derive(Debug)]
pub struct ReportWorkflowInput {
    /// Analysis results to generate report from
    pub analysis_result: AnalysisResult,
    /// Output configuration
    pub output_config: OutputConfig,
    /// Anti-pattern type definitions
    pub anti_pattern_types: HashMap<i64, AntiPatternType>,
    /// Optional AI insights
    pub ai_insights: Option<Vec<AiInsight>>,
    /// Optional codebase path for context
    pub codebase_path: Option<PathBuf>,
}

/// Output from report workflow
#[derive(Debug)]
pub struct ReportWorkflowOutput {
    /// Generated report content
    pub content: String,
    /// Output file path (if written to file)
    pub output_path: Option<PathBuf>,
    /// Report generation metrics
    pub metrics: ReportMetrics,
    /// Report metadata
    pub metadata: ReportMetadata,
}

/// Metrics collected during report generation
#[derive(Debug, Clone)]
pub struct ReportMetrics {
    /// Total generation time
    pub generation_duration: std::time::Duration,
    /// Report size in bytes
    pub report_size: usize,
    /// Number of issues included
    pub issues_included: usize,
    /// Number of code snippets included
    pub code_snippets_included: usize,
    /// Template processing time
    pub template_processing_duration: Option<std::time::Duration>,
}

/// Metadata about the generated report
#[derive(Debug, Clone)]
pub struct ReportMetadata {
    /// Report format
    pub format: String,
    /// Generation timestamp
    pub generated_at: chrono::DateTime<chrono::Utc>,
    /// Report version
    pub version: String,
    /// Whether AI insights were included
    pub includes_ai_insights: bool,
    /// Number of files analyzed
    pub files_analyzed: usize,
    /// Analysis duration
    pub analysis_duration: std::time::Duration,
}

impl ReportWorkflow {
    /// Create a new report workflow
    pub fn new() -> Result<Self, UveddiError> {
        let report_generator = ReportGenerator::new();

        Ok(Self {
            report_generator,
            markdown_generator: None,
            config: ReportWorkflowConfig::default(),
            status: WorkflowStatus::Ready,
        })
    }

    /// Create a new report workflow with custom configuration
    pub fn with_config(config: ReportWorkflowConfig) -> Result<Self, UveddiError> {
        let mut workflow = Self::new()?;
        workflow.config = config;
        Ok(workflow)
    }

    /// Generate JSON format report
    async fn generate_json_report(
        &mut self,
        input: &ReportWorkflowInput,
    ) -> Result<String, UveddiError> {
        debug!("Generating JSON report");

        let report = self
            .report_generator
            .generate_json_report(
                &input.analysis_result.analysis_run,
                &input.analysis_result.issues,
                &input.anti_pattern_types,
                input.codebase_path.as_deref(),
            )
            .map_err(|e| {
                error!("JSON report generation failed: {}", e);
                UveddiError::from(crate::report::errors::ReportGenerationError::DataExtractionError(
                    e.to_string(),
                ))
            })?;

        Ok(report.to_string())
    }

    /// Generate Markdown format report
    async fn generate_markdown_report(
        &mut self,
        input: &ReportWorkflowInput,
    ) -> Result<String, UveddiError> {
        debug!("Generating Markdown report");

        // Initialize markdown generator if not already done
        if self.markdown_generator.is_none() {
            self.markdown_generator = Some(
                MarkdownReportGenerator::new().map_err(|e| {
                    error!("Failed to initialize Markdown generator: {}", e);
                    UveddiError::from(crate::report::errors::ReportGenerationError::DataExtractionError(
                        e.to_string(),
                    ))
                })?,
            );
        }

        let markdown_generator = self.markdown_generator.as_mut().unwrap();

        let report = markdown_generator
            .generate_markdown_report(
                &input.analysis_result.analysis_run,
                &input.analysis_result.issues,
                &input.anti_pattern_types,
                input.ai_insights.as_deref(),
                input.codebase_path.as_deref(),
            )
            .await
            .map_err(|e| {
                error!("Markdown report generation failed: {}", e);
                UveddiError::from(crate::report::errors::ReportGenerationError::DataExtractionError(
                    e.to_string(),
                ))
            })?;

        Ok(report)
    }

    /// Generate HTML format report
    async fn generate_html_report(
        &mut self,
        input: &ReportWorkflowInput,
    ) -> Result<String, UveddiError> {
        debug!("Generating HTML report");

        let report = self
            .report_generator
            .generate_html_report(
                &input.analysis_result.analysis_run,
                &input.analysis_result.issues,
                &input.anti_pattern_types,
                input.output_config.file_path.as_deref(),
                input.codebase_path.as_deref().and_then(|p| p.to_str()),
            )
            .await
            .map_err(|e| {
                error!("HTML report generation failed: {}", e);
                UveddiError::from(crate::report::errors::ReportGenerationError::DataExtractionError(
                    e.to_string(),
                ))
            })?;

        Ok(report)
    }

    /// Apply custom styling and templates
    async fn apply_customizations(
        &self,
        content: String,
        format: &str,
    ) -> Result<String, UveddiError> {
        if !self.config.enable_template_processing {
            return Ok(content);
        }

        debug!("Applying customizations for {} format", format);

        // Apply custom CSS for HTML reports
        if format == "html" && self.config.custom_css.is_some() {
            // TODO: Implement CSS injection
            // This would involve parsing the HTML and injecting custom CSS
        }

        // Apply syntax highlighting if enabled
        if self.config.enable_syntax_highlighting {
            // TODO: Implement syntax highlighting
            // This would involve parsing code snippets and adding highlighting
        }

        Ok(content)
    }

    /// Write report to file
    async fn write_report_to_file(
        &self,
        content: &str,
        output_path: &PathBuf,
    ) -> Result<(), UveddiError> {
        debug!("Writing report to: {}", output_path.display());

        // Ensure parent directory exists
        if let Some(parent) = output_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| UveddiError::PathError {
                    path: parent.display().to_string(),
                    reason: format!("Failed to create parent directory: {}", e),
                    suggestion: "Check directory permissions".to_string(),
                })?;
            }
        }

        // Write content to file
        std::fs::write(output_path, content).map_err(|e| UveddiError::PathError {
            path: output_path.display().to_string(),
            reason: format!("Failed to write report file: {}", e),
            suggestion: "Check file permissions and disk space".to_string(),
        })?;

        info!("Report written to: {}", output_path.display());
        Ok(())
    }

    /// Validate generated report
    async fn validate_report(
        &self,
        content: &str,
        format: &str,
    ) -> Result<Vec<String>, UveddiError> {
        debug!("Validating {} report", format);

        let mut warnings = Vec::new();

        // Check report size
        if content.len() > self.config.max_report_size {
            warnings.push(format!(
                "Report size ({} bytes) exceeds maximum ({})",
                content.len(),
                self.config.max_report_size
            ));
        }

        // Check for empty content
        if content.trim().is_empty() {
            warnings.push("Generated report is empty".to_string());
        }

        // Format-specific validation
        match format {
            "json" => {
                // Validate JSON structure
                if let Err(e) = serde_json::from_str::<serde_json::Value>(content) {
                    warnings.push(format!("Invalid JSON structure: {}", e));
                }
            }
            "html" => {
                // Basic HTML validation
                if !content.contains("<html") || !content.contains("</html>") {
                    warnings.push("HTML report missing required structure".to_string());
                }
            }
            "markdown" => {
                // Basic Markdown validation
                if !content.contains('#') && !content.contains("##") {
                    warnings.push("Markdown report missing headers".to_string());
                }
            }
            _ => {}
        }

        if !warnings.is_empty() {
            warn!("Report validation completed with {} warnings", warnings.len());
            for warning in &warnings {
                warn!("Report validation warning: {}", warning);
            }
        }

        Ok(warnings)
    }

    /// Calculate report metrics
    fn calculate_metrics(
        &self,
        content: &str,
        generation_duration: std::time::Duration,
        input: &ReportWorkflowInput,
    ) -> ReportMetrics {
        ReportMetrics {
            generation_duration,
            report_size: content.len(),
            issues_included: input.analysis_result.issues.len(),
            code_snippets_included: 0, // TODO: Count code snippets in content
            template_processing_duration: None, // TODO: Track template processing time
        }
    }

    /// Create report metadata
    fn create_metadata(
        &self,
        format: &str,
        input: &ReportWorkflowInput,
    ) -> ReportMetadata {
        ReportMetadata {
            format: format.to_string(),
            generated_at: chrono::Utc::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            includes_ai_insights: input.ai_insights.is_some(),
            files_analyzed: input.analysis_result.metadata.files_analyzed,
            analysis_duration: input.analysis_result.metadata.analysis_duration,
        }
    }
}

impl Workflow<ReportWorkflowInput, ReportWorkflowOutput> for ReportWorkflow {
    async fn execute(&mut self, input: ReportWorkflowInput) -> Result<ReportWorkflowOutput, UveddiError> {
        let start_time = Instant::now();
        info!("Starting report generation workflow for format: {}", input.output_config.format);

        self.status = WorkflowStatus::Running;

        // Generate report content based on format
        let content = match input.output_config.format.as_str() {
            "json" => self.generate_json_report(&input).await?,
            "markdown" => self.generate_markdown_report(&input).await?,
            "html" => self.generate_html_report(&input).await?,
            _ => {
                return Err(UveddiError::config_error(
                    &format!("Unsupported output format: {}", input.output_config.format),
                    "report generation",
                ));
            }
        };

        // Apply customizations
        let content = self
            .apply_customizations(content, &input.output_config.format)
            .await?;

        // Validate report
        let validation_warnings = self
            .validate_report(&content, &input.output_config.format)
            .await?;

        // Write to file if output path is specified
        let output_path = if let Some(ref path) = input.output_config.file_path {
            self.write_report_to_file(&content, path).await?;
            Some(path.clone())
        } else {
            None
        };

        // Calculate metrics and create metadata
        let generation_duration = start_time.elapsed();
        let metrics = self.calculate_metrics(&content, generation_duration, &input);
        let metadata = self.create_metadata(&input.output_config.format, &input);

        self.status = WorkflowStatus::Completed;

        info!(
            "Report generation completed in {:?} - {} bytes",
            generation_duration,
            content.len()
        );

        if !validation_warnings.is_empty() {
            warn!("Report generated with {} warnings", validation_warnings.len());
        }

        Ok(ReportWorkflowOutput {
            content,
            output_path,
            metrics,
            metadata,
        })
    }

    fn name(&self) -> &str {
        "report_workflow"
    }

    fn can_handle(&self, input: &ReportWorkflowInput) -> bool {
        !input.analysis_result.issues.is_empty()
            && matches!(input.output_config.format.as_str(), "json" | "markdown" | "html")
    }

    fn status(&self) -> WorkflowStatus {
        self.status.clone()
    }
}

impl Default for ReportWorkflowConfig {
    fn default() -> Self {
        Self {
            enable_template_processing: true,
            enable_syntax_highlighting: true,
            max_report_size: 50 * 1024 * 1024, // 50MB
            enable_compression: false,
            custom_css: None,
            include_detailed_metrics: true,
            enable_interactive_features: true,
        }
    }
}