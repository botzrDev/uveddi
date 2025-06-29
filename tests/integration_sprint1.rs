use std::path::Path;
use tempfile::tempdir;
use std::fs;
use codeatlas::analysis::dependency_extractor::DependencyExtractor;
use codeatlas::analysis::dependency_graph::DependencyGraph;
use codeatlas::analysis::cycle_detector::CycleDetector;

#[test]
fn test_sprint1_cycle_detection() {
    let dir = tempdir().unwrap();
    let mod1_path = dir.path().join("mod1.rs");
    let mod2_path = dir.path().join("mod2.rs");

    fs::write(&mod1_path, "mod mod2;").unwrap();
    fs::write(&mod2_path, "mod mod1;").unwrap();

    let mut extractor = DependencyExtractor::new().unwrap();
    let mut deps = extractor.extract_from_file(&mod1_path).unwrap();
    deps.extend(extractor.extract_from_file(&mod2_path).unwrap());

    let mut graph = DependencyGraph::new();
    graph.build_from_dependencies(deps);
    let mut detector = CycleDetector::new();
    let results = detector.detect_cycles(&graph);

    assert_eq!(results.cycles.len(), 1);
    assert_eq!(results.cycles[0].modules.len(), 2);
}
