use std::collections::HashMap;
use tempfile::TempDir;
use uveddi::database::models::{AnalysisRun, AntiPatternType, ArchitecturalIssue};
use uveddi::report::ReportGenerator;

#[test]
fn test_markdown_report_generation() {
    // Create test data
    let analysis_run = AnalysisRun {
        run_id: Some(1),
        timestamp: chrono::Utc::now(),
        codebase_path: "/path/to/test/codebase".to_string(),
        duration_seconds: Some(2.5),
    };
    
    let anti_pattern_types = {
        let mut map = HashMap::new();
        map.insert(
            1,
            AntiPatternType {
                type_id: Some(1),
                name: "Cyclic Dependency".to_string(),
                description: "A circular dependency between modules that creates tight coupling".to_string(),
            },
        );
        map.insert(
            2,
            AntiPatternType {
                type_id: Some(2),
                name: "God Object".to_string(),
                description: "A class that has too many responsibilities and methods".to_string(),
            },
        );
        map
    };
    
    let issues = vec![
        // Cyclic dependency issue
        ArchitecturalIssue {
            issue_id: Some(1),
            analysis_run_id: 1,
            anti_pattern_type_id: 1,
            file_path: "src/cyclic_a.rs".to_string(),
            start_line: Some(10),
            end_line: Some(15),
            severity: "high".to_string(),
            description: "Cyclic dependency detected between components: ModuleA → ModuleB → ModuleC".to_string(),
            code_snippet: Some("pub fn function_a() {\n    println!(\"Function A called\");\n    crate::cyclic_b::function_b();\n}".to_string()),
            ai_explanation: Some("{\"title\":\"Circular Module Dependency\",\"explanation\":\"This creates tight coupling between components that should be independent.\",\"refactoring\":\"Extract shared functionality to a separate module.\",\"confidence\":\"high\"}".to_string()),
        },
        // God object issue
        ArchitecturalIssue {
            issue_id: Some(2),
            analysis_run_id: 1,
            anti_pattern_type_id: 2,
            file_path: "src/god_object.rs".to_string(),
            start_line: Some(5),
            end_line: Some(100),
            severity: "medium".to_string(),
            description: "God Object: SystemManager has 15 methods and too many responsibilities".to_string(),
            code_snippet: Some("struct SystemManager {\n    // fields\n}\n\nimpl SystemManager {\n    pub fn start_service(&self) {}\n    pub fn stop_service(&self) {}\n    // ... many more methods\n}".to_string()),
            ai_explanation: Some("{\"title\":\"God Object Anti-pattern\",\"explanation\":\"This class violates the Single Responsibility Principle by handling too many concerns.\",\"refactoring\":\"Split into smaller classes focused on single responsibilities.\",\"confidence\":\"high\"}".to_string()),
        },
    ];
    
    // Create temporary directory for output
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("report.md");
    
    // Generate report
    let generator = ReportGenerator::new();
    let result = generator.generate_markdown_report(
        &analysis_run,
        &issues,
        &anti_pattern_types,
        Some(&output_path),
    );
    
    // Verify success
    assert!(result.is_ok(), "Report generation should succeed");
    
    // Verify file was created
    assert!(output_path.exists(), "Report file should exist");
    
    // Read file contents
    let report_content = std::fs::read_to_string(&output_path).unwrap();
    
    // Verify report contains expected content
    assert!(report_content.contains("# Uveddi Architectural Analysis Report"), 
            "Report should have the correct title");
    assert!(report_content.contains("## Executive Summary"), 
            "Report should have an executive summary");
    assert!(report_content.contains("identified **2 architectural issues**"), 
            "Executive summary should count issues correctly");
    
    // Verify severity sections
    assert!(report_content.contains("### 🔴 High Severity Issues"), 
            "Should show high severity section");
    assert!(report_content.contains("### 🟠 Medium Severity Issues"), 
            "Should show medium severity section");
    
    // Verify anti-pattern sections
    assert!(report_content.contains("### Cyclic Dependency"), 
            "Should include cyclic dependency section");
    assert!(report_content.contains("### God Object"), 
            "Should include god object section");
    
    // Verify code snippets are included
    assert!(report_content.contains("**Code Snippet**:"), 
            "Should include code snippets");
    assert!(report_content.contains("pub fn function_a()"), 
            "Should include the actual code");
    
    // Verify AI explanations are included
    assert!(report_content.contains("**AI Analysis**:"), 
            "Should include AI analysis");
    assert!(report_content.contains("Extract shared functionality"), 
            "Should include refactoring recommendations");
    
    // Verify diagrams are included
    assert!(report_content.contains("```mermaid"), 
            "Should include Mermaid.js diagrams");
}

