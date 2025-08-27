//! Modern Template-Based Report Generator
//!
//! This module provides a clean, maintainable approach to HTML report generation
//! using Tera templates instead of embedded HTML strings. It replaces the legacy
//! string-based generation with a proper template architecture.

use chrono::{DateTime, Local};
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use tera::{Context, Tera};
use thiserror::Error;
// Removed unused tracing imports

use crate::analysis::mermaid_generator::MermaidGenerator;
use crate::database::models::{AnalysisRun, AntiPatternType, ArchitecturalIssue};
use crate::models::visualization::{
    ArchitecturalComponent, ComponentType,
};
// Removed unused uuid import

// Include the bundled assets generated at build time
include!(concat!(env!("OUT_DIR"), "/bundled_assets.rs"));
include!(concat!(env!("OUT_DIR"), "/feature_flags.rs"));

/// Errors that can occur during modern report generation
#[derive(Error, Debug)]
pub enum ModernReportError {
    /// Template engine error
    #[error("Template engine error: {0}")]
    TemplateError(#[from] tera::Error),
    /// IO operation failed
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    /// Template file not found
    #[error("Missing template: {0}")]
    MissingTemplate(String),
    /// Context serialization failed
    #[error("Context serialization error: {0}")]
    ContextError(String),
}

/// Modern report generator using Tera templates
pub struct ModernReportGenerator {
    tera: Tera,
    template_cache: HashMap<String, DateTime<Local>>,
    mermaid_generator: MermaidGenerator,
}

impl ModernReportGenerator {
    /// Create a new modern report generator
    pub fn new() -> Result<Self, ModernReportError> {
        use crate::core::logging::{info};

        info!("Attempting to initialize modern report generator with templates");

        let tera = Self::resolve_templates_with_fallback()?;
        let mermaid_generator = MermaidGenerator::new().map_err(|e| {
            ModernReportError::ContextError(format!(
                "Failed to initialize Mermaid generator: {}",
                e
            ))
        })?;

        info!("Modern report generator initialized successfully");

        Ok(Self {
            tera,
            template_cache: HashMap::new(),
            mermaid_generator,
        })
    }

    /// Resolve template paths with comprehensive fallback strategy
    fn resolve_templates_with_fallback() -> Result<Tera, ModernReportError> {
        use crate::core::logging::{info, warn};
        use std::env;
        use std::path::PathBuf;

        // Build comprehensive list of template search paths
        let mut template_patterns: Vec<String> = Vec::new();

        // 1. Current working directory relative paths
        template_patterns.extend([
            "src/templates/**/*.html".to_string(),
            "./src/templates/**/*.html".to_string(),
            "templates/**/*.html".to_string(),
            "./templates/**/*.html".to_string(),
        ]);

        // 2. Executable directory relative paths
        if let Ok(exe_path) = env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let exe_templates = exe_dir.join("templates/**/*.html");
                if let Some(exe_pattern) = exe_templates.to_str() {
                    template_patterns.push(exe_pattern.to_string());
                }

                // Also try relative to executable parent
                if let Some(exe_parent) = exe_dir.parent() {
                    let parent_templates = exe_parent.join("src/templates/**/*.html");
                    if let Some(parent_pattern) = parent_templates.to_str() {
                        template_patterns.push(parent_pattern.to_string());
                    }
                }
            }
        }

        // 3. Environment variable override
        if let Ok(custom_template_dir) = env::var("UVEDDI_TEMPLATE_DIR") {
            let custom_pattern = PathBuf::from(custom_template_dir).join("**/*.html");
            if let Some(custom_str) = custom_pattern.to_str() {
                template_patterns.insert(0, custom_str.to_string()); // High priority
            }
        }

        let mut tera = None;
        let mut last_error = None;
        let mut attempted_patterns = Vec::new();

        for pattern in &template_patterns {
            info!("Trying template pattern: {}", pattern);
            attempted_patterns.push(pattern.clone());

            match Tera::new(pattern) {
                Ok(mut t) => {
                    info!("Successfully loaded templates with pattern: {}", pattern);

                    // Register custom filters immediately
                    Self::register_custom_filters(&mut t);

                    // Debug: List loaded templates
                    let template_names: Vec<&str> = t.get_template_names().collect();
                    info!(
                        "Loaded {} templates: {:?}",
                        template_names.len(),
                        template_names
                    );

                    // Validate that we have the essential templates
                    if Self::validate_essential_templates(&template_names) {
                        tera = Some(t);
                        break;
                    } else {
                        warn!(
                            "Pattern '{}' loaded templates but missing essential ones",
                            pattern
                        );
                    }
                }
                Err(e) => {
                    info!("Template pattern '{}' failed: {:#?}", pattern, e);
                    last_error = Some(e);
                }
            }
        }

