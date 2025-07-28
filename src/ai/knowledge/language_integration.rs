//! Language-Specific Knowledge Integration System
//!
//! This module provides the integration layer that combines universal anti-pattern
//! knowledge with language-specific variations, creating enhanced patterns that
//! provide contextually rich, language-aware guidance for AI-powered analysis.

use crate::ai::knowledge::schema::*;
use crate::ai::knowledge::compression::CompressedString;
use crate::ai::knowledge::patterns::{universal, language_specific};
use std::collections::HashMap;

/// Language-specific context for pattern detection and resolution
#[derive(Debug, Clone)]
pub struct LanguageContext {
    /// Programming language
    pub language: SourceLanguage,
    /// Language-specific symptoms and indicators
    pub specific_symptoms: Vec<CompressedString>,
    /// Language-specific detection methods
    pub detection_methods: Vec<DetectionMethod>,
    /// Language-specific solution patterns
    pub solutions: Vec<SolutionPattern>,
    /// Tool and linting recommendations
    pub tools: Vec<CompressedString>,
    /// Relevant framework knowledge
    pub frameworks: Vec<FrameworkKnowledge>,
    /// Standard library guidance
    pub stdlib_guidance: Option<CompressedString>,
}

/// Enhanced pattern with language-specific variations
#[derive(Debug, Clone)]
pub struct EnhancedPattern {
    /// Base universal pattern
    pub universal: PatternKnowledge,
    /// Language-specific enhancements
    pub language_contexts: HashMap<SourceLanguage, LanguageContext>,
    /// Aggregated metadata
    pub metadata: PatternMetadata,
}

/// Metadata about pattern coverage and completeness
#[derive(Debug, Clone)]
pub struct PatternMetadata {
    /// Languages with comprehensive coverage
    pub complete_languages: Vec<SourceLanguage>,
    /// Languages with partial coverage
    pub partial_languages: Vec<SourceLanguage>,
    /// Overall completeness score (0.0 to 1.0)
    pub completeness_score: f32,
    /// Confidence in language-specific detection
    pub language_confidence: HashMap<SourceLanguage, f32>,
}

/// Integrator for combining universal and language-specific knowledge
pub struct LanguageKnowledgeIntegrator {
    /// Universal patterns from UV-332
    universal_patterns: HashMap<String, PatternKnowledge>,
    /// Language-specific libraries from UV-333
    language_libraries: HashMap<SourceLanguage, LanguageKnowledge>,
}

impl LanguageKnowledgeIntegrator {
    /// Create new integrator with universal and language-specific knowledge
    pub fn new(
        universal_patterns: HashMap<String, PatternKnowledge>,
        language_libraries: HashMap<SourceLanguage, LanguageKnowledge>,
    ) -> Self {
        Self {
            universal_patterns,
            language_libraries,
        }
    }

    /// Create enhanced patterns with language-specific variations
    pub fn create_enhanced_patterns(&self) -> HashMap<String, EnhancedPattern> {
        let mut enhanced_patterns = HashMap::new();

        for (pattern_id, universal_pattern) in &self.universal_patterns {
            let language_contexts = self.build_language_contexts(pattern_id);
            let metadata = self.calculate_pattern_metadata(pattern_id, &language_contexts);

            let enhanced_pattern = EnhancedPattern {
                universal: universal_pattern.clone(),
                language_contexts,
                metadata,
            };

            enhanced_patterns.insert(pattern_id.clone(), enhanced_pattern);
        }

        // Add language-specific only patterns
        for (language, lang_knowledge) in &self.language_libraries {
            for (pattern_id, lang_pattern) in &lang_knowledge.patterns {
                if !enhanced_patterns.contains_key(pattern_id) {
                    // This is a language-specific pattern not in universal set
                    let mut language_contexts = HashMap::new();
                    
                    if let Some(context) = self.build_language_context(*language, pattern_id) {
                        language_contexts.insert(*language, context);
                    }

                    let metadata = PatternMetadata {
                        complete_languages: vec![*language],
                        partial_languages: vec![],
                        completeness_score: 1.0, // Complete for this language
                        language_confidence: [(language.clone(), lang_pattern.detection_confidence)]
                            .into_iter().collect(),
                    };

                    let enhanced_pattern = EnhancedPattern {
                        universal: lang_pattern.clone(),
                        language_contexts,
                        metadata,
                    };

                    enhanced_patterns.insert(pattern_id.clone(), enhanced_pattern);
                }
            }
        }

        enhanced_patterns
    }

