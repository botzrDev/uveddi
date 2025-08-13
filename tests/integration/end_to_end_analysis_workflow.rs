//! End-to-end analysis workflow integration testing
//! 
//! This module tests complete analysis workflows from file discovery
//! through report generation, ensuring all components work together correctly.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tempfile::TempDir;
use tokio::fs;
use uveddi::analysis::{
    AnalysisEngine,
    config::AnalysisConfig,
    engine_builder::AnalysisEngineBuilder,
    detectors::DetectorType,
    types::{AnalysisResult, ArchitecturalIssue, IssueType, Severity},
};
use uveddi::analysis::graph::dependency::DependencyGraph;
use uveddi::report::{
    ReportGenerator,
    ReportFormat,
    ReportConfig,
};
use uveddi::ast::AstProvider;
use uveddi::ai::OllamaProvider;

/// Test complete analysis workflow for a Rust project
#[tokio::test]
async fn test_rust_project_analysis_workflow() {
    let temp_dir = create_test_rust_project().await;
    let project_path = temp_dir.path();
    
    // Configure analysis engine
    let config = AnalysisConfig {
        max_file_size: 1024 * 1024, // 1MB
        parallel_analysis: true,
        enable_ai_explanations: false, // Disable AI for faster testing
        cache_enabled: true,
        detector_types: vec![
            DetectorType::GodObject,
            DetectorType::DeadCode,
            DetectorType::CyclicDependencies,
            DetectorType::TightCoupling,
        ],
        language_filters: vec!["rust".to_string()],
        exclude_patterns: vec!["target/**".to_string()],
    };
    
    // Build and run analysis
    let engine = AnalysisEngineBuilder::new()
        .with_config(config)
        .build()
        .expect("Failed to build analysis engine");
    
    let (issues, dependency_graph) = engine.analyze(project_path).await
        .expect("Analysis failed");
    
    // Verify analysis results
    assert!(!issues.is_empty(), "Should detect some architectural issues");
    assert!(dependency_graph.node_count() > 0, "Should build dependency graph");
    
    // Check for expected issue types
    let issue_types: Vec<IssueType> = issues.iter().map(|i| i.issue_type.clone()).collect();
    assert!(issue_types.contains(&IssueType::GodObject), "Should detect god object");
    assert!(issue_types.contains(&IssueType::TightCoupling), "Should detect tight coupling");
    
    // Verify god object detection
    let god_object_issues: Vec<&ArchitecturalIssue> = issues.iter()
        .filter(|i| i.issue_type == IssueType::GodObject)
        .collect();
    
    assert!(!god_object_issues.is_empty(), "Should find god object");
    let god_object = &god_object_issues[0];
    assert!(god_object.file_path.ends_with("god_object.rs"));
    assert!(god_object.severity >= Severity::Medium);
    assert!(!god_object.description.is_empty());
    assert!(god_object.metrics.lines_of_code > 100);
    
    // Verify dependency graph structure
    assert!(dependency_graph.has_cycles(), "Test project should have cycles");
    let cycles = dependency_graph.find_cycles();
    assert!(!cycles.is_empty(), "Should detect dependency cycles");
    
    // Test report generation
    let report_config = ReportConfig {
        format: ReportFormat::Json,
        include_metrics: true,
        include_code_snippets: true,
        include_dependency_graph: true,
        output_path: Some(project_path.join("analysis_report.json")),
    };
    
    let report_generator = ReportGenerator::new(report_config);
    let report_path = report_generator.generate_report(&issues, &dependency_graph).await
        .expect("Report generation failed");
    
    // Verify report was created
    assert!(report_path.exists(), "Report file should exist");
    
    // Verify report content
    let report_content = fs::read_to_string(&report_path).await
        .expect("Failed to read report");
    let report_json: serde_json::Value = serde_json::from_str(&report_content)
        .expect("Invalid JSON report");
    
    assert!(report_json["issues"].is_array());
    assert!(report_json["dependency_graph"].is_object());
    assert!(report_json["summary"]["total_issues"].as_u64().unwrap() > 0);
}

