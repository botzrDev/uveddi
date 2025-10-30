//! # Code Duplication Detector - New Context Implementation
//!
//! This detector uses the new AnalysisContext interface for duplication detection.

use crate::ast::SourceLanguage;
use crate::database::models::ArchitecturalIssue;
use crate::engine::analysis::context::AnalysisContext;
use crate::engine::analysis::pipeline::{Detector, PipelineError};
use std::collections::HashMap;

/// Simple code duplication detection using context analysis
pub struct ContextCodeDuplicationDetector {
    /// Minimum similarity threshold (0.0 to 1.0)
    similarity_threshold: f64,
    /// Minimum block size to consider
    min_block_size: usize,
}

impl ContextCodeDuplicationDetector {
    /// Create a new context-based code duplication detector
    pub fn new(similarity_threshold: f64, min_block_size: usize) -> Self {
        Self {
            similarity_threshold,
            min_block_size,
        }
    }

    /// Create with default configuration
    pub fn default() -> Self {
        Self::new(0.85, 6) // 85% similarity, minimum 6 lines
    }

    /// Extract code blocks from the source
    fn extract_code_blocks(&self, context: &AnalysisContext) -> Vec<CodeBlock> {
        let lines: Vec<&str> = context.source.lines().collect();
        let mut blocks = Vec::new();

        // Extract blocks using a sliding window approach
        for start in 0..lines.len() {
            if start + self.min_block_size > lines.len() {
                break;
            }

            for end in (start + self.min_block_size)..=std::cmp::min(start + 20, lines.len()) {
                let block_lines = &lines[start..end];

                // Skip blocks that are mostly comments or empty lines
                if self.is_significant_block(block_lines) {
                    let content = block_lines.join("\n");
                    let normalized_content = self.normalize_content(&content);
                    let hash = self.calculate_hash(&normalized_content);

                    blocks.push(CodeBlock {
                        start_line: start + 1,
                        end_line: end,
                        content,
                        normalized_content,
                        hash,
                    });
                }
            }
        }

        blocks
    }

    /// Check if a block contains significant code (not just comments/whitespace)
    fn is_significant_block(&self, lines: &[&str]) -> bool {
        let mut significant_lines = 0;
        for line in lines {
            let trimmed = line.trim();
            if !trimmed.is_empty()
                && !trimmed.starts_with("//")
                && !trimmed.starts_with("/*")
                && !trimmed.starts_with("*")
                && !trimmed.starts_with("#")
            {
                significant_lines += 1;
            }
        }
        significant_lines >= (self.min_block_size / 2)
    }

    /// Normalize content for comparison
    fn normalize_content(&self, content: &str) -> String {
        content
            .lines()
            .map(|line| {
                // Remove leading/trailing whitespace and normalize internal whitespace
                line.trim().split_whitespace().collect::<Vec<_>>().join(" ")
            })
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Calculate a simple hash for quick comparison
    fn calculate_hash(&self, content: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        hasher.finish()
    }

    /// Calculate similarity between two blocks
    fn calculate_similarity(&self, block1: &CodeBlock, block2: &CodeBlock) -> f64 {
        if block1.hash == block2.hash {
            return 1.0;
        }

        // Use Levenshtein distance for similarity
        let distance = levenshtein_distance(&block1.normalized_content, &block2.normalized_content);
        let max_len = std::cmp::max(
            block1.normalized_content.len(),
            block2.normalized_content.len(),
        );

        if max_len == 0 {
            return 1.0;
        }

        1.0 - (distance as f64 / max_len as f64)
    }

    /// Find duplicate blocks within the file
    fn find_duplicates(&self, blocks: Vec<CodeBlock>) -> Vec<DuplicationPair> {
        let mut duplicates = Vec::new();

        for i in 0..blocks.len() {
            for j in (i + 1)..blocks.len() {
                let similarity = self.calculate_similarity(&blocks[i], &blocks[j]);

                if similarity >= self.similarity_threshold {
                    duplicates.push(DuplicationPair {
                        block1: blocks[i].clone(),
                        block2: blocks[j].clone(),
                        similarity,
                    });
                }
            }
        }

        duplicates
    }
}

impl Detector for ContextCodeDuplicationDetector {
    fn detect(&self, context: &AnalysisContext) -> Result<Vec<ArchitecturalIssue>, PipelineError> {
        let mut issues = Vec::new();

        // Extract code blocks
        let blocks = self.extract_code_blocks(context);

        if blocks.len() < 2 {
            return Ok(issues);
        }

        // Find duplicates
        let duplicates = self.find_duplicates(blocks);

        // Create issues for each duplicate pair
        for duplicate in duplicates {
            let issue = ArchitecturalIssue {
                issue_id: None,
                analysis_run_id: 0, // TODO: get from context
                anti_pattern_type_id: 1, // TODO: get from pattern type registry
                file_path: context.file_info.path.to_string_lossy().to_string(),
                start_line: Some(duplicate.block1.start_line as i32),
                end_line: Some(duplicate.block1.end_line as i32),
                line_number: Some(duplicate.block1.start_line as i32),
                column_number: Some(0),
                message: format!(
                    "Duplicate code block found (similarity: {:.1}%). Lines {}-{} are {:.1}% similar to lines {}-{}",
                    duplicate.similarity * 100.0,
                    duplicate.block1.start_line,
                    duplicate.block1.end_line,
                    duplicate.similarity * 100.0,
                    duplicate.block2.start_line,
                    duplicate.block2.end_line
                ),
                metadata: serde_json::json!({
                    "issue_type": "Code Duplication",
                    "rule_id": "code_duplication",
                    "similarity": duplicate.similarity,
                    "block1": {
                        "start_line": duplicate.block1.start_line,
                        "end_line": duplicate.block1.end_line
                    },
                    "block2": {
                        "start_line": duplicate.block2.start_line,
                        "end_line": duplicate.block2.end_line
                    }
                }).to_string(),
                detector_name: "CodeDuplicationDetector".to_string(),
                created_at: chrono::Utc::now(),
                severity: if duplicate.similarity > 0.95 {
                    "High".to_string()
                } else if duplicate.similarity > 0.85 {
                    "Medium".to_string()
                } else {
                    "Low".to_string()
                },
                description: format!(
                    "Duplicate code block found (similarity: {:.1}%). Lines {}-{} are {:.1}% similar to lines {}-{}",
                    duplicate.similarity * 100.0,
                    duplicate.block1.start_line,
                    duplicate.block1.end_line,
                    duplicate.similarity * 100.0,
                    duplicate.block2.start_line,
                    duplicate.block2.end_line
                ),
                code_snippet: Some(duplicate.block1.content.clone()),
                ai_explanation: Some(format!(
                    "Consider extracting common code into a shared function or method. Block 1: lines {}-{}, Block 2: lines {}-{}",
                    duplicate.block1.start_line,
                    duplicate.block1.end_line,
                    duplicate.block2.start_line,
                    duplicate.block2.end_line
                )),
            };

            issues.push(issue);
        }

        Ok(issues)
    }

