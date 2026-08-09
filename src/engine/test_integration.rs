//! Integration test for the new engine module
//!
//! This tests the detector migration in isolation from the rest of the codebase.

#[cfg(feature = "engine-integration")]
#[cfg(test)]
mod tests {
    use super::super::analysis::context::{FileInfo, ProjectContext};
    use super::super::analysis::{AnalysisContext, AnalysisPipeline, ContextDetectorFactory};
    use super::super::parsing::{AstBuilder, Relation, RelationKind, Symbol, SymbolKind};
    use crate::ast::SourceLanguage;
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::time::SystemTime;

    fn create_test_context() -> AnalysisContext {
        let source = r#"
struct TestStruct {
    field1: i32,
    field2: String,
    field3: bool,
    field4: Vec<i32>,
    field5: Option<String>,
    field6: HashMap<String, i32>,
    field7: Arc<Mutex<i32>>,
    field8: RefCell<String>,
    field9: Box<dyn Display>,
    field10: Rc<Vec<i32>>,
}

impl TestStruct {
    pub fn new() -> Self { todo!() }
    pub fn method1(&self) -> i32 { todo!() }
    pub fn method2(&mut self) { todo!() }
    pub fn method3(&self) -> String { todo!() }
    pub fn method4(&self, x: i32) { todo!() }
    pub fn method5(&mut self) -> bool { todo!() }
    pub fn method6(&self) -> Vec<i32> { todo!() }
}

fn unused_function() -> i32 { 42 }

fn used_function() -> String { "test".to_string() }

fn main() {
    let _ = used_function();
}
"#;

        let mut symbols = vec![Symbol {
            name: "TestStruct".to_string(),
            kind: SymbolKind::Struct,
            line: 2,
            column: 0,
            end_line: 13,
            end_column: 1,
            parent: None,
        }];

        // Add 35 methods to exceed threshold (Rust default is 30)
        for i in 1..=35 {
            symbols.push(Symbol {
                name: format!("method{}", i),
                kind: SymbolKind::Method,
                line: 16 + i,
                column: 4,
                end_line: 16 + i,
                end_column: 35,
                parent: Some("TestStruct".to_string()),
            });
        }

        // Add 25 fields to exceed threshold (Rust default is 20)
        for i in 1..=25 {
            symbols.push(Symbol {
                name: format!("field{}", i),
                kind: SymbolKind::Field,
                line: 3 + i,
                column: 4,
                end_line: 3 + i,
                end_column: 20,
                parent: Some("TestStruct".to_string()),
            });
        }

        // Functions
        symbols.push(Symbol {
            name: "unused_function".to_string(),
            kind: SymbolKind::Function,
            line: 25,
            column: 0,
            end_line: 25,
            end_column: 30,
            parent: None,
        });
        symbols.push(Symbol {
            name: "used_function".to_string(),
            kind: SymbolKind::Function,
            line: 27,
            column: 0,
            end_line: 27,
            end_column: 40,
            parent: None,
        });
        symbols.push(Symbol {
            name: "main".to_string(),
            kind: SymbolKind::Function,
            line: 29,
            column: 0,
            end_line: 31,
            end_column: 1,
            parent: None,
        });

        let relations = vec![Relation {
            from: "main".to_string(),
            to: "used_function".to_string(),
            kind: RelationKind::Calls,
        }];

        let file_info = FileInfo {
            path: PathBuf::from("test.rs"),
            language: SourceLanguage::Rust,
            lines_of_code: source.lines().count(),
            size_bytes: source.len(),
            modified_at: SystemTime::now(),
        };

        let project_context = ProjectContext {
            project_root: PathBuf::from("/test"),
            project_files: vec![],
            dependencies: vec![],
            global_symbols: vec![],
        };

        AnalysisContext::new(
            file_info,
            None,
            source.to_string(),
            symbols,
            relations,
            project_context,
        )
    }

    #[test]
    fn test_context_god_object_detector() {
        let context = create_test_context();
        let ast_builder = Arc::new(AstBuilder::new().expect("Failed to create AstBuilder"));
        let pipeline = Arc::new(AnalysisPipeline::new(ast_builder));
        let factory = ContextDetectorFactory::new(pipeline);

        let god_detector = factory
            .create_context_detector("context_god_object")
            .expect("Failed to create god object detector");

        let issues = god_detector.detect(&context).expect("Detection failed");

        // Should detect TestStruct as a god object (35 methods + 25 fields, exceeding Rust thresholds of 30/20)
        assert!(!issues.is_empty(), "Should detect god object");
        assert!(issues[0].description.contains("TestStruct"));
        assert!(issues[0].anti_pattern_type_id == 1);
    }

