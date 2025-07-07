//! Integration tests for the enhanced visualization system
//!
//! These tests validate the complete pipeline from AST components to rendered diagrams,
//! ensuring all anti-pattern templates work correctly with the new data models.

#[cfg(test)]
mod tests {
    use crate::analysis::mermaid_generator::{MermaidGenerator, MermaidGenerationError};
    use crate::models::visualization::{
        ArchitecturalComponent, ComponentType, ComponentMetrics, DiagramType, Dependency, DependencyType,
        FieldInfo, MethodSignature, ParameterInfo, Visibility
    };
    use std::collections::HashMap;
    use std::path::PathBuf;
    use uuid::Uuid;

    /// Create test components with enhanced type information
    fn create_test_components() -> Vec<ArchitecturalComponent> {
        vec![
            // Rust struct with fields
            ArchitecturalComponent {
                component_id: Uuid::new_v4(),
                name: "UserService".to_string(),
                file_path: PathBuf::from("src/services/user.rs"),
                component_type: ComponentType::RustStruct {
                    fields: vec![
                        FieldInfo {
                            name: "db_connection".to_string(),
                            field_type: "Arc<DatabaseConnection>".to_string(),
                            visibility: Visibility::Private,
                            is_optional: false,
                        },
                        FieldInfo {
                            name: "cache".to_string(),
                            field_type: "Option<Cache>".to_string(),
                            visibility: Visibility::Private,
                            is_optional: true,
                        },
                    ],
                },
                dependencies: vec![],
                metrics: ComponentMetrics {
                    lines_of_code: Some(350),
                    complexity: Some(8.5),
                    afferent_coupling: 5,
                    efferent_coupling: 12,
                    coupling_between_objects: Some(17),
                    public_methods: Some(8),
                },
                group: Some("services".to_string()),
            },
            // Python class with methods
            ArchitecturalComponent {
                component_id: Uuid::new_v4(),
                name: "DataProcessor".to_string(),
                file_path: PathBuf::from("src/processing/data.py"),
                component_type: ComponentType::PythonClass {
                    bases: vec!["BaseProcessor".to_string(), "Loggable".to_string()],
                    methods: vec![],
                    is_abstract: false,
                },
                dependencies: vec![],
                metrics: ComponentMetrics {
                    lines_of_code: Some(150),
                    complexity: Some(4.2),
                    afferent_coupling: 3,
                    efferent_coupling: 6,
                    coupling_between_objects: Some(9),
                    public_methods: Some(4),
                },
                group: Some("processing".to_string()),
            },
            // JavaScript ES Module
            ArchitecturalComponent {
                component_id: Uuid::new_v4(),
                name: "ApiClient".to_string(),
                file_path: PathBuf::from("src/api/client.js"),
                component_type: ComponentType::JavaScriptEsModule {
                    exports: vec![],
                },
                dependencies: vec![],
                metrics: ComponentMetrics {
                    lines_of_code: Some(80),
                    complexity: Some(2.1),
                    afferent_coupling: 2,
                    efferent_coupling: 4,
                    coupling_between_objects: Some(6),
                    public_methods: Some(6),
                },
                group: Some("api".to_string()),
            },
        ]
    }

    /// Create test components with cyclic dependencies
    fn create_cyclic_test_components() -> (Vec<ArchitecturalComponent>, Vec<Vec<Uuid>>, Vec<(Uuid, Uuid)>) {
        let comp1_id = Uuid::new_v4();
        let comp2_id = Uuid::new_v4();
        let comp3_id = Uuid::new_v4();

        let components = vec![
            ArchitecturalComponent {
                component_id: comp1_id,
                name: "ModuleA".to_string(),
                file_path: PathBuf::from("src/module_a.rs"),
                component_type: ComponentType::RustModule { is_public: true },
                dependencies: vec![
                    Dependency {
                        target_component_id: comp2_id,
                        dependency_type: DependencyType::Imports,
                        properties: HashMap::new(),
                    }
                ],
                metrics: ComponentMetrics::default(),
                group: Some("core".to_string()),
            },
            ArchitecturalComponent {
                component_id: comp2_id,
                name: "ModuleB".to_string(),
                file_path: PathBuf::from("src/module_b.rs"),
                component_type: ComponentType::RustModule { is_public: true },
                dependencies: vec![
                    Dependency {
                        target_component_id: comp3_id,
                        dependency_type: DependencyType::Imports,
                        properties: HashMap::new(),
                    }
                ],
                metrics: ComponentMetrics::default(),
                group: Some("core".to_string()),
            },
            ArchitecturalComponent {
                component_id: comp3_id,
                name: "ModuleC".to_string(),
                file_path: PathBuf::from("src/module_c.rs"),
                component_type: ComponentType::RustModule { is_public: true },
                dependencies: vec![
                    Dependency {
                        target_component_id: comp1_id,
                        dependency_type: DependencyType::Imports,
                        properties: HashMap::new(),
                    }
                ],
                metrics: ComponentMetrics::default(),
                group: Some("core".to_string()),
            },
        ];

        let cycles = vec![vec![comp1_id, comp2_id, comp3_id]];
        let cycle_edges = vec![(comp3_id, comp1_id)]; // The edge that creates the cycle

        (components, cycles, cycle_edges)
    }

