use std::fs;
use tempfile::tempdir;
use uveddi::analysis::detectors::dependency::DependencyType;
use uveddi::analysis::{Dependency, DependencyExtractor};

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
    fs::write(
        &file_path,
        "import React from 'react';\nconst my_mod = require('./my_mod');",
    )
    .unwrap();

    let mut extractor = DependencyExtractor::new().unwrap();
    let deps = extractor.extract_from_file(&file_path).unwrap();

    // Should extract at least some dependencies
    assert!(
        !deps.is_empty(),
        "Should extract JavaScript dependencies"
    );

    // Check that we extract the react import
    let dep_names: Vec<_> = deps.iter().map(|d| d.to_module.as_str()).collect();
    assert!(
        dep_names.iter().any(|n| n.contains("react")),
        "Should extract 'react' dependency, got: {:?}",
        dep_names
    );

    // Check for my_mod (extractor may include path prefix like "./my_mod")
    assert!(
        dep_names.iter().any(|n| n.contains("my_mod")),
        "Should extract 'my_mod' dependency, got: {:?}",
        dep_names
    );
}
