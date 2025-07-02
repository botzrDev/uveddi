use crate::database::models::{AnalysisRun, ArchitecturalIssue, AntiPatternType};
use crate::analysis::dependency_graph::ComponentNode;
use serde_json::Value;
use std::collections::HashMap;

/// Report generator following ERD specifications (ER-F-011 to ER-F-014)
pub struct ReportGenerator {
    include_ai_explanations: bool,
    include_code_snippets: bool,
    include_diagrams: bool,
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
            include_diagrams: true, // Enabled by default
        }
    }

    /// Generate comprehensive markdown report (ER-F-014)
    pub fn generate_markdown_report(
        &self,
        analysis_run: &AnalysisRun,
        issues: &[ArchitecturalIssue],
    ) -> Result<String, crate::error::UveddiError> {
        let mut report = String::new();

        // Report header with analysis summary
        report.push_str(&self.generate_header(analysis_run, issues));

        // Executive summary
        report.push_str(&self.generate_summary(issues));

        // Issues by severity
        report.push_str(&self.generate_issues_by_severity(issues));

        // Detailed issue analysis
        report.push_str(&self.generate_detailed_issues(issues));

        Ok(report)
    }

    /// Generate structured JSON output
    pub fn generate_json_report(
        &self,
        analysis_run: &AnalysisRun,
        issues: &[ArchitecturalIssue],
    ) -> Result<Value, crate::error::UveddiError> {
        let report = serde_json::json!({
            "analysis_run": {
                "run_id": analysis_run.run_id,
                "start_time": analysis_run.start_time,
                "end_time": analysis_run.end_time,
                "status": analysis_run.status,
                "total_files_analyzed": analysis_run.total_files_analyzed,
                "total_issues_found": analysis_run.total_issues_found
            },
            "summary": {
                "total_issues": issues.len(),
                "by_severity": self.calculate_severity_breakdown(issues),
                "by_category": self.calculate_category_breakdown(issues)
            },
            "issues": issues
        });

        Ok(report)
    }

    fn generate_header(&self, analysis_run: &AnalysisRun, issues: &[ArchitecturalIssue]) -> String {
        format!(
            r"# Uveddi Analysis Report

**Analysis ID:** {}
**Start Time:** {}
**Duration:** {}
**Files Analyzed:** {}
**Issues Found:** {}

---

",
            analysis_run.run_id.unwrap_or(0),
            analysis_run.start_time.format("%Y-%m-%d %H:%M:%S UTC"),
            self.calculate_duration(analysis_run),
            analysis_run.total_files_analyzed.unwrap_or(0),
            issues.len()
        )
    }

    fn generate_summary(&self, issues: &[ArchitecturalIssue]) -> String {
        let mut summary = String::from("## Executive Summary\n\n");
        let severity_breakdown = self.calculate_severity_breakdown(issues);
        let total_issues = issues.len();

        summary.push_str(&format!("Total Architectural Issues Found: {}\n\n", total_issues));
        summary.push_str("### Issues by Severity:\n");
        for (severity, count) in &severity_breakdown {
            summary.push_str(&format!("- {}: {}\n", severity, count));
        }
        summary.push('\n');
        summary
    }

    /// Helper: Lookup anti-pattern type by id
    fn lookup_antipattern_type(&self, type_id: i64) -> AntiPatternType {
        // In a real system, this would query a DB or config. Here, hardcode a few for demo.
        match type_id {
            1 => AntiPatternType {
                anti_pattern_type_id: Some(1),
                name: "God Object".to_string(),
                description: "A class that does too much".to_string(),
                category: "OO".to_string(),
            },
            2 => AntiPatternType {
                anti_pattern_type_id: Some(2),
                name: "Unstable Interface".to_string(),
                description: "Interface changes too often".to_string(),
                category: "OO".to_string(),
            },
            3 => AntiPatternType {
                anti_pattern_type_id: Some(3),
                name: "Modularity Violation".to_string(),
                description: "Module breaks encapsulation".to_string(),
                category: "Modularity".to_string(),
            },
            _ => AntiPatternType {
                anti_pattern_type_id: Some(type_id),
                name: "Unknown".to_string(),
                description: "Unknown anti-pattern".to_string(),
                category: "Unknown".to_string(),
            },
        }
    }

    fn generate_issues_by_severity(&self, issues: &[ArchitecturalIssue]) -> String {
        let mut content = String::from("## Issues by Severity\n\n");
        let mut issues_by_severity: HashMap<String, Vec<&ArchitecturalIssue>> = HashMap::new();

        for issue in issues {
            issues_by_severity.entry(issue.severity.clone()).or_default().push(issue);
        }

        let severities = ["critical", "high", "medium", "low"];
        for &severity in &severities {
            if let Some(issues_list) = issues_by_severity.get(severity) {
                content.push_str(&format!("### {}\n\n", severity.to_uppercase()));
                for issue in issues_list {
                    let ap_type = self.lookup_antipattern_type(issue.anti_pattern_type_id);
                    content.push_str(&format!(
                        "- **{}**: `{}` (Lines {}-{}) [{}]\n",
                        ap_type.name,
                        issue.file_path,
                        issue.start_line.unwrap_or(0),
                        issue.end_line.unwrap_or(0),
                        ap_type.category.clone()
                    ));
                }
                content.push('\n');
            }
        }
        content
    }

    fn generate_detailed_issues(&self, issues: &[ArchitecturalIssue]) -> String {
        let mut content = String::from("## Detailed Issue Analysis\n\n");

        for (index, issue) in issues.iter().enumerate() {
            let ap_type = self.lookup_antipattern_type(issue.anti_pattern_type_id);
            content.push_str(&format!(
                r"### Issue #{}: {}

**File:** `{}`
**Lines:** {}-{}
**Severity:** {}
**Anti-pattern:** {}
**Category:** {}

**Description:**
{}

",
                index + 1,
                ap_type.name,
                issue.file_path,
                issue.start_line.unwrap_or(0),
                issue.end_line.unwrap_or(0),
                issue.severity.to_uppercase(),
                ap_type.name,
                ap_type.category.clone(),
                ap_type.description
            ));

            // Add code snippet if available
            if self.include_code_snippets {
                if let Some(snippet) = &issue.code_snippet {
                    content.push_str(&format!(
                        r"**Code Context:**
```rust
{}
```

",
                        snippet
                    ));
                }
            }

            // Add AI explanation if available
            if self.include_ai_explanations {
                if let Some(explanation) = &issue.ai_explanation {
                    content.push_str(&format!(
                        r"**AI Analysis:**
{}

",
                        explanation
                    ));
                }
            }

            // Add Mermaid.js diagram if enabled
            if self.include_diagrams {
                if let Some(diagram) = self.generate_mermaid_diagram_for_issue(issue) {
                    content.push_str(&format!(
                        r"**Architecture Diagram:**
{}

",
                        diagram
                    ));
                }
            }

            content.push_str("---\n\n");
        }

        content
    }

    /// Generate a Mermaid.js diagram for an architectural issue using the SAM and LLM
    /// This is a stub; actual LLM integration will be added in the next step
    fn generate_mermaid_diagram_for_issue(&self, issue: &ArchitecturalIssue) -> Option<String> {
        // TODO: Extract relevant SAM subgraph for the issue
        // TODO: Serialize to JSON and prepare LLM prompt (see Ai_Diagrams.md)
        // TODO: Call AI engine to get Mermaid.js code
        // For now, return a placeholder diagram
        Some(format!(
            "```mermaid\ngraph TD\n    A[{}] --> B[Related Component]\n```\n",
            issue.description.replace('"', "'"),
        ))
    }

    /// Generate a Mermaid.js diagram for the full dependency graph
    pub fn generate_mermaid_diagram_for_graph(&self, graph: &crate::analysis::dependency_graph::LocalDependencyGraph) -> String {
        let mut diagram = String::from("```mermaid\ngraph TD\n");
        let petgraph = graph.get_petgraph();
        
        for edge in petgraph.edge_indices() {
            if let Some((from_idx, to_idx)) = petgraph.edge_endpoints(edge) {
                if let (Some(from_node), Some(to_node)) = (graph.get_node_from_index(from_idx), graph.get_node_from_index(to_idx)) {
                    let from_name = match from_node {
                        ComponentNode::Module { path } => path.split('/').last().unwrap_or(path),
                        ComponentNode::Class { name, .. } => name,
                        ComponentNode::Function { name, .. } => name,
                    };
                    let to_name = match to_node {
                        ComponentNode::Module { path } => path.split('/').last().unwrap_or(path),
                        ComponentNode::Class { name, .. } => name,
                        ComponentNode::Function { name, .. } => name,
                    };
                    diagram.push_str(&format!("    {} --> {}\n", from_name, to_name));
                }
            }
        }
        diagram.push_str("```");
        diagram
    }

    fn calculate_duration(&self, analysis_run: &AnalysisRun) -> String {
        if let Some(end_time) = analysis_run.end_time {
            let duration = end_time.signed_duration_since(analysis_run.start_time);
            format!("{} seconds", duration.num_seconds())
        } else {
            "N/A".to_string()
        }
    }

    fn calculate_severity_breakdown(&self, issues: &[ArchitecturalIssue]) -> HashMap<String, usize> {
        let mut breakdown = HashMap::new();
        for issue in issues {
            *breakdown.entry(issue.severity.clone()).or_insert(0) += 1;
        }
        breakdown
    }

    fn calculate_category_breakdown(&self, issues: &[ArchitecturalIssue]) -> HashMap<String, usize> {
        let mut breakdown = HashMap::new();
        // This would require joining with AntiPatternType to get category
        // For now, just a placeholder
        for _issue in issues {
            *breakdown.entry("unknown".to_string()).or_insert(0) += 1;
        }
        breakdown
    }
}
