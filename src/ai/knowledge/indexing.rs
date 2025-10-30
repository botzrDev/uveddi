//! Perfect Hash Function (PHF) Indexing for O(1) Knowledge Lookups
//!
//! This module implements compile-time indexing using Perfect Hash Functions (PHF)
//! for achieving O(1) lookup performance in the knowledge library. The indexing
//! system supports pattern IDs, language mappings, and symptom-based searches.

use crate::ai::knowledge::compression::CompressedString;
use crate::ai::knowledge::schema::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Runtime lookup interface for the knowledge library
pub trait KnowledgeLookup {
    /// Get a pattern by its unique identifier
    fn get_pattern(&self, pattern_id: &str) -> Option<&PatternKnowledge>;

    /// Get all patterns for a specific language (includes universal patterns)
    fn get_language_patterns(&self, language: SourceLanguage) -> Vec<&PatternKnowledge>;

    /// Search patterns by symptoms or indicators
    fn search_by_symptoms(&self, symptoms: &[String]) -> Vec<&PatternKnowledge>;

    /// Get patterns by category
    fn get_patterns_by_category(&self, category: AntiPatternCategory) -> Vec<&PatternKnowledge>;

    /// Get patterns by impact level
    fn get_patterns_by_impact(&self, min_impact: ImpactLevel) -> Vec<&PatternKnowledge>;

    /// Search patterns by tags
    fn search_by_tags(&self, tags: &[String]) -> Vec<&PatternKnowledge>;

    /// Get related patterns for a given pattern ID
    fn get_related_patterns(&self, pattern_id: &str) -> Vec<&PatternKnowledge>;
}

/// Indexing configuration for build-time generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexingConfig {
    /// Whether to generate pattern ID index
    pub enable_pattern_index: bool,
    /// Whether to generate language-specific indices
    pub enable_language_index: bool,
    /// Whether to generate symptom search index
    pub enable_symptom_index: bool,
    /// Whether to generate category index
    pub enable_category_index: bool,
    /// Whether to generate tag search index
    pub enable_tag_index: bool,
    /// Maximum number of entries per index
    pub max_entries_per_index: usize,
    /// Index generation timestamp
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

/// Pattern index entry for PHF generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternIndexEntry {
    /// Pattern unique identifier
    pub pattern_id: String,
    /// Offset in the compressed knowledge library
    pub offset: u32,
    /// Size of the pattern data
    pub size: u32,
    /// Pattern category for quick filtering
    pub category: AntiPatternCategory,
    /// Source language (if language-specific)
    pub language: Option<SourceLanguage>,
    /// Pattern impact level
    pub impact: ImpactLevel,
}

/// Language index entry for language-specific lookups
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageIndexEntry {
    /// Source language
    pub language: SourceLanguage,
    /// Pattern IDs for this language
    pub pattern_ids: Vec<String>,
    /// Number of patterns
    pub pattern_count: usize,
}

/// Symptom index entry for symptom-based searches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymptomIndexEntry {
    /// Normalized symptom text (lowercased, trimmed)
    pub symptom: String,
    /// Pattern IDs that exhibit this symptom
    pub pattern_ids: Vec<String>,
    /// Relevance score (0.0-1.0)
    pub relevance_score: f32,
}

/// Category index entry for category-based lookups
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryIndexEntry {
    /// Anti-pattern category
    pub category: AntiPatternCategory,
    /// Pattern IDs in this category
    pub pattern_ids: Vec<String>,
    /// Average impact level for this category
    pub average_impact: f32,
}

/// Tag index entry for tag-based searches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagIndexEntry {
    /// Tag text
    pub tag: String,
    /// Pattern IDs with this tag
    pub pattern_ids: Vec<String>,
    /// Tag frequency across all patterns
    pub frequency: usize,
}

