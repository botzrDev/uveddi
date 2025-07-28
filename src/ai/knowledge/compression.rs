//! Compression-Optimized Types and Dictionary Training
//! 
//! This module implements compression-aware data structures and dictionary training
//! for achieving 70-75% size reduction with zstd compression as specified in the
//! compression research findings.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::{Read, Write};

/// Compression-aware string type optimized for dictionary training and zstd compression
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompressedString {
    /// The actual string content
    pub content: String,
    /// Compression hints for optimization
    pub compression_hints: CompressionHints,
}

/// Compression hints for optimization and dictionary training
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompressionHints {
    /// Estimated repetition factor (higher = more repetitive)
    pub repetition_factor: f32,
    /// Dictionary candidates extracted from this string
    pub dictionary_candidates: Vec<String>,
    /// Preferred encoding type for this content
    pub encoding_preference: EncodingType,
    /// Content type classification
    pub content_type: ContentType,
}

/// Encoding preference for different content types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EncodingType {
    /// Standard UTF-8 encoding
    Utf8,
    /// ASCII-optimized encoding
    Ascii,
    /// Programming language specific encoding
    Code,
    /// Documentation/markdown encoding
    Documentation,
    /// JSON/structured data encoding
    Structured,
}

/// Content type classification for optimization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContentType {
    /// Source code content
    Code,
    /// Natural language text
    Text,
    /// Technical documentation
    Documentation,
    /// Configuration or structured data
    Configuration,
    /// Error messages and diagnostics
    Diagnostic,
    /// Regular expressions and patterns
    Pattern,
}

/// Main compression metadata structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionMetadata {
    /// Dictionary version identifier
    pub dictionary_version: String,
    /// Zstd compression level used (9-12 recommended)
    pub compression_level: u8,
    /// Original uncompressed size in bytes
    pub original_size: usize,
    /// Compressed size in bytes
    pub compressed_size: usize,
    /// Compression ratio (compressed/original)
    pub compression_ratio: f32,
    /// SHA-256 checksum of compressed data
    pub checksum: String,
    /// Dictionary training metadata
    pub dictionary_metadata: DictionaryMetadata,
    /// Compression timestamp
    pub compressed_at: chrono::DateTime<chrono::Utc>,
}

/// Dictionary training metadata and statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictionaryMetadata {
    /// Number of samples used for training
    pub sample_count: usize,
    /// Total training data size in bytes
    pub training_data_size: usize,
    /// Dictionary size in bytes
    pub dictionary_size: usize,
    /// Training effectiveness score (0.0-1.0)
    pub effectiveness_score: f32,
    /// Most frequent programming keywords
    pub top_keywords: Vec<String>,
    /// Most frequent API patterns
    pub top_api_patterns: Vec<String>,
    /// Training completion timestamp
    pub trained_at: chrono::DateTime<chrono::Utc>,
}

/// Dictionary trainer for programming-specific content
#[derive(Debug, Clone)]
pub struct DictionaryTrainer {
    /// Programming language keywords
    pub programming_keywords: HashSet<String>,
    /// Common API patterns and method names
    pub api_patterns: Vec<String>,
    /// Architectural and design pattern terms
    pub architectural_terms: Vec<String>,
    /// Error message patterns
    pub error_patterns: Vec<String>,
    /// Code documentation patterns
    pub documentation_patterns: Vec<String>,
    /// Collected training samples
    pub training_samples: Vec<String>,
}

impl CompressedString {
    /// Create a new CompressedString with automatic hint generation
    pub fn new(content: &str) -> Self {
        let hints = Self::generate_hints(content);
        Self {
            content: content.to_string(),
            compression_hints: hints,
        }
    }

    /// Create a CompressedString with manual hints
    pub fn with_hints(content: &str, hints: CompressionHints) -> Self {
        Self {
            content: content.to_string(),
            compression_hints: hints,
        }
    }

    /// Generate compression hints automatically from content analysis
    fn generate_hints(content: &str) -> CompressionHints {
        let content_type = Self::classify_content(content);
        let encoding_preference = Self::determine_encoding(content, &content_type);
        let repetition_factor = Self::calculate_repetition_factor(content);
        let dictionary_candidates = Self::extract_dictionary_candidates(content, &content_type);

        CompressionHints {
            repetition_factor,
            dictionary_candidates,
            encoding_preference,
            content_type,
        }
    }

