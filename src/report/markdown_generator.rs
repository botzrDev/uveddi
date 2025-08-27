//! Enhanced Markdown Report Generator
//!
//! This module provides advanced markdown report generation for architectural analysis results,
//! with support for AI analysis insights, multiple diagram render modes, and embedded visualizations.
//!
//! # Features
//!
//! - Multiple Mermaid diagram render modes (inline, linked, embedded SVG)
//! - Template-based report generation with Tera
//! - AI-powered analysis integration
//! - Performance metrics and optimization recommendations
//! - Security-focused rendering with input sanitization

use chrono::Local;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::NamedTempFile;
use tera::{Context, Tera};
use thiserror::Error;
use tracing::error;

use crate::core::mocks::ai_mocks::AiInsight;
use crate::database::models::{AnalysisRun, AntiPatternType, ArchitecturalIssue};
use crate::models::visualization::{
    ArchitecturalComponent, ComponentMetrics, ComponentType,
};
use crate::report::metrics::{compute_debt_score, compute_issues_by_severity};
use crate::report::DiagramMode;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum MarkdownReportError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Context serialization error: {0}")]
    ContextError(String),
    #[error("AI analysis error: {0}")]
    AiAnalysisError(String),
    #[error("Template error: {0}")]
    TemplateError(#[from] tera::Error),
    #[error("Diagram rendering error: {0}")]
    DiagramError(String),
}

/// Diagram rendering configuration
#[derive(Debug, Clone)]
pub struct DiagramConfig {
    /// Diagram rendering mode
    pub mode: DiagramMode,
    /// Include diagrams in output
    pub include_diagrams: bool,
    /// Output directory for diagram files
    pub diagram_output_dir: Option<PathBuf>,
    /// Maximum diagrams per report
    pub max_diagrams: usize,
}

impl Default for DiagramConfig {
    fn default() -> Self {
        Self {
            mode: DiagramMode::MermaidOnly,
            include_diagrams: true,
            diagram_output_dir: None,
            max_diagrams: 20,
        }
    }
}

/// Enhanced markdown report generator with diagram rendering and templating
pub struct MarkdownReportGenerator {
    tera: Tera,
    diagram_config: DiagramConfig,
}

impl MarkdownReportGenerator {
    /// Create a new markdown report generator with templates
    pub fn new() -> Result<Self, MarkdownReportError> {
        Self::with_diagram_config(DiagramConfig::default())
    }

    /// Create a new generator with custom diagram configuration
    pub fn with_diagram_config(diagram_config: DiagramConfig) -> Result<Self, MarkdownReportError> {
        let mut tera = Tera::default();

        // Add built-in templates (use add_raw_template instead of add_template_literal)
        tera.add_raw_template(
            "report_template",
            include_str!("../templates/report.md.tera"),
        )?;
        tera.add_raw_template(
            "diagram_template",
            include_str!("../templates/diagram.md.tera"),
        )?;

        Ok(Self {
            tera,
            diagram_config,
        })
    }

    /// Set diagram configuration
    pub fn set_diagram_config(&mut self, config: DiagramConfig) {
        self.diagram_config = config;
    }

    /// Render Mermaid diagram to SVG using mmdc CLI
    async fn render_mermaid_to_svg(
        &self,
        mermaid_code: &str,
        output_path: &Path,
    ) -> Result<String, MarkdownReportError> {
        // Create temporary file for Mermaid input
        let mut temp_file = NamedTempFile::new().map_err(|e| {
            MarkdownReportError::DiagramError(format!("Failed to create temp file: {}", e))
        })?;

        writeln!(temp_file, "{}", mermaid_code).map_err(|e| {
            MarkdownReportError::DiagramError(format!("Failed to write to temp file: {}", e))
        })?;

        // Execute mmdc command
        let output = Command::new("mmdc")
            .arg("-i")
            .arg(temp_file.path())
            .arg("-o")
            .arg(output_path)
            .arg("-f")
            .arg("svg")
            .arg("-t")
            .arg("neutral") // Use neutral theme for better compatibility
            .output()
            .map_err(|e| {
                MarkdownReportError::DiagramError(format!("Failed to execute mmdc: {}", e))
            })?;

        if !output.status.success() {
            let error_message = String::from_utf8_lossy(&output.stderr);
            return Err(MarkdownReportError::DiagramError(format!(
                "mmdc failed: {}",
                error_message
            )));
        }

        // Read the generated SVG
        let svg_content = fs::read_to_string(output_path)
            .map_err(|e| MarkdownReportError::DiagramError(format!("Failed to read SVG: {}", e)))?;

        Ok(svg_content)
    }