    fn name(&self) -> &str {
        "ContextCodeDuplicationDetector"
    }

    fn supports_language(&self, _language: &SourceLanguage) -> bool {
        true // Supports all languages using text-based analysis
    }
}

/// Represents a code block for duplication analysis
#[derive(Debug, Clone)]
struct CodeBlock {
    start_line: usize,
    end_line: usize,
    content: String,
    normalized_content: String,
    hash: u64,
}

/// Represents a pair of duplicate code blocks
#[derive(Debug)]
struct DuplicationPair {
    block1: CodeBlock,
    block2: CodeBlock,
    similarity: f64,
}

/// Calculate Levenshtein distance between two strings
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.len();
    let len2 = s2.len();
    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    // Initialize first row and column
    for i in 0..=len1 {
        matrix[i][0] = i;
    }
    for j in 0..=len2 {
        matrix[0][j] = j;
    }

    // Fill the matrix
    let chars1: Vec<char> = s1.chars().collect();
    let chars2: Vec<char> = s2.chars().collect();

    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = if chars1[i - 1] == chars2[j - 1] { 0 } else { 1 };
            matrix[i][j] = std::cmp::min(
                std::cmp::min(matrix[i - 1][j] + 1, matrix[i][j - 1] + 1),
                matrix[i - 1][j - 1] + cost,
            );
        }
    }

    matrix[len1][len2]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::analysis::context::{FileInfo, ProjectContext};
    use std::path::PathBuf;
    use std::time::SystemTime;

    #[test]
    fn test_context_code_duplication_detector() {
        let detector = ContextCodeDuplicationDetector::default();

        let duplicate_code = r#"
fn process_data(data: Vec<i32>) -> i32 {
    let mut sum = 0;
    for item in data {
        sum += item * 2;
    }
    sum
}

fn calculate_sum(values: Vec<i32>) -> i32 {
    let mut sum = 0;
    for item in values {
        sum += item * 2;
    }
    sum
}
"#;

        let file_info = FileInfo {
            path: PathBuf::from("test.rs"),
            language: SourceLanguage::Rust,
            lines_of_code: 15,
            size_bytes: duplicate_code.len(),
            modified_at: SystemTime::now(),
        };

        let project_context = ProjectContext {
            project_root: PathBuf::from("/test"),
            project_files: vec![],
            dependencies: vec![],
            global_symbols: vec![],
        };

        let context = AnalysisContext::new(
            file_info,
            None,
            duplicate_code.to_string(),
            vec![],
            vec![],
            project_context,
        );

        let issues = detector.detect(&context).unwrap();
        assert!(!issues.is_empty());
        assert!(issues[0].description.contains("Duplicate code block"));
    }
}
