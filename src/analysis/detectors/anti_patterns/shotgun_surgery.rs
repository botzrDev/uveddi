//! Shotgun Surgery Anti-Pattern Detector
//!
//! This module detects the "Shotgun Surgery" anti-pattern, which occurs when
//! making a single change requires modifications across many different classes
//! or modules. This indicates poor separation of concerns and high coupling.

use crate::analysis::{AnalysisDetector, AnalysisError};
use crate::ast::{ParsedFile, SourceLanguage};
use crate::database::models::{AntiPatternType, ArchitecturalIssue};
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Configuration for Shotgun Surgery detection
#[derive(Debug, Clone)]
pub struct ShotgunSurgeryConfig {
    /// Minimum number of related changes to constitute shotgun surgery
    pub min_related_changes: usize,
    /// Threshold for change correlation (0.0 to 1.0)
    pub correlation_threshold: f64,
    /// Maximum allowed coupled classes per feature
    pub max_coupled_classes: usize,
}

impl Default for ShotgunSurgeryConfig {
    fn default() -> Self {
        Self {
            min_related_changes: 4,
            correlation_threshold: 0.7,
            max_coupled_classes: 6,
        }
    }
}

/// Represents a change pattern that might indicate shotgun surgery
#[derive(Debug, Clone)]
struct ChangePattern {
    /// Files that are frequently changed together
    affected_files: HashSet<String>,
    /// The common functionality that requires cross-cutting changes
    functionality_area: String,
    /// Confidence score for this being shotgun surgery
    confidence: f64,
}

/// Detector for Shotgun Surgery anti-pattern
pub struct ShotgunSurgeryDetector {
    config: ShotgunSurgeryConfig,
}

impl ShotgunSurgeryDetector {
    /// Create a new Shotgun Surgery detector with default configuration
    pub fn new() -> Self {
        Self {
            config: ShotgunSurgeryConfig::default(),
        }
    }

    /// Create a new detector with custom configuration
    pub fn with_config(config: ShotgunSurgeryConfig) -> Self {
        Self { config }
    }

    /// Analyze method coupling patterns to detect shotgun surgery
    fn analyze_method_coupling(&self, ast_provider: &dyn AstProvider, file_path: &Path) -> Vec<ChangePattern> {
        let mut patterns = Vec::new();
        
        // Get all function calls and their external dependencies
        if let Ok(functions) = ast_provider.get_functions(file_path) {
            for function in functions {
                let external_calls = self.extract_external_calls(&function.name, ast_provider, file_path);
                
                if external_calls.len() >= self.config.min_related_changes {
                    let pattern = ChangePattern {
                        affected_files: external_calls.into_iter().collect::<std::collections::HashSet<String>>(), // Explicitly collect into set for uniqueness
                        functionality_area: function.name.clone(),
                        confidence: self.calculate_shotgun_confidence(&function.name, file_path),
                    };
                    patterns.push(pattern);
                }
            }
        }
        
        patterns
    }

    /// Extract external method calls that might indicate cross-cutting concerns
    fn extract_external_calls(&self, function_name: &str, ast_provider: &dyn AstProvider, file_path: &Path) -> Vec<String> {
        let mut external_calls = Vec::new();
        
        // This is a simplified implementation - in a real detector, we would:
        // 1. Parse the AST to find all method calls
        // 2. Determine which calls are to external modules/classes
        // 3. Track import statements and their usage
        // 4. Identify cross-module dependencies
        
        // For demonstration, we'll simulate finding external dependencies
        if function_name.contains("save") || function_name.contains("update") {
            external_calls.extend([
                "database_module.rs".to_string(),
                "validation_module.rs".to_string(),
                "logging_module.rs".to_string(),
                "notification_module.rs".to_string(),
            ]);
        }
        
        if function_name.contains("process") || function_name.contains("handle") {
            external_calls.extend([
                "parser_module.rs".to_string(),
                "converter_module.rs".to_string(),
                "validator_module.rs".to_string(),
                "formatter_module.rs".to_string(),
            ]);
        }
        
        external_calls
    }

    /// Calculate confidence that this represents shotgun surgery
    fn calculate_shotgun_confidence(&self, function_name: &str, file_path: &Path) -> f64 {
        let mut confidence = 0.5; // Base confidence
        
        // Higher confidence for certain naming patterns
        if function_name.contains("update") || function_name.contains("modify") {
            confidence += 0.2;
        }
        
        if function_name.contains("save") || function_name.contains("persist") {
            confidence += 0.1;
        }
        
        // Consider file location - domain-specific files are more likely to have shotgun surgery
        if let Some(filename) = file_path.file_name().and_then(|n| n.to_str()) {
            if filename.contains("service") || filename.contains("manager") {
                confidence += 0.1;
            }
        }
        
        confidence.min(1.0)
    }

