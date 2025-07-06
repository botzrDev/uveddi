//! Excessive Comments Plugin
//!
//! This plugin detects when files have an excessive amount of comments
//! relative to their code content, which can indicate poor code quality
//! or over-documentation.

use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Cursor;

use arrow_array::{RecordBatch, StringArray, UInt32Array};
use arrow_ipc::reader::StreamReader;
use regex::Regex;

// Generate the bindings from the WIT file
wit_bindgen::generate!({
    path: "../../../wit/plugin.wit",
    world: "code-analyzer",
});

/// The main plugin implementation
struct ExcessiveCommentsPlugin {
    /// Storage for AST data buffers indexed by handle ID
    ast_buffers: RefCell<HashMap<u32, Vec<u8>>>,
    /// Next available handle ID
    next_id: RefCell<u32>,
    /// Configuration for the plugin
    config: RefCell<Option<PluginConfig>>,
    /// Regex patterns for comment detection
    comment_patterns: RefCell<HashMap<String, Vec<Regex>>>,
}

impl ExcessiveCommentsPlugin {
    fn new() -> Self {
        let mut comment_patterns = HashMap::new();
        
        // Rust comment patterns
        comment_patterns.insert("rust".to_string(), vec![
            Regex::new(r"//.*").unwrap(),
            Regex::new(r"/\*.*?\*/").unwrap(),
            Regex::new(r"///.*").unwrap(),
            Regex::new(r"//!.*").unwrap(),
        ]);
        
        // JavaScript/TypeScript comment patterns
        comment_patterns.insert("javascript".to_string(), vec![
            Regex::new(r"//.*").unwrap(),
            Regex::new(r"/\*.*?\*/").unwrap(),
        ]);
        
        // Python comment patterns
        comment_patterns.insert("python".to_string(), vec![
            Regex::new(r"#.*").unwrap(),
            Regex::new(r"\"\"\".*?\"\"\"").unwrap(),
            Regex::new(r"'''.*?'''").unwrap(),
        ]);
        
        Self {
            ast_buffers: RefCell::new(HashMap::new()),
            next_id: RefCell::new(0),
            config: RefCell::new(None),
            comment_patterns: RefCell::new(comment_patterns),
        }
    }
    
    /// Count comment lines in source code
    fn count_comments(&self, source_code: &str, language: &str) -> (usize, usize) {
        let lines: Vec<&str> = source_code.lines().collect();
        let total_lines = lines.len();
        
        let comment_patterns = self.comment_patterns.borrow();
        let patterns = comment_patterns.get(language).unwrap_or(&vec![]);
        
        let mut comment_lines = 0;
        
        for line in &lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            
            for pattern in patterns {
                if pattern.is_match(trimmed) {
                    comment_lines += 1;
                    break;
                }
            }
        }
        