    #[test]
    fn test_enhanced_component_type_language_detection() {
        let rust_component = ComponentType::RustStruct { fields: vec![] };
        let python_component = ComponentType::PythonClass { 
            bases: vec![], 
            methods: vec![], 
            is_abstract: false 
        };
        let js_component = ComponentType::JavaScriptEsModule { exports: vec![] };

        assert_eq!(rust_component.language(), Some("rust"));
        assert_eq!(python_component.language(), Some("python"));
        assert_eq!(js_component.language(), Some("javascript"));
        assert!(rust_component.is_language_specific());
        assert!(python_component.is_language_specific());
        assert!(js_component.is_language_specific());
    }

    #[test]
    fn test_component_complexity_scoring() {
        let simple_component = ComponentType::RustFunction { 
            is_async: false, 
            is_const: false, 
            visibility: Visibility::Public 
        };
        let complex_component = ComponentType::RustStruct { 
            fields: vec![
                FieldInfo {
                    name: "field1".to_string(),
                    field_type: "String".to_string(),
                    visibility: Visibility::Public,
                    is_optional: false,
                },
                FieldInfo {
                    name: "field2".to_string(),
                    field_type: "i32".to_string(),
                    visibility: Visibility::Private,
                    is_optional: true,
                },
            ]
        };

        assert!(complex_component.complexity_score() > simple_component.complexity_score());
        assert_eq!(simple_component.complexity_score(), 2);
        assert_eq!(complex_component.complexity_score(), 5); // 3 base + 2 fields
    }

    #[test]
    fn test_mermaid_generator_initialization() {
        let generator = MermaidGenerator::new();
        assert!(generator.is_ok());
    }

    #[test]
    fn test_basic_component_diagram_generation() {
        let generator = MermaidGenerator::new().expect("Failed to create generator");
        let components = create_test_components();

        let result = generator.generate_diagram(
            &components,
            DiagramType::Component,
            None,
        );

        assert!(result.is_ok());
        let diagram = result.unwrap();
        assert_eq!(diagram.diagram_type, DiagramType::Component);
        assert!(!diagram.mermaid_src.is_empty());
        assert!(diagram.mermaid_src.contains("graph"));
        assert!(diagram.components.len() == components.len());
    }

    #[test]
    fn test_god_object_diagram_generation() {
        let generator = MermaidGenerator::new().expect("Failed to create generator");
        let components = create_test_components();
        
        // Mark the first component as a God Object
        let god_object_components = vec![components[0].component_id];
        let mut member_counts = HashMap::new();
        member_counts.insert(components[0].component_id, 50); // High member count

        let result = generator.generate_god_object_diagram(
            &components,
            &god_object_components,
            &member_counts,
        );

        assert!(result.is_ok());
        let diagram = result.unwrap();
        assert!(!diagram.mermaid_src.is_empty());
        assert!(diagram.mermaid_src.contains("50 members"));
        assert!(diagram.mermaid_src.contains("god-object-critical"));
    }

    #[test]
    fn test_cyclic_dependencies_diagram_generation() {
        let generator = MermaidGenerator::new().expect("Failed to create generator");
        let (components, cycles, cycle_edges) = create_cyclic_test_components();

        let result = generator.generate_cyclic_dependencies_diagram(
            &components,
            &cycles,
            &cycle_edges,
        );

        assert!(result.is_ok());
        let diagram = result.unwrap();
        assert!(!diagram.mermaid_src.is_empty());
        assert!(diagram.mermaid_src.contains("cycle-critical"));
        assert!(diagram.mermaid_src.contains("Cycle 1"));
    }

    #[test]
    fn test_dead_code_diagram_generation() {
        let generator = MermaidGenerator::new().expect("Failed to create generator");
        let components = create_test_components();
        
        // Mark the last component as dead code
        let dead_code_components = vec![components[2].component_id];
        let mut usage_metrics = HashMap::new();
        usage_metrics.insert(components[0].component_id, 0.8); // Active
        usage_metrics.insert(components[1].component_id, 0.2); // Low usage
        usage_metrics.insert(components[2].component_id, 0.0); // Dead

        let result = generator.generate_dead_code_diagram(
            &components,
            &dead_code_components,
            &usage_metrics,
        );

        assert!(result.is_ok());
        let diagram = result.unwrap();
        assert!(!diagram.mermaid_src.is_empty());
        assert!(diagram.mermaid_src.contains("dead-code-critical"));
        assert!(diagram.mermaid_src.contains("low-usage"));
    }