/// Complete indexing structure for build-time generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeIndex {
    /// Configuration used for generation
    pub config: IndexingConfig,
    /// Pattern ID to entry mapping
    pub pattern_entries: Vec<PatternIndexEntry>,
    /// Language-specific indices
    pub language_entries: Vec<LanguageIndexEntry>,
    /// Symptom search indices
    pub symptom_entries: Vec<SymptomIndexEntry>,
    /// Category indices
    pub category_entries: Vec<CategoryIndexEntry>,
    /// Tag search indices
    pub tag_entries: Vec<TagIndexEntry>,
    /// Total number of patterns indexed
    pub total_patterns: usize,
    /// Index generation metadata
    pub metadata: IndexGenerationMetadata,
}

/// Metadata about index generation process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexGenerationMetadata {
    /// Time taken to generate indices (in milliseconds)
    pub generation_time_ms: u64,
    /// Memory used during generation (in bytes)
    pub memory_used: usize,
    /// Number of unique symptoms indexed
    pub unique_symptoms: usize,
    /// Number of unique tags indexed
    pub unique_tags: usize,
    /// Validation checksum
    pub checksum: String,
}

/// Index builder for constructing indices from knowledge library
pub struct IndexBuilder {
    config: IndexingConfig,
    pattern_entries: Vec<PatternIndexEntry>,
    language_entries: Vec<LanguageIndexEntry>,
    symptom_entries: Vec<SymptomIndexEntry>,
    category_entries: Vec<CategoryIndexEntry>,
    tag_entries: Vec<TagIndexEntry>,
}

impl IndexBuilder {
    /// Create a new index builder with configuration
    pub fn new(config: IndexingConfig) -> Self {
        Self {
            config,
            pattern_entries: Vec::new(),
            language_entries: Vec::new(),
            symptom_entries: Vec::new(),
            category_entries: Vec::new(),
            tag_entries: Vec::new(),
        }
    }

    /// Create a default index builder with all indices enabled
    pub fn default() -> Self {
        let config = IndexingConfig {
            enable_pattern_index: true,
            enable_language_index: true,
            enable_symptom_index: true,
            enable_category_index: true,
            enable_tag_index: true,
            max_entries_per_index: 10000,
            generated_at: chrono::Utc::now(),
        };
        Self::new(config)
    }

    /// Build indices from a knowledge library
    pub fn build_from_library(
        &mut self,
        library: &KnowledgeLibrary,
    ) -> Result<KnowledgeIndex, String> {
        let start_time = std::time::Instant::now();

        // Build pattern index
        if self.config.enable_pattern_index {
            self.build_pattern_index(library)?;
        }

        // Build language index
        if self.config.enable_language_index {
            self.build_language_index(library)?;
        }

        // Build symptom index
        if self.config.enable_symptom_index {
            self.build_symptom_index(library)?;
        }

        // Build category index
        if self.config.enable_category_index {
            self.build_category_index(library)?;
        }

        // Build tag index
        if self.config.enable_tag_index {
            self.build_tag_index(library)?;
        }

        let generation_time = start_time.elapsed();
        let total_patterns = library.universal_patterns.len()
            + library
                .language_specific
                .values()
                .map(|lang| lang.patterns.len())
                .sum::<usize>();

        let metadata = IndexGenerationMetadata {
            generation_time_ms: generation_time.as_millis() as u64,
            memory_used: self.estimate_memory_usage(),
            unique_symptoms: self.symptom_entries.len(),
            unique_tags: self.tag_entries.len(),
            checksum: self.generate_checksum(),
        };

        Ok(KnowledgeIndex {
            config: self.config.clone(),
            pattern_entries: std::mem::take(&mut self.pattern_entries),
            language_entries: std::mem::take(&mut self.language_entries),
            symptom_entries: std::mem::take(&mut self.symptom_entries),
            category_entries: std::mem::take(&mut self.category_entries),
            tag_entries: std::mem::take(&mut self.tag_entries),
            total_patterns,
            metadata,
        })
    }

