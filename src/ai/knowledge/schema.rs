//! Core Schema Types for AI Knowledge Library
//! 
//! This module defines the hierarchical schema structures optimized for AI consumption
//! and compression. The design follows the UV-330 epic requirements for a compressed
//! knowledge base achieving <10MB size and <100ms retrieval performance.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::ai::knowledge::compression::{CompressedString, CompressionMetadata};

/// Primary source languages supported by the knowledge library
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SourceLanguage {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Java,
    Go,
    CSharp,
    Cpp,
    /// Universal patterns that apply across languages
    Universal,
}

/// Anti-pattern categories for hierarchical organization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AntiPatternCategory {
    /// Object-oriented design issues
    ObjectOriented,
    /// Architectural and structural problems
    Architectural,
    /// Performance and optimization issues
    Performance,
    /// Security vulnerabilities and risks
    Security,
    /// Memory management problems
    Memory,
    /// Concurrency and threading issues
    Concurrency,
    /// Error handling and recovery
    ErrorHandling,
    /// Code organization and maintainability
    Maintainability,
    /// Resource management and cleanup
    Resources,
    /// API design and interface issues
    ApiDesign,
}

/// Impact severity levels for prioritization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ImpactLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Detection method types for pattern identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DetectionMethod {
    /// Static analysis patterns
    StaticAnalysis {
        pattern: String,
        confidence: f32,
    },
    /// AST-based detection
    AstPattern {
        query: String,
        node_types: Vec<String>,
    },
    /// Metric-based thresholds
    MetricThreshold {
        metric_name: String,
        threshold: f64,
        operator: ComparisonOperator,
    },
    /// Dependency graph analysis
    DependencyAnalysis {
        graph_pattern: String,
        relationship_type: String,
    },
    /// Regular expression matching
    RegexPattern {
        pattern: String,
        context: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonOperator {
    GreaterThan,
    LessThan,
    Equal,
    GreaterThanOrEqual,
    LessThanOrEqual,
}

/// Solution pattern types for remediation guidance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolutionPattern {
    /// Unique identifier for the solution
    pub id: String,
    /// Human-readable title
    pub title: String,
    /// Detailed implementation guidance
    pub implementation: CompressedString,
    /// Code examples demonstrating the solution
    pub examples: Vec<CodeExample>,
    /// Effort estimation for implementation
    pub effort_level: EffortLevel,
    /// Prerequisites and dependencies
    pub prerequisites: Vec<String>,
    /// Expected impact of applying the solution
    pub expected_impact: ImpactLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffortLevel {
    Trivial,     // < 1 hour
    Low,         // 1-4 hours
    Medium,      // 1-2 days
    High,        // 3-5 days
    Significant, // 1-2 weeks
}

/// Code examples with language-specific variations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeExample {
    /// Programming language for the example
    pub language: SourceLanguage,
    /// Problem demonstration code
    pub problem_code: CompressedString,
    /// Solution code
    pub solution_code: CompressedString,
    /// Explanatory notes
    pub explanation: CompressedString,
    /// File path context (optional)
    pub file_context: Option<String>,
}

/// Collection of code examples organized by language
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeExamples {
    /// Primary examples for the most common languages
    pub primary: Vec<CodeExample>,
    /// Additional language-specific variations
    pub variations: HashMap<SourceLanguage, Vec<CodeExample>>,
}

/// Language-specific information and variations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageSpecificInfo {
    /// Language-specific symptoms and indicators
    pub symptoms: Vec<CompressedString>,
    /// Common manifestations in this language
    pub manifestations: Vec<CompressedString>,
    /// Language-specific detection methods
    pub detection_methods: Vec<DetectionMethod>,
    /// Framework or library specific considerations
    pub framework_notes: HashMap<String, CompressedString>,
    /// Standard library patterns to avoid/use
    pub stdlib_guidance: Option<CompressedString>,
}

/// Language-specific knowledge collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageKnowledge {
    /// Language metadata
    pub language: SourceLanguage,
    /// Language-specific anti-patterns
    pub patterns: HashMap<String, PatternKnowledge>,
    /// Common idioms and best practices
    pub idioms: Vec<LanguageIdiom>,
    /// Framework-specific knowledge
    pub frameworks: HashMap<String, FrameworkKnowledge>,
    /// Standard library guidance
    pub stdlib_patterns: Vec<StdlibPattern>,
}

/// Language idioms and best practices
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageIdiom {
    pub name: String,
    pub description: CompressedString,
    pub example: CodeExample,
    pub when_to_use: CompressedString,
    pub alternatives: Vec<String>,
}

/// Framework-specific knowledge and patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkKnowledge {
    pub framework_name: String,
    pub version_range: String,
    pub specific_patterns: Vec<String>,
    pub best_practices: Vec<CompressedString>,
    pub common_pitfalls: Vec<CompressedString>,
}

/// Standard library usage patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StdlibPattern {
    pub pattern_name: String,
    pub description: CompressedString,
    pub recommended_usage: CompressedString,
    pub alternatives: Vec<String>,
}