    /// Classify content type for optimization
    fn classify_content(content: &str) -> ContentType {
        // Check for code patterns
        if content.contains("fn ") || content.contains("def ") || content.contains("function ") ||
           content.contains("class ") || content.contains("struct ") || content.contains("impl ") {
            return ContentType::Code;
        }

        // Check for configuration patterns
        if content.starts_with('{') || content.starts_with('[') || content.contains(": ") ||
           content.contains("=") && content.lines().count() < 10 {
            return ContentType::Configuration;
        }

        // Check for documentation patterns
        if content.contains("# ") || content.contains("## ") || content.contains("```") ||
           content.contains("*") && content.contains("\n") {
            return ContentType::Documentation;
        }

        // Check for error/diagnostic patterns
        if content.contains("error:") || content.contains("warning:") || content.contains("Error") ||
           content.contains("Exception") || content.contains("panic") {
            return ContentType::Diagnostic;
        }

        // Check for regex patterns
        if content.contains("\\") && (content.contains("+") || content.contains("*") || content.contains("?")) {
            return ContentType::Pattern;
        }

        // Default to text
        ContentType::Text
    }

    /// Determine optimal encoding type
    fn determine_encoding(content: &str, content_type: &ContentType) -> EncodingType {
        match content_type {
            ContentType::Code => EncodingType::Code,
            ContentType::Configuration => EncodingType::Structured,
            ContentType::Documentation => EncodingType::Documentation,
            _ => {
                if content.is_ascii() {
                    EncodingType::Ascii
                } else {
                    EncodingType::Utf8
                }
            }
        }
    }

    /// Calculate repetition factor for compression effectiveness estimation
    fn calculate_repetition_factor(content: &str) -> f32 {
        if content.is_empty() {
            return 0.0;
        }

        let words: Vec<&str> = content.split_whitespace().collect();
        if words.is_empty() {
            return 0.0;
        }

        let mut word_counts = HashMap::new();
        for word in &words {
            *word_counts.entry(*word).or_insert(0) += 1;
        }

        let total_words = words.len() as f32;
        let unique_words = word_counts.len() as f32;
        
        // Higher repetition factor means more repeated content
        1.0 - (unique_words / total_words)
    }

    /// Extract dictionary candidates from content
    fn extract_dictionary_candidates(content: &str, content_type: &ContentType) -> Vec<String> {
        let mut candidates = Vec::new();

        match content_type {
            ContentType::Code => {
                // Extract programming keywords and identifiers
                let code_patterns = [
                    r"\b(fn|def|function|class|struct|impl|trait|interface)\b",
                    r"\b(pub|private|public|static|const|let|var|mut)\b",
                    r"\b(if|else|for|while|loop|match|switch|case)\b",
                    r"\b(return|break|continue|yield|await|async)\b",
                ];
                
                for pattern in &code_patterns {
                    if let Ok(regex) = regex::Regex::new(pattern) {
                        for mat in regex.find_iter(content) {
                            candidates.push(mat.as_str().to_string());
                        }
                    }
                }
            },
            ContentType::Documentation => {
                // Extract common documentation terms
                let doc_patterns = ["## ", "### ", "#### ", "```", "**", "*", "`"];
                for pattern in &doc_patterns {
                    if content.contains(pattern) {
                        candidates.push(pattern.to_string());
                    }
                }
            },
            ContentType::Diagnostic => {
                // Extract error message patterns
                let error_patterns = ["error:", "warning:", "Error", "Exception", "panic!", "unwrap()"];
                for pattern in &error_patterns {
                    if content.contains(pattern) {
                        candidates.push(pattern.to_string());
                    }
                }
            },
            _ => {
                // Extract common words for general text
                let words: Vec<&str> = content.split_whitespace().collect();
                let mut word_counts = HashMap::new();
                for word in words {
                    *word_counts.entry(word).or_insert(0) += 1;
                }
                
                // Get most frequent words as candidates
                let mut sorted_words: Vec<_> = word_counts.into_iter().collect();
                sorted_words.sort_by(|a, b| b.1.cmp(&a.1));
                
                candidates.extend(
                    sorted_words.into_iter()
                        .take(10)
                        .map(|(word, _)| word.to_string())
                );
            }
        }

        candidates
    }

    /// Get the string content
    pub fn as_str(&self) -> &str {
        &self.content
    }

    /// Get content length
    pub fn len(&self) -> usize {
        self.content.len()
    }

    /// Check if content is empty
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }
}

