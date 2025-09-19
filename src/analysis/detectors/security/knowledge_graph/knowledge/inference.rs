//! Inference engine for knowledge graph reasoning
//!
//! This module provides inference capabilities to derive new knowledge
//! from existing facts in the knowledge graph, including rule-based
//! reasoning and probabilistic inference.

use crate::analysis::detectors::security::knowledge_graph::types::{
    CodeEntity, CodeRelationship, SecurityQuery, SecurityKnowledgeResult,
    AntiPatternInfo, ArchitecturalCorrelation,
};
use crate::analysis::AnalysisError;
use std::collections::{HashMap, HashSet};
use tracing::{debug, info};

/// Inference engine for knowledge graph reasoning
pub struct InferenceEngine {
    rules: Vec<InferenceRule>,
    fact_cache: HashMap<String, InferredFact>,
}

impl InferenceEngine {
    pub fn new() -> Self {
        Self {
            rules: Self::default_rules(),
            fact_cache: HashMap::new(),
        }
    }

    /// Perform inference on a set of entities and relationships
    pub async fn infer_knowledge(
        &mut self,
        entities: &[CodeEntity],
        relationships: &[CodeRelationship],
    ) -> Result<InferenceResult, AnalysisError> {
        info!("Starting inference on {} entities and {} relationships",
              entities.len(), relationships.len());

        let mut inferred_facts = Vec::new();
        let mut new_relationships = Vec::new();
        let mut security_insights = Vec::new();

        // Apply all inference rules
        for rule in &self.rules {
            let rule_results = self.apply_rule(rule, entities, relationships).await?;
            inferred_facts.extend(rule_results.facts);
            new_relationships.extend(rule_results.relationships);
            security_insights.extend(rule_results.security_insights);
        }

        // Perform transitive inference
        let transitive_relationships = self.infer_transitive_relationships(relationships).await?;
        new_relationships.extend(transitive_relationships);

        // Cache results for future queries
        for fact in &inferred_facts {
            self.fact_cache.insert(fact.id.clone(), fact.clone());
        }

        Ok(InferenceResult {
            inferred_facts,
            new_relationships,
            security_insights,
        })
    }

    /// Query the inference engine for specific knowledge
    pub async fn query_inferred_knowledge(
        &self,
        query: &SecurityQuery,
    ) -> Result<QueryInferenceResult, AnalysisError> {
        debug!("Querying inference engine: {:?}", query);

        match query {
            SecurityQuery::ArchitecturalCorrelation { location, issue_type, .. } => {
                let correlations = self.infer_architectural_correlations(location, issue_type).await?;
                Ok(QueryInferenceResult {
                    relevant_facts: self.find_relevant_facts_for_location(&location.file_path),
                    inferred_relationships: Vec::new(),
                    confidence_score: 0.7,
                    correlations,
                })
            }
            SecurityQuery::DependencyAnalysis { entity_id } => {
                let dependency_insights = self.infer_dependency_implications(entity_id).await?;
                Ok(QueryInferenceResult {
                    relevant_facts: self.find_relevant_facts_for_entity(entity_id),
                    inferred_relationships: dependency_insights,
                    confidence_score: 0.8,
                    correlations: Vec::new(),
                })
            }
            SecurityQuery::PatternSearch { pattern_type } => {
                let pattern_facts = self.find_facts_by_pattern(pattern_type);
                Ok(QueryInferenceResult {
                    relevant_facts: pattern_facts,
                    inferred_relationships: Vec::new(),
                    confidence_score: 0.6,
                    correlations: Vec::new(),
                })
            }
        }
    }

    /// Apply a single inference rule
    async fn apply_rule(
        &self,
        rule: &InferenceRule,
        entities: &[CodeEntity],
        relationships: &[CodeRelationship],
    ) -> Result<RuleResult, AnalysisError> {
        match rule {
            InferenceRule::HighCouplingRisk => self.apply_high_coupling_rule(entities, relationships).await,
            InferenceRule::PrivilegeEscalation => self.apply_privilege_escalation_rule(entities, relationships).await,
            InferenceRule::DataFlowSecurity => self.apply_data_flow_security_rule(entities, relationships).await,
            InferenceRule::AntiPatternAmplification => self.apply_anti_pattern_amplification_rule(entities, relationships).await,
            InferenceRule::TrustBoundaryViolation => self.apply_trust_boundary_rule(entities, relationships).await,
        }
    }