/// Core pattern knowledge structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternKnowledge {
    /// Unique identifier for the pattern
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Detailed definition and description
    pub definition: CompressedString,
    /// Observable symptoms and indicators
    pub symptoms: Vec<CompressedString>,
    /// Impact severity and assessment
    pub impact: ImpactLevel,
    /// Category for hierarchical organization
    pub category: AntiPatternCategory,
    /// Detection methods and techniques
    pub detection_methods: Vec<DetectionMethod>,
    /// Available solution patterns
    pub solutions: Vec<SolutionPattern>,
    /// Code examples demonstrating the pattern
    pub examples: CodeExamples,
    /// Language-specific variations and notes
    pub language_variations: HashMap<SourceLanguage, LanguageSpecificInfo>,
    /// Related patterns (parent-child relationships)
    pub related_patterns: Vec<String>,
    /// Tags for flexible categorization
    pub tags: Vec<String>,
    /// Frequency of occurrence (for prioritization)
    pub frequency_score: f32,
    /// Confidence in detection methods
    pub detection_confidence: f32,
}

/// Detection context mapping for efficient lookups
pub type DetectionContextMap = HashMap<String, Vec<String>>;

/// Solution pattern mapping for quick access
pub type SolutionPatternMap = HashMap<String, Vec<SolutionPattern>>;

/// Main knowledge library structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeLibrary {
    /// Library metadata and versioning
    pub metadata: LibraryMetadata,
    /// Universal anti-patterns that apply across languages
    pub universal_patterns: HashMap<String, PatternKnowledge>,
    /// Language-specific knowledge and patterns
    pub language_specific: HashMap<SourceLanguage, LanguageKnowledge>,
    /// Detection context mappings for efficient lookups
    pub detection_context: DetectionContextMap,
    /// Solution pattern mappings
    pub solution_patterns: SolutionPatternMap,
    /// Compression metadata and statistics
    pub compression_metadata: CompressionMetadata,
    /// Index metadata for PHF lookups
    pub index_metadata: IndexMetadata,
}

/// Library metadata and versioning information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryMetadata {
    /// Schema version for migration support
    pub schema_version: String,
    /// Library content version
    pub content_version: String,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last update timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// Total number of patterns
    pub pattern_count: usize,
    /// Supported languages
    pub supported_languages: Vec<SourceLanguage>,
    /// Compression statistics summary
    pub compression_summary: CompressionSummary,
    /// Contributors and sources
    pub contributors: Vec<String>,
    /// License information
    pub license: String,
}

/// Compression statistics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionSummary {
    /// Original size in bytes
    pub original_size: usize,
    /// Compressed size in bytes
    pub compressed_size: usize,
    /// Compression ratio (compressed/original)
    pub compression_ratio: f32,
    /// Compression algorithm used
    pub algorithm: String,
    /// Dictionary training effectiveness
    pub dictionary_effectiveness: f32,
}

/// Index metadata for PHF and other indexing structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexMetadata {
    /// PHF generation timestamp
    pub generated_at: chrono::DateTime<chrono::Utc>,
    /// Number of indexed entries
    pub entry_count: usize,
    /// Index size in bytes
    pub index_size: usize,
    /// Hash function parameters
    pub hash_params: HashMap<String, String>,
    /// Index validation checksum
    pub checksum: String,
}

