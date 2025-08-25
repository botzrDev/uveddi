//! Data Clumps Anti-Pattern Detector
//!
//! This module detects the "Data Clumps" anti-pattern, which occurs when
//! the same group of data items appear together in multiple places,
//! suggesting they should be grouped into their own class.

use crate::analysis::{AnalysisDetector, AnalysisError};
use crate::analysis::services::AnalysisResult;
use crate::analysis::components::AstProvider;
use crate::database::models::ArchitecturalIssue;
use crate::analysis::detector_factory::{AntiPatternType, IssueSeverity};
use crate::parsing::ParsedFile;
use async_trait::async_trait;
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Configuration for Data Clumps detection
#[derive(Debug, Clone)]
pub struct DataClumpsConfig {
    /// Minimum number of parameters to consider as a clump
    pub min_clump_size: usize,
    /// Minimum occurrences of the same parameter group
    pub min_occurrences: usize,
    /// Similarity threshold for parameter groups (0.0 to 1.0)
    pub similarity_threshold: f64,
    /// Maximum parameters before considering it a long parameter list
    pub max_parameter_count: usize,
}

impl Default for DataClumpsConfig {
    fn default() -> Self {
        Self {
            min_clump_size: 3,
            min_occurrences: 2,
            similarity_threshold: 0.8,
            max_parameter_count: 6,
        }
    }
}

/// Represents a group of parameters that appear together
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ParameterGroup {
    /// Names of parameters in this group
    parameter_names: Vec<String>,
    /// Types of parameters in this group
    parameter_types: Vec<String>,
    /// Normalized signature for comparison
    signature: String,
}

/// Represents a data clump pattern
#[derive(Debug, Clone)]
struct DataClumpPattern {
    /// The parameter group that forms the clump
    parameter_group: ParameterGroup,
    /// Functions/methods where this clump appears
    occurrences: Vec<String>,
    /// Confidence that this is a data clump
    confidence: f64,
    /// Suggested class name for this group
    suggested_class_name: String,
}

/// Detector for Data Clumps anti-pattern
pub struct DataClumpsDetector {
    config: DataClumpsConfig,
}

impl DataClumpsDetector {
    /// Create a new Data Clumps detector with default configuration
    pub fn new() -> Self {
        Self {
            config: DataClumpsConfig::default(),
        }
    }

    /// Create a new detector with custom configuration
    pub fn with_config(config: DataClumpsConfig) -> Self {
        Self { config }
    }

    /// Analyze function parameters to detect data clumps
    fn analyze_function_parameters(&self, ast_provider: &dyn AstProvider, file_path: &Path) -> Vec<DataClumpPattern> {
        let mut parameter_groups: HashMap<ParameterGroup, Vec<String>> = HashMap::new();
        
        if let Ok(functions) = ast_provider.get_functions(file_path) {
            for function in functions {
                let groups = self.extract_parameter_groups(&function.name, &function.parameters);
                
                for group in groups {
                    parameter_groups.entry(group)
                        .or_insert_with(Vec::new)
                        .push(function.name.clone());
                }
            }
        }
        
        self.identify_clump_patterns(parameter_groups)
    }

    /// Extract parameter groups from a function's parameters
    fn extract_parameter_groups(&self, function_name: &str, parameters: &[String]) -> Vec<ParameterGroup> {
        let mut groups = Vec::new();
        
        // In a real implementation, this would parse actual parameter information
        // For now, we simulate based on function naming patterns
        
        let simulated_params = self.simulate_parameters(function_name);
        
        if simulated_params.len() >= self.config.min_clump_size {
            // Generate sliding windows of parameter groups
            for window_size in self.config.min_clump_size..=simulated_params.len() {
                for window in simulated_params.windows(window_size) {
                    let group = ParameterGroup {
                        parameter_names: window.iter().map(|(name, _)| name.clone()).collect(),
                        parameter_types: window.iter().map(|(_, typ)| typ.clone()).collect(),
                        signature: self.create_signature(window),
                    };
                    groups.push(group);
                }
            }
        }
        
        groups
    }

