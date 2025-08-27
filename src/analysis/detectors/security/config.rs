//! Configuration structures for the security detection system
//!
//! This module provides comprehensive configuration options for all aspects
//! of the security detector, including multi-agent settings, detection thresholds,
//! false positive mitigation, and AI integration parameters.

use crate::ast::SourceLanguage;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main configuration for the security detector system
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable taint analysis for injection detection
    pub enable_taint_analysis: bool,

    /// Enable Software Composition Analysis (SCA) for dependency vulnerabilities
    pub enable_sca: bool,

    /// Enable AI-powered analysis enhancement
    pub enable_ai_enhancement: bool,

    /// Minimum confidence threshold for reporting issues (0.0 - 1.0)
    pub confidence_threshold: f64,

    /// Maximum number of issues to report per file
    pub max_issues_per_file: usize,

    /// Enable parallel analysis for performance
    pub enable_parallel_analysis: bool,

    /// Multi-agent system configuration
    pub multi_agent: MultiAgentConfig,

    /// Taint analysis specific configuration
    pub taint_analysis: TaintAnalysisConfig,

    /// False positive mitigation configuration
    pub false_positive_mitigation: FalsePositiveConfig,

    /// Validation engine configuration
    pub validation: ValidationConfig,

    /// Language-specific configurations
    pub language_configs: HashMap<SourceLanguage, LanguageSecurityConfig>,

    /// File and directory exclusion patterns
    pub exclusions: ExclusionPatterns,

    /// Integration settings
    pub integrations: IntegrationConfig,
}

impl SecurityConfig {
    /// Create a new configuration with production-ready defaults
    pub fn production() -> Self {
        Self {
            enable_taint_analysis: true,
            enable_sca: true,
            enable_ai_enhancement: true,
            confidence_threshold: 0.7,
            max_issues_per_file: 50,
            enable_parallel_analysis: true,
            multi_agent: MultiAgentConfig::production(),
            taint_analysis: TaintAnalysisConfig::production(),
            false_positive_mitigation: FalsePositiveConfig::aggressive(),
            validation: ValidationConfig::strict(),
            language_configs: Self::default_language_configs(),
            exclusions: ExclusionPatterns::default(),
            integrations: IntegrationConfig::default(),
        }
    }

    /// Create a development-friendly configuration with faster analysis
    pub fn development() -> Self {
        Self {
            enable_taint_analysis: true,
            enable_sca: false,            // Disabled for faster development cycles
            enable_ai_enhancement: false, // Disabled for faster analysis
            confidence_threshold: 0.5,
            max_issues_per_file: 20,
            enable_parallel_analysis: true,
            multi_agent: MultiAgentConfig::development(),
            taint_analysis: TaintAnalysisConfig::development(),
            false_positive_mitigation: FalsePositiveConfig::moderate(),
            validation: ValidationConfig::permissive(),
            language_configs: Self::default_language_configs(),
            exclusions: ExclusionPatterns::development(),
            integrations: IntegrationConfig::minimal(),
        }
    }

    /// Create configuration optimized for CI/CD environments
    pub fn ci_cd() -> Self {
        Self {
            enable_taint_analysis: true,
            enable_sca: true,
            enable_ai_enhancement: false, // Disabled for deterministic results
            confidence_threshold: 0.8,    // Higher threshold for CI/CD
            max_issues_per_file: 100,
            enable_parallel_analysis: true,
            multi_agent: MultiAgentConfig::ci_cd(),
            taint_analysis: TaintAnalysisConfig::ci_cd(),
            false_positive_mitigation: FalsePositiveConfig::strict(),
            validation: ValidationConfig::strict(),
            language_configs: Self::default_language_configs(),
            exclusions: ExclusionPatterns::ci_cd(),
            integrations: IntegrationConfig::ci_cd(),
        }
    }

