//! Detection of file operation sinks (file writes, path traversal)

use super::TaintSinkDetector;
use crate::analysis::detectors::security::taint_analysis::types::TaintSink;
use crate::analysis::detectors::security::types::SecurityIssueType;
use crate::analysis::AnalysisError;
use crate::ast::{ParsedFile, SourceLanguage};
use std::collections::HashMap;

/// Detector for file operation sinks
pub struct FileSinkDetector {
    patterns: HashMap<SourceLanguage, Vec<String>>,
}

impl FileSinkDetector {
    pub fn new() -> Self {
        let mut detector = Self {
            patterns: HashMap::new(),
        };
        detector.initialize_patterns();
        detector
    }

    fn initialize_patterns(&mut self) {
        // Rust patterns
        self.patterns.insert(
            SourceLanguage::Rust,
            vec![
                "std::fs::write".to_string(),
                "std::fs::File::create".to_string(),
                "std::fs::OpenOptions".to_string(),
                "std::fs::copy".to_string(),
                "std::fs::rename".to_string(),
                "std::fs::remove_file".to_string(),
                "std::fs::remove_dir".to_string(),
                "tokio::fs::write".to_string(),
                "tokio::fs::File::create".to_string(),
                "serde_json::to_writer".to_string(),
                "std::io::Write::write".to_string(),
            ],
        );

        // Python patterns
        self.patterns.insert(
            SourceLanguage::Python,
            vec![
                "open(".to_string(),
                "file.write".to_string(),
                "file.writelines".to_string(),
                "shutil.copy".to_string(),
                "shutil.move".to_string(),
                "os.rename".to_string(),
                "os.remove".to_string(),
                "os.unlink".to_string(),
                "pathlib.Path.write_text".to_string(),
                "pathlib.Path.write_bytes".to_string(),
                "json.dump".to_string(),
                "pickle.dump".to_string(),
            ],
        );

        // JavaScript/TypeScript patterns
        self.patterns.insert(
            SourceLanguage::JavaScript,
            vec![
                "fs.writeFile".to_string(),
                "fs.writeFileSync".to_string(),
                "fs.createWriteStream".to_string(),
                "fs.copyFile".to_string(),
                "fs.rename".to_string(),
                "fs.unlink".to_string(),
                "fs.rmdir".to_string(),
                "path.join".to_string(),
                "path.resolve".to_string(),
                "require('fs')".to_string(),
            ],
        );

        self.patterns.insert(
            SourceLanguage::TypeScript,
            self.patterns
                .get(&SourceLanguage::JavaScript)
                .unwrap()
                .clone(),
        );
    }

    /// Create taint sinks for file write operations
    fn create_file_write_sinks(&self, language: SourceLanguage) -> Vec<TaintSink> {
        match language {
            SourceLanguage::Rust => vec![
                TaintSink::new(
                    "rust_fs_write".to_string(),
                    "std::fs::write".to_string(),
                    SecurityIssueType::PathTraversal,
                    "File write with potential path traversal".to_string(),
                )
                .with_language(language),
                TaintSink::new(
                    "rust_file_create".to_string(),
                    "std::fs::File::create".to_string(),
                    SecurityIssueType::PathTraversal,
                    "File creation with user-controlled path".to_string(),
                )
                .with_language(language),
                TaintSink::new(
                    "rust_tokio_write".to_string(),
                    "tokio::fs::write".to_string(),
                    SecurityIssueType::PathTraversal,
                    "Async file write with potential path traversal".to_string(),
                )
                .with_language(language),
                TaintSink::new(
                    "rust_io_write".to_string(),
                    "std::io::Write::write".to_string(),
                    SecurityIssueType::PathTraversal,
                    "Direct write operation with user data".to_string(),
                )
                .with_language(language),
            ],
            SourceLanguage::Python => vec![
                TaintSink::new(
                    "python_file_open".to_string(),
                    "open(".to_string(),
                    SecurityIssueType::PathTraversal,
                    "File opening with user-controlled path".to_string(),
                )
                .with_language(language)
                .with_vulnerable_params(vec![0]), // First parameter is the file path
                TaintSink::new(
                    "python_file_write".to_string(),
                    "file.write".to_string(),
                    SecurityIssueType::PathTraversal,
                    "File write with user-controlled content".to_string(),
                )
                .with_language(language),
                TaintSink::new(
                    "python_pathlib_write".to_string(),
                    "pathlib.Path.write_text".to_string(),
                    SecurityIssueType::PathTraversal,
                    "Pathlib write with user data".to_string(),
                )
                .with_language(language),
                TaintSink::new(
                    "python_json_dump".to_string(),
                    "json.dump".to_string(),
                    SecurityIssueType::PathTraversal,
                    "JSON file dump with user data".to_string(),
                )
                .with_language(language),
            ],
            SourceLanguage::JavaScript | SourceLanguage::TypeScript => vec![
                TaintSink::new(
                    "js_fs_write".to_string(),
                    "fs.writeFile".to_string(),
                    SecurityIssueType::PathTraversal,
                    "File write with user-controlled path or content".to_string(),
                )
                .with_language(language),
                TaintSink::new(
                    "js_fs_write_sync".to_string(),
                    "fs.writeFileSync".to_string(),
                    SecurityIssueType::PathTraversal,
                    "Synchronous file write with user data".to_string(),
                )
                .with_language(language),
                TaintSink::new(
                    "js_create_stream".to_string(),
                    "fs.createWriteStream".to_string(),
                    SecurityIssueType::PathTraversal,
                    "Write stream creation with user path".to_string(),
                )
                .with_language(language),
            ],
            _ => Vec::new(),
        }
    }

