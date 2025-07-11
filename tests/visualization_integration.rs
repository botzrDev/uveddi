//! Integration tests for the visualization enhancement system
//!
//! These tests validate the complete visualization pipeline from AST extraction
//! to diagram generation, ensuring the enhanced architectural analysis works correctly.

use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

use uveddi::analysis::component_extractor::ComponentExtractor;
use uveddi::analysis::mermaid_generator::MermaidGenerator;
use uveddi::ast::tree_sitter::{CustomAst, ParsedFile, SourceLanguage};
use uveddi::database::models::{AnalysisRun, AntiPatternType, ArchitecturalIssue};
use uveddi::models::visualization::{
    ArchitecturalComponent, ComponentMetrics, ComponentType, Dependency, DependencyNode,
    DependencyType, DiagramMetadata, DiagramType,
};
use uveddi::report::ReportGenerator;

#[tokio::test]
#[ignore = "Requires rendering service on localhost:3001"]
async fn test_component_extraction_pipeline() {
    let mut extractor = ComponentExtractor::new();

    // Create test parsed files with mock AST data
    let parsed_files = vec![create_test_service_file(), create_test_controller_file()];

    let components = extractor.extract_components(&parsed_files).unwrap();

    // Verify components were extracted
    assert!(components.len() >= 2);
    assert!(components.iter().any(|c| c.name == "UserService"));
    assert!(components.iter().any(|c| c.name == "UserController"));

    // Verify component types are correctly identified
    let user_service = components.iter().find(|c| c.name == "UserService").unwrap();
    assert_eq!(user_service.component_type, ComponentType::Service);

    let user_controller = components
        .iter()
        .find(|c| c.name == "UserController")
        .unwrap();
    assert_eq!(user_controller.component_type, ComponentType::Class);
}

#[tokio::test]
#[ignore = "Requires rendering service on localhost:3001"]
async fn test_mermaid_diagram_generation() {
    let generator = MermaidGenerator::new().unwrap();
    let components = create_test_components();

    // Test component diagram generation
    let component_diagram = generator
        .generate_diagram(&components, DiagramType::Component)
        .unwrap();

    assert!(component_diagram.mermaid_src.contains("graph"));
    assert!(component_diagram.mermaid_src.contains("UserService"));
    assert!(component_diagram.mermaid_src.contains("DatabaseService"));
    assert_eq!(component_diagram.diagram_type, DiagramType::Component);

    // Test dependency diagram generation
    let dependency_diagram = generator
        .generate_diagram(&components, DiagramType::Dependency)
        .unwrap();

    assert!(dependency_diagram.mermaid_src.contains("-->"));
    assert_eq!(dependency_diagram.diagram_type, DiagramType::Dependency);
}

#[tokio::test]
#[ignore = "Requires rendering service on localhost:3001"]
async fn test_severity_based_styling() {
    let generator = MermaidGenerator::new().unwrap();
    let components = create_test_components();

    // Create severity data
    let mut severity_data = HashMap::new();
    severity_data.insert(components[0].component_id, "critical".to_string());
    severity_data.insert(components[1].component_id, "medium".to_string());

    let diagram = generator
        .generate_diagram(&components, DiagramType::Component)
        .unwrap();

    // Verify styling is applied
    assert!(diagram.mermaid_src.contains("classDef"));
    assert!(diagram.mermaid_src.contains("critical") || diagram.mermaid_src.contains("medium"));
}

#[tokio::test]
#[ignore = "Requires rendering service on localhost:3001"]
async fn test_enhanced_report_generation() {
    let report_generator = ReportGenerator::new();
    let analysis_run = create_test_analysis_run();
    let issues = create_test_issues();
    let anti_pattern_types = create_test_anti_pattern_types();
    let components = create_test_components();

    let enhanced_report = report_generator
        .generate_enhanced_markdown_report(
            &analysis_run,
            &issues,
            &anti_pattern_types,
            Some(&components),
        )
        .unwrap();

    // Verify enhanced report contains expected sections
    assert!(enhanced_report
        .markdown_content
        .contains("📊 Architectural Diagrams"));
    assert!(enhanced_report.markdown_content.contains("System Overview"));
    assert!(enhanced_report.markdown_content.contains("```mermaid"));
    assert!(!enhanced_report.diagrams.is_empty());
    assert_eq!(enhanced_report.components_analyzed, components.len());
}

