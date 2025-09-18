//! Refactoring suggestions for long methods

use crate::analysis::detectors::anti_patterns::long_methods::types::{
    LanguageThresholds, MethodMetrics,
};

/// Generates refactoring suggestions for long methods
pub struct RefactoringSuggester;

impl RefactoringSuggester {
    /// Generate comprehensive refactoring suggestions
    pub fn generate_suggestions(
        method: &MethodMetrics,
        thresholds: &LanguageThresholds,
    ) -> Vec<RefactoringSuggestion> {
        let mut suggestions = Vec::new();

        // Size-based suggestions
        if method.logical_loc > thresholds.max_logical_loc * 2 {
            suggestions.push(RefactoringSuggestion {
                category: SuggestionCategory::ExtractMethod,
                priority: Priority::High,
                description: format!(
                    "Method has {} logical lines (threshold: {}). Consider extracting sections into separate methods.",
                    method.logical_loc, thresholds.max_logical_loc
                ),
                technique: "Extract Method refactoring - identify logical sections and extract them into well-named methods".to_string(),
                estimated_effort: EstimatedEffort::Medium,
            });
        }

        // Complexity-based suggestions
        if method.cyclomatic_complexity > thresholds.max_cyclomatic_complexity * 2 {
            suggestions.push(RefactoringSuggestion {
                category: SuggestionCategory::ReduceComplexity,
                priority: Priority::High,
                description: format!(
                    "Cyclomatic complexity is {} (threshold: {}). Simplify conditional logic.",
                    method.cyclomatic_complexity, thresholds.max_cyclomatic_complexity
                ),
                technique: "Use polymorphism, strategy pattern, or guard clauses to reduce complexity".to_string(),
                estimated_effort: EstimatedEffort::High,
            });
        }

        // Nesting-based suggestions
        if method.max_nesting_depth > thresholds.max_nesting_depth + 2 {
            suggestions.push(RefactoringSuggestion {
                category: SuggestionCategory::FlattenNesting,
                priority: Priority::Medium,
                description: format!(
                    "Maximum nesting depth is {} (threshold: {}). Flatten nested conditions.",
                    method.max_nesting_depth, thresholds.max_nesting_depth
                ),
                technique: "Use guard clauses, early returns, or continue statements to reduce nesting".to_string(),
                estimated_effort: EstimatedEffort::Low,
            });
        }

        // Parameter-based suggestions
        if method.parameter_count > thresholds.max_parameters + 3 {
            suggestions.push(RefactoringSuggestion {
                category: SuggestionCategory::ParameterObject,
                priority: Priority::Medium,
                description: format!(
                    "Method has {} parameters (threshold: {}). Consider using a parameter object.",
                    method.parameter_count, thresholds.max_parameters
                ),
                technique: "Group related parameters into a struct/class or use a configuration object".to_string(),
                estimated_effort: EstimatedEffort::Medium,
            });
        }

        // Cognitive load suggestions
        if method.cognitive_complexity > thresholds.max_cognitive_complexity * 2 {
            suggestions.push(RefactoringSuggestion {
                category: SuggestionCategory::ReduceCognitiveLoad,
                priority: Priority::High,
                description: format!(
                    "Cognitive complexity is {} (threshold: {}). Method is difficult to understand.",
                    method.cognitive_complexity, thresholds.max_cognitive_complexity
                ),
                technique: "Break down complex logic, use descriptive variable names, and add comments".to_string(),
                estimated_effort: EstimatedEffort::High,
            });
        }

        // Specific pattern-based suggestions
        suggestions.extend(Self::generate_pattern_suggestions(method, thresholds));

        suggestions
    }

    /// Generate pattern-specific suggestions
    fn generate_pattern_suggestions(
        method: &MethodMetrics,
        thresholds: &LanguageThresholds,
    ) -> Vec<RefactoringSuggestion> {
        let mut suggestions = Vec::new();

        // Long parameter list pattern
        if method.parameter_count > 7 {
            suggestions.push(RefactoringSuggestion {
                category: SuggestionCategory::ParameterObject,
                priority: Priority::High,
                description: "Very long parameter list detected. This often indicates the method is doing too much.".to_string(),
                technique: "Consider the Parameter Object pattern or breaking the method into smaller, focused methods".to_string(),
                estimated_effort: EstimatedEffort::High,
            });
        }

        // High complexity with high nesting
        if method.cyclomatic_complexity > thresholds.max_cyclomatic_complexity
            && method.max_nesting_depth > thresholds.max_nesting_depth
        {
            suggestions.push(RefactoringSuggestion {
                category: SuggestionCategory::DecomposeConditional,
                priority: Priority::High,
                description: "Method has both high complexity and deep nesting. This suggests complex conditional logic.".to_string(),
                technique: "Use the Decompose Conditional pattern to extract conditions into well-named methods".to_string(),
                estimated_effort: EstimatedEffort::Medium,
            });
        }

        // Very long with moderate complexity
        if method.logical_loc > thresholds.max_logical_loc * 3
            && method.cyclomatic_complexity <= thresholds.max_cyclomatic_complexity
        {
            suggestions.push(RefactoringSuggestion {
                category: SuggestionCategory::ExtractMethod,
                priority: Priority::Medium,
                description: "Method is very long but not complex. Likely contains sequential operations.".to_string(),
                technique: "Extract sequential sections into well-named helper methods to improve readability".to_string(),
                estimated_effort: EstimatedEffort::Low,
            });
        }

        suggestions
    }

