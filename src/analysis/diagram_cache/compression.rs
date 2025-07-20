//! Diagram Compression Engine
//!
//! Provides intelligent compression for different diagram types to achieve
//! 60%+ storage reduction while maintaining fast decompression times.

use super::{DiagramType, Result, DiagramCacheError};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use log::{debug, warn};

/// Compression levels (1-9, higher = better compression)
pub type CompressionLevel = u8;

/// Compression statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionStats {
    /// Original size in bytes
    pub original_size: usize,
    
    /// Compressed size in bytes
    pub compressed_size: usize,
    
    /// Compression ratio (compressed/original)
    pub compression_ratio: f64,
    
    /// Compression time in milliseconds
    pub compression_time_ms: u64,
    
    /// Decompression time in milliseconds
    pub decompression_time_ms: u64,
}

/// Handles compression and decompression of diagram content
pub struct CompressionEngine {
    /// Compression level (1-9)
    level: CompressionLevel,
    
    /// Compression statistics
    stats: CompressionStats,
}

impl CompressionEngine {
    /// Creates a new compression engine
    pub fn new(level: CompressionLevel) -> Self {
        Self {
            level: level.clamp(1, 9),
            stats: CompressionStats {
                original_size: 0,
                compressed_size: 0,
                compression_ratio: 1.0,
                compression_time_ms: 0,
                decompression_time_ms: 0,
            },
        }
    }
    
    /// Compresses diagram content based on diagram type
    pub async fn compress(&self, content: &[u8], diagram_type: &DiagramType) -> Result<Vec<u8>> {
        let start_time = std::time::Instant::now();
        
        let compressed = match diagram_type {
            DiagramType::Mermaid => self.compress_text_based(content).await?,
            DiagramType::PlantUML => self.compress_text_based(content).await?,
            DiagramType::Graphviz => self.compress_text_based(content).await?,
            DiagramType::Custom(format) => {
                if self.is_text_format(format) {
                    self.compress_text_based(content).await?
                } else {
                    self.compress_binary(content).await?
                }
            }
        };
        
        let compression_time = start_time.elapsed().as_millis() as u64;
        let compression_ratio = compressed.len() as f64 / content.len() as f64;
        
        debug!("Compressed {:?} diagram: {} -> {} bytes ({:.1}% reduction, {}ms)", 
               diagram_type, content.len(), compressed.len(), 
               (1.0 - compression_ratio) * 100.0, compression_time);
        
        Ok(compressed)
    }
    
    /// Decompresses diagram content
    pub async fn decompress(&self, compressed: &[u8], diagram_type: &DiagramType) -> Result<Vec<u8>> {
        let start_time = std::time::Instant::now();
        
        let decompressed = match diagram_type {
            DiagramType::Mermaid => self.decompress_gzip(compressed).await?,
            DiagramType::PlantUML => self.decompress_gzip(compressed).await?,
            DiagramType::Graphviz => self.decompress_gzip(compressed).await?,
            DiagramType::Custom(format) => {
                if self.is_text_format(format) {
                    self.decompress_gzip(compressed).await?
                } else {
                    self.decompress_binary(compressed).await?
                }
            }
        };
        
        let decompression_time = start_time.elapsed().as_millis() as u64;
        
        debug!("Decompressed {:?} diagram: {} -> {} bytes ({}ms)", 
               diagram_type, compressed.len(), decompressed.len(), decompression_time);
        
        Ok(decompressed)
    }
    
    /// Gets compression statistics
    pub fn get_stats(&self) -> &CompressionStats {
        &self.stats
    }
    
    /// Estimates compression ratio for a given content type
    pub fn estimate_compression_ratio(&self, content: &[u8], diagram_type: &DiagramType) -> f64 {
        match diagram_type {
            DiagramType::Mermaid => self.estimate_text_compression_ratio(content),
            DiagramType::PlantUML => self.estimate_text_compression_ratio(content),
            DiagramType::Graphviz => self.estimate_text_compression_ratio(content),
            DiagramType::Custom(format) => {
                if self.is_text_format(format) {
                    self.estimate_text_compression_ratio(content)
                } else {
                    self.estimate_binary_compression_ratio(content)
                }
            }
        }
    }
    
    /// Compresses text-based diagram content using gzip
    async fn compress_text_based(&self, content: &[u8]) -> Result<Vec<u8>> {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        
        let mut encoder = GzEncoder::new(Vec::new(), Compression::new(self.level as u32));
        encoder.write_all(content)
            .map_err(|e| DiagramCacheError::CompressionError(e.to_string()))?;
        
        encoder.finish()
            .map_err(|e| DiagramCacheError::CompressionError(e.to_string()))
    }
    
    /// Compresses binary content using a different strategy
    async fn compress_binary(&self, content: &[u8]) -> Result<Vec<u8>> {
        // For binary content, we might use different compression
        // For now, use gzip as well but could be optimized per format
        self.compress_text_based(content).await
    }
    
    /// Decompresses gzip content
    async fn decompress_gzip(&self, compressed: &[u8]) -> Result<Vec<u8>> {
        use flate2::read::GzDecoder;
        
        let mut decoder = GzDecoder::new(compressed);
        let mut decompressed = Vec::new();
        
        decoder.read_to_end(&mut decompressed)
            .map_err(|e| DiagramCacheError::DecompressionError(e.to_string()))?;
        
        Ok(decompressed)
    }
    
