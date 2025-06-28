use std::path::Path;
use clap::Args;
use log::{info, error};

use crate::ingestion::file_scanner::FileScanner;
use crate::analysis::dependency_extractor::DependencyExtractor;
use crate::analysis::dependency_graph::DependencyGraph;
use crate::analysis::cycle_detector::CycleDetector;
use crate::report::text_reporter::TextReporter;

#[derive(Args)]
pub struct AnalyzeCommand {
    /// Path to the directory to analyze
    pub path: String,
    /// Show file paths in output
    #[arg(long, default_value = "true")]
    pub show_paths: bool,
    /// Show line numbers in output  
    #[arg(long, default_value = "false")]
    pub show_lines: bool,
    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

impl AnalyzeCommand {
    pub fn execute(&self) -> Result<(), AnalysisError> {
        info!("Starting analysis of: {}", self.path);
        let path = Path::new(&self.path);
        if !path.exists() {
            return Err(AnalysisError::PathNotFound(self.path.clone()));
        }
        let scanner = FileScanner::new();
        let files = scanner.scan_directory(path)
            .map_err(AnalysisError::ScanError)?;
        if files.is_empty() {
            println!("No Rust files found in {}", self.path);
            return Ok(());
        }
        info!("Found {} files to analyze", files.len());
        let extractor = DependencyExtractor::new()
            .map_err(AnalysisError::RegexError)?;
        let mut all_dependencies = Vec::new();
        for file in &files {
            match extractor.extract_from_file(file) {
                Ok(mut deps) => all_dependencies.append(&mut deps),
                Err(e) => {
                    error!("Failed to extract dependencies from {}: {}", file.display(), e);
                }
            }
        }
        info!("Extracted {} dependencies", all_dependencies.len());
        let mut graph = DependencyGraph::new();
        graph.build_from_dependencies(all_dependencies);
        let mut detector = CycleDetector::new();
        let results = detector.detect_cycles(&graph);
        let reporter = TextReporter::new()
            .with_file_paths(self.show_paths)
            .with_line_numbers(self.show_lines);
        reporter.generate_report(&results)
            .map_err(AnalysisError::ReportError)?;
        Ok(())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum AnalysisError {
    #[error("Path not found: {0}")]
    PathNotFound(String),
    #[error("File scanning error: {0}")]
    ScanError(#[from] crate::ingestion::file_scanner::ScanError),
    #[error("Dependency extraction error: {0}")]
    ExtractionError(#[from] crate::analysis::dependency_extractor::ExtractionError),
    #[error("Regular expression error: {0}")]
    RegexError(regex::Error),
    #[error("Report generation error: {0}")]
    ReportError(std::io::Error),
}