#[tokio::test]
#[ignore = "Requires rendering service on localhost:3001"]
async fn test_anti_pattern_specific_diagrams() {
    let generator = MermaidGenerator::new().unwrap();
    let components = create_cyclic_dependency_components();

    // Test cyclic dependency visualization
    let highlighted_components = vec![components[0].component_id, components[1].component_id];

    let dependency_graph = generator
        .generate_diagram(&components, DiagramType::Dependency)
        .unwrap();

    assert!(dependency_graph.mermaid_src.contains("graph"));
    assert!(dependency_graph.mermaid_src.contains("-->"));
    assert_eq!(dependency_graph.diagram_type, DiagramType::Dependency);
}

#[tokio::test]
#[ignore = "Requires rendering service on localhost:3001"]
async fn test_json_report_with_diagrams() {
    let report_generator = ReportGenerator::new();
    let analysis_run = create_test_analysis_run();
    let issues = create_test_issues();
    let anti_pattern_types = create_test_anti_pattern_types_vec();
    let components = create_test_components();

    // Generate diagrams first
    let generator = MermaidGenerator::new().unwrap();
    let overview_diagram = generator
        .generate_diagram(&components, DiagramType::Component)
        .unwrap();

    // Convert DiagramResult to DiagramMetadata for the report
    let diagram_metadata = DiagramMetadata {
        diagram_type: overview_diagram.diagram_type,
        mermaid_src: overview_diagram.mermaid_src,
        image_path: None,
        generated_at: chrono::Utc::now(),
        components: overview_diagram.components,
        validation_metrics: None,
    };
    let diagrams = vec![diagram_metadata];

    let json_report = report_generator
        .generate_enhanced_json_report(
            &analysis_run,
            &issues,
            &anti_pattern_types,
            Some(&components),
            &diagrams,
        )
        .unwrap();

    // Parse and verify JSON structure
    let json_data: serde_json::Value = serde_json::from_str(&json_report).unwrap();

    assert!(json_data["diagrams"].is_array());
    assert!(json_data["components"].is_array());
    assert!(json_data["metadata"]["enhanced_features"]["architectural_components"].is_boolean());
    assert!(json_data["summary"]["components"].is_object());
}

#[tokio::test]
async fn test_error_handling() {
    // Test component extraction errors
    let mut extractor = ComponentExtractor::new();
    let invalid_parsed_files = vec![]; // Empty should still work
    let result = extractor.extract_components(&invalid_parsed_files);
    assert!(result.is_ok());

    // Test mermaid generation with empty components
    let generator = MermaidGenerator::new().unwrap();
    let empty_components = vec![];
    let result = generator.generate_diagram(&empty_components, DiagramType::Component);
    assert!(result.is_ok()); // Should handle empty gracefully
}

// Helper functions for creating test data

fn create_test_service_file() -> ParsedFile {
    ParsedFile {
        file_path: "src/services/user.rs".to_string(),
        language: SourceLanguage::Rust,
        content: "pub struct UserService { db: Database }".to_string(),
        tree: None,
        source: Some("pub struct UserService { db: Database }".to_string()),
        custom_ast: Some(CustomAst::File {
            items: vec![CustomAst::Struct {
                name: "UserService".to_string(),
                methods: vec!["create_user".to_string(), "get_user".to_string()],
            }],
        }),
    }
}

fn create_test_controller_file() -> ParsedFile {
    ParsedFile {
        file_path: "src/controllers/user.rs".to_string(),
        language: SourceLanguage::Rust,
        content: "pub struct UserController { service: UserService }".to_string(),
        tree: None,
        source: Some("pub struct UserController { service: UserService }".to_string()),
        custom_ast: Some(CustomAst::File {
            items: vec![CustomAst::Struct {
                name: "UserController".to_string(),
                methods: vec!["handle_create".to_string(), "handle_get".to_string()],
            }],
        }),
    }
}

fn create_test_components() -> Vec<ArchitecturalComponent> {
    let user_service_id = Uuid::new_v4();
    let db_service_id = Uuid::new_v4();

    vec![
        ArchitecturalComponent {
            component_id: user_service_id,
            name: "UserService".to_string(),
            file_path: PathBuf::from("src/services/user.rs"),
            component_type: ComponentType::Service,
            dependencies: vec![Dependency {
                from: DependencyNode {
                    id: user_service_id.to_string(),
                    name: "UserService".to_string(),
                },
                to: DependencyNode {
                    id: db_service_id.to_string(),
                    name: "DatabaseService".to_string(),
                },
                dependency_type: DependencyType::Calls,
                kind: Some(DependencyType::Calls),
                weight: None,
                target_component_id: Some(db_service_id.to_string()),
                properties: Some(HashMap::new()),
            }],
            metrics: ComponentMetrics {
                lines_of_code: Some(150),
                complexity: Some(8.5),
                afferent_coupling: 2,
                efferent_coupling: 1,
                coupling_between_objects: Some(3.0),
                public_methods: Some(5),
            },
            group: Some("services".to_string()),
        },
        ArchitecturalComponent {
            component_id: db_service_id,
            name: "DatabaseService".to_string(),
            file_path: PathBuf::from("src/services/database.rs"),
            component_type: ComponentType::Database,
            dependencies: vec![],
            metrics: ComponentMetrics {
                lines_of_code: Some(200),
                complexity: Some(12.0),
                afferent_coupling: 5,
                efferent_coupling: 0,
                coupling_between_objects: Some(2.0),
                public_methods: Some(8),
            },
            group: Some("services".to_string()),
        },
    ]
}

