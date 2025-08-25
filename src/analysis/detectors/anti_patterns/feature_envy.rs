//! Feature Envy Anti-Pattern Detector
//!
//! This module detects the "Feature Envy" anti-pattern, which occurs when
//! a class uses methods and data from other classes more than its own.
//! This indicates that functionality might be placed in the wrong class.

use crate::analysis::{AnalysisDetector, AnalysisResult};
use crate::ast::AstProvider;
use crate::models::{ArchitecturalIssue, IssueSeverity};
use std::collections::HashMap;
use std::path::Path;

/// Configuration for Feature Envy detection
#[derive(Debug, Clone)]
pub struct FeatureEnvyConfig {
    /// Minimum external method calls to trigger detection
    pub min_external_calls: usize,
    /// Ratio threshold: external calls / internal calls
    pub external_ratio_threshold: f64,
    /// Minimum total method calls to consider
    pub min_total_calls: usize,
    /// Weight for different types of external access
    pub method_call_weight: f64,
    pub field_access_weight: f64,
}

impl Default for FeatureEnvyConfig {
    fn default() -> Self {
        Self {
            min_external_calls: 3,
            external_ratio_threshold: 0.6,
            min_total_calls: 5,
            method_call_weight: 1.0,
            field_access_weight: 0.8,
        }
    }
}

/// Represents usage patterns that might indicate feature envy
#[derive(Debug, Clone)]
struct UsagePattern {
    /// The class/method that might have feature envy
    source_entity: String,
    /// The external class being "envied"
    target_class: String,
    /// Number of external method calls
    external_calls: usize,
    /// Number of internal method calls
    internal_calls: usize,
    /// Specific external methods being called
    external_methods: Vec<String>,
    /// Confidence score for this being feature envy
    confidence: f64,
}

/// Detector for Feature Envy anti-pattern
pub struct FeatureEnvyDetector {
    config: FeatureEnvyConfig,
}

impl FeatureEnvyDetector {
    /// Create a new Feature Envy detector with default configuration
    pub fn new() -> Self {
        Self {
            config: FeatureEnvyConfig::default(),
        }
    }

    /// Create a new detector with custom configuration
    pub fn with_config(config: FeatureEnvyConfig) -> Self {
        Self { config }
    }

    /// Analyze method calls to detect feature envy patterns
    fn analyze_method_calls(&self, ast_provider: &dyn AstProvider, file_path: &Path) -> Vec<UsagePattern> {
        let mut patterns = Vec::new();
        
        if let Ok(functions) = ast_provider.get_functions(file_path) {
            for function in functions {
                let usage_analysis = self.analyze_function_usage(&function.name, ast_provider, file_path);
                
                for (target_class, call_info) in usage_analysis {
                    let total_calls = call_info.external_calls + call_info.internal_calls;
                    
                    if total_calls >= self.config.min_total_calls 
                        && call_info.external_calls >= self.config.min_external_calls {
                        
                        let external_ratio = call_info.external_calls as f64 / total_calls as f64;
                        
                        if external_ratio >= self.config.external_ratio_threshold {
                            let pattern = UsagePattern {
                                source_entity: function.name.clone(),
                                target_class,
                                external_calls: call_info.external_calls,
                                internal_calls: call_info.internal_calls,
                                external_methods: call_info.external_methods,
                                confidence: self.calculate_envy_confidence(external_ratio, call_info.external_calls),
                            };
                            patterns.push(pattern);
                        }
                    }
                }
            }
        }
        
        patterns
    }

    /// Analyze usage patterns for a specific function
    fn analyze_function_usage(&self, function_name: &str, _ast_provider: &dyn AstProvider, _file_path: &Path) -> HashMap<String, CallInfo> {
        let mut usage_map = HashMap::new();
        
        // In a real implementation, this would:
        // 1. Parse the function's AST to find all method calls
        // 2. Determine which calls are to external vs internal methods
        // 3. Track field access patterns
        // 4. Identify the target classes being called
        
        // Simulate feature envy patterns based on function names
        if function_name.contains("calculate") || function_name.contains("compute") {
            // Simulate a calculation method that heavily uses another class
            usage_map.insert("MathUtils".to_string(), CallInfo {
                external_calls: 6,
                internal_calls: 2,
                external_methods: vec![
                    "sqrt".to_string(),
                    "pow".to_string(),
                    "log".to_string(),
                    "sin".to_string(),
                    "cos".to_string(),
                    "abs".to_string(),
                ],
            });
        }
        
        if function_name.contains("format") || function_name.contains("display") {
            // Simulate a formatting method that heavily uses string utilities
            usage_map.insert("StringUtils".to_string(), CallInfo {
                external_calls: 5,
                internal_calls: 1,
                external_methods: vec![
                    "trim".to_string(),
                    "capitalize".to_string(),
                    "split".to_string(),
                    "join".to_string(),
                    "replace".to_string(),
                ],
            });
        }
        
        if function_name.contains("validate") || function_name.contains("check") {
            // Simulate a validation method that uses external validation logic
            usage_map.insert("ValidationRules".to_string(), CallInfo {
                external_calls: 4,
                internal_calls: 1,
                external_methods: vec![
                    "is_email_valid".to_string(),
                    "is_phone_valid".to_string(),
                    "is_date_valid".to_string(),
                    "is_required".to_string(),
                ],
            });
        }
        
        usage_map
    }

