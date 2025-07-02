//! Cycle detection tests

#[cfg(test)]
mod tests {
    use uveddi::analysis::{LocalDependencyGraph, ComponentNode, LocalDependencyType, CycleDetector};

    #[test]
    fn test_simple_cycle_detection() {
        let mut graph = LocalDependencyGraph::new();
        
        // Create a simple cycle: A -> B -> A
        let node_a = ComponentNode::Module { path: "module_a".to_string() };
        let node_b = ComponentNode::Module { path: "module_b".to_string() };
        
        graph.add_dependency(&node_a, &node_b, LocalDependencyType::Import);
        graph.add_dependency(&node_b, &node_a, LocalDependencyType::Import);
        
        let detector = CycleDetector::new();
        let issues = detector.detect_cycles(&graph, 0);
        
        // Check that at least one cycle-related issue was detected
        assert!(!issues.is_empty(), "Should detect cycle-related issues");
    }

    #[test]
    fn test_no_cycle_detection() {
        let mut graph = LocalDependencyGraph::new();
        
        // Create a linear dependency chain: A -> B -> C
        let node_a = ComponentNode::Module { path: "module_a".to_string() };
        let node_b = ComponentNode::Module { path: "module_b".to_string() };
        let node_c = ComponentNode::Module { path: "module_c".to_string() };
        
        graph.add_dependency(&node_a, &node_b, LocalDependencyType::Import);
        graph.add_dependency(&node_b, &node_c, LocalDependencyType::Import);
        
        let detector = CycleDetector::new();
        let issues = detector.detect_cycles(&graph, 0);
        
        // Filter for cycle-specific issues (if any)
        let cycle_issues: Vec<_> = issues.iter()
            .filter(|issue| issue.description.contains("cycle") || issue.description.contains("Cycle"))
            .collect();
        
        assert!(cycle_issues.is_empty(), "Should not detect any cycles in linear dependency chain");
    }

    #[test]
    fn test_complex_cycle_detection() {
        let mut graph = LocalDependencyGraph::new();
        
        // Create a more complex cycle: A -> B -> C -> A
        let node_a = ComponentNode::Module { path: "module_a".to_string() };
        let node_b = ComponentNode::Module { path: "module_b".to_string() };
        let node_c = ComponentNode::Module { path: "module_c".to_string() };
        
        graph.add_dependency(&node_a, &node_b, LocalDependencyType::Import);
        graph.add_dependency(&node_b, &node_c, LocalDependencyType::Import);
        graph.add_dependency(&node_c, &node_a, LocalDependencyType::Import);
        
        let detector = CycleDetector::new();
        let issues = detector.detect_cycles(&graph, 0);
        
        // Check that cycle-related issues were detected
        assert!(!issues.is_empty(), "Should detect cycle-related issues in complex cycle");
    }
}
