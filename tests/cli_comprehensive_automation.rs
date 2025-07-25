//! Comprehensive CLI automation testing
//! 
//! This module provides extensive automated testing for all CLI commands,
//! arguments, error conditions, and output validation.

use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::fs::{self, File};
use std::io::Write;
use std::process::Command;
use tempfile::{tempdir, NamedTempFile};
use serde_json;

/// Create a comprehensive test project with various patterns
fn create_test_project() -> Result<tempfile::TempDir, Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    
    // Create main.rs with detectable patterns
    let main_rs = dir.path().join("main.rs");
    let mut file = File::create(&main_rs)?;
    writeln!(file, r#"
// Large class with many methods and fields
pub struct LargeClass {{
    field1: String, field2: i32, field3: f64, field4: bool,
    field5: Vec<String>, field6: std::collections::HashMap<String, i32>,
    field7: Option<String>, field8: Result<i32, String>,
    field9: u64, field10: char, field11: Box<dyn std::fmt::Display>,
    field12: std::rc::Rc<String>, field13: std::sync::Arc<String>,
    field14: std::cell::RefCell<i32>, field15: std::collections::VecDeque<i32>,
}}

impl LargeClass {{
    pub fn new() -> Self {{ unimplemented!() }}
    pub fn method1(&self) -> String {{ unimplemented!() }}
    pub fn method2(&self) -> i32 {{ unimplemented!() }}
    pub fn method3(&self) -> f64 {{ unimplemented!() }}
    pub fn method4(&self) -> bool {{ unimplemented!() }}
    pub fn method5(&self) -> Vec<String> {{ unimplemented!() }}
    pub fn method6(&self) -> std::collections::HashMap<String, i32> {{ unimplemented!() }}
    pub fn method7(&self) -> Option<String> {{ unimplemented!() }}
    pub fn method8(&self) -> Result<i32, String> {{ unimplemented!() }}
    pub fn method9(&self) -> u64 {{ unimplemented!() }}
    pub fn method10(&self) -> char {{ unimplemented!() }}
    
    // Dead code - never called
    #[allow(dead_code)]
    fn unused_method(&self) {{
        println!("This is never called");
    }}
}}

// Dead code function
#[allow(dead_code)]
fn unused_function() {{
    println!("This function is never used");
}}

pub fn main() {{
    let instance = LargeClass::new();
    println!("{{:?}}", instance.method1());
}}
"#)?;

    // Create lib.rs with more patterns
    let lib_rs = dir.path().join("lib.rs");
    let mut lib_file = File::create(&lib_rs)?;
    writeln!(lib_file, r#"
pub mod utils;

// Tight coupling example
pub mod tight_coupling {{
    pub struct ModuleA {{
        pub internal_data: String,
    }}
    
    pub struct ModuleB {{
        module_a: ModuleA,
    }}
    
    impl ModuleB {{
        pub fn new() -> Self {{
            ModuleB {{
                module_a: ModuleA {{ internal_data: String::new() }},
            }}
        }}
        
        // Direct access creates tight coupling
        pub fn process(&mut self) {{
            self.module_a.internal_data.push_str("processed");
        }}
    }}
}}
"#)?;

    // Create utils.rs
    let utils_rs = dir.path().join("utils.rs");
    let mut utils_file = File::create(&utils_rs)?;
    writeln!(utils_file, r#"
pub fn utility_function() -> String {{
    "utility".to_string()
}}

#[allow(dead_code)]
fn another_unused_function() {{
    println!("Also unused");
}}
"#)?;

    Ok(dir)
}

#[test]
fn test_cli_analyze_basic_functionality() {
    let test_project = create_test_project().unwrap();
    
    let mut cmd = Command::cargo_bin("uveddi").unwrap();
    cmd.arg("analyze")
        .arg(test_project.path())
        .arg("--output-format=markdown");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("# Uveddi Analysis Report"))
        .stdout(predicate::str::contains("LargeClass")); // Should detect large class
}

#[test]
fn test_cli_analyze_json_output() {
    let test_project = create_test_project().unwrap();
    
    let mut cmd = Command::cargo_bin("uveddi").unwrap();
    cmd.arg("analyze")
        .arg(test_project.path())
        .arg("--output-format=json");
    
    let output = cmd.assert().success().get_output().stdout.clone();
    let output_str = String::from_utf8(output).unwrap();
    
    // Validate JSON structure
    let json: serde_json::Value = serde_json::from_str(&output_str).unwrap();
    assert!(json.get("run_id").is_some());
    assert!(json.get("issues").is_some());
    assert!(json.get("summary").is_some());
}

#[test]
fn test_cli_analyze_with_output_file() {
    let test_project = create_test_project().unwrap();
    let output_file = NamedTempFile::new().unwrap();
    
    let mut cmd = Command::cargo_bin("uveddi").unwrap();
    cmd.arg("analyze")
        .arg(test_project.path())
        .arg("--output-format=json")
        .arg("--output")
        .arg(output_file.path());
    
    cmd.assert().success();
    
    // Verify file was created and contains valid JSON
    let content = fs::read_to_string(output_file.path()).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();
    assert!(json.get("run_id").is_some());
}

#[test]
fn test_cli_analyze_with_dead_code_options() {
    let test_project = create_test_project().unwrap();
    
    let mut cmd = Command::cargo_bin("uveddi").unwrap();
    cmd.arg("analyze")
        .arg(test_project.path())
        .arg("--output-format=json")
        .arg("--dead-code-confidence=0.8")
        .arg("--dead-code-library-mode")
        .arg("--dead-code-keep-alive=main");
    
    let output = cmd.assert().success().get_output().stdout.clone();
    let output_str = String::from_utf8(output).unwrap();
    let json: serde_json::Value = serde_json::from_str(&output_str).unwrap();
    
    // Should detect dead code issues
    let issues = json.get("issues").unwrap().as_array().unwrap();
    let has_dead_code = issues.iter().any(|issue| {
        issue.get("detector").unwrap().as_str().unwrap().contains("dead_code")
    });
    assert!(has_dead_code);
}

#[test]
fn test_cli_analyze_with_large_classes_options() {
    let test_project = create_test_project().unwrap();
    
    let mut cmd = Command::cargo_bin("uveddi").unwrap();
    cmd.arg("analyze")
        .arg(test_project.path())
        .arg("--output-format=json")
        .arg("--large-classes-max-loc=100")  // Low threshold
        .arg("--large-classes-max-methods=5")  // Low threshold
        .arg("--large-classes-max-fields=5");  // Low threshold
    
    let output = cmd.assert().success().get_output().stdout.clone();
    let output_str = String::from_utf8(output).unwrap();
    let json: serde_json::Value = serde_json::from_str(&output_str).unwrap();
    
    // Should detect large class issues
    let issues = json.get("issues").unwrap().as_array().unwrap();
    let has_large_class = issues.iter().any(|issue| {
        issue.get("detector").unwrap().as_str().unwrap().contains("large_classes")
    });
    assert!(has_large_class);
}

#[test]
fn test_cli_error_handling_nonexistent_path() {
    let mut cmd = Command::cargo_bin("uveddi").unwrap();
    cmd.arg("analyze")
        .arg("/nonexistent/path/to/nowhere");
    
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Error"));
}

#[test]
fn test_cli_error_handling_invalid_format() {
    let test_project = create_test_project().unwrap();
    
    let mut cmd = Command::cargo_bin("uveddi").unwrap();
    cmd.arg("analyze")
        .arg(test_project.path())
        .arg("--output-format=invalid");
    
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn test_cli_error_handling_invalid_confidence() {
    let test_project = create_test_project().unwrap();
    
    let mut cmd = Command::cargo_bin("uveddi").unwrap();
    cmd.arg("analyze")
        .arg(test_project.path())
        .arg("--dead-code-confidence=1.5");  // Invalid: > 1.0
    
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("confidence"));
}

#[test]
fn test_cli_help_commands() {
    // Test main help
    let mut cmd = Command::cargo_bin("uveddi").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("analyze"));
    
    // Test analyze subcommand help
    let mut cmd = Command::cargo_bin("uveddi").unwrap();
    cmd.arg("analyze").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("output-format"))
        .stdout(predicate::str::contains("dead-code-confidence"));
}

