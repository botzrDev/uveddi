//! Type definitions for long methods detection

use crate::analysis::detectors::base::Severity;
use serde::{Deserialize, Serialize};

/// Represents metrics collected for a method/function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodMetrics {
    /// Name of the method/function
    pub name: String,
    /// File path where the method is defined
    pub file_path: String,
    /// Starting line number
    pub start_line: u32,
    /// Ending line number
    pub end_line: u32,
    /// Logical Lines of Code (excluding comments and blank lines)
    pub logical_loc: u32,
    /// Number of statements
    pub statement_count: u32,
    /// Number of parameters
    pub parameter_count: u32,
    /// Maximum nesting depth
    pub max_nesting_depth: u32,
    /// Cyclomatic complexity
    pub cyclomatic_complexity: u32,
    /// Cognitive complexity
    pub cognitive_complexity: u32,
    /// Whether the method is exported/public
    pub is_exported: bool,
    /// Code snippet of the method definition
    pub code_snippet: String,
    /// Method type (function, method, async, etc.)
    pub method_type: String,
}

impl MethodMetrics {
    /// Calculate a composite score for the method length
    pub fn calculate_score(&self, thresholds: &LanguageThresholds) -> u32 {
        let mut score = 0u32;

        // Size metrics (40% weight)
        if self.logical_loc > thresholds.max_logical_loc {
            score += ((self.logical_loc - thresholds.max_logical_loc) * 2).min(40);
        }
        if self.statement_count > thresholds.max_statements {
            score += ((self.statement_count - thresholds.max_statements) * 1).min(20);
        }

        // Complexity metrics (40% weight)
        if self.cyclomatic_complexity > thresholds.max_cyclomatic_complexity {
            score += ((self.cyclomatic_complexity - thresholds.max_cyclomatic_complexity) * 3).min(30);
        }
        if self.cognitive_complexity > thresholds.max_cognitive_complexity {
            score += ((self.cognitive_complexity - thresholds.max_cognitive_complexity) * 2).min(20);
        }

        // Structural metrics (20% weight)
        if self.max_nesting_depth > thresholds.max_nesting_depth {
            score += ((self.max_nesting_depth - thresholds.max_nesting_depth) * 5).min(10);
        }
        if self.parameter_count > thresholds.max_parameters {
            score += ((self.parameter_count - thresholds.max_parameters) * 3).min(10);
        }

        score.min(100)
    }

    /// Determine severity based on score
    pub fn get_severity(&self, thresholds: &LanguageThresholds) -> Severity {
        let score = self.calculate_score(thresholds);
        match score {
            0..=25 => Severity::Info,
            26..=50 => Severity::Low,
            51..=75 => Severity::Medium,
            76..=90 => Severity::High,
            _ => Severity::Critical,
        }
    }

    /// Generate refactoring suggestions based on metrics
    pub fn get_refactoring_suggestions(&self, thresholds: &LanguageThresholds) -> Vec<String> {
        let mut suggestions = Vec::new();

        if self.logical_loc > thresholds.max_logical_loc * 2 {
            suggestions.push(
                "Extract logical sections into separate functions using the Extract Function refactoring"
                    .to_string(),
            );
        }

        if self.cyclomatic_complexity > thresholds.max_cyclomatic_complexity * 2 {
            suggestions.push(
                "Reduce complexity by simplifying conditional logic or using polymorphism"
                    .to_string(),
            );
        }

        if self.max_nesting_depth > thresholds.max_nesting_depth + 2 {
            suggestions.push(
                "Flatten nested conditions using guard clauses or early returns".to_string(),
            );
        }

        if self.parameter_count > thresholds.max_parameters + 3 {
            suggestions.push(
                "Consider using a parameter object to group related parameters".to_string(),
            );
        }

        suggestions
    }
}

/// Language-specific thresholds for long method detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageThresholds {
    /// Maximum logical lines of code
    pub max_logical_loc: u32,
    /// Maximum number of statements
    pub max_statements: u32,
    /// Maximum number of parameters
    pub max_parameters: u32,
    /// Maximum nesting depth
    pub max_nesting_depth: u32,
    /// Maximum cyclomatic complexity
    pub max_cyclomatic_complexity: u32,
    /// Maximum cognitive complexity
    pub max_cognitive_complexity: u32,
}

impl Default for LanguageThresholds {
    fn default() -> Self {
        Self {
            max_logical_loc: 50,
            max_statements: 40,
            max_parameters: 5,
            max_nesting_depth: 4,
            max_cyclomatic_complexity: 10,
            max_cognitive_complexity: 15,
        }
    }
}

/// Detection result containing all found long methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LongMethodsResult {
    /// All detected long methods
    pub methods: Vec<MethodMetrics>,
    /// Issues found during analysis
    pub issues: Vec<crate::analysis::detectors::base::Issue>,
    /// Total number of methods analyzed
    pub total_methods_analyzed: usize,
    /// Number of methods exceeding thresholds
    pub methods_exceeding_thresholds: usize,
}

/// Method type classifications
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MethodType {
    Function,
    Method,
    AsyncFunction,
    AsyncMethod,
    Constructor,
    Destructor,
    Getter,
    Setter,
    Static,
    Lambda,
    Closure,
}

impl std::fmt::Display for MethodType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Function => write!(f, "function"),
            Self::Method => write!(f, "method"),
            Self::AsyncFunction => write!(f, "async_function"),
            Self::AsyncMethod => write!(f, "async_method"),
            Self::Constructor => write!(f, "constructor"),
            Self::Destructor => write!(f, "destructor"),
            Self::Getter => write!(f, "getter"),
            Self::Setter => write!(f, "setter"),
            Self::Static => write!(f, "static"),
            Self::Lambda => write!(f, "lambda"),
            Self::Closure => write!(f, "closure"),
        }
    }
}