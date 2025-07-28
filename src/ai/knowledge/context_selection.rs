//! Dynamic Context Selection Engine
//!
//! This module implements intelligent context selection algorithms that dynamically
//! choose the most relevant knowledge from the comprehensive pattern library based
//! on analysis context, detected issues, and AI prompt requirements.

use crate::ai::knowledge::schema::*;
use crate::ai::knowledge::language_integration::{EnhancedPattern, LanguageContext};
use std::collections::HashMap;
use std::time::Instant;

/// Dynamic context selection engine with multi-dimensional scoring
pub struct DynamicContextSelector {
    /// Enhanced patterns with language-specific variations
    enhanced_patterns: HashMap<String, EnhancedPattern>,
    /// Context selection configuration
    config: ContextSelectionConfig,
    /// Usage analytics for adaptive learning
    analytics: ContextAnalytics,
    /// Performance metrics
    metrics: SelectionMetrics,
}

/// Configuration for context selection behavior
#[derive(Debug, Clone)]
pub struct ContextSelectionConfig {
    /// Maximum number of patterns to include in context
    pub max_patterns: usize,
    /// Token budget for AI prompts
    pub token_budget: usize,
    /// Minimum relevance score threshold
    pub relevance_threshold: f32,
    /// Weight factors for different scoring dimensions
    pub scoring_weights: ScoringWeights,
    /// Enable adaptive learning from usage patterns
    pub enable_adaptive_learning: bool,
    /// Context selection strategy
    pub selection_strategy: SelectionStrategy,
}

/// Weights for multi-dimensional relevance scoring
#[derive(Debug, Clone)]
pub struct ScoringWeights {
    /// Language match importance (0.0 to 1.0)
    pub language_match: f32,
    /// Framework relevance importance
    pub framework_relevance: f32,
    /// Pattern similarity importance
    pub pattern_similarity: f32,
    /// Detection confidence importance
    pub detection_confidence: f32,
    /// Solution applicability importance
    pub solution_applicability: f32,
    /// Historical usage importance (adaptive learning)
    pub usage_frequency: f32,
}

/// Context selection strategies
#[derive(Debug, Clone)]
pub enum SelectionStrategy {
    /// Select highest scoring patterns
    TopScoring,
    /// Diversified selection across pattern categories
    Diversified,
    /// Focused selection on specific problem domain
    Focused { domain: AntiPatternCategory },
    /// Adaptive selection based on user preferences
    Adaptive,
}

/// Analysis context for intelligent pattern selection
#[derive(Debug, Clone)]
pub struct AnalysisContext {
    /// Programming language being analyzed
    pub language: SourceLanguage,
    /// Detected frameworks and libraries
    pub frameworks: Vec<String>,
    /// Detected anti-patterns and issues
    pub detected_patterns: Vec<DetectedPattern>,
    /// Codebase characteristics
    pub codebase_info: CodebaseInfo,
    /// User intent and analysis goals
    pub user_intent: UserIntent,
    /// Token budget constraints
    pub token_constraints: TokenConstraints,
}

/// Information about detected patterns in the codebase
#[derive(Debug, Clone)]
pub struct DetectedPattern {
    /// Pattern identifier
    pub pattern_id: String,
    /// Detection confidence (0.0 to 1.0)
    pub confidence: f32,
    /// Severity level
    pub severity: SeverityLevel,
    /// Location context
    pub location: LocationContext,
    /// Related patterns
    pub related_patterns: Vec<String>,
}

/// Severity levels for pattern detection
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub enum SeverityLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Location context for detected patterns
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LocationContext {
    /// File path
    pub file_path: String,
    /// Line number range
    pub line_range: (usize, usize),
    /// Function or class context
    pub context_name: Option<String>,
}

/// Codebase characteristics for context selection
#[derive(Debug, Clone)]
pub struct CodebaseInfo {
    /// Estimated codebase size
    pub size_category: CodebaseSizeCategory,
    /// Architectural patterns detected
    pub architectural_patterns: Vec<String>,
    /// Complexity metrics
    pub complexity_metrics: ComplexityMetrics,
    /// Team experience level
    pub team_experience: ExperienceLevel,
}

/// Codebase size categories
#[derive(Debug, Clone)]
pub enum CodebaseSizeCategory {
    Small,    // < 10k LOC
    Medium,   // 10k - 100k LOC
    Large,    // 100k - 1M LOC
    VeryLarge, // > 1M LOC
}

/// Complexity metrics for codebase analysis
#[derive(Debug, Clone)]
pub struct ComplexityMetrics {
    /// Cyclomatic complexity average
    pub cyclomatic_complexity: f32,
    /// Nesting depth average
    pub nesting_depth: f32,
    /// Coupling metrics
    pub coupling_score: f32,
    /// Technical debt ratio
    pub tech_debt_ratio: f32,
}

/// Team experience levels
#[derive(Debug, Clone)]
pub enum ExperienceLevel {
    Junior,
    Mid,
    Senior,
    Expert,
}

/// User intent for targeted knowledge selection
#[derive(Debug, Clone)]
pub struct UserIntent {
    /// Analysis goal
    pub goal: AnalysisGoal,
    /// Desired explanation depth
    pub explanation_depth: ExplanationDepth,
    /// Focus areas
    pub focus_areas: Vec<AntiPatternCategory>,
    /// Preferred solution types
    pub solution_preferences: Vec<SolutionType>,
}

/// Analysis goals
#[derive(Debug, Clone)]
pub enum AnalysisGoal {
    QuickOverview,
    DetailedAnalysis,
    SecurityAudit,
    PerformanceOptimization,
    MaintenanceReview,
    CodeReview,
}

/// Explanation depth preferences
#[derive(Debug, Clone)]
pub enum ExplanationDepth {
    Brief,
    Standard,
    Detailed,
    Comprehensive,
}

/// Solution type preferences
#[derive(Debug, Clone)]
pub enum SolutionType {
    Quick,
    Comprehensive,
    Educational,
    Production,
}