    /// Apply high coupling risk inference rule
    async fn apply_high_coupling_rule(
        &self,
        entities: &[CodeEntity],
        relationships: &[CodeRelationship],
    ) -> Result<RuleResult, AnalysisError> {
        let mut facts = Vec::new();
        let mut security_insights = Vec::new();

        // Build entity connection counts
        let mut connection_counts = HashMap::new();
        for relationship in relationships {
            *connection_counts.entry(relationship.from_entity.clone()).or_insert(0) += 1;
            *connection_counts.entry(relationship.to_entity.clone()).or_insert(0) += 1;
        }

        // Identify highly coupled entities
        for (entity_id, count) in connection_counts {
            if count > 10 {  // Threshold for high coupling
                if let Some(entity) = entities.iter().find(|e| e.id == entity_id) {
                    facts.push(InferredFact {
                        id: format!("high_coupling_{}", entity_id),
                        fact_type: FactType::HighCoupling,
                        subject: entity_id.clone(),
                        confidence: (count as f64 / 20.0).min(1.0),
                        evidence: vec![format!("Entity has {} connections", count)],
                    });

                    security_insights.push(SecurityInsight {
                        insight_type: InsightType::ArchitecturalRisk,
                        description: format!(
                            "Entity '{}' has high coupling ({} connections), increasing security risk",
                            entity.name, count
                        ),
                        severity: (count as f64 / 20.0).min(1.0),
                        affected_entities: vec![entity_id],
                    });
                }
            }
        }

        Ok(RuleResult {
            facts,
            relationships: Vec::new(),
            security_insights,
        })
    }

    /// Apply privilege escalation inference rule
    async fn apply_privilege_escalation_rule(
        &self,
        entities: &[CodeEntity],
        relationships: &[CodeRelationship],
    ) -> Result<RuleResult, AnalysisError> {
        let mut facts = Vec::new();
        let mut security_insights = Vec::new();

        // Identify privilege levels from entity names/metadata
        let high_privilege: HashSet<_> = entities.iter()
            .filter(|e| e.name.to_lowercase().contains("admin") ||
                       e.name.to_lowercase().contains("root") ||
                       e.name.to_lowercase().contains("system"))
            .map(|e| e.id.clone())
            .collect();

        let low_privilege: HashSet<_> = entities.iter()
            .filter(|e| e.name.to_lowercase().contains("user") ||
                       e.name.to_lowercase().contains("guest") ||
                       e.name.to_lowercase().contains("public"))
            .map(|e| e.id.clone())
            .collect();

        // Look for paths from low to high privilege
        for low_priv in &low_privilege {
            for high_priv in &high_privilege {
                if self.has_path_through_relationships(low_priv, high_priv, relationships) {
                    facts.push(InferredFact {
                        id: format!("escalation_{}_{}", low_priv, high_priv),
                        fact_type: FactType::PrivilegeEscalation,
                        subject: format!("{}:{}", low_priv, high_priv),
                        confidence: 0.8,
                        evidence: vec!["Direct path from low to high privilege entity".to_string()],
                    });

                    security_insights.push(SecurityInsight {
                        insight_type: InsightType::SecurityThreat,
                        description: "Potential privilege escalation path detected".to_string(),
                        severity: 0.9,
                        affected_entities: vec![low_priv.clone(), high_priv.clone()],
                    });
                }
            }
        }

        Ok(RuleResult {
            facts,
            relationships: Vec::new(),
            security_insights,
        })
    }