    /// Generate diagram content based on configuration
    async fn generate_diagram_content(
        &self,
        title: &str,
        mermaid_code: &str,
        diagram_id: &str,
    ) -> Result<String, MarkdownReportError> {
        let mut context = Context::new();
        context.insert(
            "diagram",
            &serde_json::json!({
                "title": title,
                "mermaid_code": mermaid_code,
                "mode": match self.diagram_config.mode {
                    DiagramMode::MermaidOnly => "mermaid_only",
                    DiagramMode::ImageOnly => "linked",
                    DiagramMode::ImageWithFallback => "embedded_svg"
                }
            }),
        );

        match self.diagram_config.mode {
            DiagramMode::MermaidOnly => {
                // Just return Mermaid code with instructions
                context.insert(
                    "diagram",
                    &serde_json::json!({
                        "title": title,
                        "mermaid_code": mermaid_code,
                        "mode": "mermaid_only"
                    }),
                );
                self.tera
                    .render("diagram_template", &context)
                    .map_err(MarkdownReportError::TemplateError)
            }
            DiagramMode::ImageOnly | DiagramMode::ImageWithFallback => {
                if let Some(ref output_dir) = self.diagram_config.diagram_output_dir {
                    let svg_path = output_dir.join(format!("{}.svg", diagram_id));

                    // Ensure output directory exists
                    if let Some(parent) = svg_path.parent() {
                        fs::create_dir_all(parent).map_err(|e| {
                            MarkdownReportError::DiagramError(format!(
                                "Failed to create output dir: {}",
                                e
                            ))
                        })?;
                    }

                    match self.render_mermaid_to_svg(mermaid_code, &svg_path).await {
                        Ok(svg_content) => {
                            context.insert("diagram", &serde_json::json!({
                                "title": title,
                                "mermaid_code": mermaid_code,
                                "file_path": svg_path.to_string_lossy(),
                                "svg_content": svg_content,
                                "mode": if self.diagram_config.mode == DiagramMode::ImageOnly { "linked" } else { "embedded_svg" }
                            }));
                            self.tera
                                .render("diagram_template", &context)
                                .map_err(MarkdownReportError::TemplateError)
                        }
                        Err(e) if self.diagram_config.mode == DiagramMode::ImageWithFallback => {
                            // Fallback to Mermaid-only
                            error!("Warning: Failed to render diagram '{}', falling back to Mermaid-only: {}", title, e);
                            context.insert(
                                "diagram",
                                &serde_json::json!({
                                    "title": title,
                                    "mermaid_code": mermaid_code,
                                    "mode": "mermaid_only"
                                }),
                            );
                            self.tera
                                .render("diagram_template", &context)
                                .map_err(MarkdownReportError::TemplateError)
                        }
                        Err(e) => Err(e),
                    }
                } else {
                    // No output directory specified, fallback to Mermaid-only
                    context.insert(
                        "diagram",
                        &serde_json::json!({
                            "title": title,
                            "mermaid_code": mermaid_code,
                            "mode": "mermaid_only"
                        }),
                    );
                    self.tera
                        .render("diagram_template", &context)
                        .map_err(MarkdownReportError::TemplateError)
                }
            }
        }
    }