impl KnowledgeLibrary {
    /// Create a new empty knowledge library
    pub fn new() -> Self {
        Self {
            metadata: LibraryMetadata {
                schema_version: "1.0.0".to_string(),
                content_version: "0.1.0".to_string(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                pattern_count: 0,
                supported_languages: vec![
                    SourceLanguage::Universal,
                    SourceLanguage::Rust,
                    SourceLanguage::Python,
                    SourceLanguage::JavaScript,
                    SourceLanguage::TypeScript,
                ],
                compression_summary: CompressionSummary {
                    original_size: 0,
                    compressed_size: 0,
                    compression_ratio: 0.0,
                    algorithm: "zstd".to_string(),
                    dictionary_effectiveness: 0.0,
                },
                contributors: vec!["Uveddi Core Team".to_string()],
                license: "MIT".to_string(),
            },
            universal_patterns: HashMap::new(),
            language_specific: HashMap::new(),
            detection_context: HashMap::new(),
            solution_patterns: HashMap::new(),
            compression_metadata: CompressionMetadata::default(),
            index_metadata: IndexMetadata {
                generated_at: chrono::Utc::now(),
                entry_count: 0,
                index_size: 0,
                hash_params: HashMap::new(),
                checksum: String::new(),
            },
        }
    }

    /// Add a universal pattern to the library
    pub fn add_universal_pattern(&mut self, pattern: PatternKnowledge) {
        self.universal_patterns.insert(pattern.id.clone(), pattern);
        self.metadata.pattern_count = self.universal_patterns.len()
            + self.language_specific.values()
                .map(|lang| lang.patterns.len())
                .sum::<usize>();
        self.metadata.updated_at = chrono::Utc::now();
    }

    /// Add language-specific knowledge
    pub fn add_language_knowledge(&mut self, language: SourceLanguage, knowledge: LanguageKnowledge) {
        self.language_specific.insert(language, knowledge);
        self.metadata.pattern_count = self.universal_patterns.len()
            + self.language_specific.values()
                .map(|lang| lang.patterns.len())
                .sum::<usize>();
        self.metadata.updated_at = chrono::Utc::now();
    }

    /// Get all patterns for a specific language (includes universal patterns)
    pub fn get_language_patterns(&self, language: &SourceLanguage) -> Vec<&PatternKnowledge> {
        let mut patterns = Vec::new();
        
        // Add universal patterns
        patterns.extend(self.universal_patterns.values());
        
        // Add language-specific patterns
        if let Some(lang_knowledge) = self.language_specific.get(language) {
            patterns.extend(lang_knowledge.patterns.values());
        }
        
        patterns
    }

    /// Search patterns by category
    pub fn get_patterns_by_category(&self, category: &AntiPatternCategory) -> Vec<&PatternKnowledge> {
        let mut patterns = Vec::new();
        
        // Search universal patterns
        patterns.extend(
            self.universal_patterns.values()
                .filter(|p| &p.category == category)
        );
        
        // Search language-specific patterns
        for lang_knowledge in self.language_specific.values() {
            patterns.extend(
                lang_knowledge.patterns.values()
                    .filter(|p| &p.category == category)
            );
        }
        
        patterns
    }

    /// Validate the library structure and integrity
    pub fn validate(&self) -> Result<(), String> {
        // Check for duplicate pattern IDs
        let mut all_ids = std::collections::HashSet::new();
        
        for pattern in self.universal_patterns.values() {
            if !all_ids.insert(&pattern.id) {
                return Err(format!("Duplicate pattern ID: {}", pattern.id));
            }
        }
        
        for lang_knowledge in self.language_specific.values() {
            for pattern in lang_knowledge.patterns.values() {
                if !all_ids.insert(&pattern.id) {
                    return Err(format!("Duplicate pattern ID: {}", pattern.id));
                }
            }
        }
        
        // Validate pattern count
        let actual_count = self.universal_patterns.len()
            + self.language_specific.values()
                .map(|lang| lang.patterns.len())
                .sum::<usize>();
                
        if actual_count != self.metadata.pattern_count {
            return Err(format!(
                "Pattern count mismatch: metadata says {}, actual is {}",
                self.metadata.pattern_count, actual_count
            ));
        }
        
        Ok(())
    }
}

impl Default for KnowledgeLibrary {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knowledge_library_creation() {
        let library = KnowledgeLibrary::new();
        assert_eq!(library.metadata.pattern_count, 0);
        assert!(library.universal_patterns.is_empty());
        assert!(library.language_specific.is_empty());
    }

    #[test]
    fn test_add_universal_pattern() {
        let mut library = KnowledgeLibrary::new();
        let pattern = PatternKnowledge {
            id: "god_object".to_string(),
            name: "God Object".to_string(),
            definition: CompressedString::new("A class that does too much"),
            symptoms: vec![CompressedString::new("Large class size")],
            impact: ImpactLevel::High,
            category: AntiPatternCategory::ObjectOriented,
            detection_methods: vec![],
            solutions: vec![],
            examples: CodeExamples {
                primary: vec![],
                variations: HashMap::new(),
            },
            language_variations: HashMap::new(),
            related_patterns: vec![],
            tags: vec!["oop".to_string(), "design".to_string()],
            frequency_score: 0.8,
            detection_confidence: 0.9,
        };

        library.add_universal_pattern(pattern);
        assert_eq!(library.metadata.pattern_count, 1);
        assert!(library.universal_patterns.contains_key("god_object"));
    }

    #[test]
    fn test_library_validation() {
        let library = KnowledgeLibrary::new();
        assert!(library.validate().is_ok());
    }

    #[test]
    fn test_get_patterns_by_category() {
        let mut library = KnowledgeLibrary::new();
        let pattern = PatternKnowledge {
            id: "god_object".to_string(),
            name: "God Object".to_string(),
            definition: CompressedString::new("A class that does too much"),
            symptoms: vec![],
            impact: ImpactLevel::High,
            category: AntiPatternCategory::ObjectOriented,
            detection_methods: vec![],
            solutions: vec![],
            examples: CodeExamples {
                primary: vec![],
                variations: HashMap::new(),
            },
            language_variations: HashMap::new(),
            related_patterns: vec![],
            tags: vec![],
            frequency_score: 0.8,
            detection_confidence: 0.9,
        };

        library.add_universal_pattern(pattern);
        let oo_patterns = library.get_patterns_by_category(&AntiPatternCategory::ObjectOriented);
        assert_eq!(oo_patterns.len(), 1);
        assert_eq!(oo_patterns[0].id, "god_object");
    }
}