        (comment_lines, total_lines)
    }
    
    /// Analyze AST nodes for excessive comments
    fn analyze_ast_buffer(&self, buffer: &[u8]) -> Result<Vec<ArchitecturalIssue>, String> {
        let cursor = Cursor::new(buffer);
        let mut reader = StreamReader::try_new(cursor, None)
            .map_err(|e| format!("Failed to create Arrow reader: {}", e))?;
        
        let mut issues = Vec::new();
        let config = self.config.borrow();
        let threshold = config.as_ref()
            .map(|c| c.severity_threshold)
            .unwrap_or(0.3); // Default 30% threshold
        
        while let Some(batch_result) = reader.next() {
            let batch = batch_result.map_err(|e| format!("Failed to read batch: {}", e))?;
            issues.extend(self.analyze_batch(&batch, threshold)?);
        }
        
        Ok(issues)
    }
    
    /// Analyze a single record batch
    fn analyze_batch(&self, batch: &RecordBatch, threshold: f32) -> Result<Vec<ArchitecturalIssue>, String> {
        let mut issues = Vec::new();
        
        // Extract relevant columns
        let node_types = batch
            .column_by_name("node_type")
            .and_then(|col| col.as_any().downcast_ref::<StringArray>())
            .ok_or("Missing or invalid node_type column")?;
        
        let texts = batch
            .column_by_name("text")
            .and_then(|col| col.as_any().downcast_ref::<StringArray>())
            .ok_or("Missing or invalid text column")?;
        
        let file_paths = batch
            .column_by_name("file_path")
            .and_then(|col| col.as_any().downcast_ref::<StringArray>())
            .ok_or("Missing or invalid file_path column")?;
        
        let languages = batch
            .column_by_name("language")
            .and_then(|col| col.as_any().downcast_ref::<StringArray>())
            .ok_or("Missing or invalid language column")?;
        
        let start_lines = batch
            .column_by_name("start_line")
            .and_then(|col| col.as_any().downcast_ref::<UInt32Array>())
            .ok_or("Missing or invalid start_line column")?;
        
        // Group by file and analyze each file
        let mut files: HashMap<String, Vec<(String, String, u32)>> = HashMap::new();
        
        for i in 0..batch.num_rows() {
            let file_path = file_paths.value(i).to_string();
            let node_type = node_types.value(i).to_string();
            let text = texts.value(i).to_string();
            let start_line = start_lines.value(i);
            
            files.entry(file_path)
                .or_insert_with(Vec::new)
                .push((node_type, text, start_line));
        }
        
        // Analyze each file
        for (file_path, nodes) in files {
            if let Some(language) = languages.value(0).split('/').next() {
                // Reconstruct source code from AST nodes
                let mut source_lines: HashMap<u32, String> = HashMap::new();
                
                for (node_type, text, line_num) in &nodes {
                    if node_type == "comment" || text.trim().starts_with("//") || text.trim().starts_with("/*") {
                        source_lines.insert(*line_num, text.clone());
                    }
                }
                
                if !source_lines.is_empty() {
                    let total_nodes = nodes.len();
                    let comment_nodes = source_lines.len();
                    let comment_ratio = comment_nodes as f32 / total_nodes as f32;
                    
                    if comment_ratio > threshold {
                        let severity = if comment_ratio > 0.5 {
                            Severity::High
                        } else if comment_ratio > 0.4 {
                            Severity::Medium
                        } else {
                            Severity::Low
                        };
                        
                        let first_comment_line = source_lines.keys().min().copied().unwrap_or(1);
                        
                        issues.push(ArchitecturalIssue {
                            file_path: file_path.clone(),
                            start_line: first_comment_line,
                            end_line: first_comment_line,
                            start_column: 1,
                            end_column: 80,
                            anti_pattern_type: "excessive-comments".to_string(),
                            message: format!(
                                "File has excessive comments: {:.1}% of nodes are comments",
                                comment_ratio * 100.0
                            ),
                            description: format!(
                                "This file contains {} comment nodes out of {} total nodes ({:.1}%), \
                                which exceeds the threshold of {:.1}%. Consider reducing documentation \
                                or improving code clarity.",
                                comment_nodes, total_nodes, comment_ratio * 100.0, threshold * 100.0
                            ),
                            confidence: 0.8,
                            severity,
                            suggested_fix: Some(
                                "Review comments for necessity. Remove redundant comments and \
                                improve code self-documentation through better naming.".to_string()
                            ),
                            metadata: vec![
                                ("comment_ratio".to_string(), format!("{:.3}", comment_ratio)),
                                ("comment_count".to_string(), comment_nodes.to_string()),
                                ("total_nodes".to_string(), total_nodes.to_string()),
                                ("language".to_string(), language.to_string()),
                            ],
                        });
                    }
                }
            }
        }
        
        Ok(issues)
    }
}

impl Guest for ExcessiveCommentsPlugin {
    type PluginInterface = ExcessiveCommentsPluginInterface;
    
    fn plugin_interface() -> Self::PluginInterface {
        ExcessiveCommentsPluginInterface::new()
    }
}

/// Implementation of the plugin interface
struct ExcessiveCommentsPluginInterface {
    plugin: ExcessiveCommentsPlugin,
}

impl ExcessiveCommentsPluginInterface {
    fn new() -> Self {
        logging::log("info", "Excessive Comments Plugin initialized");
        Self {
            plugin: ExcessiveCommentsPlugin::new(),
        }
    }
}