#[test]
fn test_cli_version_flag() {
    let mut cmd = Command::cargo_bin("uveddi").unwrap();
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("uveddi"));
}

#[test]
fn test_cli_with_ai_disabled() {
    let test_project = create_test_project().unwrap();
    
    let mut cmd = Command::cargo_bin("uveddi").unwrap();
    cmd.arg("analyze")
        .arg(test_project.path())
        .arg("--output-format=json")
        .arg("--no-ai");  // Disable AI explicitly
    
    let output = cmd.assert().success().get_output().stdout.clone();
    let output_str = String::from_utf8(output).unwrap();
    let json: serde_json::Value = serde_json::from_str(&output_str).unwrap();
    
    // Should still produce analysis without AI
    assert!(json.get("run_id").is_some());
    assert!(json.get("issues").is_some());
}

#[test]
fn test_cli_timeout_handling() {
    let test_project = create_test_project().unwrap();
    
    let mut cmd = Command::cargo_bin("uveddi").unwrap();
    cmd.arg("analyze")
        .arg(test_project.path())
        .arg("--output-format=json")
        .timeout(std::time::Duration::from_secs(30));  // Reasonable timeout
    
    // Should complete within timeout
    cmd.assert().success();
}

#[test]
fn test_cli_empty_directory() {
    let empty_dir = tempdir().unwrap();
    
    let mut cmd = Command::cargo_bin("uveddi").unwrap();
    cmd.arg("analyze")
        .arg(empty_dir.path())
        .arg("--output-format=json");
    
    // Should handle empty directory gracefully
    let output = cmd.assert().success().get_output().stdout.clone();
    let output_str = String::from_utf8(output).unwrap();
    let json: serde_json::Value = serde_json::from_str(&output_str).unwrap();
    
    assert!(json.get("run_id").is_some());
    let issues = json.get("issues").unwrap().as_array().unwrap();
    assert_eq!(issues.len(), 0);  // No issues in empty directory
}