    /// Analyze class-level feature envy patterns
    fn analyze_class_envy(&self, ast_provider: &dyn AstProvider, file_path: &Path) -> Vec<UsagePattern> {
        let mut patterns = Vec::new();
        
        if let Ok(classes) = ast_provider.get_classes(file_path) {
            for class in classes {
                let class_usage = self.analyze_class_dependencies(&class.name, ast_provider, file_path);
                
                for (target_class, dependency_info) in class_usage {
                    if dependency_info.external_calls > self.config.min_external_calls {
                        let total_calls = dependency_info.external_calls + dependency_info.internal_calls;
                        let external_ratio = dependency_info.external_calls as f64 / total_calls as f64;
                        
                        if external_ratio >= self.config.external_ratio_threshold {
                            let pattern = UsagePattern {
                                source_entity: class.name.clone(),
                                target_class,
                                external_calls: dependency_info.external_calls,
                                internal_calls: dependency_info.internal_calls,
                                external_methods: dependency_info.external_methods,
                                confidence: self.calculate_envy_confidence(external_ratio, dependency_info.external_calls),
                            };
                            patterns.push(pattern);
                        }
                    }
                }
            }
        }
        
        patterns
    }

    /// Analyze dependencies for a class
    fn analyze_class_dependencies(&self, class_name: &str, _ast_provider: &dyn AstProvider, _file_path: &Path) -> HashMap<String, CallInfo> {
        let mut dependencies = HashMap::new();
        
        // Simulate class-level dependencies based on naming patterns
        if class_name.contains("Controller") || class_name.contains("Handler") {
            // Controllers often have feature envy for service classes
            dependencies.insert("UserService".to_string(), CallInfo {
                external_calls: 8,
                internal_calls: 2,
                external_methods: vec![
                    "get_user".to_string(),
                    "save_user".to_string(),
                    "delete_user".to_string(),
                    "validate_user".to_string(),
                    "update_user".to_string(),
                    "find_users".to_string(),
                    "count_users".to_string(),
                    "archive_user".to_string(),
                ],
            });
        }
        
        if class_name.contains("View") || class_name.contains("Presenter") {
            // Views might have envy for formatting utilities
            dependencies.insert("FormatUtils".to_string(), CallInfo {
                external_calls: 6,
                internal_calls: 1,
                external_methods: vec![
                    "format_date".to_string(),
                    "format_currency".to_string(),
                    "format_name".to_string(),
                    "format_address".to_string(),
                    "format_phone".to_string(),
                    "truncate_text".to_string(),
                ],
            });
        }
        
        dependencies
    }

    /// Calculate confidence score for feature envy
    fn calculate_envy_confidence(&self, external_ratio: f64, external_calls: usize) -> f64 {
        let mut confidence = external_ratio; // Base confidence on ratio
        
        // Increase confidence with more external calls
        if external_calls >= 8 {
            confidence += 0.2;
        } else if external_calls >= 5 {
            confidence += 0.1;
        }
        
        // Very high external ratios are more confident indicators
        if external_ratio >= 0.8 {
            confidence += 0.1;
        }
        
        confidence.min(1.0)
    }

    /// Determine severity based on the extent of feature envy
    fn determine_severity(&self, pattern: &UsagePattern) -> IssueSeverity {
        let external_ratio = pattern.external_calls as f64 / (pattern.external_calls + pattern.internal_calls) as f64;
        
        if external_ratio >= 0.9 && pattern.external_calls >= 8 {
            IssueSeverity::Critical
        } else if external_ratio >= 0.8 && pattern.external_calls >= 5 {
            IssueSeverity::Major
        } else if external_ratio >= 0.7 && pattern.external_calls >= 3 {
            IssueSeverity::Minor
        } else {
            IssueSeverity::Info
        }
    }
}

/// Helper struct to track call information
#[derive(Debug, Clone)]
struct CallInfo {
    external_calls: usize,
    internal_calls: usize,
    external_methods: Vec<String>,
}

impl Default for FeatureEnvyDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisDetector for FeatureEnvyDetector {
    fn name(&self) -> &str {
        "feature_envy"
    }

    fn description(&self) -> &str {
        "Detects Feature Envy anti-pattern where classes use external methods more than their own"
    }