impl GuestPluginInterface for ExcessiveCommentsPluginInterface {
    fn get_info(&mut self) -> PluginInfo {
        PluginInfo {
            name: "excessive-comments".to_string(),
            version: "1.0.0".to_string(),
            author: "Uveddi Team".to_string(),
            description: "Detects files with excessive comment density".to_string(),
            supported_languages: vec![
                "rust".to_string(),
                "javascript".to_string(),
                "typescript".to_string(),
                "python".to_string(),
            ],
            anti_pattern_types: vec!["excessive-comments".to_string()],
            requires_graph_analysis: false,
        }
    }
    
    fn initialize(&mut self, config: PluginConfig) -> Result<(), PluginError> {
        logging::log("info", &format!("Initializing with threshold: {}", config.severity_threshold));
        
        *self.plugin.config.borrow_mut() = Some(config);
        
        Ok(())
    }
    
    fn load_ast(&mut self, ast_buffer: Vec<u8>) -> Result<Resource<AstHandle>, PluginError> {
        let mut next_id = self.plugin.next_id.borrow_mut();
        let mut ast_buffers = self.plugin.ast_buffers.borrow_mut();
        
        let id = *next_id;
        *next_id += 1;
        
        let size = ast_buffer.len() as u64;
        ast_buffers.insert(id, ast_buffer);
        
        logging::log("debug", &format!("Stored AST buffer with handle ID: {} (size: {} bytes)", id, size));
        
        // Create the resource handle
        let handle = AstHandle::new(AstHandleRep { id });
        Ok(Resource::new_own(handle))
    }
    
    fn detect_issues(&mut self, ast: Resource<AstHandle>) -> Result<Vec<ArchitecturalIssue>, PluginError> {
        let handle_rep = ast.rep();
        let id = handle_rep.id;
        
        logging::log("debug", &format!("Analyzing AST with handle ID: {}", id));
        
        let ast_buffers = self.plugin.ast_buffers.borrow();
        let buffer = ast_buffers
            .get(&id)
            .ok_or_else(|| PluginError::InvalidInput(format!("Invalid ast-handle: {}", id)))?;
        
        match self.plugin.analyze_ast_buffer(buffer) {
            Ok(issues) => {
                logging::log("info", &format!("Found {} excessive comment issues", issues.len()));
                Ok(issues)
            }
            Err(e) => {
                logging::log("error", &format!("Analysis failed: {}", e));
                Err(PluginError::ProcessingError(e))
            }
        }
    }
    
    fn detect_graph_issues(&mut self, _dependencies: Vec<DependencyInfo>) -> Result<Vec<ArchitecturalIssue>, PluginError> {
        // This plugin doesn't perform graph analysis
        Ok(Vec::new())
    }
    
    fn free_ast(&mut self, handle: Resource<AstHandle>) -> Result<(), PluginError> {
        let handle_rep = handle.rep();
        let id = handle_rep.id;
        
        let mut ast_buffers = self.plugin.ast_buffers.borrow_mut();
        if ast_buffers.remove(&id).is_some() {
            logging::log("debug", &format!("Freed AST buffer with handle ID: {}", id));
            Ok(())
        } else {
            let error_msg = format!("Attempted to free invalid ast-handle: {}", id);
            logging::log("error", &error_msg);
            Err(PluginError::InvalidInput(error_msg))
        }
    }
    
    fn cleanup(&mut self) -> Result<(), PluginError> {
        logging::log("info", "Cleaning up Excessive Comments Plugin");
        
        // Clear all stored AST buffers
        self.plugin.ast_buffers.borrow_mut().clear();
        *self.plugin.next_id.borrow_mut() = 0;
        
        Ok(())
    }
}

// Export the component
export!(ExcessiveCommentsPlugin);

// Internal handle representation
struct AstHandleRep {
    id: u32,
}

impl AstHandle {
    fn new(rep: AstHandleRep) -> Self {
        Self { rep }
    }
    
    fn rep(&self) -> &AstHandleRep {
        &self.rep
    }
}

impl GuestAstHandle for AstHandle {
    fn new(rep: AstHandleRep) -> Self {
        Self::new(rep)
    }
    
    fn size(&self) -> u64 {
        // This would need access to the actual buffer size
        // For now, return a placeholder
        0
    }
    
    fn language(&self) -> String {
        // This would need access to the actual language info
        // For now, return a placeholder
        "unknown".to_string()
    }
}