/// Token budget constraints for AI prompts
#[derive(Debug, Clone)]
pub struct TokenConstraints {
    /// Maximum total tokens
    pub max_tokens: usize,
    /// Reserved tokens for base prompt
    pub reserved_tokens: usize,
    /// Available tokens for knowledge context
    pub available_tokens: usize,
    /// Token estimation per pattern
    pub tokens_per_pattern: usize,
}

/// Pattern candidate for context selection
#[derive(Debug, Clone)]
pub struct PatternCandidate {
    /// Enhanced pattern with language variations
    pub pattern: EnhancedPattern,
    /// Type of match (direct, related, proactive)
    pub match_type: MatchType,
    /// Base relevance score
    pub base_score: f32,
    /// Context that triggered this candidate
    pub detection_context: Option<DetectedPattern>,
}

/// Types of pattern matches
#[derive(Debug, Clone)]
pub enum MatchType {
    /// Directly detected in the codebase
    Direct,
    /// Related to detected patterns
    Related,
    /// Language or framework specific
    LanguageSpecific,
    /// Proactive suggestion
    Proactive,
}

/// Scored pattern with relevance assessment
#[derive(Debug, Clone)]
pub struct ScoredPattern {
    /// Pattern candidate
    pub candidate: PatternCandidate,
    /// Overall relevance score
    pub relevance_score: f32,
    /// Detailed score breakdown
    pub score_breakdown: ScoreBreakdown,
}

/// Detailed breakdown of scoring dimensions
#[derive(Debug, Clone)]
pub struct ScoreBreakdown {
    /// Language match score
    pub language_score: f32,
    /// Framework relevance score
    pub framework_score: f32,
    /// Pattern similarity score
    pub similarity_score: f32,
    /// Detection confidence score
    pub confidence_score: f32,
    /// Solution applicability score
    pub solution_score: f32,
    /// Usage frequency score
    pub usage_score: f32,
}

/// Selected context for AI prompt building
#[derive(Debug, Clone)]
pub struct SelectedContext {
    /// Selected patterns with relevance scores
    pub patterns: Vec<ScoredPattern>,
    /// Language-specific context
    pub language_context: Option<LanguageContext>,
    /// Framework-specific guidance
    pub framework_guidance: Vec<FrameworkKnowledge>,
    /// Context metadata
    pub metadata: ContextMetadata,
    /// Selection performance metrics
    pub selection_metrics: SelectionPerformanceMetrics,
}

/// Metadata about the selected context
#[derive(Debug, Clone)]
pub struct ContextMetadata {
    /// Total patterns considered
    pub total_candidates: usize,
    /// Patterns selected
    pub selected_count: usize,
    /// Average relevance score
    pub average_relevance: f32,
    /// Token usage
    pub token_usage: TokenUsage,
    /// Selection strategy used
    pub strategy_used: SelectionStrategy,
    /// Selection time
    pub selection_time_ms: u64,
}

/// Token usage information
#[derive(Debug, Clone)]
pub struct TokenUsage {
    /// Tokens used for selected patterns
    pub used_tokens: usize,
    /// Available tokens
    pub available_tokens: usize,
    /// Utilization percentage
    pub utilization_percent: f32,
}

/// Performance metrics for selection operations
#[derive(Debug, Clone)]
pub struct SelectionPerformanceMetrics {
    /// Candidate generation time
    pub candidate_generation_ms: u64,
    /// Scoring time
    pub scoring_time_ms: u64,
    /// Strategy application time
    pub strategy_time_ms: u64,
    /// Token optimization time
    pub optimization_time_ms: u64,
    /// Total selection time
    pub total_time_ms: u64,
}

/// Context selection errors
#[derive(Debug, thiserror::Error)]
pub enum ContextSelectionError {
    #[error("No patterns available for selection")]
    NoPatternsAvailable,
    #[error("Token budget too small: {budget} tokens available, minimum {minimum} required")]
    InsufficientTokenBudget { budget: usize, minimum: usize },
    #[error("Invalid scoring configuration: {details}")]
    InvalidScoringConfig { details: String },
    #[error("Analytics error: {source}")]
    AnalyticsError { source: Box<dyn std::error::Error + Send + Sync> },
    #[error("Pattern processing error: {details}")]
    PatternProcessingError { details: String },
}

impl DynamicContextSelector {
    /// Create new context selector with enhanced patterns
    pub fn new(
        enhanced_patterns: HashMap<String, EnhancedPattern>,
        config: ContextSelectionConfig,
    ) -> Self {
        Self {
            enhanced_patterns,
            config,
            analytics: ContextAnalytics::new(),
            metrics: SelectionMetrics::new(),
        }
    }

    /// Select optimal knowledge context for AI prompt building
    pub fn select_context(
        &mut self,
        analysis_context: &AnalysisContext,
    ) -> Result<SelectedContext, ContextSelectionError> {
        let start_time = Instant::now();

        // 1. Generate candidate patterns
        let candidate_start = Instant::now();
        let candidates = self.generate_candidates(analysis_context)?;
        let candidate_time = candidate_start.elapsed().as_millis() as u64;

        // 2. Score patterns using multi-dimensional algorithm
        let scoring_start = Instant::now();
        let scored_patterns = self.score_patterns(&candidates, analysis_context)?;
        let scoring_time = scoring_start.elapsed().as_millis() as u64;

        // 3. Apply selection strategy
        let strategy_start = Instant::now();
        let selected_patterns = self.apply_selection_strategy(&scored_patterns, analysis_context)?;
        let strategy_time = strategy_start.elapsed().as_millis() as u64;

        // 4. Optimize for token budget
        let optimization_start = Instant::now();
        let optimized_selection = self.optimize_for_token_budget(&selected_patterns, analysis_context)?;
        let optimization_time = optimization_start.elapsed().as_millis() as u64;

        // 5. Build final context
        let selected_context = self.build_selected_context(
            optimized_selection,
            analysis_context,
            SelectionPerformanceMetrics {
                candidate_generation_ms: candidate_time,
                scoring_time_ms: scoring_time,
                strategy_time_ms: strategy_time,
                optimization_time_ms: optimization_time,
                total_time_ms: start_time.elapsed().as_millis() as u64,
            }
        )?;

        // 6. Update analytics and metrics
        self.update_analytics(&selected_context, analysis_context);
        self.metrics.record_selection_time(start_time.elapsed());

        Ok(selected_context)
    }

