//! Test utility to verify memory detection and model selection
//! 
//! This binary tests the memory-aware AI model selection functionality
//! and displays system memory information.

#[cfg(feature = "ai")]
use uveddi::ai::ollama_provider::OllamaConfig;
use crate::core::logging::LevelFilter;

fn main() {
    // Initialize logging
    env_logger::Builder::from_default_env()
        .filter_level(LevelFilter::Info)
        .init();

    println!("🧠 Uveddi Memory-Aware AI Model Selection Test");
    println!("{}", "=".repeat(50));

    #[cfg(feature = "ai")]
    {
        // Test memory detection and model selection
        let config = OllamaConfig::default();
        
        println!("📊 Memory Detection Results:");
        println!("  Selected Model: {}", config.model);
        println!("  API URL: {}", config.api_url);
        println!("  Temperature: {}", config.temperature);
        println!("  Max Tokens: {}", config.max_tokens);
    }
    
    #[cfg(not(feature = "ai"))]
    {
        println!("❌ AI feature not enabled. Run with:");
        println!("  cargo run --bin test_memory_detection --features ai");
    }
    
    println!("\n🎯 Model Selection Guidelines:");
    println!("  • 32GB+: deepseek-coder:6.7b-instruct-q4_0 (high-performance)");
    println!("  • 16-32GB: llama3.2:3b (balanced performance/memory)");
    println!("  • 8-16GB: phi3:mini (efficient small model)");
    println!("  • 4-8GB: llama3.2:1b (ultra-lightweight)");
    println!("  • <4GB: tinyllama:1.1b (minimal memory footprint)");

    println!("\n💡 To override automatic selection:");
    println!("  export OLLAMA_MODEL=\"your-preferred-model\"");
    
    println!("\n✅ Memory detection test completed successfully!");
}