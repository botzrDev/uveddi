//! Reporting module for Uveddi
//!
//! This module provides report generation utilities, including markdown and JSON output for analysis results.
//! It supports ERD-compliant reporting, AI explanations, code snippets, and diagram integration.
//!
//
//! # Main Features
//!
//! - **Markdown Report**: Comprehensive reports in markdown format, including executive summary, issues by severity, and detailed issue analysis.
//! - **JSON Report**: Structured JSON output for integration with other tools or for automated processing.
//! - **AI Integration**: Optional AI-generated explanations and code snippets for deeper insights.
//! - **Diagram Support**: Integration of architecture diagrams using Mermaid.js for visual representation of issues.
//! - **Customizable**: Options to include/exclude AI explanations, code snippets, and diagrams in the reports.
//! - **Performance**: Efficient processing and reporting, capable of handling large analysis results.

use crate::analysis::graph::ComponentNode;
use crate::database::models::{AnalysisRun, AntiPatternType, ArchitecturalIssue};
use serde_json::Value;
use std::collections::HashMap;
use chrono::{Local, DateTime};
use std::fs;
use std::io::Write;
use std::path::Path;
use log::{info, error, warn};

/// Report generator following ERD specifications (ER-F-011 to ER-F-014)
pub struct ReportGenerator {
    include_ai_explanations: bool,
    include_code_snippets: bool,
    include_diagrams: bool,
    include_severity_summary: bool,
    include_remediation_steps: bool,
}

impl Default for ReportGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl ReportGenerator {
    pub fn new() -> Self {
        Self {
            include_ai_explanations: true,
            include_code_snippets: true,
            include_diagrams: true, 
            include_severity_summary: true,
            include_remediation_steps: true,
        }
    }
    
    /// Configure whether AI explanations should be included in the report
    pub fn with_ai_explanations(mut self, include: bool) -> Self {
        self.include_ai_explanations = include;
        self
    }
    
    /// Configure whether code snippets should be included in the report
    pub fn with_code_snippets(mut self, include: bool) -> Self {
        self.include_code_snippets = include;
        self
    }
    
    /// Configure whether diagrams should be included in the report
    pub fn with_diagrams(mut self, include: bool) -> Self {
        self.include_diagrams = include;
        self
    }
    
    /// Configure whether severity summary should be included in the report
    pub fn with_severity_summary(mut self, include: bool) -> Self {
        self.include_severity_summary = include;
        self
    }
    
    /// Configure whether remediation steps should be included in the report
    pub fn with_remediation_steps(mut self, include: bool) -> Self {
        self.include_remediation_steps = include;
        self
    }
    
    /// Generate a Markdown report for the given analysis run and issues
    pub fn generate_markdown_report(
        &self,
        analysis_run: &AnalysisRun,
        issues: &[ArchitecturalIssue],
        anti_pattern_types: &HashMap<i64, AntiPatternType>,
        output_path: Option<&Path>,
    ) -> Result<String, String> {
        info!("Generating markdown report with {} issues", issues.len());
        
        let now: DateTime<Local> = Local::now();
        let report_title = format!("# Uveddi Architectural Analysis Report\n\n_Generated on {}_\n\n", now.format("%Y-%m-%d %H:%M:%S"));
        
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
    
    /// Generate the executive summary section
    fn generate_executive_summary(&self, analysis_run: &AnalysisRun, issues: &[ArchitecturalIssue]) -> String {
        let total_issues = issues.len();
        let high_severity = issues.iter().filter(|i| i.severity == "high").count();
        let medium_severity = issues.iter().filter(|i| i.severity == "medium").count();
        let low_severity = issues.iter().filter(|i| i.severity == "low").count();
        
        let unique_files = issues.iter().map(|i| &i.file_path).collect::<std::collections::HashSet<_>>().len();
        
        format!(
            "This report analyzes the codebase at `{}` and identified **{} architectural issues** across **{} files**.\n\n\
            - **High Severity**: {} issues\n\
            - **Medium Severity**: {} issues\n\
            - **Low Severity**: {} issues\n\n\
            The analysis took {:.2} seconds to complete.",
            analysis_run.codebase_path,
            total_issues,
            unique_files,
            high_severity,
            medium_severity,
            low_severity,
            analysis_run.duration_seconds.unwrap_or(0.0)
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
                    if let Ok(json) = serde_json::from_str::<Value>(issue.ai_explanation.as_ref().unwrap()) {
                        if let Some(title) = json.get("title").and_then(|v| v.as_str()) {
                            analysis.push_str(&format!("*{}*\n\n", title));
                        }
                        
                        if let Some(explanation) = json.get("explanation").and_then(|v| v.as_str()) {
                            analysis.push_str(&format!("{}\n\n", explanation));
                        }
                        
                        if self.include_remediation_steps {
                            if let Some(refactoring) = json.get("refactoring").and_then(|v| v.as_str()) {
                                analysis.push_str("**Recommended Refactoring**:\n\n");
                                analysis.push_str(&format!("{}\n\n", refactoring));
                            }
                        }
                    } else {
                        // Fall back to raw text if not valid JSON
                        analysis.push_str(&format!("{}\n\n", issue.ai_explanation.as_ref().unwrap()));
                    }
                }
                
                analysis.push_str("\n");
            }
        }
        
        analysis
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
                        dependencies.insert((parts[i].trim().to_string(), parts[i + 1].trim().to_string()));
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
                        if (trimmed.starts_with("fn ") || trimmed.starts_with("pub fn ") || 
                            trimmed.starts_with("def ") || trimmed.starts_with("function ")) && 
                           trimmed.contains("(") {
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
    
    /// Generate a JSON report
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
        metadata.insert("codebasePath".to_string(), Value::String(analysis_run.codebase_path.clone()));
        metadata.insert("timestamp".to_string(), Value::String(Local::now().to_rfc3339()));
        metadata.insert("durationSeconds".to_string(), Value::Number(analysis_run.duration_seconds.unwrap_or(0.0).into()));
        report.insert("metadata".to_string(), Value::Object(metadata));
        
        // Add issues
        let mut json_issues = Vec::new();
        for issue in issues {
            let mut json_issue = serde_json::Map::new();
            
            json_issue.insert("description".to_string(), Value::String(issue.description.clone()));
            json_issue.insert("filePath".to_string(), Value::String(issue.file_path.clone()));
            json_issue.insert("severity".to_string(), Value::String(issue.severity.clone()));
            
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
                    json_issue.insert("aiExplanation".to_string(), Value::String(explanation.clone()));
                }
            }
            
            // Add anti-pattern type information
            if let Some(anti_pattern) = anti_pattern_types.get(&issue.anti_pattern_type_id) {
                json_issue.insert("antiPatternType".to_string(), Value::String(anti_pattern.name.clone()));
                json_issue.insert("antiPatternDescription".to_string(), Value::String(anti_pattern.description.clone()));
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
}
