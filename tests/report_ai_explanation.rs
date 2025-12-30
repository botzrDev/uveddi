//! Unit tests for report generator AI explanation rendering
//!
//! NOTE: This test module references outdated struct fields and methods.
//! Enable by removing the #![cfg(feature = "report-tests")] gate after updating.

#![cfg(feature = "report-tests")]

use chrono::Utc;
use std::collections::HashMap;
use uveddi::database::models::{AnalysisRun, AntiPatternType, ArchitecturalIssue};
use uveddi::report::ReportGenerator;

#[test]
fn report_includes_ai_explanation_when_present() {
    let run = AnalysisRun {
        run_id: Some(1),
        project_id: 1,
        start_time: Utc::now(),
        end_time: Some(Utc::now()),
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
        start_line: Some(1),
        end_line: Some(10),
        severity: "high".to_string(),
        description: "God Object".to_string(),
        code_snippet: Some("struct GodObject { ... }".to_string()),
        ai_explanation: Some("This is an AI explanation.".to_string()),
    }];

    // Create anti_pattern_types HashMap
    let mut anti_pattern_types = HashMap::new();
    anti_pattern_types.insert(
        1,
        AntiPatternType {
            anti_pattern_type_id: Some(1),
            name: "God Object".to_string(),
            description: "A class that knows too much or does too much".to_string(),
            category: "structural".to_string(),
        },
    );

    let report = ReportGenerator::new()
        .generate_markdown_report(&run, &issues, &anti_pattern_types, None)
        .unwrap();
    assert!(report.contains("AI Analysis"));
    assert!(report.contains("This is an AI explanation."));
}

#[test]
fn report_excludes_ai_explanation_when_absent() {
    let run = AnalysisRun {
        run_id: Some(1),
        project_id: 1,
        start_time: Utc::now(),
        end_time: Some(Utc::now()),
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
        start_line: Some(1),
        end_line: Some(10),
        severity: "high".to_string(),
        description: "God Object".to_string(),
        code_snippet: Some("struct GodObject { ... }".to_string()),
        ai_explanation: None,
    }];

    // Create anti_pattern_types HashMap
    let mut anti_pattern_types = HashMap::new();
    anti_pattern_types.insert(
        1,
        AntiPatternType {
            anti_pattern_type_id: Some(1),
            name: "God Object".to_string(),
            description: "A class that knows too much or does too much".to_string(),
            category: "structural".to_string(),
        },
    );

    let report = ReportGenerator::new()
        .generate_markdown_report(&run, &issues, &anti_pattern_types, None)
        .unwrap();
    assert!(!report.contains("AI Analysis"));
}
