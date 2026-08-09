use std::fs;
use tempfile::tempdir;
use uveddi::analysis::{
    ComponentNode, CycleDetector, DependencyExtractor, LocalDependencyGraph, LocalDependencyType,
};

#[test]
fn test_sprint1_cycle_detection() {
    let dir = tempdir().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();
    let mod1_path = src_dir.join("mod1.rs");
    let mod2_path = src_dir.join("mod2.rs");

    fs::write(&mod1_path, "pub mod mod2;").unwrap();
    fs::write(&mod2_path, "pub mod mod1;").unwrap();

    let mut extractor = DependencyExtractor::new().unwrap();
    let mut graph = LocalDependencyGraph::new();

    let deps1 = extractor.extract_from_file(&mod1_path).unwrap();
    for dep in deps1 {
        let from_node = ComponentNode::Module {
            path: "mod1".to_string(),
        };
        // Extract just the module name from the path
        let module_name = std::path::Path::new(&dep.to_module)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let to_node = ComponentNode::Module { path: module_name };
        graph.add_dependency(&from_node, &to_node, LocalDependencyType::Import);
    }

    let deps2 = extractor.extract_from_file(&mod2_path).unwrap();
    for dep in deps2 {
        let from_node = ComponentNode::Module {
            path: "mod2".to_string(),
        };
        // Extract just the module name from the path
        let module_name = std::path::Path::new(&dep.to_module)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let to_node = ComponentNode::Module { path: module_name };
        graph.add_dependency(&from_node, &to_node, LocalDependencyType::Import);
    }

    let detector = CycleDetector::new();
    let results = detector.detect_cycles(&graph, 1); // Dummy analysis_run_id

    // The cycle detector may report cycles from different starting points
    // (mod1 -> mod2 -> mod1) and (mod2 -> mod1 -> mod2)
    assert!(!results.is_empty(), "Should detect at least one cycle");

    // Verify that at least one result mentions both modules
    let has_both_modules = results
        .iter()
        .any(|r| r.description.contains("mod1") && r.description.contains("mod2"));
    assert!(
        has_both_modules,
        "At least one cycle should involve both mod1 and mod2"
    );
}