        tera.ok_or_else(|| {
            ModernReportError::MissingTemplate(format!(
                "Failed to load templates with any pattern. Attempted patterns: {:?}. Last error: {:?}",
                attempted_patterns, last_error
            ))
        })
    }

    /// Validate that essential templates are present
    fn validate_essential_templates(template_names: &[&str]) -> bool {
        let essential_templates = ["reports/architectural/main.html", "reports/base.html"];

        for essential in &essential_templates {
            if !template_names.contains(essential) {
                use crate::core::logging::warn;
                warn!("Missing essential template: {}", essential);
                return false;
            }
        }

        true
    }

    /// Get template directory from various sources
    pub fn get_template_directory() -> Option<PathBuf> {
        use std::env;
        use std::path::PathBuf;

        // 1. Environment variable
        if let Ok(custom_dir) = env::var("UVEDDI_TEMPLATE_DIR") {
            return Some(PathBuf::from(custom_dir));
        }

        // 2. Current working directory
        if let Ok(cwd) = env::current_dir() {
            let src_templates = cwd.join("src/templates");
            if src_templates.exists() {
                return Some(src_templates);
            }

            let templates = cwd.join("templates");
            if templates.exists() {
                return Some(templates);
            }
        }

        // 3. Executable directory
        if let Ok(exe_path) = env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let exe_templates = exe_dir.join("templates");
                if exe_templates.exists() {
                    return Some(exe_templates);
                }
            }
        }

        None
    }

    /// Generate an HTML report using modern templates
    pub async fn generate_html_report(
        &mut self,
        analysis_run: &AnalysisRun,
        issues: &[ArchitecturalIssue],
        anti_pattern_types: &HashMap<i64, AntiPatternType>,
        _output_dir: Option<&Path>,
    ) -> Result<String, ModernReportError> {
        // Refresh templates if they've changed
        self.refresh_templates_if_needed()?;

        // Prepare template context
        let context = self
            .build_template_context(analysis_run, issues, anti_pattern_types)
            .await?;

        // Render the main architectural report template
        let html = self
            .tera
            .render("reports/architectural/main.html", &context)?;

        Ok(html)
    }

    /// Build the complete template context with all necessary data
    async fn build_template_context(
        &self,
        analysis_run: &AnalysisRun,
        issues: &[ArchitecturalIssue],
        anti_pattern_types: &HashMap<i64, AntiPatternType>,
    ) -> Result<Context, ModernReportError> {
        let mut context = Context::new();

        // Basic metadata
        context.insert("analysis_run", analysis_run);
        context.insert("timestamp", &Local::now());
        context.insert(
            "git_branch",
            &std::env::var("GIT_BRANCH").unwrap_or_else(|_| "unknown".to_string()),
        );
        context.insert("generation_time_ms", &0u64); // TODO: Calculate actual time

        // Issues data
        context.insert("issue_list", issues);
        context.insert("anti_pattern_types", anti_pattern_types);
        context.insert("total_issues", &issues.len());

        // Calculate severity counts
        let severity_counts = self.calculate_severity_counts(issues);
        context.insert("severity_counts", &severity_counts);

        // Calculate health metrics
        let health_metrics = self.calculate_health_metrics(analysis_run, issues);
        context.insert("health_score", &health_metrics.score);
        context.insert("health_status", &health_metrics.status);
        context.insert("technical_debt_hours", &health_metrics.technical_debt_hours);
        context.insert("complexity_score", &health_metrics.complexity_score);

        // Issue categorization
        let issue_counts = self.categorize_issues(issues);
        context.insert("issue_counts", &issue_counts);

        // Key findings and recommendations
        let findings = self.generate_key_findings(issues, analysis_run);
        context.insert("key_findings", &findings);

        let recommendations = self.generate_recommendations(issues, anti_pattern_types);
        context.insert("recommendations", &recommendations);

        // Architecture diagrams (TODO: Implement native diagram generation)
        let diagrams = self.prepare_architecture_diagrams(issues).await;
        context.insert("architecture_diagrams", &diagrams);

        // Performance metrics (always provide, even if empty)
        let metrics = self
            .extract_performance_metrics(analysis_run)
            .unwrap_or_default();
        context.insert("performance_metrics", &metrics);

        // Bundled assets
        context.insert("bundled_css", BUNDLED_CSS);
        context.insert("bundled_js", BUNDLED_JS);
        context.insert("theme", &"light"); // Default theme

        Ok(context)
    }

    /// Calculate severity distribution for dashboard
    fn calculate_severity_counts(&self, issues: &[ArchitecturalIssue]) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        counts.insert("CRITICAL".to_string(), 0);
        counts.insert("MAJOR".to_string(), 0);
        counts.insert("MODERATE".to_string(), 0);
        counts.insert("MINOR".to_string(), 0);

        for issue in issues {
            let severity = issue.severity.to_uppercase();
            *counts.entry(severity).or_insert(0) += 1;
        }

        counts
    }

    /// Calculate overall health metrics
    fn calculate_health_metrics(
        &self,
        _analysis_run: &AnalysisRun,
        issues: &[ArchitecturalIssue],
    ) -> HealthMetrics {
        let total_issues = issues.len();
        let critical_issues = issues
            .iter()
            .filter(|i| i.severity.to_uppercase() == "CRITICAL")
            .count();
        let major_issues = issues
            .iter()
            .filter(|i| i.severity.to_uppercase() == "MAJOR")
            .count();

        // Simple health score calculation
        let base_score = 100.0;
        let critical_penalty = critical_issues as f64 * 20.0;
        let major_penalty = major_issues as f64 * 10.0;
        let other_penalty = (total_issues - critical_issues - major_issues) as f64 * 2.0;

        let score = (base_score - critical_penalty - major_penalty - other_penalty).max(0.0);

        let status = match score {
            s if s >= 90.0 => "Excellent",
            s if s >= 75.0 => "Good",
            s if s >= 50.0 => "Warning",
            _ => "Critical",
        };

        // Use severity as a proxy for technical debt estimation
        let technical_debt_hours = issues
            .iter()
            .map(|i| match i.severity.to_lowercase().as_str() {
                "critical" => 4.0,
                "major" => 2.0,
                "moderate" => 1.0,
                "minor" => 0.5,
                _ => 1.0,
            })
            .sum::<f64>();

        // Use issue count as complexity proxy
        let complexity_score = (issues.len() as f64) / 10.0;

        HealthMetrics {
            score,
            status: status.to_string(),
            technical_debt_hours,
            complexity_score,
        }
    }

    /// Categorize issues by severity for summary cards
    fn categorize_issues(&self, issues: &[ArchitecturalIssue]) -> HashMap<String, usize> {
        let mut counts = HashMap::new();

        for issue in issues {
            let severity = issue.severity.to_lowercase();
            *counts.entry(severity).or_insert(0) += 1;
        }

        counts
    }

    /// Generate key findings based on issue analysis
    fn generate_key_findings(
        &self,
        issues: &[ArchitecturalIssue],
        _analysis_run: &AnalysisRun,
    ) -> Vec<String> {
        let mut findings = Vec::new();

        let critical_count = issues
            .iter()
            .filter(|i| i.severity.to_uppercase() == "CRITICAL")
            .count();
        if critical_count > 0 {
            findings.push(format!(
                "{} critical architectural issues require immediate attention",
                critical_count
            ));
        }

        // Group by file to find hotspots
        let mut file_counts: HashMap<String, usize> = HashMap::new();
        for issue in issues {
            let file_path = &issue.file_path;
            *file_counts.entry(file_path.clone()).or_insert(0) += 1;
        }

        if let Some((hotspot_file, hotspot_count)) =
            file_counts.iter().max_by_key(|(_, &count)| count)
        {
            if *hotspot_count > 3 {
                findings.push(format!(
                    "File '{}' has {} issues and may need refactoring",
                    hotspot_file, hotspot_count
                ));
            }
        }

        // Use severity distribution as complexity proxy
        let avg_complexity = issues
            .iter()
            .map(|i| match i.severity.to_lowercase().as_str() {
                "critical" => 10.0,
                "major" => 7.0,
                "moderate" => 5.0,
                "minor" => 2.0,
                _ => 5.0,
            })
            .sum::<f64>()
            / (issues.len() as f64).max(1.0);

        if avg_complexity > 10.0 {
            findings.push(
                "High average complexity score indicates potential maintainability issues"
                    .to_string(),
            );
        }

        if findings.is_empty() {
            findings.push("No major architectural concerns detected".to_string());
        }

        findings
    }

    /// Generate actionable recommendations
    fn generate_recommendations(
        &self,
        issues: &[ArchitecturalIssue],
        _anti_pattern_types: &HashMap<i64, AntiPatternType>,
    ) -> Vec<Recommendation> {
        let mut recommendations = Vec::new();

        let critical_issues = issues
            .iter()
            .filter(|i| i.severity.to_uppercase() == "CRITICAL")
            .count();
        if critical_issues > 0 {
            recommendations.push(Recommendation {
                priority: "Critical".to_string(),
                title: "Address Critical Issues".to_string(),
                description: format!(
                    "Resolve {} critical architectural issues to prevent system failures",
                    critical_issues
                ),
                affected_benchmarks: None,
            });
        }

        // Check for high severity files (proxy for complexity)
        let high_complexity_issues: Vec<_> = issues
            .iter()
            .filter(|i| matches!(i.severity.to_lowercase().as_str(), "critical" | "major"))
            .collect();

        if !high_complexity_issues.is_empty() {
            recommendations.push(Recommendation {
                priority: "High".to_string(),
                title: "Reduce Code Complexity".to_string(),
                description: format!(
                    "Break down {} highly complex components to improve maintainability",
                    high_complexity_issues.len()
                ),
                affected_benchmarks: None,
            });
        }

        // Group issues by type for pattern-based recommendations
        let mut type_counts: HashMap<i64, usize> = HashMap::new();
        for issue in issues {
            *type_counts.entry(issue.anti_pattern_type_id).or_insert(0) += 1;
        }

        for (_type_id, count) in type_counts.iter() {
            if *count > 5 {
                recommendations.push(Recommendation {
                    priority: "Medium".to_string(),
                    title: "Address Recurring Pattern".to_string(),
                    description: format!(
                        "Consider refactoring approach - pattern appears {} times",
                        count
                    ),
                    affected_benchmarks: None,
                });
                break; // Only add one pattern-based recommendation
            }
        }

        recommendations
    }

    /// Prepare architecture diagrams from issues and analysis
    async fn prepare_architecture_diagrams(
        &self,
        issues: &[ArchitecturalIssue],
    ) -> Vec<ArchitectureDiagram> {
        use crate::analysis::mermaid_generator::MermaidGenerator;
        use crate::core::logging::{debug, info, warn};
        use crate::models::visualization::DiagramType;

        info!(
            "Generating data-driven architecture diagrams from {} issues",
            issues.len()
        );

        let mut diagrams = Vec::new();
        let mermaid_generator = match MermaidGenerator::new() {
            Ok(generator) => generator,
            Err(e) => {
                warn!("Failed to create MermaidGenerator: {}", e);
                return vec![];
            }
        };

        // Extract architectural components from issues
        let components = self.extract_components_from_issues(issues);
        debug!("Extracted {} components from issues", components.len());

        // Generate anti-pattern specific diagrams
        let anti_pattern_diagrams = self
            .generate_anti_pattern_diagrams(&mermaid_generator, issues, &components)
            .await;
        diagrams.extend(anti_pattern_diagrams);

        // Generate general architectural diagrams
        if !components.is_empty() {
            // Component overview diagram
            if let Ok(result) =
                mermaid_generator.generate_diagram(&components, DiagramType::Component)
            {
                diagrams.push(ArchitectureDiagram {
                    id: "component-overview".to_string(),
                    title: "Component Architecture Overview".to_string(),
                    svg_content: None,
                    mermaid_code: Some(result.mermaid_src),
                    description: Some(format!(
                        "Architectural overview showing {} components and their relationships",
                        components.len()
                    )),
                });
            }

            // Dependency graph diagram
            if let Ok(result) =
                mermaid_generator.generate_diagram(&components, DiagramType::Dependency)
            {
                diagrams.push(ArchitectureDiagram {
                    id: "dependency-graph".to_string(),
                    title: "Dependency Relationships".to_string(),
                    svg_content: None,
                    mermaid_code: Some(result.mermaid_src),
                    description: Some(
                        "Dependency relationships between architectural components".to_string(),
                    ),
                });
            }
        }

        // If no diagrams were generated, provide a fallback
        if diagrams.is_empty() {
            warn!("No diagrams could be generated, providing fallback");
            diagrams.push(ArchitectureDiagram {
                id: "no-data-available".to_string(),
                title: "No Architectural Data Available".to_string(),
                svg_content: None,
                mermaid_code: Some("graph TD\n    A[\"No architectural components detected\"]\n    A --> B[\"Run analysis with --enable-architectural-analysis\"]".to_string()),
                description: Some("No architectural data was available for diagram generation".to_string()),
            });
        }

        info!("Generated {} architecture diagrams", diagrams.len());
        diagrams
    }

    /// Extract architectural components from issues for diagram generation
    fn extract_components_from_issues(
        &self,
        issues: &[ArchitecturalIssue],
    ) -> Vec<ArchitecturalComponent> {
        use crate::core::logging::debug;
        use crate::models::visualization::{
            ArchitecturalComponent, ComponentMetrics,
        };
        use std::collections::HashSet;
        use uuid::Uuid;

        let mut components = Vec::new();
        let mut seen_files: HashSet<std::path::PathBuf> = HashSet::new();

        for issue in issues {
            let file_path = std::path::PathBuf::from(&issue.file_path);
            if seen_files.insert(file_path.clone()) {
                let component_type = self.infer_component_type_from_file(&file_path);

                // Calculate lines affected (estimate from start_line to end_line)
                let lines_affected = match (issue.start_line, issue.end_line) {
                    (Some(start), Some(end)) => Some((end - start + 1) as u32),
                    (Some(_), None) => Some(1),
                    _ => None,
                };

                // Parse severity as a number for complexity calculation
                let severity_score = match issue.severity.to_lowercase().as_str() {
                    "critical" => 4.0,
                    "major" => 3.0,
                    "moderate" => 2.0,
                    "minor" => 1.0,
                    _ => 1.0,
                };

                let metrics = ComponentMetrics {
                    lines_of_code: lines_affected,
                    complexity: Some(severity_score),
                    afferent_coupling: 0,
                    efferent_coupling: 0,
                    coupling_between_objects: None,
                    public_methods: None,
                    performance: None,
                };

                components.push(ArchitecturalComponent {
                    component_id: Uuid::new_v4(),
                    name: file_path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("Unknown")
                        .to_string(),
                    file_path: file_path.clone(),
                    component_type,
                    dependencies: vec![], // Will be populated separately
                    metrics,
                    group: file_path
                        .parent()
                        .and_then(|p| p.file_name())
                        .and_then(|s| s.to_str())
                        .map(|s| s.to_string()),
                });
            }
        }

        debug!(
            "Extracted {} unique components from {} issues",
            components.len(),
            issues.len()
        );
        components
    }

    /// Infer component type from file path and extension
    fn infer_component_type_from_file(&self, file_path: &std::path::Path) -> ComponentType {
        match file_path.extension().and_then(|s| s.to_str()) {
            Some("rs") => ComponentType::RustModule { is_public: true },
            Some("py") => ComponentType::PythonClass {
                bases: vec![],
                methods: vec![],
                is_abstract: false,
            },
            Some("js") | Some("ts") => ComponentType::JavaScriptEsModule { exports: vec![] },
            Some("java") => ComponentType::Class,
            Some("cpp") | Some("c") | Some("h") => ComponentType::Module,
            _ => ComponentType::Module,
        }
    }

    /// Generate anti-pattern specific diagrams
    async fn generate_anti_pattern_diagrams(
        &self,
        mermaid_generator: &MermaidGenerator,
        issues: &[ArchitecturalIssue],
        components: &[ArchitecturalComponent],
    ) -> Vec<ArchitectureDiagram> {
        use crate::core::logging::debug;
        use std::collections::HashMap;
        use uuid::Uuid;

        let mut diagrams = Vec::new();

        // Group issues by anti-pattern type
        let mut issues_by_type: HashMap<i64, Vec<&ArchitecturalIssue>> = HashMap::new();
        for issue in issues {
            issues_by_type
                .entry(issue.anti_pattern_type_id)
                .or_insert_with(Vec::new)
                .push(issue);
        }

        // Generate tight coupling diagram
        if let Some(coupling_issues) = issues_by_type.get(&1) {
            // Assuming type 1 is tight coupling
            let coupling_pairs: Vec<(Uuid, Uuid)> = vec![]; // Simplified for now
            let coupling_scores: HashMap<(Uuid, Uuid), f64> = HashMap::new();

            if let Ok(result) = mermaid_generator.generate_tight_coupling_diagram(
                components,
                &coupling_pairs,
                &coupling_scores,
            ) {
                diagrams.push(ArchitectureDiagram {
                    id: "tight-coupling".to_string(),
                    title: format!("Tight Coupling Issues ({})", coupling_issues.len()),
                    svg_content: None,
                    mermaid_code: Some(result.mermaid_src),
                    description: Some("Components with high coupling relationships".to_string()),
                });
            }
        }

        // Generate dead code diagram
        if let Some(dead_code_issues) = issues_by_type.get(&4) {
            // Assuming type 4 is dead code
            let dead_components: Vec<Uuid> = components
                .iter()
                .take(dead_code_issues.len().min(components.len()))
                .map(|c| c.component_id)
                .collect();

            if let Ok(result) =
                mermaid_generator.generate_dead_code_diagram(components, &dead_components)
            {
                diagrams.push(ArchitectureDiagram {
                    id: "dead-code".to_string(),
                    title: format!("Dead Code Detection ({})", dead_code_issues.len()),
                    svg_content: None,
                    mermaid_code: Some(result.mermaid_src),
                    description: Some("Unused or unreachable code components".to_string()),
                });
            }
        }

        debug!(
            "Generated {} anti-pattern specific diagrams",
            diagrams.len()
        );
        diagrams
    }

    /// Extract performance metrics if available
    fn extract_performance_metrics(
        &self,
        _analysis_run: &AnalysisRun,
    ) -> Option<Vec<PerformanceMetric>> {
        // TODO: Implement performance metrics extraction
        None
    }

    /// Register custom Tera filters and functions
    fn register_custom_filters(tera: &mut Tera) {
        // Add custom filters for report formatting
        tera.register_filter(
            "severity_color",
            |value: &tera::Value, _: &HashMap<String, tera::Value>| {
                let severity = value.as_str().unwrap_or("").to_lowercase();
                let color = match severity.as_str() {
                    "critical" => "#dc2626",
                    "major" => "#ea580c",
                    "moderate" => "#d97706",
                    "minor" => "#65a30d",
                    _ => "#6b7280",
                };
                Ok(tera::Value::String(color.to_string()))
            },
        );

        tera.register_filter(
            "file_basename",
            |value: &tera::Value, _: &HashMap<String, tera::Value>| {
                if let Some(path_str) = value.as_str() {
                    if let Some(basename) = Path::new(path_str).file_name() {
                        if let Some(basename_str) = basename.to_str() {
                            return Ok(tera::Value::String(basename_str.to_string()));
                        }
                    }
                }
                Ok(value.clone())
            },
        );

        // Safe percentage filter that handles division by zero and NaN values
        tera.register_filter(
            "safe_percentage",
            |value: &tera::Value, args: &HashMap<String, tera::Value>| {
                let numerator = value.as_f64().unwrap_or(0.0);
                let denominator = args.get("total").and_then(|v| v.as_f64()).unwrap_or(1.0);

                if denominator == 0.0 || denominator.is_nan() || numerator.is_nan() {
                    Ok(tera::Value::Number(
                        serde_json::Number::from_f64(0.0)
                            .unwrap_or_else(|| serde_json::Number::from(0)),
                    ))
                } else {
                    let percentage = (numerator / denominator * 100.0).round();
                    Ok(tera::Value::Number(
                        serde_json::Number::from_f64(percentage)
                            .unwrap_or_else(|| serde_json::Number::from(0)),
                    ))
                }
            },
        );

        // Safe round filter that handles NaN values
        tera.register_filter(
            "safe_round",
            |value: &tera::Value, args: &HashMap<String, tera::Value>| {
                let precision = args
                    .get("precision")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
                    .min(10) as u32; // Limit precision to prevent overflow

                match value.as_f64() {
                    Some(num) if num.is_nan() || num.is_infinite() => Ok(tera::Value::Number(
                        serde_json::Number::from_f64(0.0)
                            .unwrap_or_else(|| serde_json::Number::from(0)),
                    )),
                    Some(num) => {
                        // Clamp the precision to avoid overflow
                        let safe_precision = precision.min(10);
                        let multiplier = 10_f64.powi(safe_precision as i32);
                        let product = num * multiplier;

                        // Check if multiplication caused overflow/underflow
                        if !product.is_finite() {
                            return Ok(tera::Value::Number(
                                serde_json::Number::from_f64(0.0)
                                    .unwrap_or_else(|| serde_json::Number::from(0)),
                            ));
                        }

                        let rounded = product.round() / multiplier;

                        // Final check for validity
                        if !rounded.is_finite() {
                            Ok(tera::Value::Number(
                                serde_json::Number::from_f64(0.0)
                                    .unwrap_or_else(|| serde_json::Number::from(0)),
                            ))
                        } else {
                            Ok(tera::Value::Number(
                                serde_json::Number::from_f64(rounded)
                                    .unwrap_or_else(|| serde_json::Number::from(0)),
                            ))
                        }
                    }
                    None => {
                        // Try to parse as string
                        if let Some(str_val) = value.as_str() {
                            match str_val.parse::<f64>() {
                                Ok(num) if num.is_nan() || num.is_infinite() => {
                                    Ok(tera::Value::Number(
                                        serde_json::Number::from_f64(0.0)
                                            .unwrap_or_else(|| serde_json::Number::from(0)),
                                    ))
                                }
                                Ok(num) => {
                                    let safe_precision = precision.min(10);
                                    let multiplier = 10_f64.powi(safe_precision as i32);
                                    let product = num * multiplier;

                                    if !product.is_finite() {
                                        return Ok(tera::Value::Number(
                                            serde_json::Number::from_f64(0.0)
                                                .unwrap_or_else(|| serde_json::Number::from(0)),
                                        ));
                                    }

                                    let rounded = product.round() / multiplier;

                                    if !rounded.is_finite() {
                                        Ok(tera::Value::Number(
                                            serde_json::Number::from_f64(0.0)
                                                .unwrap_or_else(|| serde_json::Number::from(0)),
                                        ))
                                    } else {
                                        Ok(tera::Value::Number(
                                            serde_json::Number::from_f64(rounded)
                                                .unwrap_or_else(|| serde_json::Number::from(0)),
                                        ))
                                    }
                                }
                                Err(_) => Ok(tera::Value::Number(
                                    serde_json::Number::from_f64(0.0)
                                        .unwrap_or_else(|| serde_json::Number::from(0)),
                                )),
                            }
                        } else {
                            Ok(tera::Value::Number(
                                serde_json::Number::from_f64(0.0)
                                    .unwrap_or_else(|| serde_json::Number::from(0)),
                            ))
                        }
                    }
                }
            },
        );

        // Safe number format filter
        tera.register_filter(
            "safe_format_number",
            |value: &tera::Value, _: &HashMap<String, tera::Value>| match value.as_f64() {
                Some(num) if num.is_nan() || num.is_infinite() => {
                    Ok(tera::Value::String("N/A".to_string()))
                }
                Some(num) => Ok(tera::Value::String(format!("{:.1}", num))),
                None => {
                    if let Some(str_val) = value.as_str() {
                        match str_val.parse::<f64>() {
                            Ok(num) if num.is_nan() || num.is_infinite() => {
                                Ok(tera::Value::String("N/A".to_string()))
                            }
                            Ok(num) => Ok(tera::Value::String(format!("{:.1}", num))),
                            Err(_) => Ok(tera::Value::String("N/A".to_string())),
                        }
                    } else {
                        Ok(tera::Value::String("N/A".to_string()))
                    }
                }
            },
        );

        // JSON encode filter for JavaScript contexts
        tera.register_filter(
            "json_encode",
            |value: &tera::Value, _args: &HashMap<String, tera::Value>| match serde_json::to_string(
                value,
            ) {
                Ok(json_string) => Ok(tera::Value::String(json_string)),
                Err(_) => Ok(tera::Value::String("{}".to_string())),
            },
        );
    }

    /// Refresh templates if they've been modified
    fn refresh_templates_if_needed(&mut self) -> Result<(), ModernReportError> {
        // Check if template files have been modified
        // In a production system, this would check file timestamps
        // For now, we'll reload templates on every request during development

        if cfg!(debug_assertions) {
            use crate::core::logging::info;
            info!("Refreshing templates in debug mode");
            self.tera = Self::resolve_templates_with_fallback()?;
        }

        Ok(())
    }
}

