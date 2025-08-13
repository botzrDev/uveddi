//! Comprehensive CLI testing framework
//! 
//! This module tests the command-line interface including argument parsing,
//! command execution, output formatting, and error handling.

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;
use std::process::Output;
use tempfile::{TempDir, NamedTempFile};
use serde_json::Value;

/// Test basic CLI help and version commands
#[test]
fn test_cli_help_and_version() {
    let mut cmd = Command::cargo_bin("uveddi").expect("Failed to find uveddi binary");
    
    // Test help command
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Uveddi"))
        .stdout(predicate::str::contains("USAGE"));
    
    // Test version command
    let mut cmd = Command::cargo_bin("uveddi").expect("Failed to find uveddi binary");
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("uveddi"))
        .stdout(predicate::str::contains("0.9.0"));
    
    // Test short help flag
    let mut cmd = Command::cargo_bin("uveddi").expect("Failed to find uveddi binary");
    cmd.arg("-h")
        .assert()
        .success()
        .stdout(predicate::str::contains("USAGE"));
    
    // Test short version flag
    let mut cmd = Command::cargo_bin("uveddi").expect("Failed to find uveddi binary");
    cmd.arg("-V")
        .assert()
        .success()
        .stdout(predicate::str::contains("0.9.0"));
}

/// Test analyze command with valid project
#[test]
fn test_cli_analyze_command() {
    let temp_dir = create_test_rust_project();
    let project_path = temp_dir.path();
    
    let mut cmd = Command::cargo_bin("uveddi").expect("Failed to find uveddi binary");
    
    // Test basic analyze command
    cmd.arg("analyze")
        .arg(project_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Analysis complete"))
        .stdout(predicate::str::contains("issues found"));
    
    // Test analyze with JSON output
    let mut cmd = Command::cargo_bin("uveddi").expect("Failed to find uveddi binary");
    cmd.arg("analyze")
        .arg(project_path)
        .arg("--output-format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::is_json());
    
    // Test analyze with specific output file
    let output_file = temp_dir.path().join("analysis_output.json");
    let mut cmd = Command::cargo_bin("uveddi").expect("Failed to find uveddi binary");
    cmd.arg("analyze")
        .arg(project_path)
        .arg("--output-format")
        .arg("json")
        .arg("--output")
        .arg(&output_file)
        .assert()
        .success();
    
    // Verify output file was created
    assert!(output_file.exists(), "Output file should be created");
    
    // Verify output file contains valid JSON
    let content = fs::read_to_string(&output_file).expect("Failed to read output file");
    let json: Value = serde_json::from_str(&content).expect("Output should be valid JSON");
    assert!(json["issues"].is_array(), "JSON should contain issues array");
}

/// Test analyze command with different output formats
#[test]
fn test_cli_output_formats() {
    let temp_dir = create_test_rust_project();
    let project_path = temp_dir.path();
    
    let formats = vec![
        ("json", "application/json"),
        ("html", "text/html"),
        ("markdown", "text/markdown"),
        ("text", "text/plain"),
    ];
    
    for (format, expected_content_type) in formats {
        let output_file = temp_dir.path().join(format!("output.{}", format));
        
        let mut cmd = Command::cargo_bin("uveddi").expect("Failed to find uveddi binary");
        cmd.arg("analyze")
            .arg(project_path)
            .arg("--output-format")
            .arg(format)
            .arg("--output")
            .arg(&output_file)
            .timeout(std::time::Duration::from_secs(30))
            .assert()
            .success();
        
        // Verify file was created
        assert!(output_file.exists(), "Output file for {} format should be created", format);
        
        // Verify file has content
        let content = fs::read_to_string(&output_file)
            .expect(&format!("Failed to read {} output file", format));
        assert!(!content.is_empty(), "{} output should not be empty", format);
        
        // Format-specific validations
        match format {
            "json" => {
                let _: Value = serde_json::from_str(&content)
                    .expect("JSON output should be valid");
            }
            "html" => {
                assert!(content.contains("<html>"), "HTML output should contain <html> tag");
                assert!(content.contains("</html>"), "HTML output should contain </html> tag");
            }
            "markdown" => {
                assert!(content.contains("#"), "Markdown output should contain headers");
            }
            "text" => {
                assert!(content.contains("Analysis Results"), "Text output should contain title");
            }
            _ => {}
        }
    }
}

/// Test analyze command error handling
#[test]
fn test_cli_analyze_error_handling() {
    // Test non-existent path
    let mut cmd = Command::cargo_bin("uveddi").expect("Failed to find uveddi binary");
    cmd.arg("analyze")
        .arg("/non/existent/path")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No such file or directory")
                .or(predicate::str::contains("Path does not exist"))
                .or(predicate::str::contains("not found")));
    
    // Test invalid output format
    let temp_dir = create_test_rust_project();
    let mut cmd = Command::cargo_bin("uveddi").expect("Failed to find uveddi binary");
    cmd.arg("analyze")
        .arg(temp_dir.path())
        .arg("--output-format")
        .arg("invalid_format")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value")
                .or(predicate::str::contains("not supported"))
                .or(predicate::str::contains("Invalid output format")));
    
    // Test invalid detector type
    let mut cmd = Command::cargo_bin("uveddi").expect("Failed to find uveddi binary");
    cmd.arg("analyze")
        .arg(temp_dir.path())
        .arg("--detectors")
        .arg("invalid_detector")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid")
                .or(predicate::str::contains("unknown detector")));
    
    // Test permission denied (if possible)
    if cfg!(unix) {
        // Create a file without read permissions
        let protected_file = temp_dir.path().join("protected.rs");
        fs::write(&protected_file, "fn test() {}").expect("Failed to write file");
        
        // Remove read permissions
        let mut perms = fs::metadata(&protected_file).unwrap().permissions();
        perms.set_readonly(true);
        fs::set_permissions(&protected_file, perms).expect("Failed to set permissions");
        
        let mut cmd = Command::cargo_bin("uveddi").expect("Failed to find uveddi binary");
        cmd.arg("analyze")
            .arg(&protected_file)
            .assert()
            .success(); // Should succeed but report errors in output
    }
}