    #[test]
    fn test_context_dead_code_detector() {
        let context = create_test_context();
        let ast_builder = Arc::new(AstBuilder::new().expect("Failed to create AstBuilder"));
        let pipeline = Arc::new(AnalysisPipeline::new(ast_builder));
        let factory = ContextDetectorFactory::new(pipeline);

        let dead_code_detector = factory
            .create_context_detector("context_dead_code")
            .expect("Failed to create dead code detector");

        let issues = dead_code_detector
            .detect(&context)
            .expect("Detection failed");

        // Should detect unused_function as dead code
        assert!(!issues.is_empty(), "Should detect dead code");
        let unused_issue = issues
            .iter()
            .find(|issue| issue.description.contains("unused_function"));
        assert!(unused_issue.is_some(), "Should find unused_function issue");
    }

    #[test]
    fn test_context_code_duplication_detector() {
        let context = create_test_context();
        let ast_builder = Arc::new(AstBuilder::new().expect("Failed to create AstBuilder"));
        let pipeline = Arc::new(AnalysisPipeline::new(ast_builder));
        let factory = ContextDetectorFactory::new(pipeline);

        let duplication_detector = factory
            .create_context_detector("context_code_duplication")
            .expect("Failed to create code duplication detector");

        let issues = duplication_detector
            .detect(&context)
            .expect("Detection failed");

        // This test code doesn't have significant duplication, so issues may be empty
        // That's expected behavior
    }

    #[test]
    fn test_analysis_pipeline_with_context_detectors() {
        let context = create_test_context();
        let ast_builder = Arc::new(AstBuilder::new().expect("Failed to create AstBuilder"));
        let pipeline = Arc::new(
            AnalysisPipeline::new(ast_builder.clone()).with_performance_instrumentation(true),
        );
        let factory = ContextDetectorFactory::new(pipeline.clone());

        // Create pipeline with all context detectors
        let analysis_pipeline = AnalysisPipeline::new(ast_builder)
            .with_performance_instrumentation(true)
            .with_detector(
                factory
                    .create_context_detector("context_god_object")
                    .unwrap(),
            )
            .with_detector(
                factory
                    .create_context_detector("context_code_duplication")
                    .unwrap(),
            )
            .with_detector(
                factory
                    .create_context_detector("context_dead_code")
                    .unwrap(),
            );

        let result = analysis_pipeline
            .analyze(context)
            .expect("Pipeline analysis failed");

        // Should have some issues
        assert!(!result.issues.is_empty(), "Pipeline should detect issues");

        // Should have performance metrics
        assert!(
            result.performance_metrics.is_some(),
            "Should have performance metrics"
        );
        let metrics = result.performance_metrics.unwrap();
        assert!(metrics.files_processed > 0, "Should have processed files");
        assert!(
            metrics.detector_times.len() > 0,
            "Should have detector timing data"
        );
    }

    #[test]
    fn test_detector_migration_status() {
        let ast_builder = Arc::new(AstBuilder::new().expect("Failed to create AstBuilder"));
        let pipeline = Arc::new(AnalysisPipeline::new(ast_builder));
        let factory = ContextDetectorFactory::new(pipeline);

        let status = factory.migration_status();

        assert!(status.total_detectors > 0, "Should have total detectors");
        assert!(status.migrated_count > 0, "Should have migrated detectors");
        assert!(
            status.migrated_count <= status.total_detectors,
            "Migrated should not exceed total"
        );
        assert!(status.progress_percentage() > 0.0, "Should have progress");
        assert!(
            status.progress_percentage() <= 100.0,
            "Progress should not exceed 100%"
        );

        // Check specific migrated detectors
        assert!(status
            .migrated_detectors
            .contains(&"GodObjectDetector".to_string()));
        assert!(status
            .migrated_detectors
            .contains(&"CodeDuplicationDetector".to_string()));
        assert!(status
            .migrated_detectors
            .contains(&"DeadCodeDetector".to_string()));
    }
}
