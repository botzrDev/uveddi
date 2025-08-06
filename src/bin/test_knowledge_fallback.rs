//! Test the knowledge-only fallback for AI explanations
//! 
//! This binary tests the knowledge-based explanation generation when AI models aren't available

#[cfg(feature = "ai")]
use uveddi::ai::engine::AiAnalysisEngine;
#[cfg(feature = "ai")]
use uveddi::ai::knowledge::types::KnowledgeContext;
#[cfg(feature = "ai")]
use uveddi::database::models::ArchitecturalIssue;
use crate::core::logging::LevelFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::Builder::from_default_env()
        .filter_level(LevelFilter::Info)
        .init();

    println!("📚 Knowledge-Only Fallback Test");
    println!("{}", "=".repeat(35));

    #[cfg(feature = "ai")]
    {
        // Create AI engine 
        let ai_engine = AiAnalysisEngine::new();
        
        // Create a test issue for God Object pattern
        let mut test_issue = ArchitecturalIssue {
            issue_id: None,
            analysis_run_id: 1,
            anti_pattern_type_id: 1,
            file_path: "test_file.py".to_string(),
            start_line: Some(10),
            end_line: Some(50),
            severity: "CRITICAL".to_string(),
            description: "God Object detected: VeryLargeClassWithTooManyMethods has 25 methods and 15 fields".to_string(),
            code_snippet: Some("class VeryLargeClassWithTooManyMethods:\n    def __init__(self):\n        self.field1 = None\n        # ... 15 fields total\n    \n    def method1(self): pass\n    def method2(self): pass\n    # ... 25 methods total".to_string()),
            ai_explanation: None,
        };

        // Create mock knowledge context for God Object pattern
        let knowledge_context = create_mock_knowledge_context();

        println!("🧪 Testing knowledge-only explanation generation...");
        
        // Test knowledge-enhanced analysis (will fallback to knowledge-only if no AI)
        match ai_engine.analyze_issue_with_knowledge(&mut test_issue, &knowledge_context).await {
            Ok(()) => {
                if let Some(explanation) = &test_issue.ai_explanation {
                    println!("✅ Knowledge-based explanation generated successfully");
                    println!("📝 Explanation length: {} characters", explanation.len());
                    println!("\n📄 Generated Explanation:");
                    println!("{}", "─".repeat(50));
                    println!("{}", explanation);
                    println!("{}", "─".repeat(50));
                } else {
                    println!("❌ No explanation generated");
                }
            },
            Err(e) => {
                println!("❌ Explanation generation failed: {}", e);
            }
        }
    }
    
    #[cfg(not(feature = "ai"))]
    {
        println!("❌ AI feature not enabled. Run with:");
        println!("  cargo run --bin test_knowledge_fallback --features ai");
    }

    println!("\n✅ Knowledge fallback test completed!");
    Ok(())
}

#[cfg(feature = "ai")]
fn create_mock_knowledge_context() -> KnowledgeContext {
    use uveddi::ai::knowledge::schema::{AntiPatternCategory, ImpactLevel, PatternKnowledge};
    use uveddi::ai::knowledge::compression::CompressedString;
    use std::collections::HashMap;
    
    let god_object_pattern = PatternKnowledge {
        id: "god_object".to_string(),
        name: "God Object".to_string(),
        definition: CompressedString::new("A class that knows too much or does too much. This anti-pattern violates the Single Responsibility Principle by concentrating too many responsibilities in a single class."),
        impact: ImpactLevel::High,
        category: AntiPatternCategory::ObjectOriented,
        symptoms: vec![
            CompressedString::new("Class has excessive number of methods (>20)"),
            CompressedString::new("Class has excessive number of fields (>10)"),
            CompressedString::new("Class file exceeds reasonable size (>500 lines)"),
            CompressedString::new("Class has dependencies on many other classes"),
            CompressedString::new("Class handles multiple unrelated concerns"),
        ],
        detection_methods: vec![],  // Simplified for test
        solutions: vec![],  // Simplified for test
        examples: uveddi::ai::knowledge::schema::CodeExamples {
            primary: vec![],
            variations: HashMap::new(),
        },
        language_variations: HashMap::new(),
        related_patterns: vec![],
        tags: vec![],
        frequency_score: 0.8,
        detection_confidence: 0.9,
    };

    KnowledgeContext {
        selected_patterns: vec![god_object_pattern],
        relevance_score: 0.95,
        selection_time_ms: 150,
        library_version: "test-v1.0".to_string(),
        token_count: 500,
    }
}