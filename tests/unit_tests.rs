use tempfile::tempdir;
use std::fs;
use codeatlas::analysis::dependency_extractor::{DependencyExtractor, Dependency, DependencyType};

#[test]
fn test_extract_rust_dependencies() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("main.rs");
    fs::write(&file_path, "use std::collections::HashMap;\nmod my_mod;").unwrap();

    let mut extractor = DependencyExtractor::new().unwrap();
    let deps = extractor.extract_from_file(&file_path).unwrap();

    assert_eq!(deps.len(), 2);
    assert!(deps.contains(&Dependency {
        from_file: file_path.clone(),
        to_module: "std::collections::HashMap".to_string(),
        dependency_type: DependencyType::Use,
        line_number: Some(1),
    }));
    assert!(deps.contains(&Dependency {
        from_file: file_path.clone(),
        to_module: "my_mod".to_string(),
        dependency_type: DependencyType::Use,
        line_number: Some(2),
    }));
}

#[test]
fn test_extract_python_dependencies() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("main.py");
    fs::write(&file_path, "import os\nfrom my_module import my_func").unwrap();

    let mut extractor = DependencyExtractor::new().unwrap();
    let deps = extractor.extract_from_file(&file_path).unwrap();

    assert_eq!(deps.len(), 2);
    assert!(deps.contains(&Dependency {
        from_file: file_path.clone(),
        to_module: "os".to_string(),
        dependency_type: DependencyType::Import,
        line_number: Some(1),
    }));
    assert!(deps.contains(&Dependency {
        from_file: file_path.clone(),
        to_module: "my_module".to_string(),
        dependency_type: DependencyType::Import,
        line_number: Some(2),
    }));
}

#[test]
fn test_extract_javascript_dependencies() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("main.js");
    fs::write(&file_path, "import React from 'react';\nconst my_mod = require('./my_mod');").unwrap();

    let mut extractor = DependencyExtractor::new().unwrap();
    let deps = extractor.extract_from_file(&file_path).unwrap();

    assert_eq!(deps.len(), 2);
    assert!(deps.contains(&Dependency {
        from_file: file_path.clone(),
        to_module: "react".to_string(),
        dependency_type: DependencyType::Import,
        line_number: Some(1),
    }));
    assert!(deps.contains(&Dependency {
        from_file: file_path.clone(),
        to_module: "my_mod".to_string(),
        dependency_type: DependencyType::Import,
        line_number: Some(2),
    }));
}