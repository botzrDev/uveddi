//! tests/analysis/anti_patterns/unstable_interface_detector.rs

use uveddi::analysis::anti_patterns::unstable_interface_detector::UnstableInterfaceDetector;
use uveddi::analysis::dependency_graph::DependencyGraph;
use std::path::PathBuf;

#[test]
fn test_unstable_interface_detector() {
    // 1. Create a dependency graph
    let mut graph = DependencyGraph::new();
    let analysis_run_id = 1;

    // 2. Add modules and dependencies
    let stable_module = "stable_module".to_string();
    let unstable_module = "unstable_module".to_string();
    let module1 = "module1".to_string();
    let module2 = "module2".to_string();
    let module3 = "module3".to_string();
    let module4 = "module4".to_string();
    let module5 = "module5".to_string();

    graph.add_module(stable_module.clone(), PathBuf::from("src/stable.rs"));
    graph.add_module(unstable_module.clone(), PathBuf::from("src/unstable.rs"));
    graph.add_module(module1.clone(), PathBuf::from("src/module1.rs"));
    graph.add_module(module2.clone(), PathBuf::from("src/module2.rs"));
    graph.add_module(module3.clone(), PathBuf::from("src/module3.rs"));
    graph.add_module(module4.clone(), PathBuf::from("src/module4.rs"));
    graph.add_module(module5.clone(), PathBuf::from("src/module5.rs"));

    // `unstable_module` is a dependency for 5 other modules
    graph.add_dependency(module1.clone(), unstable_module.clone());
    graph.add_dependency(module2.clone(), unstable_module.clone());
    graph.add_dependency(module3.clone(), unstable_module.clone());
    graph.add_dependency(module4.clone(), unstable_module.clone());
    graph.add_dependency(module5.clone(), unstable_module.clone());

    // `stable_module` is a dependency for 1 other module
    graph.add_dependency(module1.clone(), stable_module.clone());

    // 3. Run the detector
    let detector = UnstableInterfaceDetector::new();
    let issues = detector.detect_in_graph(&graph, analysis_run_id);

    // 4. Assert the results
    assert_eq!(issues.len(), 1);
    let issue = &issues[0];
    assert_eq!(issue.file_path, "src/unstable.rs");
    assert_eq!(issue.severity, "medium");
    assert!(issue.description.contains("high fan-in (5 dependents)"));
}