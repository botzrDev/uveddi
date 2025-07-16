//! Test fixtures for generating consistent test data
//!
//! This module provides fixtures and data generators for creating
//! consistent test scenarios across the test suite.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

use crate::analysis::config::AnalysisConfig;
use crate::analysis::detectors::dependency::Dependency;
use crate::ast::{ParsedFile, SourceLanguage};
use crate::database::models::{ArchitecturalIssue, AntiPatternType};

/// Test fixtures container that manages temporary test data
pub struct TestFixtures {
    pub temp_dir: TempDir,
    pub sample_rust_file: PathBuf,
    pub sample_python_file: PathBuf,
    pub sample_js_file: PathBuf,
    pub sample_config: AnalysisConfig,
}

impl TestFixtures {
    /// Creates a new test fixtures instance with sample files
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        
        // Create sample Rust file
        let sample_rust_file = temp_dir.path().join("sample.rs");
        fs::write(&sample_rust_file, SAMPLE_RUST_CODE)?;
        
        // Create sample Python file
        let sample_python_file = temp_dir.path().join("sample.py");
        fs::write(&sample_python_file, SAMPLE_PYTHON_CODE)?;
        
        // Create sample JavaScript file
        let sample_js_file = temp_dir.path().join("sample.js");
        fs::write(&sample_js_file, SAMPLE_JS_CODE)?;
        
        // Create sample config
        let sample_config = create_test_config();
        
        Ok(Self {
            temp_dir,
            sample_rust_file,
            sample_python_file,
            sample_js_file,
            sample_config,
        })
    }
    
    /// Creates a new test fixtures instance with custom files
    pub fn with_custom_files(files: Vec<(&str, &str)>) -> Result<Self, Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        
        for (filename, content) in files {
            let file_path = temp_dir.path().join(filename);
            if let Some(parent) = file_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&file_path, content)?;
        }
        
        let sample_config = create_test_config();
        
        Ok(Self {
            sample_rust_file: temp_dir.path().join("sample.rs"),
            sample_python_file: temp_dir.path().join("sample.py"),
            sample_js_file: temp_dir.path().join("sample.js"),
            temp_dir,
            sample_config,
        })
    }
    
    /// Get the path to the temporary directory
    pub fn temp_path(&self) -> &std::path::Path {
        self.temp_dir.path()
    }
    
    /// Create a new file in the temporary directory
    pub fn create_file(&self, filename: &str, content: &str) -> Result<PathBuf, std::io::Error> {
        let file_path = self.temp_dir.path().join(filename);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&file_path, content)?;
        Ok(file_path)
    }
}

/// Sample Rust code for testing
pub const SAMPLE_RUST_CODE: &str = r#"
use std::collections::HashMap;

pub struct LargeClass {
    field1: String,
    field2: i32,
    field3: HashMap<String, i32>,
    field4: Vec<String>,
    field5: Option<String>,
}

impl LargeClass {
    pub fn new() -> Self {
        Self {
            field1: String::new(),
            field2: 0,
            field3: HashMap::new(),
            field4: Vec::new(),
            field5: None,
        }
    }
    
    pub fn method1(&self) -> String {
        self.field1.clone()
    }
    
    pub fn method2(&mut self, value: i32) {
        self.field2 = value;
    }
    
    pub fn method3(&self) -> Option<&str> {
        self.field5.as_deref()
    }
    
    fn dead_code_method(&self) {
        // This method is never called
        println!("Dead code");
    }
}

pub fn magic_number_function() -> i32 {
    42 // Magic number
}

pub fn long_method() -> Result<String, Box<dyn std::error::Error>> {
    let mut result = String::new();
    
    // This is a very long method that should be detected
    for i in 0..100 {
        result.push_str(&format!("Line {}\n", i));
    }
    
    result.push_str("More processing...\n");
    result.push_str("Even more processing...\n");
    result.push_str("Yet more processing...\n");
    result.push_str("Still more processing...\n");
    result.push_str("Final processing...\n");
    
    Ok(result)
}
"#;

/// Sample Python code for testing
pub const SAMPLE_PYTHON_CODE: &str = r#"
import os
import sys
from typing import Dict, List, Optional

class LargeClass:
    def __init__(self):
        self.field1 = ""
        self.field2 = 0
        self.field3 = {}
        self.field4 = []
        self.field5 = None
    
    def method1(self) -> str:
        return self.field1
    
    def method2(self, value: int):
        self.field2 = value
    
    def method3(self) -> Optional[str]:
        return self.field5
    
    def dead_code_method(self):
        # This method is never called
        print("Dead code")

def magic_number_function() -> int:
    return 42  # Magic number

def long_method() -> str:
    result = ""
    
    # This is a very long method that should be detected
    for i in range(100):
        result += f"Line {i}\n"
    
    result += "More processing...\n"
    result += "Even more processing...\n"
    result += "Yet more processing...\n"
    result += "Still more processing...\n"
    result += "Final processing...\n"
    
    return result

if __name__ == "__main__":
    print("Sample Python code")
"#;

/// Sample JavaScript code for testing
pub const SAMPLE_JS_CODE: &str = r#"
const fs = require('fs');
const path = require('path');