    fn default_language_configs() -> HashMap<SourceLanguage, LanguageSecurityConfig> {
        let mut configs = HashMap::new();

        // Rust configuration - memory safety focus
        configs.insert(
            SourceLanguage::Rust,
            LanguageSecurityConfig {
                enable_unsafe_analysis: true,
                enable_crypto_analysis: true,
                enable_deserialization_analysis: true,
                custom_patterns: vec![
                    "use std::mem::transmute".to_string(),
                    "unsafe {".to_string(),
                    ".unwrap()".to_string(), // Potential panic sources
                ],
                taint_sources: vec![
                    "std::env::args".to_string(),
                    "std::env::var".to_string(),
                    "std::fs::read_to_string".to_string(),
                    "tokio::io::AsyncReadExt::read".to_string(),
                ],
                taint_sinks: vec![
                    "std::process::Command::new".to_string(),
                    "sqlx::query".to_string(),
                    "std::fs::write".to_string(),
                ],
            },
        );

        // Python configuration - injection and deserialization focus
        configs.insert(
            SourceLanguage::Python,
            LanguageSecurityConfig {
                enable_unsafe_analysis: false,
                enable_crypto_analysis: true,
                enable_deserialization_analysis: true,
                custom_patterns: vec![
                    "eval(".to_string(),
                    "exec(".to_string(),
                    "subprocess.".to_string(),
                    "os.system".to_string(),
                ],
                taint_sources: vec![
                    "input(".to_string(),
                    "sys.argv".to_string(),
                    "request.".to_string(),
                    "open(".to_string(),
                ],
                taint_sinks: vec![
                    "cursor.execute".to_string(),
                    "os.system".to_string(),
                    "subprocess.".to_string(),
                    "eval(".to_string(),
                ],
            },
        );

        // JavaScript/TypeScript configuration - web security focus
        configs.insert(
            SourceLanguage::JavaScript,
            LanguageSecurityConfig {
                enable_unsafe_analysis: false,
                enable_crypto_analysis: true,
                enable_deserialization_analysis: true,
                custom_patterns: vec![
                    "eval(".to_string(),
                    "innerHTML =".to_string(),
                    "document.write".to_string(),
                    "setTimeout(".to_string(),
                ],
                taint_sources: vec![
                    "req.params".to_string(),
                    "req.query".to_string(),
                    "req.body".to_string(),
                    "location.search".to_string(),
                ],
                taint_sinks: vec![
                    "innerHTML".to_string(),
                    "document.write".to_string(),
                    "eval(".to_string(),
                    "setTimeout(".to_string(),
                ],
            },
        );

        configs.insert(
            SourceLanguage::TypeScript,
            configs[&SourceLanguage::JavaScript].clone(),
        );

        configs
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self::production()
    }
}

/// Configuration for the multi-agent security analysis system
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultiAgentConfig {
    /// Enable the orchestrator agent
    pub enable_orchestrator: bool,

    /// Enable taint analysis agent
    pub enable_taint_agent: bool,

    /// Enable configuration analysis agent
    pub enable_config_agent: bool,

    /// Enable dependency analysis agent (SCA)
    pub enable_dependency_agent: bool,

    /// Enable validation agent for cross-validation
    pub enable_validation_agent: bool,

    /// Enable AI enhancement agent
    pub enable_ai_agent: bool,

    /// Maximum number of concurrent agents
    pub max_concurrent_agents: usize,

    /// Agent timeout in seconds
    pub agent_timeout_seconds: u64,

    /// Inter-agent communication configuration
    pub communication: AgentCommunicationConfig,

    /// Knowledge graph integration settings
    pub knowledge_graph_integration: bool,
}

impl MultiAgentConfig {
    pub fn production() -> Self {
        Self {
            enable_orchestrator: true,
            enable_taint_agent: true,
            enable_config_agent: true,
            enable_dependency_agent: true,
            enable_validation_agent: true,
            enable_ai_agent: true,
            max_concurrent_agents: 4,
            agent_timeout_seconds: 300,
            communication: AgentCommunicationConfig::default(),
            knowledge_graph_integration: true,
        }
    }

    pub fn development() -> Self {
        Self {
            enable_orchestrator: true,
            enable_taint_agent: true,
            enable_config_agent: false,     // Disabled for speed
            enable_dependency_agent: false, // Disabled for speed
            enable_validation_agent: false, // Disabled for speed
            enable_ai_agent: false,         // Disabled for speed
            max_concurrent_agents: 2,
            agent_timeout_seconds: 60,
            communication: AgentCommunicationConfig::development(),
            knowledge_graph_integration: false,
        }
    }

    pub fn ci_cd() -> Self {
        Self {
            enable_orchestrator: true,
            enable_taint_agent: true,
            enable_config_agent: true,
            enable_dependency_agent: true,
            enable_validation_agent: true,
            enable_ai_agent: false,     // Disabled for deterministic results
            max_concurrent_agents: 8,   // Higher for CI/CD performance
            agent_timeout_seconds: 600, // Longer timeout for large codebases
            communication: AgentCommunicationConfig::default(),
            knowledge_graph_integration: false, // Disabled for CI/CD speed
        }
    }
}

