//! tests/analysis/anti_patterns/modularity_violation_detector.rs

use uveddi::analysis::anti_patterns::modularity_violation_detector::ModularityViolationDetector;
use uveddi::analysis::dependency_graph::DependencyGraph;
use std::path::PathBuf;

#[test]
fn test_modularity_violation_detector() {
    // 1. Create a dependency graph
    let mut graph = DependencyGraph::new();
    let analysis_run_id = 1;

    // 2. Add modules from different communities
    let auth_service = "auth::service".to_string();
    let auth_utils = "auth::utils".to_string();
    let user_model = "user::model".to_string();
    let user_controller = "user::controller".to_string();
    let payment_service = "payment::service".to_string();

    graph.add_module(auth_service.clone(), PathBuf::from("src/auth/service.rs"));
    graph.add_module(auth_utils.clone(), PathBuf::from("src/auth/utils.rs"));
    graph.add_module(user_model.clone(), PathBuf::from("src/user/model.rs"));
    graph.add_module(user_controller.clone(), PathBuf::from("src/user/controller.rs"));
    graph.add_module(payment_service.clone(), PathBuf::from("src/payment/service.rs"));

    // 3. Create cross-community dependencies
    // Strong dependency from 'auth' to 'user' (3 edges)
    graph.add_dependency(auth_service.clone(), user_model.clone());
    graph.add_dependency(auth_service.clone(), user_controller.clone());
    graph.add_dependency(auth_utils.clone(), user_model.clone());

    // Weak dependency from 'payment' to 'user' (1 edge)
    graph.add_dependency(payment_service.clone(), user_model.clone());

    // 4. Run the detector
    let detector = ModularityViolationDetector::new();
    let issues = detector.detect_in_graph(&graph, analysis_run_id);

    // 5. Assert the results
    assert_eq!(issues.len(), 1);
    let issue = &issues[0];
    assert_eq!(issue.file_path, "auth -> user");
    assert_eq!(issue.severity, "medium");
    assert!(issue.description.contains("Strong dependency (3 edges)"));
}