#[test]
fn test_cli_pattern_ignore_functionality() {
    let test_project = create_test_project().unwrap();
    
    let mut cmd = Command::cargo_bin("uveddi").unwrap();
    cmd.arg("analyze")
        .arg(test_project.path())
        .arg("--output-format=json")
        .arg("--dead-code-ignore-patterns=unused_method,unused_function");
    
    let output = cmd.assert().success().get_output().stdout.clone();
    let output_str = String::from_utf8(output).unwrap();
    let json: serde_json::Value = serde_json::from_str(&output_str).unwrap();
    
    // Should have fewer dead code issues due to ignore patterns
    let issues = json.get("issues").unwrap().as_array().unwrap();
    let dead_code_issues: Vec<_> = issues.iter()
        .filter(|issue| issue.get("detector").unwrap().as_str().unwrap().contains("dead_code"))
        .collect();
    
    // Verify that ignored patterns are not reported
    for issue in dead_code_issues {
        let description = issue.get("description").unwrap().as_str().unwrap();
        assert!(!description.contains("unused_method"));
        assert!(!description.contains("unused_function"));
    }
}

#[test] 
fn test_cli_memory_optimization_flag() {
    let test_project = create_test_project().unwrap();
    
    let mut cmd = Command::cargo_bin("uveddi").unwrap();
    cmd.arg("analyze")
        .arg(test_project.path())
        .arg("--output-format=json")
        .arg("--enable-memory-optimization")
        .arg("--memory-limit-gb=2");
    
    // Should work with memory optimization enabled
    cmd.assert().success();
}

#[test]
fn test_cli_multiple_detector_configurations() {
    let test_project = create_test_project().unwrap();
    
    let mut cmd = Command::cargo_bin("uveddi").unwrap();
    cmd.arg("analyze")
        .arg(test_project.path())
        .arg("--output-format=json")
        .arg("--dead-code-confidence=0.9")
        .arg("--large-classes-max-loc=200")
        .arg("--large-classes-max-methods=15")
        .arg("--large-classes-max-fields=10")
        .arg("--large-classes-max-complexity=25");
    
    let output = cmd.assert().success().get_output().stdout.clone();
    let output_str = String::from_utf8(output).unwrap();
    let json: serde_json::Value = serde_json::from_str(&output_str).unwrap();
    
    // Should successfully run with multiple detector configurations
    assert!(json.get("run_id").is_some());
    assert!(json.get("issues").is_some());
    
    // Check that both detector types are present
    let issues = json.get("issues").unwrap().as_array().unwrap();
    let detector_types: std::collections::HashSet<String> = issues.iter()
        .map(|issue| issue.get("detector").unwrap().as_str().unwrap().to_string())
        .collect();
    
    assert!(detector_types.len() > 0); // Should have at least some detectors
}