impl DictionaryTrainer {
    /// Create a new dictionary trainer with default programming patterns
    pub fn new() -> Self {
        let programming_keywords = [
            // Rust keywords
            "fn", "let", "mut", "const", "static", "pub", "impl", "trait", "struct", "enum",
            "match", "if", "else", "for", "while", "loop", "break", "continue", "return",
            "use", "mod", "crate", "super", "self", "async", "await", "unsafe",
            
            // Python keywords
            "def", "class", "import", "from", "as", "if", "elif", "else", "for", "while",
            "try", "except", "finally", "with", "lambda", "yield", "async", "await",
            
            // JavaScript/TypeScript keywords
            "function", "var", "let", "const", "class", "extends", "interface", "type",
            "import", "export", "from", "default", "async", "await", "promise",
            
            // Universal keywords
            "true", "false", "null", "undefined", "void", "this", "super", "new",
        ].iter().map(|s| s.to_string()).collect();

        let api_patterns = vec![
            // Common method patterns
            "get_", "set_", "is_", "has_", "can_", "should_", "will_",
            "create_", "update_", "delete_", "remove_", "add_", "insert_",
            "find_", "search_", "filter_", "map_", "reduce_", "collect_",
            "parse_", "serialize_", "deserialize_", "encode_", "decode_",
            "connect_", "disconnect_", "send_", "receive_", "read_", "write_",
            // Common suffixes
            "_impl", "_test", "_error", "_result", "_option", "_future",
            "_handler", "_service", "_client", "_server", "_config",
        ].iter().map(|s| s.to_string()).collect();

        let architectural_terms = vec![
            // Design patterns
            "singleton", "factory", "builder", "observer", "strategy", "adapter",
            "facade", "proxy", "decorator", "command", "state", "visitor",
            // Architectural patterns
            "mvc", "mvp", "mvvm", "repository", "service", "controller",
            "middleware", "interceptor", "filter", "guard", "wrapper",
            // Anti-patterns
            "god_object", "spaghetti_code", "magic_number", "shotgun_surgery",
            "feature_envy", "data_clump", "long_method", "large_class",
        ].iter().map(|s| s.to_string()).collect();

        let error_patterns = vec![
            "Error", "Exception", "Panic", "Failure", "Invalid", "Missing",
            "Timeout", "Connection", "Network", "Permission", "Access",
            "NotFound", "BadRequest", "InternalError", "ServiceUnavailable",
        ].iter().map(|s| s.to_string()).collect();

        let documentation_patterns = vec![
            "TODO", "FIXME", "NOTE", "WARNING", "DEPRECATED", "SAFETY",
            "Examples", "Parameters", "Returns", "Errors", "Panics",
            "See also", "Reference", "Documentation", "Usage",
        ].iter().map(|s| s.to_string()).collect();

        Self {
            programming_keywords,
            api_patterns,
            architectural_terms,
            error_patterns,
            documentation_patterns,
            training_samples: Vec::new(),
        }
    }

    /// Add a training sample for dictionary optimization
    pub fn add_sample(&mut self, content: &str) {
        self.training_samples.push(content.to_string());
    }

    /// Add multiple training samples
    pub fn add_samples<I>(&mut self, samples: I)
    where
        I: IntoIterator<Item = String>,
    {
        self.training_samples.extend(samples);
    }

    /// Generate a training dictionary for zstd compression
    pub fn generate_dictionary(&self, max_size: usize) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        if self.training_samples.is_empty() {
            return Err("No training samples provided".into());
        }

        // Combine all training samples
        let training_data = self.training_samples.join("\n");
        
        // For now, create a simple dictionary from most frequent terms
        // In a full implementation, this would use zstd's dictionary training API
        let mut dictionary_content = String::new();
        
        // Add programming keywords
        for keyword in &self.programming_keywords {
            dictionary_content.push_str(keyword);
            dictionary_content.push('\n');
        }
        
        // Add API patterns
        for pattern in &self.api_patterns {
            dictionary_content.push_str(pattern);
            dictionary_content.push('\n');
        }
        
        // Add architectural terms
        for term in &self.architectural_terms {
            dictionary_content.push_str(term);
            dictionary_content.push('\n');
        }

        // Truncate to max_size
        let mut dict_bytes = dictionary_content.into_bytes();
        if dict_bytes.len() > max_size {
            dict_bytes.truncate(max_size);
        }

