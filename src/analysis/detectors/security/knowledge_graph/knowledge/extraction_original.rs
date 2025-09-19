//! Knowledge extraction from code entities and structures
//!
//! This module handles the extraction of semantic and structural knowledge
//! from code entities, preparing them for graph construction and analysis.

use crate::analysis::detectors::security::knowledge_graph::types::{
    CodeEntity, SemanticEnrichment, CodeRelationship, RelationshipType,
};
use crate::analysis::AnalysisError;
use crate::ast::SourceLanguage;
use std::collections::HashMap;
use tracing::{debug, info};

/// Knowledge extractor for code entities
pub struct KnowledgeExtractor {
    language: SourceLanguage,
    extraction_config: ExtractionConfig,
}

impl KnowledgeExtractor {
    pub fn new(language: SourceLanguage) -> Self {
        Self {
            language,
            extraction_config: ExtractionConfig::default(),
        }
    }

    /// Extract knowledge from a code entity
    pub async fn extract_entity_knowledge(&self, entity: &CodeEntity) -> Result<EntityKnowledge, AnalysisError> {
        debug!("Extracting knowledge from entity: {}", entity.name);

        let structural_knowledge = self.extract_structural_knowledge(entity).await?;
        let semantic_knowledge = self.extract_semantic_knowledge(entity).await?;
        let security_knowledge = self.extract_security_knowledge(entity).await?;

        Ok(EntityKnowledge {
            entity_id: entity.id.clone(),
            structural: structural_knowledge,
            semantic: semantic_knowledge,
            security: security_knowledge,
        })
    }

    /// Extract multiple entities and their relationships
    pub async fn extract_batch_knowledge(
        &self,
        entities: &[CodeEntity],
    ) -> Result<BatchKnowledgeResult, AnalysisError> {
        info!("Extracting knowledge from {} entities", entities.len());

        let mut entity_knowledge = Vec::new();
        let mut relationships = Vec::new();

        // Extract individual entity knowledge
        for entity in entities {
            let knowledge = self.extract_entity_knowledge(entity).await?;
            entity_knowledge.push(knowledge);
        }

        // Extract relationships between entities
        relationships.extend(self.extract_call_relationships(entities).await?);
        relationships.extend(self.extract_dependency_relationships(entities).await?);
        relationships.extend(self.extract_inheritance_relationships(entities).await?);

        Ok(BatchKnowledgeResult {
            entity_knowledge,
            relationships,
        })
    }

    /// Extract structural knowledge (deterministic facts)
    async fn extract_structural_knowledge(&self, entity: &CodeEntity) -> Result<StructuralKnowledge, AnalysisError> {
        let mut properties = HashMap::new();
        let mut metrics = HashMap::new();

        // Extract basic properties
        properties.insert("name".to_string(), entity.name.clone());
        properties.insert("type".to_string(), format!("{:?}", entity.entity_type));
        properties.insert("language".to_string(), format!("{:?}", entity.language));
        properties.insert("file".to_string(), entity.location.file_path.display().to_string());

        // Calculate basic metrics
        let line_count = entity.location.end_line - entity.location.start_line + 1;
        metrics.insert("line_count".to_string(), line_count as f64);

        // Extract complexity indicators from metadata
        if let Some(complexity) = entity.metadata.get("cyclomatic_complexity") {
            if let Ok(complexity_value) = serde_json::from_value::<f64>(complexity.clone()) {
                metrics.insert("cyclomatic_complexity".to_string(), complexity_value);
            }
        }

        // Extract parameter count for functions
        if let Some(params) = entity.metadata.get("parameter_count") {
            if let Ok(param_count) = serde_json::from_value::<f64>(params.clone()) {
                metrics.insert("parameter_count".to_string(), param_count);
            }
        }

        Ok(StructuralKnowledge {
            properties,
            metrics,
            patterns: self.identify_structural_patterns(entity),
        })
    }