#[test]
fn test_json_report_generation() {
    // Create test data (same as markdown test)
    let analysis_run = AnalysisRun {
        run_id: Some(1),
        timestamp: chrono::Utc::now(),
        codebase_path: "/path/to/test/codebase".to_string(),
        duration_seconds: Some(2.5),
    };
    
    let anti_pattern_types = {
        let mut map = HashMap::new();
        map.insert(
            1,
            AntiPatternType {
                type_id: Some(1),
                name: "Cyclic Dependency".to_string(),
                description: "A circular dependency between modules that creates tight coupling".to_string(),
            },
        );
        map.insert(
            2,
            AntiPatternType {
                type_id: Some(2),
                name: "God Object".to_string(),
                description: "A class that has too many responsibilities and methods".to_string(),
            },
        );
        map
    };
    
    let issues = vec![
        // Cyclic dependency issue
        ArchitecturalIssue {
            issue_id: Some(1),
            analysis_run_id: 1,
            anti_pattern_type_id: 1,
            file_path: "src/cyclic_a.rs".to_string(),
            start_line: Some(10),
            end_line: Some(15),
            severity: "high".to_string(),
            description: "Cyclic dependency detected between components: ModuleA → ModuleB → ModuleC".to_string(),
            code_snippet: Some("pub fn function_a() {\n    println!(\"Function A called\");\n    crate::cyclic_b::function_b();\n}".to_string()),
            ai_explanation: Some("{\"title\":\"Circular Module Dependency\",\"explanation\":\"This creates tight coupling between components that should be independent.\",\"refactoring\":\"Extract shared functionality to a separate module.\",\"confidence\":\"high\"}".to_string()),
        },
        // God object issue
        ArchitecturalIssue {
            issue_id: Some(2),
            analysis_run_id: 1,
            anti_pattern_type_id: 2,
            file_path: "src/god_object.rs".to_string(),
            start_line: Some(5),
            end_line: Some(100),
            severity: "medium".to_string(),
            description: "God Object: SystemManager has 15 methods and too many responsibilities".to_string(),
            code_snippet: Some("struct SystemManager {\n    // fields\n}\n\nimpl SystemManager {\n    pub fn start_service(&self) {}\n    pub fn stop_service(&self) {}\n    // ... many more methods\n}".to_string()),
            ai_explanation: Some("{\"title\":\"God Object Anti-pattern\",\"explanation\":\"This class violates the Single Responsibility Principle by handling too many concerns.\",\"refactoring\":\"Split into smaller classes focused on single responsibilities.\",\"confidence\":\"high\"}".to_string()),
        },
    ];
    
    // Create temporary directory for output
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("report.json");
    
    // Generate report
    let generator = ReportGenerator::new();
    let result = generator.generate_json_report(
        &analysis_run,
        &issues,
        &anti_pattern_types,
        Some(&output_path),
    );
    
    // Verify success
    assert!(result.is_ok(), "JSON report generation should succeed");
    
    // Verify file was created
    assert!(output_path.exists(), "JSON report file should exist");
    
    // Read file contents
    let json_content = std::fs::read_to_string(&output_path).unwrap();
    
    // Parse JSON to verify structure
    let json: serde_json::Value = serde_json::from_str(&json_content).unwrap();
    
    // Verify JSON structure
    assert!(json.is_object(), "Should be a JSON object");
    assert!(json.get("metadata").is_some(), "Should have metadata object");
    assert!(json.get("issues").is_some(), "Should have issues array");
    
    // Verify issues count
    let issues_array = json.get("issues").unwrap().as_array().unwrap();
    assert_eq!(issues_array.len(), 2, "Should have 2 issues");
    
    // Verify first issue contents
    let first_issue = &issues_array[0];
    assert_eq!(first_issue.get("severity").unwrap(), "high", "Should have correct severity");
    assert!(first_issue.get("description").unwrap().as_str().unwrap().contains("Cyclic dependency"), 
            "Should have correct description");
    
    // Verify anti-pattern type info is included
    assert!(first_issue.get("antiPatternType").is_some(), 
            "Should include anti-pattern type");
    assert_eq!(first_issue.get("antiPatternType").unwrap(), "Cyclic Dependency", 
            "Should have correct anti-pattern type");
}
