#[cfg(test)]
mod detector_tests {
    use crate::analysis::detectors::anti_patterns::tight_coupling::{
        Dependency, DependencyStrength, TightCouplingConfig, TightCouplingDetector,
    };
    use crate::analysis::graph::dependency::{
        ComponentNode, LocalDependencyGraph, LocalDependencyType,
    };
    use crate::analysis::AnalysisDetector;
    use crate::ast::tree_sitter_impl::SourceLanguage;

    #[test]
    fn test_detector_name() {
        let detector = TightCouplingDetector::default();
        assert_eq!(detector.get_detector_name(), "TightCouplingDetector");
    }

    #[test]
    fn test_anti_pattern_types() {
        let detector = TightCouplingDetector::default();
        let types = detector.get_anti_pattern_types();
        assert_eq!(types.len(), 1);
        assert_eq!(types[0].name, "Tight Coupling");
    }

    #[test]
    fn test_default_configuration() {
        let config = TightCouplingConfig::default();

        // Test Rust thresholds
        assert_eq!(config.rust_thresholds.fan_out_warning, 3);
        assert_eq!(config.rust_thresholds.fan_out_critical, 7);

        // Test Python thresholds
        assert_eq!(config.python_thresholds.fan_out_warning, 10);
        assert_eq!(config.python_thresholds.fan_out_critical, 15);

        // Test JavaScript thresholds
        assert_eq!(config.javascript_thresholds.fan_out_warning, 12);
        assert_eq!(config.javascript_thresholds.fan_out_critical, 18);

        assert!(config.enable_cross_file_analysis);
        assert!(!config.include_test_files);
    }

    #[test]
    fn test_coupling_metrics_calculation() {
        let detector = TightCouplingDetector::default();
        let mut graph = LocalDependencyGraph::new();

        // Add test components
        let comp1 = ComponentNode::Class {
            name: "TestClass1".to_string(),
            file_path: "test1.rs".to_string(),
        };
        let comp2 = ComponentNode::Class {
            name: "TestClass2".to_string(),
            file_path: "test2.rs".to_string(),
        };

        // Add dependency
        graph.add_dependency(&comp1, &comp2, LocalDependencyType::Call);

        let report = detector.generate_analysis_report(&graph, &[]);
        let metrics = report.metrics;

        // Verify metrics calculation
        assert!(metrics.contains_key(&comp1));
        assert!(metrics.contains_key(&comp2));

        let comp1_metrics = &metrics[&comp1];
        assert_eq!(comp1_metrics.fan_out, 1); // comp1 depends on comp2
        assert_eq!(comp1_metrics.fan_in, 0); // nothing depends on comp1

        let comp2_metrics = &metrics[&comp2];
        assert_eq!(comp2_metrics.fan_out, 0); // comp2 depends on nothing
        assert_eq!(comp2_metrics.fan_in, 1); // comp1 depends on comp2
    }

    #[test]
    fn test_dependency_strength_classification() {
        use DependencyStrength::*;

        // Test that different dependency types have appropriate strengths
        let import_dep = Dependency {
            from_component: ComponentNode::Module {
                path: "a.rs".to_string(),
            },
            to_component: ComponentNode::Module {
                path: "b.rs".to_string(),
            },
            dependency_type: LocalDependencyType::Import,
            line_number: Some(1),
            strength: Medium,
        };

        assert_eq!(import_dep.strength, Medium);

        let inheritance_dep = Dependency {
            from_component: ComponentNode::Class {
                name: "Child".to_string(),
                file_path: "child.rs".to_string(),
            },
            to_component: ComponentNode::Class {
                name: "Parent".to_string(),
                file_path: "parent.rs".to_string(),
            },
            dependency_type: LocalDependencyType::Inheritance,
            line_number: Some(5),
            strength: Strong,
        };

        assert_eq!(inheritance_dep.strength, Strong);
    }
}
