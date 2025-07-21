use uveddi::analysis::AnalysisEngine;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    println!("Starting simple analysis test...");
    
    let mut engine = AnalysisEngine::new()?;
    println!("Analysis engine created successfully");
    
    let path = Path::new("src/lib.rs");
    println!("Starting analysis of: {}", path.display());
    
    let (issues, graph) = engine.analyze(path).await?;
    println!("Analysis completed! Found {} issues", issues.len());
    
    Ok(())
}