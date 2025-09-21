//! Abstraction quality scoring for leaky abstraction detection.

use super::{BoundaryIntegrityScore, EncapsulationScore};
use crate::analysis::detectors::anti_patterns::leaky_abstraction::types::{
    AnalysisContext, ArchitecturalConfig,
};
use crate::analysis::AnalysisError;
use crate::ast::tree_sitter_impl::ParsedFile;

/// Provides comprehensive abstraction quality scoring.
pub struct AbstractionScorer {
    config: ArchitecturalConfig,
}

impl AbstractionScorer {
    /// Creates a new abstraction scorer.
    pub fn new(config: ArchitecturalConfig) -> Self {
        Self { config }
    }

    /// Calculates a comprehensive abstraction quality score.
    pub fn calculate_abstraction_score(
        &self,
        parsed_file: &ParsedFile,
        context: &AnalysisContext,
    ) -> Result<AbstractionQualityScore, AnalysisError> {
        let mut score = AbstractionQualityScore::new();

        // Calculate component scores
        score.encapsulation_score = self.calculate_encapsulation_score(parsed_file, context)?;
        score.boundary_score = self.calculate_boundary_score(parsed_file, context)?;
        score.interface_score = self.calculate_interface_score(parsed_file, context)?;
        score.dependency_score = self.calculate_dependency_score(parsed_file, context)?;

        // Calculate overall score as weighted average
        score.overall_score = self.calculate_weighted_score(&score);

        Ok(score)
    }

    /// Calculates encapsulation quality score.
    fn calculate_encapsulation_score(
        &self,
        _parsed_file: &ParsedFile,
        _context: &AnalysisContext,
    ) -> Result<f64, AnalysisError> {
        // This would analyze encapsulation patterns
        // For now, returning a placeholder score
        Ok(0.75)
    }

    /// Calculates boundary integrity score.
    fn calculate_boundary_score(
        &self,
        _parsed_file: &ParsedFile,
        _context: &AnalysisContext,
    ) -> Result<f64, AnalysisError> {
        // This would analyze boundary violations
        // For now, returning a placeholder score
        Ok(0.80)
    }

    /// Calculates interface design score.
    fn calculate_interface_score(
        &self,
        _parsed_file: &ParsedFile,
        _context: &AnalysisContext,
    ) -> Result<f64, AnalysisError> {
        // This would analyze interface quality
        // For now, returning a placeholder score
        Ok(0.65)
    }

    /// Calculates dependency management score.
    fn calculate_dependency_score(
        &self,
        _parsed_file: &ParsedFile,
        _context: &AnalysisContext,
    ) -> Result<f64, AnalysisError> {
        // This would analyze dependency patterns
        // For now, returning a placeholder score
        Ok(0.70)
    }

    /// Calculates weighted overall score.
    fn calculate_weighted_score(&self, score: &AbstractionQualityScore) -> f64 {
        let weights = ScoreWeights::default();

        (score.encapsulation_score * weights.encapsulation_weight
            + score.boundary_score * weights.boundary_weight
            + score.interface_score * weights.interface_weight
            + score.dependency_score * weights.dependency_weight)
            / (weights.encapsulation_weight
                + weights.boundary_weight
                + weights.interface_weight
                + weights.dependency_weight)
    }

    /// Analyzes abstraction patterns for scoring.
    pub fn analyze_abstraction_patterns(&self, code: &str) -> AbstractionPatternAnalysis {
        let mut analysis = AbstractionPatternAnalysis::new();

        // Analyze various patterns
        analysis.visibility_patterns = self.analyze_visibility_patterns(code);
        analysis.interface_patterns = self.analyze_interface_patterns(code);
        analysis.dependency_patterns = self.analyze_dependency_patterns(code);
        analysis.encapsulation_patterns = self.analyze_encapsulation_patterns(code);

        analysis
    }

    /// Analyzes visibility patterns in code.
    fn analyze_visibility_patterns(&self, code: &str) -> Vec<String> {
        let mut patterns = Vec::new();

        if code.contains("pub ") {
            patterns.push("Public visibility used".to_string());
        }
        if code.contains("private ") {
            patterns.push("Private visibility used".to_string());
        }
        if code.contains("_") {
            patterns.push("Underscore naming convention".to_string());
        }

        patterns
    }

    /// Analyzes interface patterns in code.
    fn analyze_interface_patterns(&self, code: &str) -> Vec<String> {
        let mut patterns = Vec::new();

        if code.contains("trait ") {
            patterns.push("Trait definition found".to_string());
        }
        if code.contains("interface ") {
            patterns.push("Interface definition found".to_string());
        }
        if code.contains("impl ") {
            patterns.push("Implementation block found".to_string());
        }

        patterns
    }