    /// Analyze class relationships to identify shotgun surgery patterns
    fn analyze_class_relationships(&self, ast_provider: &dyn AstProvider, file_path: &Path) -> Vec<ChangePattern> {
        let mut patterns = Vec::new();
        
        // Get class information and analyze their dependencies
        if let Ok(classes) = ast_provider.get_classes(file_path) {
            for class in classes {
                let dependencies = self.analyze_class_dependencies(&class.name, ast_provider, file_path);
                
                if dependencies.len() > self.config.max_coupled_classes {
                    let pattern = ChangePattern {
                        affected_files: dependencies.into_iter().collect::<std::collections::HashSet<String>>(), // Explicitly collect into set for uniqueness
                        functionality_area: format!("Class: {}", class.name),
                        confidence: 0.8,
                    };
                    patterns.push(pattern);
                }
            }
        }
        
        patterns
    }

    /// Analyze dependencies for a specific class
    fn analyze_class_dependencies(&self, class_name: &str, _ast_provider: &dyn AstProvider, _file_path: &Path) -> Vec<String> {
        let mut dependencies = Vec::new();
        
        // Simplified dependency analysis - in practice, this would:
        // 1. Parse import statements
        // 2. Analyze method calls to other classes
        // 3. Track field access patterns
        // 4. Identify inheritance relationships
        
        // Simulate finding dependencies based on class patterns
        if class_name.contains("Service") || class_name.contains("Manager") {
            dependencies.extend([
                "repository.rs".to_string(),
                "validator.rs".to_string(),
                "logger.rs".to_string(),
                "notifier.rs".to_string(),
                "cache.rs".to_string(),
                "config.rs".to_string(),
            ]);
        }
        
        dependencies
    }

    /// Determine the severity of shotgun surgery issues
    fn determine_severity(&self, pattern: &ChangePattern) -> IssueSeverity {
        let affected_count = pattern.affected_files.len();
        
        if affected_count >= 8 && pattern.confidence > 0.8 {
            IssueSeverity::Critical
        } else if affected_count >= 6 && pattern.confidence > 0.7 {
            IssueSeverity::Major
        } else if affected_count >= 4 && pattern.confidence > 0.6 {
            IssueSeverity::Minor
        } else {
            IssueSeverity::Info
        }
    }
}

impl Default for ShotgunSurgeryDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisDetector for ShotgunSurgeryDetector {
    async fn detect_issues(
        &self,
        parsed_file: &ParsedFile,
    ) -> Result<Vec<ArchitecturalIssue>, AnalysisError> {
        let mut issues = Vec::new();
        
        // Simplified implementation for demonstration
        // In a real implementation, this would analyze the AST for cross-cutting concerns
        match parsed_file.language {
            SourceLanguage::Rust => {
                // Analyze Rust-specific patterns
                if self.has_shotgun_surgery_indicators(parsed_file) {
                    // Create metadata with issue_type and additional information
                    let mut metadata = serde_json::Map::new();
                    metadata.insert("issue_type".to_string(), serde_json::Value::String("shotgun_surgery".to_string()));
                    metadata.insert("rule_id".to_string(), serde_json::Value::String("shotgun_surgery".to_string()));
                    metadata.insert("confidence_score".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(0.7).unwrap_or(serde_json::Number::from(0))));
                    metadata.insert("impact_score".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(0.6).unwrap_or(serde_json::Number::from(0))));
                    metadata.insert("tags".to_string(), serde_json::Value::Array(vec![
                        serde_json::Value::String("maintainability".to_string()),
                        serde_json::Value::String("coupling".to_string()),
                        serde_json::Value::String("separation-of-concerns".to_string()),
                    ]));

                    let title = "Potential Shotgun Surgery pattern detected".to_string();
                    let description = "This file may require changes across multiple modules when implementing new features, indicating poor separation of concerns.".to_string();

                    let issue = ArchitecturalIssue {
                        issue_id: None,
                        analysis_run_id: 0, // TODO: Get from context
                        anti_pattern_type_id: 1, // TODO: Get from anti-pattern mapping
                        file_path: parsed_file.file_path.to_string_lossy().to_string(),
                        start_line: Some(1),
                        end_line: None,
                        line_number: Some(1),
                        column_number: None,
                        message: title.clone(),
                        metadata: serde_json::to_string(&metadata).unwrap_or("{}".to_string()),
                        detector_name: "ShotgunSurgeryDetector".to_string(),
                        created_at: chrono::Utc::now(),
                        severity: "minor".to_string(),
                        description,
                        code_snippet: None,
                        ai_explanation: Some("Consider refactoring to reduce cross-cutting concerns and improve modularity.".to_string()),
                    };
                    issues.push(issue);
                }
            }
            _ => {
                // Handle other languages if needed
            }
        }
        
        Ok(issues)
    }

    fn get_anti_pattern_types(&self) -> Vec<AntiPatternType> {
        vec![AntiPatternType::ShotgunSurgery]
    }

    fn get_detector_name(&self) -> &'static str {
        "shotgun_surgery"
    }
}