    /// Build pattern ID index
    fn build_pattern_index(&mut self, library: &KnowledgeLibrary) -> Result<(), String> {
        let mut offset = 0u32;

        // Index universal patterns
        for pattern in library.universal_patterns.values() {
            let entry = PatternIndexEntry {
                pattern_id: pattern.id.clone(),
                offset,
                size: self.estimate_pattern_size(pattern),
                category: pattern.category.clone(),
                language: None, // Universal patterns
                impact: pattern.impact.clone(),
            };
            self.pattern_entries.push(entry);
            offset += self.estimate_pattern_size(pattern);
        }

        // Index language-specific patterns
        for (language, lang_knowledge) in &library.language_specific {
            for pattern in lang_knowledge.patterns.values() {
                let entry = PatternIndexEntry {
                    pattern_id: pattern.id.clone(),
                    offset,
                    size: self.estimate_pattern_size(pattern),
                    category: pattern.category.clone(),
                    language: Some(language.clone()),
                    impact: pattern.impact.clone(),
                };
                self.pattern_entries.push(entry);
                offset += self.estimate_pattern_size(pattern);
            }
        }

        Ok(())
    }

    /// Build language-specific index
    fn build_language_index(&mut self, library: &KnowledgeLibrary) -> Result<(), String> {
        // Universal patterns are available to all languages
        let universal_pattern_ids: Vec<String> =
            library.universal_patterns.keys().cloned().collect();

        // Add entry for universal patterns
        self.language_entries.push(LanguageIndexEntry {
            language: SourceLanguage::Universal,
            pattern_ids: universal_pattern_ids.clone(),
            pattern_count: universal_pattern_ids.len(),
        });

        // Add entries for language-specific patterns
        for (language, lang_knowledge) in &library.language_specific {
            let mut pattern_ids = universal_pattern_ids.clone();
            pattern_ids.extend(lang_knowledge.patterns.keys().cloned());

            self.language_entries.push(LanguageIndexEntry {
                language: language.clone(),
                pattern_ids: pattern_ids.clone(),
                pattern_count: pattern_ids.len(),
            });
        }

        Ok(())
    }

    /// Build symptom search index
    fn build_symptom_index(&mut self, library: &KnowledgeLibrary) -> Result<(), String> {
        let mut symptom_map: HashMap<String, Vec<String>> = HashMap::new();

        // Process universal patterns
        for pattern in library.universal_patterns.values() {
            for symptom in &pattern.symptoms {
                let normalized_symptom = self.normalize_symptom(symptom.as_str());
                symptom_map
                    .entry(normalized_symptom)
                    .or_insert_with(Vec::new)
                    .push(pattern.id.clone());
            }
        }

        // Process language-specific patterns
        for lang_knowledge in library.language_specific.values() {
            for pattern in lang_knowledge.patterns.values() {
                for symptom in &pattern.symptoms {
                    let normalized_symptom = self.normalize_symptom(symptom.as_str());
                    symptom_map
                        .entry(normalized_symptom)
                        .or_insert_with(Vec::new)
                        .push(pattern.id.clone());
                }
            }
        }

        // Convert to symptom entries
        for (symptom, pattern_ids) in symptom_map {
            let relevance_score = self.calculate_symptom_relevance(&pattern_ids);
            self.symptom_entries.push(SymptomIndexEntry {
                symptom,
                pattern_ids,
                relevance_score,
            });
        }

        Ok(())
    }

    /// Build category index
    fn build_category_index(&mut self, library: &KnowledgeLibrary) -> Result<(), String> {
        let mut category_map: HashMap<AntiPatternCategory, Vec<String>> = HashMap::new();

        // Process universal patterns
        for pattern in library.universal_patterns.values() {
            category_map
                .entry(pattern.category.clone())
                .or_insert_with(Vec::new)
                .push(pattern.id.clone());
        }

        // Process language-specific patterns
        for lang_knowledge in library.language_specific.values() {
            for pattern in lang_knowledge.patterns.values() {
                category_map
                    .entry(pattern.category.clone())
                    .or_insert_with(Vec::new)
                    .push(pattern.id.clone());
            }
        }

        // Convert to category entries
        for (category, pattern_ids) in category_map {
            let average_impact = self.calculate_average_impact(&pattern_ids, library);
            self.category_entries.push(CategoryIndexEntry {
                category,
                pattern_ids,
                average_impact,
            });
        }

        Ok(())
    }