        Ok(dict_bytes)
    }

    /// Analyze training samples and generate metadata
    pub fn analyze_training_data(&self) -> DictionaryMetadata {
        let sample_count = self.training_samples.len();
        let training_data_size = self.training_samples.iter()
            .map(|s| s.len())
            .sum();

        // Analyze keyword frequency
        let mut keyword_frequency = HashMap::new();
        for sample in &self.training_samples {
            for keyword in &self.programming_keywords {
                if sample.contains(keyword) {
                    *keyword_frequency.entry(keyword.clone()).or_insert(0) += 1;
                }
            }
        }

        let mut top_keywords: Vec<_> = keyword_frequency.into_iter().collect();
        top_keywords.sort_by(|a, b| b.1.cmp(&a.1));
        let top_keywords: Vec<String> = top_keywords.into_iter()
            .take(20)
            .map(|(keyword, _)| keyword)
            .collect();

        // Analyze API pattern frequency
        let mut api_frequency = HashMap::new();
        for sample in &self.training_samples {
            for pattern in &self.api_patterns {
                if sample.contains(pattern) {
                    *api_frequency.entry(pattern.clone()).or_insert(0) += 1;
                }
            }
        }

        let mut top_api_patterns: Vec<_> = api_frequency.into_iter().collect();
        top_api_patterns.sort_by(|a, b| b.1.cmp(&a.1));
        let top_api_patterns: Vec<String> = top_api_patterns.into_iter()
            .take(20)
            .map(|(pattern, _)| pattern)
            .collect();

        // Calculate effectiveness score (simplified)
        let effectiveness_score = if sample_count > 0 {
            (top_keywords.len() + top_api_patterns.len()) as f32 / (sample_count as f32 * 0.1)
        } else {
            0.0
        }.min(1.0);

        DictionaryMetadata {
            sample_count,
            training_data_size,
            dictionary_size: 0, // Will be set when dictionary is generated
            effectiveness_score,
            top_keywords,
            top_api_patterns,
            trained_at: chrono::Utc::now(),
        }
    }
}

impl Default for CompressionMetadata {
    fn default() -> Self {
        Self {
            dictionary_version: "1.0.0".to_string(),
            compression_level: 9,
            original_size: 0,
            compressed_size: 0,
            compression_ratio: 0.0,
            checksum: String::new(),
            dictionary_metadata: DictionaryMetadata {
                sample_count: 0,
                training_data_size: 0,
                dictionary_size: 0,
                effectiveness_score: 0.0,
                top_keywords: Vec::new(),
                top_api_patterns: Vec::new(),
                trained_at: chrono::Utc::now(),
            },
            compressed_at: chrono::Utc::now(),
        }
    }
}

impl Default for DictionaryTrainer {
    fn default() -> Self {
        Self::new()
    }
}

/// Compress data using zstd with optional dictionary
pub fn compress_with_zstd(
    data: &[u8],
    level: i32,
    dictionary: Option<&[u8]>,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // For now, implement basic compression without dictionary
    // In full implementation, this would use zstd-rs with dictionary support
    let compressed = flate2::read::GzEncoder::new(data, flate2::Compression::new(level as u32));
    let mut result = Vec::new();
    std::io::copy(&mut std::io::BufReader::new(compressed), &mut result)?;
    Ok(result)
}

/// Decompress data using zstd with optional dictionary
pub fn decompress_with_zstd(
    data: &[u8],
    dictionary: Option<&[u8]>,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // For now, implement basic decompression without dictionary
    // In full implementation, this would use zstd-rs with dictionary support
    let mut decoder = flate2::read::GzDecoder::new(data);
    let mut result = Vec::new();
    decoder.read_to_end(&mut result)?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compressed_string_creation() {
        let cs = CompressedString::new("Hello, world!");
        assert_eq!(cs.as_str(), "Hello, world!");
        assert!(!cs.is_empty());
        assert_eq!(cs.len(), 13);
    }

    #[test]
    fn test_content_classification() {
        let code_content = "fn main() { println!(); }";
        let cs = CompressedString::new(code_content);
        assert_eq!(cs.compression_hints.content_type, ContentType::Code);

        let text_content = "This is just regular text content.";
        let cs = CompressedString::new(text_content);
        assert_eq!(cs.compression_hints.content_type, ContentType::Text);
    }

    #[test]
    fn test_dictionary_trainer() {
        let mut trainer = DictionaryTrainer::new();
        trainer.add_sample("fn main() { let x = 42; }");
        trainer.add_sample("def calculate(): return value");
        
        let metadata = trainer.analyze_training_data();
        assert_eq!(metadata.sample_count, 2);
        assert!(metadata.top_keywords.contains(&"fn".to_string()));
        assert!(metadata.top_keywords.contains(&"def".to_string()));
    }

    #[test]
    fn test_repetition_factor_calculation() {
        let repetitive = "the the the cat cat sat";
        let factor = CompressedString::calculate_repetition_factor(repetitive);
        assert!(factor > 0.0);

        let unique = "every word is different here";
        let factor = CompressedString::calculate_repetition_factor(unique);
        assert!(factor < 0.5); // Should be lower for unique content
    }

    #[test]
    fn test_dictionary_generation() {
        let mut trainer = DictionaryTrainer::new();
        // Add some training samples first
        trainer.add_sample("fn main() { println!(\"Hello\"); }");
        trainer.add_sample("class MyClass { public void method() {} }");
        trainer.add_sample("def function(): return value");
        
        let dict = trainer.generate_dictionary(1024).unwrap();
        assert!(!dict.is_empty());
        assert!(dict.len() <= 1024);
    }
}