    /// Apply data flow security rule
    async fn apply_data_flow_security_rule(
        &self,
        entities: &[CodeEntity],
        relationships: &[CodeRelationship],
    ) -> Result<RuleResult, AnalysisError> {
        let mut facts = Vec::new();
        let mut security_insights = Vec::new();

        // Identify data sources and sinks
        let data_sources: HashSet<_> = entities.iter()
            .filter(|e| e.name.to_lowercase().contains("input") ||
                       e.name.to_lowercase().contains("request") ||
                       e.name.to_lowercase().contains("user"))
            .map(|e| e.id.clone())
            .collect();

        let sensitive_sinks: HashSet<_> = entities.iter()
            .filter(|e| e.name.to_lowercase().contains("database") ||
                       e.name.to_lowercase().contains("file") ||
                       e.name.to_lowercase().contains("exec"))
            .map(|e| e.id.clone())
            .collect();

        // Check for unvalidated data flows
        for source in &data_sources {
            for sink in &sensitive_sinks {
                if self.has_unvalidated_path(source, sink, entities, relationships) {
                    facts.push(InferredFact {
                        id: format!("unsafe_flow_{}_{}", source, sink),
                        fact_type: FactType::UnsafeDataFlow,
                        subject: format!("{}:{}", source, sink),
                        confidence: 0.7,
                        evidence: vec!["Unvalidated data flow to sensitive sink".to_string()],
                    });

                    security_insights.push(SecurityInsight {
                        insight_type: InsightType::SecurityVulnerability,
                        description: "Potential unsafe data flow detected".to_string(),
                        severity: 0.8,
                        affected_entities: vec![source.clone(), sink.clone()],
                    });
                }
            }
        }

        Ok(RuleResult {
            facts,
            relationships: Vec::new(),
            security_insights,
        })
    }

    /// Apply anti-pattern amplification rule
    async fn apply_anti_pattern_amplification_rule(
        &self,
        entities: &[CodeEntity],
        relationships: &[CodeRelationship],
    ) -> Result<RuleResult, AnalysisError> {
        let mut facts = Vec::new();
        let mut security_insights = Vec::new();

        // Identify god objects (entities with many outgoing relationships)
        for entity in entities {
            let outgoing_count = relationships.iter()
                .filter(|r| r.from_entity == entity.id)
                .count();

            if outgoing_count > 15 {
                facts.push(InferredFact {
                    id: format!("god_object_{}", entity.id),
                    fact_type: FactType::GodObject,
                    subject: entity.id.clone(),
                    confidence: (outgoing_count as f64 / 25.0).min(1.0),
                    evidence: vec![format!("Entity has {} outgoing dependencies", outgoing_count)],
                });

                security_insights.push(SecurityInsight {
                    insight_type: InsightType::ArchitecturalRisk,
                    description: format!(
                        "God object '{}' increases security complexity and attack surface",
                        entity.name
                    ),
                    severity: (outgoing_count as f64 / 25.0).min(1.0),
                    affected_entities: vec![entity.id.clone()],
                });
            }
        }

        Ok(RuleResult {
            facts,
            relationships: Vec::new(),
            security_insights,
        })
    }

    /// Apply trust boundary violation rule
    async fn apply_trust_boundary_rule(
        &self,
        entities: &[CodeEntity],
        relationships: &[CodeRelationship],
    ) -> Result<RuleResult, AnalysisError> {
        let mut facts = Vec::new();
        let mut security_insights = Vec::new();

        // Identify trust boundaries based on entity context
        let trusted: HashSet<_> = entities.iter()
            .filter(|e| e.location.file_path.display().to_string().contains("internal") ||
                       e.location.file_path.display().to_string().contains("core"))
            .map(|e| e.id.clone())
            .collect();

        let untrusted: HashSet<_> = entities.iter()
            .filter(|e| e.location.file_path.display().to_string().contains("external") ||
                       e.location.file_path.display().to_string().contains("api") ||
                       e.location.file_path.display().to_string().contains("public"))
            .map(|e| e.id.clone())
            .collect();

        // Check for direct connections from untrusted to trusted without validation
        for untrusted_entity in &untrusted {
            for trusted_entity in &trusted {
                if relationships.iter().any(|r| r.from_entity == *untrusted_entity && r.to_entity == *trusted_entity) {
                    facts.push(InferredFact {
                        id: format!("boundary_violation_{}_{}", untrusted_entity, trusted_entity),
                        fact_type: FactType::TrustBoundaryViolation,
                        subject: format!("{}:{}", untrusted_entity, trusted_entity),
                        confidence: 0.9,
                        evidence: vec!["Direct connection from untrusted to trusted context".to_string()],
                    });

                    security_insights.push(SecurityInsight {
                        insight_type: InsightType::SecurityVulnerability,
                        description: "Trust boundary violation detected".to_string(),
                        severity: 0.9,
                        affected_entities: vec![untrusted_entity.clone(), trusted_entity.clone()],
                    });
                }
            }
        }

        Ok(RuleResult {
            facts,
            relationships: Vec::new(),
            security_insights,
        })
    }

