//! Smart configuration validation and suggestions system
//!
//! This module provides comprehensive configuration validation with intelligent
//! suggestions for optimal settings based on project characteristics.

use crate::config::{Config, DeadCodeConfig, LargeClassConfig, LanguageThresholds};
use crate::error::UveddiError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub suggestions: Vec<ConfigSuggestion>,
    pub performance_score: f64,
    pub completeness_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub severity: ErrorSeverity,
    pub fix_suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub field: String,
    pub message: String,
    pub impact: ImpactLevel,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSuggestion {
    pub category: SuggestionCategory,
    pub title: String,
    pub description: String,
    pub before: Option<String>,
    pub after: String,
    pub benefit: String,
    pub priority: Priority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImpactLevel {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionCategory {
    Performance,
    Accuracy,
    Security,
    Maintenance,
    Compatibility,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone)]
pub struct ProjectContext {
    pub languages: Vec<String>,
    pub estimated_size: ProjectSize,
    pub has_tests: bool,
    pub has_ci: bool,
    pub framework: Option<String>,
    pub build_tools: Vec<String>,
    pub team_size: TeamSize,
}

#[derive(Debug, Clone)]
pub enum ProjectSize {
    Small,   // < 1k LOC
    Medium,  // 1k-10k LOC
    Large,   // 10k-100k LOC
    Huge,    // 100k+ LOC
}

#[derive(Debug, Clone)]
pub enum TeamSize {
    Individual,  // 1 person
    Small,       // 2-5 people
    Medium,      // 6-20 people
    Large,       // 20+ people
}

pub struct SmartConfigValidator {
    context: Option<ProjectContext>,
}

impl SmartConfigValidator {
    pub fn new() -> Self {
        Self { context: None }
    }

    pub fn with_context(mut self, context: ProjectContext) -> Self {
        self.context = Some(context);
        self
    }

    pub async fn validate(&self, config: &Config, project_path: Option<&Path>) -> Result<ValidationResult, UveddiError> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut suggestions = Vec::new();

        // Basic validation
        self.validate_basic_structure(config, &mut errors);
        
        // AI configuration validation
        self.validate_ai_config(config, &mut errors, &mut warnings, &mut suggestions);
        
        // Dead code configuration validation
        self.validate_dead_code_config(config, &mut errors, &mut warnings, &mut suggestions);
        
        // Large classes configuration validation
        self.validate_large_classes_config(config, &mut errors, &mut warnings, &mut suggestions);
        
        // Context-aware suggestions
        if let Some(ref context) = self.context {
            self.generate_context_aware_suggestions(config, context, &mut suggestions);
        }
        
        // Project-specific validation
        if let Some(path) = project_path {
            self.validate_project_specific_settings(config, path, &mut warnings, &mut suggestions).await;
        }

        let performance_score = self.calculate_performance_score(config, &suggestions);
        let completeness_score = self.calculate_completeness_score(config);

        Ok(ValidationResult {
            is_valid: errors.iter().all(|e| !matches!(e.severity, ErrorSeverity::Critical)),
            errors,
            warnings,
            suggestions,
            performance_score,
            completeness_score,
        })
    }

    fn validate_basic_structure(&self, config: &Config, errors: &mut Vec<ValidationError>) {
        // Check if configuration is completely empty
        if config.ollama_model.is_none() && config.dead_code.is_none() && config.large_classes.is_none() {
            errors.push(ValidationError {
                field: "root".to_string(),
                message: "Configuration appears to be empty".to_string(),
                severity: ErrorSeverity::High,
                fix_suggestion: Some("Run 'uveddi init' to generate a complete configuration".to_string()),
            });
        }
    }

    fn validate_ai_config(&self, config: &Config, errors: &mut Vec<ValidationError>, warnings: &mut Vec<ValidationWarning>, suggestions: &mut Vec<ConfigSuggestion>) {
        if let Some(ref model) = config.ollama_model {
            // Check model name format
            if !model.contains(':') {
                errors.push(ValidationError {
                    field: "ollama_model".to_string(),
                    message: "Model name should include version tag (e.g., 'model:version')".to_string(),
                    severity: ErrorSeverity::Medium,
                    fix_suggestion: Some(format!("Change '{}' to '{}:latest' or specify a version", model, model)),
                });
            }

            // Suggest optimal models based on project context
            if let Some(ref context) = self.context {
                let optimal_model = self.suggest_optimal_model(context);
                if model != &optimal_model {
                    suggestions.push(ConfigSuggestion {
                        category: SuggestionCategory::Performance,
                        title: "Optimal AI Model".to_string(),
                        description: format!("Based on your project size ({:?}), consider using a different model", context.estimated_size),
                        before: Some(model.clone()),
                        after: optimal_model,
                        benefit: "Better performance and accuracy for your project size".to_string(),
                        priority: Priority::Medium,
                    });
                }
            }

            // Check for deprecated models
            let deprecated_models = ["codellama:7b", "llama2:7b"];
            if deprecated_models.iter().any(|&deprecated| model.starts_with(deprecated)) {
                warnings.push(ValidationWarning {
                    field: "ollama_model".to_string(),
                    message: format!("Model '{}' may be deprecated or suboptimal", model),
                    impact: ImpactLevel::Medium,
                    recommendation: "Consider upgrading to 'deepseek-coder:6.7b-instruct-q4_0' for better code analysis".to_string(),
                });
            }
        } else {
            // No AI model configured
            suggestions.push(ConfigSuggestion {
                category: SuggestionCategory::Accuracy,
                title: "Enable AI Analysis".to_string(),
                description: "AI-powered analysis provides detailed explanations and suggestions".to_string(),
                before: None,
                after: "ollama_model = \"deepseek-coder:6.7b-instruct-q4_0\"".to_string(),
                benefit: "Get intelligent insights and explanations for detected issues".to_string(),
                priority: Priority::High,
            });
        }
    }

    fn validate_dead_code_config(&self, config: &Config, errors: &mut Vec<ValidationError>, warnings: &mut Vec<ValidationWarning>, suggestions: &mut Vec<ConfigSuggestion>) {
        if let Some(ref dc_config) = config.dead_code {
            // Validate confidence threshold
            if let Some(confidence) = dc_config.confidence_threshold {
                if confidence < 0.1 {
                    warnings.push(ValidationWarning {
                        field: "dead_code.confidence_threshold".to_string(),
                        message: "Very low confidence threshold may produce false positives".to_string(),
                        impact: ImpactLevel::High,
                        recommendation: "Consider using a threshold of at least 0.5 for reliable results".to_string(),
                    });
                } else if confidence > 0.95 {
                    warnings.push(ValidationWarning {
                        field: "dead_code.confidence_threshold".to_string(),
                        message: "Very high confidence threshold may miss actual dead code".to_string(),
                        impact: ImpactLevel::Medium,
                        recommendation: "Consider using a threshold between 0.7-0.9 for balanced results".to_string(),
                    });
                }
            }

            // Validate ignore patterns
            if let Some(ref patterns) = dc_config.ignore_patterns {
                for pattern in patterns {
                    if pattern.contains("**/") && !pattern.starts_with("**/") {
                        warnings.push(ValidationWarning {
                            field: "dead_code.ignore_patterns".to_string(),
                            message: format!("Pattern '{}' may not work as expected", pattern),
                            impact: ImpactLevel::Low,
                            recommendation: "Use patterns like '**/*.test.js' or 'tests/**' for recursive matching".to_string(),
                        });
                    }
                }

                // Suggest common missing patterns
                let common_patterns = ["node_modules/**", "dist/**", "build/**", "target/**"];
                let missing_patterns: Vec<&str> = common_patterns
                    .iter()
                    .filter(|&&pattern| !patterns.iter().any(|p| p.contains(&pattern[..pattern.len()-3])))
                    .copied()
                    .collect();

                if !missing_patterns.is_empty() {
                    suggestions.push(ConfigSuggestion {
                        category: SuggestionCategory::Performance,
                        title: "Add Common Ignore Patterns".to_string(),
                        description: "Exclude build directories and dependencies from analysis".to_string(),
                        before: None,
                        after: format!("ignore_patterns = {:#?}", missing_patterns),
                        benefit: "Faster analysis and fewer false positives".to_string(),
                        priority: Priority::High,
                    });
                }
            }

            // Context-aware suggestions for library mode
            if let Some(ref context) = self.context {
                let should_be_library = context.languages.iter().any(|lang| 
                    matches!(lang.as_str(), "rust" | "typescript") && 
                    context.framework.as_ref().map_or(false, |f| f.contains("lib"))
                );
                
                if should_be_library && !dc_config.library_mode.unwrap_or(false) {
                    suggestions.push(ConfigSuggestion {
                        category: SuggestionCategory::Accuracy,
                        title: "Enable Library Mode".to_string(),
                        description: "Your project appears to be a library - enable library mode for better analysis".to_string(),
                        before: Some("library_mode = false".to_string()),
                        after: "library_mode = true".to_string(),
                        benefit: "More accurate dead code detection for library projects".to_string(),
                        priority: Priority::Medium,
                    });
                }
            }
        } else {
            // No dead code configuration
            suggestions.push(ConfigSuggestion {
                category: SuggestionCategory::Accuracy,
                title: "Configure Dead Code Detection".to_string(),
                description: "Dead code detection helps identify unused functions and variables".to_string(),
                before: None,
                after: "[dead_code]\nconfidence_threshold = 0.8\nlibrary_mode = false".to_string(),
                benefit: "Identify and remove unused code to improve maintainability".to_string(),
                priority: Priority::High,
            });
        }
    }

    fn validate_large_classes_config(&self, config: &Config, errors: &mut Vec<ValidationError>, warnings: &mut Vec<ValidationWarning>, suggestions: &mut Vec<ConfigSuggestion>) {
        if let Some(ref lc_config) = config.large_classes {
            // Validate thresholds are reasonable
            if let Some(max_loc) = lc_config.max_logical_loc {
                if max_loc < 50 {
                    warnings.push(ValidationWarning {
                        field: "large_classes.max_logical_loc".to_string(),
                        message: "Very low LOC threshold may flag normal classes".to_string(),
                        impact: ImpactLevel::Medium,
                        recommendation: "Consider using at least 100 LOC as threshold".to_string(),
                    });
                } else if max_loc > 1000 {
                    warnings.push(ValidationWarning {
                        field: "large_classes.max_logical_loc".to_string(),
                        message: "Very high LOC threshold may miss problematic classes".to_string(),
                        impact: ImpactLevel::Medium,
                        recommendation: "Consider using 200-400 LOC as threshold for better detection".to_string(),
                    });
                }
            }

            // Validate language overrides
            if let Some(ref overrides) = lc_config.language_overrides {
                for (lang, thresholds) in overrides {
                    if let Some(max_loc) = thresholds.max_logical_loc {
                        if max_loc == lc_config.max_logical_loc.unwrap_or(0) {
                            warnings.push(ValidationWarning {
                                field: format!("large_classes.language_overrides.{}", lang),
                                message: format!("Language override for {} has same value as default", lang),
                                impact: ImpactLevel::Low,
                                recommendation: "Remove redundant language overrides".to_string(),
                            });
                        }
                    }
                }
            }

            // Context-aware threshold suggestions
            if let Some(ref context) = self.context {
                let optimal_thresholds = self.suggest_optimal_thresholds(context);
                if let Some(current_loc) = lc_config.max_logical_loc {
                    if (current_loc as f64 - optimal_thresholds.max_logical_loc as f64).abs() > 50.0 {
                        suggestions.push(ConfigSuggestion {
                            category: SuggestionCategory::Accuracy,
                            title: "Optimize LOC Threshold".to_string(),
                            description: format!("Based on your project type ({:?}), adjust LOC threshold", context.estimated_size),
                            before: Some(format!("max_logical_loc = {}", current_loc)),
                            after: format!("max_logical_loc = {}", optimal_thresholds.max_logical_loc),
                            benefit: "Better balance between detection accuracy and false positives".to_string(),
                            priority: Priority::Medium,
                        });
                    }
                }
            }
        } else {
            // No large classes configuration
            suggestions.push(ConfigSuggestion {
                category: SuggestionCategory::Accuracy,
                title: "Configure Large Classes Detection".to_string(),
                description: "Large class detection helps identify god objects and maintainability issues".to_string(),
                before: None,
                after: "[large_classes]\nmax_logical_loc = 250\nmax_methods = 25".to_string(),
                benefit: "Identify classes that violate single responsibility principle".to_string(),
                priority: Priority::High,
            });
        }
    }

    fn generate_context_aware_suggestions(&self, config: &Config, context: &ProjectContext, suggestions: &mut Vec<ConfigSuggestion>) {
        // Language-specific suggestions
        for language in &context.languages {
            match language.as_str() {
                "rust" => self.add_rust_specific_suggestions(config, suggestions),
                "python" => self.add_python_specific_suggestions(config, suggestions),
                "javascript" | "typescript" => self.add_js_ts_specific_suggestions(config, suggestions),
                _ => {}
            }
        }

        // Team size specific suggestions
        match context.team_size {
            TeamSize::Large => {
                suggestions.push(ConfigSuggestion {
                    category: SuggestionCategory::Maintenance,
                    title: "Strict Thresholds for Large Teams".to_string(),
                    description: "Large teams benefit from stricter code quality thresholds".to_string(),
                    before: None,
                    after: "# Lower thresholds for better maintainability\nmax_logical_loc = 150\nmax_methods = 15".to_string(),
                    benefit: "Prevent code complexity issues in large team environments".to_string(),
                    priority: Priority::Medium,
                });
            },
            TeamSize::Individual => {
                if config.dead_code.as_ref().map_or(true, |dc| dc.confidence_threshold.unwrap_or(0.8) < 0.9) {
                    suggestions.push(ConfigSuggestion {
                        category: SuggestionCategory::Performance,
                        title: "Higher Confidence for Solo Development".to_string(),
                        description: "Solo developers can use higher confidence thresholds".to_string(),
                        before: Some("confidence_threshold = 0.8".to_string()),
                        after: "confidence_threshold = 0.9".to_string(),
                        benefit: "Fewer false positives when you know your codebase well".to_string(),
                        priority: Priority::Low,
                    });
                }
            },
            _ => {}
        }

        // CI/CD specific suggestions
        if context.has_ci {
            suggestions.push(ConfigSuggestion {
                category: SuggestionCategory::Maintenance,
                title: "CI/CD Optimizations".to_string(),
                description: "Optimize configuration for continuous integration environments".to_string(),
                before: None,
                after: "# CI-friendly settings\ntimeout_seconds = 300\nparallel_analysis = true".to_string(),
                benefit: "Faster analysis in CI/CD pipelines".to_string(),
                priority: Priority::Medium,
            });
        }
    }

    fn add_rust_specific_suggestions(&self, _config: &Config, suggestions: &mut Vec<ConfigSuggestion>) {
        suggestions.push(ConfigSuggestion {
            category: SuggestionCategory::Accuracy,
            title: "Rust-Specific Patterns".to_string(),
            description: "Add Rust-specific ignore patterns for better analysis".to_string(),
            before: None,
            after: r#"ignore_patterns = [
    "target/**",
    "**/target/**", 
    "Cargo.lock"
]"#.to_string(),
            benefit: "Exclude Rust build artifacts and dependencies".to_string(),
            priority: Priority::High,
        });
    }

    fn add_python_specific_suggestions(&self, _config: &Config, suggestions: &mut Vec<ConfigSuggestion>) {
        suggestions.push(ConfigSuggestion {
            category: SuggestionCategory::Accuracy,
            title: "Python-Specific Patterns".to_string(),
            description: "Add Python-specific ignore patterns for better analysis".to_string(),
            before: None,
            after: r#"ignore_patterns = [
    "__pycache__/**",
    "*.pyc",
    ".venv/**",
    "venv/**"
]"#.to_string(),
            benefit: "Exclude Python bytecode and virtual environments".to_string(),
            priority: Priority::High,
        });
    }

    fn add_js_ts_specific_suggestions(&self, _config: &Config, suggestions: &mut Vec<ConfigSuggestion>) {
        suggestions.push(ConfigSuggestion {
            category: SuggestionCategory::Performance,
            title: "JavaScript/TypeScript Optimizations".to_string(),
            description: "Optimize for JavaScript/TypeScript projects".to_string(),
            before: None,
            after: r#"ignore_patterns = [
    "node_modules/**",
    "dist/**", 
    "build/**",
    "*.min.js"
]"#.to_string(),
            benefit: "Exclude dependencies and build artifacts".to_string(),
            priority: Priority::High,
        });
    }

    async fn validate_project_specific_settings(&self, config: &Config, project_path: &Path, _warnings: &mut Vec<ValidationWarning>, suggestions: &mut Vec<ConfigSuggestion>) {
        // Check if project has package.json but no Node.js patterns
        if project_path.join("package.json").exists() {
            if let Some(ref dc_config) = config.dead_code {
                if let Some(ref patterns) = dc_config.ignore_patterns {
                    if !patterns.iter().any(|p| p.contains("node_modules")) {
                        suggestions.push(ConfigSuggestion {
                            category: SuggestionCategory::Performance,
                            title: "Add Node.js Ignore Patterns".to_string(),
                            description: "Detected package.json - add Node.js specific ignore patterns".to_string(),
                            before: None,
                            after: r#"ignore_patterns = ["node_modules/**", "dist/**", "build/**"]"#.to_string(),
                            benefit: "Significantly faster analysis by excluding dependencies".to_string(),
                            priority: Priority::High,
                        });
                    }
                }
            }
        }

        // Check if project has Cargo.toml but no Rust patterns  
        if project_path.join("Cargo.toml").exists() {
            if let Some(ref dc_config) = config.dead_code {
                if let Some(ref patterns) = dc_config.ignore_patterns {
                    if !patterns.iter().any(|p| p.contains("target")) {
                        suggestions.push(ConfigSuggestion {
                            category: SuggestionCategory::Performance,
                            title: "Add Rust Ignore Patterns".to_string(),
                            description: "Detected Cargo.toml - add Rust specific ignore patterns".to_string(),
                            before: None,
                            after: r#"ignore_patterns = ["target/**", "**/target/**"]"#.to_string(),
                            benefit: "Exclude Rust build artifacts for faster analysis".to_string(),
                            priority: Priority::High,
                        });
                    }
                }
            }
        }
    }

    fn suggest_optimal_model(&self, context: &ProjectContext) -> String {
        match context.estimated_size {
            ProjectSize::Small => "deepseek-coder:6.7b-instruct-q4_0".to_string(),
            ProjectSize::Medium => "deepseek-coder:6.7b-instruct-q4_0".to_string(),
            ProjectSize::Large => "codellama:7b-instruct".to_string(),
            ProjectSize::Huge => "llama2:7b-chat".to_string(),
        }
    }

    fn suggest_optimal_thresholds(&self, context: &ProjectContext) -> OptimalThresholds {
        let base_thresholds = match context.estimated_size {
            ProjectSize::Small => OptimalThresholds { max_logical_loc: 200, max_methods: 20 },
            ProjectSize::Medium => OptimalThresholds { max_logical_loc: 250, max_methods: 25 },
            ProjectSize::Large => OptimalThresholds { max_logical_loc: 300, max_methods: 30 },
            ProjectSize::Huge => OptimalThresholds { max_logical_loc: 400, max_methods: 35 },
        };

        // Adjust for team size
        match context.team_size {
            TeamSize::Large => OptimalThresholds {
                max_logical_loc: (base_thresholds.max_logical_loc as f64 * 0.8) as u32,
                max_methods: (base_thresholds.max_methods as f64 * 0.8) as u32,
            },
            _ => base_thresholds,
        }
    }

    fn calculate_performance_score(&self, config: &Config, suggestions: &[ConfigSuggestion]) -> f64 {
        let mut score = 100.0;

        // Deduct points for missing performance optimizations
        let performance_suggestions = suggestions.iter()
            .filter(|s| matches!(s.category, SuggestionCategory::Performance))
            .count();
        
        score -= (performance_suggestions as f64 * 10.0).min(50.0);

        // Add points for having AI model configured
        if config.ollama_model.is_some() {
            score += 10.0;
        }

        // Add points for having ignore patterns
        if let Some(ref dc_config) = config.dead_code {
            if dc_config.ignore_patterns.as_ref().map_or(false, |p| !p.is_empty()) {
                score += 15.0;
            }
        }

        score.max(0.0).min(100.0)
    }

    fn calculate_completeness_score(&self, config: &Config) -> f64 {
        let mut score = 0.0;
        let total_possible = 100.0;

        // AI configuration (25 points)
        if config.ollama_model.is_some() {
            score += 25.0;
        }

        // Dead code configuration (35 points)
        if let Some(ref dc_config) = config.dead_code {
            score += 15.0; // Base points for having the section
            if dc_config.confidence_threshold.is_some() { score += 5.0; }
            if dc_config.library_mode.is_some() { score += 5.0; }
            if dc_config.ignore_patterns.as_ref().map_or(false, |p| !p.is_empty()) { score += 5.0; }
            if dc_config.keep_alive_patterns.as_ref().map_or(false, |p| !p.is_empty()) { score += 5.0; }
        }

        // Large classes configuration (40 points)
        if let Some(ref lc_config) = config.large_classes {
            score += 20.0; // Base points for having the section
            if lc_config.max_logical_loc.is_some() { score += 5.0; }
            if lc_config.max_methods.is_some() { score += 5.0; }
            if lc_config.max_fields.is_some() { score += 2.5; }
            if lc_config.max_cyclomatic_complexity.is_some() { score += 2.5; }
            if lc_config.ignore_patterns.as_ref().map_or(false, |p| !p.is_empty()) { score += 2.5; }
            if lc_config.language_overrides.as_ref().map_or(false, |o| !o.is_empty()) { score += 2.5; }
        }

        (score / total_possible * 100.0_f64).min(100.0_f64)
    }
}