    /// Generate candidate patterns based on analysis context
    fn generate_candidates(
        &self,
        context: &AnalysisContext,
    ) -> Result<Vec<PatternCandidate>, ContextSelectionError> {
        let mut candidates = Vec::new();

        // 1. Direct matches from detected patterns
        for detected in &context.detected_patterns {
            if let Some(pattern) = self.enhanced_patterns.get(&detected.pattern_id) {
                candidates.push(PatternCandidate {
                    pattern: pattern.clone(),
                    match_type: MatchType::Direct,
                    base_score: detected.confidence,
                    detection_context: Some(detected.clone()),
                });
            }
        }

        // 2. Related patterns through similarity
        for detected in &context.detected_patterns {
            let related = self.find_related_patterns(&detected.pattern_id, context)?;
            for related_pattern in related {
                candidates.push(PatternCandidate {
                    pattern: related_pattern,
                    match_type: MatchType::Related,
                    base_score: 0.7, // Lower base score for related patterns
                    detection_context: Some(detected.clone()),
                });
            }
        }

        // 3. Language and framework specific patterns
        let language_patterns = self.find_language_patterns(context)?;
        candidates.extend(language_patterns);

        // 4. Proactive suggestions based on codebase characteristics
        let proactive_patterns = self.find_proactive_patterns(context)?;
        candidates.extend(proactive_patterns);

        Ok(candidates)
    }

    /// Find related patterns based on pattern relationships
    fn find_related_patterns(
        &self,
        pattern_id: &str,
        context: &AnalysisContext,
    ) -> Result<Vec<EnhancedPattern>, ContextSelectionError> {
        let mut related = Vec::new();

        if let Some(base_pattern) = self.enhanced_patterns.get(pattern_id) {
            // Find patterns by related_patterns field
            for related_id in &base_pattern.universal.related_patterns {
                if let Some(related_pattern) = self.enhanced_patterns.get(related_id) {
                    related.push(related_pattern.clone());
                }
            }

            // Find patterns by category similarity
            for (id, pattern) in &self.enhanced_patterns {
                if id != pattern_id && pattern.universal.category == base_pattern.universal.category {
                    related.push(pattern.clone());
                }
            }

            // Find patterns by tag similarity
            for (id, pattern) in &self.enhanced_patterns {
                if id != pattern_id {
                    let common_tags = base_pattern.universal.tags.iter()
                        .filter(|tag| pattern.universal.tags.contains(tag))
                        .count();
                    
                    if common_tags > 0 && common_tags >= base_pattern.universal.tags.len() / 2 {
                        related.push(pattern.clone());
                    }
                }
            }
        }

        Ok(related)
    }

    /// Find language-specific patterns relevant to the context
    fn find_language_patterns(
        &self,
        context: &AnalysisContext,
    ) -> Result<Vec<PatternCandidate>, ContextSelectionError> {
        let mut candidates = Vec::new();

        for (pattern_id, pattern) in &self.enhanced_patterns {
            if pattern.language_contexts.contains_key(&context.language) {
                // Skip if already included as direct match
                let already_included = context.detected_patterns.iter()
                    .any(|detected| detected.pattern_id == *pattern_id);

                if !already_included {
                    candidates.push(PatternCandidate {
                        pattern: pattern.clone(),
                        match_type: MatchType::LanguageSpecific,
                        base_score: 0.6,
                        detection_context: None,
                    });
                }
            }
        }

        Ok(candidates)
    }

    /// Find proactive pattern suggestions based on codebase characteristics
    fn find_proactive_patterns(
        &self,
        context: &AnalysisContext,
    ) -> Result<Vec<PatternCandidate>, ContextSelectionError> {
        let mut candidates = Vec::new();

        // Suggest patterns based on codebase size
        match context.codebase_info.size_category {
            CodebaseSizeCategory::Large | CodebaseSizeCategory::VeryLarge => {
                // Suggest architectural patterns for large codebases
                for (pattern_id, pattern) in &self.enhanced_patterns {
                    if pattern.universal.category == AntiPatternCategory::Architectural {
                        candidates.push(PatternCandidate {
                            pattern: pattern.clone(),
                            match_type: MatchType::Proactive,
                            base_score: 0.5,
                            detection_context: None,
                        });
                    }
                }
            }
            _ => {}
        }

        // Suggest patterns based on complexity metrics
        if context.codebase_info.complexity_metrics.cyclomatic_complexity > 10.0 {
            for (pattern_id, pattern) in &self.enhanced_patterns {
                if pattern.universal.tags.contains(&"complexity".to_string()) {
                    candidates.push(PatternCandidate {
                        pattern: pattern.clone(),
                        match_type: MatchType::Proactive,
                        base_score: 0.4,
                        detection_context: None,
                    });
                }
            }
        }

        Ok(candidates)
    }

    /// Score patterns using multi-dimensional relevance algorithm
    fn score_patterns(
        &self,
        candidates: &[PatternCandidate],
        context: &AnalysisContext,
    ) -> Result<Vec<ScoredPattern>, ContextSelectionError> {
        let mut scored = Vec::new();

        for candidate in candidates {
            let score = self.calculate_relevance_score(candidate, context)?;

            if score >= self.config.relevance_threshold {
                scored.push(ScoredPattern {
                    candidate: candidate.clone(),
                    relevance_score: score,
                    score_breakdown: self.calculate_score_breakdown(candidate, context)?,
                });
            }
        }

        // Sort by relevance score (descending)
        scored.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());