class LargeClass {
    constructor() {
        this.field1 = "";
        this.field2 = 0;
        this.field3 = {};
        this.field4 = [];
        this.field5 = null;
    }
    
    method1() {
        return this.field1;
    }
    
    method2(value) {
        this.field2 = value;
    }
    
    method3() {
        return this.field5;
    }
    
    deadCodeMethod() {
        // This method is never called
        console.log("Dead code");
    }
}

function magicNumberFunction() {
    return 42; // Magic number
}

function longMethod() {
    let result = "";
    
    // This is a very long method that should be detected
    for (let i = 0; i < 100; i++) {
        result += `Line ${i}\n`;
    }
    
    result += "More processing...\n";
    result += "Even more processing...\n";
    result += "Yet more processing...\n";
    result += "Still more processing...\n";
    result += "Final processing...\n";
    
    return result;
}

module.exports = {
    LargeClass,
    magicNumberFunction,
    longMethod
};
"#;

/// Creates a test configuration
fn create_test_config() -> AnalysisConfig {
    AnalysisConfig {
        project_root: "/tmp/test_project".to_string(),
        excluded_paths: vec![".git".to_string(), "target".to_string(), "node_modules".to_string()],
        language_configs: HashMap::new(),
        performance_config: None,
        plugin_configs: HashMap::new(),
        cache_config: None,
        ai_config: None,
    }
}

/// Sample dependency generators
pub mod dependencies {
    use super::*;
    
    pub fn create_sample_rust_dependencies() -> Vec<Dependency> {
        vec![
            Dependency {
                name: "std::collections::HashMap".to_string(),
                from_file: "src/main.rs".to_string(),
                to_file: "std".to_string(),
                dependency_type: "use".to_string(),
                line_number: 1,
            },
            Dependency {
                name: "custom_module::CustomStruct".to_string(),
                from_file: "src/main.rs".to_string(),
                to_file: "src/custom_module.rs".to_string(),
                dependency_type: "use".to_string(),
                line_number: 2,
            },
        ]
    }
    
    pub fn create_sample_python_dependencies() -> Vec<Dependency> {
        vec![
            Dependency {
                name: "os".to_string(),
                from_file: "src/main.py".to_string(),
                to_file: "os".to_string(),
                dependency_type: "import".to_string(),
                line_number: 1,
            },
            Dependency {
                name: "sys".to_string(),
                from_file: "src/main.py".to_string(),
                to_file: "sys".to_string(),
                dependency_type: "import".to_string(),
                line_number: 2,
            },
        ]
    }
}

/// Sample architectural issues generators
pub mod issues {
    use super::*;
    
    pub fn create_sample_dead_code_issue() -> ArchitecturalIssue {
        ArchitecturalIssue {
            id: None,
            issue_type: AntiPatternType::DeadCode,
            description: "Unused method detected".to_string(),
            file_path: "src/sample.rs".to_string(),
            line_number: Some(42),
            column_number: Some(5),
            severity: "medium".to_string(),
            confidence: 0.8,
            metadata: None,
        }
    }
    
    pub fn create_sample_large_class_issue() -> ArchitecturalIssue {
        ArchitecturalIssue {
            id: None,
            issue_type: AntiPatternType::LargeClass,
            description: "Class has too many methods and fields".to_string(),
            file_path: "src/sample.rs".to_string(),
            line_number: Some(5),
            column_number: Some(1),
            severity: "high".to_string(),
            confidence: 0.9,
            metadata: None,
        }
    }
    
    pub fn create_sample_magic_values_issue() -> ArchitecturalIssue {
        ArchitecturalIssue {
            id: None,
            issue_type: AntiPatternType::MagicValues,
            description: "Magic number detected: 42".to_string(),
            file_path: "src/sample.rs".to_string(),
            line_number: Some(50),
            column_number: Some(5),
            severity: "low".to_string(),
            confidence: 0.7,
            metadata: None,
        }
    }
    
    pub fn create_sample_long_method_issue() -> ArchitecturalIssue {
        ArchitecturalIssue {
            id: None,
            issue_type: AntiPatternType::LongMethods,
            description: "Method is too long (30+ lines)".to_string(),
            file_path: "src/sample.rs".to_string(),
            line_number: Some(55),
            column_number: Some(1),
            severity: "medium".to_string(),
            confidence: 0.8,
            metadata: None,
        }
    }
}

/// Sample parsed file generators
pub mod parsed_files {
    use super::*;
    
    pub fn create_sample_rust_parsed_file() -> ParsedFile {
        ParsedFile {
            path: PathBuf::from("src/sample.rs"),
            content: SAMPLE_RUST_CODE.to_string(),
            language: SourceLanguage::Rust,
            ast: None, // For testing purposes
        }
    }
    
    pub fn create_sample_python_parsed_file() -> ParsedFile {
        ParsedFile {
            path: PathBuf::from("src/sample.py"),
            content: SAMPLE_PYTHON_CODE.to_string(),
            language: SourceLanguage::Python,
            ast: None, // For testing purposes
        }
    }
    
    pub fn create_sample_js_parsed_file() -> ParsedFile {
        ParsedFile {
            path: PathBuf::from("src/sample.js"),
            content: SAMPLE_JS_CODE.to_string(),
            language: SourceLanguage::JavaScript,
            ast: None, // For testing purposes
        }
    }
}