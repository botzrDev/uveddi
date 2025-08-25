//! Plugin Development Framework
//!
//! This module provides the development framework and APIs for creating
//! custom knowledge plugins, including traits, macros, and utilities.

#[cfg(feature = "ai")]
use crate::ai::knowledge::compression::CompressedString;
#[cfg(feature = "ai")]
use crate::ai::knowledge::context_selection::{LocationContext, SeverityLevel};
#[cfg(feature = "ai")]
use crate::ai::knowledge::schema::*;
use crate::plugins::knowledge::*;
use crate::plugins::PluginError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn, error, debug};

// Stub types for when AI features are disabled
#[cfg(not(feature = "ai"))]
pub use crate::plugins::integration::AntiPatternCategory;
#[cfg(not(feature = "ai"))]
pub use crate::plugins::integration::DetectionMethod;
#[cfg(not(feature = "ai"))]
pub use crate::plugins::integration::LocationContext;
#[cfg(not(feature = "ai"))]
pub use crate::plugins::integration::PatternKnowledge;
#[cfg(not(feature = "ai"))]
pub use crate::plugins::integration::SeverityLevel;
#[cfg(not(feature = "ai"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedString(String);
#[cfg(not(feature = "ai"))]
impl CompressedString {
    pub fn new(s: &str) -> Self {
        Self(s.to_string())
    }
}
#[cfg(not(feature = "ai"))]
pub use crate::plugins::integration::SolutionPattern;
#[cfg(not(feature = "ai"))]
pub use crate::plugins::integration::SourceLanguage;
#[cfg(not(feature = "ai"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImpactLevel {
    Low,
    Medium,
    High,
}
#[cfg(not(feature = "ai"))]
pub use crate::plugins::integration::LanguageKnowledge;
#[cfg(not(feature = "ai"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EffortLevel {
    Low,
    Medium,
    High,
}
#[cfg(not(feature = "ai"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeExamples {
    pub good: Vec<String>,
    pub bad: Vec<String>,
}

/// Plugin development macros for easy plugin creation
#[macro_export]
macro_rules! define_knowledge_plugin {
    (
        name: $name:expr,
        version: $version:expr,
        author: $author:expr,
        description: $description:expr,
        plugin_type: $plugin_type:expr,
        supported_languages: [$($lang:expr),*],
        implementation: $impl_type:ty
    ) => {
        pub fn create_plugin() -> Box<dyn $crate::plugins::knowledge::KnowledgePlugin> {
            let metadata = $crate::plugins::knowledge::KnowledgePluginMetadata {
                id: format!("{}-{}", $name.replace(" ", "-").to_lowercase(), $version),
                name: $name.to_string(),
                version: $version.to_string(),
                author: $author.to_string(),
                description: $description.to_string(),
                plugin_type: $plugin_type,
                supported_languages: vec![$($lang),*],
                dependencies: vec![],
                api_version: "1.0.0".to_string(),
                capabilities: $crate::plugins::knowledge::KnowledgePluginCapabilities::default(),
                permissions: $crate::plugins::knowledge::KnowledgePluginPermissions::default(),
            };

            Box::new(<$impl_type>::new(metadata))
        }
    };
}

/// Example anti-pattern plugin implementation
pub struct ExampleAntiPatternPlugin {
    metadata: KnowledgePluginMetadata,
    patterns: Vec<PatternKnowledge>,
    initialized: bool,
}

#[async_trait]
impl KnowledgePlugin for ExampleAntiPatternPlugin {
    fn metadata(&self) -> &KnowledgePluginMetadata {
        &self.metadata
    }

    async fn initialize(&mut self) -> Result<(), PluginError> {
        if self.initialized {
            return Ok(());
        }

        // Load custom patterns
        self.patterns = self.load_custom_patterns().await?;
        self.initialized = true;

        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), PluginError> {
        self.patterns.clear();
        self.initialized = false;
        Ok(())
    }

    async fn get_patterns(&self) -> Result<Vec<PatternKnowledge>, PluginError> {
        if !self.initialized {
            return Err(PluginError::Execution("Plugin not initialized".to_string()));
        }

        Ok(self.patterns.clone())
    }

