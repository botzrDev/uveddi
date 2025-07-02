use tempfile::tempdir;
use std::fs;
use uveddi::analysis::dependency_extractor::DependencyExtractor;
use uveddi::analysis::dependency_graph::{LocalDependencyGraph, ComponentNode, LocalDependencyType};
use uveddi::analysis::cycle_detector::CycleDetector;

#[test]
fn test_sprint1_cycle_detection() {
    let dir = tempdir().unwrap();
    let mod1_path = dir.path().join("mod1.rs");
    let mod2_path = dir.path().join("mod2.rs");

    fs::write(&mod1_path, "pub mod mod2;").unwrap();
    fs::write(&mod2_path, "pub mod mod1;").unwrap();

    let mut extractor = DependencyExtractor::new().unwrap();
    let mut graph = LocalDependencyGraph::new();

    let deps1 = extractor.extract_from_file(&mod1_path).unwrap();
    for dep in deps1 {
        let from_node = ComponentNode::Module { path: dep.from_file.to_string_lossy().into_owned() };
        let to_node = ComponentNode::Module { path: dep.to_module };
        graph.add_dependency(&from_node, &to_node, LocalDependencyType::Import);
    }

    let deps2 = extractor.extract_from_file(&mod2_path).unwrap();
    for dep in deps2 {
        let from_node = ComponentNode::Module { path: dep.from_file.to_string_lossy().into_owned() };
        let to_node = ComponentNode::Module { path: dep.to_module };
        graph.add_dependency(&from_node, &to_node, LocalDependencyType::Import);
    }

    let detector = CycleDetector::new();
    let results = detector.detect_cycles(&graph, 1); // Dummy analysis_run_id

    assert_eq!(results.len(), 1);
    let description = &results[0].description;
    assert!(description.contains("mod1.rs"));
    assert!(description.contains("mod2.rs"));
}
