//! Plugin Integration with Knowledge System
//!
//! This module integrates plugin-provided knowledge with the core knowledge
//! library while maintaining performance and consistency.

#[cfg(feature = "ai")]
use crate::ai::knowledge::context_selection::*;
#[cfg(feature = "ai")]
use crate::ai::knowledge::loader::KnowledgeLibraryLoader;
#[cfg(feature = "ai")]
use crate::ai::knowledge::schema::*;
use crate::plugins::knowledge::*;
use crate::plugins::PluginError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// Stub types for when AI features are disabled
#[cfg(not(feature = "ai"))]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KnowledgeLibrary {
    pub universal_patterns: HashMap<String, PatternKnowledge>,
    pub language_specific: HashMap<SourceLanguage, LanguageKnowledge>,
    pub metadata: LibraryMetadata,
}

#[cfg(not(feature = "ai"))]
#[derive(Debug, Clone, Default)]
pub struct KnowledgeLibraryLoader;

#[cfg(not(feature = "ai"))]
#[derive(Debug, Clone, Hash, PartialEq, Eq, Copy, Serialize, Deserialize)]
pub enum SourceLanguage {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Java,
    Universal,
}

#[cfg(not(feature = "ai"))]
#[derive(Debug, Clone, Hash, PartialEq, Eq, Copy, Serialize, Deserialize)]
pub enum AntiPatternCategory {
    GodObject,
    CyclicDependency,
    MagicValues,
    GlobalState,
    TightCoupling,
    ResourceLeak,
    SilentFailure,
    CodeDuplication,
    LeakyAbstraction,
    DeadCode,
    LongMethod,
    LargeClass,
}

#[cfg(not(feature = "ai"))]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PatternKnowledge {
    pub id: String,
    pub category: AntiPatternCategory,
    pub detection_confidence: f32,
    pub tags: Vec<String>,
    pub solutions: Vec<SolutionPattern>,
    pub detection_methods: Vec<DetectionMethod>,
}

#[cfg(not(feature = "ai"))]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LanguageKnowledge {
    pub patterns: HashMap<String, PatternKnowledge>,
    pub frameworks: HashMap<String, FrameworkKnowledge>,
}

#[cfg(not(feature = "ai"))]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FrameworkKnowledge {
    pub specific_patterns: Vec<String>,
}

#[cfg(not(feature = "ai"))]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LibraryMetadata {
    pub supported_languages: Vec<SourceLanguage>,
}

#[cfg(not(feature = "ai"))]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SolutionPattern {
    pub id: String,
}

#[cfg(not(feature = "ai"))]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DetectionMethod {
    pub id: String,
}

#[cfg(not(feature = "ai"))]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnalysisContext {
    pub language: SourceLanguage,
    pub frameworks: Vec<String>,
    pub detected_patterns: Vec<DetectedPattern>,
}

#[cfg(not(feature = "ai"))]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DetectedPattern {
    pub pattern_id: String,
    pub confidence: f32,
}

#[cfg(not(feature = "ai"))]
#[derive(Debug, Clone, Hash, PartialEq, Eq, Copy, Serialize, Deserialize)]
pub enum SeverityLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[cfg(not(feature = "ai"))]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LocationContext {
    pub file_path: String,
    pub line_number: u32,
}

#[cfg(not(feature = "ai"))]
impl Default for SeverityLevel {
    fn default() -> Self {
        SeverityLevel::Medium
    }
}

#[cfg(not(feature = "ai"))]
impl Default for AntiPatternCategory {
    fn default() -> Self {
        AntiPatternCategory::GodObject
    }
}

#[cfg(not(feature = "ai"))]
impl Default for SourceLanguage {
    fn default() -> Self {
        SourceLanguage::Universal
    }
}

/// Plugin knowledge integration manager
pub struct PluginKnowledgeIntegrator {
    /// Core knowledge library
    core_library: Arc<RwLock<KnowledgeLibrary>>,
    /// Plugin knowledge library
    plugin_library: Arc<RwLock<PluginKnowledgeLibrary>>,
    /// Integration cache for performance
    integration_cache: Arc<RwLock<IntegrationCache>>,
    /// Performance metrics
    metrics: IntegrationMetrics,
}

