use std::path::PathBuf;
use clap::Args;
use log::info;

use crate::application::{AnalysisOrchestrator, AnalysisConfig};
use crate::error::{UveddiError, ErrContext};

#[derive(Args)]
pub struct AnalyzeCommand {
    /// Path to analyze
    pub path: PathBuf,
    
    /// Output format (text, json, markdown)
    #[arg(long, default_value = "markdown")]
    pub output_format: String,
    
    /// Output file path
    #[arg(long)]
    pub output: Option<PathBuf>,
    
    /// Enable AI analysis (requires API key or local model)
    #[arg(long)]
    pub enable_ai: bool,
    
    /// OpenAI API key
    #[arg(long, env = "OPENAI_API_KEY")]
    pub openai_api_key: Option<String>,

    /// Ollama API URL (for local AI)
    #[arg(long, env = "OLLAMA_API_URL")]
    pub ollama_api_url: Option<String>,

    /// Ollama model name (for local AI)
    #[arg(long, env = "OLLAMA_MODEL")]
    pub ollama_model: Option<String>,
}

impl AnalyzeCommand {
    pub async fn execute(&self) -> Result<(), UveddiError> {
        info!("Starting analysis of: {}", self.path.display());
        
        // Create application layer orchestrator
        let mut orchestrator = AnalysisOrchestrator::new()
            .err_context("Failed to initialize analysis orchestrator")?;
        
        // Configure analysis parameters
        let config = AnalysisConfig {
            target_path: self.path.clone(),
            output_format: self.output_format.clone(),
            output_file: self.output.clone(),
            enable_ai: self.enable_ai,
            openai_api_key: self.openai_api_key.clone(),
            ollama_api_url: self.ollama_api_url.clone(),
            ollama_model: self.ollama_model.clone(),
        };
        
        // Execute analysis through application layer
        let report = orchestrator.execute_analysis(config).await
            .err_context("Analysis execution failed")?;
        
        // Output results
        if self.output.is_none() {
            println!("{}", report.content);
        }
        
        // Log summary
        info!(
            "Analysis completed: {} files analyzed, {} issues found, AI enhanced: {}", 
            report.metadata.files_analyzed,
            report.metadata.issues_found,
            report.metadata.ai_enhanced
        );
        
        Ok(())
    }
}
