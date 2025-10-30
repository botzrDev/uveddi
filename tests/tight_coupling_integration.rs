//! Integration tests for tight coupling detector
//! Tests only public API functionality

use std::path::Path;
use uveddi::analysis::detectors::anti_patterns::tight_coupling::{
    TightCouplingConfig, TightCouplingDetector,
};
use uveddi::analysis::graph::dependency::{
    ComponentNode, LocalDependencyGraph, LocalDependencyType,
};
use uveddi::analysis::AnalysisDetector;
use uveddi::ast::tree_sitter_impl::{AstParser, ParsedFile, SourceLanguage};

fn create_test_rust_file_with_high_coupling() -> ParsedFile {
    let source = r#"
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::net::TcpStream;
use std::thread;
use std::sync::Mutex;
use std::time::Duration;

struct HighlyCoupledStruct {
    file_handler: File,
    network_client: TcpStream,
    cache: HashMap<String, String>,
    mutex: Mutex<i32>,
}

impl HighlyCoupledStruct {
    fn new() -> Self {
        let file = File::open("test.txt").unwrap();
        let stream = TcpStream::connect("127.0.0.1:8080").unwrap();
        let cache = HashMap::new();
        let mutex = Mutex::new(0);
        
        Self {
            file_handler: file,
            network_client: stream,
            cache,
            mutex,
        }
    }
    
    fn process_data(&self) {
        let mut buffer = Vec::new();
        self.file_handler.read_to_end(&mut buffer);
        self.cache.insert("key".to_string(), "value".to_string());
        let _guard = self.mutex.lock().unwrap();
    }
}
"#;

    let mut parser = AstParser::new().expect("Failed to create parser");
    parser
        .parse_content(
            source,
            Path::new("test_high_coupling.rs"),
            SourceLanguage::Rust,
        )
        .expect("Failed to parse test source")
}

fn create_test_python_file() -> ParsedFile {
    let source = r#"
import os
import sys
import json
import sqlite3
from datetime import datetime
from collections import defaultdict

class DataProcessor:
    def __init__(self):
        self.db = sqlite3.connect("data.db")
        self.cache = defaultdict(list)
        
    def process_file(self, filename):
        with open(filename, 'r') as f:
            data = json.load(f)
        
        cursor = self.db.cursor()
        cursor.execute("INSERT INTO data VALUES (?, ?)", (filename, str(data)))
        self.db.commit()
        
        self.cache[filename].append(data)
        return data
"#;

    let mut parser = AstParser::new().expect("Failed to create parser");
    parser
        .parse_content(source, Path::new("test_python.py"), SourceLanguage::Python)
        .expect("Failed to parse test source")
}

#[test]
fn test_detector_basic_functionality() {
    let detector = TightCouplingDetector::default();

    // Test detector name
    assert_eq!(detector.get_detector_name(), "TightCouplingDetector");

    // Test anti-pattern types
    let types = detector.get_anti_pattern_types();
    assert_eq!(types.len(), 1);
    assert_eq!(types[0].name, "Tight Coupling");
    assert_eq!(types[0].category, "structural");
}

#[test]
fn test_configuration_defaults() {
    let config = TightCouplingConfig::default();

    // Test Rust thresholds
    assert_eq!(config.rust_thresholds.fan_out_warning, 7);
    assert_eq!(config.rust_thresholds.fan_out_critical, 12);

    // Test Python thresholds
    assert_eq!(config.python_thresholds.fan_out_warning, 8);
    assert_eq!(config.python_thresholds.fan_out_critical, 15);

    // Test JavaScript thresholds
    assert_eq!(config.javascript_thresholds.fan_out_warning, 6);
    assert_eq!(config.javascript_thresholds.fan_out_critical, 10);

    // Test configuration flags
    assert!(config.enable_cross_file_analysis);
    assert!(!config.include_test_files);
}