/// Integration cache for optimized lookups
#[derive(Debug, Clone)]
pub struct IntegrationCache {
    /// Merged pattern index (pattern_id -> source)
    pattern_index: HashMap<String, PatternSource>,
    /// Language-specific pattern cache
    language_patterns: HashMap<SourceLanguage, Vec<String>>,
    /// Framework-specific pattern cache
    framework_patterns: HashMap<String, Vec<String>>,
    /// Category-based pattern cache
    category_patterns: HashMap<AntiPatternCategory, Vec<String>>,
    /// Cache timestamp for invalidation
    last_updated: chrono::DateTime<chrono::Utc>,
}

/// Source of a pattern (core library or plugin)
#[derive(Debug, Clone)]
pub enum PatternSource {
    Core,
    Plugin(String), // plugin_id
}

/// Integration performance metrics
#[derive(Debug, Clone)]
pub struct IntegrationMetrics {
    /// Total patterns available
    pub total_patterns: usize,
    /// Core library patterns
    pub core_patterns: usize,
    /// Plugin patterns
    pub plugin_patterns: usize,
    /// Average retrieval time
    pub avg_retrieval_time_ms: f64,
    /// Cache hit rate
    pub cache_hit_rate: f64,
    /// Last update timestamp
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl PluginKnowledgeIntegrator {
    /// Create new plugin knowledge integrator
    pub fn new(core_library: KnowledgeLibrary) -> Self {
        Self {
            core_library: Arc::new(RwLock::new(core_library)),
            plugin_library: Arc::new(RwLock::new(PluginKnowledgeLibrary::new())),
            integration_cache: Arc::new(RwLock::new(IntegrationCache::new())),
            metrics: IntegrationMetrics::new(),
        }
    }

    /// Update plugin knowledge library
    pub async fn update_plugin_knowledge(
        &mut self,
        plugin_knowledge: PluginKnowledgeLibrary,
    ) -> Result<(), PluginError> {
        // Update plugin library
        {
            let mut plugin_lib = self.plugin_library.write().await;
            *plugin_lib = plugin_knowledge;
        }

        // Rebuild integration cache
        self.rebuild_cache().await?;

        // Update metrics
        self.update_metrics().await;

        Ok(())
    }

    /// Get integrated context for analysis
    pub async fn get_integrated_context(
        &self,
        analysis_context: &AnalysisContext,
        max_patterns: usize,
    ) -> Result<IntegratedKnowledgeContext, PluginError> {
        let start_time = std::time::Instant::now();

        // Get relevant patterns from both core and plugin libraries
        let core_patterns = self.get_core_patterns(analysis_context).await?;
        let plugin_patterns = self.get_plugin_patterns(analysis_context).await?;

        // Merge and rank patterns
        let merged_patterns = self
            .merge_and_rank_patterns(
                core_patterns,
                plugin_patterns,
                analysis_context,
                max_patterns,
            )
            .await?;

        // Get related solutions and detectors
        let solutions = self.get_related_solutions(&merged_patterns).await?;
        let detectors = self.get_related_detectors(&merged_patterns).await?;

        let retrieval_time = start_time.elapsed().as_millis() as f64;

        Ok(IntegratedKnowledgeContext {
            patterns: merged_patterns,
            solutions,
            detectors,
            plugin_contributions: self.get_plugin_contributions().await?,
            retrieval_time_ms: retrieval_time,
            cache_hit: self.was_cache_hit(analysis_context).await,
        })
    }

    /// Get patterns by category with plugin integration
    pub async fn get_patterns_by_category(
        &self,
        category: AntiPatternCategory,
    ) -> Result<Vec<PatternKnowledge>, PluginError> {
        let cache = self.integration_cache.read().await;

        if let Some(pattern_ids) = cache.category_patterns.get(&category) {
            let mut patterns = Vec::new();

            for pattern_id in pattern_ids {
                if let Some(pattern) = self.get_pattern_by_id(pattern_id).await? {
                    patterns.push(pattern);
                }
            }

            Ok(patterns)
        } else {
            Ok(vec![])
        }
    }

    /// Get patterns by language with plugin integration
    pub async fn get_patterns_by_language(
        &self,
        language: SourceLanguage,
    ) -> Result<Vec<PatternKnowledge>, PluginError> {
        let cache = self.integration_cache.read().await;

        if let Some(pattern_ids) = cache.language_patterns.get(&language) {
            let mut patterns = Vec::new();

            for pattern_id in pattern_ids {
                if let Some(pattern) = self.get_pattern_by_id(pattern_id).await? {
                    patterns.push(pattern);
                }
            }

            Ok(patterns)
        } else {
            Ok(vec![])
        }
    }

    /// Get framework-specific patterns
    pub async fn get_framework_patterns(
        &self,
        framework: &str,
    ) -> Result<Vec<PatternKnowledge>, PluginError> {
        let cache = self.integration_cache.read().await;

        if let Some(pattern_ids) = cache.framework_patterns.get(framework) {
            let mut patterns = Vec::new();

            for pattern_id in pattern_ids {
                if let Some(pattern) = self.get_pattern_by_id(pattern_id).await? {
                    patterns.push(pattern);
                }
            }

            Ok(patterns)
        } else {
            Ok(vec![])
        }
    }

    /// Get integration metrics
    pub fn get_metrics(&self) -> &IntegrationMetrics {
        &self.metrics
    }

    /// Rebuild integration cache
    async fn rebuild_cache(&mut self) -> Result<(), PluginError> {
        let mut cache = self.integration_cache.write().await;
        cache.clear();

        // Index core library patterns
        {
            let core_lib = self.core_library.read().await;
            self.index_core_patterns(&core_lib, &mut cache).await;
        }

        // Index plugin library patterns
        {
            let plugin_lib = self.plugin_library.read().await;
            self.index_plugin_patterns(&plugin_lib, &mut cache).await;
        }

        cache.last_updated = chrono::Utc::now();
        Ok(())
    }

    /// Index patterns from core library
    async fn index_core_patterns(&self, core_lib: &KnowledgeLibrary, cache: &mut IntegrationCache) {
        // Index universal patterns
        for (pattern_id, pattern) in &core_lib.universal_patterns {
            cache
                .pattern_index
                .insert(pattern_id.clone(), PatternSource::Core);

            // Index by category
            cache
                .category_patterns
                .entry(pattern.category)
                .or_insert_with(Vec::new)
                .push(pattern_id.clone());

            // Index by language (universal applies to all)
            for language in &core_lib.metadata.supported_languages {
                cache
                    .language_patterns
                    .entry(*language)
                    .or_insert_with(Vec::new)
                    .push(pattern_id.clone());
            }
        }

        // Index language-specific patterns
        for (language, lang_knowledge) in &core_lib.language_specific {
            for (pattern_id, pattern) in &lang_knowledge.patterns {
                cache
                    .pattern_index
                    .insert(pattern_id.clone(), PatternSource::Core);

                cache
                    .category_patterns
                    .entry(pattern.category)
                    .or_insert_with(Vec::new)
                    .push(pattern_id.clone());

                cache
                    .language_patterns
                    .entry(*language)
                    .or_insert_with(Vec::new)
                    .push(pattern_id.clone());
            }

            // Index framework patterns
            for (framework_name, framework_knowledge) in &lang_knowledge.frameworks {
                for pattern_id in &framework_knowledge.specific_patterns {
                    cache
                        .framework_patterns
                        .entry(framework_name.clone())
                        .or_insert_with(Vec::new)
                        .push(pattern_id.clone());
                }
            }
        }
    }

    /// Index patterns from plugin library
    async fn index_plugin_patterns(
        &self,
        plugin_lib: &PluginKnowledgeLibrary,
        cache: &mut IntegrationCache,
    ) {
        for (plugin_id, plugin_knowledge) in plugin_lib.get_all_plugin_knowledge() {
            for pattern in &plugin_knowledge.patterns {
                cache
                    .pattern_index
                    .insert(pattern.id.clone(), PatternSource::Plugin(plugin_id.clone()));

                cache
                    .category_patterns
                    .entry(pattern.category)
                    .or_insert_with(Vec::new)
                    .push(pattern.id.clone());

                // Plugin patterns may support multiple languages
                // This would require additional metadata in the plugin system
                cache
                    .language_patterns
                    .entry(SourceLanguage::Universal)
                    .or_insert_with(Vec::new)
                    .push(pattern.id.clone());
            }
        }
    }

    /// Get patterns from core library for analysis context
    async fn get_core_patterns(
        &self,
        context: &AnalysisContext,
    ) -> Result<Vec<PatternKnowledge>, PluginError> {
        let core_lib = self.core_library.read().await;
        let mut patterns = Vec::new();

        // Get universal patterns
        patterns.extend(core_lib.universal_patterns.values().cloned());

        // Get language-specific patterns
        if let Some(lang_knowledge) = core_lib.language_specific.get(&context.language) {
            patterns.extend(lang_knowledge.patterns.values().cloned());
        }

        Ok(patterns)
    }

    /// Get patterns from plugin library for analysis context
    async fn get_plugin_patterns(
        &self,
        _context: &AnalysisContext,
    ) -> Result<Vec<PatternKnowledge>, PluginError> {
        let plugin_lib = self.plugin_library.read().await;
        let mut patterns = Vec::new();

        for plugin_knowledge in plugin_lib.get_all_plugin_knowledge().values() {
            patterns.extend(plugin_knowledge.patterns.clone());
        }

        Ok(patterns)
    }

    /// Merge and rank patterns from core and plugin libraries
    async fn merge_and_rank_patterns(
        &self,
        core_patterns: Vec<PatternKnowledge>,
        plugin_patterns: Vec<PatternKnowledge>,
        context: &AnalysisContext,
        max_patterns: usize,
    ) -> Result<Vec<RankedPattern>, PluginError> {
        let mut all_patterns = Vec::new();

        // Add core patterns with source information
        for pattern in core_patterns {
            all_patterns.push(RankedPattern {
                pattern,
                source: PatternSource::Core,
                relevance_score: 0.0, // Will be calculated
            });
        }

        // Add plugin patterns with source information
        for pattern in plugin_patterns {
            all_patterns.push(RankedPattern {
                pattern,
                source: PatternSource::Plugin("unknown".to_string()), // Would be properly tracked
                relevance_score: 0.0,                                 // Will be calculated
            });
        }

        // Calculate relevance scores
        for ranked_pattern in &mut all_patterns {
            ranked_pattern.relevance_score =
                self.calculate_relevance_score(&ranked_pattern.pattern, context);
        }

        // Sort by relevance score (descending)
        all_patterns.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());

        // Take top patterns
        all_patterns.truncate(max_patterns);

        Ok(all_patterns)
    }