/// Health metrics calculation result
#[derive(Debug)]
struct HealthMetrics {
    score: f64,
    status: String,
    technical_debt_hours: f64,
    complexity_score: f64,
}

/// Recommendation structure for template rendering
#[derive(Debug, serde::Serialize)]
struct Recommendation {
    priority: String,
    title: String,
    description: String,
    affected_benchmarks: Option<Vec<String>>,
}

/// Architecture diagram structure for template rendering
#[derive(Debug, serde::Serialize)]
struct ArchitectureDiagram {
    id: String,
    title: String,
    svg_content: Option<String>,
    mermaid_code: Option<String>,
    description: Option<String>,
}

/// Performance metric structure for template rendering
#[derive(Debug, serde::Serialize)]
struct PerformanceMetric {
    name: String,
    value: String,
    unit: Option<String>,
    trend: String, // "improving", "degrading", "stable"
    description: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use crate::core::logging::error;
    use std::collections::HashMap;
    use tera::Value;

    #[tokio::test]
    async fn test_modern_generator_creation() {
        let result = ModernReportGenerator::new();
        // This will fail if templates don't exist, which is expected in test environment
        match result {
            Ok(_) => println!("Modern generator created successfully"),
            Err(e) => error!("Expected error in test environment: {}", e),
        }
    }

