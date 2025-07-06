//! Reporting output and summary tests
//!
//! This module tests the markdown and JSON report generation for analysis runs and architectural issues.
//! It ensures that reports contain the expected content and are formatted correctly.
//! Additionally, it verifies that summary statistics are accurate and that reports include
//! both code snippets and AI-generated explanations where applicable.

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use std::collections::HashMap;
    use uveddi::database::models::{AnalysisRun, ArchitecturalIssue, AntiPatternType};
    use uveddi::report::ReportGenerator;

    #[test]
    fn markdown_report_output() {
        // Test that the markdown report contains key issue details and AI analysis
        let analysis_run = AnalysisRun {
            run_id: Some(1),
            project_id: 1,
            start_time: Utc::now(),
            end_time: None,
            status: "completed".to_string(),
            total_files_analyzed: Some(1),
            total_issues_found: Some(1),
            analysis_config: "{}".to_string(),
        };
        let issues = vec![ArchitecturalIssue {
            issue_id: Some(1),
            analysis_run_id: 1,
            anti_pattern_type_id: 1,
            file_path: "src/main.rs".to_string(),
            start_line: Some(10),
            end_line: Some(20),
            severity: "high".to_string(),
            description: "God Object with too many methods".to_string(),
            code_snippet: Some("struct GodObject { ... }".to_string()),
            ai_explanation: Some("This is a God Object because...".to_string()),
        }];
        
        // Create anti_pattern_types HashMap
        let mut anti_pattern_types = HashMap::new();
        anti_pattern_types.insert(1, AntiPatternType {
            anti_pattern_type_id: Some(1),
            name: "God Object".to_string(),
            description: "A class that knows too much or does too much".to_string(),
            category: "structural".to_string(),
        });
        
        let generator = ReportGenerator::new();
        let report = generator
            .generate_markdown_report(&analysis_run, &issues, &anti_pattern_types, None)
            .unwrap();
        assert!(report.contains("God Object"));
        assert!(report.contains("AI Analysis"));
        assert!(report.contains("src/main.rs"));
        assert!(report.contains("struct GodObject"));
    }

    #[test]
    fn json_report_output() {
        // Test that the JSON report contains the correct structure and issue details
        let analysis_run = AnalysisRun {
            run_id: Some(1),
            project_id: 1,
            start_time: Utc::now(),
            end_time: None,
            status: "completed".to_string(),
            total_files_analyzed: Some(1),
            total_issues_found: Some(1),
            analysis_config: "{}".to_string(),
        };
        let issues = vec![ArchitecturalIssue {
            issue_id: Some(1),
            analysis_run_id: 1,
            anti_pattern_type_id: 1,
            file_path: "src/main.rs".to_string(),
            start_line: Some(10),
            end_line: Some(20),
            severity: "high".to_string(),
            description: "God Object with too many methods".to_string(),
            code_snippet: Some("struct GodObject { ... }".to_string()),
            ai_explanation: Some("This is a God Object because...".to_string()),
        }];
        
        // Create anti_pattern_types HashMap
        let mut anti_pattern_types = HashMap::new();
        anti_pattern_types.insert(1, AntiPatternType {
            anti_pattern_type_id: Some(1),
            name: "God Object".to_string(),
            description: "A class that knows too much or does too much".to_string(),
            category: "structural".to_string(),
        });
        
        let generator = ReportGenerator::new();
        let report = generator
            .generate_json_report(&analysis_run, &issues, &anti_pattern_types, None)
            .unwrap();
        let report_str = serde_json::to_string(&report).unwrap();
        let json: serde_json::Value = serde_json::from_str(&report_str).unwrap();
        assert_eq!(json["run_id"], 1);
        assert_eq!(
            json["issues"][0]["description"],
            "God Object with too many methods"
        );
    }

    #[test]
    fn report_summary_statistics() {
        // Test that the report summary includes correct statistics for files analyzed and issues found
        let analysis_run = AnalysisRun {
            run_id: Some(1),
            project_id: 1,
            start_time: Utc::now(),
            end_time: None,
            status: "completed".to_string(),
            total_files_analyzed: Some(10),
            total_issues_found: Some(5),
            analysis_config: "{}".to_string(),
        };
        let issues = vec![];
        
        // Create anti_pattern_types HashMap (empty for this test)
        let anti_pattern_types = HashMap::new();
        
        let generator = ReportGenerator::new();
        let report = generator
            .generate_markdown_report(&analysis_run, &issues, &anti_pattern_types, None)
            .unwrap();
        assert!(report.contains("**Files Analyzed:** 10"));
        assert!(report.contains("**Issues Found:** 5"));
    }

    #[test]
    fn report_includes_code_and_ai_context() {
        // Test that the report includes both code snippets and AI context for issues
        let analysis_run = AnalysisRun {
            run_id: Some(1),
            project_id: 1,
            start_time: Utc::now(),
            end_time: None,
            status: "completed".to_string(),
            total_files_analyzed: Some(1),
            total_issues_found: Some(1),
            analysis_config: "{}".to_string(),
        };
        let issues = vec![ArchitecturalIssue {
            issue_id: Some(1),
            analysis_run_id: 1,
            anti_pattern_type_id: 1,
            file_path: "src/main.rs".to_string(),
            start_line: Some(10),
            end_line: Some(20),
            severity: "high".to_string(),
            description: "God Object with too many methods".to_string(),
            code_snippet: Some("struct GodObject { ... }".to_string()),
            ai_explanation: Some("This is a God Object because...".to_string()),
        }];
        
        // Create anti_pattern_types HashMap
        let mut anti_pattern_types = HashMap::new();
        anti_pattern_types.insert(1, AntiPatternType {
            anti_pattern_type_id: Some(1),
            name: "God Object".to_string(),
            description: "A class that knows too much or does too much".to_string(),
            category: "structural".to_string(),
        });
        
        let generator = ReportGenerator::new();
        let report = generator
            .generate_markdown_report(&analysis_run, &issues, &anti_pattern_types, None)
            .unwrap();
        assert!(report.contains("struct GodObject { ... }"));
        assert!(report.contains("This is a God Object because..."));
    }
}