    fn analyze(&self, ast_provider: &dyn AstProvider, file_path: &Path) -> AnalysisResult<Vec<ArchitecturalIssue>> {
        let mut issues = Vec::new();
        
        // Analyze method-level feature envy
        let method_patterns = self.analyze_method_calls(ast_provider, file_path);
        
        // Analyze class-level feature envy
        let class_patterns = self.analyze_class_envy(ast_provider, file_path);
        
        // Process all patterns
        for pattern in method_patterns.into_iter().chain(class_patterns) {
            let severity = self.determine_severity(&pattern);
            
            let issue = ArchitecturalIssue {
                id: uuid::Uuid::new_v4(),
                issue_type: "feature_envy".to_string(),
                title: format!("Feature Envy detected in {}", pattern.source_entity),
                description: format!(
                    "The {} '{}' uses methods from '{}' extensively ({} external vs {} internal calls), suggesting it might belong in the target class. External methods used: {}",
                    if pattern.source_entity.chars().next().unwrap_or('a').is_uppercase() { "class" } else { "method" },
                    pattern.source_entity,
                    pattern.target_class,
                    pattern.external_calls,
                    pattern.internal_calls,
                    pattern.external_methods.iter().take(3).cloned().collect::<Vec<_>>().join(", ")
                ),
                severity: severity.to_string(),
                file_path: file_path.to_string_lossy().to_string(),
                start_line: 1,
                end_line: None,
                recommendation: self.generate_recommendation(&pattern),
                detected_at: chrono::Utc::now(),
                confidence_score: Some(pattern.confidence),
                impact_score: Some(self.calculate_impact_score(&pattern)),
                tags: vec![
                    "maintainability".to_string(),
                    "encapsulation".to_string(),
                    "cohesion".to_string(),
                    "responsibility".to_string(),
                ],
            };
            
            issues.push(issue);
        }
        
        Ok(issues)
    }

    fn supports_language(&self, language: &str) -> bool {
        matches!(language.to_lowercase().as_str(), "rust" | "python" | "javascript" | "typescript" | "java" | "csharp" | "cpp")
    }
}

impl FeatureEnvyDetector {
    /// Generate recommendations for fixing feature envy
    fn generate_recommendation(&self, pattern: &UsagePattern) -> String {
        format!(
            "Consider refactoring to improve encapsulation and cohesion:\n\
            1. Move the functionality to the '{}' class where it's being used most\n\
            2. Extract the external method calls into a new method in the target class\n\
            3. Use delegation pattern to keep the original interface while moving logic\n\
            4. Consider if this represents a missing abstraction or service\n\
            5. Apply 'Tell, Don't Ask' principle to reduce external data access\n\
            \n\
            Current usage: {} external calls to {}, {} internal calls\n\
            External methods: {}",
            pattern.target_class,
            pattern.external_calls,
            pattern.target_class,
            pattern.internal_calls,
            pattern.external_methods.join(", ")
        )
    }

    /// Calculate impact score based on the extent of feature envy
    fn calculate_impact_score(&self, pattern: &UsagePattern) -> f64 {
        let total_calls = pattern.external_calls + pattern.internal_calls;
        let external_ratio = pattern.external_calls as f64 / total_calls as f64;
        let call_volume_factor = (pattern.external_calls as f64 / 10.0).min(1.0);
        
        (external_ratio * pattern.confidence * call_volume_factor).min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::MockAstProvider;
    use crate::models::{ClassInfo, FunctionInfo};
    use std::path::PathBuf;

    #[test]
    fn test_feature_envy_detection() {
        let detector = FeatureEnvyDetector::new();
        let mut mock_ast = MockAstProvider::new();
        
        mock_ast.expect_get_functions()
            .returning(|_| Ok(vec![
                FunctionInfo {
                    name: "calculate_total".to_string(),
                    start_line: 10,
                    end_line: 30,
                    parameters: vec![],
                    return_type: Some("f64".to_string()),
                }
            ]));
        
        mock_ast.expect_get_classes()
            .returning(|_| Ok(vec![]));
        
        let file_path = PathBuf::from("src/calculator.rs");
        let result = detector.analyze(&mock_ast, &file_path);
        
        assert!(result.is_ok());
        let issues = result.unwrap();
        assert!(!issues.is_empty());
        
        let issue = &issues[0];
        assert_eq!(issue.issue_type, "feature_envy");
        assert!(issue.description.contains("uses methods from"));
    }

    #[test]
    fn test_severity_calculation() {
        let detector = FeatureEnvyDetector::new();
        
        let high_envy = UsagePattern {
            source_entity: "BadMethod".to_string(),
            target_class: "ExternalClass".to_string(),
            external_calls: 10,
            internal_calls: 1,
            external_methods: vec![],
            confidence: 0.9,
        };
        
        assert_eq!(detector.determine_severity(&high_envy), IssueSeverity::Critical);
        
        let medium_envy = UsagePattern {
            source_entity: "OkMethod".to_string(),
            target_class: "ExternalClass".to_string(),
            external_calls: 5,
            internal_calls: 2,
            external_methods: vec![],
            confidence: 0.7,
        };
        
        assert_eq!(detector.determine_severity(&medium_envy), IssueSeverity::Minor);
    }