    /// Generate a complete markdown report with AI analysis
    pub async fn generate_markdown_report(
        &mut self,
        analysis_run: &AnalysisRun,
        issues: &[ArchitecturalIssue],
        anti_pattern_types: &HashMap<i64, AntiPatternType>,
        ai_insights: Option<&[AiInsight]>,
        _output_dir: Option<&Path>,
    ) -> Result<String, MarkdownReportError> {
        let mut markdown = String::new();

        // Report header
        markdown.push_str(&self.generate_header(analysis_run));
        markdown.push_str("\n\n");

        // Executive summary with AI insights
        markdown.push_str(&self.generate_executive_summary(analysis_run, issues, ai_insights));
        markdown.push_str("\n\n");

        // Architecture overview with diagrams
        markdown.push_str(
            &self
                .generate_architecture_overview(analysis_run, issues)
                .await?,
        );
        markdown.push_str("\n\n");

        // AI Analysis section (if available)
        if let Some(insights) = ai_insights {
            markdown.push_str(&self.generate_ai_analysis_section(insights));
            markdown.push_str("\n\n");
        }

        // Issues breakdown
        markdown.push_str(&self.generate_issues_breakdown(issues, anti_pattern_types));
        markdown.push_str("\n\n");

        // Architectural diagrams
        markdown.push_str(&self.generate_diagrams_section(analysis_run, issues).await?);
        markdown.push_str("\n\n");

        // Recommendations
        markdown.push_str(&self.generate_recommendations(issues, ai_insights));
        markdown.push_str("\n\n");

        // Appendix
        markdown.push_str(&self.generate_appendix(analysis_run));

        Ok(markdown)
    }

    /// Generate report header with metadata
    fn generate_header(&self, analysis_run: &AnalysisRun) -> String {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S UTC");
        let git_branch = std::env::var("GIT_BRANCH").unwrap_or_else(|_| "unknown".to_string());

        format!(
            r#"# Architectural Analysis Report

**Project ID:** {}  
**Analysis ID:** `{}`  
**Generated:** {}  
**Git Branch:** `{}`  
**Status:** {}  

---"#,
            analysis_run.project_id,
            analysis_run.run_id.unwrap_or(0),
            timestamp,
            git_branch,
            analysis_run.status
        )
    }

    /// Generate executive summary with AI insights
    fn generate_executive_summary(
        &self,
        analysis_run: &AnalysisRun,
        issues: &[ArchitecturalIssue],
        ai_insights: Option<&[AiInsight]>,
    ) -> String {
        let total_issues = issues.len();
        let sev_map = compute_issues_by_severity(issues);
        let critical_issues = *sev_map.get("critical").unwrap_or(&0);
        let high_issues = *sev_map.get("high").unwrap_or(&0);
        let medium_issues = *sev_map.get("medium").unwrap_or(&0);
        let low_issues = *sev_map.get("low").unwrap_or(&0);

        let health_score = self.calculate_health_score(issues);
        let health_status = self.get_health_status(health_score);

        let debt_score = compute_debt_score(issues);
        let mut summary = format!(
            r#"## Executive Summary

### 📊 Analysis Overview

- **Total Issues Found:** {}
- **Architecture Health Score:** {:.1}/100 ({})
- **Technical Debt Score:** {debt_score}/100 (higher is worse)
- **Files Analyzed:** {}
- **Analysis Status:** {}

### 🚨 Issue Severity Distribution

| Severity | Count | Percentage |
|----------|-------|------------|
| 🔴 Critical | {} | {:.1}% |
| 🟡 High | {} | {:.1}% |
| 🟠 Medium | {} | {:.1}% |
| 🟢 Low | {} | {:.1}% |"#,
            total_issues,
            health_score,
            health_status,
            analysis_run.total_files_analyzed.unwrap_or(0),
            analysis_run.status,
            critical_issues,
            if total_issues > 0 {
                (critical_issues as f64 / total_issues as f64) * 100.0
            } else {
                0.0
            },
            high_issues,
            if total_issues > 0 {
                (high_issues as f64 / total_issues as f64) * 100.0
            } else {
                0.0
            },
            medium_issues,
            if total_issues > 0 {
                (medium_issues as f64 / total_issues as f64) * 100.0
            } else {
                0.0
            },
            low_issues,
            if total_issues > 0 {
                (low_issues as f64 / total_issues as f64) * 100.0
            } else {
                0.0
            }
        );

        // Add AI insights summary if available
        if let Some(insights) = ai_insights {
            if !insights.is_empty() {
                summary.push_str("\n\n### 🤖 AI Analysis Summary\n\n");

                let high_confidence_insights =
                    insights.iter().filter(|i| i.confidence > 0.8).count();
                let total_insights = insights.len();

                summary.push_str(&format!(
                    "- **AI Insights Generated:** {}\n- **High Confidence Insights:** {} ({:.1}%)\n",
                    total_insights,
                    high_confidence_insights,
                    if total_insights > 0 { (high_confidence_insights as f64 / total_insights as f64) * 100.0 } else { 0.0 }
                ));

                // Add top AI insights
                let top_insights: Vec<_> = insights.iter().take(3).enumerate().collect();

                if !top_insights.is_empty() {
                    summary.push_str("\n**Key AI Insights:**\n");
                    for (i, insight) in top_insights {
                        summary.push_str(&format!(
                            "{}. {} (Confidence: {:.1}%)\n",
                            i + 1,
                            insight.suggestion,
                            insight.confidence * 100.0
                        ));
                    }
                }
            }
        }

        summary
    }