impl ShotgunSurgeryDetector {
    /// Check for indicators of shotgun surgery pattern
    fn has_shotgun_surgery_indicators(&self, parsed_file: &ParsedFile) -> bool {
        // Simplified heuristic - in practice would analyze AST for:
        // - Multiple module dependencies
        // - Cross-cutting concerns
        // - Widespread method calls
        let file_name = parsed_file.file_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        
        // Common patterns that might indicate shotgun surgery
        file_name.contains("service") || 
        file_name.contains("manager") || 
        file_name.contains("controller") ||
        file_name.contains("handler")
    }

    /// Generate recommendations for fixing shotgun surgery
    fn generate_recommendation(&self, pattern: &ChangePattern) -> String {
        format!(
            "Consider refactoring to reduce cross-cutting concerns:\n\
            1. Extract common functionality into a dedicated service or utility class\n\
            2. Use dependency injection to reduce direct dependencies\n\
            3. Apply the Facade pattern to provide a unified interface\n\
            4. Consider using Observer pattern for notifications\n\
            5. Implement proper separation of concerns by domain boundaries\n\
            \n\
            Affected areas ({} files): {}",
            pattern.affected_files.len(),
            pattern.affected_files.iter().take(5).cloned().collect::<Vec<_>>().join(", ")
        )
    }

    /// Calculate impact score based on the extent of shotgun surgery
    fn calculate_impact_score(&self, pattern: &ChangePattern) -> f64 {
        let base_impact = pattern.affected_files.len() as f64 / 10.0; // Normalize by max expected files
        let confidence_multiplier = pattern.confidence;
        
        (base_impact * confidence_multiplier).min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::MockAstProvider;
    use crate::models::{ClassInfo, FunctionInfo};
    use std::path::PathBuf;

    #[test]
    fn test_shotgun_surgery_detection() {
        let detector = ShotgunSurgeryDetector::new();
        let mut mock_ast = MockAstProvider::new();
        
        // Mock a function that would trigger shotgun surgery detection
        mock_ast.expect_get_functions()
            .returning(|_| Ok(vec![
                FunctionInfo {
                    name: "save_user_data".to_string(),
                    start_line: 10,
                    end_line: 50,
                    parameters: vec![],
                    return_type: Some("Result<(), Error>".to_string()),
                }
            ]));
        
        mock_ast.expect_get_classes()
            .returning(|_| Ok(vec![
                ClassInfo {
                    name: "UserService".to_string(),
                    start_line: 1,
                    end_line: 100,
                    methods: vec![],
                    fields: vec![],
                }
            ]));
        
        let file_path = PathBuf::from("src/user_service.rs");
        let result = detector.analyze(&mock_ast, &file_path);
        
        assert!(result.is_ok());
        let issues = result.unwrap();
        assert!(!issues.is_empty());
        
        let issue = &issues[0];
        assert_eq!(issue.issue_type, "shotgun_surgery");
        assert!(issue.description.contains("modifications across"));
    }

    #[test]
    fn test_severity_determination() {
        let detector = ShotgunSurgeryDetector::new();
        
        let high_coupling_pattern = ChangePattern {
            affected_files: (1..=10).map(|i| format!("file_{}.rs", i)).collect(),
            functionality_area: "critical_function".to_string(),
            confidence: 0.9,
        };
        
        assert_eq!(detector.determine_severity(&high_coupling_pattern), IssueSeverity::Critical);
        
        let medium_coupling_pattern = ChangePattern {
            affected_files: (1..=5).map(|i| format!("file_{}.rs", i)).collect(),
            functionality_area: "moderate_function".to_string(),
            confidence: 0.7,
        };
        
        assert_eq!(detector.determine_severity(&medium_coupling_pattern), IssueSeverity::Minor);
    }
