//! Helper functions for code duplication detection

use super::types::{ClonePair, CodeBlock};
use crate::analysis::detectors::base::Issue;

/// Helper functions for clone pair processing
pub struct ClonePairHelpers;

impl ClonePairHelpers {
    /// Creates a unique key for clone pair deduplication
    pub fn create_pair_key(pair: &ClonePair) -> String {
        let (file1, line1, file2, line2) = if pair.block1.file_path < pair.block2.file_path ||
            (pair.block1.file_path == pair.block2.file_path && pair.block1.start_line < pair.block2.start_line) {
            (&pair.block1.file_path, pair.block1.start_line, &pair.block2.file_path, pair.block2.start_line)
        } else {
            (&pair.block2.file_path, pair.block2.start_line, &pair.block1.file_path, pair.block1.start_line)
        };

        format!("{}:{}:{}:{}", file1, line1, file2, line2)
    }

    /// Converts clone pairs to issues for reporting
    pub fn convert_pairs_to_issues(pairs: &[ClonePair]) -> Vec<Issue> {
        pairs
            .iter()
            .map(|pair| {
                let issue_id = format!("code_duplication_{}", pair.clone_type);
                let title = format!("{} - Code duplication detected", pair.clone_type);
                let description = format!(
                    "Duplicate code found between {}:{}-{} and {}:{}-{} (similarity: {:.1}%)",
                    pair.block1.file_path,
                    pair.block1.start_line,
                    pair.block1.end_line,
                    pair.block2.file_path,
                    pair.block2.start_line,
                    pair.block2.end_line,
                    pair.similarity * 100.0
                );

                Issue::new(
                    issue_id,
                    title,
                    description,
                    pair.clone_type.severity(),
                    pair.block1.file_path.clone(),
                    pair.block1.start_line,
                    pair.block1.end_line,
                )
                .with_metadata("clone_type".to_string(), serde_json::Value::String(pair.clone_type.to_string()))
                .with_metadata("similarity".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(pair.similarity).unwrap()))
                .with_metadata("duplicate_file".to_string(), serde_json::Value::String(pair.block2.file_path.clone()))
                .with_metadata("duplicate_lines".to_string(), serde_json::Value::String(format!("{}-{}", pair.block2.start_line, pair.block2.end_line)))
                .with_suggestion(format!("Consider extracting the common logic into a shared function or module"))
            })
            .collect()
    }

    /// Filters code blocks based on minimum requirements
    pub fn filter_blocks(blocks: Vec<CodeBlock>, min_tokens: usize, min_lines: u32) -> Vec<CodeBlock> {
        blocks
            .into_iter()
            .filter(|block| {
                block.token_count() >= min_tokens && block.line_count() >= min_lines
            })
            .collect()
    }

    /// Deduplicates and filters clone pairs
    pub fn deduplicate_pairs(pairs: Vec<ClonePair>, max_pairs: Option<usize>) -> Vec<ClonePair> {
        let mut unique_pairs = Vec::new();
        let mut seen_pairs = std::collections::HashSet::new();

        for pair in pairs {
            let key = Self::create_pair_key(&pair);
            if !seen_pairs.contains(&key) {
                unique_pairs.push(pair);
                seen_pairs.insert(key);
            }
        }

        // Sort by similarity (highest first)
        unique_pairs.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap_or(std::cmp::Ordering::Equal));

        // Apply maximum results limit
        if let Some(max_pairs) = max_pairs {
            unique_pairs.truncate(max_pairs);
        }

        unique_pairs
    }
}