    /// Calculate relevance score for a pattern
    fn calculate_relevance_score(
        &self,
        pattern: &PatternKnowledge,
        context: &AnalysisContext,
    ) -> f32 {
        let mut score: f32 = 0.0;

        // Base score from pattern confidence
        score += pattern.detection_confidence * 0.3;

        // Language match bonus
        // This would require additional metadata to determine pattern language support
        score += 0.2;

        // Framework relevance
        for framework in &context.frameworks {
            if pattern.tags.iter().any(|tag| tag.contains(framework)) {
                score += 0.2;
                break;
            }
        }

        // Detected pattern relevance
        for detected in &context.detected_patterns {
            if detected.pattern_id == pattern.id {
                score += detected.confidence * 0.3;
            }
        }

        score.min(1.0)
    }

    /// Get related solutions for patterns
    async fn get_related_solutions(
        &self,
        patterns: &[RankedPattern],
    ) -> Result<Vec<SolutionPattern>, PluginError> {
        let mut solutions = Vec::new();

        for ranked_pattern in patterns {
            solutions.extend(ranked_pattern.pattern.solutions.clone());
        }

        // Remove duplicates
        solutions.dedup_by(|a, b| a.id == b.id);

        Ok(solutions)
    }

    /// Get related detectors for patterns
    async fn get_related_detectors(
        &self,
        patterns: &[RankedPattern],
    ) -> Result<Vec<DetectionMethod>, PluginError> {
        let mut detectors = Vec::new();

        for ranked_pattern in patterns {
            detectors.extend(ranked_pattern.pattern.detection_methods.clone());
        }

        Ok(detectors)
    }

