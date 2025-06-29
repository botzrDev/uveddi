use clap::Args;
use log::info;

/// Command to initialize and set up local AI (Ollama)
#[derive(Args)]
pub struct InitLocalAiCommand {
    /// Optional: Model name to download and configure
    #[arg(long, default_value = "mistral:7b-instruct-v0.2-q4_K_M")]
    pub model: String,
}

impl InitLocalAiCommand {
    pub async fn execute(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Initializing local AI (Ollama) with model: {}", self.model);
        // Check if Ollama is installed
        let ollama_installed = std::process::Command::new("ollama")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if !ollama_installed {
            println!("Ollama is not installed. Please install Ollama from https://ollama.com/download and ensure it is in your PATH.");
            return Ok(());
        }
        // Check if Ollama is running
        let ollama_running = reqwest::get("http://localhost:11434/api/tags").await.is_ok();
        if !ollama_running {
            println!("Ollama server is not running. Please start it with `ollama serve` in another terminal.");
            return Ok(());
        }
        // Download model if needed (stub)
        println!("Ollama is installed and running. [stub] Would download model: {}", self.model);
        Ok(())
    }
}