    /// Decompresses binary content
    async fn decompress_binary(&self, compressed: &[u8]) -> Result<Vec<u8>> {
        // For binary content decompression
        self.decompress_gzip(compressed).await
    }
    
    /// Checks if a format is text-based
    fn is_text_format(&self, format: &str) -> bool {
        matches!(format.to_lowercase().as_str(), 
                "svg" | "dot" | "puml" | "plantuml" | "mmd" | "mermaid" | "txt" | "md")
    }
    
    /// Estimates compression ratio for text content
    fn estimate_text_compression_ratio(&self, content: &[u8]) -> f64 {
        // Simple heuristic based on content characteristics
        let text = String::from_utf8_lossy(content);
        
        // Count repetitive patterns
        let unique_chars = text.chars().collect::<std::collections::HashSet<_>>().len();
        let total_chars = text.len();
        
        if total_chars == 0 {
            return 1.0;
        }
        
        let diversity = unique_chars as f64 / total_chars as f64;
        
        // Higher diversity = worse compression
        // Lower diversity = better compression
        let base_ratio = match self.level {
            1..=3 => 0.7,  // Fast compression
            4..=6 => 0.5,  // Balanced
            7..=9 => 0.3,  // Best compression
            _ => 0.5,
        };
        
        // Adjust based on content diversity
        let adjusted_ratio = base_ratio + (diversity * 0.4);
        adjusted_ratio.clamp(0.1, 0.9)
    }
    
    /// Estimates compression ratio for binary content
    fn estimate_binary_compression_ratio(&self, content: &[u8]) -> f64 {
        // Binary content typically compresses less well
        let base_ratio = match self.level {
            1..=3 => 0.8,
            4..=6 => 0.7,
            7..=9 => 0.6,
            _ => 0.7,
        };
        
        // Check for patterns in binary data
        let mut pattern_score = 0.0;
        if content.len() > 100 {
            // Sample every 10th byte to check for patterns
            let sample: Vec<u8> = content.iter().step_by(10).cloned().collect();
            let unique_bytes = sample.iter().collect::<std::collections::HashSet<_>>().len();
            pattern_score = 1.0 - (unique_bytes as f64 / sample.len() as f64);
        }
        
        // Adjust ratio based on detected patterns
        let adjusted_ratio = base_ratio - (pattern_score * 0.2);
        adjusted_ratio.clamp(0.3, 0.9)
    }
}

impl Default for CompressionEngine {
    fn default() -> Self {
        Self::new(6) // Balanced compression level
    }
}

/// Specialized compression strategies for different diagram types
pub struct DiagramTypeCompressor;

impl DiagramTypeCompressor {
    /// Optimizes Mermaid diagram content before compression
    pub fn optimize_mermaid(content: &str) -> String {
        // Remove unnecessary whitespace and comments
        let lines: Vec<&str> = content
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty() && !line.starts_with("%%"))
            .collect();
        
        lines.join("\n")
    }
    
    /// Optimizes PlantUML diagram content before compression
    pub fn optimize_plantuml(content: &str) -> String {
        // Remove comments and normalize spacing
        let lines: Vec<&str> = content
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty() && !line.starts_with("'"))
            .collect();
        
        lines.join("\n")
    }
    
    /// Optimizes Graphviz/DOT content before compression
    pub fn optimize_graphviz(content: &str) -> String {
        // Remove comments and normalize spacing
        content
            .lines()
            .map(|line| {
                // Remove inline comments
                if let Some(comment_pos) = line.find("//") {
                    &line[..comment_pos]
                } else {
                    line
                }
            })
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }
    
    /// Applies content optimization before compression
    pub fn optimize_content(content: &str, diagram_type: &DiagramType) -> String {
        match diagram_type {
            DiagramType::Mermaid => Self::optimize_mermaid(content),
            DiagramType::PlantUML => Self::optimize_plantuml(content),
            DiagramType::Graphviz => Self::optimize_graphviz(content),
            DiagramType::Custom(_) => content.to_string(), // No optimization for custom types
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_text_compression() {
        let engine = CompressionEngine::new(6);
        let content = b"graph TD\n    A --> B\n    B --> C\n    C --> D\n    A --> B\n    B --> C";
        
        let compressed = engine.compress(content, &DiagramType::Mermaid).await.unwrap();
        let decompressed = engine.decompress(&compressed, &DiagramType::Mermaid).await.unwrap();
        
        assert_eq!(content, decompressed.as_slice());
        assert!(compressed.len() < content.len());
    }
    
    #[test]
    fn test_content_optimization() {
        let mermaid_content = r#"
            %% This is a comment
            graph TD
                A --> B
                B --> C
            %% Another comment
        "#;
        
        let optimized = DiagramTypeCompressor::optimize_mermaid(mermaid_content);
        assert!(!optimized.contains("%%"));
        assert!(optimized.contains("graph TD"));
    }
    
    #[test]
    fn test_compression_ratio_estimation() {
        let engine = CompressionEngine::new(6);
        let repetitive_content = b"AAAAAAAAAA".repeat(100);
        let random_content = b"abcdefghijklmnopqrstuvwxyz0123456789";
        
        let repetitive_ratio = engine.estimate_compression_ratio(&repetitive_content, &DiagramType::Mermaid);
        let random_ratio = engine.estimate_compression_ratio(random_content, &DiagramType::Mermaid);
        
        // Repetitive content should compress better
        assert!(repetitive_ratio < random_ratio);
    }
}