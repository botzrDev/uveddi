//! Sanitization detection and taint level modification

use super::patterns::{get_rust_sanitizers, get_python_sanitizers, get_javascript_sanitizers};
use crate::analysis::detectors::security::taint_analysis::types::{
    TaintLevel, DataFlowNode, DataFlowNodeType, SanitizationPoint
};
use crate::ast::SourceLanguage;
use std::collections::HashMap;

/// Detector for sanitization points and their effectiveness
pub struct SanitizerDetector {
    sanitizers: HashMap<String, SanitizationPoint>,
    language_sanitizers: HashMap<SourceLanguage, Vec<SanitizationPoint>>,
}

impl SanitizerDetector {
    pub fn new() -> Self {
        let mut detector = Self {
            sanitizers: HashMap::new(),
            language_sanitizers: HashMap::new(),
        };
        detector.initialize_default_sanitizers();
        detector
    }

    /// Initialize default sanitizers for supported languages
    fn initialize_default_sanitizers(&mut self) {
        self.add_language_sanitizers(SourceLanguage::Rust, get_rust_sanitizers());
        self.add_language_sanitizers(SourceLanguage::Python, get_python_sanitizers());

        let js_sanitizers = get_javascript_sanitizers();
        self.add_language_sanitizers(SourceLanguage::JavaScript, js_sanitizers.clone());
        self.add_language_sanitizers(SourceLanguage::TypeScript, js_sanitizers);
    }

    /// Add sanitizers for a specific language
    fn add_language_sanitizers(&mut self, language: SourceLanguage, sanitizers: Vec<SanitizationPoint>) {
        for sanitizer in sanitizers {
            self.sanitizers.insert(sanitizer.id.clone(), sanitizer.clone());
            self.language_sanitizers
                .entry(language)
                .or_default()
                .push(sanitizer);
        }
    }

    /// Detect sanitization at a given node and return modified taint level
    pub fn detect_sanitization(
        &self,
        node: &DataFlowNode,
        current_taint: TaintLevel,
    ) -> TaintLevel {
        // Check if this node represents a sanitization call
        if let DataFlowNodeType::FunctionCall { function_name, .. } = &node.node_type {
            // Look for matching sanitizer patterns
            for sanitizer in self.sanitizers.values() {
                if function_name.contains(&sanitizer.pattern) {
                    // Apply sanitization based on effectiveness
                    return self.apply_sanitization(&sanitizer, current_taint);
                }
            }
        }

        current_taint
    }

    /// Apply sanitization based on sanitizer effectiveness
    fn apply_sanitization(
        &self,
        sanitizer: &SanitizationPoint,
        current_taint: TaintLevel,
    ) -> TaintLevel {
        match current_taint {
            TaintLevel::Untainted => TaintLevel::Untainted,
            TaintLevel::Sanitized => TaintLevel::Sanitized,
            TaintLevel::Low => {
                if sanitizer.effectiveness > 0.8 {
                    TaintLevel::Sanitized
                } else {
                    TaintLevel::Low
                }
            }
            TaintLevel::Medium => {
                if sanitizer.effectiveness > 0.9 {
                    TaintLevel::Sanitized
                } else if sanitizer.effectiveness > 0.6 {
                    TaintLevel::Low
                } else {
                    TaintLevel::Medium
                }
            }
            TaintLevel::High => {
                if sanitizer.effectiveness > 0.95 {
                    TaintLevel::Sanitized
                } else if sanitizer.effectiveness > 0.8 {
                    TaintLevel::Medium
                } else if sanitizer.effectiveness > 0.6 {
                    TaintLevel::Low
                } else {
                    TaintLevel::High
                }
            }
        }
    }

    /// Check if a function call pattern matches any known sanitizers
    pub fn is_sanitizer_pattern(&self, pattern: &str) -> bool {
        self.sanitizers.values().any(|s| pattern.contains(&s.pattern))
    }

    /// Get sanitizer by ID
    pub fn get_sanitizer(&self, id: &str) -> Option<&SanitizationPoint> {
        self.sanitizers.get(id)
    }

    /// Get all sanitizers
    pub fn get_all_sanitizers(&self) -> Vec<&SanitizationPoint> {
        self.sanitizers.values().collect()
    }

    /// Add custom sanitizer
    pub fn add_custom_sanitizer(&mut self, sanitizer: SanitizationPoint) {
        self.sanitizers.insert(sanitizer.id.clone(), sanitizer);
    }

    /// Get all sanitizers for a language
    pub fn get_language_sanitizers(&self, language: SourceLanguage) -> Vec<&SanitizationPoint> {
        self.language_sanitizers
            .get(&language)
            .map(|sanitizers| sanitizers.iter().collect())
            .unwrap_or_default()
    }
}

impl Default for SanitizerDetector {
    fn default() -> Self {
        Self::new()
    }
}