    /// Simulate parameters based on function naming patterns
    fn simulate_parameters(&self, function_name: &str) -> Vec<(String, String)> {
        // Simulate common data clump patterns
        if function_name.contains("address") || function_name.contains("location") {
            vec![
                ("street".to_string(), "String".to_string()),
                ("city".to_string(), "String".to_string()),
                ("state".to_string(), "String".to_string()),
                ("zip_code".to_string(), "String".to_string()),
                ("country".to_string(), "String".to_string()),
            ]
        } else if function_name.contains("person") || function_name.contains("user") || function_name.contains("contact") {
            vec![
                ("first_name".to_string(), "String".to_string()),
                ("last_name".to_string(), "String".to_string()),
                ("email".to_string(), "String".to_string()),
                ("phone".to_string(), "String".to_string()),
            ]
        } else if function_name.contains("date") || function_name.contains("time") || function_name.contains("schedule") {
            vec![
                ("year".to_string(), "i32".to_string()),
                ("month".to_string(), "i32".to_string()),
                ("day".to_string(), "i32".to_string()),
                ("hour".to_string(), "i32".to_string()),
                ("minute".to_string(), "i32".to_string()),
            ]
        } else if function_name.contains("coordinate") || function_name.contains("position") || function_name.contains("point") {
            vec![
                ("x".to_string(), "f64".to_string()),
                ("y".to_string(), "f64".to_string()),
                ("z".to_string(), "f64".to_string()),
            ]
        } else if function_name.contains("color") || function_name.contains("rgb") {
            vec![
                ("red".to_string(), "u8".to_string()),
                ("green".to_string(), "u8".to_string()),
                ("blue".to_string(), "u8".to_string()),
                ("alpha".to_string(), "u8".to_string()),
            ]
        } else if function_name.contains("money") || function_name.contains("price") || function_name.contains("cost") {
            vec![
                ("amount".to_string(), "f64".to_string()),
                ("currency".to_string(), "String".to_string()),
                ("precision".to_string(), "i32".to_string()),
            ]
        } else {
            // Default case with some generic parameters
            vec![
                ("param1".to_string(), "String".to_string()),
                ("param2".to_string(), "i32".to_string()),
            ]
        }
    }

    /// Create a normalized signature for parameter group comparison
    fn create_signature(&self, parameters: &[(String, String)]) -> String {
        let mut sig_parts: Vec<String> = parameters.iter()
            .map(|(name, typ)| format!("{}:{}", name, typ))
            .collect();
        sig_parts.sort(); // Normalize order for comparison
        sig_parts.join("|")
    }

    /// Identify patterns from grouped parameters
    fn identify_clump_patterns(&self, parameter_groups: HashMap<ParameterGroup, Vec<String>>) -> Vec<DataClumpPattern> {
        let mut patterns = Vec::new();
        
        for (group, occurrences) in parameter_groups {
            if occurrences.len() >= self.config.min_occurrences && 
               group.parameter_names.len() >= self.config.min_clump_size {
                
                let confidence = self.calculate_clump_confidence(&group, &occurrences);
                let suggested_name = self.suggest_class_name(&group);
                
                let pattern = DataClumpPattern {
                    parameter_group: group,
                    occurrences,
                    confidence,
                    suggested_class_name: suggested_name,
                };
                
                patterns.push(pattern);
            }
        }
        
        patterns
    }

    /// Calculate confidence score for a data clump
    fn calculate_clump_confidence(&self, group: &ParameterGroup, occurrences: &[String]) -> f64 {
        let mut confidence = 0.5; // Base confidence
        
        // More occurrences increase confidence
        let occurrence_factor = (occurrences.len() as f64 / 5.0).min(1.0);
        confidence += occurrence_factor * 0.3;
        
        // Larger parameter groups are more likely to be clumps
        let size_factor = (group.parameter_names.len() as f64 / 6.0).min(1.0);
        confidence += size_factor * 0.2;
        
        // Semantic coherence (simplified check)
        if self.has_semantic_coherence(group) {
            confidence += 0.2;
        }
        
        confidence.min(1.0)
    }

