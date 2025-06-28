use crate::ast::tree_sitter::{AstParser, ParsedFile};
use crate::analysis::{AnalysisDetector, AnalysisError};
use crate::analysis::anti_patterns::god_object_detector::GodObjectDetector;
use crate::database::models::ArchitecturalIssue;
use std::path::{Path, PathBuf};
use log::{info, warn};

pub struct AnalysisEngine {
    ast_parser: AstParser,
    detectors: Vec<Box<dyn AnalysisDetector>>,
    files_analyzed: i32,
}

impl AnalysisEngine {
    pub fn new() -> Result<Self, AnalysisError> {
        Ok(Self {
            ast_parser: AstParser::new()?,
            detectors: vec![
                Box::new(GodObjectDetector::new(15, 20)),
            ],
            files_analyzed: 0,
        })
    }

    pub async fn analyze_directory(&mut self, path: &Path) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut all_issues = Vec::new();
        self.files_analyzed = 0;

        for entry in walkdir::WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file()) {
            
            let file_path = entry.path();
            info!("Analyzing file: {}", file_path.display());

            match self.ast_parser.parse_file(file_path) {
                Ok(parsed_file) => {
                    self.files_analyzed += 1;
                    for detector in &self.detectors {
                        match detector.detect_issues(&parsed_file) {
                            Ok(mut issues) => all_issues.append(&mut issues),
                            Err(e) => warn!("Error running detector {} on {}: {}", detector.get_detector_name(), file_path.display(), e),
                        }
                    }
                },
                Err(e) => warn!("Failed to parse file {}: {}", file_path.display(), e),
            }
        }
        info!("Analyzed {} files.", self.files_analyzed);
        Ok(all_issues)
    }

    pub fn get_anti_pattern_types(&self) -> Vec<crate::database::models::AntiPatternType> {
        let mut types = Vec::new();
        for detector in &self.detectors {
            types.extend(detector.get_anti_pattern_types());
        }
        types
    }

    pub fn get_files_analyzed(&self) -> i32 {
        self.files_analyzed
    }
}
