use std::path::Path;
use std::time::Duration;
use uveddi::analysis::AnalysisEngine;

fn main() {
    env_logger::init();
    
    println!("Testing analysis engine hang...");
    
    // Create a simple test that should complete quickly
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    let result = rt.block_on(async {
        println!("Creating analysis engine...");
        let mut engine = AnalysisEngine::new_with_memory_cache()?;
        println!("Engine created successfully");
        
        // Create a simple temp file to analyze
        let temp_dir = tempfile::tempdir()?;
        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(&test_file, "fn main() { println!(\"Hello, world!\"); }")?;
        
        println!("Starting analysis of temp file...");
        let (issues, _graph) = engine.analyze(&test_file).await?;
        println!("Analysis completed! Found {} issues", issues.len());
        
        Ok::<(), Box<dyn std::error::Error>>(())
    });
    
    match result {
        Ok(()) => println!("✅ Analysis completed successfully"),
        Err(e) => eprintln!("❌ Analysis failed: {}", e),
    }
}