    /// Create taint sinks for file manipulation operations
    fn create_file_manipulation_sinks(&self, language: SourceLanguage) -> Vec<TaintSink> {
        match language {
            SourceLanguage::Rust => vec![
                TaintSink::new(
                    "rust_fs_copy".to_string(),
                    "std::fs::copy".to_string(),
                    SecurityIssueType::PathTraversal,
                    "File copy with user-controlled paths".to_string(),
                )
                .with_language(language),
                TaintSink::new(
                    "rust_fs_rename".to_string(),
                    "std::fs::rename".to_string(),
                    SecurityIssueType::PathTraversal,
                    "File rename with user-controlled paths".to_string(),
                )
                .with_language(language),
                TaintSink::new(
                    "rust_fs_remove".to_string(),
                    "std::fs::remove_file".to_string(),
                    SecurityIssueType::PathTraversal,
                    "File deletion with user-controlled path".to_string(),
                )
                .with_language(language),
            ],
            SourceLanguage::Python => vec![
                TaintSink::new(
                    "python_shutil_copy".to_string(),
                    "shutil.copy".to_string(),
                    SecurityIssueType::PathTraversal,
                    "File copy operation with user paths".to_string(),
                )
                .with_language(language),
                TaintSink::new(
                    "python_os_rename".to_string(),
                    "os.rename".to_string(),
                    SecurityIssueType::PathTraversal,
                    "File rename with user-controlled paths".to_string(),
                )
                .with_language(language),
                TaintSink::new(
                    "python_os_remove".to_string(),
                    "os.remove".to_string(),
                    SecurityIssueType::PathTraversal,
                    "File deletion with user-controlled path".to_string(),
                )
                .with_language(language),
            ],
            SourceLanguage::JavaScript | SourceLanguage::TypeScript => vec![
                TaintSink::new(
                    "js_fs_copy".to_string(),
                    "fs.copyFile".to_string(),
                    SecurityIssueType::PathTraversal,
                    "File copy with user-controlled paths".to_string(),
                )
                .with_language(language),
                TaintSink::new(
                    "js_fs_rename".to_string(),
                    "fs.rename".to_string(),
                    SecurityIssueType::PathTraversal,
                    "File rename with user paths".to_string(),
                )
                .with_language(language),
                TaintSink::new(
                    "js_fs_unlink".to_string(),
                    "fs.unlink".to_string(),
                    SecurityIssueType::PathTraversal,
                    "File deletion with user-controlled path".to_string(),
                )
                .with_language(language),
            ],
            _ => Vec::new(),
        }
    }

    /// Create taint sinks for path-related operations
    fn create_path_sinks(&self, language: SourceLanguage) -> Vec<TaintSink> {
        match language {
            SourceLanguage::JavaScript | SourceLanguage::TypeScript => vec![
                TaintSink::new(
                    "js_path_join".to_string(),
                    "path.join".to_string(),
                    SecurityIssueType::PathTraversal,
                    "Path joining with user-controlled components".to_string(),
                )
                .with_language(language),
                TaintSink::new(
                    "js_path_resolve".to_string(),
                    "path.resolve".to_string(),
                    SecurityIssueType::PathTraversal,
                    "Path resolution with user input".to_string(),
                )
                .with_language(language),
            ],
            _ => Vec::new(),
        }
    }
}

impl TaintSinkDetector for FileSinkDetector {
    fn detect_sinks(&self, file: &ParsedFile) -> Result<Vec<TaintSink>, AnalysisError> {
        let mut sinks = Vec::new();

        // Get different types of file operation sinks
        sinks.extend(self.create_file_write_sinks(file.language));
        sinks.extend(self.create_file_manipulation_sinks(file.language));
        sinks.extend(self.create_path_sinks(file.language));

        // TODO: Implement actual AST-based detection
        // This would involve:
        // 1. Walking the AST to find function calls
        // 2. Matching against file operation patterns
        // 3. Creating TaintSink objects with proper location info
        // 4. Identifying which parameters are vulnerable

        Ok(sinks)
    }

    fn get_patterns_for_language(&self, language: SourceLanguage) -> Vec<String> {
        self.patterns
            .get(&language)
            .cloned()
            .unwrap_or_else(Vec::new)
    }
}

impl Default for FileSinkDetector {
    fn default() -> Self {
        Self::new()
    }
}