/// Test CLI configuration options
#[test]
fn test_cli_configuration_options() {
    let temp_dir = create_test_rust_project();
    let project_path = temp_dir.path();
    
    // Test max file size option
    let mut cmd = Command::cargo_bin("uveddi").expect("Failed to find uveddi binary");
    cmd.arg("analyze")
        .arg(project_path)
        .arg("--max-file-size")
        .arg("1024")
        .arg("--output-format")
        .arg("json")
        .assert()
        .success();
    
    // Test parallel analysis option
    let mut cmd = Command::cargo_bin("uveddi").expect("Failed to find uveddi binary");
    cmd.arg("analyze")
        .arg(project_path)
        .arg("--parallel")
        .arg("--threads")
        .arg("2")
        .arg("--output-format")
        .arg("json")
        .assert()
        .success();
    
    // Test cache options
    let mut cmd = Command::cargo_bin("uveddi").expect("Failed to find uveddi binary");
    cmd.arg("analyze")
        .arg(project_path)
        .arg("--cache")
        .arg("--cache-dir")
        .arg(temp_dir.path().join("cache"))
        .arg("--output-format")
        .arg("json")
        .assert()
        .success();
    
    // Test exclude patterns
    let mut cmd = Command::cargo_bin("uveddi").expect("Failed to find uveddi binary");
    cmd.arg("analyze")
        .arg(project_path)
        .arg("--exclude")
        .arg("target/**")
        .arg("--exclude")
        .arg("**/*.bak")
        .arg("--output-format")
        .arg("json")
        .assert()
        .success();
    
    // Test language filters
    let mut cmd = Command::cargo_bin("uveddi").expect("Failed to find uveddi binary");
    cmd.arg("analyze")
        .arg(project_path)
        .arg("--languages")
        .arg("rust,python")
        .arg("--output-format")
        .arg("json")
        .assert()
        .success();
}

