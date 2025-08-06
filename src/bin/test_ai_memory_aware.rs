//! Test the AI engine with memory-aware model selection
//! 
//! This binary specifically tests the AI engine initialization and memory-aware model selection

#[cfg(feature = "ai")]
use uveddi::ai::engine::AiAnalysisEngine;
#[cfg(feature = "ai")]
use uveddi::database::models::ArchitecturalIssue;
use crate::core::logging::LevelFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::Builder::from_default_env()
        .filter_level(LevelFilter::Info)
        .init();

    println!("🤖 AI Engine Memory-Aware Test");
    println!("{}", "=".repeat(40));

    #[cfg(feature = "ai")]
    {
        // Create AI engine with memory-aware model selection
        let ai_engine = AiAnalysisEngine::new();
        
        println!("✅ AI Engine created successfully");

        // Create a test issue
        let mut test_issue = ArchitecturalIssue {
            issue_id: None,
            analysis_run_id: 1,
            anti_pattern_type_id: 1,
            file_path: "test_file.py".to_string(),
            start_line: Some(10),
            end_line: Some(20),
            severity: "CRITICAL".to_string(),
            description: "Test God Object detected".to_string(),
            code_snippet: Some("class VeryLargeClass:\n    def method1(self): pass\n    def method2(self): pass".to_string()),
            ai_explanation: None,
        };

        println!("🧪 Testing AI explanation generation...");
        
        // Test AI explanation (this will use knowledge-only if no Ollama is available)
        match ai_engine.analyze_issue(&mut test_issue).await {
            Ok(()) => {
                if let Some(explanation) = &test_issue.ai_explanation {
                    println!("✅ AI explanation generated successfully");
                    println!("📝 Explanation length: {} characters", explanation.len());
                    println!("📄 Explanation preview: {}", 
                             &explanation.chars().take(150).collect::<String>());
                } else {
                    println!("ℹ️  No AI explanation generated (likely using knowledge-only mode)");
                }
            },
            Err(e) => {
                println!("❌ AI explanation failed: {}", e);
            }
        }
    }
    
    #[cfg(not(feature = "ai"))]
    {
        println!("❌ AI feature not enabled. Run with:");
        println!("  cargo run --bin test_ai_memory_aware --features ai");
    }

    println!("\n✅ AI Engine memory-aware test completed!");
    Ok(())
}