/// Test multi-language project analysis
#[tokio::test]
async fn test_multi_language_analysis_workflow() {
    let temp_dir = create_multi_language_project().await;
    let project_path = temp_dir.path();
    
    // Configure for multi-language analysis
    let config = AnalysisConfig {
        max_file_size: 1024 * 1024,
        parallel_analysis: true,
        enable_ai_explanations: false,
        cache_enabled: true,
        detector_types: vec![
            DetectorType::GodObject,
            DetectorType::DeadCode,
            DetectorType::MagicValues,
        ],
        language_filters: vec!["rust".to_string(), "python".to_string(), "javascript".to_string()],
        exclude_patterns: vec!["node_modules/**".to_string(), "__pycache__/**".to_string()],
    };
    
    let engine = AnalysisEngineBuilder::new()
        .with_config(config)
        .build()
        .expect("Failed to build analysis engine");
    
    let (issues, dependency_graph) = engine.analyze(project_path).await
        .expect("Multi-language analysis failed");
    
    // Verify issues from different languages
    let rust_issues: Vec<&ArchitecturalIssue> = issues.iter()
        .filter(|i| i.file_path.ends_with(".rs"))
        .collect();
    let python_issues: Vec<&ArchitecturalIssue> = issues.iter()
        .filter(|i| i.file_path.ends_with(".py"))
        .collect();
    let js_issues: Vec<&ArchitecturalIssue> = issues.iter()
        .filter(|i| i.file_path.ends_with(".js"))
        .collect();
    
    assert!(!rust_issues.is_empty(), "Should find Rust issues");
    assert!(!python_issues.is_empty(), "Should find Python issues");
    assert!(!js_issues.is_empty(), "Should find JavaScript issues");
    
    // Verify language-specific detections
    let magic_value_issues: Vec<&ArchitecturalIssue> = issues.iter()
        .filter(|i| i.issue_type == IssueType::MagicValues)
        .collect();
    assert!(!magic_value_issues.is_empty(), "Should detect magic values across languages");
    
    // Verify dependency graph spans languages
    let nodes: Vec<&str> = dependency_graph.nodes()
        .map(|n| n.file_path.as_str())
        .collect();
    
    assert!(nodes.iter().any(|n| n.ends_with(".rs")));
    assert!(nodes.iter().any(|n| n.ends_with(".py")));
    assert!(nodes.iter().any(|n| n.ends_with(".js")));
}

/// Test incremental analysis workflow
#[tokio::test]
async fn test_incremental_analysis_workflow() {
    let temp_dir = create_test_rust_project().await;
    let project_path = temp_dir.path();
    
    let config = AnalysisConfig {
        max_file_size: 1024 * 1024,
        parallel_analysis: true,
        enable_ai_explanations: false,
        cache_enabled: true,
        incremental_mode: true,
        detector_types: vec![DetectorType::GodObject, DetectorType::DeadCode],
        language_filters: vec!["rust".to_string()],
        exclude_patterns: vec![],
    };
    
    let engine = AnalysisEngineBuilder::new()
        .with_config(config.clone())
        .build()
        .expect("Failed to build analysis engine");
    
    // First analysis run
    let (initial_issues, initial_graph) = engine.analyze(project_path).await
        .expect("Initial analysis failed");
    
    let initial_issue_count = initial_issues.len();
    assert!(initial_issue_count > 0, "Should find issues in initial run");
    
    // Modify a file to introduce new issues
    let new_god_object = project_path.join("src/new_god.rs");
    fs::write(&new_god_object, create_god_object_content()).await
        .expect("Failed to write new god object");
    
    // Second analysis run (incremental)
    let (updated_issues, updated_graph) = engine.analyze(project_path).await
        .expect("Incremental analysis failed");
    
    // Should detect new issues
    assert!(updated_issues.len() > initial_issue_count, 
           "Incremental analysis should detect new issues");
    
    // Verify new god object is detected
    let new_god_issues: Vec<&ArchitecturalIssue> = updated_issues.iter()
        .filter(|i| i.file_path == new_god_object && i.issue_type == IssueType::GodObject)
        .collect();
    assert!(!new_god_issues.is_empty(), "Should detect new god object");
    
    // Dependency graph should be updated
    assert!(updated_graph.node_count() > initial_graph.node_count(),
           "Dependency graph should include new file");
}