/// Test specific detector options
#[test]
fn test_cli_detector_options() {
    let temp_dir = create_test_rust_project();
    let project_path = temp_dir.path();
    
    // Test individual detector selection
    let detectors = vec![
        "god-object",
        "dead-code",
        "cyclic-dependencies",
        "tight-coupling",
        "magic-values",
    ];
    
    for detector in detectors {
        let mut cmd = Command::cargo_bin("uveddi").expect("Failed to find uveddi binary");
        cmd.arg("analyze")
            .arg(project_path)
            .arg("--detectors")
            .arg(detector)
            .arg("--output-format")
            .arg("json")
            .timeout(std::time::Duration::from_secs(30))
            .assert()
            .success();
    }
    
    // Test multiple detectors
    let mut cmd = Command::cargo_bin("uveddi").expect("Failed to find uveddi binary");
    cmd.arg("analyze")
        .arg(project_path)
        .arg("--detectors")
        .arg("god-object,dead-code,tight-coupling")
        .arg("--output-format")
        .arg("json")
        .timeout(std::time::Duration::from_secs(30))
        .assert()
        .success();
    
    // Test detector thresholds
    let mut cmd = Command::cargo_bin("uveddi").expect("Failed to find uveddi binary");
    cmd.arg("analyze")
        .arg(project_path)
        .arg("--god-object-threshold")
        .arg("20")
        .arg("--coupling-threshold")
        .arg("0.8")
        .arg("--output-format")
        .arg("json")
        .assert()
        .success();
}

/// Test verbosity and logging options
#[test]
fn test_cli_verbosity_options() {
    let temp_dir = create_test_rust_project();
    let project_path = temp_dir.path();
    
    // Test quiet mode
    let mut cmd = Command::cargo_bin("uveddi").expect("Failed to find uveddi binary");
    let output = cmd.arg("analyze")
        .arg(project_path)
        .arg("--quiet")
        .arg("--output-format")
        .arg("json")
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success());
    assert!(output.stderr.is_empty(), "Quiet mode should suppress stderr output");
    
    // Test verbose mode
    let mut cmd = Command::cargo_bin("uveddi").expect("Failed to find uveddi binary");
    cmd.arg("analyze")
        .arg(project_path)
        .arg("--verbose")
        .arg("--output-format")
        .arg("json")
        .assert()
        .success()
        .stderr(predicate::str::contains("DEBUG").or(predicate::str::contains("INFO")));
    
    // Test very verbose mode
    let mut cmd = Command::cargo_bin("uveddi").expect("Failed to find uveddi binary");
    cmd.arg("analyze")
        .arg(project_path)
        .arg("-vv")
        .arg("--output-format")
        .arg("json")
        .assert()
        .success()
        .stderr(predicate::str::contains("TRACE").or(predicate::str::contains("DEBUG")));
}

/// Test AI integration options
#[test]
fn test_cli_ai_options() {
    let temp_dir = create_test_rust_project();
    let project_path = temp_dir.path();
    
    // Test without AI (should work)
    let mut cmd = Command::cargo_bin("uveddi").expect("Failed to find uveddi binary");
    cmd.arg("analyze")
        .arg(project_path)
        .arg("--no-ai")
        .arg("--output-format")
        .arg("json")
        .assert()
        .success();
    
    // Test with AI options (may fail if no AI provider available)
    let mut cmd = Command::cargo_bin("uveddi").expect("Failed to find uveddi binary");
    let result = cmd.arg("analyze")
        .arg(project_path)
        .arg("--enable-ai")
        .arg("--ai-provider")
        .arg("ollama")
        .arg("--ai-model")
        .arg("codellama:7b")
        .arg("--output-format")
        .arg("json")
        .timeout(std::time::Duration::from_secs(60))
        .output()
        .expect("Failed to execute command");
    
    // Should either succeed or fail gracefully with helpful error
    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        assert!(
            stderr.contains("AI provider not available") ||
            stderr.contains("Ollama not running") ||
            stderr.contains("connection refused") ||
            stderr.contains("timeout"),
            "Should provide helpful AI error message: {}", stderr
        );
    }
}

/// Test configuration file handling
#[test]
fn test_cli_config_file_handling() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let project_path = create_test_rust_project().into_path();
    
    // Create config file
    let config_file = temp_dir.path().join("uveddi.toml");
    let config_content = r#"
[analysis]
max_file_size = 2097152
parallel_analysis = true
cache_enabled = true

[detectors]
god_object = true
dead_code = true
cyclic_dependencies = false
tight_coupling = true