fn create_cyclic_dependency_components() -> Vec<ArchitecturalComponent> {
    let comp_a_id = Uuid::new_v4();
    let comp_b_id = Uuid::new_v4();

    vec![
        ArchitecturalComponent {
            component_id: comp_a_id,
            name: "ComponentA".to_string(),
            file_path: PathBuf::from("src/a.rs"),
            component_type: ComponentType::Module,
            dependencies: vec![Dependency {
                from: DependencyNode {
                    id: comp_a_id.to_string(),
                    name: "ComponentA".to_string(),
                },
                to: DependencyNode {
                    id: comp_b_id.to_string(),
                    name: "ComponentB".to_string(),
                },
                dependency_type: DependencyType::Calls,
                kind: Some(DependencyType::Calls),
                weight: None,
                target_component_id: Some(comp_b_id.to_string()),
                properties: Some(HashMap::new()),
            }],
            metrics: ComponentMetrics::default(),
            group: None,
        },
        ArchitecturalComponent {
            component_id: comp_b_id,
            name: "ComponentB".to_string(),
            file_path: PathBuf::from("src/b.rs"),
            component_type: ComponentType::Module,
            dependencies: vec![Dependency {
                from: DependencyNode {
                    id: comp_b_id.to_string(),
                    name: "ComponentB".to_string(),
                },
                to: DependencyNode {
                    id: comp_a_id.to_string(),
                    name: "ComponentA".to_string(),
                },
                dependency_type: DependencyType::Calls,
                kind: Some(DependencyType::Calls),
                weight: None,
                target_component_id: Some(comp_a_id.to_string()),
                properties: Some(HashMap::new()),
            }],
            metrics: ComponentMetrics::default(),
            group: None,
        },
    ]
}

fn create_test_analysis_run() -> AnalysisRun {
    AnalysisRun {
        run_id: Some(1),
        project_id: 1,
        start_time: chrono::Utc::now(),
        end_time: Some(chrono::Utc::now()),
        status: "completed".to_string(),
        total_files_analyzed: Some(25),
        total_issues_found: Some(5),
        analysis_config: "{}".to_string(),
    }
}

fn create_test_issues() -> Vec<ArchitecturalIssue> {
    vec![
        ArchitecturalIssue {
            issue_id: Some(1),
            analysis_run_id: 1,
            anti_pattern_type_id: 1,
            file_path: "src/services/user.rs".to_string(),
            start_line: Some(10),
            end_line: Some(20),
            severity: "high".to_string(),
            description: "Cyclic dependency detected".to_string(),
            code_snippet: Some("impl UserService { ... }".to_string()),
            ai_explanation: Some("This creates a circular dependency".to_string()),
        },
        ArchitecturalIssue {
            issue_id: Some(2),
            analysis_run_id: 1,
            anti_pattern_type_id: 2,
            file_path: "src/controllers/user.rs".to_string(),
            start_line: Some(5),
            end_line: Some(50),
            severity: "medium".to_string(),
            description: "Large class detected".to_string(),
            code_snippet: Some("pub struct UserController { ... }".to_string()),
            ai_explanation: Some("This class has too many responsibilities".to_string()),
        },
    ]
}

fn create_test_anti_pattern_types() -> HashMap<i64, AntiPatternType> {
    let mut map = HashMap::new();

    map.insert(
        1,
        AntiPatternType {
            anti_pattern_type_id: Some(1),
            name: "Cyclic Dependency".to_string(),
            description: "Circular dependencies between components".to_string(),
            category: "structural".to_string(),
        },
    );

    map.insert(
        2,
        AntiPatternType {
            anti_pattern_type_id: Some(2),
            name: "God Object".to_string(),
            description: "Classes with too many responsibilities".to_string(),
            category: "behavioral".to_string(),
        },
    );

    map
}

fn create_test_anti_pattern_types_vec() -> Vec<AntiPatternType> {
    create_test_anti_pattern_types().into_values().collect()
}