/// Test analysis with AI explanations
#[tokio::test]
async fn test_ai_enhanced_analysis_workflow() {
    // Skip if no AI provider is available
    if std::env::var("OLLAMA_HOST").is_err() {
        return;
    }
    
    let temp_dir = create_test_rust_project().await;
    let project_path = temp_dir.path();
    
    let config = AnalysisConfig {
        max_file_size: 1024 * 1024,
        parallel_analysis: false, // Disable for AI to avoid rate limits
        enable_ai_explanations: true,
        cache_enabled: true,
        detector_types: vec![DetectorType::GodObject],
        language_filters: vec!["rust".to_string()],
        exclude_patterns: vec![],
        ai_config: Some(AiConfig {
            provider: "ollama".to_string(),
            model: "codellama:7b".to_string(),
            temperature: 0.3,
            max_tokens: 1000,
        }),
    };
    
    let engine = AnalysisEngineBuilder::new()
        .with_config(config)
        .with_ai_provider(Arc::new(OllamaProvider::new().await
            .expect("Failed to create Ollama provider")))
        .build()
        .expect("Failed to build analysis engine");
    
    let (issues, _) = engine.analyze(project_path).await
        .expect("AI-enhanced analysis failed");
    
    // Verify AI explanations are included
    let god_object_issues: Vec<&ArchitecturalIssue> = issues.iter()
        .filter(|i| i.issue_type == IssueType::GodObject)
        .collect();
    
    assert!(!god_object_issues.is_empty(), "Should detect god object");
    
    for issue in god_object_issues {
        assert!(issue.ai_explanation.is_some(), "Should have AI explanation");
        let explanation = issue.ai_explanation.as_ref().unwrap();
        assert!(!explanation.is_empty(), "AI explanation should not be empty");
        assert!(explanation.len() > 50, "AI explanation should be substantial");
    }
}

/// Test error handling in analysis workflow
#[tokio::test]
async fn test_analysis_error_handling_workflow() {
    let temp_dir = create_problematic_project().await;
    let project_path = temp_dir.path();
    
    let config = AnalysisConfig {
        max_file_size: 1024, // Very small limit to trigger errors
        parallel_analysis: true,
        enable_ai_explanations: false,
        cache_enabled: true,
        detector_types: vec![DetectorType::GodObject],
        language_filters: vec!["rust".to_string()],
        exclude_patterns: vec![],
        error_handling: ErrorHandlingMode::ContinueOnError,
    };
    
    let engine = AnalysisEngineBuilder::new()
        .with_config(config)
        .build()
        .expect("Failed to build analysis engine");
    
    let (issues, dependency_graph) = engine.analyze(project_path).await
        .expect("Analysis should succeed despite individual file errors");
    
    // Should have some successful analyses despite errors
    assert!(!issues.is_empty() || dependency_graph.node_count() > 0,
           "Should have some successful analysis results");
    
    // Check error reporting
    let error_summary = engine.get_error_summary();
    assert!(error_summary.file_errors > 0, "Should report file errors");
    assert!(error_summary.skipped_files.len() > 0, "Should report skipped files");
    
    // Verify specific error types
    assert!(error_summary.errors.iter().any(|e| 
        e.error_type == ErrorType::FileTooLarge
    ), "Should report file size errors");
}

/// Test performance with large codebase
#[tokio::test]
async fn test_large_codebase_performance() {
    let temp_dir = create_large_test_project(1000).await; // 1000 files
    let project_path = temp_dir.path();
    
    let config = AnalysisConfig {
        max_file_size: 1024 * 1024,
        parallel_analysis: true,
        enable_ai_explanations: false,
        cache_enabled: true,
        detector_types: vec![DetectorType::GodObject, DetectorType::DeadCode],
        language_filters: vec!["rust".to_string()],
        exclude_patterns: vec![],
        performance_mode: true,
    };
    
    let engine = AnalysisEngineBuilder::new()
        .with_config(config)
        .build()
        .expect("Failed to build analysis engine");
    
    let start_time = std::time::Instant::now();
    let (issues, dependency_graph) = engine.analyze(project_path).await
        .expect("Large codebase analysis failed");
    let analysis_time = start_time.elapsed();
    
    // Performance assertions
    assert!(analysis_time.as_secs() < 60, 
           "Analysis took too long: {:?}", analysis_time);
    
    // Should process most files
    assert!(dependency_graph.node_count() > 800, 
           "Should process most files: {}", dependency_graph.node_count());
    
    // Should find proportional number of issues
    assert!(issues.len() > 50, 
           "Should find reasonable number of issues: {}", issues.len());
    
    // Memory usage should be reasonable
    let memory_usage = engine.get_memory_usage();
    assert!(memory_usage < 2 * 1024 * 1024 * 1024, // 2GB
           "Memory usage too high: {} bytes", memory_usage);
}

