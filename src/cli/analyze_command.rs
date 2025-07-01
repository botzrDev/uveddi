use std::path::PathBuf;
use clap::Args;
use log::{info, error, warn};
use chrono::Utc;

use crate::database::crud::Database;
use crate::analysis::analysis_engine::AnalysisEngine;
use crate::ai::engine::AiAnalysisEngine;
use crate::report::ReportGenerator;
use crate::plugin::initialize_plugins;
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
        if !self.path.exists() {
            return Err(UveddiError::PathNotFound(self.path.display().to_string()))
                .err_context("Input path validation failed");
        }
        info!("Starting analysis of: {}", self.path.display());
        
        // Initialize components with context
        let mut database = Database::new()
            .err_context("Failed to initialize database")?;
        let mut analysis_engine = AnalysisEngine::new()
            .err_context("Failed to initialize analysis engine")?;
        let mut ai_engine = AiAnalysisEngine::new();
        let plugin_manager = initialize_plugins();
        
        // Configure AI if enabled with context
        if self.enable_ai {
            if let Some(api_key) = &self.openai_api_key {
                ai_engine = ai_engine.with_openai_api(api_key.clone());
            } else {
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
        
        // Store anti-pattern types with context
        for mut anti_pattern_type in analysis_engine.get_anti_pattern_types() {
            database.store_anti_pattern_type(&mut anti_pattern_type)
                .err_context("Failed to store anti-pattern type")?;
        }

        // Create analysis run with context
        let mut analysis_run = database.create_analysis_run(&self.path)
            .err_context("Failed to create analysis run")?;
        
        // Run analysis with context
        let (mut issues, dependency_graph) = analysis_engine.analyze(&self.path).await
            .err_context("Analysis failed")?;

        // Run plugins with error logging
        info!("Running analysis plugins...");
        let plugin_results = plugin_manager.run_plugins(&dependency_graph);
        for result in plugin_results {
            match result {
                Ok(plugin_issues) => issues.extend(plugin_issues.into_iter().map(|i| i.into())),
                Err(e) => error!("Plugin execution failed: {}", e),
            }
        }
        
        // Enhance with AI analysis with error logging
        if self.enable_ai {
            info!("Enhancing issues with AI analysis...");
            for issue in &mut issues {
                let dummy_ast = crate::ast::CustomAst::default();
                if let Err(e) = ai_engine.analyze_issue(issue, &dummy_ast).await {
                    error!("AI analysis failed for issue in {}: {}", issue.file_path, e);
                }
            }
        }
        
        // Update analysis run with context
        analysis_run.total_files_analyzed = Some(analysis_engine.get_files_analyzed());
        analysis_run.total_issues_found = Some(issues.len() as i32);
        analysis_run.end_time = Some(Utc::now());
        analysis_run.status = "completed".to_string();
        database.update_analysis_run(&analysis_run)
            .err_context("Failed to update analysis run")?;

        // Store results with context
        database.store_issues(&issues)
            .err_context("Failed to store analysis issues")?;
        
        // Generate report with context
        let report_generator = ReportGenerator::new();
        let report = match self.output_format.as_str() {
            "json" => report_generator.generate_json_report(&analysis_run, &issues)?
                .err_context("JSON report generation failed")?
                .to_string(),
            "markdown" => report_generator.generate_markdown_report(&analysis_run, &issues)?
                .err_context("Markdown report generation failed")?,
            _ => return Err(UveddiError::UnsupportedOutputFormat(self.output_format.clone()))
                .err_context("Unsupported output format specified"),
        };
        
        // Output report with context
        if let Some(output_path) = &self.output {
            std::fs::write(output_path, report)
                .err_context(format!("Failed to write report to {}", output_path.display()))?;
            println!("Report written to: {}", output_path.display());
        } else {
            println!("{}", report);
        }
        
        Ok(())
    }
}