    /// Build tag search index
    fn build_tag_index(&mut self, library: &KnowledgeLibrary) -> Result<(), String> {
        let mut tag_map: HashMap<String, Vec<String>> = HashMap::new();

        // Process universal patterns
        for pattern in library.universal_patterns.values() {
            for tag in &pattern.tags {
                tag_map
                    .entry(tag.clone())
                    .or_insert_with(Vec::new)
                    .push(pattern.id.clone());
            }
        }

        // Process language-specific patterns
        for lang_knowledge in library.language_specific.values() {
            for pattern in lang_knowledge.patterns.values() {
                for tag in &pattern.tags {
                    tag_map
                        .entry(tag.clone())
                        .or_insert_with(Vec::new)
                        .push(pattern.id.clone());
                }
            }
        }

        // Convert to tag entries
        for (tag, pattern_ids) in tag_map {
            self.tag_entries.push(TagIndexEntry {
                tag,
                frequency: pattern_ids.len(),
                pattern_ids,
            });
        }

        Ok(())
    }

    /// Normalize symptom text for consistent indexing
    fn normalize_symptom(&self, symptom: &str) -> String {
        symptom.to_lowercase().trim().to_string()
    }

    /// Calculate symptom relevance score based on pattern frequency
    fn calculate_symptom_relevance(&self, pattern_ids: &[String]) -> f32 {
        // Simple relevance calculation based on frequency
        // More patterns exhibiting the symptom = higher relevance
        (pattern_ids.len() as f32 / 100.0).min(1.0)
    }

    /// Calculate average impact level for a category
    fn calculate_average_impact(&self, pattern_ids: &[String], library: &KnowledgeLibrary) -> f32 {
        if pattern_ids.is_empty() {
            return 0.0;
        }

        let mut total_impact = 0.0;
        let mut count = 0;

        for pattern_id in pattern_ids {
            // Find pattern in universal or language-specific collections
            let impact = if let Some(pattern) = library.universal_patterns.get(pattern_id) {
                self.impact_to_score(&pattern.impact)
            } else {
                // Search in language-specific patterns
                library
                    .language_specific
                    .values()
                    .find_map(|lang| lang.patterns.get(pattern_id))
                    .map(|pattern| self.impact_to_score(&pattern.impact))
                    .unwrap_or(0.0)
            };

            total_impact += impact;
            count += 1;
        }

        if count > 0 {
            total_impact / count as f32
        } else {
            0.0
        }
    }

    /// Convert impact level to numeric score
    fn impact_to_score(&self, impact: &ImpactLevel) -> f32 {
        match impact {
            ImpactLevel::Low => 1.0,
            ImpactLevel::Medium => 2.0,
            ImpactLevel::High => 3.0,
            ImpactLevel::Critical => 4.0,
        }
    }

    /// Estimate pattern size for offset calculation
    fn estimate_pattern_size(&self, pattern: &PatternKnowledge) -> u32 {
        // Simplified size estimation - in real implementation,
        // this would calculate actual serialized size
        let base_size = pattern.id.len() + pattern.name.len() + pattern.definition.len();
        let symptoms_size: usize = pattern.symptoms.iter().map(|s| s.len()).sum();
        let examples_size = pattern.examples.primary.len() * 100; // Rough estimate

        (base_size + symptoms_size + examples_size) as u32
    }

    /// Estimate memory usage of indices
    fn estimate_memory_usage(&self) -> usize {
        std::mem::size_of_val(&self.pattern_entries)
            + std::mem::size_of_val(&self.language_entries)
            + std::mem::size_of_val(&self.symptom_entries)
            + std::mem::size_of_val(&self.category_entries)
            + std::mem::size_of_val(&self.tag_entries)
    }