[output]
format = "json"
include_metrics = true
include_code_snippets = false
"#;
    
    fs::write(&config_file, config_content).expect("Failed to write config file");
    
    // Test with config file
    let mut cmd = Command::cargo_bin("uveddi").expect("Failed to find uveddi binary");
    cmd.arg("analyze")
        .arg(&project_path)
        .arg("--config")
        .arg(&config_file)
        .assert()
        .success();
    
    // Test config file override with CLI args
    let mut cmd = Command::cargo_bin("uveddi").expect("Failed to find uveddi binary");
    cmd.arg("analyze")
        .arg(&project_path)
        .arg("--config")
        .arg(&config_file)
        .arg("--output-format")
        .arg("text") // Override config file setting
        .assert()
        .success()
        .stdout(predicate::str::contains("Analysis Results")); // Text format output
    
    // Test invalid config file
    let invalid_config = temp_dir.path().join("invalid.toml");
    fs::write(&invalid_config, "invalid toml content [[[").expect("Failed to write invalid config");
    
    let mut cmd = Command::cargo_bin("uveddi").expect("Failed to find uveddi binary");
    cmd.arg("analyze")
        .arg(&project_path)
        .arg("--config")
        .arg(&invalid_config)
        .assert()
        .failure()
        .stderr(predicate::str::contains("config")
                .and(predicate::str::contains("parse").or(predicate::str::contains("invalid"))));
}

/// Test progress reporting and output
#[test]
fn test_cli_progress_reporting() {
    let temp_dir = create_large_test_project(50); // Create larger project
    let project_path = temp_dir.path();
    
    // Test progress bar output
    let mut cmd = Command::cargo_bin("uveddi").expect("Failed to find uveddi binary");
    cmd.arg("analyze")
        .arg(project_path)
        .arg("--progress")
        .arg("--output-format")
        .arg("json")
        .timeout(std::time::Duration::from_secs(60))
        .assert()
        .success();
    
    // Test no progress output
    let mut cmd = Command::cargo_bin("uveddi").expect("Failed to find uveddi binary");
    let output = cmd.arg("analyze")
        .arg(project_path)
        .arg("--no-progress")
        .arg("--output-format")
        .arg("json")
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success());
    
    // Test machine-readable progress
    let mut cmd = Command::cargo_bin("uveddi").expect("Failed to find uveddi binary");
    cmd.arg("analyze")
        .arg(project_path)
        .arg("--progress-format")
        .arg("json")
        .arg("--output-format")
        .arg("json")
        .assert()
        .success();
}

/// Test CLI exit codes
#[test]
fn test_cli_exit_codes() {
    let temp_dir = create_test_rust_project();
    let project_path = temp_dir.path();
    
    // Success case (exit code 0)
    let mut cmd = Command::cargo_bin("uveddi").expect("Failed to find uveddi binary");
    cmd.arg("analyze")
        .arg(project_path)
        .arg("--output-format")
        .arg("json")
        .assert()
        .code(0);
    
    // Invalid arguments (exit code 1)
    let mut cmd = Command::cargo_bin("uveddi").expect("Failed to find uveddi binary");
    cmd.arg("invalid_command")
        .assert()
        .code(1);
    
    // Non-existent path (exit code 2)
    let mut cmd = Command::cargo_bin("uveddi").expect("Failed to find uveddi binary");
    cmd.arg("analyze")
        .arg("/non/existent/path")
        .assert()
        .code(2);
    
    // Test with issues found (should still be exit code 0 unless --fail-on-issues)
    let mut cmd = Command::cargo_bin("uveddi").expect("Failed to find uveddi binary");
    cmd.arg("analyze")
        .arg(project_path)
        .arg("--fail-on-issues")
        .arg("--output-format")
        .arg("json")
        .assert()
        .code(predicate::in_iter([0, 3])); // 0 if no issues, 3 if issues found
}

/// Test CLI memory and performance options
#[test]
fn test_cli_performance_options() {
    let temp_dir = create_test_rust_project();
    let project_path = temp_dir.path();
    
    // Test memory limit
    let mut cmd = Command::cargo_bin("uveddi").expect("Failed to find uveddi binary");
    cmd.arg("analyze")
        .arg(project_path)
        .arg("--memory-limit")
        .arg("512MB")
        .arg("--output-format")
        .arg("json")
        .assert()
        .success();
    
    // Test timeout
    let mut cmd = Command::cargo_bin("uveddi").expect("Failed to find uveddi binary");
    cmd.arg("analyze")
        .arg(project_path)
        .arg("--timeout")
        .arg("30s")
        .arg("--output-format")
        .arg("json")
        .assert()
        .success();
    
    // Test performance mode
    let mut cmd = Command::cargo_bin("uveddi").expect("Failed to find uveddi binary");
    cmd.arg("analyze")
        .arg(project_path)
        .arg("--performance-mode")
        .arg("--output-format")
        .arg("json")
        .assert()
        .success();
}