    /// Analyzes dependency patterns in code.
    fn analyze_dependency_patterns(&self, code: &str) -> Vec<String> {
        let mut patterns = Vec::new();

        if code.contains("use ") {
            patterns.push("External dependency used".to_string());
        }
        if code.contains("import ") {
            patterns.push("Import statement found".to_string());
        }
        if code.contains("require(") {
            patterns.push("Require statement found".to_string());
        }

        patterns
    }

    /// Analyzes encapsulation patterns in code.
    fn analyze_encapsulation_patterns(&self, code: &str) -> Vec<String> {
        let mut patterns = Vec::new();

        if code.contains("struct ") {
            patterns.push("Struct definition found".to_string());
        }
        if code.contains("class ") {
            patterns.push("Class definition found".to_string());
        }
        if code.contains("module ") {
            patterns.push("Module definition found".to_string());
        }

        patterns
    }

    /// Provides recommendations based on score analysis.
    pub fn provide_recommendations(&self, score: &AbstractionQualityScore) -> Vec<String> {
        let mut recommendations = Vec::new();

        if score.encapsulation_score < 0.6 {
            recommendations.push(
                "Consider improving encapsulation by using private fields and getter methods"
                    .to_string(),
            );
        }

        if score.boundary_score < 0.6 {
            recommendations.push(
                "Review architectural boundaries and reduce cross-layer dependencies".to_string(),
            );
        }

        if score.interface_score < 0.6 {
            recommendations.push(
                "Improve interface design by using abstractions instead of concrete types"
                    .to_string(),
            );
        }

        if score.dependency_score < 0.6 {
            recommendations.push(
                "Reduce coupling by using dependency injection and interface segregation"
                    .to_string(),
            );
        }

        if score.overall_score < 0.5 {
            recommendations.push(
                "Consider a major refactoring to improve overall abstraction quality".to_string(),
            );
        }

        recommendations
    }
}

/// Represents comprehensive abstraction quality metrics.
#[derive(Debug, Clone)]
pub struct AbstractionQualityScore {
    /// Overall abstraction quality score (0.0 to 1.0)
    pub overall_score: f64,

    /// Encapsulation quality score
    pub encapsulation_score: f64,

    /// Boundary integrity score
    pub boundary_score: f64,

    /// Interface design score
    pub interface_score: f64,

    /// Dependency management score
    pub dependency_score: f64,

    /// Detailed analysis results
    pub details: Vec<String>,
}

impl AbstractionQualityScore {
    /// Creates a new abstraction quality score.
    pub fn new() -> Self {
        Self {
            overall_score: 0.0,
            encapsulation_score: 0.0,
            boundary_score: 0.0,
            interface_score: 0.0,
            dependency_score: 0.0,
            details: Vec::new(),
        }
    }

    /// Gets a descriptive quality level.
    pub fn quality_level(&self) -> &'static str {
        match self.overall_score {
            s if s >= 0.8 => "Excellent",
            s if s >= 0.6 => "Good",
            s if s >= 0.4 => "Fair",
            s if s >= 0.2 => "Poor",
            _ => "Critical",
        }
    }

    /// Adds a detailed analysis note.
    pub fn add_detail(&mut self, detail: String) {
        self.details.push(detail);
    }
}

/// Weight configuration for score calculation.
#[derive(Debug, Clone)]
pub struct ScoreWeights {
    pub encapsulation_weight: f64,
    pub boundary_weight: f64,
    pub interface_weight: f64,
    pub dependency_weight: f64,
}

impl Default for ScoreWeights {
    fn default() -> Self {
        Self {
            encapsulation_weight: 0.3,
            boundary_weight: 0.3,
            interface_weight: 0.2,
            dependency_weight: 0.2,
        }
    }
}

/// Analysis of abstraction patterns in code.
#[derive(Debug, Clone)]
pub struct AbstractionPatternAnalysis {
    pub visibility_patterns: Vec<String>,
    pub interface_patterns: Vec<String>,
    pub dependency_patterns: Vec<String>,
    pub encapsulation_patterns: Vec<String>,
}

impl AbstractionPatternAnalysis {
    /// Creates a new pattern analysis.
    pub fn new() -> Self {
        Self {
            visibility_patterns: Vec::new(),
            interface_patterns: Vec::new(),
            dependency_patterns: Vec::new(),
            encapsulation_patterns: Vec::new(),
        }
    }

    /// Gets the total number of patterns detected.
    pub fn total_patterns(&self) -> usize {
        self.visibility_patterns.len()
            + self.interface_patterns.len()
            + self.dependency_patterns.len()
            + self.encapsulation_patterns.len()
    }
}

impl Clone for AbstractionScorer {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
        }
    }
}

impl Default for AbstractionQualityScore {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for AbstractionPatternAnalysis {
    fn default() -> Self {
        Self::new()
    }
}
