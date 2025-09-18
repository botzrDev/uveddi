//! Threshold management for clone detection

use crate::analysis::detectors::anti_patterns::code_duplication::types::CloneType;
use crate::ast::tree_sitter_impl::SourceLanguage;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Manages thresholds for different types of clones and languages
pub struct ThresholdManager {
    /// Language-specific thresholds
    language_thresholds: HashMap<SourceLanguage, LanguageThresholds>,
    /// Default thresholds
    default_thresholds: LanguageThresholds,
    /// Adaptive threshold settings
    adaptive_settings: AdaptiveSettings,
}

/// Thresholds for a specific language
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageThresholds {
    /// Minimum similarity for Type-1 clones (exact)
    pub type1_threshold: f64,
    /// Minimum similarity for Type-2 clones (renamed)
    pub type2_threshold: f64,
    /// Minimum similarity for Type-3 clones (near-miss)
    pub type3_threshold: f64,
    /// Minimum similarity for Type-4 clones (semantic)
    pub type4_threshold: f64,
    /// Minimum token count for consideration
    pub min_tokens: usize,
    /// Minimum line count for consideration
    pub min_lines: usize,
    /// Maximum acceptable false positive rate
    pub max_false_positive_rate: f64,
}

/// Settings for adaptive threshold adjustment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptiveSettings {
    /// Whether to enable adaptive thresholds
    pub enabled: bool,
    /// Learning rate for threshold adjustment
    pub learning_rate: f64,
    /// Minimum threshold values (safety bounds)
    pub min_thresholds: LanguageThresholds,
    /// Maximum threshold values (safety bounds)
    pub max_thresholds: LanguageThresholds,
    /// Number of samples needed for adaptation
    pub min_samples: usize,
}

impl ThresholdManager {
    /// Creates a new threshold manager with default settings
    pub fn new() -> Self {
        let default_thresholds = LanguageThresholds::default();
        let mut language_thresholds = HashMap::new();

        // Language-specific tuning
        language_thresholds.insert(SourceLanguage::Rust, LanguageThresholds {
            type1_threshold: 0.95,
            type2_threshold: 0.85,
            type3_threshold: 0.75,
            type4_threshold: 0.65,
            min_tokens: 50,
            min_lines: 8,
            max_false_positive_rate: 0.05,
        });

        language_thresholds.insert(SourceLanguage::Python, LanguageThresholds {
            type1_threshold: 0.92,
            type2_threshold: 0.80,
            type3_threshold: 0.70,
            type4_threshold: 0.60,
            min_tokens: 40,
            min_lines: 6,
            max_false_positive_rate: 0.08,
        });

        language_thresholds.insert(SourceLanguage::JavaScript, LanguageThresholds {
            type1_threshold: 0.90,
            type2_threshold: 0.78,
            type3_threshold: 0.68,
            type4_threshold: 0.58,
            min_tokens: 35,
            min_lines: 5,
            max_false_positive_rate: 0.10,
        });

        language_thresholds.insert(SourceLanguage::TypeScript, LanguageThresholds {
            type1_threshold: 0.93,
            type2_threshold: 0.82,
            type3_threshold: 0.72,
            type4_threshold: 0.62,
            min_tokens: 45,
            min_lines: 7,
            max_false_positive_rate: 0.06,
        });

        Self {
            language_thresholds,
            default_thresholds,
            adaptive_settings: AdaptiveSettings::default(),
        }
    }

    /// Gets the threshold for a specific clone type and language
    pub fn get_threshold(&self, clone_type: &CloneType, language: &SourceLanguage) -> f64 {
        let thresholds = self.language_thresholds
            .get(language)
            .unwrap_or(&self.default_thresholds);

        match clone_type {
            CloneType::Type1 => thresholds.type1_threshold,
            CloneType::Type2 => thresholds.type2_threshold,
            CloneType::Type3 => thresholds.type3_threshold,
            CloneType::Type4 => thresholds.type4_threshold,
        }
    }

    /// Gets the minimum token count for a language
    pub fn get_min_tokens(&self, language: &SourceLanguage) -> usize {
        self.language_thresholds
            .get(language)
            .map(|t| t.min_tokens)
            .unwrap_or(self.default_thresholds.min_tokens)
    }

    /// Gets the minimum line count for a language
    pub fn get_min_lines(&self, language: &SourceLanguage) -> usize {
        self.language_thresholds
            .get(language)
            .map(|t| t.min_lines)
            .unwrap_or(self.default_thresholds.min_lines)
    }

    /// Updates threshold for a specific clone type and language
    pub fn update_threshold(&mut self, clone_type: &CloneType, language: &SourceLanguage, new_threshold: f64) {
        let thresholds = self.language_thresholds
            .entry(*language)
            .or_insert_with(|| self.default_thresholds.clone());

        match clone_type {
            CloneType::Type1 => thresholds.type1_threshold = new_threshold,
            CloneType::Type2 => thresholds.type2_threshold = new_threshold,
            CloneType::Type3 => thresholds.type3_threshold = new_threshold,
            CloneType::Type4 => thresholds.type4_threshold = new_threshold,
        }
    }

