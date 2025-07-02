use async_trait::async_trait;
use clap::Args;
use log::info;

#[async_trait]
pub trait LocalAiSetup {
    async fn is_ollama_installed(&self) -> bool;
    async fn is_ollama_running(&self) -> bool;
    async fn download_model(&self, model: &str);
}

pub struct OllamaSetup;

#[async_trait]
impl LocalAiSetup for OllamaSetup {
    async fn is_ollama_installed(&self) -> bool {
        std::process::Command::new("ollama")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    async fn is_ollama_running(&self) -> bool {
        reqwest::get("http://localhost:11434/api/tags")
            .await
            .is_ok()
    }

    async fn download_model(&self, model: &str) {
        println!(
            "Ollama is installed and running. [stub] Would download model: {}",
            model
        );
    }
}

/// Command to initialize and set up local AI (Ollama)
#[derive(Args)]
pub struct InitLocalAiCommand {
    /// Optional: Model name to download and configure
    #[arg(long, default_value = "mistral:7b-instruct-v0.2-q4_K_M")]
    pub model: String,
}

impl InitLocalAiCommand {
    pub async fn execute<T: LocalAiSetup>(
        &self,
        setup: &T,
    ) -> Result<(), crate::error::UveddiError> {
        info!("Initializing local AI (Ollama) with model: {}", self.model);

        if !setup.is_ollama_installed().await {
            println!("Ollama is not installed. Please install Ollama from https://ollama.com/download and ensure it is in your PATH.");
            return Ok(());
        }

        if !setup.is_ollama_running().await {
            println!("Ollama server is not running. Please start it with `ollama serve` in another terminal.");
            return Ok(());
        }

        setup.download_model(&self.model).await;
        Ok(())
    }
}