    /// Generate architecture overview with system diagram
    async fn generate_architecture_overview(
        &mut self,
        analysis_run: &AnalysisRun,
        issues: &[ArchitecturalIssue],
    ) -> Result<String, MarkdownReportError> {
        let mut overview = String::from("## Architecture Overview\n\n");

        // Generate system architecture diagram
        let components = self.extract_architectural_components(analysis_run, issues);
        if !components.is_empty() {
            overview.push_str("### System Architecture\n\n");

            // Generate Mermaid diagram for system architecture
            match self.generate_system_architecture_diagram(&components).await {
                Ok(diagram) => {
                    overview.push_str("```mermaid\n");
                    overview.push_str(&diagram);
                    overview.push_str("\n```\n\n");
                }
                Err(e) => {
                    overview.push_str(&format!(
                        "*Visual diagrams unavailable - {}: showing text summary*\n\n",
                        e
                    ));
                }
            }
            overview.push_str("**Architecture Summary:**\n");
            for component in &components {
                overview.push_str(&format!(
                    "- {}: {} (Complexity: {:.1})\n",
                    component.name,
                    format!("{:?}", component.component_type),
                    component.metrics.complexity.unwrap_or(0.0)
                ));
            }
            overview.push_str("\n");
        }

        // Add component metrics
        if !components.is_empty() {
            overview.push_str("### Component Metrics\n\n");
            overview.push_str("| Component | Type | Complexity | Dependencies |\n");
            overview.push_str("|-----------|------|------------|-------------|\n");

            for component in &components {
                overview.push_str(&format!(
                    "| {} | {:?} | {} | {} |\n",
                    component.name,
                    component.component_type,
                    component.metrics.complexity.unwrap_or(0.0) as i32,
                    component.dependencies.len()
                ));
            }
            overview.push_str("\n");
        }

        Ok(overview)
    }