    /// Adjusts thresholds based on false positive feedback
    pub fn adjust_for_false_positives(&mut self, language: &SourceLanguage, clone_type: &CloneType, false_positive_rate: f64) {
        if !self.adaptive_settings.enabled {
            return;
        }

        let current_threshold = self.get_threshold(clone_type, language);
        let max_fp_rate = self.language_thresholds
            .get(language)
            .map(|t| t.max_false_positive_rate)
            .unwrap_or(self.default_thresholds.max_false_positive_rate);

        if false_positive_rate > max_fp_rate {
            // Increase threshold to reduce false positives
            let adjustment = self.adaptive_settings.learning_rate * (false_positive_rate - max_fp_rate);
            let new_threshold = (current_threshold + adjustment).min(1.0);

            // Apply safety bounds
            let bounded_threshold = self.apply_threshold_bounds(new_threshold, clone_type);
            self.update_threshold(clone_type, language, bounded_threshold);
        }
    }

    /// Adjusts thresholds based on false negative feedback
    pub fn adjust_for_false_negatives(&mut self, language: &SourceLanguage, clone_type: &CloneType, false_negative_rate: f64) {
        if !self.adaptive_settings.enabled {
            return;
        }

        let current_threshold = self.get_threshold(clone_type, language);

        if false_negative_rate > 0.1 { // Threshold for concerning false negative rate
            // Decrease threshold to reduce false negatives
            let adjustment = self.adaptive_settings.learning_rate * false_negative_rate;
            let new_threshold = (current_threshold - adjustment).max(0.0);

            // Apply safety bounds
            let bounded_threshold = self.apply_threshold_bounds(new_threshold, clone_type);
            self.update_threshold(clone_type, language, bounded_threshold);
        }
    }

    /// Applies safety bounds to threshold values
    fn apply_threshold_bounds(&self, threshold: f64, clone_type: &CloneType) -> f64 {
        let min_bound = match clone_type {
            CloneType::Type1 => self.adaptive_settings.min_thresholds.type1_threshold,
            CloneType::Type2 => self.adaptive_settings.min_thresholds.type2_threshold,
            CloneType::Type3 => self.adaptive_settings.min_thresholds.type3_threshold,
            CloneType::Type4 => self.adaptive_settings.min_thresholds.type4_threshold,
        };

        let max_bound = match clone_type {
            CloneType::Type1 => self.adaptive_settings.max_thresholds.type1_threshold,
            CloneType::Type2 => self.adaptive_settings.max_thresholds.type2_threshold,
            CloneType::Type3 => self.adaptive_settings.max_thresholds.type3_threshold,
            CloneType::Type4 => self.adaptive_settings.max_thresholds.type4_threshold,
        };

        threshold.max(min_bound).min(max_bound)
    }

    /// Gets recommended threshold based on codebase characteristics
    pub fn get_recommended_threshold(&self, language: &SourceLanguage, codebase_size: usize, diversity_score: f64) -> LanguageThresholds {
        let base_thresholds = self.language_thresholds
            .get(language)
            .unwrap_or(&self.default_thresholds)
            .clone();

        // Adjust based on codebase size
        let size_factor = if codebase_size > 100_000 {
            1.05 // Slightly higher thresholds for large codebases
        } else if codebase_size < 10_000 {
            0.95 // Slightly lower thresholds for small codebases
        } else {
            1.0
        };

        // Adjust based on diversity (high diversity = lower risk of false positives)
        let diversity_factor = if diversity_score > 0.8 {
            0.98 // Can afford slightly lower thresholds
        } else if diversity_score < 0.3 {
            1.02 // Need higher thresholds to avoid false positives
        } else {
            1.0
        };

        let adjustment_factor = size_factor * diversity_factor;

        LanguageThresholds {
            type1_threshold: (base_thresholds.type1_threshold * adjustment_factor).min(1.0),
            type2_threshold: (base_thresholds.type2_threshold * adjustment_factor).min(1.0),
            type3_threshold: (base_thresholds.type3_threshold * adjustment_factor).min(1.0),
            type4_threshold: (base_thresholds.type4_threshold * adjustment_factor).min(1.0),
            ..base_thresholds
        }
    }

    /// Enables or disables adaptive thresholds
    pub fn set_adaptive_enabled(&mut self, enabled: bool) {
        self.adaptive_settings.enabled = enabled;
    }

    /// Sets the learning rate for adaptive adjustments
    pub fn set_learning_rate(&mut self, rate: f64) {
        self.adaptive_settings.learning_rate = rate.max(0.0).min(1.0);
    }
}

impl Default for LanguageThresholds {
    fn default() -> Self {
        Self {
            type1_threshold: 0.90,
            type2_threshold: 0.80,
            type3_threshold: 0.70,
            type4_threshold: 0.60,
            min_tokens: 40,
            min_lines: 6,
            max_false_positive_rate: 0.08,
        }
    }
}

impl Default for AdaptiveSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            learning_rate: 0.1,
            min_thresholds: LanguageThresholds {
                type1_threshold: 0.7,
                type2_threshold: 0.6,
                type3_threshold: 0.5,
                type4_threshold: 0.4,
                min_tokens: 20,
                min_lines: 3,
                max_false_positive_rate: 0.15,
            },
            max_thresholds: LanguageThresholds {
                type1_threshold: 0.99,
                type2_threshold: 0.95,
                type3_threshold: 0.90,
                type4_threshold: 0.85,
                min_tokens: 100,
                min_lines: 15,
                max_false_positive_rate: 0.02,
            },
            min_samples: 10,
        }
    }
}

impl Default for ThresholdManager {
    fn default() -> Self {
        Self::new()
    }
}