    /// Build language contexts for a specific pattern
    fn build_language_contexts(&self, pattern_id: &str) -> HashMap<SourceLanguage, LanguageContext> {
        let mut contexts = HashMap::new();

        for (language, _) in &self.language_libraries {
            if let Some(context) = self.build_language_context(*language, pattern_id) {
                contexts.insert(*language, context);
            }
        }

        contexts
    }

    /// Build language context for a specific language and pattern
    fn build_language_context(
        &self, 
        language: SourceLanguage, 
        pattern_id: &str
    ) -> Option<LanguageContext> {
        let lang_knowledge = self.language_libraries.get(&language)?;
        
        // Get language-specific pattern if it exists
        let lang_pattern = lang_knowledge.patterns.get(pattern_id)?;
        
        // Extract language-specific information
        let specific_symptoms = if let Some(lang_specific) = lang_pattern.language_variations.get(&language) {
            lang_specific.symptoms.clone()
        } else {
            // Fallback to pattern's general symptoms
            lang_pattern.symptoms.clone()
        };

        let detection_methods = lang_pattern.detection_methods.clone();
        let solutions = lang_pattern.solutions.clone();
        
        // Get tool recommendations (placeholder - would be extracted from pattern)
        let tools = self.extract_tool_recommendations(language, pattern_id);
        
        // Get relevant frameworks
        let frameworks = self.get_relevant_frameworks(language, pattern_id);
        
        // Get stdlib guidance
        let stdlib_guidance = self.get_stdlib_guidance(language, pattern_id);

        Some(LanguageContext {
            language,
            specific_symptoms,
            detection_methods,
            solutions,
            tools,
            frameworks,
            stdlib_guidance,
        })
    }

    /// Calculate metadata for pattern completeness and confidence
    fn calculate_pattern_metadata(
        &self,
        pattern_id: &str,
        language_contexts: &HashMap<SourceLanguage, LanguageContext>,
    ) -> PatternMetadata {
        let mut complete_languages = Vec::new();
        let mut partial_languages = Vec::new();
        let mut language_confidence = HashMap::new();

        for (language, context) in language_contexts {
            // Determine completeness based on available information
            let completeness = self.assess_language_completeness(language, pattern_id, context);
            
            if completeness > 0.8 {
                complete_languages.push(*language);
            } else if completeness > 0.3 {
                partial_languages.push(*language);
            }

            // Calculate confidence based on detection methods and solutions
            let confidence = self.calculate_language_confidence(context);
            language_confidence.insert(*language, confidence);
        }

        let completeness_score = if language_contexts.is_empty() {
            0.0
        } else {
            complete_languages.len() as f32 / self.language_libraries.len() as f32
        };

        PatternMetadata {
            complete_languages,
            partial_languages,
            completeness_score,
            language_confidence,
        }
    }

    /// Assess completeness of language-specific information
    fn assess_language_completeness(
        &self,
        language: &SourceLanguage,
        pattern_id: &str,
        context: &LanguageContext,
    ) -> f32 {
        let mut score = 0.0;
        let mut max_score = 0.0;

        // Symptoms (weight: 20%)
        max_score += 0.2;
        if !context.specific_symptoms.is_empty() {
            score += 0.2;
        }

        // Detection methods (weight: 25%)
        max_score += 0.25;
        if !context.detection_methods.is_empty() {
            score += 0.25;
        }

        // Solutions (weight: 30%)
        max_score += 0.3;
        if !context.solutions.is_empty() {
            score += 0.3;
        }

        // Tools (weight: 10%)
        max_score += 0.1;
        if !context.tools.is_empty() {
            score += 0.1;
        }

        // Framework knowledge (weight: 10%)
        max_score += 0.1;
        if !context.frameworks.is_empty() {
            score += 0.1;
        }

        // Standard library guidance (weight: 5%)
        max_score += 0.05;
        if context.stdlib_guidance.is_some() {
            score += 0.05;
        }

        score / max_score
    }

    /// Calculate confidence for language-specific detection
    fn calculate_language_confidence(&self, context: &LanguageContext) -> f32 {
        if context.detection_methods.is_empty() {
            return 0.5; // Default confidence
        }

        // Average confidence from detection methods
        let total_confidence: f32 = context.detection_methods.iter()
            .map(|method| match method {
                DetectionMethod::StaticAnalysis { confidence, .. } => *confidence,
                DetectionMethod::MetricThreshold { .. } => 0.9, // High confidence for metrics
                DetectionMethod::AstPattern { .. } => 0.85,
                DetectionMethod::DependencyAnalysis { .. } => 0.8,
                DetectionMethod::RegexPattern { .. } => 0.7,
            })
            .sum();

        total_confidence / context.detection_methods.len() as f32
    }