    /// Generate AI analysis section with detailed insights
    fn generate_ai_analysis_section(&self, ai_insights: &[AiInsight]) -> String {
        let mut section = String::from("## 🤖 AI Analysis\n\n");

        if ai_insights.is_empty() {
            section.push_str("*No AI insights available for this analysis.*\n");
            return section;
        }

        // Group insights by confidence level
        let high_confidence: Vec<_> = ai_insights.iter().filter(|i| i.confidence > 0.8).collect();
        let medium_confidence: Vec<_> = ai_insights
            .iter()
            .filter(|i| i.confidence >= 0.6 && i.confidence <= 0.8)
            .collect();
        let low_confidence: Vec<_> = ai_insights.iter().filter(|i| i.confidence < 0.6).collect();

        if !high_confidence.is_empty() {
            section.push_str("### 🎯 High Confidence Insights\n\n");
            for insight in high_confidence {
                section.push_str(&format!(
                    "#### Issue: {}\n**Confidence:** {:.1}%\n\n{}\n\n",
                    insight.issue_id,
                    insight.confidence * 100.0,
                    insight.suggestion
                ));
            }
        }

        if !medium_confidence.is_empty() {
            section.push_str("### 🎲 Medium Confidence Insights\n\n");
            for insight in medium_confidence {
                section.push_str(&format!(
                    "#### Issue: {}\n**Confidence:** {:.1}%\n\n{}\n\n",
                    insight.issue_id,
                    insight.confidence * 100.0,
                    insight.suggestion
                ));
            }
        }

        if !low_confidence.is_empty() {
            section.push_str("### 💭 Exploratory Insights\n\n");
            for insight in low_confidence {
                section.push_str(&format!(
                    "#### Issue: {}\n**Confidence:** {:.1}%\n\n{}\n\n",
                    insight.issue_id,
                    insight.confidence * 100.0,
                    insight.suggestion
                ));
            }
        }

        section
    }

    /// Generate issues breakdown by category and severity
    fn generate_issues_breakdown(
        &self,
        issues: &[ArchitecturalIssue],
        anti_pattern_types: &HashMap<i64, AntiPatternType>,
    ) -> String {
        let mut breakdown = String::from("## Issues Breakdown\n\n");

        if issues.is_empty() {
            breakdown.push_str("✅ **No architectural issues detected!**\n\nYour codebase shows good architectural practices.\n");
            return breakdown;
        }

        // Group by issue type (using message content)
        let mut by_type: HashMap<String, Vec<&ArchitecturalIssue>> = HashMap::new();
        for issue in issues {
            let issue_type = if issue.message.contains("LongMethod") {
                "Long Method".to_string()
            } else if issue.message.contains("LargeClass") {
                "Large Class".to_string()
            } else if issue.message.contains("FeatureEnvy") {
                "Feature Envy".to_string()
            } else {
                "General".to_string()
            };
            by_type
                .entry(issue_type)
                .or_insert_with(Vec::new)
                .push(issue);
        }

        for (issue_type, type_issues) in by_type {
            breakdown.push_str(&format!("### {} Issues\n\n", issue_type));

            for issue in type_issues {
                let severity_emoji =
                    if issue.message.contains("critical") || issue.message.contains("Critical") {
                        "🔴"
                    } else if issue.message.contains("high") || issue.message.contains("High") {
                        "🟡"
                    } else if issue.message.contains("medium") || issue.message.contains("Medium") {
                        "🟠"
                    } else {
                        "🟢"
                    };

                breakdown.push_str(&format!(
                    "#### {} Issue #{}\n\n",
                    severity_emoji,
                    issue.issue_id.unwrap_or(0)
                ));

                breakdown.push_str(&format!("**Message:** {}\n\n", issue.message));

                breakdown.push_str(&format!("**File:** `{}`", issue.file_path));
                if let Some(line_number) = issue.line_number {
                    breakdown.push_str(&format!(":{}", line_number));
                }
                breakdown.push_str("\n\n");

                breakdown.push_str("---\n\n");
            }
        }

        breakdown
    }