    async fn get_detectors(&self) -> Result<Vec<DetectionMethod>, PluginError> {
        // Return plugin-specific detection methods
        Ok(vec![])
    }

    async fn get_solutions(&self) -> Result<Vec<SolutionPattern>, PluginError> {
        // Return plugin-specific solutions
        Ok(vec![])
    }

    async fn get_language_knowledge(
        &self,
        _language: SourceLanguage,
    ) -> Result<Option<LanguageKnowledge>, PluginError> {
        Ok(None)
    }

    async fn get_framework_knowledge(
        &self,
        _framework: &str,
    ) -> Result<Option<crate::plugins::knowledge::FrameworkKnowledge>, PluginError> {
        Ok(None)
    }

    async fn health_check(&self) -> Result<PluginHealthStatus, PluginError> {
        Ok(PluginHealthStatus::Healthy)
    }
}

impl ExampleAntiPatternPlugin {
    pub fn new(metadata: KnowledgePluginMetadata) -> Self {
        Self {
            metadata,
            patterns: vec![],
            initialized: false,
        }
    }

    async fn load_custom_patterns(&self) -> Result<Vec<PatternKnowledge>, PluginError> {
        use std::collections::HashMap;

        // Example: Load patterns from plugin-specific source
        // TODO: Fix PatternKnowledge struct fields - using available fields only
        #[cfg(feature = "ai")]
        let custom_pattern = PatternKnowledge {
            id: "singleton_abuse".to_string(),
            name: "Singleton Abuse".to_string(),
            definition: CompressedString::new(
                "Overuse of singleton pattern causing tight coupling",
            ),
            symptoms: vec![CompressedString::new("Global state management")],
            impact: crate::ai::knowledge::schema::ImpactLevel::Medium,
            category: AntiPatternCategory::ObjectOriented,
            detection_methods: vec![],
            solutions: vec![],
            examples: crate::ai::knowledge::schema::CodeExamples {
                primary: vec![],
                variations: HashMap::new(),
            },
            language_variations: HashMap::new(),
            related_patterns: vec![],
            tags: vec!["singleton".to_string()],
            frequency_score: 0.7,
            detection_confidence: 0.8,
        };

        #[cfg(not(feature = "ai"))]
        let custom_pattern = PatternKnowledge {
            id: "singleton_abuse".to_string(),
            category: AntiPatternCategory::GlobalState,
            detection_confidence: 0.8,
            tags: vec!["singleton".to_string()],
            solutions: vec![],
            detection_methods: vec![],
        };

        Ok(vec![custom_pattern])
    }
}