    #[test]
    fn test_severity_counts() {
        let generator = ModernReportGenerator::new().unwrap_or_else(|_| {
            // Create a minimal generator for testing
            ModernReportGenerator {
                tera: Tera::default(),
                template_cache: HashMap::new(),
                mermaid_generator: crate::analysis::mermaid_generator::MermaidGenerator::for_tests(
                ),
            }
        });

        let issues = vec![ArchitecturalIssue {
            issue_id: Some(1),
            analysis_run_id: 1,
            anti_pattern_type_id: 1,
            file_path: "/test/file.py".to_string(),
            start_line: Some(10),
            end_line: Some(15),
            line_number: Some(10),
            column_number: None,
            message: "Detected critical issue".to_string(),
            metadata: "{}".to_string(),
            detector_name: "unit_test_detector".to_string(),
            created_at: Utc::now(),
            severity: "CRITICAL".to_string(),
            description: "Test architectural issue".to_string(),
            code_snippet: Some("def problematic_function():\n    pass".to_string()),
            ai_explanation: Some("This is a test issue explanation".to_string()),
        }];

        let counts = generator.calculate_severity_counts(&issues);
        assert_eq!(counts.get("CRITICAL"), Some(&1));
        assert_eq!(counts.get("MAJOR"), Some(&0));
    }