/// Test analysis configuration validation
#[tokio::test]
async fn test_configuration_validation_workflow() {
    let temp_dir = create_test_rust_project().await;
    let project_path = temp_dir.path();
    
    // Test invalid configuration
    let invalid_config = AnalysisConfig {
        max_file_size: 0, // Invalid
        parallel_analysis: true,
        enable_ai_explanations: false,
        cache_enabled: true,
        detector_types: vec![], // Empty detectors
        language_filters: vec!["unsupported_language".to_string()], // Invalid language
        exclude_patterns: vec!["[invalid_regex".to_string()], // Invalid regex
    };
    
    let result = AnalysisEngineBuilder::new()
        .with_config(invalid_config)
        .build();
    
    assert!(result.is_err(), "Should reject invalid configuration");
    
    let error = result.unwrap_err();
    assert!(error.to_string().contains("max_file_size"));
    assert!(error.to_string().contains("detector_types"));
    
    // Test valid configuration
    let valid_config = AnalysisConfig {
        max_file_size: 1024 * 1024,
        parallel_analysis: true,
        enable_ai_explanations: false,
        cache_enabled: true,
        detector_types: vec![DetectorType::GodObject],
        language_filters: vec!["rust".to_string()],
        exclude_patterns: vec!["target/**".to_string()],
    };
    
    let engine = AnalysisEngineBuilder::new()
        .with_config(valid_config)
        .build()
        .expect("Should accept valid configuration");
    
    let (issues, _) = engine.analyze(project_path).await
        .expect("Analysis with valid config should succeed");
    
    assert!(!issues.is_empty(), "Should find issues with valid config");
}

#[cfg(test)]
mod helpers {
    use super::*;
    