    /// Extract semantic knowledge (AI-generated insights)
    async fn extract_semantic_knowledge(&self, entity: &CodeEntity) -> Result<SemanticKnowledge, AnalysisError> {
        // In a full implementation, this would use AI to generate insights
        // For now, we'll create placeholder semantic knowledge

        let purpose = self.infer_entity_purpose(entity);
        let concepts = self.extract_conceptual_tags(entity);
        let context = self.determine_architectural_context(entity);

        Ok(SemanticKnowledge {
            purpose,
            concepts,
            context,
            confidence: 0.7, // Placeholder confidence
        })
    }

    /// Extract security-relevant knowledge
    async fn extract_security_knowledge(&self, entity: &CodeEntity) -> Result<SecurityKnowledge, AnalysisError> {
        let risk_indicators = self.identify_risk_indicators(entity);
        let attack_surface = self.assess_attack_surface(entity);
        let trust_boundaries = self.identify_trust_boundaries(entity);

        Ok(SecurityKnowledge {
            risk_indicators,
            attack_surface,
            trust_boundaries,
            privilege_level: self.infer_privilege_level(entity),
        })
    }

    /// Extract call relationships between entities
    async fn extract_call_relationships(&self, entities: &[CodeEntity]) -> Result<Vec<CodeRelationship>, AnalysisError> {
        let mut relationships = Vec::new();

        for caller in entities {
            for callee in entities {
                if caller.id != callee.id && self.entities_have_call_relationship(caller, callee) {
                    relationships.push(CodeRelationship {
                        from_entity: caller.id.clone(),
                        to_entity: callee.id.clone(),
                        relationship_type: RelationshipType::Calls,
                        strength: 0.8,
                    });
                }
            }
        }

        Ok(relationships)
    }

    /// Extract dependency relationships
    async fn extract_dependency_relationships(&self, entities: &[CodeEntity]) -> Result<Vec<CodeRelationship>, AnalysisError> {
        let mut relationships = Vec::new();

        for dependent in entities {
            for dependency in entities {
                if dependent.id != dependency.id && self.entities_have_dependency(dependent, dependency) {
                    relationships.push(CodeRelationship {
                        from_entity: dependent.id.clone(),
                        to_entity: dependency.id.clone(),
                        relationship_type: RelationshipType::DependsOn,
                        strength: 0.7,
                    });
                }
            }
        }

        Ok(relationships)
    }

    /// Extract inheritance relationships
    async fn extract_inheritance_relationships(&self, entities: &[CodeEntity]) -> Result<Vec<CodeRelationship>, AnalysisError> {
        let mut relationships = Vec::new();

        for child in entities {
            for parent in entities {
                if child.id != parent.id && self.entities_have_inheritance(child, parent) {
                    relationships.push(CodeRelationship {
                        from_entity: child.id.clone(),
                        to_entity: parent.id.clone(),
                        relationship_type: RelationshipType::Inherits,
                        strength: 0.9,
                    });
                }
            }
        }

        Ok(relationships)
    }

    /// Identify structural patterns in an entity
    fn identify_structural_patterns(&self, entity: &CodeEntity) -> Vec<String> {
        let mut patterns = Vec::new();

        // Check for singleton pattern indicators
        if entity.name.to_lowercase().contains("singleton") {
            patterns.push("Singleton".to_string());
        }

        // Check for factory pattern indicators
        if entity.name.to_lowercase().contains("factory") || entity.name.to_lowercase().contains("builder") {
            patterns.push("Factory".to_string());
        }

        // Check for observer pattern indicators
        if entity.name.to_lowercase().contains("observer") || entity.name.to_lowercase().contains("listener") {
            patterns.push("Observer".to_string());
        }

        patterns
    }

    /// Infer the purpose of an entity from its characteristics
    fn infer_entity_purpose(&self, entity: &CodeEntity) -> String {
        let name_lower = entity.name.to_lowercase();

        if name_lower.contains("test") {
            "Testing and validation".to_string()
        } else if name_lower.contains("util") || name_lower.contains("helper") {
            "Utility and helper functionality".to_string()
        } else if name_lower.contains("config") || name_lower.contains("setting") {
            "Configuration management".to_string()
        } else if name_lower.contains("auth") || name_lower.contains("login") {
            "Authentication and authorization".to_string()
        } else if name_lower.contains("validate") || name_lower.contains("check") {
            "Validation and verification".to_string()
        } else {
            match entity.entity_type {
                crate::analysis::detectors::security::knowledge_graph::types::EntityType::Function => "Function implementation".to_string(),
                crate::analysis::detectors::security::knowledge_graph::types::EntityType::Class => "Class definition and behavior".to_string(),
                crate::analysis::detectors::security::knowledge_graph::types::EntityType::Module => "Module organization and exports".to_string(),
                _ => "General code entity".to_string(),
            }
        }
    }

