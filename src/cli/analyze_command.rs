use std::path::PathBuf;
use clap::Args;
use log::{info, error, warn};
use chrono::Utc;

use crate::database::crud::Database;
use crate::analysis::analysis_engine::AnalysisEngine;
use crate::ai::engine::AiAnalysisEngine;
use crate::analysis::AnalysisError;
use crate::report::ReportGenerator;

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
    pub async fn execute(&self) -> Result<(), AnalysisError> {
        if !self.path.exists() {
            return Err(AnalysisError::InvalidInputPath { path: self.path.clone() });
        }
        info!("Starting analysis of: {}", self.path.display());
        
        // Initialize components
        let mut database = Database::new()?;
        let mut analysis_engine = AnalysisEngine::new()?;
        let mut ai_engine = AiAnalysisEngine::new();
        
        // Configure AI if enabled
        if self.enable_ai {
            if let Some(api_key) = &self.openai_api_key {
                ai_engine = ai_engine.with_openai_api(api_key.clone());
            } else {
                // Try to configure Ollama if no OpenAI key is provided
                let ollama_api_url = self.ollama_api_url.clone()
                    .or_else(|| std::env::var("OLLAMA_API_URL").ok())
                    .unwrap_or_else(|| "http://localhost:11434".to_string());
                let ollama_model = self.ollama_model.clone()
                    .or_else(|| std::env::var("OLLAMA_MODEL").ok())
                    .unwrap_or_else(|| "deepseek-coder:6.7b-instruct-q4_0".to_string());
                ai_engine = ai_engine.with_ollama(&ollama_model, &ollama_api_url);
                warn!("AI analysis enabled with local Ollama model: {} at {}", ollama_model, ollama_api_url);
            }
        }
        
        // Store anti-pattern types in DB
        for mut anti_pattern_type in analysis_engine.get_anti_pattern_types() {
            database.store_anti_pattern_type(&mut anti_pattern_type)?;
        }

        // Create analysis run
        let mut analysis_run = database.create_analysis_run(&self.path)?;
        
        // Run analysis
        let mut issues = analysis_engine.analyze(&self.path).await?;
        
        // Enhance with AI analysis
        if self.enable_ai {
            info!("Enhancing issues with AI analysis...");
            // This loop needs to be careful about borrowing `issues` and `parsed_file`
            // For now, we'll assume `analyze_issue` can work without re-parsing the file
            // or that `ParsedFile` can be retrieved/cloned if needed.
            // A more robust solution would involve passing `ParsedFile` or its relevant parts
            // along with the issue from the analysis engine.
            for issue in &mut issues {
                // For now, pass a dummy AST for AI analysis
                let dummy_ast = crate::ast::CustomAst::default();
                match ai_engine.analyze_issue(issue, &dummy_ast).await {
                    Ok(_) => {},
                    Err(e) => error!("AI analysis failed for issue in {}: {}", issue.file_path, e),
                }
            }
        }
        
        // Update analysis run with results
        analysis_run.total_files_analyzed = Some(analysis_engine.get_files_analyzed());
        analysis_run.total_issues_found = Some(issues.len() as i32);
        analysis_run.end_time = Some(Utc::now());
        analysis_run.status = "completed".to_string();
        database.update_analysis_run(&analysis_run)?;

        // Store results
        database.store_issues(&issues)?;
        
        // Generate report
        let report_generator = ReportGenerator::new();
        let report = match self.output_format.as_str() {
            "json" => report_generator.generate_json_report(&analysis_run, &issues)?.to_string(),
            "markdown" => report_generator.generate_markdown_report(&analysis_run, &issues)?,
            _ => return Err(AnalysisError::UnsupportedOutputFormat(self.output_format.clone())),
        };
        
        // Output report
        if let Some(output_path) = &self.output {
            std::fs::write(output_path, report)?;
            println!("Report written to: {}", output_path.display());
        } else {
            println!("{}", report);
        }
        
        Ok(())
    }
}



