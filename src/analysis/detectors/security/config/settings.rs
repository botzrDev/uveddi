//! High-level configuration types for the security detector

use serde::{Deserialize, Serialize};

use crate::analysis::detectors::security::agents::config::MultiAgentConfig;

/// Top-level configuration for the security detector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub enable_taint_analysis: bool,
    pub enable_sca: bool,
    pub enable_ai_enhancement: bool,
    pub confidence_threshold: f64,
    pub max_issues_per_file: usize,
    pub enable_caching: bool,
    pub parallel_analysis: bool,
    pub timeout_seconds: u64,
    pub multi_agent: MultiAgentConfig,
    pub false_positive: FalsePositiveConfig,
    pub taint_analysis: TaintAnalysisConfig,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self::development()
    }
}

impl SecurityConfig {
    /// Production-ready configuration balancing precision and coverage
    pub fn production() -> Self {
        Self {
            enable_taint_analysis: true,
            enable_sca: true,
            enable_ai_enhancement: true,
            confidence_threshold: 0.7,
            max_issues_per_file: 50,
            enable_caching: true,
            parallel_analysis: true,
            timeout_seconds: 300,
            multi_agent: MultiAgentConfig::production(),
            false_positive: FalsePositiveConfig::balanced(),
            taint_analysis: TaintAnalysisConfig::production(),
        }
    }

    /// Developer-friendly configuration with relaxed thresholds
    pub fn development() -> Self {
        Self {
            enable_taint_analysis: true,
            enable_sca: false,
            enable_ai_enhancement: false,
            confidence_threshold: 0.5,
            max_issues_per_file: 100,
            enable_caching: true,
            parallel_analysis: true,
            timeout_seconds: 600,
            multi_agent: MultiAgentConfig::development(),
            false_positive: FalsePositiveConfig::minimal(),
            taint_analysis: TaintAnalysisConfig::development(),
        }
    }

    /// Configuration tailored for CI/CD pipelines
    pub fn ci_cd() -> Self {
        Self {
            enable_taint_analysis: true,
            enable_sca: true,
            enable_ai_enhancement: false,
            confidence_threshold: 0.8,
            max_issues_per_file: 30,
            enable_caching: false,
            parallel_analysis: true,
            timeout_seconds: 180,
            multi_agent: MultiAgentConfig::production(),
            false_positive: FalsePositiveConfig::aggressive(),
            taint_analysis: TaintAnalysisConfig::production(),
        }
    }

    /// Lightweight configuration for focused analysis
    pub fn minimal() -> Self {
        Self {
            enable_taint_analysis: true,
            enable_sca: false,
            enable_ai_enhancement: false,
            confidence_threshold: 0.6,
            max_issues_per_file: 20,
            enable_caching: true,
            parallel_analysis: false,
            timeout_seconds: 120,
            multi_agent: MultiAgentConfig::development(),
            false_positive: FalsePositiveConfig::minimal(),
            taint_analysis: TaintAnalysisConfig::development(),
        }
    }
}

/// Configuration for false positive mitigation strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FalsePositiveConfig {
    pub enable_heuristic_filtering: bool,
    pub enable_contextual_filtering: bool,
    pub enable_statistical_filtering: bool,
    pub enable_ml_filtering: bool,
    pub enable_bayesian_optimization: bool,
    pub exclude_test_files: bool,
    pub exclude_generated_files: bool,
    pub exclude_third_party: bool,
    pub min_lines_threshold: usize,
    pub min_tokens_threshold: usize,
    pub min_confidence_threshold: f64,
    pub max_similar_issues: usize,
    pub suppression_comments: Vec<String>,
}

impl FalsePositiveConfig {
    /// Balanced configuration for day-to-day development
    pub fn balanced() -> Self {
        Self {
            enable_heuristic_filtering: true,
            enable_contextual_filtering: true,
            enable_statistical_filtering: false,
            enable_ml_filtering: false,
            enable_bayesian_optimization: true,
            exclude_test_files: true,
            exclude_generated_files: false,
            exclude_third_party: true,
            min_lines_threshold: 5,
            min_tokens_threshold: 20,
            min_confidence_threshold: 0.5,
            max_similar_issues: 5,
            suppression_comments: default_suppression_comments(),
        }
    }

    /// Aggressive filtering for signal-focused environments
    pub fn aggressive() -> Self {
        Self {
            enable_heuristic_filtering: true,
            enable_contextual_filtering: true,
            enable_statistical_filtering: true,
            enable_ml_filtering: true,
            enable_bayesian_optimization: true,
            exclude_test_files: true,
            exclude_generated_files: true,
            exclude_third_party: true,
            min_lines_threshold: 10,
            min_tokens_threshold: 30,
            min_confidence_threshold: 0.7,
            max_similar_issues: 3,
            suppression_comments: default_suppression_comments(),
        }
    }