    /// Extract conceptual tags from entity
    fn extract_conceptual_tags(&self, entity: &CodeEntity) -> Vec<String> {
        let mut concepts = Vec::new();
        let name_lower = entity.name.to_lowercase();

        // Domain concepts
        if name_lower.contains("user") || name_lower.contains("account") {
            concepts.push("User Management".to_string());
        }
        if name_lower.contains("data") || name_lower.contains("db") || name_lower.contains("database") {
            concepts.push("Data Management".to_string());
        }
        if name_lower.contains("api") || name_lower.contains("http") || name_lower.contains("rest") {
            concepts.push("API".to_string());
        }
        if name_lower.contains("security") || name_lower.contains("auth") {
            concepts.push("Security".to_string());
        }

        concepts
    }

    /// Determine architectural context
    fn determine_architectural_context(&self, entity: &CodeEntity) -> String {
        let file_path = entity.location.file_path.display().to_string().to_lowercase();

        if file_path.contains("controller") || file_path.contains("handler") {
            "Presentation Layer".to_string()
        } else if file_path.contains("service") || file_path.contains("business") {
            "Business Logic Layer".to_string()
        } else if file_path.contains("repository") || file_path.contains("dao") || file_path.contains("data") {
            "Data Access Layer".to_string()
        } else if file_path.contains("config") || file_path.contains("setting") {
            "Configuration Layer".to_string()
        } else {
            "General Implementation".to_string()
        }
    }

    /// Identify risk indicators in an entity
    fn identify_risk_indicators(&self, entity: &CodeEntity) -> Vec<String> {
        let mut indicators = Vec::new();
        let name_lower = entity.name.to_lowercase();

        // High-risk function names
        if name_lower.contains("eval") || name_lower.contains("exec") {
            indicators.push("Dynamic Code Execution".to_string());
        }
        if name_lower.contains("unsafe") || name_lower.contains("raw") {
            indicators.push("Unsafe Operations".to_string());
        }
        if name_lower.contains("admin") || name_lower.contains("root") {
            indicators.push("Administrative Privileges".to_string());
        }
        if name_lower.contains("password") || name_lower.contains("secret") || name_lower.contains("key") {
            indicators.push("Sensitive Data Handling".to_string());
        }

        indicators
    }

    /// Assess attack surface of an entity
    fn assess_attack_surface(&self, entity: &CodeEntity) -> f64 {
        let mut surface_score = 0.0;

        // Public entities have higher attack surface
        if entity.metadata.get("visibility").map(|v| v.as_str()) == Some(Some("public")) {
            surface_score += 0.3;
        }

        // Functions with many parameters
        if let Some(param_count) = entity.metadata.get("parameter_count") {
            if let Ok(count) = serde_json::from_value::<u32>(param_count.clone()) {
                surface_score += (count as f64) * 0.1;
            }
        }

        // Network-related entities
        let name_lower = entity.name.to_lowercase();
        if name_lower.contains("http") || name_lower.contains("web") || name_lower.contains("api") {
            surface_score += 0.4;
        }

        surface_score.min(1.0)
    }

    /// Identify trust boundaries
    fn identify_trust_boundaries(&self, entity: &CodeEntity) -> Vec<String> {
        let mut boundaries = Vec::new();
        let name_lower = entity.name.to_lowercase();

        if name_lower.contains("validate") || name_lower.contains("sanitize") {
            boundaries.push("Input Validation".to_string());
        }
        if name_lower.contains("auth") || name_lower.contains("permission") {
            boundaries.push("Authorization Check".to_string());
        }
        if name_lower.contains("encrypt") || name_lower.contains("hash") {
            boundaries.push("Cryptographic Boundary".to_string());
        }

        boundaries
    }