        Ok(scored)
    }

    /// Calculate multi-dimensional relevance score
    fn calculate_relevance_score(
        &self,
        candidate: &PatternCandidate,
        context: &AnalysisContext,
    ) -> Result<f32, ContextSelectionError> {
        let weights = &self.config.scoring_weights;
        let mut total_score = 0.0;
        let mut total_weight = 0.0;

        // 1. Language match score
        let language_score = self.calculate_language_score(candidate, context);
        total_score += language_score * weights.language_match;
        total_weight += weights.language_match;

        // 2. Framework relevance score
        let framework_score = self.calculate_framework_score(candidate, context);
        total_score += framework_score * weights.framework_relevance;
        total_weight += weights.framework_relevance;

        // 3. Pattern similarity score
        let similarity_score = self.calculate_similarity_score(candidate, context);
        total_score += similarity_score * weights.pattern_similarity;
        total_weight += weights.pattern_similarity;

        // 4. Detection confidence score
        let confidence_score = candidate.base_score;
        total_score += confidence_score * weights.detection_confidence;
        total_weight += weights.detection_confidence;

        // 5. Solution applicability score
        let solution_score = self.calculate_solution_score(candidate, context);
        total_score += solution_score * weights.solution_applicability;
        total_weight += weights.solution_applicability;

        // 6. Usage frequency score (adaptive learning)
        if self.config.enable_adaptive_learning {
            let usage_score = self.analytics.get_usage_score(&candidate.pattern.universal.id);
            total_score += usage_score * weights.usage_frequency;
            total_weight += weights.usage_frequency;
        }

        // Normalize by total weight
        Ok(if total_weight > 0.0 { total_score / total_weight } else { 0.0 })
    }

    /// Calculate detailed score breakdown for transparency
    fn calculate_score_breakdown(
        &self,
        candidate: &PatternCandidate,
        context: &AnalysisContext,
    ) -> Result<ScoreBreakdown, ContextSelectionError> {
        Ok(ScoreBreakdown {
            language_score: self.calculate_language_score(candidate, context),
            framework_score: self.calculate_framework_score(candidate, context),
            similarity_score: self.calculate_similarity_score(candidate, context),
            confidence_score: candidate.base_score,
            solution_score: self.calculate_solution_score(candidate, context),
            usage_score: if self.config.enable_adaptive_learning {
                self.analytics.get_usage_score(&candidate.pattern.universal.id)
            } else {
                0.5
            },
        })
    }

    /// Calculate language-specific relevance score
    fn calculate_language_score(
        &self,
        candidate: &PatternCandidate,
        context: &AnalysisContext,
    ) -> f32 {
        // Exact language match
        if candidate.pattern.language_contexts.contains_key(&context.language) {
            return 1.0;
        }

        // Language family similarity based on context language
        let language_similarity = match context.language {
            SourceLanguage::JavaScript => {
                if candidate.pattern.language_contexts.contains_key(&SourceLanguage::TypeScript) {
                    0.9
                } else {
                    0.3
                }
            },
            SourceLanguage::TypeScript => {
                if candidate.pattern.language_contexts.contains_key(&SourceLanguage::JavaScript) {
                    0.9
                } else {
                    0.3
                }
            },
            SourceLanguage::Java => {
                if candidate.pattern.language_contexts.contains_key(&SourceLanguage::CSharp) {
                    0.7
                } else {
                    0.3
                }
            },
            _ => 0.3, // Some cross-language applicability
        };

        language_similarity
    }

    /// Calculate framework relevance score
    fn calculate_framework_score(
        &self,
        candidate: &PatternCandidate,
        context: &AnalysisContext,
    ) -> f32 {
        if context.frameworks.is_empty() {
            return 0.5; // Neutral score when no frameworks detected
        }

        let mut max_score: f32 = 0.0;

        // Check if pattern has framework-specific knowledge
        if let Some(lang_context) = candidate.pattern.language_contexts.get(&context.language) {
            for framework in &context.frameworks {
                for pattern_framework in &lang_context.frameworks {
                    if pattern_framework.framework_name.to_lowercase().contains(&framework.to_lowercase()) {
                        max_score = max_score.max(1.0);
                    }
                }
            }
        }

        max_score
    }

    /// Calculate pattern similarity score using semantic analysis
    fn calculate_similarity_score(
        &self,
        candidate: &PatternCandidate,
        context: &AnalysisContext,
    ) -> f32 {
        let mut similarity_score: f32 = 0.0;

        for detected in &context.detected_patterns {
            // Direct pattern match
            if candidate.pattern.universal.id == detected.pattern_id {
                similarity_score = similarity_score.max(1.0);
                continue;
            }

            // Category similarity
            let category_similarity = self.calculate_category_similarity(
                &candidate.pattern.universal.category,
                &detected.pattern_id,
            );
            similarity_score = similarity_score.max(category_similarity);

            // Tag-based similarity
            let tag_similarity = self.calculate_tag_similarity(
                &candidate.pattern.universal.tags,
                &detected.pattern_id,
            );
            similarity_score = similarity_score.max(tag_similarity);
        }

        similarity_score
    }

    /// Calculate category-based similarity
    fn calculate_category_similarity(
        &self,
        candidate_category: &AntiPatternCategory,
        detected_pattern_id: &str,
    ) -> f32 {
        if let Some(detected_pattern) = self.enhanced_patterns.get(detected_pattern_id) {
            if detected_pattern.universal.category == *candidate_category {
                return 0.8; // High similarity for same category
            }
            
            // Define category relationships
            match (candidate_category, &detected_pattern.universal.category) {
                (AntiPatternCategory::ObjectOriented, AntiPatternCategory::Architectural) => 0.6,
                (AntiPatternCategory::Architectural, AntiPatternCategory::ObjectOriented) => 0.6,
                (AntiPatternCategory::Performance, AntiPatternCategory::Memory) => 0.7,
                (AntiPatternCategory::Memory, AntiPatternCategory::Performance) => 0.7,
                (AntiPatternCategory::Security, AntiPatternCategory::ErrorHandling) => 0.5,
                _ => 0.2,
            }
        } else {
            0.0
        }
    }

    /// Calculate tag-based similarity
    fn calculate_tag_similarity(
        &self,
        candidate_tags: &[String],
        detected_pattern_id: &str,
    ) -> f32 {
        if let Some(detected_pattern) = self.enhanced_patterns.get(detected_pattern_id) {
            let common_tags = candidate_tags.iter()
                .filter(|tag| detected_pattern.universal.tags.contains(tag))
                .count();
            
            if candidate_tags.is_empty() || detected_pattern.universal.tags.is_empty() {
                return 0.0;
            }
            
            let total_unique_tags = candidate_tags.len() + detected_pattern.universal.tags.len() - common_tags;
            common_tags as f32 / total_unique_tags as f32
        } else {
            0.0
        }
    }

    /// Calculate solution applicability score
    fn calculate_solution_score(
        &self,
        candidate: &PatternCandidate,
        context: &AnalysisContext,
    ) -> f32 {
        if candidate.pattern.universal.solutions.is_empty() {
            return 0.2; // Low score for patterns without solutions
        }

        let mut applicability_score = 0.0;
        let solution_count = candidate.pattern.universal.solutions.len() as f32;

        for solution in &candidate.pattern.universal.solutions {
            let mut solution_score = 0.5; // Base score

            // Adjust based on effort level preference
            match (&solution.effort_level, &context.user_intent.goal) {
                (EffortLevel::Trivial | EffortLevel::Low, AnalysisGoal::QuickOverview) => {
                    solution_score += 0.3;
                }
                (EffortLevel::Medium | EffortLevel::High, AnalysisGoal::DetailedAnalysis) => {
                    solution_score += 0.3;
                }
                (EffortLevel::Significant, AnalysisGoal::MaintenanceReview) => {
                    solution_score += 0.2;
                }
                _ => {}
            }

            // Adjust based on impact level
            match solution.expected_impact {
                ImpactLevel::Critical => solution_score += 0.2,
                ImpactLevel::High => solution_score += 0.1,
                _ => {}
            }

            applicability_score += solution_score;
        }

        (applicability_score / solution_count).min(1.0)
    }

    /// Apply selection strategy to choose final patterns
    fn apply_selection_strategy(
        &self,
        scored_patterns: &[ScoredPattern],
        context: &AnalysisContext,
    ) -> Result<Vec<ScoredPattern>, ContextSelectionError> {
        match self.config.selection_strategy {
            SelectionStrategy::TopScoring => {
                Ok(scored_patterns.iter()
                    .take(self.config.max_patterns)
                    .cloned()
                    .collect())
            },
            SelectionStrategy::Diversified => {
                self.apply_diversified_selection(scored_patterns, context)
            },
            SelectionStrategy::Focused { ref domain } => {
                self.apply_focused_selection(scored_patterns, domain, context)
            },
            SelectionStrategy::Adaptive => {
                self.apply_adaptive_selection(scored_patterns, context)
            },
        }
    }

    /// Apply diversified selection strategy
    fn apply_diversified_selection(
        &self,
        scored_patterns: &[ScoredPattern],
        _context: &AnalysisContext,
    ) -> Result<Vec<ScoredPattern>, ContextSelectionError> {
        let mut selected = Vec::new();
        let mut category_counts: HashMap<AntiPatternCategory, usize> = HashMap::new();
        let max_per_category = (self.config.max_patterns / 4).max(1); // Distribute across categories

        for pattern in scored_patterns {
            let category = &pattern.candidate.pattern.universal.category;
            let current_count = category_counts.get(category).unwrap_or(&0);

            if *current_count < max_per_category && selected.len() < self.config.max_patterns {
                selected.push(pattern.clone());
                category_counts.insert(*category, current_count + 1);
            }
        }

        // Fill remaining slots with highest scoring patterns
        for pattern in scored_patterns {
            if selected.len() >= self.config.max_patterns {
                break;
            }

            if !selected.iter().any(|s| s.candidate.pattern.universal.id == pattern.candidate.pattern.universal.id) {
                selected.push(pattern.clone());
            }
        }

        Ok(selected)
    }

    /// Apply focused selection strategy
    fn apply_focused_selection(
        &self,
        scored_patterns: &[ScoredPattern],
        domain: &AntiPatternCategory,
        _context: &AnalysisContext,
    ) -> Result<Vec<ScoredPattern>, ContextSelectionError> {
        let focused_patterns: Vec<_> = scored_patterns.iter()
            .filter(|pattern| pattern.candidate.pattern.universal.category == *domain)
            .take(self.config.max_patterns)
            .cloned()
            .collect();

        Ok(focused_patterns)
    }

    /// Apply adaptive selection strategy
    fn apply_adaptive_selection(
        &self,
        scored_patterns: &[ScoredPattern],
        context: &AnalysisContext,
    ) -> Result<Vec<ScoredPattern>, ContextSelectionError> {
        // For now, fall back to diversified selection
        // Future implementation would use ML or user preference learning
        self.apply_diversified_selection(scored_patterns, context)
    }

    /// Optimize selection for token budget constraints
    fn optimize_for_token_budget(
        &self,
        selected_patterns: &[ScoredPattern],
        context: &AnalysisContext,
    ) -> Result<Vec<ScoredPattern>, ContextSelectionError> {
        let mut optimized = Vec::new();
        let mut used_tokens = 0;
        let available_tokens = context.token_constraints.available_tokens;

        for pattern in selected_patterns {
            let pattern_tokens = self.estimate_pattern_tokens(&pattern.candidate.pattern);

            if used_tokens + pattern_tokens <= available_tokens {
                optimized.push(pattern.clone());
                used_tokens += pattern_tokens;
            } else {
                // Try to fit a smaller version of the pattern
                if let Some(compressed_pattern) = self.compress_pattern_for_budget(
                    pattern,
                    available_tokens - used_tokens
                ) {
                    optimized.push(compressed_pattern);
                    break; // Budget exhausted
                }
            }
        }

        Ok(optimized)
    }

    /// Estimate token count for a pattern
    fn estimate_pattern_tokens(&self, pattern: &EnhancedPattern) -> usize {
        // Rough estimation based on content length
        let mut tokens = 0;

        // Base pattern info
        tokens += pattern.universal.name.len() / 4; // ~4 chars per token
        tokens += pattern.universal.definition.as_str().len() / 4;
        tokens += pattern.universal.symptoms.iter()
            .map(|s| s.as_str().len() / 4)
            .sum::<usize>();

        // Solutions
        tokens += pattern.universal.solutions.iter()
            .map(|s| {
                s.title.len() / 4 + 
                s.implementation.as_str().len() / 4 +
                s.examples.iter().map(|e| e.explanation.as_str().len() / 4).sum::<usize>()
            })
            .sum::<usize>();

        tokens.max(50) // Minimum token estimate
    }

    /// Try to compress pattern for budget constraints
    fn compress_pattern_for_budget(
        &self,
        pattern: &ScoredPattern,
        available_tokens: usize,
    ) -> Option<ScoredPattern> {
        if available_tokens < 50 {
            return None; // Too small to fit any useful pattern
        }

        // Create a compressed version by reducing examples and details
        let mut compressed_pattern = pattern.clone();
        
        // Keep only the most essential information
        compressed_pattern.candidate.pattern.universal.examples.primary.truncate(1);
        compressed_pattern.candidate.pattern.universal.solutions.truncate(1);

        Some(compressed_pattern)
    }

    /// Build the final selected context
    fn build_selected_context(
        &self,
        optimized_selection: Vec<ScoredPattern>,
        context: &AnalysisContext,
        performance_metrics: SelectionPerformanceMetrics,
    ) -> Result<SelectedContext, ContextSelectionError> {
        let selected_count = optimized_selection.len();
        let average_relevance = if selected_count > 0 {
            optimized_selection.iter()
                .map(|p| p.relevance_score)
                .sum::<f32>() / selected_count as f32
        } else {
            0.0
        };

        let used_tokens = optimized_selection.iter()
            .map(|p| self.estimate_pattern_tokens(&p.candidate.pattern))
            .sum::<usize>();

        let token_usage = TokenUsage {
            used_tokens,
            available_tokens: context.token_constraints.available_tokens,
            utilization_percent: (used_tokens as f32 / context.token_constraints.available_tokens as f32) * 100.0,
        };

        let language_context = if let Some(first_pattern) = optimized_selection.first() {
            first_pattern.candidate.pattern.language_contexts.get(&context.language).cloned()
        } else {
            None
        };

        let framework_guidance = if let Some(lang_context) = &language_context {
            lang_context.frameworks.clone()
        } else {
            vec![]
        };

        let metadata = ContextMetadata {
            total_candidates: context.detected_patterns.len(),
            selected_count,
            average_relevance,
            token_usage,
            strategy_used: self.config.selection_strategy.clone(),
            selection_time_ms: performance_metrics.total_time_ms,
        };

        Ok(SelectedContext {
            patterns: optimized_selection,
            language_context,
            framework_guidance,
            metadata,
            selection_metrics: performance_metrics,
        })
    }

    /// Update analytics based on selection results
    fn update_analytics(
        &mut self,
        selected_context: &SelectedContext,
        analysis_context: &AnalysisContext,
    ) {
        self.analytics.update_selection_analytics(selected_context, analysis_context);
    }
}