    /// Minimal filtering for exploratory analysis
    pub fn minimal() -> Self {
        Self {
            enable_heuristic_filtering: false,
            enable_contextual_filtering: true,
            enable_statistical_filtering: false,
            enable_ml_filtering: false,
            enable_bayesian_optimization: false,
            exclude_test_files: true,
            exclude_generated_files: false,
            exclude_third_party: false,
            min_lines_threshold: 3,
            min_tokens_threshold: 10,
            min_confidence_threshold: 0.3,
            max_similar_issues: 10,
            suppression_comments: default_suppression_comments(),
        }
    }

    /// Moderate configuration balancing noise reduction and recall
    pub fn moderate() -> Self {
        Self {
            enable_heuristic_filtering: true,
            enable_contextual_filtering: true,
            enable_statistical_filtering: true,
            enable_ml_filtering: false,
            enable_bayesian_optimization: true,
            exclude_test_files: true,
            exclude_generated_files: true,
            exclude_third_party: true,
            min_lines_threshold: 4,
            min_tokens_threshold: 16,
            min_confidence_threshold: 0.6,
            max_similar_issues: 4,
            suppression_comments: default_suppression_comments(),
        }
    }
}

fn default_suppression_comments() -> Vec<String> {
    vec![
        "UVEDDI:IGNORE".to_string(),
        "NOSEC".to_string(),
        "NOLINT".to_string(),
    ]
}

/// Configuration for the taint analysis engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaintAnalysisConfig {
    pub track_implicit_flows: bool,
    pub context_sensitive: bool,
    pub field_sensitive: bool,
    pub max_path_length: usize,
    pub max_iterations: usize,
    pub enable_sanitizers: bool,
    pub detect_second_order: bool,
}

impl Default for TaintAnalysisConfig {
    fn default() -> Self {
        Self::production()
    }
}

impl TaintAnalysisConfig {
    /// Production configuration with full precision
    pub fn production() -> Self {
        Self {
            track_implicit_flows: true,
            context_sensitive: true,
            field_sensitive: true,
            max_path_length: 100,
            max_iterations: 1000,
            enable_sanitizers: true,
            detect_second_order: true,
        }
    }

    /// Development configuration tuned for faster iterations
    pub fn development() -> Self {
        Self {
            track_implicit_flows: true,
            context_sensitive: true,
            field_sensitive: true,
            max_path_length: 80,
            max_iterations: 600,
            enable_sanitizers: true,
            detect_second_order: true,
        }
    }

    /// Configuration for constrained environments
    pub fn minimal() -> Self {
        Self {
            track_implicit_flows: false,
            context_sensitive: false,
            field_sensitive: true,
            max_path_length: 40,
            max_iterations: 300,
            enable_sanitizers: true,
            detect_second_order: false,
        }
    }
}

/// Language-specific configuration for taint sources, sinks, and sanitizers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageConfig {
    pub custom_sources: Vec<String>,
    pub custom_sinks: Vec<String>,
    pub custom_sanitizers: Vec<String>,
    pub enable_unsafe_analysis: bool,
    pub check_memory_safety: bool,
    pub detect_race_conditions: bool,
    pub check_type_confusion: bool,
    pub detect_deserialization: bool,
    pub analyze_imports: bool,
    pub detect_prototype_pollution: bool,
    pub check_dom_xss: bool,
    pub analyze_dependencies: bool,
}

impl LanguageConfig {
    /// Rust-specific language configuration
    pub fn rust() -> Self {
        Self {
            custom_sources: vec![
                "std::env::args".to_string(),
                "std::env::var".to_string(),
            ],
            custom_sinks: vec![
                "std::process::Command::new".to_string(),
                "sqlx::query".to_string(),
            ],
            custom_sanitizers: vec!["html_escape::encode".to_string()],
            enable_unsafe_analysis: true,
            check_memory_safety: true,
            detect_race_conditions: true,
            check_type_confusion: false,
            detect_deserialization: false,
            analyze_imports: false,
            detect_prototype_pollution: false,
            check_dom_xss: false,
            analyze_dependencies: true,
        }
    }

    /// Python-specific language configuration
    pub fn python() -> Self {
        Self {
            custom_sources: vec!["input(".to_string(), "sys.argv".to_string()],
            custom_sinks: vec!["eval(".to_string(), "os.system".to_string()],
            custom_sanitizers: vec!["html.escape".to_string()],
            enable_unsafe_analysis: false,
            check_memory_safety: false,
            detect_race_conditions: false,
            check_type_confusion: true,
            detect_deserialization: true,
            analyze_imports: true,
            detect_prototype_pollution: false,
            check_dom_xss: false,
            analyze_dependencies: true,
        }
    }

    /// JavaScript-specific language configuration
    pub fn javascript() -> Self {
        Self {
            custom_sources: vec!["req.query".to_string(), "req.body".to_string()],
            custom_sinks: vec!["eval(".to_string(), "document.write".to_string()],
            custom_sanitizers: vec!["DOMPurify.sanitize".to_string()],
            enable_unsafe_analysis: false,
            check_memory_safety: false,
            detect_race_conditions: false,
            check_type_confusion: false,
            detect_deserialization: false,
            analyze_imports: true,
            detect_prototype_pollution: true,
            check_dom_xss: true,
            analyze_dependencies: true,
        }
    }
}