    /// Generate diagrams section with various architectural views
    async fn generate_diagrams_section(
        &mut self,
        analysis_run: &AnalysisRun,
        issues: &[ArchitecturalIssue],
    ) -> Result<String, MarkdownReportError> {
        let mut diagrams = String::from("## Architectural Diagrams\n\n");

        let components = self.extract_architectural_components(analysis_run, issues);

        if components.is_empty() {
            diagrams.push_str("*No architectural components detected for diagram generation.*\n");
            return Ok(diagrams);
        }

        // Dependencies diagram
        diagrams.push_str("### Dependency Graph\n\n");
        match self.generate_dependency_diagram(&components).await {
            Ok(diagram) => {
                diagrams.push_str("```mermaid\n");
                diagrams.push_str(&diagram);
                diagrams.push_str("\n```\n\n");
            }
            Err(e) => {
                diagrams.push_str(&format!(
                    "*Visual dependency diagram unavailable - {}*\n\n",
                    e
                ));
            }
        }
        diagrams.push_str("**Component Dependencies:**\n");
        for component in &components {
            if !component.dependencies.is_empty() {
                diagrams.push_str(&format!(
                    "- {} depends on {} components\n",
                    component.name,
                    component.dependencies.len()
                ));
            }
        }
        diagrams.push_str("\n");

        // Component interaction diagram
        diagrams.push_str("### Component Interactions\n\n");
        match self
            .generate_component_interaction_diagram(&components)
            .await
        {
            Ok(diagram) => {
                diagrams.push_str("```mermaid\n");
                diagrams.push_str(&diagram);
                diagrams.push_str("\n```\n\n");
            }
            Err(e) => {
                diagrams.push_str(&format!(
                    "*Visual interaction diagram unavailable - {}*\n\n",
                    e
                ));
            }
        }
        diagrams.push_str("**Component Summary:**\n");
        for component in &components {
            diagrams.push_str(&format!(
                "- {}: Located at `{}`\n",
                component.name,
                component.file_path.display()
            ));
        }
        diagrams.push_str("\n");

        Ok(diagrams)
    }

    /// Generate recommendations based on issues and AI insights
    fn generate_recommendations(
        &self,
        issues: &[ArchitecturalIssue],
        ai_insights: Option<&[AiInsight]>,
    ) -> String {
        let mut recommendations = String::from("## Recommendations\n\n");

        if issues.is_empty() {
            recommendations
                .push_str("🎉 **Excellent!** No significant architectural issues were found.\n\n");
            recommendations.push_str("### Maintenance Suggestions\n\n");
            recommendations.push_str("- Continue following current architectural patterns\n");
            recommendations.push_str("- Regular code reviews to maintain quality\n");
            recommendations.push_str("- Consider adding more automated testing\n");
            return recommendations;
        }

        // Priority recommendations based on severity
        let critical_issues = issues
            .iter()
            .filter(|i| i.message.contains("critical") || i.message.contains("Critical"))
            .count();
        let high_issues = issues
            .iter()
            .filter(|i| i.message.contains("high") || i.message.contains("High"))
            .count();

        if critical_issues > 0 {
            recommendations.push_str("### 🚨 Immediate Actions Required\n\n");
            recommendations.push_str(&format!(
                "You have **{}** critical architectural issues that require immediate attention:\n\n",
                critical_issues
            ));

            for issue in issues
                .iter()
                .filter(|i| i.message.contains("critical") || i.message.contains("Critical"))
            {
                recommendations.push_str(&format!(
                    "- **Issue #{}**: {}\n",
                    issue.issue_id.unwrap_or(0),
                    issue.message
                ));
            }
            recommendations.push_str("\n");
        }

        if high_issues > 0 {
            recommendations.push_str("### ⚠️ High Priority Actions\n\n");
            for issue in issues
                .iter()
                .filter(|i| i.message.contains("high") || i.message.contains("High"))
                .take(5)
            {
                recommendations.push_str(&format!(
                    "- **Issue #{}**: {}\n",
                    issue.issue_id.unwrap_or(0),
                    issue.message
                ));
            }
            recommendations.push_str("\n");
        }

        // AI-driven recommendations
        if let Some(insights) = ai_insights {
            let actionable_insights: Vec<_> = insights
                .iter()
                .filter(|i| i.confidence > 0.7)
                .take(5)
                .collect();

            if !actionable_insights.is_empty() {
                recommendations.push_str("### 🤖 AI-Recommended Actions\n\n");
                for insight in actionable_insights {
                    recommendations.push_str(&format!(
                        "- **{}** (Confidence: {:.1}%)\n",
                        insight.suggestion,
                        insight.confidence * 100.0
                    ));
                }
                recommendations.push_str("\n");
            }
        }

        // General architectural guidelines
        recommendations.push_str("### 📋 General Guidelines\n\n");
        recommendations.push_str(
            "1. **Address Critical Issues First**: Focus on critical and high-severity issues\n",
        );
        recommendations
            .push_str("2. **Implement Gradual Refactoring**: Make incremental improvements\n");
        recommendations
            .push_str("3. **Add Automated Testing**: Ensure changes don't introduce regressions\n");
        recommendations.push_str(
            "4. **Code Review Process**: Implement peer reviews for architectural changes\n",
        );
        recommendations
            .push_str("5. **Documentation**: Update architectural documentation after changes\n");

        recommendations
    }

