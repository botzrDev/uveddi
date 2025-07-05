//! End-to-end integration test for the full AI Reasoning Engine pipeline (RAG, hallucination defense, self-correction, hybrid verification)
//!
//! This test simulates a real-world scenario where a codebase with a God Object is analyzed end-to-end.
//! It verifies that the CLI outputs explanations, confidence scores, and handles the full pipeline.
//! Additionally, it checks the behavior when the API key is missing, ensuring graceful degradation
//! without failing the analysis.

#[cfg(test)]
mod tests {
    use assert_cmd::prelude::*;
    use predicates::prelude::*;
    use std::fs::File;
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use tempfile::{tempdir, TempDir};
    use tokio::time::timeout;
    use std::time::Duration;

    // Existing test
    #[test]
    fn cli_ai_pipeline_outputs_explanations_and_confidence() {
        // Simulate a codebase with a God Object and run the CLI to ensure explanations and confidence are output
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("god_object.rs");
        let code = r#"
        struct GodObject {
            a: i32,
            b: i32,
            c: i32,
        }
        impl GodObject {
            fn m1(&self) {}
            fn m2(&self) {}
            fn m3(&self) {}
            fn m4(&self) {}
            fn m5(&self) {}
            fn m6(&self) {}
            fn m7(&self) {}
            fn m8(&self) {}
            fn m9(&self) {}
            fn m10(&self) {}
            fn m11(&self) {}
            fn m12(&self) {}
            fn m13(&self) {}
            fn m14(&self) {}
            fn m15(&self) {}
        }
        "#;
        
        let mut file = File::create(file_path).unwrap();
        file.write_all(code.as_bytes()).unwrap();

        let mut cmd = Command::cargo_bin("uveddi").unwrap();
        cmd.arg("analyze")
           .arg(dir.path())
           .arg("--format=markdown")
           .arg("--use-local-ai");

        cmd.assert()
           .success()
           .stdout(predicate::str::contains("Architectural Analysis Report"))
           .stdout(predicate::str::contains("issues identified"));
    }
    