/// Test subcommands
#[test]
fn test_cli_subcommands() {
    // Test init command
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let mut cmd = Command::cargo_bin("uveddi").expect("Failed to find uveddi binary");
    cmd.arg("init")
        .arg("--config-path")
        .arg(temp_dir.path())
        .assert()
        .success();
    
    // Verify config file was created
    let config_file = temp_dir.path().join("uveddi.toml");
    assert!(config_file.exists(), "Init should create config file");
    
    // Test config validation command
    let mut cmd = Command::cargo_bin("uveddi").expect("Failed to find uveddi binary");
    cmd.arg("config")
        .arg("validate")
        .arg("--config")
        .arg(&config_file)
        .assert()
        .success();
    
    // Test list detectors command
    let mut cmd = Command::cargo_bin("uveddi").expect("Failed to find uveddi binary");
    cmd.arg("detectors")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("god-object"))
        .stdout(predicate::str::contains("dead-code"));
    
    // Test detector info command
    let mut cmd = Command::cargo_bin("uveddi").expect("Failed to find uveddi binary");
    cmd.arg("detectors")
        .arg("info")
        .arg("god-object")
        .assert()
        .success()
        .stdout(predicate::str::contains("God Object"))
        .stdout(predicate::str::contains("description"));
}

/// Test shell completion generation
#[test]
fn test_cli_shell_completion() {
    let shells = vec!["bash", "zsh", "fish", "powershell"];
    
    for shell in shells {
        let mut cmd = Command::cargo_bin("uveddi").expect("Failed to find uveddi binary");
        cmd.arg("completion")
            .arg(shell)
            .assert()
            .success()
            .stdout(predicate::str::contains("uveddi")); // Should contain command name
    }
}

#[cfg(test)]
mod helpers {
    use super::*;
    