#[tokio::test]
async fn test_rust_file_analysis() {
    let detector = TightCouplingDetector::default();
    let test_file = create_test_rust_file_with_high_coupling();

    // Test single file analysis
    let result = detector.detect_issues(&test_file).await;
    assert!(result.is_ok(), "Analysis should complete without errors");

    let issues = result.unwrap();
    println!("Found {} tight coupling issues in Rust file", issues.len());

    // Verify issue structure if any are found
    for issue in &issues {
        assert!(
            !issue.description.is_empty(),
            "Issue should have description"
        );
        assert!(!issue.severity.is_empty(), "Issue should have severity");
        assert!(
            issue.file_path.contains("test_high_coupling.rs"),
            "Issue should reference correct file"
        );
        println!("Rust Issue: {}", issue.description);
    }
}

#[tokio::test]
async fn test_python_file_analysis() {
    let detector = TightCouplingDetector::default();
    let test_file = create_test_python_file();

    // Test Python file analysis
    let result = detector.detect_issues(&test_file).await;
    assert!(
        result.is_ok(),
        "Python analysis should complete without errors"
    );

    let issues = result.unwrap();
    println!(
        "Found {} tight coupling issues in Python file",
        issues.len()
    );

    // Verify issue structure if any are found
    for issue in &issues {
        assert!(
            !issue.description.is_empty(),
            "Issue should have description"
        );
        assert!(
            issue.file_path.contains("test_python.py"),
            "Issue should reference correct file"
        );
        println!("Python Issue: {}", issue.description);
    }
}

#[test]
fn test_dependency_graph_analysis() {
    let detector = TightCouplingDetector::default();

    // Create a simple dependency graph for testing
    let mut graph = LocalDependencyGraph::new();

    // Add some test components
    let component1 = ComponentNode::Module {
        path: "src/main.rs".to_string(),
    };
    let component2 = ComponentNode::Module {
        path: "src/lib.rs".to_string(),
    };
    let component3 = ComponentNode::Class {
        name: "TestClass".to_string(),
        file_path: "src/test.rs".to_string(),
    };

    // Add nodes to graph
    graph.add_component(component1.clone());
    graph.add_component(component2.clone());
    graph.add_component(component3.clone());

    // Add some dependencies
    graph.add_dependency(&component1, &component2, LocalDependencyType::Import);
    graph.add_dependency(&component2, &component3, LocalDependencyType::Call);

    // Test that the graph has components
    assert!(
        graph.get_petgraph().node_count() > 0,
        "Graph should have nodes"
    );

    // Test graph-level analysis through public API
    let issues = detector.detect_graph_issues(&graph, 1);
    println!("Found {} graph-level coupling issues", issues.len());

    // Verify issues have proper analysis_run_id
    for issue in &issues {
        assert_eq!(
            issue.analysis_run_id, 1,
            "Issue should have correct analysis_run_id"
        );
        println!("Graph Issue: {}", issue.description);
    }
}

#[tokio::test]
async fn test_detector_creation_with_custom_config() {
    // Test creation with custom config
    let mut config = TightCouplingConfig::default();
    config.rust_thresholds.fan_out_warning = 5;
    config.rust_thresholds.fan_out_critical = 8;

    let detector = TightCouplingDetector::new(config.clone());
    assert_eq!(detector.get_detector_name(), "TightCouplingDetector");

    // Test that the detector works with custom config
    let test_file = create_test_rust_file_with_high_coupling();
    let result = detector.detect_issues(&test_file).await;
    assert!(result.is_ok(), "Analysis with custom config should work");
}

#[tokio::test]
async fn test_multi_language_support() {
    let detector = TightCouplingDetector::default();

    // Test Rust file
    let rust_file = create_test_rust_file_with_high_coupling();
    let rust_result = detector.detect_issues(&rust_file).await;
    assert!(rust_result.is_ok(), "Rust analysis should work");

    // Test Python file
    let python_file = create_test_python_file();
    let python_result = detector.detect_issues(&python_file).await;
    assert!(python_result.is_ok(), "Python analysis should work");

    println!("Multi-language support verified: Rust and Python files analyzed successfully");
}