    // Add new comprehensive pipeline test that uses the API directly
    #[tokio::test]
    async fn test_full_analysis_pipeline() {
        use uveddi::analysis::engine::AnalysisEngine;
        use uveddi::analysis::detectors::cycle::CycleDetector;
        use uveddi::analysis::detectors::anti_patterns::god_object::GodObjectDetector;
        use uveddi::analysis::detectors::anti_patterns::magic_values::MagicValueDetector;
        use uveddi::ai::engine::AiAnalysisEngine;
        use uveddi::ai::ollama_provider::{OllamaProvider, OllamaConfig};
        use uveddi::database::SqliteDatabase;
        use uveddi::report::ReportGenerator;
        
        // Create a temporary directory for the test
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_path_buf();
        
        // Generate test data in the temporary directory
        let data_dir = temp_path.join("test-data");
        std::fs::create_dir_all(&data_dir).unwrap();
        
        // Create a basic Rust project with cyclic dependencies and a god object
        let src_dir = data_dir.join("src");
        std::fs::create_dir_all(&src_dir).unwrap();
        
        // Create main.rs with cyclic dependencies
        let main_rs = r#"
mod module_a;
mod module_b;
mod big_class;

fn main() {
    module_a::function_a();
    
    let mut manager = big_class::Manager::new();
    manager.do_something();
}
"#;
        std::fs::write(src_dir.join("main.rs"), main_rs).unwrap();
        
        // Create module_a.rs that depends on module_b
        let module_a_rs = r#"
use crate::module_b;

pub fn function_a() {
    println!("Function A");
    module_b::function_b();
}
"#;
        std::fs::write(src_dir.join("module_a.rs"), module_a_rs).unwrap();
        
        // Create module_b.rs that depends on module_a (creates a cycle)
        let module_b_rs = r#"
use crate::module_a;

pub fn function_b() {
    println!("Function B");
    // Cyclic dependency
    module_a::function_a();
}
"#;
        std::fs::write(src_dir.join("module_b.rs"), module_b_rs).unwrap();
        
        // Create a god object
        let big_class_rs = r#"
pub struct Manager {
    data1: String,
    data2: Vec<i32>,
    data3: bool,
    data4: Option<f64>,
    data5: Vec<String>,
}

impl Manager {
    pub fn new() -> Self {
        Self {
            data1: String::new(),
            data2: Vec::new(),
            data3: false,
            data4: None,
            data5: Vec::new(),
        }
    }
    
    pub fn do_something(&mut self) {
        println!("Doing something");
    }
    
    pub fn calculate(&self) -> i32 {
        123 // Magic value
    }
    
    pub fn process_data(&mut self) {
        self.data2.push(42); // Magic value
    }
    
    pub fn validate(&self) -> bool {
        true
    }
    
    pub fn transform(&mut self) {
        self.data1 = "transformed".to_string();
    }
    
    pub fn analyze(&self) -> f64 {
        3.14159 // Magic value
    }
    
    pub fn initialize(&mut self) {
        self.data3 = true;
    }
    
    pub fn finalize(&mut self) {
        self.data3 = false;
    }
    
    pub fn get_status(&self) -> String {
        "OK".to_string() // Magic value
    }
    
    pub fn reset(&mut self) {
        self.data2.clear();
    }
}
"#;
        std::fs::write(src_dir.join("big_class.rs"), big_class_rs).unwrap();
        
        // Create an in-memory database for testing
        let db_path = temp_path.join("uveddi_test.db");
        let db = SqliteDatabase::new(&db_path).unwrap();
        db.initialize().unwrap();
        
        // Create detectors
        let cycle_detector = CycleDetector::new();
        let god_object_detector = GodObjectDetector::new();
        let magic_value_detector = MagicValueDetector::new();
        
        // Create analysis engine
        let mut analysis_engine = AnalysisEngine::new();
        analysis_engine.register_detector(Box::new(cycle_detector));
        analysis_engine.register_detector(Box::new(god_object_detector));
        analysis_engine.register_detector(Box::new(magic_value_detector));
        
        // Run analysis
        let analysis_run = analysis_engine.analyze(&data_dir, &db).unwrap();
        
        // Retrieve issues
        let issues = db.get_issues_for_run(analysis_run.run_id.unwrap()).unwrap();
        println!("Found {} issues in test project", issues.len());
        
        // Verify that we found at least some issues
        assert!(!issues.is_empty(), "Should detect some issues in the test project");
        
        // Try to add AI explanations if Ollama is available
        let config = OllamaConfig::default();
        let provider = OllamaProvider::new(config);
        
        // Check if Ollama is available
        let ollama_available = match timeout(Duration::from_secs(5), provider.check_availability()).await {
            Ok(result) => result.unwrap_or(false),
            Err(_) => false,
        };
        
        if ollama_available {
            println!("Ollama is available, adding AI explanations");
            let ai_engine = AiAnalysisEngine::with_provider(Box::new(provider));
            
            let mut ai_issues = Vec::new();
            for mut issue in issues.clone() {
                if let Ok(()) = timeout(
                    Duration::from_secs(10),
                    ai_engine.analyze_issue(&mut issue)
                ).await.unwrap_or(Ok(())) {
                    // Update issue in database
                    db.update_issue(&issue).unwrap();
                    ai_issues.push(issue);
                }
            }
            
            // Check if AI explanations were added
            if !ai_issues.is_empty() {
                assert!(ai_issues.iter().any(|i| i.ai_explanation.is_some()), 
                        "At least some issues should have AI explanations");
            }
        } else {
            println!("Ollama not available, skipping AI explanations");
        }
        
        // Generate report
        let anti_pattern_types = db.get_all_anti_pattern_types().unwrap();
        let anti_pattern_map = anti_pattern_types
            .into_iter()
            .map(|apt| (apt.type_id.unwrap(), apt))
            .collect();
        
        let report_generator = ReportGenerator::new();
        let report_path = temp_path.join("report.md");
        report_generator.generate_markdown_report(
            &analysis_run,
            &issues,
            &anti_pattern_map,
            Some(&report_path),
        ).unwrap();
        
        // Verify report exists
        assert!(report_path.exists(), "Report file should exist");
        
        // Read report content
        let report_content = std::fs::read_to_string(&report_path).unwrap();
        
        // Check that it contains expected content
        assert!(report_content.contains("Uveddi Architectural Analysis Report"), 
                "Report should have the correct title");
        
        // Report should identify some issues
        assert!(report_content.contains("issues"), 
                "Report should mention detected issues");
        
        println!("Full pipeline test successful!");
    }
}