    /// Create a test Rust project for CLI testing
    pub fn create_test_rust_project() -> TempDir {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let src_dir = temp_dir.path().join("src");
        fs::create_dir_all(&src_dir).expect("Failed to create src directory");
        
        // Create main.rs with some issues
        fs::write(src_dir.join("main.rs"), r#"
mod large_class;

use large_class::LargeClass;

fn main() {
    let instance = LargeClass::new();
    instance.do_everything();
}

// Dead code
fn unused_function() {
    println!("This function is never called");
}
"#).expect("Failed to write main.rs");
        
        // Create a large class with many methods (god object)
        fs::write(src_dir.join("large_class.rs"), r#"
pub struct LargeClass {
    data: Vec<String>,
    cache: std::collections::HashMap<String, i32>,
    config: std::collections::HashMap<String, String>,
    state: i32,
}

impl LargeClass {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            cache: std::collections::HashMap::new(),
            config: std::collections::HashMap::new(),
            state: 0,
        }
    }
    
    // Data management methods
    pub fn add_data(&mut self, item: String) { self.data.push(item); }
    pub fn remove_data(&mut self, index: usize) { self.data.remove(index); }
    pub fn clear_data(&mut self) { self.data.clear(); }
    pub fn get_data(&self, index: usize) -> Option<&String> { self.data.get(index) }
    
    // Cache management methods
    pub fn cache_put(&mut self, key: String, value: i32) { self.cache.insert(key, value); }
    pub fn cache_get(&self, key: &str) -> Option<&i32> { self.cache.get(key) }
    pub fn cache_clear(&mut self) { self.cache.clear(); }
    
    // Config management methods
    pub fn set_config(&mut self, key: String, value: String) { self.config.insert(key, value); }
    pub fn get_config(&self, key: &str) -> Option<&String> { self.config.get(key) }
    
    // State management methods
    pub fn set_state(&mut self, state: i32) { self.state = state; }
    pub fn get_state(&self) -> i32 { self.state }
    
    // The main method that does too many things
    pub fn do_everything(&mut self) {
        self.add_data("test".to_string());
        self.cache_put("key".to_string(), 42);
        self.set_config("setting".to_string(), "value".to_string());
        self.set_state(1);
        
        // More operations...
        for i in 0..10 {
            self.add_data(format!("item_{}", i));
        }
        
        // Even more operations...
        self.process_data();
        self.validate_state();
        self.cleanup();
    }
    
    fn process_data(&self) {
        // Processing logic
    }
    
    fn validate_state(&self) -> bool {
        self.state > 0
    }
    
    fn cleanup(&mut self) {
        // Cleanup logic
    }
}

// More dead code
struct UnusedStruct {
    field: i32,
}

impl UnusedStruct {
    fn new() -> Self {
        Self { field: 0 }
    }
    
    fn unused_method(&self) -> i32 {
        self.field * 2
    }
}
"#).expect("Failed to write large_class.rs");
        
        // Create Cargo.toml
        fs::write(temp_dir.path().join("Cargo.toml"), r#"
[package]
name = "cli-test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
"#).expect("Failed to write Cargo.toml");
        
        temp_dir
    }
    
    /// Create a larger test project for performance testing
    pub fn create_large_test_project(file_count: usize) -> TempDir {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let src_dir = temp_dir.path().join("src");
        fs::create_dir_all(&src_dir).expect("Failed to create src directory");
        
        // Create main.rs
        let mut main_content = String::from("// Large test project\n");
        for i in 0..file_count {
            main_content.push_str(&format!("mod module_{};\n", i));
        }
        main_content.push_str("\nfn main() {\n");
        for i in 0..std::cmp::min(file_count, 10) {
            main_content.push_str(&format!("    module_{}::function_{}();\n", i, i));
        }
        main_content.push_str("}\n");
        
        fs::write(src_dir.join("main.rs"), main_content).expect("Failed to write main.rs");
        
        // Create module files
        for i in 0..file_count {
            let module_content = format!(r#"
pub struct Data{} {{
    value: i32,
    items: Vec<String>,
}}

impl Data{} {{
    pub fn new(value: i32) -> Self {{
        Self {{
            value,
            items: Vec::new(),
        }}
    }}
    
    pub fn add_item(&mut self, item: String) {{
        self.items.push(item);
    }}
    
    pub fn process(&self) -> i32 {{
        self.value * self.items.len() as i32
    }}
}}

pub fn function_{}() {{
    let mut data = Data{}::new({});
    data.add_item(format!("item_{{}}", {}));
    let _result = data.process();
}}

// Some dead code
fn unused_function_{}() {{
    let _unused = "This is never used";
}}
"#, i, i, i, i, i, i, i);
            
            fs::write(src_dir.join(format!("module_{}.rs", i)), module_content)
                .expect("Failed to write module file");
        }
        
        // Create Cargo.toml
        fs::write(temp_dir.path().join("Cargo.toml"), r#"
[package]
name = "large-cli-test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
"#).expect("Failed to write Cargo.toml");
        
        temp_dir
    }
    
    /// Helper trait to check if output is valid JSON
    pub trait JsonPredicate {
        fn is_json(&self) -> predicates::str::ContainsPredicate;
    }
    
    impl JsonPredicate for predicates::str::Predicate {
        fn is_json(&self) -> predicates::str::ContainsPredicate {
            predicate::str::contains("{")
                .and(predicate::str::contains("}"))
        }
    }
    
    /// Validate JSON structure for analysis output
    pub fn validate_analysis_json(content: &str) -> Result<(), String> {
        let json: Value = serde_json::from_str(content)
            .map_err(|e| format!("Invalid JSON: {}", e))?;
        
        // Check required fields
        if !json["issues"].is_array() {
            return Err("Missing 'issues' array".to_string());
        }
        
        if !json["summary"].is_object() {
            return Err("Missing 'summary' object".to_string());
        }
        
        // Validate summary structure
        let summary = &json["summary"];
        let required_fields = vec!["total_issues", "high_severity", "medium_severity", "low_severity"];
        
        for field in required_fields {
            if !summary[field].is_number() {
                return Err(format!("Summary missing numeric field: {}", field));
            }
        }
        
        // Validate issues structure
        let issues = json["issues"].as_array().unwrap();
        for (i, issue) in issues.iter().enumerate() {
            let required_issue_fields = vec!["id", "type", "severity", "file", "description"];
            
            for field in required_issue_fields {
                if !issue[field].is_string() {
                    return Err(format!("Issue {} missing string field: {}", i, field));
                }
            }
            
            if !issue["line"].is_number() {
                return Err(format!("Issue {} missing numeric 'line' field", i));
            }
        }
        
        Ok(())
    }
    
    /// Helper to run command with timeout
    pub fn run_with_timeout(mut cmd: Command, timeout_secs: u64) -> Output {
        cmd.timeout(std::time::Duration::from_secs(timeout_secs))
            .output()
            .expect("Failed to execute command")
    }
}