    /// Create a test Rust project with known architectural issues
    pub async fn create_test_rust_project() -> TempDir {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let src_dir = temp_dir.path().join("src");
        fs::create_dir_all(&src_dir).await.expect("Failed to create src directory");
        
        // Main file
        fs::write(src_dir.join("main.rs"), r#"
mod god_object;
mod utils;
mod circular_a;

use god_object::GodObject;

fn main() {
    let god = GodObject::new();
    god.do_everything();
}
"#).await.expect("Failed to write main.rs");
        
        // God object with many responsibilities
        fs::write(src_dir.join("god_object.rs"), create_god_object_content()).await
            .expect("Failed to write god_object.rs");
        
        // Utility module with dead code
        fs::write(src_dir.join("utils.rs"), r#"
pub fn used_function() -> i32 {
    42
}

pub fn unused_function() -> String {
    "This function is never called".to_string()
}

fn private_unused() {
    // This is dead code
}

pub struct UnusedStruct {
    field: i32,
}
"#).await.expect("Failed to write utils.rs");
        
        // Circular dependency A
        fs::write(src_dir.join("circular_a.rs"), r#"
use crate::circular_b::B;

pub struct A {
    b_ref: B,
}

impl A {
    pub fn new() -> Self {
        A { b_ref: B::new() }
    }
}
"#).await.expect("Failed to write circular_a.rs");
        
        // Circular dependency B
        fs::write(src_dir.join("circular_b.rs"), r#"
use crate::circular_a::A;

pub struct B {
    value: i32,
}

impl B {
    pub fn new() -> Self {
        B { value: 0 }
    }
    
    pub fn create_a(&self) -> A {
        A::new() // This creates a circular dependency
    }
}
"#).await.expect("Failed to write circular_b.rs");
        
        // Cargo.toml
        fs::write(temp_dir.path().join("Cargo.toml"), r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
"#).await.expect("Failed to write Cargo.toml");
        
        temp_dir
    }
    
    /// Create god object content with many methods and responsibilities
    pub fn create_god_object_content() -> &'static str {
        r#"
pub struct GodObject {
    // Data management
    data: Vec<String>,
    cache: std::collections::HashMap<String, i32>,
    
    // UI state
    window_size: (u32, u32),
    theme: String,
    
    // Network state
    connections: Vec<String>,
    
    // File system state
    current_dir: std::path::PathBuf,
    open_files: Vec<std::fs::File>,
    
    // Configuration
    settings: std::collections::HashMap<String, String>,
}

impl GodObject {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            cache: std::collections::HashMap::new(),
            window_size: (800, 600),
            theme: "dark".to_string(),
            connections: Vec::new(),
            current_dir: std::path::PathBuf::from("."),
            open_files: Vec::new(),
            settings: std::collections::HashMap::new(),
        }
    }
    
    // Data management methods (should be separate module)
    pub fn add_data(&mut self, item: String) { self.data.push(item); }
    pub fn remove_data(&mut self, index: usize) { self.data.remove(index); }
    pub fn clear_data(&mut self) { self.data.clear(); }
    pub fn get_data(&self, index: usize) -> Option<&String> { self.data.get(index) }
    pub fn search_data(&self, query: &str) -> Vec<usize> {
        self.data.iter().enumerate()
            .filter(|(_, item)| item.contains(query))
            .map(|(i, _)| i)
            .collect()
    }
    
    // Cache management methods (should be separate module)
    pub fn cache_put(&mut self, key: String, value: i32) { self.cache.insert(key, value); }
    pub fn cache_get(&self, key: &str) -> Option<&i32> { self.cache.get(key) }
    pub fn cache_clear(&mut self) { self.cache.clear(); }
    pub fn cache_size(&self) -> usize { self.cache.len() }
    
    // UI methods (should be separate module)
    pub fn set_window_size(&mut self, width: u32, height: u32) {
        self.window_size = (width, height);
    }
    pub fn get_window_size(&self) -> (u32, u32) { self.window_size }
    pub fn set_theme(&mut self, theme: String) { self.theme = theme; }
    pub fn get_theme(&self) -> &str { &self.theme }
    pub fn render_ui(&self) -> String { format!("Rendering UI with theme: {}", self.theme) }
    
    // Network methods (should be separate module)
    pub fn connect(&mut self, address: String) { self.connections.push(address); }
    pub fn disconnect(&mut self, address: &str) {
        self.connections.retain(|conn| conn != address);
    }
    pub fn send_data(&self, address: &str, data: &str) -> Result<(), String> {
        if self.connections.contains(&address.to_string()) {
            Ok(())
        } else {
            Err("Not connected".to_string())
        }
    }
    
    // File system methods (should be separate module)
    pub fn change_directory(&mut self, path: std::path::PathBuf) {
        self.current_dir = path;
    }
    pub fn get_current_directory(&self) -> &std::path::Path { &self.current_dir }
    pub fn list_files(&self) -> Result<Vec<String>, std::io::Error> {
        std::fs::read_dir(&self.current_dir)?
            .map(|entry| entry.map(|e| e.file_name().to_string_lossy().to_string()))
            .collect()
    }
    
    // Configuration methods (should be separate module)
    pub fn set_setting(&mut self, key: String, value: String) {
        self.settings.insert(key, value);
    }
    pub fn get_setting(&self, key: &str) -> Option<&String> { self.settings.get(key) }
    pub fn load_config(&mut self, _path: &str) -> Result<(), String> { Ok(()) }
    pub fn save_config(&self, _path: &str) -> Result<(), String> { Ok(()) }
    
    // The main method that tries to do everything
    pub fn do_everything(&mut self) {
        self.add_data("test".to_string());
        self.cache_put("key".to_string(), 42);
        self.set_theme("light".to_string());
        self.connect("localhost:8080".to_string());
        self.change_directory(std::path::PathBuf::from("/tmp"));
        self.set_setting("verbose".to_string(), "true".to_string());
        
        // More complex operations that span multiple responsibilities
        if let Ok(files) = self.list_files() {
            for file in files {
                self.add_data(file);
            }
        }
        
        // UI rendering based on data
        let _ = self.render_ui();
        
        // Network operations based on cache
        for (key, _) in &self.cache {
            let _ = self.send_data("localhost:8080", key);
        }
    }
}
"#
    }
    
    /// Create a multi-language project
    pub async fn create_multi_language_project() -> TempDir {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        
        // Rust code
        let rust_dir = temp_dir.path().join("rust_src");
        fs::create_dir_all(&rust_dir).await.expect("Failed to create rust directory");
        
        fs::write(rust_dir.join("main.rs"), r#"
const MAGIC_NUMBER: i32 = 42; // Magic value
const ANOTHER_MAGIC: f64 = 3.14159; // Another magic value

fn main() {
    println!("Magic numbers: {}, {}", MAGIC_NUMBER, ANOTHER_MAGIC);
}
"#).await.expect("Failed to write Rust file");
        
        // Python code
        let python_dir = temp_dir.path().join("python_src");
        fs::create_dir_all(&python_dir).await.expect("Failed to create python directory");
        
        fs::write(python_dir.join("main.py"), r#"
# God object in Python
class EverythingManager:
    def __init__(self):
        self.data = []
        self.cache = {}
        self.config = {}
        self.ui_state = {}
        self.network_connections = []
    
    def add_data(self, item):
        self.data.append(item)
    
    def cache_item(self, key, value):
        self.cache[key] = value
    
    def set_config(self, key, value):
        self.config[key] = value
    
    def update_ui(self, state):
        self.ui_state.update(state)
    
    def connect_network(self, host):
        self.network_connections.append(host)
    
    def process_everything(self):
        # Magic numbers everywhere
        for i in range(100):  # Magic number
            self.add_data(f"item_{i}")
            if i % 7 == 0:  # Magic number
                self.cache_item(f"cache_{i}", i * 3.14159)  # Magic number

def unused_function():
    """This function is never called"""
    return 999  # Magic number

if __name__ == "__main__":
    manager = EverythingManager()
    manager.process_everything()
"#).await.expect("Failed to write Python file");
        
        // JavaScript code
        let js_dir = temp_dir.path().join("js_src");
        fs::create_dir_all(&js_dir).await.expect("Failed to create js directory");
        
        fs::write(js_dir.join("main.js"), r#"
// God object in JavaScript
class UniversalController {
    constructor() {
        this.state = {};
        this.cache = new Map();
        this.eventHandlers = {};
        this.networkConfig = {};
        this.uiComponents = [];
    }
    
    // Data management
    setState(key, value) {
        this.state[key] = value;
    }
    
    // Cache management
    cacheData(key, data) {
        this.cache.set(key, data);
    }
    
    // Event handling
    addEventListener(event, handler) {
        if (!this.eventHandlers[event]) {
            this.eventHandlers[event] = [];
        }
        this.eventHandlers[event].push(handler);
    }
    
    // Network operations
    configureNetwork(host, port = 8080) { // Magic number
        this.networkConfig = { host, port };
    }
    
    // UI management
    addComponent(component) {
        this.uiComponents.push(component);
    }
    
    // Main processing method
    processAll() {
        // Magic numbers everywhere
        const BATCH_SIZE = 50; // Magic number
        const TIMEOUT = 5000; // Magic number
        const MAX_RETRIES = 3; // Magic number
        
        for (let i = 0; i < BATCH_SIZE; i++) {
            this.setState(`item_${i}`, i * 2.718); // Magic number
            
            if (i % 10 === 0) { // Magic number
                this.cacheData(`batch_${i}`, { processed: true, timestamp: Date.now() });
            }
        }
        
        setTimeout(() => {
            this.cleanup();
        }, TIMEOUT);
    }
    
    cleanup() {
        // This method does too many things
        this.state = {};
        this.cache.clear();
        this.eventHandlers = {};
        this.networkConfig = {};
        this.uiComponents = [];
    }
}

// Dead code
function neverUsedFunction() {
    const DEAD_MAGIC = 404; // Magic number in dead code
    return DEAD_MAGIC;
}

// Another dead class
class DeadClass {
    constructor() {
        this.value = 123; // Magic number
    }
}

const controller = new UniversalController();
controller.processAll();
"#).await.expect("Failed to write JavaScript file");
        
        temp_dir
    }
    
    /// Create a project with problematic files for error testing
    pub async fn create_problematic_project() -> TempDir {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let src_dir = temp_dir.path().join("src");
        fs::create_dir_all(&src_dir).await.expect("Failed to create src directory");
        
        // Large file that exceeds size limit
        let large_content = "fn large_function() {\n".to_string() + 
            &"    println!(\"line\");\n".repeat(1000) + 
            "}\n";
        fs::write(src_dir.join("large_file.rs"), large_content).await
            .expect("Failed to write large file");
        
        // File with syntax errors
        fs::write(src_dir.join("syntax_error.rs"), r#"
fn broken_syntax( {
    let x = ;
    missing_semicolon()
    return "unclosed string
}
"#).await.expect("Failed to write syntax error file");
        
        // Binary file (should be skipped)
        fs::write(src_dir.join("binary.bin"), &[0x00, 0x01, 0x02, 0x03, 0xFF, 0xFE]).await
            .expect("Failed to write binary file");
        
        // Valid file for comparison
        fs::write(src_dir.join("valid.rs"), r#"
fn valid_function() -> i32 {
    42
}
"#).await.expect("Failed to write valid file");
        
        temp_dir
    }
    
    /// Create a large test project with specified number of files
    pub async fn create_large_test_project(file_count: usize) -> TempDir {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let src_dir = temp_dir.path().join("src");
        fs::create_dir_all(&src_dir).await.expect("Failed to create src directory");
        
        // Create main.rs with module declarations
        let mut main_content = String::from("// Auto-generated large project\n");
        for i in 0..file_count {
            main_content.push_str(&format!("mod module_{};\n", i));
        }
        main_content.push_str("\nfn main() {\n");
        for i in 0..std::cmp::min(file_count, 100) {
            main_content.push_str(&format!("    module_{}::function_{}();\n", i, i));
        }
        main_content.push_str("}\n");
        
        fs::write(src_dir.join("main.rs"), main_content).await
            .expect("Failed to write main.rs");
        
        // Create individual module files
        for i in 0..file_count {
            let module_content = format!(r#"
// Module {} - auto-generated
use std::collections::HashMap;

pub struct Data{} {{
    items: Vec<String>,
    metadata: HashMap<String, i32>,
}}

impl Data{} {{
    pub fn new() -> Self {{
        Self {{
            items: Vec::new(),
            metadata: HashMap::new(),
        }}
    }}
    
    pub fn add_item(&mut self, item: String) {{
        self.items.push(item);
        self.metadata.insert(format!("item_{{}}", self.items.len()), {});
    }}
    
    pub fn get_items(&self) -> &Vec<String> {{
        &self.items
    }}
    
    pub fn process_data(&self) -> i32 {{
        let magic_number = {}; // Magic value
        self.items.len() as i32 * magic_number
    }}
}}

pub fn function_{}() {{
    let mut data = Data{}::new();
    data.add_item(format!("test_item_{{}}_{{}}", {}, {}));
    let _result = data.process_data();
}}

// Some dead code
fn unused_function_{}() {{
    let _unused_var = "This is never used";
}}
"#, i, i, i, (i * 7) % 100, i, i, i, i, i, i);
            
            fs::write(src_dir.join(format!("module_{}.rs", i)), module_content).await
                .expect("Failed to write module file");
        }
        
        // Create Cargo.toml
        fs::write(temp_dir.path().join("Cargo.toml"), r#"
[package]
name = "large-test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
"#).await.expect("Failed to write Cargo.toml");
        
        temp_dir
    }
    
    #[derive(Debug, Clone)]
    pub struct AiConfig {
        pub provider: String,
        pub model: String,
        pub temperature: f32,
        pub max_tokens: usize,
    }
    
    #[derive(Debug, Clone)]
    pub enum ErrorHandlingMode {
        StopOnError,
        ContinueOnError,
        CollectErrors,
    }
    
    #[derive(Debug, Clone)]
    pub enum ErrorType {
        FileTooLarge,
        SyntaxError,
        ParseError,
        IoError,
        Timeout,
    }
    
    #[derive(Debug)]
    pub struct ErrorSummary {
        pub file_errors: usize,
        pub skipped_files: Vec<PathBuf>,
        pub errors: Vec<AnalysisError>,
    }
    
    #[derive(Debug)]
    pub struct AnalysisError {
        pub file_path: PathBuf,
        pub error_type: ErrorType,
        pub message: String,
    }
}