    /// Infer privilege level of an entity
    fn infer_privilege_level(&self, entity: &CodeEntity) -> PrivilegeLevel {
        let name_lower = entity.name.to_lowercase();

        if name_lower.contains("admin") || name_lower.contains("root") || name_lower.contains("system") {
            PrivilegeLevel::Administrative
        } else if name_lower.contains("user") || name_lower.contains("client") {
            PrivilegeLevel::User
        } else if name_lower.contains("guest") || name_lower.contains("anonymous") {
            PrivilegeLevel::Guest
        } else {
            PrivilegeLevel::Standard
        }
    }

    /// Check if entities have call relationship
    fn entities_have_call_relationship(&self, caller: &CodeEntity, callee: &CodeEntity) -> bool {
        // Simple heuristic: if callee name appears in caller's metadata
        caller.metadata.values().any(|value| {
            if let Ok(str_value) = serde_json::from_value::<String>(value.clone()) {
                str_value.contains(&callee.name)
            } else {
                false
            }
        })
    }

    /// Check if entities have dependency relationship
    fn entities_have_dependency(&self, dependent: &CodeEntity, dependency: &CodeEntity) -> bool {
        // Check for import/use statements or similar dependencies
        dependent.metadata.get("imports")
            .or_else(|| dependent.metadata.get("uses"))
            .or_else(|| dependent.metadata.get("requires"))
            .map(|imports| {
                if let Ok(imports_str) = serde_json::from_value::<String>(imports.clone()) {
                    imports_str.contains(&dependency.name)
                } else {
                    false
                }
            })
            .unwrap_or(false)
    }

    /// Check if entities have inheritance relationship
    fn entities_have_inheritance(&self, child: &CodeEntity, parent: &CodeEntity) -> bool {
        // Check for inheritance indicators in metadata
        child.metadata.get("extends")
            .or_else(|| child.metadata.get("implements"))
            .or_else(|| child.metadata.get("inherits"))
            .map(|inheritance| {
                if let Ok(inheritance_str) = serde_json::from_value::<String>(inheritance.clone()) {
                    inheritance_str.contains(&parent.name)
                } else {
                    false
                }
            })
            .unwrap_or(false)
    }
}

/// Configuration for knowledge extraction
#[derive(Debug, Clone)]
pub struct ExtractionConfig {
    pub extract_semantics: bool,
    pub extract_security: bool,
    pub ai_confidence_threshold: f64,
}

impl Default for ExtractionConfig {
    fn default() -> Self {
        Self {
            extract_semantics: true,
            extract_security: true,
            ai_confidence_threshold: 0.5,
        }
    }
}

/// Extracted knowledge for a single entity
#[derive(Debug, Clone)]
pub struct EntityKnowledge {
    pub entity_id: String,
    pub structural: StructuralKnowledge,
    pub semantic: SemanticKnowledge,
    pub security: SecurityKnowledge,
}

/// Structural knowledge about an entity
#[derive(Debug, Clone)]
pub struct StructuralKnowledge {
    pub properties: HashMap<String, String>,
    pub metrics: HashMap<String, f64>,
    pub patterns: Vec<String>,
}

/// Semantic knowledge about an entity
#[derive(Debug, Clone)]
pub struct SemanticKnowledge {
    pub purpose: String,
    pub concepts: Vec<String>,
    pub context: String,
    pub confidence: f64,
}

/// Security knowledge about an entity
#[derive(Debug, Clone)]
pub struct SecurityKnowledge {
    pub risk_indicators: Vec<String>,
    pub attack_surface: f64,
    pub trust_boundaries: Vec<String>,
    pub privilege_level: PrivilegeLevel,
}

/// Privilege levels for entities
#[derive(Debug, Clone)]
pub enum PrivilegeLevel {
    Guest,
    User,
    Standard,
    Administrative,
}

/// Result of batch knowledge extraction
#[derive(Debug, Clone)]
pub struct BatchKnowledgeResult {
    pub entity_knowledge: Vec<EntityKnowledge>,
    pub relationships: Vec<CodeRelationship>,
}