    /// Get plugin contributions summary
    async fn get_plugin_contributions(&self) -> Result<PluginContributions, PluginError> {
        let plugin_lib = self.plugin_library.read().await;

        let mut contributions = PluginContributions {
            total_plugins: plugin_lib.plugin_count(),
            pattern_contributions: HashMap::new(),
            framework_support: Vec::new(),
        };

        for (plugin_id, plugin_knowledge) in plugin_lib.get_all_plugin_knowledge() {
            contributions
                .pattern_contributions
                .insert(plugin_id.clone(), plugin_knowledge.patterns.len());
        }

        Ok(contributions)
    }

    /// Check if this was a cache hit
    async fn was_cache_hit(&self, _context: &AnalysisContext) -> bool {
        // Implementation would track cache hits
        false
    }

    /// Get pattern by ID from either core or plugin library
    async fn get_pattern_by_id(
        &self,
        pattern_id: &str,
    ) -> Result<Option<PatternKnowledge>, PluginError> {
        let cache = self.integration_cache.read().await;

        if let Some(source) = cache.pattern_index.get(pattern_id) {
            match source {
                PatternSource::Core => {
                    let core_lib = self.core_library.read().await;

                    // Check universal patterns
                    if let Some(pattern) = core_lib.universal_patterns.get(pattern_id) {
                        return Ok(Some(pattern.clone()));
                    }

                    // Check language-specific patterns
                    for lang_knowledge in core_lib.language_specific.values() {
                        if let Some(pattern) = lang_knowledge.patterns.get(pattern_id) {
                            return Ok(Some(pattern.clone()));
                        }
                    }
                }
                PatternSource::Plugin(plugin_id) => {
                    let plugin_lib = self.plugin_library.read().await;
                    if let Some(plugin_knowledge) = plugin_lib.get_plugin_knowledge(plugin_id) {
                        for pattern in &plugin_knowledge.patterns {
                            if pattern.id == pattern_id {
                                return Ok(Some(pattern.clone()));
                            }
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    /// Update integration metrics
    async fn update_metrics(&mut self) {
        let core_lib = self.core_library.read().await;
        let plugin_lib = self.plugin_library.read().await;

        let core_patterns = core_lib.universal_patterns.len()
            + core_lib
                .language_specific
                .values()
                .map(|lang| lang.patterns.len())
                .sum::<usize>();

        let plugin_patterns = plugin_lib
            .get_all_plugin_knowledge()
            .values()
            .map(|plugin| plugin.patterns.len())
            .sum::<usize>();

        self.metrics = IntegrationMetrics {
            total_patterns: core_patterns + plugin_patterns,
            core_patterns,
            plugin_patterns,
            avg_retrieval_time_ms: self.metrics.avg_retrieval_time_ms, // Would be calculated from actual measurements
            cache_hit_rate: self.metrics.cache_hit_rate, // Would be calculated from actual cache hits
            last_updated: chrono::Utc::now(),
        };
    }
}

/// Integrated knowledge context combining core and plugin knowledge
#[derive(Debug, Clone)]
pub struct IntegratedKnowledgeContext {
    /// Ranked patterns from both core and plugins
    pub patterns: Vec<RankedPattern>,
    /// Related solutions
    pub solutions: Vec<SolutionPattern>,
    /// Related detection methods
    pub detectors: Vec<DetectionMethod>,
    /// Plugin contribution summary
    pub plugin_contributions: PluginContributions,
    /// Retrieval performance
    pub retrieval_time_ms: f64,
    /// Whether this was served from cache
    pub cache_hit: bool,
}

/// Pattern with ranking and source information
#[derive(Debug, Clone)]
pub struct RankedPattern {
    /// The pattern itself
    pub pattern: PatternKnowledge,
    /// Source of the pattern
    pub source: PatternSource,
    /// Relevance score for this context
    pub relevance_score: f32,
}

/// Plugin contributions summary
#[derive(Debug, Clone)]
pub struct PluginContributions {
    /// Total number of active plugins
    pub total_plugins: usize,
    /// Pattern contributions by plugin
    pub pattern_contributions: HashMap<String, usize>,
    /// Framework support provided by plugins
    pub framework_support: Vec<String>,
}

impl IntegrationCache {
    pub fn new() -> Self {
        Self {
            pattern_index: HashMap::new(),
            language_patterns: HashMap::new(),
            framework_patterns: HashMap::new(),
            category_patterns: HashMap::new(),
            last_updated: chrono::Utc::now(),
        }
    }

    pub fn clear(&mut self) {
        self.pattern_index.clear();
        self.language_patterns.clear();
        self.framework_patterns.clear();
        self.category_patterns.clear();
    }
}

impl IntegrationMetrics {
    pub fn new() -> Self {
        Self {
            total_patterns: 0,
            core_patterns: 0,
            plugin_patterns: 0,
            avg_retrieval_time_ms: 0.0,
            cache_hit_rate: 0.0,
            last_updated: chrono::Utc::now(),
        }
    }
}