    /// Get prioritized suggestions
    pub fn get_prioritized_suggestions(
        method: &MethodMetrics,
        thresholds: &LanguageThresholds,
    ) -> Vec<RefactoringSuggestion> {
        let mut suggestions = Self::generate_suggestions(method, thresholds);

        // Sort by priority (High > Medium > Low)
        suggestions.sort_by(|a, b| a.priority.cmp(&b.priority));

        suggestions
    }

    /// Generate a refactoring plan
    pub fn generate_refactoring_plan(
        method: &MethodMetrics,
        thresholds: &LanguageThresholds,
    ) -> RefactoringPlan {
        let suggestions = Self::get_prioritized_suggestions(method, thresholds);

        let total_effort = suggestions.iter()
            .map(|s| s.estimated_effort.to_hours())
            .sum();

        let phases = Self::group_suggestions_into_phases(&suggestions);

        RefactoringPlan {
            method_name: method.name.clone(),
            total_estimated_hours: total_effort,
            phases,
            suggestions,
        }
    }

    /// Group suggestions into logical phases
    fn group_suggestions_into_phases(suggestions: &[RefactoringSuggestion]) -> Vec<RefactoringPhase> {
        let mut phases = Vec::new();

        // Phase 1: Quick wins (low effort, immediate improvement)
        let quick_wins: Vec<_> = suggestions.iter()
            .filter(|s| s.estimated_effort == EstimatedEffort::Low)
            .cloned()
            .collect();

        if !quick_wins.is_empty() {
            phases.push(RefactoringPhase {
                name: "Quick Wins".to_string(),
                description: "Low-effort improvements for immediate benefit".to_string(),
                suggestions: quick_wins,
            });
        }

        // Phase 2: Structure improvements (medium effort)
        let structure_improvements: Vec<_> = suggestions.iter()
            .filter(|s| s.estimated_effort == EstimatedEffort::Medium)
            .cloned()
            .collect();

        if !structure_improvements.is_empty() {
            phases.push(RefactoringPhase {
                name: "Structure Improvements".to_string(),
                description: "Medium-effort changes to improve method structure".to_string(),
                suggestions: structure_improvements,
            });
        }

        // Phase 3: Major refactoring (high effort)
        let major_refactoring: Vec<_> = suggestions.iter()
            .filter(|s| s.estimated_effort == EstimatedEffort::High)
            .cloned()
            .collect();

        if !major_refactoring.is_empty() {
            phases.push(RefactoringPhase {
                name: "Major Refactoring".to_string(),
                description: "High-effort changes requiring significant restructuring".to_string(),
                suggestions: major_refactoring,
            });
        }

        phases
    }
}

/// A refactoring suggestion
#[derive(Debug, Clone)]
pub struct RefactoringSuggestion {
    pub category: SuggestionCategory,
    pub priority: Priority,
    pub description: String,
    pub technique: String,
    pub estimated_effort: EstimatedEffort,
}

/// Categories of refactoring suggestions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SuggestionCategory {
    ExtractMethod,
    ReduceComplexity,
    FlattenNesting,
    ParameterObject,
    ReduceCognitiveLoad,
    DecomposeConditional,
}

/// Priority levels for suggestions
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low = 1,
    Medium = 2,
    High = 3,
}

/// Estimated effort for refactoring
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EstimatedEffort {
    Low,    // 1-2 hours
    Medium, // 3-8 hours
    High,   // 8+ hours
}

impl EstimatedEffort {
    fn to_hours(&self) -> u32 {
        match self {
            Self::Low => 2,
            Self::Medium => 5,
            Self::High => 12,
        }
    }
}

/// A complete refactoring plan
#[derive(Debug, Clone)]
pub struct RefactoringPlan {
    pub method_name: String,
    pub total_estimated_hours: u32,
    pub phases: Vec<RefactoringPhase>,
    pub suggestions: Vec<RefactoringSuggestion>,
}

/// A phase in the refactoring plan
#[derive(Debug, Clone)]
pub struct RefactoringPhase {
    pub name: String,
    pub description: String,
    pub suggestions: Vec<RefactoringSuggestion>,
}