/// Agent communication configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentCommunicationConfig {
    /// Enable message passing between agents
    pub enable_message_passing: bool,

    /// Maximum message queue size per agent
    pub max_queue_size: usize,

    /// Message timeout in milliseconds
    pub message_timeout_ms: u64,

    /// Enable result sharing between agents
    pub enable_result_sharing: bool,
}

impl AgentCommunicationConfig {
    pub fn development() -> Self {
        Self {
            enable_message_passing: false,
            max_queue_size: 10,
            message_timeout_ms: 1000,
            enable_result_sharing: false,
        }
    }
}

impl Default for AgentCommunicationConfig {
    fn default() -> Self {
        Self {
            enable_message_passing: true,
            max_queue_size: 100,
            message_timeout_ms: 5000,
            enable_result_sharing: true,
        }
    }
}

/// Configuration for individual agents
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentConfig {
    pub name: String,
    pub enabled: bool,
    pub priority: i32,
    pub timeout_seconds: u64,
    pub max_memory_mb: usize,
    pub custom_parameters: HashMap<String, serde_json::Value>,
}

impl AgentConfig {
    pub fn new(name: String) -> Self {
        Self {
            name,
            enabled: true,
            priority: 0,
            timeout_seconds: 60,
            max_memory_mb: 512,
            custom_parameters: HashMap::new(),
        }
    }
}

/// Taint analysis specific configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaintAnalysisConfig {
    /// Enable inter-procedural analysis
    pub enable_interprocedural: bool,

    /// Enable field-sensitive analysis
    pub enable_field_sensitive: bool,

    /// Enable path-sensitive analysis
    pub enable_path_sensitive: bool,

    /// Maximum analysis depth
    pub max_depth: usize,

    /// Maximum number of paths to explore
    pub max_paths: usize,

    /// Enable sanitization detection
    pub enable_sanitizer_detection: bool,

    /// Custom taint sources (language-specific patterns will be added)
    pub custom_sources: Vec<String>,

    /// Custom taint sinks (language-specific patterns will be added)
    pub custom_sinks: Vec<String>,

    /// Custom sanitizers
    pub custom_sanitizers: Vec<String>,
}

impl TaintAnalysisConfig {
    pub fn production() -> Self {
        Self {
            enable_interprocedural: true,
            enable_field_sensitive: true,
            enable_path_sensitive: false, // Too expensive for production
            max_depth: 10,
            max_paths: 1000,
            enable_sanitizer_detection: true,
            custom_sources: Vec::new(),
            custom_sinks: Vec::new(),
            custom_sanitizers: Vec::new(),
        }
    }

    pub fn development() -> Self {
        Self {
            enable_interprocedural: true,
            enable_field_sensitive: false, // Disabled for speed
            enable_path_sensitive: false,
            max_depth: 5,
            max_paths: 100,
            enable_sanitizer_detection: true,
            custom_sources: Vec::new(),
            custom_sinks: Vec::new(),
            custom_sanitizers: Vec::new(),
        }
    }

    pub fn ci_cd() -> Self {
        Self::production()
    }
}

/// False positive mitigation configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FalsePositiveConfig {
    /// Enable heuristic filtering
    pub enable_heuristic_filtering: bool,

    /// Enable contextual filtering
    pub enable_contextual_filtering: bool,

    /// Enable statistical frequency analysis
    pub enable_statistical_filtering: bool,

    /// Minimum code size thresholds
    pub min_lines_threshold: usize,
    pub min_tokens_threshold: usize,

    /// File exclusion patterns
    pub exclude_test_files: bool,
    pub exclude_generated_files: bool,
    pub exclude_third_party: bool,

    /// Enable Bayesian optimization for adaptive tuning
    pub enable_bayesian_optimization: bool,

    /// Suppression comment patterns
    pub suppression_comments: Vec<String>,

    /// Context-aware rules
    pub context_rules: HashMap<String, Vec<String>>,
}

