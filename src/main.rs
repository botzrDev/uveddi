/// CodeAtlas - A tool for code analysis and exploration
/// 
/// This is the main entry point for the CodeAtlas application.

mod database;

use database::DatabaseManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Welcome to CodeAtlas!");
    println!("A Rust-based code analysis and exploration tool.");
    
    // Initialize database
    let _db = DatabaseManager::new(None).await?;
    println!("Database initialized successfully!");
    
    // TODO: Implement core functionality
    // - Code parsing and analysis
    // - Project structure visualization
    // - Dependency mapping
    // - Code metrics calculation
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main_runs() {
        // Test that main function can be called without panicking
        main();
    }
}