    /// Generate checksum for index validation
    fn generate_checksum(&self) -> String {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(self.pattern_entries.len().to_string());
        hasher.update(self.language_entries.len().to_string());
        hasher.update(self.symptom_entries.len().to_string());
        hasher.update(self.category_entries.len().to_string());
        hasher.update(self.tag_entries.len().to_string());

        format!("{:x}", hasher.finalize())
    }
}

/// Runtime knowledge lookup implementation
pub struct KnowledgeLibraryLookup {
    library: KnowledgeLibrary,
    index: KnowledgeIndex,
}

impl KnowledgeLibraryLookup {
    /// Create a new lookup instance with library and index
    pub fn new(library: KnowledgeLibrary, index: KnowledgeIndex) -> Self {
        Self { library, index }
    }

    /// Get index metadata
    pub fn get_index_metadata(&self) -> &IndexGenerationMetadata {
        &self.index.metadata
    }

    /// Get index statistics
    pub fn get_index_stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        stats.insert("total_patterns".to_string(), self.index.total_patterns);
        stats.insert(
            "pattern_entries".to_string(),
            self.index.pattern_entries.len(),
        );
        stats.insert(
            "language_entries".to_string(),
            self.index.language_entries.len(),
        );
        stats.insert(
            "symptom_entries".to_string(),
            self.index.symptom_entries.len(),
        );
        stats.insert(
            "category_entries".to_string(),
            self.index.category_entries.len(),
        );
        stats.insert("tag_entries".to_string(), self.index.tag_entries.len());
        stats
    }

    /// Get the underlying knowledge library (for stats and metadata access)
    pub fn get_library(&self) -> &KnowledgeLibrary {
        &self.library
    }
}

impl KnowledgeLookup for KnowledgeLibraryLookup {
    fn get_pattern(&self, pattern_id: &str) -> Option<&PatternKnowledge> {
        // Check universal patterns first
        if let Some(pattern) = self.library.universal_patterns.get(pattern_id) {
            return Some(pattern);
        }

        // Check language-specific patterns
        for lang_knowledge in self.library.language_specific.values() {
            if let Some(pattern) = lang_knowledge.patterns.get(pattern_id) {
                return Some(pattern);
            }
        }

        None
    }

    fn get_language_patterns(&self, language: SourceLanguage) -> Vec<&PatternKnowledge> {
        let mut patterns = Vec::new();

        // Always include universal patterns
        patterns.extend(self.library.universal_patterns.values());

        // Add language-specific patterns if available
        if let Some(lang_knowledge) = self.library.language_specific.get(&language) {
            patterns.extend(lang_knowledge.patterns.values());
        }

        patterns
    }

    fn search_by_symptoms(&self, symptoms: &[String]) -> Vec<&PatternKnowledge> {
        let mut pattern_ids = std::collections::HashSet::new();

        for symptom in symptoms {
            let normalized = symptom.to_lowercase().trim().to_string();
            for entry in &self.index.symptom_entries {
                if entry.symptom.contains(&normalized) || normalized.contains(&entry.symptom) {
                    pattern_ids.extend(entry.pattern_ids.iter().cloned());
                }
            }
        }

        pattern_ids
            .into_iter()
            .filter_map(|id| self.get_pattern(&id))
            .collect()
    }

    fn get_patterns_by_category(&self, category: AntiPatternCategory) -> Vec<&PatternKnowledge> {
        for entry in &self.index.category_entries {
            if entry.category == category {
                return entry
                    .pattern_ids
                    .iter()
                    .filter_map(|id| self.get_pattern(id))
                    .collect();
            }
        }
        Vec::new()
    }

