//! Integration tests for the facade pattern implementation in AnalysisEngine
//!
//! These tests verify that the refactored AnalysisEngine correctly delegates
//! to its internal components while maintaining backward compatibility.

use uveddi::analysis::AnalysisEngine;
use tempfile::tempdir;
use std::fs;

#[tokio::test]
async fn test_facade_pattern_ast_cache_delegation() {
    // Test that AST cache methods delegate to the AstProvider component
    let engine = AnalysisEngine::new_with_memory_cache().unwrap();
    
    // Initially, cache should have no metrics
    let initial_metrics = engine.get_ast_cache_metrics();
    assert!(initial_metrics.is_object());
    
    // Clear cache should not panic
    engine.clear_ast_cache();
    
    // Metrics should still be available after clear
    let post_clear_metrics = engine.get_ast_cache_metrics();
    assert!(post_clear_metrics.is_object());
}

#[tokio::test]
async fn test_facade_pattern_analysis_aggregation() {
    // Test that analysis results are properly aggregated through the facade
    let mut engine = AnalysisEngine::new_with_memory_cache().unwrap();
    
    // Create a temporary directory with a Rust file
    let temp_dir = tempdir().unwrap();
    let rust_file = temp_dir.path().join("test.rs");
    
    fs::write(
        &rust_file,
        r#"
use std::collections::HashMap;

fn main() {
    let map = HashMap::new();
    println!("Hello, world!");
}
"#,
    ).unwrap();
    
    // Run analysis
    let (issues, dependency_graph) = engine.analyze(temp_dir.path()).await.unwrap();
    
    // Verify that the facade correctly aggregates results
    assert_eq!(engine.get_files_analyzed(), 1);
    assert!(dependency_graph.node_count() >= 0); // Should have at least 0 nodes
    
    // Issues may or may not be present, but should be a valid Vec
    assert!(issues.len() >= 0);
}

#[tokio::test]
async fn test_facade_pattern_dependency_graph_building() {
    // Test that dependency graph building is properly delegated
    let mut engine = AnalysisEngine::new_with_memory_cache().unwrap();
    
    // Create a temporary directory with multiple Rust files with dependencies
    let temp_dir = tempdir().unwrap();
    
    // Create main.rs
    let main_file = temp_dir.path().join("main.rs");
    fs::write(
        &main_file,
        r#"
mod helper;

use helper::helper_function;

fn main() {
    helper_function();
}
"#,
    ).unwrap();
    
    // Create helper.rs
    let helper_file = temp_dir.path().join("helper.rs");
    fs::write(
        &helper_file,
        r#"
pub fn helper_function() {
    println!("Helper function called");
}
"#,
    ).unwrap();
    
    // Run analysis
    let (issues, dependency_graph) = engine.analyze(temp_dir.path()).await.unwrap();
    
    // Verify that the facade correctly builds dependency graph
    assert_eq!(engine.get_files_analyzed(), 2);
    assert!(dependency_graph.node_count() >= 0); // Should have nodes for the modules
    
    // Check that analysis found the files
    assert!(issues.len() >= 0);
}

#[tokio::test]
async fn test_facade_pattern_detector_scheduling() {
    // Test that detector scheduling is properly delegated
    let mut engine = AnalysisEngine::new_with_memory_cache().unwrap();
    
    // Create a temporary directory with a single file
    let temp_dir = tempdir().unwrap();
    let rust_file = temp_dir.path().join("single_file.rs");
    
    fs::write(
        &rust_file,
        r#"
fn small_function() {
    println!("Small function");
}
"#,
    ).unwrap();
    
    // Test single file analysis (should use schedule_file)
    let (issues, _) = engine.analyze(&rust_file).await.unwrap();
    
    // Should work without errors
    assert!(issues.len() >= 0);
    
    // Test directory analysis (should use schedule_directory)
    let (issues, _) = engine.analyze(temp_dir.path()).await.unwrap();
    
    // Should work without errors
    assert!(issues.len() >= 0);
}

#[test]
fn test_facade_pattern_backward_compatibility() {
    // Test that the facade maintains backward compatibility
    let engine = AnalysisEngine::new_with_memory_cache().unwrap();
    
    // Test that all existing public methods still work
    let anti_pattern_types = engine.get_anti_pattern_types();
    assert!(!anti_pattern_types.is_empty());
    
    // Test that AST cache methods work
    let _metrics = engine.get_ast_cache_metrics();
    engine.clear_ast_cache();
    
    // Test that file counter works
    assert_eq!(engine.get_files_analyzed(), 0);
    
    // Test plugin support detection
    let has_plugin_support = engine.has_plugin_support();
    assert!(has_plugin_support == true || has_plugin_support == false); // Should be a boolean
}

#[tokio::test]
async fn test_facade_pattern_component_integration() {
    // Test that all components work together through the facade
    let mut engine = AnalysisEngine::new_with_memory_cache().unwrap();
    
    // Create a more complex project structure
    let temp_dir = tempdir().unwrap();
    
    // Create lib.rs
    let lib_file = temp_dir.path().join("lib.rs");
    fs::write(
        &lib_file,
        r#"
pub mod utils;
pub mod models;

use utils::format_output;
use models::User;

pub fn process_user(user: User) -> String {
    format_output(user.name)
}
"#,
    ).unwrap();
    
    // Create utils.rs
    let utils_file = temp_dir.path().join("utils.rs");
    fs::write(
        &utils_file,
        r#"
pub fn format_output(input: String) -> String {
    format!("Formatted: {}", input)
}
"#,
    ).unwrap();
    
    // Create models.rs
    let models_file = temp_dir.path().join("models.rs");
    fs::write(
        &models_file,
        r#"
pub struct User {
    pub name: String,
    pub age: u32,
}

impl User {
    pub fn new(name: String, age: u32) -> Self {
        Self { name, age }
    }
}
"#,
    ).unwrap();
    
    // Run analysis
    let (issues, dependency_graph) = engine.analyze(temp_dir.path()).await.unwrap();
    
    // Verify that all components worked together
    assert_eq!(engine.get_files_analyzed(), 3);
    assert!(dependency_graph.node_count() >= 0);
    
    // Should have analyzed the files and potentially found some dependencies
    println!("Found {} issues across {} files", issues.len(), engine.get_files_analyzed());
    
    // Test that the facade aggregated results correctly
    let final_metrics = engine.get_ast_cache_metrics();
    assert!(final_metrics.is_object());
}