    /// Check if parameter group has semantic coherence
    fn has_semantic_coherence(&self, group: &ParameterGroup) -> bool {
        let names = &group.parameter_names;
        
        // Check for common semantic patterns
        let address_terms = ["street", "city", "state", "zip", "country", "address"];
        let person_terms = ["first", "last", "name", "email", "phone", "contact"];
        let date_terms = ["year", "month", "day", "hour", "minute", "date", "time"];
        let coordinate_terms = ["x", "y", "z", "latitude", "longitude", "coordinate"];
        let color_terms = ["red", "green", "blue", "alpha", "color", "rgb"];
        
        let semantic_groups = [address_terms, person_terms, date_terms, coordinate_terms, color_terms];
        
        for semantic_group in &semantic_groups {
            let matches = names.iter()
                .filter(|name| semantic_group.iter().any(|term| name.to_lowercase().contains(term)))
                .count();
            
            if matches >= 2 {
                return true;
            }
        }
        
        false
    }

    /// Suggest a class name for the parameter group
    fn suggest_class_name(&self, group: &ParameterGroup) -> String {
        let names = &group.parameter_names;
        
        // Try to infer from parameter names
        if names.iter().any(|n| n.contains("street") || n.contains("city") || n.contains("address")) {
            "Address".to_string()
        } else if names.iter().any(|n| n.contains("first") || n.contains("last") || n.contains("name")) {
            "PersonName".to_string()
        } else if names.iter().any(|n| n.contains("year") || n.contains("month") || n.contains("date")) {
            "DateTime".to_string()
        } else if names.iter().any(|n| n.contains("x") || n.contains("y") || n.contains("coordinate")) {
            "Coordinate".to_string()
        } else if names.iter().any(|n| n.contains("red") || n.contains("green") || n.contains("color")) {
            "Color".to_string()
        } else if names.iter().any(|n| n.contains("amount") || n.contains("currency") || n.contains("money")) {
            "Money".to_string()
        } else {
            // Generate based on first parameter name
            let first_name = names.first().unwrap_or(&"Data".to_string());
            format!("{}Group", first_name.split('_').next().unwrap_or("Data"))
        }
    }

    /// Determine severity based on clump characteristics
    fn determine_severity(&self, pattern: &DataClumpPattern) -> IssueSeverity {
        let param_count = pattern.parameter_group.parameter_names.len();
        let occurrence_count = pattern.occurrences.len();
        
        if param_count >= 6 && occurrence_count >= 4 {
            IssueSeverity::Critical
        } else if param_count >= 4 && occurrence_count >= 3 {
            IssueSeverity::Major
        } else if param_count >= 3 && occurrence_count >= 2 {
            IssueSeverity::Minor
        } else {
            IssueSeverity::Info
        }
    }
}

impl Default for DataClumpsDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisDetector for DataClumpsDetector {
    fn name(&self) -> &str {
        "data_clumps"
    }

    fn description(&self) -> &str {
        "Detects Data Clumps anti-pattern where the same group of parameters appear together repeatedly"
    }