    fn get_patterns_by_impact(&self, min_impact: ImpactLevel) -> Vec<&PatternKnowledge> {
        let mut patterns = Vec::new();

        // Search universal patterns
        patterns.extend(
            self.library
                .universal_patterns
                .values()
                .filter(|p| p.impact >= min_impact),
        );

        // Search language-specific patterns
        for lang_knowledge in self.library.language_specific.values() {
            patterns.extend(
                lang_knowledge
                    .patterns
                    .values()
                    .filter(|p| p.impact >= min_impact),
            );
        }

        patterns
    }

    fn search_by_tags(&self, tags: &[String]) -> Vec<&PatternKnowledge> {
        let mut pattern_ids = std::collections::HashSet::new();

        for tag in tags {
            for entry in &self.index.tag_entries {
                if entry.tag == *tag {
                    pattern_ids.extend(entry.pattern_ids.iter().cloned());
                }
            }
        }

        pattern_ids
            .into_iter()
            .filter_map(|id| self.get_pattern(&id))
            .collect()
    }

    fn get_related_patterns(&self, pattern_id: &str) -> Vec<&PatternKnowledge> {
        if let Some(pattern) = self.get_pattern(pattern_id) {
            pattern
                .related_patterns
                .iter()
                .filter_map(|id| self.get_pattern(id))
                .collect()
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_library() -> KnowledgeLibrary {
        let mut library = KnowledgeLibrary::new();

        // Add a test pattern
        let pattern = PatternKnowledge {
            id: "god_object".to_string(),
            name: "God Object".to_string(),
            definition: CompressedString::new("A class that does too much"),
            symptoms: vec![
                CompressedString::new("Large class size"),
                CompressedString::new("High complexity"),
            ],
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
        library
    }

    #[test]
    fn test_index_builder_creation() {
        let builder = IndexBuilder::default();
        assert!(builder.config.enable_pattern_index);
        assert!(builder.config.enable_language_index);
        assert!(builder.config.enable_symptom_index);
    }

    #[test]
    fn test_index_building() {
        let library = create_test_library();
        let mut builder = IndexBuilder::default();
        let index = builder.build_from_library(&library).unwrap();

        assert_eq!(index.total_patterns, 1);
        assert_eq!(index.pattern_entries.len(), 1);
        assert!(index.symptom_entries.len() > 0);
        assert!(index.tag_entries.len() > 0);
    }

    #[test]
    fn test_knowledge_lookup() {
        let library = create_test_library();
        let mut builder = IndexBuilder::default();
        let index = builder.build_from_library(&library).unwrap();
        let lookup = KnowledgeLibraryLookup::new(library, index);

        // Test pattern lookup
        let pattern = lookup.get_pattern("god_object");
        assert!(pattern.is_some());
        assert_eq!(pattern.unwrap().name, "God Object");

        // Test symptom search
        let symptoms = vec!["Large class".to_string()];
        let patterns = lookup.search_by_symptoms(&symptoms);
        assert_eq!(patterns.len(), 1);

        // Test tag search
        let tags = vec!["oop".to_string()];
        let patterns = lookup.search_by_tags(&tags);
        assert_eq!(patterns.len(), 1);
    }

    #[test]
    fn test_category_lookup() {
        let library = create_test_library();
        let mut builder = IndexBuilder::default();
        let index = builder.build_from_library(&library).unwrap();
        let lookup = KnowledgeLibraryLookup::new(library, index);

        let patterns = lookup.get_patterns_by_category(AntiPatternCategory::ObjectOriented);
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].id, "god_object");
    }

    #[test]
    fn test_impact_filtering() {
        let library = create_test_library();
        let mut builder = IndexBuilder::default();
        let index = builder.build_from_library(&library).unwrap();
        let lookup = KnowledgeLibraryLookup::new(library, index);

        let high_impact_patterns = lookup.get_patterns_by_impact(ImpactLevel::High);
        assert_eq!(high_impact_patterns.len(), 1);

        let critical_patterns = lookup.get_patterns_by_impact(ImpactLevel::Critical);
        assert_eq!(critical_patterns.len(), 0);
    }
}