/// Adaptive learning system for context selection improvement
#[derive(Debug, Clone)]
pub struct ContextAnalytics {
    /// Usage frequency tracking
    usage_frequency: HashMap<String, UsageStats>,
    /// User feedback tracking
    feedback_history: Vec<FeedbackEntry>,
    /// Performance metrics
    performance_history: Vec<PerformanceEntry>,
    /// Learning configuration
    learning_config: LearningConfig,
}

/// Usage statistics for patterns
#[derive(Debug, Clone)]
pub struct UsageStats {
    /// Total times pattern was selected
    pub selection_count: usize,
    /// Total times pattern was used in successful analysis
    pub success_count: usize,
    /// Average user rating
    pub average_rating: f32,
    /// Last used timestamp
    pub last_used: chrono::DateTime<chrono::Utc>,
    /// Context success rate
    pub context_success_rate: f32,
}

/// User feedback entry
#[derive(Debug, Clone)]
pub struct FeedbackEntry {
    /// Pattern ID
    pub pattern_id: String,
    /// User rating (1-5)
    pub rating: u8,
    /// Context information
    pub context: String,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Performance tracking entry
#[derive(Debug, Clone)]
pub struct PerformanceEntry {
    /// Selection time
    pub selection_time_ms: u64,
    /// Number of patterns considered
    pub patterns_considered: usize,
    /// Number of patterns selected
    pub patterns_selected: usize,
    /// Token utilization
    pub token_utilization: f32,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Learning configuration
#[derive(Debug, Clone)]
pub struct LearningConfig {
    /// Enable usage frequency tracking
    pub track_usage_frequency: bool,
    /// Enable user feedback collection
    pub collect_user_feedback: bool,
    /// Learning rate for adaptive adjustments
    pub learning_rate: f32,
    /// Minimum data points before applying learning
    pub min_data_points: usize,
}

impl ContextAnalytics {
    /// Create new analytics system
    pub fn new() -> Self {
        Self {
            usage_frequency: HashMap::new(),
            feedback_history: Vec::new(),
            performance_history: Vec::new(),
            learning_config: LearningConfig {
                track_usage_frequency: true,
                collect_user_feedback: true,
                learning_rate: 0.1,
                min_data_points: 10,
            },
        }
    }