impl FalsePositiveConfig {
    pub fn aggressive() -> Self {
        Self {
            enable_heuristic_filtering: true,
            enable_contextual_filtering: true,
            enable_statistical_filtering: true,
            min_lines_threshold: 5,
            min_tokens_threshold: 10,
            exclude_test_files: true,
            exclude_generated_files: true,
            exclude_third_party: true,
            enable_bayesian_optimization: true,
            suppression_comments: vec![
                "UVEDDI:IGNORE".to_string(),
                "SECURITY:OK".to_string(),
                "FALSE-POSITIVE".to_string(),
            ],
            context_rules: HashMap::new(),
        }
    }

    pub fn moderate() -> Self {
        Self {
            enable_heuristic_filtering: true,
            enable_contextual_filtering: false,
            enable_statistical_filtering: false,
            min_lines_threshold: 3,
            min_tokens_threshold: 5,
            exclude_test_files: true,
            exclude_generated_files: true,
            exclude_third_party: false,
            enable_bayesian_optimization: false,
            suppression_comments: vec!["UVEDDI:IGNORE".to_string()],
            context_rules: HashMap::new(),
        }
    }

    pub fn strict() -> Self {
        Self {
            enable_heuristic_filtering: false,
            enable_contextual_filtering: false,
            enable_statistical_filtering: false,
            min_lines_threshold: 1,
            min_tokens_threshold: 1,
            exclude_test_files: false,
            exclude_generated_files: true, // Still exclude generated files
            exclude_third_party: false,
            enable_bayesian_optimization: false,
            suppression_comments: Vec::new(),
            context_rules: HashMap::new(),
        }
    }
}

/// Validation engine configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationConfig {
    /// Enable cross-validation between detectors
    pub enable_cross_validation: bool,

    /// Enable AI hallucination detection
    pub enable_hallucination_detection: bool,

    /// Enable confidence scoring
    pub enable_confidence_scoring: bool,

    /// Minimum number of detectors that must agree
    pub min_detector_agreement: usize,

    /// Enable grounding with knowledge graph
    pub enable_knowledge_graph_grounding: bool,

    /// Validation timeout in seconds
    pub validation_timeout_seconds: u64,
}

impl ValidationConfig {
    pub fn strict() -> Self {
        Self {
            enable_cross_validation: true,
            enable_hallucination_detection: true,
            enable_confidence_scoring: true,
            min_detector_agreement: 2,
            enable_knowledge_graph_grounding: true,
            validation_timeout_seconds: 30,
        }
    }

    pub fn permissive() -> Self {
        Self {
            enable_cross_validation: false,
            enable_hallucination_detection: false,
            enable_confidence_scoring: true,
            min_detector_agreement: 1,
            enable_knowledge_graph_grounding: false,
            validation_timeout_seconds: 10,
        }
    }
}

/// Language-specific security configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LanguageSecurityConfig {
    /// Enable unsafe code analysis (relevant for Rust)
    pub enable_unsafe_analysis: bool,

    /// Enable cryptographic library analysis
    pub enable_crypto_analysis: bool,

    /// Enable deserialization vulnerability analysis
    pub enable_deserialization_analysis: bool,

    /// Custom security patterns to detect
    pub custom_patterns: Vec<String>,

    /// Language-specific taint sources
    pub taint_sources: Vec<String>,

    /// Language-specific taint sinks
    pub taint_sinks: Vec<String>,
}

/// File and directory exclusion patterns
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExclusionPatterns {
    /// File patterns to exclude
    pub file_patterns: Vec<String>,

    /// Directory patterns to exclude
    pub directory_patterns: Vec<String>,

    /// Language-specific exclusions
    pub language_exclusions: HashMap<SourceLanguage, Vec<String>>,
}

impl ExclusionPatterns {
    pub fn development() -> Self {
        Self {
            file_patterns: vec![
                "**/test/**".to_string(),
                "**/tests/**".to_string(),
                "**/*_test.rs".to_string(),
                "**/*_test.py".to_string(),
                "**/*.test.js".to_string(),
                "**/*.test.ts".to_string(),
                "**/node_modules/**".to_string(),
                "**/target/**".to_string(),
                "**/.git/**".to_string(),
            ],
            directory_patterns: vec![
                "node_modules".to_string(),
                "target".to_string(),
                ".git".to_string(),
                "__pycache__".to_string(),
            ],
            language_exclusions: HashMap::new(),
        }
    }