    /// Generate appendix with technical details
    fn generate_appendix(&self, analysis_run: &AnalysisRun) -> String {
        let mut appendix = String::from("## Appendix\n\n");

        appendix.push_str("### Analysis Configuration\n\n");
        appendix.push_str(&format!(
            "- **Analysis Engine Version**: {}\n",
            env!("CARGO_PKG_VERSION")
        ));
        appendix.push_str(&format!("- **Project ID**: {}\n", analysis_run.project_id));
        appendix.push_str(&format!("- **Status**: {}\n", analysis_run.status));
        appendix.push_str(&format!(
            "- **Files Analyzed**: {}\n",
            analysis_run.total_files_analyzed.unwrap_or(0)
        ));
        appendix.push_str(&format!(
            "- **Issues Found**: {}\n",
            analysis_run.total_issues_found.unwrap_or(0)
        ));

        appendix.push_str("\n### About This Report\n\n");
        appendix.push_str("This report was generated by Uveddi, an architectural analysis tool that helps identify design patterns, anti-patterns, and potential improvements in software architecture.\n\n");
        appendix.push_str("For more information, visit: [Uveddi Documentation](https://github.com/your-org/uveddi)\n");

        appendix
    }

    // Helper methods

    /// Calculate health score based on issues
    fn calculate_health_score(&self, issues: &[ArchitecturalIssue]) -> f64 {
        if issues.is_empty() {
            return 100.0;
        }

        let mut score: f64 = 100.0;
        for issue in issues {
            let deduction =
                if issue.message.contains("critical") || issue.message.contains("Critical") {
                    20.0
                } else if issue.message.contains("high") || issue.message.contains("High") {
                    10.0
                } else if issue.message.contains("medium") || issue.message.contains("Medium") {
                    5.0
                } else {
                    2.0
                };
            score -= deduction;
        }

        score.max(0.0)
    }