    /// Get usage-based score for adaptive learning
    pub fn get_usage_score(&self, pattern_id: &str) -> f32 {
        if let Some(stats) = self.usage_frequency.get(pattern_id) {
            let frequency_score = (stats.selection_count as f32).ln() / 10.0; // Logarithmic scaling
            let success_score = stats.context_success_rate;
            let rating_score = stats.average_rating / 5.0; // Normalize to 0-1

            // Weighted combination
            (frequency_score * 0.3 + success_score * 0.5 + rating_score * 0.2).min(1.0)
        } else {
            0.5 // Neutral score for new patterns
        }
    }

    /// Update analytics based on selection results
    pub fn update_selection_analytics(
        &mut self,
        selected_context: &SelectedContext,
        analysis_context: &AnalysisContext,
    ) {
        for scored_pattern in &selected_context.patterns {
            let pattern_id = &scored_pattern.candidate.pattern.universal.id;

            let stats = self.usage_frequency.entry(pattern_id.clone())
                .or_insert_with(|| UsageStats {
                    selection_count: 0,
                    success_count: 0,
                    average_rating: 3.0,
                    last_used: chrono::Utc::now(),
                    context_success_rate: 0.5,
                });

            stats.selection_count += 1;
            stats.last_used = chrono::Utc::now();
        }

        // Record performance metrics
        self.performance_history.push(PerformanceEntry {
            selection_time_ms: selected_context.selection_metrics.total_time_ms,
            patterns_considered: selected_context.metadata.total_candidates,
            patterns_selected: selected_context.metadata.selected_count,
            token_utilization: selected_context.metadata.token_usage.utilization_percent,
            timestamp: chrono::Utc::now(),
        });
    }