// Temporarily commented out complex pattern
/*
        let _complex_pattern = PatternKnowledge {
            id: "custom_singleton_abuse".to_string(),
            name: "Singleton Abuse".to_string(),
            definition: CompressedString::new(
                "Overuse of singleton pattern leading to hidden dependencies and testing difficulties"
            ),
            symptoms: vec![
                CompressedString::new("Multiple singleton instances in codebase"),
                CompressedString::new("Singletons with mutable state"),
                CompressedString::new("Difficulty in unit testing due to global state"),
            ],
            impact: ImpactLevel::High,
            category: AntiPatternCategory::ObjectOriented,
            detection_methods: vec![
                DetectionMethod::MetricThreshold {
                    metric_name: "singleton_count".to_string(),
                    threshold: 3.0,
                    operator: ComparisonOperator::GreaterThan,
                }
            ],
            solutions: vec![
                SolutionPattern {
                    id: "dependency_injection_solution".to_string(),
                    title: "Dependency Injection".to_string(),
                    implementation: CompressedString::new(
                        "Replace singletons with dependency injection:\n\
                        1. Identify singleton dependencies\n\
                        2. Create interfaces for singleton services\n\
                        3. Implement dependency injection container\n\
                        4. Refactor code to use injected dependencies\n\
                        Benefits: Improved testability, reduced coupling, better separation of concerns"
                    ),
                    examples: vec![],
                    effort_level: EffortLevel::Medium,
                    prerequisites: vec!["Understanding of dependency injection patterns".to_string()],
                    expected_impact: ImpactLevel::High,
                }
            ],
            examples: CodeExamples {
                primary: vec![],
                variations: HashMap::new(),
            },
            language_variations: HashMap::new(),
            related_patterns: vec!["god_object".to_string()],
            tags: vec!["singleton".to_string(), "dependency-injection".to_string()],
            frequency_score: 0.6,
            detection_confidence: 0.8,
        };

        Ok(vec![custom_pattern])
    }
}

/// Example enterprise plugin for organization-specific patterns
pub struct ExampleEnterprisePlugin {
    metadata: KnowledgePluginMetadata,
    org_patterns: HashMap<String, Vec<PatternKnowledge>>,
    compliance_knowledge: HashMap<String, ComplianceKnowledge>,
    initialized: bool,
}

#[async_trait]
impl KnowledgePlugin for ExampleEnterprisePlugin {
    fn metadata(&self) -> &KnowledgePluginMetadata {
        &self.metadata
    }

    async fn initialize(&mut self) -> Result<(), PluginError> {
        if self.initialized {
            return Ok(());
        }

        // Load organization-specific patterns and compliance knowledge
        self.load_enterprise_knowledge().await?;
        self.initialized = true;

        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), PluginError> {
        self.org_patterns.clear();
        self.compliance_knowledge.clear();
        self.initialized = false;
        Ok(())
    }

    async fn get_patterns(&self) -> Result<Vec<PatternKnowledge>, PluginError> {
        if !self.initialized {
            return Err(PluginError::Execution("Plugin not initialized".to_string()));
        }

        // Return all organization patterns
        let mut all_patterns = Vec::new();
        for patterns in self.org_patterns.values() {
            all_patterns.extend(patterns.clone());
        }
        Ok(all_patterns)
    }

    async fn get_detectors(&self) -> Result<Vec<DetectionMethod>, PluginError> {
        Ok(vec![])
    }

    async fn get_solutions(&self) -> Result<Vec<SolutionPattern>, PluginError> {
        Ok(vec![])
    }

    async fn get_language_knowledge(
        &self,
        _language: SourceLanguage,
    ) -> Result<Option<LanguageKnowledge>, PluginError> {
        Ok(None)
    }

    async fn get_framework_knowledge(
        &self,
        _framework: &str,
    ) -> Result<Option<crate::plugins::knowledge::FrameworkKnowledge>, PluginError> {
        Ok(None)
    }

    async fn health_check(&self) -> Result<PluginHealthStatus, PluginError> {
        Ok(PluginHealthStatus::Healthy)
    }
}

#[async_trait]
impl EnterprisePlugin for ExampleEnterprisePlugin {
    async fn get_organization_patterns(
        &self,
        org_id: &str,
    ) -> Result<Vec<PatternKnowledge>, PluginError> {
        if let Some(patterns) = self.org_patterns.get(org_id) {
            Ok(patterns.clone())
        } else {
            Ok(vec![])
        }
    }

    async fn get_compliance_knowledge(
        &self,
        compliance_framework: &str,
    ) -> Result<ComplianceKnowledge, PluginError> {
        if let Some(knowledge) = self.compliance_knowledge.get(compliance_framework) {
            Ok(knowledge.clone())
        } else {
            Err(PluginError::Execution(format!(
                "Compliance framework '{}' not supported",
                compliance_framework
            )))
        }
    }

    async fn validate_against_policies(
        &self,
        code: &str,
        policies: &[OrganizationPolicy],
    ) -> Result<PolicyValidationResult, PluginError> {
        let mut violations = Vec::new();
        let mut compliant_count = 0;

        for policy in policies {
            // Simple example validation - in practice this would be more sophisticated
            if self.check_policy_compliance(code, policy) {
                compliant_count += 1;
            } else {
                violations.push(PolicyViolation {
                    policy_id: policy.id.clone(),
                    description: format!("Code violates policy: {}", policy.name),
                    severity: match policy.enforcement {
                        EnforcementLevel::Warning => {
                            SeverityLevel::Low
                        }
                        EnforcementLevel::Error => {
                            SeverityLevel::Medium
                        }
                        EnforcementLevel::Blocking => {
                            SeverityLevel::High
                        }
                    },
                    location: LocationContext {
                        file_path: "unknown".to_string(),
                        line_range: (0, 0),
                        context_name: None,
                    },
                });
            }
        }

        let score = if policies.is_empty() {
            1.0
        } else {
            compliant_count as f32 / policies.len() as f32
        };

        Ok(PolicyValidationResult {
            compliant: violations.is_empty(),
            violations,
            score,
        })
    }
}

impl ExampleEnterprisePlugin {
    pub fn new(metadata: KnowledgePluginMetadata) -> Self {
        Self {
            metadata,
            org_patterns: HashMap::new(),
            compliance_knowledge: HashMap::new(),
            initialized: false,
        }
    }

    async fn load_enterprise_knowledge(&mut self) -> Result<(), PluginError> {
        #[cfg(feature = "ai")]
        use crate::ai::knowledge::compression::CompressedString;
        #[cfg(not(feature = "ai"))]
        use crate::plugins::development::CompressedString;

        // Example organization patterns
        let org_pattern = PatternKnowledge {
            id: "enterprise_logging_standard".to_string(),
            name: "Enterprise Logging Standard Violation".to_string(),
            definition: CompressedString::new(
                "Code that doesn't follow the organization's logging standards",
            ),
            symptoms: vec![
                CompressedString::new("Use of println! instead of structured logging"),
                CompressedString::new("Missing correlation IDs in log messages"),
                CompressedString::new("Inconsistent log levels"),
            ],
            impact: ImpactLevel::Medium,
            category: AntiPatternCategory::Maintainability,
            detection_methods: vec![DetectionMethod::RegexPattern {
                pattern: r"println!\s*\(".to_string(),
                context: "logging_standard_violation".to_string(),
            }],
            solutions: vec![SolutionPattern {
                id: "structured_logging_solution".to_string(),
                title: "Structured Logging".to_string(),
                implementation: CompressedString::new(
                    "Use organization's logging framework:\n\
                        1. Replace println! with log macros\n\
                        2. Add correlation IDs to all log messages\n\
                        3. Use appropriate log levels\n\
                        Benefits: Better observability, consistent log format, easier debugging",
                ),
                examples: vec![],
                effort_level: EffortLevel::Low,
                prerequisites: vec!["Access to organization's logging framework".to_string()],
                expected_impact: ImpactLevel::Medium,
            }],
            examples: CodeExamples {
                primary: vec![],
                variations: HashMap::new(),
            },
            language_variations: HashMap::new(),
            related_patterns: vec![],
            tags: vec!["logging".to_string(), "enterprise".to_string()],
            frequency_score: 0.8,
            detection_confidence: 0.9,
        };

        self.org_patterns
            .insert("example_org".to_string(), vec![org_pattern]);

        // Example compliance knowledge
        let compliance = ComplianceKnowledge {
            framework: "SOX".to_string(),
            required_patterns: vec!["audit_logging".to_string()],
            forbidden_patterns: vec!["hardcoded_secrets".to_string()],
            rules: vec![ComplianceRule {
                id: "sox_001".to_string(),
                description: "All financial operations must be logged".to_string(),
                severity: SeverityLevel::High,
                detection: DetectionMethod::StaticAnalysis {
                    pattern: "financial_operation_without_logging".to_string(),
                    confidence: 0.85,
                },
            }],
        };

        self.compliance_knowledge
            .insert("SOX".to_string(), compliance);

        Ok(())
    }

    fn check_policy_compliance(&self, code: &str, policy: &OrganizationPolicy) -> bool {
        // Simple example - check if code contains forbidden patterns
        for pattern in &policy.patterns {
            if pattern == "println_usage" && code.contains("println!") {
                return false;
            }
        }
        true
    }
}

/// Example framework-specific plugin (e.g., for React patterns)
pub struct ExampleFrameworkPlugin {
    metadata: KnowledgePluginMetadata,
    framework_knowledge: HashMap<String, crate::plugins::knowledge::FrameworkKnowledge>,
    initialized: bool,
}

#[async_trait]
impl KnowledgePlugin for ExampleFrameworkPlugin {
    fn metadata(&self) -> &KnowledgePluginMetadata {
        &self.metadata
    }

    async fn initialize(&mut self) -> Result<(), PluginError> {
        if self.initialized {
            return Ok(());
        }

        self.load_framework_knowledge().await?;
        self.initialized = true;

        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), PluginError> {
        self.framework_knowledge.clear();
        self.initialized = false;
        Ok(())
    }

    async fn get_patterns(&self) -> Result<Vec<PatternKnowledge>, PluginError> {
        let mut all_patterns = Vec::new();
        for framework_knowledge in self.framework_knowledge.values() {
            all_patterns.extend(framework_knowledge.patterns.clone());
        }
        Ok(all_patterns)
    }

    async fn get_detectors(&self) -> Result<Vec<DetectionMethod>, PluginError> {
        Ok(vec![])
    }

    async fn get_solutions(&self) -> Result<Vec<SolutionPattern>, PluginError> {
        Ok(vec![])
    }

    async fn get_language_knowledge(
        &self,
        _language: SourceLanguage,
    ) -> Result<Option<LanguageKnowledge>, PluginError> {
        Ok(None)
    }

    async fn get_framework_knowledge(
        &self,
        framework: &str,
    ) -> Result<Option<crate::plugins::knowledge::FrameworkKnowledge>, PluginError> {
        Ok(self.framework_knowledge.get(framework).cloned())
    }

    async fn health_check(&self) -> Result<PluginHealthStatus, PluginError> {
        Ok(PluginHealthStatus::Healthy)
    }
}

impl ExampleFrameworkPlugin {
    pub fn new(metadata: KnowledgePluginMetadata) -> Self {
        Self {
            metadata,
            framework_knowledge: HashMap::new(),
            initialized: false,
        }
    }

    async fn load_framework_knowledge(&mut self) -> Result<(), PluginError> {
        #[cfg(feature = "ai")]
        use crate::ai::knowledge::compression::CompressedString;
        #[cfg(not(feature = "ai"))]
        use crate::plugins::development::CompressedString;

        // Example React framework knowledge
        let react_pattern = PatternKnowledge {
            id: "react_unnecessary_rerender".to_string(),
            name: "Unnecessary React Re-renders".to_string(),
            definition: CompressedString::new(
                "Components that re-render unnecessarily due to improper dependency management"
            ),
            symptoms: vec![
                CompressedString::new("Missing dependency arrays in useEffect"),
                CompressedString::new("Creating objects/functions in render"),
                CompressedString::new("Not using React.memo for expensive components"),
            ],
            impact: ImpactLevel::Medium,
            category: AntiPatternCategory::Performance,
            detection_methods: vec![
                DetectionMethod::AstPattern {
                    query: "useEffect_without_deps".to_string(),
                    node_types: vec!["CallExpression".to_string(), "useEffect".to_string()],
                }
            ],
            solutions: vec![
                SolutionPattern {
                    id: "react_performance_optimization".to_string(),
                    title: "Optimize React Performance".to_string(),
                    implementation: CompressedString::new(
                        "Use proper dependency management and memoization:\n\
                        1. Add proper dependency arrays to useEffect\n\
                        2. Use useCallback for function dependencies\n\
                        3. Use useMemo for expensive calculations\n\
                        4. Wrap components with React.memo when appropriate\n\
                        Benefits: Reduced unnecessary re-renders, better performance, improved user experience"
                    ),
                    examples: vec![],
                    effort_level: EffortLevel::Medium,
                    prerequisites: vec!["Understanding of React hooks and memoization".to_string()],
                    expected_impact: ImpactLevel::Medium,
                }
            ],
            examples: CodeExamples {
                primary: vec![],
                variations: HashMap::new(),
            },
            language_variations: HashMap::new(),
            related_patterns: vec!["performance_issues".to_string()],
            tags: vec!["react".to_string(), "performance".to_string()],
            frequency_score: 0.7,
            detection_confidence: 0.8,
        };

        let react_knowledge = crate::plugins::knowledge::FrameworkKnowledge {
            name: "React".to_string(),
            version: "18.x".to_string(),
            patterns: vec![react_pattern],
            best_practices: vec![
                "Use functional components with hooks".to_string(),
                "Implement proper error boundaries".to_string(),
                "Use React.StrictMode in development".to_string(),
            ],
            pitfalls: vec![
                "Mutating state directly".to_string(),
                "Not cleaning up side effects".to_string(),
                "Overusing useEffect".to_string(),
            ],
        };

        self.framework_knowledge
            .insert("React".to_string(), react_knowledge);

        Ok(())
    }
}

// Default implementations for plugin capabilities and permissions
impl Default for KnowledgePluginCapabilities {
    fn default() -> Self {
        Self {
            provides_patterns: true,
            provides_detectors: false,
            provides_solutions: true,
            provides_language_support: false,
            provides_framework_knowledge: false,
            requires_network: false,
            requires_filesystem: false,
        }
    }
}

impl Default for KnowledgePluginPermissions {
    fn default() -> Self {
        Self {
            network: NetworkPermissions {
                allow_http: false,
                allow_https: false,
                allowed_domains: vec![],
                allowed_ports: vec![],
            },
            filesystem: FilesystemPermissions {
                allow_read: false,
                allow_write: false,
                allowed_paths: vec![],
            },
            system: SystemPermissions {
                allow_process_execution: false,
                allow_env_access: false,
                max_memory_mb: 100,
                max_execution_time_ms: 5000,
            },
            data: DataPermissions {
                allow_knowledge_read: true,
                allow_knowledge_write: false,
                allow_analysis_data: true,
            },
        }
    }
}

// Plugin creation functions - separated to avoid macro conflicts
pub fn create_anti_pattern_plugin() -> Box<dyn KnowledgePlugin> {
    let metadata = KnowledgePluginMetadata {
        id: "example-anti-pattern-plugin-1.0.0".to_string(),
        name: "Example Anti-Pattern Plugin".to_string(),
        version: "1.0.0".to_string(),
        author: "Uveddi Community".to_string(),
        description: "Example plugin demonstrating custom anti-pattern definitions".to_string(),
        plugin_type: KnowledgePluginType::AntiPattern,
        supported_languages: vec![SourceLanguage::Rust, SourceLanguage::Python],
        dependencies: vec![],
        api_version: "1.0.0".to_string(),
        capabilities: KnowledgePluginCapabilities::default(),
        permissions: KnowledgePluginPermissions::default(),
    };

    Box::new(ExampleAntiPatternPlugin::new(metadata))
}

pub fn create_enterprise_plugin() -> Box<dyn KnowledgePlugin> {
    let metadata = KnowledgePluginMetadata {
        id: "enterprise-compliance-plugin-1.0.0".to_string(),
        name: "Enterprise Compliance Plugin".to_string(),
        version: "1.0.0".to_string(),
        author: "Enterprise Team".to_string(),
        description: "Organization-specific patterns and compliance rules".to_string(),
        plugin_type: KnowledgePluginType::Enterprise,
        supported_languages: vec![SourceLanguage::Universal],
        dependencies: vec![],
        api_version: "1.0.0".to_string(),
        capabilities: KnowledgePluginCapabilities::default(),
        permissions: KnowledgePluginPermissions::default(),
    };

    Box::new(ExampleEnterprisePlugin::new(metadata))
}

pub fn create_framework_plugin() -> Box<dyn KnowledgePlugin> {
    let metadata = KnowledgePluginMetadata {
        id: "react-framework-plugin-1.0.0".to_string(),
        name: "React Framework Plugin".to_string(),
        version: "1.0.0".to_string(),
        author: "Frontend Team".to_string(),
        description: "React-specific anti-patterns and best practices".to_string(),
        plugin_type: KnowledgePluginType::Framework,
        supported_languages: vec![SourceLanguage::JavaScript, SourceLanguage::TypeScript],
        dependencies: vec![],
        api_version: "1.0.0".to_string(),
        capabilities: KnowledgePluginCapabilities::default(),
        permissions: KnowledgePluginPermissions::default(),
    };

    Box::new(ExampleFrameworkPlugin::new(metadata))
}
*/