    /// Infer transitive relationships
    async fn infer_transitive_relationships(
        &self,
        relationships: &[CodeRelationship],
    ) -> Result<Vec<CodeRelationship>, AnalysisError> {
        let mut transitive = Vec::new();

        // Build adjacency map
        let mut adjacency = HashMap::new();
        for rel in relationships {
            adjacency.entry(rel.from_entity.clone())
                .or_insert_with(Vec::new)
                .push((rel.to_entity.clone(), rel.relationship_type.clone(), rel.strength));
        }

        // Find transitive dependencies (A -> B -> C implies A -> C with lower strength)
        for (source, targets) in &adjacency {
            for (intermediate, rel_type, strength1) in targets {
                if let Some(intermediate_targets) = adjacency.get(intermediate) {
                    for (final_target, _, strength2) in intermediate_targets {
                        if source != final_target && !self.direct_relationship_exists(source, final_target, relationships) {
                            // Create transitive relationship with reduced strength
                            let transitive_strength = (strength1 * strength2) * 0.7; // Reduce strength for transitivity
                            if transitive_strength > 0.3 { // Only keep strong transitive relationships
                                transitive.push(CodeRelationship {
                                    from_entity: source.clone(),
                                    to_entity: final_target.clone(),
                                    relationship_type: rel_type.clone(),
                                    strength: transitive_strength,
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(transitive)
    }

    /// Check if direct relationship exists
    fn direct_relationship_exists(&self, from: &str, to: &str, relationships: &[CodeRelationship]) -> bool {
        relationships.iter().any(|r| r.from_entity == from && r.to_entity == to)
    }

    /// Check if there's a path between entities
    fn has_path_through_relationships(&self, from: &str, to: &str, relationships: &[CodeRelationship]) -> bool {
        let mut visited = HashSet::new();
        self.dfs_path_exists(from, to, relationships, &mut visited)
    }

    /// DFS to check path existence
    fn dfs_path_exists(
        &self,
        current: &str,
        target: &str,
        relationships: &[CodeRelationship],
        visited: &mut HashSet<String>,
    ) -> bool {
        if current == target {
            return true;
        }

        if visited.contains(current) {
            return false;
        }

        visited.insert(current.to_string());

        for rel in relationships {
            if rel.from_entity == current {
                if self.dfs_path_exists(&rel.to_entity, target, relationships, visited) {
                    return true;
                }
            }
        }

        false
    }

    /// Check for unvalidated data flow path
    fn has_unvalidated_path(
        &self,
        source: &str,
        sink: &str,
        entities: &[CodeEntity],
        relationships: &[CodeRelationship],
    ) -> bool {
        // Simple heuristic: check if there's a direct path without validation entities
        let validation_entities: HashSet<_> = entities.iter()
            .filter(|e| e.name.to_lowercase().contains("validate") ||
                       e.name.to_lowercase().contains("sanitize") ||
                       e.name.to_lowercase().contains("check"))
            .map(|e| e.id.as_str())
            .collect();

        let mut visited = HashSet::new();
        self.has_path_without_validation(source, sink, relationships, &validation_entities, &mut visited)
    }

    /// DFS to find path without validation
    fn has_path_without_validation(
        &self,
        current: &str,
        target: &str,
        relationships: &[CodeRelationship],
        validation_entities: &HashSet<&str>,
        visited: &mut HashSet<String>,
    ) -> bool {
        if current == target {
            return true;
        }

        if visited.contains(current) || validation_entities.contains(current) {
            return false;
        }

        visited.insert(current.to_string());

        for rel in relationships {
            if rel.from_entity == current {
                if self.has_path_without_validation(&rel.to_entity, target, relationships, validation_entities, visited) {
                    return true;
                }
            }
        }

        false
    }

    /// Infer architectural correlations
    async fn infer_architectural_correlations(
        &self,
        location: &crate::analysis::detectors::security::types::SecurityLocation,
        issue_type: &crate::analysis::detectors::security::types::SecurityIssueType,
    ) -> Result<Vec<ArchitecturalCorrelation>, AnalysisError> {
        // Placeholder implementation
        Ok(Vec::new())
    }

    /// Infer dependency implications
    async fn infer_dependency_implications(&self, entity_id: &str) -> Result<Vec<CodeRelationship>, AnalysisError> {
        // Placeholder implementation
        Ok(Vec::new())
    }

    /// Find relevant facts for a location
    fn find_relevant_facts_for_location(&self, _file_path: &std::path::Path) -> Vec<InferredFact> {
        // Placeholder implementation
        Vec::new()
    }

    /// Find relevant facts for an entity
    fn find_relevant_facts_for_entity(&self, _entity_id: &str) -> Vec<InferredFact> {
        // Placeholder implementation
        Vec::new()
    }

    /// Find facts by pattern
    fn find_facts_by_pattern(&self, _pattern_type: &str) -> Vec<InferredFact> {
        // Placeholder implementation
        Vec::new()
    }

    /// Default inference rules
    fn default_rules() -> Vec<InferenceRule> {
        vec![
            InferenceRule::HighCouplingRisk,
            InferenceRule::PrivilegeEscalation,
            InferenceRule::DataFlowSecurity,
            InferenceRule::AntiPatternAmplification,
            InferenceRule::TrustBoundaryViolation,
        ]
    }
}

/// Inference rules for knowledge derivation
#[derive(Debug, Clone)]
pub enum InferenceRule {
    HighCouplingRisk,
    PrivilegeEscalation,
    DataFlowSecurity,
    AntiPatternAmplification,
    TrustBoundaryViolation,
}

/// Result of applying an inference rule
#[derive(Debug, Clone)]
pub struct RuleResult {
    pub facts: Vec<InferredFact>,
    pub relationships: Vec<CodeRelationship>,
    pub security_insights: Vec<SecurityInsight>,
}

/// Complete inference result
#[derive(Debug, Clone)]
pub struct InferenceResult {
    pub inferred_facts: Vec<InferredFact>,
    pub new_relationships: Vec<CodeRelationship>,
    pub security_insights: Vec<SecurityInsight>,
}

/// Result of querying the inference engine
#[derive(Debug, Clone)]
pub struct QueryInferenceResult {
    pub relevant_facts: Vec<InferredFact>,
    pub inferred_relationships: Vec<CodeRelationship>,
    pub confidence_score: f64,
    pub correlations: Vec<ArchitecturalCorrelation>,
}

/// Inferred fact from reasoning
#[derive(Debug, Clone)]
pub struct InferredFact {
    pub id: String,
    pub fact_type: FactType,
    pub subject: String,
    pub confidence: f64,
    pub evidence: Vec<String>,
}

/// Types of inferred facts
#[derive(Debug, Clone)]
pub enum FactType {
    HighCoupling,
    PrivilegeEscalation,
    UnsafeDataFlow,
    GodObject,
    TrustBoundaryViolation,
    SecurityRisk,
}

/// Security insights from inference
#[derive(Debug, Clone)]
pub struct SecurityInsight {
    pub insight_type: InsightType,
    pub description: String,
    pub severity: f64,
    pub affected_entities: Vec<String>,
}

/// Types of security insights
#[derive(Debug, Clone)]
pub enum InsightType {
    ArchitecturalRisk,
    SecurityThreat,
    SecurityVulnerability,
    PerformanceImpact,
}