    /// Add user feedback for pattern
    pub fn add_feedback(&mut self, pattern_id: String, rating: u8, context: String) {
        self.feedback_history.push(FeedbackEntry {
            pattern_id: pattern_id.clone(),
            rating,
            context,
            timestamp: chrono::Utc::now(),
        });

        // Update average rating for the pattern
        if let Some(stats) = self.usage_frequency.get_mut(&pattern_id) {
            let feedback_count = self.feedback_history.iter()
                .filter(|f| f.pattern_id == pattern_id)
                .count() as f32;
            
            let total_rating: f32 = self.feedback_history.iter()
                .filter(|f| f.pattern_id == pattern_id)
                .map(|f| f.rating as f32)
                .sum();

            stats.average_rating = total_rating / feedback_count;
        }
    }
}

/// Performance metrics tracking for selection operations
#[derive(Debug)]
pub struct SelectionMetrics {
    /// Recent selection times
    recent_times: Vec<std::time::Duration>,
    /// Average selection time
    average_time_ms: f64,
    /// Total selections performed
    total_selections: usize,
}

impl SelectionMetrics {
    /// Create new metrics tracker
    pub fn new() -> Self {
        Self {
            recent_times: Vec::new(),
            average_time_ms: 0.0,
            total_selections: 0,
        }
    }

    /// Record a selection operation time
    pub fn record_selection_time(&mut self, duration: std::time::Duration) {
        self.recent_times.push(duration);
        self.total_selections += 1;

        // Keep only recent 100 measurements
        if self.recent_times.len() > 100 {
            self.recent_times.remove(0);
        }

        // Update average
        let total_ms: u128 = self.recent_times.iter()
            .map(|d| d.as_millis())
            .sum();
        self.average_time_ms = total_ms as f64 / self.recent_times.len() as f64;
    }

    /// Get average selection time in milliseconds
    pub fn get_average_time_ms(&self) -> f64 {
        self.average_time_ms
    }

