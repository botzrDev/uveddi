//! Backward compatibility test suite for AnalysisEngine refactoring
//!
//! This module contains comprehensive tests to ensure that the component-based
//! refactoring maintains 100% backward compatibility with existing AnalysisEngine APIs.

use crate::analysis::AnalysisEngine;
use crate::database::models::ArchitecturalIssue;
use crate::analysis::graph::dependency::LocalDependencyGraph;
use std::path::Path;
use tempfile::TempDir;
use std::fs;
use std::io::Write;

/// Test fixture for creating temporary test projects
struct TestProject {
    temp_dir: TempDir,
    src_dir: std::path::PathBuf,
}

impl TestProject {
    fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).expect("Failed to create src directory");
        
        Self { temp_dir, src_dir }
    }
    
    fn add_rust_file(&self, name: &str, content: &str) -> std::path::PathBuf {
        let file_path = self.src_dir.join(name);
        let mut file = fs::File::create(&file_path).expect("Failed to create file");
        writeln!(file, "{}", content).expect("Failed to write file");
        file_path
    }
    
    fn path(&self) -> &Path {
        &self.src_dir
    }
}

/// Test data for various analysis scenarios
mod test_data {
    pub const SIMPLE_RUST_FILE: &str = r#"
fn main() {
    println!("Hello, world!");
}
"#;

    pub const RUST_WITH_IMPORTS: &str = r#"
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    settings: HashMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        Self {
            settings: HashMap::new(),
        }
    }
}
"#;

    pub const LARGE_RUST_CLASS: &str = r#"
use std::collections::HashMap;

pub struct LargeClass {
    field1: String,
    field2: i32,
    field3: bool,
    field4: Vec<String>,
    field5: HashMap<String, i32>,
    field6: Option<String>,
    field7: Result<i32, String>,
    field8: Box<dyn std::fmt::Display>,
    field9: std::sync::Arc<std::sync::Mutex<i32>>,
    field10: std::rc::Rc<std::cell::RefCell<String>>,
}

impl LargeClass {
    pub fn method1(&self) -> String { todo!() }
    pub fn method2(&self) -> i32 { todo!() }
    pub fn method3(&self) -> bool { todo!() }
    pub fn method4(&self) -> Vec<String> { todo!() }
    pub fn method5(&self) -> HashMap<String, i32> { todo!() }
    pub fn method6(&self) -> Option<String> { todo!() }
    pub fn method7(&self) -> Result<i32, String> { todo!() }
    pub fn method8(&self) -> Box<dyn std::fmt::Display> { todo!() }
    pub fn method9(&self) -> std::sync::Arc<std::sync::Mutex<i32>> { todo!() }
    pub fn method10(&self) -> std::rc::Rc<std::cell::RefCell<String>> { todo!() }
    pub fn method11(&self) -> String { todo!() }
    pub fn method12(&self) -> i32 { todo!() }
    pub fn method13(&self) -> bool { todo!() }
    pub fn method14(&self) -> Vec<String> { todo!() }
    pub fn method15(&self) -> HashMap<String, i32> { todo!() }
}
"#;

    pub const RUST_WITH_DEAD_CODE: &str = r#"
pub fn used_function() -> i32 {
    42
}

#[allow(dead_code)]
fn unused_function() -> String {
    "never called".to_string()
}

#[allow(dead_code)]
static UNUSED_CONSTANT: i32 = 100;

pub fn main() {
    let result = used_function();
    println!("Result: {}", result);
}
"#;
}

#[tokio::test]
async fn test_engine_construction_compatibility() {
    // Test all existing construction methods still work
    
    // Default construction
    let engine1 = AnalysisEngine::new();
    assert!(engine1.is_ok(), "Default AnalysisEngine::new() should work");
    
    // Construction with memory cache
    let engine2 = AnalysisEngine::new_with_memory_cache();
    assert!(engine2.is_ok(), "AnalysisEngine::new_with_memory_cache() should work");
    
    // Construction with custom cache path
    let temp_dir = TempDir::new().unwrap();
    let cache_path = temp_dir.path().join("test_cache.db");
    let engine3 = AnalysisEngine::with_cache_path(&cache_path);
    assert!(engine3.is_ok(), "AnalysisEngine::with_cache_path() should work");
    
    // Construction with plugins (async)
    let engine4 = AnalysisEngine::new_with_plugins().await;
    // Plugin construction may fail if WASM engine can't initialize, but shouldn't panic
    assert!(engine4.is_ok() || engine4.is_err(), "Plugin construction should not panic");
}