struct OptimalThresholds {
    max_logical_loc: u32,
    max_methods: u32,
}

impl Default for SmartConfigValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to validate a configuration file and provide suggestions
pub async fn validate_config_file(path: &Path) -> Result<ValidationResult, UveddiError> {
    let config = Config::from_file(path.to_string_lossy().as_ref())?;
    let validator = SmartConfigValidator::new();
    validator.validate(&config, Some(path.parent().unwrap_or(path))).await
}

/// Helper function to validate a configuration with project context
pub async fn validate_config_with_context(config: &Config, context: ProjectContext, project_path: Option<&Path>) -> Result<ValidationResult, UveddiError> {
    let validator = SmartConfigValidator::new().with_context(context);
    validator.validate(config, project_path).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_empty_config_validation() {
        let config = Config {
            ollama_model: None,
            dead_code: None,
            large_classes: None,
        };
        
        let validator = SmartConfigValidator::new();
        let result = validator.validate(&config, None).await.unwrap();
        
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
        assert!(result.completeness_score < 50.0);
    }

    #[tokio::test]  
    async fn test_optimal_config_validation() {
        let config = Config {
            ollama_model: Some("deepseek-coder:6.7b-instruct-q4_0".to_string()),
            dead_code: Some(DeadCodeConfig {
                confidence_threshold: Some(0.8),
                library_mode: Some(false),
                ignore_patterns: Some(vec!["test/**".to_string()]),
                keep_alive_patterns: Some(vec!["main".to_string()]),
            }),
            large_classes: Some(LargeClassConfig {
                max_logical_loc: Some(250),
                max_methods: Some(25),
                max_fields: Some(20),
                max_cyclomatic_complexity: Some(15),
                max_cognitive_complexity: Some(20),
                max_lcom_score: Some(0.8),
                max_coupling: Some(30),
                ignore_patterns: Some(vec!["test/**".to_string()]),
                language_overrides: None,
            }),
        };
        
        let validator = SmartConfigValidator::new();
        let result = validator.validate(&config, None).await.unwrap();
        
        assert!(result.is_valid);
        assert!(result.completeness_score > 80.0);
        assert!(result.performance_score > 70.0);
    }
}