    pub fn ci_cd() -> Self {
        Self {
            file_patterns: vec![
                "**/node_modules/**".to_string(),
                "**/target/**".to_string(),
                "**/.git/**".to_string(),
            ],
            directory_patterns: vec![
                "node_modules".to_string(),
                "target".to_string(),
                ".git".to_string(),
            ],
            language_exclusions: HashMap::new(),
        }
    }
}

impl Default for ExclusionPatterns {
    fn default() -> Self {
        Self {
            file_patterns: vec![
                "**/node_modules/**".to_string(),
                "**/target/**".to_string(),
                "**/.git/**".to_string(),
                "**/__pycache__/**".to_string(),
            ],
            directory_patterns: vec![
                "node_modules".to_string(),
                "target".to_string(),
                ".git".to_string(),
                "__pycache__".to_string(),
            ],
            language_exclusions: HashMap::new(),
        }
    }
}

/// Integration configuration for external systems
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntegrationConfig {
    /// Enable SARIF report generation
    pub enable_sarif_output: bool,

    /// Enable integration with vulnerability databases
    pub enable_vulnerability_db: bool,

    /// Enable CI/CD integration features
    pub enable_ci_integration: bool,

    /// External API configurations
    pub external_apis: HashMap<String, ApiConfig>,

    /// Webhook configurations for notifications
    pub webhooks: Vec<WebhookConfig>,
}

impl IntegrationConfig {
    pub fn minimal() -> Self {
        Self {
            enable_sarif_output: false,
            enable_vulnerability_db: false,
            enable_ci_integration: false,
            external_apis: HashMap::new(),
            webhooks: Vec::new(),
        }
    }

    pub fn ci_cd() -> Self {
        Self {
            enable_sarif_output: true,
            enable_vulnerability_db: true,
            enable_ci_integration: true,
            external_apis: HashMap::new(),
            webhooks: Vec::new(),
        }
    }
}

impl Default for IntegrationConfig {
    fn default() -> Self {
        Self::minimal()
    }
}

/// External API configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApiConfig {
    pub endpoint: String,
    pub api_key: Option<String>,
    pub timeout_seconds: u64,
    pub retry_attempts: usize,
}

/// Webhook configuration for notifications
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub name: String,
    pub url: String,
    pub events: Vec<String>,
    pub secret: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_security_config() {
        let config = SecurityConfig::default();
        assert!(config.enable_taint_analysis);
        assert!(config.enable_sca);
        assert_eq!(config.confidence_threshold, 0.7);
    }

    #[test]
    fn test_production_config() {
        let config = SecurityConfig::production();
        assert!(config.enable_ai_enhancement);
        assert!(config.multi_agent.enable_validation_agent);
        assert_eq!(config.confidence_threshold, 0.7);
    }

    #[test]
    fn test_development_config() {
        let config = SecurityConfig::development();
        assert!(!config.enable_sca); // Should be disabled for speed
        assert!(!config.enable_ai_enhancement); // Should be disabled for speed
        assert_eq!(config.confidence_threshold, 0.5);
    }

    #[test]
    fn test_ci_cd_config() {
        let config = SecurityConfig::ci_cd();
        assert!(!config.enable_ai_enhancement); // Should be disabled for determinism
        assert_eq!(config.confidence_threshold, 0.8); // Higher threshold
        assert_eq!(config.multi_agent.max_concurrent_agents, 8);
    }

    #[test]
    fn test_false_positive_config_levels() {
        let aggressive = FalsePositiveConfig::aggressive();
        let moderate = FalsePositiveConfig::moderate();
        let strict = FalsePositiveConfig::strict();

        assert!(aggressive.enable_heuristic_filtering);
        assert!(moderate.enable_heuristic_filtering);
        assert!(!strict.enable_heuristic_filtering);

        assert!(aggressive.enable_statistical_filtering);
        assert!(!moderate.enable_statistical_filtering);
        assert!(!strict.enable_statistical_filtering);
    }

    #[test]
    fn test_language_specific_configs() {
        let config = SecurityConfig::default();

        // Check Rust configuration
        let rust_config = &config.language_configs[&SourceLanguage::Rust];
        assert!(rust_config.enable_unsafe_analysis);
        assert!(rust_config
            .taint_sources
            .contains(&"std::env::args".to_string()));

        // Check Python configuration
        let python_config = &config.language_configs[&SourceLanguage::Python];
        assert!(!python_config.enable_unsafe_analysis);
        assert!(python_config.custom_patterns.contains(&"eval(".to_string()));
    }
}