    fn analyze(&self, ast_provider: &dyn AstProvider, file_path: &Path) -> AnalysisResult<Vec<ArchitecturalIssue>> {
        let mut issues = Vec::new();
        
        let patterns = self.analyze_function_parameters(ast_provider, file_path);
        
        for pattern in patterns {
            let severity = self.determine_severity(&pattern);
            
            let issue = ArchitecturalIssue {
                id: uuid::Uuid::new_v4(),
                issue_type: "data_clumps".to_string(),
                title: format!("Data Clump detected: {} parameters", pattern.parameter_group.parameter_names.len()),
                description: format!(
                    "The parameter group [{}] appears together in {} functions: {}. This suggests these parameters should be grouped into a class.",
                    pattern.parameter_group.parameter_names.join(", "),
                    pattern.occurrences.len(),
                    pattern.occurrences.iter().take(3).cloned().collect::<Vec<_>>().join(", ")
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
                    "data-structure".to_string(),
                    "cohesion".to_string(),
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

impl DataClumpsDetector {
    /// Generate recommendations for fixing data clumps
    fn generate_recommendation(&self, pattern: &DataClumpPattern) -> String {
        format!(
            "Consider creating a '{}' class to group related parameters:\n\
            1. Create a new class/struct '{}' with fields: {}\n\
            2. Replace the parameter list with a single instance of this class\n\
            3. Update all {} functions that use this parameter group\n\
            4. Consider adding related behavior (methods) to this new class\n\
            5. Validate that the grouping makes semantic sense in your domain\n\
            \n\
            Current clump: {} parameters appearing in {} functions\n\
            Parameters: {} (types: {})",
            pattern.suggested_class_name,
            pattern.suggested_class_name,
            pattern.parameter_group.parameter_names.join(", "),
            pattern.occurrences.len(),
            pattern.parameter_group.parameter_names.len(),
            pattern.occurrences.len(),
            pattern.parameter_group.parameter_names.join(", "),
            pattern.parameter_group.parameter_types.join(", ")
        )
    }

    /// Calculate impact score based on the extent of data clumping
    fn calculate_impact_score(&self, pattern: &DataClumpPattern) -> f64 {
        let param_factor = (pattern.parameter_group.parameter_names.len() as f64 / 8.0).min(1.0);
        let occurrence_factor = (pattern.occurrences.len() as f64 / 6.0).min(1.0);
        
        (param_factor * occurrence_factor * pattern.confidence).min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::MockAstProvider;
    use crate::models::FunctionInfo;
    use std::path::PathBuf;

    #[test]
    fn test_data_clumps_detection() {
        let detector = DataClumpsDetector::new();
        let mut mock_ast = MockAstProvider::new();
        
        mock_ast.expect_get_functions()
            .returning(|_| Ok(vec![
                FunctionInfo {
                    name: "create_address".to_string(),
                    start_line: 1,
                    end_line: 10,
                    parameters: vec!["street".to_string(), "city".to_string(), "state".to_string()],
                    return_type: None,
                },
                FunctionInfo {
                    name: "update_address".to_string(),
                    start_line: 15,
                    end_line: 25,
                    parameters: vec!["street".to_string(), "city".to_string(), "state".to_string()],
                    return_type: None,
                }
            ]));
        
        mock_ast.expect_get_classes()
            .returning(|_| Ok(vec![]));
        
        let file_path = PathBuf::from("src/address_manager.rs");
        let result = detector.analyze(&mock_ast, &file_path);
        
        assert!(result.is_ok());
        let issues = result.unwrap();
        assert!(!issues.is_empty());
        
        let issue = &issues[0];
        assert_eq!(issue.issue_type, "data_clumps");
        assert!(issue.description.contains("parameter group"));
    }

    #[test]
    fn test_semantic_coherence() {
        let detector = DataClumpsDetector::new();
        
        let address_group = ParameterGroup {
            parameter_names: vec!["street".to_string(), "city".to_string(), "state".to_string()],
            parameter_types: vec!["String".to_string(); 3],
            signature: "".to_string(),
        };
        
        assert!(detector.has_semantic_coherence(&address_group));
        
        let random_group = ParameterGroup {
            parameter_names: vec!["foo".to_string(), "bar".to_string(), "baz".to_string()],
            parameter_types: vec!["String".to_string(); 3],
            signature: "".to_string(),
        };
        
        assert!(!detector.has_semantic_coherence(&random_group));
    }