    /// Extract tool and linting recommendations for language/pattern combination
    fn extract_tool_recommendations(&self, language: SourceLanguage, pattern_id: &str) -> Vec<CompressedString> {
        // This would be implemented based on the pattern and language
        // For now, return some common tools per language
        match language {
            SourceLanguage::Rust => vec![
                CompressedString::new("clippy for lint detection"),
                CompressedString::new("cargo-audit for security"),
                CompressedString::new("miri for memory safety"),
            ],
            SourceLanguage::Python => vec![
                CompressedString::new("pylint for code analysis"),
                CompressedString::new("mypy for type checking"),
                CompressedString::new("bandit for security"),
            ],
            SourceLanguage::JavaScript => vec![
                CompressedString::new("eslint for code quality"),
                CompressedString::new("jshint for error detection"),
                CompressedString::new("sonarjs for code smells"),
            ],
            SourceLanguage::TypeScript => vec![
                CompressedString::new("tslint/eslint with TypeScript rules"),
                CompressedString::new("typescript compiler strict mode"),
                CompressedString::new("type-coverage for type completeness"),
            ],
            SourceLanguage::Java => vec![
                CompressedString::new("SpotBugs for bug detection"),
                CompressedString::new("PMD for code analysis"),
                CompressedString::new("SonarQube for quality gates"),
            ],
            _ => vec![],
        }
    }

    /// Get relevant framework knowledge for language/pattern combination
    fn get_relevant_frameworks(&self, language: SourceLanguage, pattern_id: &str) -> Vec<FrameworkKnowledge> {
        if let Some(lang_knowledge) = self.language_libraries.get(&language) {
            // Return frameworks that are relevant to this pattern
            // This is a simplified implementation - could be more sophisticated
            lang_knowledge.frameworks.values().cloned().collect()
        } else {
            vec![]
        }
    }

    /// Get standard library guidance for language/pattern combination
    fn get_stdlib_guidance(&self, language: SourceLanguage, pattern_id: &str) -> Option<CompressedString> {
        if let Some(lang_knowledge) = self.language_libraries.get(&language) {
            // Find relevant stdlib pattern
            for stdlib_pattern in &lang_knowledge.stdlib_patterns {
                // This is simplified - would match based on pattern type
                if pattern_id.contains(&stdlib_pattern.pattern_name.to_lowercase()) {
                    return Some(stdlib_pattern.recommended_usage.clone());
                }
            }
        }
        None
    }

    /// Get language-specific detection context for AI analysis
    pub fn get_detection_context(
        &self,
        language: SourceLanguage,
        pattern_id: &str,
    ) -> Option<LanguageContext> {
        self.build_language_context(language, pattern_id)
    }

    /// Get enhanced pattern by ID
    pub fn get_enhanced_pattern(&self, pattern_id: &str) -> Option<EnhancedPattern> {
        let enhanced_patterns = self.create_enhanced_patterns();
        enhanced_patterns.get(pattern_id).cloned()
    }

    /// Get all patterns for a specific language with enhancements
    pub fn get_language_patterns(&self, language: SourceLanguage) -> Vec<EnhancedPattern> {
        let enhanced_patterns = self.create_enhanced_patterns();
        enhanced_patterns
            .values()
            .filter(|pattern| pattern.language_contexts.contains_key(&language))
            .cloned()
            .collect()
    }

    /// Get patterns by category with language enhancements
    pub fn get_patterns_by_category(&self, category: AntiPatternCategory) -> Vec<EnhancedPattern> {
        let enhanced_patterns = self.create_enhanced_patterns();
        enhanced_patterns
            .values()
            .filter(|pattern| pattern.universal.category == category)
            .cloned()
            .collect()
    }