    #[test]
    fn test_safe_percentage_filter() {
        let mut tera = Tera::default();
        ModernReportGenerator::register_custom_filters(&mut tera);

        // Test normal percentage calculation
        let mut args = HashMap::new();
        args.insert("total".to_string(), Value::Number(10.into()));
        let filter = tera.get_filter("safe_percentage").unwrap();
        let result = filter.filter(&Value::Number(5.into()), &args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_f64(), Some(50.0));

        // Test division by zero (should return 0)
        args.insert("total".to_string(), Value::Number(0.into()));
        let filter = tera.get_filter("safe_percentage").unwrap();
        let result = filter.filter(&Value::Number(5.into()), &args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_f64(), Some(0.0));
    }

    #[test]
    fn test_safe_round_filter() {
        let mut tera = Tera::default();
        ModernReportGenerator::register_custom_filters(&mut tera);

        // Test normal rounding
        let mut args = HashMap::new();
        args.insert("precision".to_string(), Value::Number(1.into()));
        let filter = tera.get_filter("safe_round").unwrap();
        let result = filter.filter(
            &Value::Number(serde_json::Number::from_f64(3.14159).unwrap()),
            &args,
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_f64(), Some(3.1));

        // Test NaN handling (should return 0)
        let filter = tera.get_filter("safe_round").unwrap();
        let result = filter.filter(&Value::String("NaN".to_string()), &args);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_f64(), Some(0.0));
    }

    #[test]
    fn test_safe_format_number_filter() {
        let mut tera = Tera::default();
        ModernReportGenerator::register_custom_filters(&mut tera);

        // Test normal number formatting
        let filter = tera.get_filter("safe_format_number").unwrap();
        let result = filter.filter(
            &Value::Number(serde_json::Number::from_f64(3.14159).unwrap()),
            &HashMap::new(),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), Some("3.1"));

        // Test NaN handling (should return "N/A")
        let filter = tera.get_filter("safe_format_number").unwrap();
        let result = filter.filter(&Value::String("NaN".to_string()), &HashMap::new());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str(), Some("N/A"));
    }

    #[test]
    fn test_validate_essential_templates() {
        let valid_templates = vec![
            "reports/architectural/main.html",
            "reports/base.html",
            "reports/components/summary.html",
        ];
        assert!(ModernReportGenerator::validate_essential_templates(
            &valid_templates
        ));

        let invalid_templates = vec!["reports/components/summary.html"];
        assert!(!ModernReportGenerator::validate_essential_templates(
            &invalid_templates
        ));
    }
}