    #[test]
    fn test_large_class_diagram_generation() {
        let generator = MermaidGenerator::new().expect("Failed to create generator");
        let components = create_test_components();
        
        let mut size_metrics = HashMap::new();
        size_metrics.insert(components[0].component_id, (600, 15.5)); // Extra large
        size_metrics.insert(components[1].component_id, (350, 8.2));  // Large
        size_metrics.insert(components[2].component_id, (80, 2.1));   // Small

        let result = generator.generate_large_class_diagram(
            &components,
            &size_metrics,
        );

        assert!(result.is_ok());
        let diagram = result.unwrap();
        assert!(!diagram.mermaid_src.is_empty());
        assert!(diagram.mermaid_src.contains("600 LOC"));
        assert!(diagram.mermaid_src.contains("large-class-critical"));
        assert!(diagram.mermaid_src.contains("Size Legend"));
    }

    #[test]
    fn test_tight_coupling_diagram_generation() {
        let generator = MermaidGenerator::new().expect("Failed to create generator");
        let components = create_test_components();
        
        let mut coupling_metrics = HashMap::new();
        coupling_metrics.insert(components[0].component_id, (5, 12)); // High coupling
        coupling_metrics.insert(components[1].component_id, (3, 6));  // Medium coupling
        coupling_metrics.insert(components[2].component_id, (2, 4));  // Low coupling

        let mut coupling_strengths = HashMap::new();
        coupling_strengths.insert((components[0].component_id, components[1].component_id), 0.9);

        let result = generator.generate_tight_coupling_diagram(
            &components,
            &coupling_metrics,
            &coupling_strengths,
        );

        assert!(result.is_ok());
        let diagram = result.unwrap();
        assert!(!diagram.mermaid_src.is_empty());
        assert!(diagram.mermaid_src.contains("17 connections")); // 5 + 12
        assert!(diagram.mermaid_src.contains("high-coupling"));
        assert!(diagram.mermaid_src.contains("very_high"));
    }

    #[test]
    fn test_diagram_with_severity_styling() {
        let generator = MermaidGenerator::new().expect("Failed to create generator");
        let components = create_test_components();
        
        let mut severity_data = HashMap::new();
        severity_data.insert(components[0].component_id, "critical".to_string());
        severity_data.insert(components[1].component_id, "high".to_string());

        let result = generator.generate_diagram(
            &components,
            DiagramType::Component,
            Some(&severity_data),
        );

        assert!(result.is_ok());
        let diagram = result.unwrap();
        assert!(!diagram.mermaid_src.is_empty());
        assert!(diagram.mermaid_src.contains("severity-critical"));
        assert!(diagram.mermaid_src.contains("severity-high"));
    }

    #[test]
    fn test_multiple_diagram_types() {
        let generator = MermaidGenerator::new().expect("Failed to create generator");
        let components = create_test_components();

        let diagram_types = vec![
            DiagramType::Component,
            DiagramType::Dependency,
            DiagramType::Sequence,
        ];

        for diagram_type in diagram_types {
            let result = generator.generate_diagram(
                &components,
                diagram_type.clone(),
                None,
            );
            
            assert!(result.is_ok(), "Failed to generate {:?}", diagram_type);
            let diagram = result.unwrap();
            assert_eq!(diagram.diagram_type, diagram_type);
            assert!(!diagram.mermaid_src.is_empty());
        }
    }

    #[test]
    fn test_diagram_metadata_validation() {
        let generator = MermaidGenerator::new().expect("Failed to create generator");
        let components = create_test_components();

        let result = generator.generate_diagram(
            &components,
            DiagramType::Component,
            None,
        );

        assert!(result.is_ok());
        let diagram = result.unwrap();
        
        // Validate metadata
        assert_eq!(diagram.diagram_type, DiagramType::Component);
        assert!(diagram.image_path.is_none()); // No image rendering in this test
        assert_eq!(diagram.components.len(), components.len());
        assert!(diagram.generated_at <= chrono::Utc::now());
        assert!(diagram.validation_metrics.is_none()); // Not implemented yet
    }

    #[test]
    fn test_template_error_handling() {
        // This test would require invalid template setup, 
        // but we'll test the error types are properly defined
        let error = MermaidGenerationError::TemplateRenderError("Test error".to_string());
        assert!(error.to_string().contains("Test error"));
        
        let error2 = MermaidGenerationError::TemplateLoadError("Load error".to_string());
        assert!(error2.to_string().contains("Load error"));
    }
}