    /// Generate language-specific indices for fast lookups
    pub fn generate_language_indices(&self) -> LanguageIndices {
        let enhanced_patterns = self.create_enhanced_patterns();
        
        let mut language_to_patterns = HashMap::new();
        let mut framework_to_patterns = HashMap::new();
        let mut tool_to_patterns = HashMap::new();

        for (pattern_id, pattern) in &enhanced_patterns {
            // Index by language
            for (language, context) in &pattern.language_contexts {
                language_to_patterns
                    .entry(*language)
                    .or_insert_with(Vec::new)
                    .push(pattern_id.clone());

                // Index by framework
                for framework in &context.frameworks {
                    framework_to_patterns
                        .entry(framework.framework_name.clone())
                        .or_insert_with(Vec::new)
                        .push(pattern_id.clone());
                }

                // Index by tools
                for tool in &context.tools {
                    let tool_name = tool.as_str().split(' ').next().unwrap_or("").to_string();
                    tool_to_patterns
                        .entry(tool_name)
                        .or_insert_with(Vec::new)
                        .push(pattern_id.clone());
                }
            }
        }

        LanguageIndices {
            language_to_patterns,
            framework_to_patterns,
            tool_to_patterns,
        }
    }
}

/// Indices for efficient language-specific pattern lookups
#[derive(Debug, Clone)]
pub struct LanguageIndices {
    /// Language -> Pattern IDs mapping
    pub language_to_patterns: HashMap<SourceLanguage, Vec<String>>,
    /// Framework -> Pattern IDs mapping
    pub framework_to_patterns: HashMap<String, Vec<String>>,
    /// Tool -> Pattern IDs mapping
    pub tool_to_patterns: HashMap<String, Vec<String>>,
}

/// Factory for creating integrated knowledge systems
pub struct IntegratedKnowledgeFactory;

impl IntegratedKnowledgeFactory {
    /// Create fully integrated language-aware knowledge system
    pub fn create_integrated_system() -> LanguageKnowledgeIntegrator {
        let universal_patterns = universal::create_universal_patterns();
        let language_libraries = language_specific::create_language_specific_libraries();
        
        LanguageKnowledgeIntegrator::new(universal_patterns, language_libraries)
    }

    /// Create system with custom patterns and libraries
    pub fn create_custom_system(
        universal_patterns: HashMap<String, PatternKnowledge>,
        language_libraries: HashMap<SourceLanguage, LanguageKnowledge>,
    ) -> LanguageKnowledgeIntegrator {
        LanguageKnowledgeIntegrator::new(universal_patterns, language_libraries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integrator_creation() {
        let integrator = IntegratedKnowledgeFactory::create_integrated_system();
        assert!(!integrator.universal_patterns.is_empty());
        assert!(!integrator.language_libraries.is_empty());
    }

    #[test]
    fn test_enhanced_pattern_generation() {
        let integrator = IntegratedKnowledgeFactory::create_integrated_system();
        let enhanced_patterns = integrator.create_enhanced_patterns();
        
        assert!(!enhanced_patterns.is_empty());
        
        // Check that god_object pattern has language enhancements
        if let Some(god_object) = enhanced_patterns.get("god_object") {
            assert!(!god_object.language_contexts.is_empty());
            assert!(god_object.metadata.completeness_score > 0.0);
        }
    }

    #[test]
    fn test_language_context_building() {
        let integrator = IntegratedKnowledgeFactory::create_integrated_system();
        
        if let Some(context) = integrator.get_detection_context(
            SourceLanguage::Rust,
            "god_object"
        ) {
            assert_eq!(context.language, SourceLanguage::Rust);
            assert!(!context.specific_symptoms.is_empty());
        }
    }

    #[test]
    fn test_language_patterns_retrieval() {
        let integrator = IntegratedKnowledgeFactory::create_integrated_system();
        let rust_patterns = integrator.get_language_patterns(SourceLanguage::Rust);
        
        assert!(!rust_patterns.is_empty());
        
        // All patterns should have Rust context
        for pattern in rust_patterns {
            assert!(pattern.language_contexts.contains_key(&SourceLanguage::Rust));
        }
    }

    #[test]
    fn test_indices_generation() {
        let integrator = IntegratedKnowledgeFactory::create_integrated_system();
        let indices = integrator.generate_language_indices();
        
        assert!(!indices.language_to_patterns.is_empty());
        assert!(!indices.framework_to_patterns.is_empty());
        assert!(!indices.tool_to_patterns.is_empty());
    }

    #[test]
    fn test_completeness_assessment() {
        let integrator = IntegratedKnowledgeFactory::create_integrated_system();
        let enhanced_patterns = integrator.create_enhanced_patterns();
        
        for (pattern_id, pattern) in enhanced_patterns {
            // Metadata should be calculated
            assert!(pattern.metadata.completeness_score >= 0.0);
            assert!(pattern.metadata.completeness_score <= 1.0);
            
            // Should have confidence scores for languages with contexts
            for language in pattern.language_contexts.keys() {
                assert!(pattern.metadata.language_confidence.contains_key(language));
            }
        }
    }
}