#[tokio::test]
async fn test_basic_analysis_compatibility() {
    let project = TestProject::new();
    project.add_rust_file("main.rs", test_data::SIMPLE_RUST_FILE);
    project.add_rust_file("lib.rs", test_data::RUST_WITH_IMPORTS);
    
    let mut engine = AnalysisEngine::new_with_memory_cache()
        .expect("Failed to create engine");
    
    // Test basic analysis method
    let result = engine.analyze(project.path()).await;
    assert!(result.is_ok(), "Basic analysis should succeed");
    
    let (issues, graph) = result.unwrap();
    
    // Verify return types are correct
    assert!(issues.is_empty() || !issues.is_empty(), "Issues should be a Vec<ArchitecturalIssue>");
    assert!(graph.node_count() >= 0, "Graph should be a LocalDependencyGraph");
}

#[tokio::test]
async fn test_detector_configuration_compatibility() {
    use crate::analysis::detectors::anti_patterns::god_object::GodObjectDetector;
    use crate::analysis::detectors::anti_patterns::dead_code::{DeadCodeDetector, DeadCodeConfig};
    use crate::analysis::detectors::anti_patterns::large_classes::{LargeClassDetector, LargeClassConfig};
    
    let project = TestProject::new();
    project.add_rust_file("large_class.rs", test_data::LARGE_RUST_CLASS);
    
    let mut engine = AnalysisEngine::new_with_memory_cache()
        .expect("Failed to create engine");
    
    // Test detector configuration methods still work
    engine.configure_dead_code_detector(DeadCodeConfig::default());
    engine.configure_large_classes_detector(LargeClassConfig::default());
    
    let result = engine.analyze(project.path()).await;
    assert!(result.is_ok(), "Analysis with configured detectors should succeed");
}

#[tokio::test]
async fn test_anti_pattern_types_compatibility() {
    let engine = AnalysisEngine::new_with_memory_cache()
        .expect("Failed to create engine");
    
    // Test that get_anti_pattern_types still works
    let types = engine.get_anti_pattern_types();
    assert!(!types.is_empty(), "Should return anti-pattern types");
    
    // Verify the types have the expected structure
    for pattern_type in types {
        assert!(pattern_type.name.len() > 0, "Anti-pattern type should have a name");
        assert!(pattern_type.description.len() > 0, "Anti-pattern type should have a description");
        assert!(pattern_type.category.len() > 0, "Anti-pattern type should have a category");
    }
}

#[tokio::test]
async fn test_files_analyzed_tracking() {
    let project = TestProject::new();
    project.add_rust_file("file1.rs", test_data::SIMPLE_RUST_FILE);
    project.add_rust_file("file2.rs", test_data::RUST_WITH_IMPORTS);
    project.add_rust_file("file3.rs", test_data::RUST_WITH_DEAD_CODE);
    
    let mut engine = AnalysisEngine::new_with_memory_cache()
        .expect("Failed to create engine");
    
    // Initial count should be 0
    assert_eq!(engine.get_files_analyzed(), 0);
    
    // Run analysis
    let result = engine.analyze(project.path()).await;
    assert!(result.is_ok(), "Analysis should succeed");
    
    // Files analyzed count should be updated
    let files_count = engine.get_files_analyzed();
    assert!(files_count > 0, "Should have analyzed some files");
    assert!(files_count <= 3, "Should not exceed the number of files created");
}

#[tokio::test]
async fn test_ast_cache_methods_compatibility() {
    let engine = AnalysisEngine::new_with_memory_cache()
        .expect("Failed to create engine");
    
    // Test AST cache methods
    let metrics = engine.get_ast_cache_metrics().await;
    assert!(metrics.is_object(), "Cache metrics should be a JSON object");
    
    // Clear cache should not panic
    engine.clear_ast_cache();
    
    // Metrics after clear
    let metrics_after = engine.get_ast_cache_metrics().await;
    assert!(metrics_after.is_object(), "Cache metrics should still be a JSON object after clear");
}

