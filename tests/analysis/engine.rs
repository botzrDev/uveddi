use uveddi::analysis::AnalysisEngine;
use tempfile;

#[test]
fn test_new_engine_creation() {
    let engine = AnalysisEngine::new_with_memory_cache();
    assert!(engine.is_ok());
}

#[test]
fn test_get_anti_pattern_types() {
    let engine = AnalysisEngine::new_with_memory_cache().unwrap();
    let types = engine.get_anti_pattern_types();

    let names: Vec<_> = types.iter().map(|t| t.name.as_str()).collect();

    assert!(names.contains(&"God Object"));
    assert!(names.contains(&"Code Duplication"));
    assert!(names.contains(&"Cyclic Dependency"));
}

#[test]
fn test_get_files_analyzed_initially_zero() {
    let engine = AnalysisEngine::new_with_memory_cache().unwrap();
    assert_eq!(engine.get_files_analyzed(), 0);
}

#[tokio::test]
async fn test_analyze_empty_directory() {
    let mut engine = AnalysisEngine::new_with_memory_cache().unwrap();
    
    // Create a temporary empty directory
    let temp_dir = tempfile::tempdir().unwrap();
    let (issues, _graph) = engine.analyze(temp_dir.path()).await.unwrap();
    
    assert_eq!(issues.len(), 0);
    assert_eq!(engine.get_files_analyzed(), 0);
}

#[tokio::test]
async fn test_analyze_simple_rust_file() {
    let mut engine = AnalysisEngine::new_with_memory_cache().unwrap();
    
    // Create a temporary directory with a simple Rust file
    let temp_dir = tempfile::tempdir().unwrap();
    let rust_file = temp_dir.path().join("test.rs");
    
    // Create a simple Rust file with a dependency
    std::fs::write(&rust_file, r#"
use std::collections::HashMap;

fn main() {
    let map = HashMap::new();
    println!("Hello, world!");
}
"#).unwrap();
    
    let (issues, _graph) = engine.analyze(temp_dir.path()).await.unwrap();
    
    // Should have analyzed one file
    assert_eq!(engine.get_files_analyzed(), 1);
    
    // May or may not have issues depending on the detectors, but should not panic
    println!("Found {} issues", issues.len());
}

#[tokio::test]
async fn test_analyze_god_object_detection() {
    let mut engine = AnalysisEngine::new_with_memory_cache().unwrap();
    
    // Create a temporary directory with a Rust file that should trigger god object detection
    let temp_dir = tempfile::tempdir().unwrap();
    let rust_file = temp_dir.path().join("god_object.rs");
    
    // Create a struct with many methods (should trigger god object detector)
    std::fs::write(&rust_file, r#"
struct GodObject {
    field1: i32,
    field2: String,
    field3: Vec<i32>,
    field4: HashMap<String, i32>,
    field5: Option<String>,
    field6: bool,
    field7: f64,
    field8: char,
    field9: u64,
}

impl GodObject {
    fn method1(&self) -> i32 { self.field1 }
    fn method2(&self) -> &String { &self.field2 }
    fn method3(&self) -> &Vec<i32> { &self.field3 }
    fn method4(&self) -> bool { self.field6 }
    fn method5(&self) -> f64 { self.field7 }
    fn method6(&self) -> char { self.field8 }
    fn method7(&self) -> u64 { self.field9 }
    fn method8(&mut self) { self.field1 += 1; }
    fn method9(&mut self) { self.field6 = !self.field6; }
    fn method10(&self) -> String { format!("{:?}", self.field3) }
}
"#).unwrap();
    
    let (issues, _graph) = engine.analyze(temp_dir.path()).await.unwrap();
    
    // Should have analyzed one file
    assert_eq!(engine.get_files_analyzed(), 1);
    
    // Should detect at least one issue (god object)
    println!("Found {} issues", issues.len());
    assert!(issues.len() > 0, "Expected to find god object issue");
    
    // Check if we found a god object issue
    let has_god_object = issues.iter().any(|issue| 
        issue.description.to_lowercase().contains("god object") ||
        issue.description.to_lowercase().contains("too many")
    );
    assert!(has_god_object, "Expected to find God Object anti-pattern");
}