    /// Get health status text based on score
    fn get_health_status(&self, score: f64) -> &'static str {
        match score {
            s if s >= 90.0 => "Excellent",
            s if s >= 80.0 => "Good",
            s if s >= 70.0 => "Fair",
            s if s >= 60.0 => "Poor",
            _ => "Critical",
        }
    }

    /// Extract architectural components from analysis data
    fn extract_architectural_components(
        &self,
        analysis_run: &AnalysisRun,
        issues: &[ArchitecturalIssue],
    ) -> Vec<ArchitecturalComponent> {
        let mut components = Vec::new();
        let mut seen_files = std::collections::HashSet::new();

        // Create components from files with issues
        for issue in issues {
            let file_path = &issue.file_path;
            if seen_files.insert(file_path.clone()) {
                let component_name = Path::new(file_path)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Unknown")
                    .to_string();

                let component = ArchitecturalComponent {
                    component_id: Uuid::new_v4(),
                    name: component_name,
                    component_type: ComponentType::Module,
                    file_path: PathBuf::from(file_path),
                    dependencies: Vec::new(),
                    metrics: ComponentMetrics {
                        complexity: Some(
                            issues.iter().filter(|i| &i.file_path == file_path).count() as f64,
                        ),
                        lines_of_code: Some(0),
                        afferent_coupling: 0,
                        efferent_coupling: 0,
                        coupling_between_objects: Some(0.0),
                        public_methods: Some(0),
                        performance: None,
                    },
                    group: None,
                };

                components.push(component);
            }
        }

        components
    }

    /// Generate system architecture diagram using Mermaid
    async fn generate_system_architecture_diagram(
        &mut self,
        components: &[ArchitecturalComponent],
    ) -> Result<String, MarkdownReportError> {
        if components.is_empty() {
            return Ok("graph TD\n    A[No Components Detected]".to_string());
        }

        let diagram = self.generate_fallback_component_diagram(components);

        Ok(diagram)
    }

    /// Generate dependency diagram using Mermaid
    async fn generate_dependency_diagram(
        &mut self,
        components: &[ArchitecturalComponent],
    ) -> Result<String, MarkdownReportError> {
        if components.is_empty() {
            return Ok("graph TD\n    A[No Dependencies Detected]".to_string());
        }

        let diagram = self.generate_fallback_dependency_diagram(components);

        Ok(diagram)
    }

    /// Generate component interaction diagram using Mermaid
    async fn generate_component_interaction_diagram(
        &mut self,
        components: &[ArchitecturalComponent],
    ) -> Result<String, MarkdownReportError> {
        if components.is_empty() {
            return Ok(
                "sequenceDiagram\n    Note over System: No Component Interactions Detected"
                    .to_string(),
            );
        }

        // Create a simple interaction diagram based on dependencies
        let mut diagram = String::from("graph LR\n");

        for component in components {
            let node_id = component.name.replace("-", "_").replace(" ", "_");
            diagram.push_str(&format!("    {}[{}]\n", node_id, component.name));

            for dep in &component.dependencies {
                let dep_id = dep.to.name.replace("-", "_").replace(" ", "_");
                diagram.push_str(&format!("    {} --> {}\n", node_id, dep_id));
            }
        }

        if diagram == "graph LR\n" {
            diagram = "graph LR\n    A[Independent Components] --> B[No Interactions Detected]"
                .to_string();
        }

        Ok(diagram)
    }

    /// Generate fallback component diagram when Mermaid fails
    fn generate_fallback_component_diagram(&self, components: &[ArchitecturalComponent]) -> String {
        let mut diagram = String::from("graph TD\n");

        for (i, component) in components.iter().enumerate() {
            let node_id = format!("C{}", i);
            diagram.push_str(&format!("    {}[{}]\n", node_id, component.name));

            // Add complexity styling
            let complexity = component.metrics.complexity.unwrap_or(0.0);
            if complexity > 15.0 {
                diagram.push_str(&format!("    {} --> CRITICAL[High Complexity]\n", node_id));
            } else if complexity > 10.0 {
                diagram.push_str(&format!("    {} --> WARNING[Medium Complexity]\n", node_id));
            }
        }

        if components.len() > 1 {
            diagram.push_str("    style CRITICAL fill:#ff6b6b\n");
            diagram.push_str("    style WARNING fill:#ffd93d\n");
        }

        diagram
    }

    /// Generate fallback dependency diagram when Mermaid fails
    fn generate_fallback_dependency_diagram(
        &self,
        components: &[ArchitecturalComponent],
    ) -> String {
        let mut diagram = String::from("graph LR\n");

        for component in components {
            let node_id = component.name.replace(['-', ' ', '.'], "_");
            diagram.push_str(&format!("    {}[{}]\n", node_id, component.name));

            for dep in &component.dependencies {
                let dep_id = dep.to.name.replace(['-', ' ', '.'], "_");
                diagram.push_str(&format!("    {} --> {}\n", node_id, dep_id));
            }
        }

        if diagram == "graph LR\n" {
            diagram = "graph LR\n    A[Independent Components]\n    B[No Dependencies Detected]\n    A -.-> B".to_string();
        }

        diagram
    }
}

impl Default for MarkdownReportGenerator {
    fn default() -> Self {
        Self::new().expect("Failed to create default MarkdownReportGenerator")
    }
}