    /// Get total number of selections
    pub fn get_total_selections(&self) -> usize {
        self.total_selections
    }
}

impl Default for ContextSelectionConfig {
    fn default() -> Self {
        Self {
            max_patterns: 10,
            token_budget: 4000,
            relevance_threshold: 0.3,
            scoring_weights: ScoringWeights {
                language_match: 0.25,
                framework_relevance: 0.15,
                pattern_similarity: 0.20,
                detection_confidence: 0.20,
                solution_applicability: 0.15,
                usage_frequency: 0.05,
            },
            enable_adaptive_learning: true,
            selection_strategy: SelectionStrategy::Diversified,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::knowledge::compression::CompressedString;

    fn create_test_context() -> AnalysisContext {
        AnalysisContext {
            language: SourceLanguage::Rust,
            frameworks: vec!["tokio".to_string(), "serde".to_string()],
            detected_patterns: vec![
                DetectedPattern {
                    pattern_id: "god_object".to_string(),
                    confidence: 0.9,
                    severity: SeverityLevel::High,
                    location: LocationContext {
                        file_path: "src/main.rs".to_string(),
                        line_range: (100, 200),
                        context_name: Some("MainService".to_string()),
                    },
                    related_patterns: vec!["tight_coupling".to_string()],
                }
            ],
            codebase_info: CodebaseInfo {
                size_category: CodebaseSizeCategory::Medium,
                architectural_patterns: vec!["MVC".to_string()],
                complexity_metrics: ComplexityMetrics {
                    cyclomatic_complexity: 8.5,
                    nesting_depth: 3.2,
                    coupling_score: 0.6,
                    tech_debt_ratio: 0.15,
                },
                team_experience: ExperienceLevel::Mid,
            },
            user_intent: UserIntent {
                goal: AnalysisGoal::DetailedAnalysis,
                explanation_depth: ExplanationDepth::Standard,
                focus_areas: vec![AntiPatternCategory::ObjectOriented],
                solution_preferences: vec![SolutionType::Comprehensive],
            },
            token_constraints: TokenConstraints {
                max_tokens: 4000,
                reserved_tokens: 1000,
                available_tokens: 3000,
                tokens_per_pattern: 200,
            },
        }
    }

    fn create_test_pattern() -> EnhancedPattern {
        let mut language_contexts = HashMap::new();
        language_contexts.insert(SourceLanguage::Rust, LanguageContext {
            language: SourceLanguage::Rust,
            specific_symptoms: vec![CompressedString::new("Large struct with many fields")],
            detection_methods: vec![],
            solutions: vec![],
            tools: vec![CompressedString::new("clippy")],
            frameworks: vec![],
            stdlib_guidance: None,
        });

        EnhancedPattern {
            universal: PatternKnowledge {
                id: "god_object".to_string(),
                name: "God Object".to_string(),
                definition: CompressedString::new("A class or struct that knows too much or does too much"),
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
                related_patterns: vec!["tight_coupling".to_string()],
                tags: vec!["oop".to_string(), "design".to_string()],
                frequency_score: 0.8,
                detection_confidence: 0.9,
            },
            language_contexts,
            metadata: crate::ai::knowledge::language_integration::PatternMetadata {
                complete_languages: vec![SourceLanguage::Rust],
                partial_languages: vec![],
                completeness_score: 1.0,
                language_confidence: [(SourceLanguage::Rust, 0.9)].into_iter().collect(),
            },
        }
    }

    #[test]
    fn test_context_selector_creation() {
        let mut patterns = HashMap::new();
        patterns.insert("god_object".to_string(), create_test_pattern());
        
        let config = ContextSelectionConfig::default();
        let selector = DynamicContextSelector::new(patterns, config);
        
        assert_eq!(selector.enhanced_patterns.len(), 1);
    }

    #[test]
    fn test_candidate_generation() {
        let mut patterns = HashMap::new();
        patterns.insert("god_object".to_string(), create_test_pattern());
        
        let config = ContextSelectionConfig::default();
        let selector = DynamicContextSelector::new(patterns, config);
        
        let context = create_test_context();
        let candidates = selector.generate_candidates(&context).unwrap();
        
        assert!(!candidates.is_empty());
        assert_eq!(candidates[0].pattern.universal.id, "god_object");
        assert!(matches!(candidates[0].match_type, MatchType::Direct));
    }

    #[test]
    fn test_language_score_calculation() {
        let mut patterns = HashMap::new();
        patterns.insert("god_object".to_string(), create_test_pattern());
        
        let config = ContextSelectionConfig::default();
        let selector = DynamicContextSelector::new(patterns, config);
        
        let context = create_test_context();
        let candidate = PatternCandidate {
            pattern: create_test_pattern(),
            match_type: MatchType::Direct,
            base_score: 0.9,
            detection_context: None,
        };
        
        let score = selector.calculate_language_score(&candidate, &context);
        assert_eq!(score, 1.0); // Exact match for Rust
    }

    #[test]
    fn test_pattern_scoring() {
        let mut patterns = HashMap::new();
        patterns.insert("god_object".to_string(), create_test_pattern());
        
        let config = ContextSelectionConfig::default();
        let selector = DynamicContextSelector::new(patterns, config);
        
        let context = create_test_context();
        let candidates = vec![PatternCandidate {
            pattern: create_test_pattern(),
            match_type: MatchType::Direct,
            base_score: 0.9,
            detection_context: None,
        }];
        
        let scored = selector.score_patterns(&candidates, &context).unwrap();
        assert!(!scored.is_empty());
        assert!(scored[0].relevance_score > 0.0);
    }

    #[test]
    fn test_token_budget_optimization() {
        let mut patterns = HashMap::new();
        patterns.insert("god_object".to_string(), create_test_pattern());
        
        let config = ContextSelectionConfig::default();
        let selector = DynamicContextSelector::new(patterns, config);
        
        let context = create_test_context();
        let scored = vec![ScoredPattern {
            candidate: PatternCandidate {
                pattern: create_test_pattern(),
                match_type: MatchType::Direct,
                base_score: 0.9,
                detection_context: None,
            },
            relevance_score: 0.8,
            score_breakdown: ScoreBreakdown {
                language_score: 1.0,
                framework_score: 0.5,
                similarity_score: 1.0,
                confidence_score: 0.9,
                solution_score: 0.6,
                usage_score: 0.5,
            },
        }];
        
        let optimized = selector.optimize_for_token_budget(&scored, &context).unwrap();
        assert!(!optimized.is_empty());
    }

    #[test]
    fn test_diversified_selection() {
        let mut patterns = HashMap::new();
        patterns.insert("god_object".to_string(), create_test_pattern());
        
        // Create another pattern with different category
        let mut arch_pattern = create_test_pattern();
        arch_pattern.universal.id = "tight_coupling".to_string();
        arch_pattern.universal.category = AntiPatternCategory::Architectural;
        patterns.insert("tight_coupling".to_string(), arch_pattern);
        
        let config = ContextSelectionConfig {
            selection_strategy: SelectionStrategy::Diversified,
            ..Default::default()
        };
        let selector = DynamicContextSelector::new(patterns, config);
        
        let context = create_test_context();
        let scored = vec![
            ScoredPattern {
                candidate: PatternCandidate {
                    pattern: create_test_pattern(),
                    match_type: MatchType::Direct,
                    base_score: 0.9,
                    detection_context: None,
                },
                relevance_score: 0.8,
                score_breakdown: ScoreBreakdown {
                    language_score: 1.0,
                    framework_score: 0.5,
                    similarity_score: 1.0,
                    confidence_score: 0.9,
                    solution_score: 0.6,
                    usage_score: 0.5,
                },
            }
        ];
        
        let selected = selector.apply_diversified_selection(&scored, &context).unwrap();
        assert!(!selected.is_empty());
    }

    #[test]
    fn test_analytics_usage_tracking() {
        let mut analytics = ContextAnalytics::new();
        
        // Test initial neutral score
        assert_eq!(analytics.get_usage_score("test_pattern"), 0.5);
        
        // Add feedback
        analytics.add_feedback("test_pattern".to_string(), 4, "Good detection".to_string());
        
        // Verify feedback was recorded
        assert_eq!(analytics.feedback_history.len(), 1);
    }
}