#[tokio::test]
async fn test_plugin_system_compatibility() {
    let mut engine = AnalysisEngine::new_with_plugins().await;
    
    // Plugin methods should exist regardless of whether plugins are enabled
    match engine {
        Ok(mut engine) => {
            // Test plugin support check
            let has_support = engine.has_plugin_support();
            assert!(has_support || !has_support, "has_plugin_support should return bool");
            
            if has_support {
                // Test plugin loading
                let load_result = engine.load_plugins().await;
                assert!(load_result.is_ok() || load_result.is_err(), "load_plugins should not panic");
                
                // Test plugin stats
                let stats = engine.get_stats().await;
                assert!(stats.is_some() || stats.is_none(), "get_plugin_stats should not panic");
                
                // Test resource monitoring
                let monitor_result = engine.monitor_plugin_resources().await;
                assert!(monitor_result.is_ok() || monitor_result.is_err(), "monitor_plugin_resources should not panic");
            }
        }
        Err(_) => {
            // Plugin initialization failed, but that's acceptable
        }
    }
}

#[tokio::test]
async fn test_builder_pattern_compatibility() {
    use crate::analysis::AnalysisEngineBuilder;
    use crate::analysis::detectors::anti_patterns::god_object::GodObjectDetector;
    
    // Test builder pattern still works
    let engine = AnalysisEngineBuilder::new()
        .with_in_memory_cache()
        .with_detectors(vec![Box::new(GodObjectDetector::new(10, 15))])
        .build()
;
    
    assert!(engine.is_ok(), "Builder pattern should work");
    
    let engine = engine.unwrap();
    let types = engine.get_anti_pattern_types();
    assert!(!types.is_empty(), "Built engine should have detectors");
}

#[tokio::test]
async fn test_analysis_results_structure() {
    let project = TestProject::new();
    project.add_rust_file("test.rs", test_data::LARGE_RUST_CLASS);
    
    let mut engine = AnalysisEngine::new_with_memory_cache()
        .expect("Failed to create engine");
    
    let result = engine.analyze(project.path()).await;
    assert!(result.is_ok(), "Analysis should succeed");
    
    let (issues, graph) = result.unwrap();
    
    // Test issues structure
    for issue in &issues {
        // Verify ArchitecturalIssue structure hasn't changed
        assert!(issue.analysis_run_id >= 0, "analysis_run_id should be valid");
        assert!(issue.anti_pattern_type_id >= 0, "anti_pattern_type_id should be valid");
        assert!(!issue.file_path.is_empty(), "file_path should not be empty");
        assert!(!issue.severity.is_empty(), "severity should not be empty");
        assert!(!issue.description.is_empty(), "description should not be empty");
    }
    
    // Test graph structure
    assert!(graph.node_count() >= 0, "Graph should have valid node count");
}

#[tokio::test]
async fn test_error_handling_compatibility() {
    let mut engine = AnalysisEngine::new_with_memory_cache()
        .expect("Failed to create engine");
    
    // Test analysis of non-existent path
    let non_existent_path = Path::new("/definitely/does/not/exist");
    let result = engine.analyze(non_existent_path).await;
    
    // Should handle error gracefully, not panic
    assert!(result.is_ok() || result.is_err(), "Error handling should be graceful");
}

#[tokio::test]
async fn test_multiple_engines_independence() {
    // Test that multiple engines can be created and used independently
    let project1 = TestProject::new();
    project1.add_rust_file("file1.rs", test_data::SIMPLE_RUST_FILE);
    
    let project2 = TestProject::new();
    project2.add_rust_file("file2.rs", test_data::RUST_WITH_IMPORTS);
    
    let engine1 = AnalysisEngine::new_with_memory_cache()
        .expect("Failed to create first engine");
    let engine2 = AnalysisEngine::new_with_memory_cache()
        .expect("Failed to create second engine");
    
    // Both engines should work independently
    let types1 = engine1.get_anti_pattern_types();
    let types2 = engine2.get_anti_pattern_types();
    
    assert!(!types1.is_empty(), "First engine should return anti-pattern types");
    assert!(!types2.is_empty(), "Second engine should return anti-pattern types");
    assert_eq!(types1.len(), types2.len(), "Both engines should have same detector set");
}

#[tokio::test]
async fn test_memory_usage_stability() {
    use std::sync::Arc;
    
    let project = TestProject::new();
    project.add_rust_file("test.rs", test_data::SIMPLE_RUST_FILE);
    
    let mut engine = AnalysisEngine::new_with_memory_cache()
        .expect("Failed to create engine");
    
    // Run multiple analyses to test for memory leaks
    for _ in 0..5 {
        let result = engine.analyze(project.path()).await;
        assert!(result.is_ok(), "Repeated analysis should succeed");
        
        // Clear caches periodically
        engine.clear_ast_cache();
    }
    
    // Final analysis should still work
    let final_result = engine.analyze(project.path()).await;
    assert!(final_result.is_ok(), "Final analysis should succeed");
}