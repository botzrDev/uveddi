//! Knowledge validation and consistency checking
//!
//! This module provides validation capabilities for knowledge graph facts,
//! ensuring consistency and reliability of extracted and inferred knowledge.

use crate::analysis::detectors::security::knowledge_graph::knowledge::extraction::{
    EntityKnowledge, BatchKnowledgeResult,
};
use crate::analysis::detectors::security::knowledge_graph::knowledge::inference::{
    InferenceResult, InferredFact, SecurityInsight,
};
use crate::analysis::detectors::security::knowledge_graph::types::{
    CodeEntity, CodeRelationship, RelationshipType,
};
use crate::analysis::AnalysisError;
use std::collections::{HashMap, HashSet};
use tracing::{debug, warn, info};

/// Knowledge validator for ensuring data quality and consistency
pub struct KnowledgeValidator {
    validation_rules: Vec<ValidationRule>,
    consistency_checks: Vec<ConsistencyCheck>,
}

impl KnowledgeValidator {
    pub fn new() -> Self {
        Self {
            validation_rules: Self::default_validation_rules(),
            consistency_checks: Self::default_consistency_checks(),
        }
    }

    /// Validate extracted knowledge
    pub async fn validate_extracted_knowledge(
        &self,
        knowledge: &BatchKnowledgeResult,
    ) -> Result<ValidationResult, AnalysisError> {
        info!("Validating extracted knowledge for {} entities", knowledge.entity_knowledge.len());

        let mut issues = Vec::new();
        let mut confidence_adjustments = HashMap::new();

        // Apply validation rules to entity knowledge
        for entity_knowledge in &knowledge.entity_knowledge {
            for rule in &self.validation_rules {
                if let Some(issue) = self.apply_validation_rule(rule, entity_knowledge).await? {
                    issues.push(issue);
                }
            }
        }

        // Validate relationships
        let relationship_issues = self.validate_relationships(&knowledge.relationships).await?;
        issues.extend(relationship_issues);

        // Apply consistency checks
        let consistency_issues = self.apply_consistency_checks(knowledge).await?;
        issues.extend(consistency_issues);

        Ok(ValidationResult {
            is_valid: issues.iter().all(|i| i.severity < 0.8),
            issues,
            confidence_adjustments,
            validation_score: self.calculate_validation_score(&issues),
        })
    }

    /// Validate inferred knowledge
    pub async fn validate_inferred_knowledge(
        &self,
        inference: &InferenceResult,
        base_knowledge: &BatchKnowledgeResult,
    ) -> Result<InferenceValidation, AnalysisError> {
        info!("Validating {} inferred facts", inference.inferred_facts.len());

        let mut fact_validations = Vec::new();
        let mut insight_validations = Vec::new();

        // Validate each inferred fact
        for fact in &inference.inferred_facts {
            let validation = self.validate_inferred_fact(fact, base_knowledge).await?;
            fact_validations.push(validation);
        }

        // Validate security insights
        for insight in &inference.security_insights {
            let validation = self.validate_security_insight(insight, base_knowledge).await?;
            insight_validations.push(validation);
        }

        // Check for logical contradictions
        let contradictions = self.detect_logical_contradictions(inference).await?;

        Ok(InferenceValidation {
            fact_validations,
            insight_validations,
            contradictions,
            overall_confidence: self.calculate_inference_confidence(&fact_validations),
        })
    }

    /// Cross-validate knowledge across multiple sources
    pub async fn cross_validate_knowledge(
        &self,
        primary_knowledge: &BatchKnowledgeResult,
        secondary_sources: &[BatchKnowledgeResult],
    ) -> Result<CrossValidationResult, AnalysisError> {
        info!("Cross-validating knowledge across {} sources", secondary_sources.len() + 1);

        let mut agreements = Vec::new();
        let mut disagreements = Vec::new();
        let mut unique_insights = Vec::new();

        // Compare entity knowledge across sources
        for primary_entity in &primary_knowledge.entity_knowledge {
            let mut found_agreements = 0;
            let mut found_disagreements = 0;

            for secondary_knowledge in secondary_sources {
                if let Some(secondary_entity) = secondary_knowledge.entity_knowledge.iter()
                    .find(|e| e.entity_id == primary_entity.entity_id) {

                    let comparison = self.compare_entity_knowledge(primary_entity, secondary_entity);

                    if comparison.agreement_score > 0.7 {
                        found_agreements += 1;
                        agreements.push(KnowledgeAgreement {
                            entity_id: primary_entity.entity_id.clone(),
                            agreement_type: AgreementType::EntityKnowledge,
                            confidence: comparison.agreement_score,
                            sources: vec!["primary".to_string(), "secondary".to_string()],
                        });
                    } else if comparison.agreement_score < 0.3 {
                        found_disagreements += 1;
                        disagreements.push(KnowledgeDisagreement {
                            entity_id: primary_entity.entity_id.clone(),
                            disagreement_type: DisagreementType::ConflictingFacts,
                            severity: 1.0 - comparison.agreement_score,
                            details: comparison.differences,
                        });
                    }
                }
            }

            // Entity found only in primary source
            if found_agreements == 0 && found_disagreements == 0 {
                unique_insights.push(UniqueInsight {
                    entity_id: primary_entity.entity_id.clone(),
                    source: "primary".to_string(),
                    insight_type: InsightType::EntitySpecific,
                    confidence: primary_entity.semantic.confidence,
                });
            }
        }

        Ok(CrossValidationResult {
            agreements,
            disagreements,
            unique_insights,
            overall_agreement_score: self.calculate_overall_agreement(&agreements, &disagreements),
        })
    }

    /// Apply a single validation rule to entity knowledge
    async fn apply_validation_rule(
        &self,
        rule: &ValidationRule,
        knowledge: &EntityKnowledge,
    ) -> Result<Option<ValidationIssue>, AnalysisError> {
        match rule {
            ValidationRule::SemanticConsistency => self.check_semantic_consistency(knowledge).await,
            ValidationRule::StructuralValidity => self.check_structural_validity(knowledge).await,
            ValidationRule::SecurityRelevance => self.check_security_relevance(knowledge).await,
            ValidationRule::ConfidenceThreshold => self.check_confidence_threshold(knowledge).await,
            ValidationRule::DataCompleteness => self.check_data_completeness(knowledge).await,
        }
    }

    /// Check semantic consistency of knowledge
    async fn check_semantic_consistency(&self, knowledge: &EntityKnowledge) -> Result<Option<ValidationIssue>, AnalysisError> {
        // Check if semantic purpose aligns with structural patterns
        let purpose_lower = knowledge.semantic.purpose.to_lowercase();
        let has_matching_patterns = knowledge.structural.patterns.iter()
            .any(|pattern| purpose_lower.contains(&pattern.to_lowercase()));

        if !has_matching_patterns && !knowledge.structural.patterns.is_empty() {
            return Ok(Some(ValidationIssue {
                issue_type: IssueType::SemanticInconsistency,
                entity_id: knowledge.entity_id.clone(),
                description: "Semantic purpose doesn't align with structural patterns".to_string(),
                severity: 0.6,
                suggested_fix: "Review semantic analysis or structural pattern detection".to_string(),
            }));
        }

        Ok(None)
    }

    /// Check structural validity
    async fn check_structural_validity(&self, knowledge: &EntityKnowledge) -> Result<Option<ValidationIssue>, AnalysisError> {
        // Check for impossible metrics (e.g., negative line counts)
        for (metric_name, &value) in &knowledge.structural.metrics {
            if metric_name.contains("count") && value < 0.0 {
                return Ok(Some(ValidationIssue {
                    issue_type: IssueType::StructuralInvalidity,
                    entity_id: knowledge.entity_id.clone(),
                    description: format!("Invalid metric value: {} = {}", metric_name, value),
                    severity: 0.9,
                    suggested_fix: "Check metric calculation logic".to_string(),
                }));
            }
        }

        Ok(None)
    }

    /// Check security relevance
    async fn check_security_relevance(&self, knowledge: &EntityKnowledge) -> Result<Option<ValidationIssue>, AnalysisError> {
        // If entity has high attack surface but no risk indicators, flag as inconsistent
        if knowledge.security.attack_surface > 0.7 && knowledge.security.risk_indicators.is_empty() {
            return Ok(Some(ValidationIssue {
                issue_type: IssueType::SecurityInconsistency,
                entity_id: knowledge.entity_id.clone(),
                description: "High attack surface but no identified risk indicators".to_string(),
                severity: 0.7,
                suggested_fix: "Review risk indicator identification logic".to_string(),
            }));
        }

        Ok(None)
    }

    /// Check confidence threshold
    async fn check_confidence_threshold(&self, knowledge: &EntityKnowledge) -> Result<Option<ValidationIssue>, AnalysisError> {
        if knowledge.semantic.confidence < 0.3 {
            return Ok(Some(ValidationIssue {
                issue_type: IssueType::LowConfidence,
                entity_id: knowledge.entity_id.clone(),
                description: format!("Low confidence score: {:.2}", knowledge.semantic.confidence),
                severity: 0.5,
                suggested_fix: "Consider gathering more evidence or improving analysis".to_string(),
            }));
        }

        Ok(None)
    }

    /// Check data completeness
    async fn check_data_completeness(&self, knowledge: &EntityKnowledge) -> Result<Option<ValidationIssue>, AnalysisError> {
        let mut missing_fields = Vec::new();

        if knowledge.structural.properties.is_empty() {
            missing_fields.push("structural properties");
        }
        if knowledge.semantic.concepts.is_empty() {
            missing_fields.push("semantic concepts");
        }
        if knowledge.security.trust_boundaries.is_empty() && knowledge.security.attack_surface > 0.5 {
            missing_fields.push("trust boundaries");
        }

        if !missing_fields.is_empty() {
            return Ok(Some(ValidationIssue {
                issue_type: IssueType::IncompleteData,
                entity_id: knowledge.entity_id.clone(),
                description: format!("Missing data fields: {}", missing_fields.join(", ")),
                severity: 0.4,
                suggested_fix: "Ensure all extraction modules are functioning correctly".to_string(),
            }));
        }

        Ok(None)
    }

    /// Validate relationships for consistency
    async fn validate_relationships(&self, relationships: &[CodeRelationship]) -> Result<Vec<ValidationIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Check for impossible relationships
        for rel in relationships {
            if rel.from_entity == rel.to_entity {
                issues.push(ValidationIssue {
                    issue_type: IssueType::SelfRelationship,
                    entity_id: rel.from_entity.clone(),
                    description: "Entity has relationship with itself".to_string(),
                    severity: 0.8,
                    suggested_fix: "Check relationship extraction logic".to_string(),
                });
            }

            if rel.strength < 0.0 || rel.strength > 1.0 {
                issues.push(ValidationIssue {
                    issue_type: IssueType::InvalidStrength,
                    entity_id: rel.from_entity.clone(),
                    description: format!("Invalid relationship strength: {}", rel.strength),
                    severity: 0.6,
                    suggested_fix: "Normalize relationship strength values".to_string(),
                });
            }
        }

        // Check for circular inheritance
        if self.has_circular_inheritance(relationships) {
            issues.push(ValidationIssue {
                issue_type: IssueType::CircularInheritance,
                entity_id: "multiple".to_string(),
                description: "Circular inheritance detected".to_string(),
                severity: 0.9,
                suggested_fix: "Review inheritance relationship extraction".to_string(),
            });
        }

        Ok(issues)
    }

    /// Apply consistency checks
    async fn apply_consistency_checks(&self, knowledge: &BatchKnowledgeResult) -> Result<Vec<ValidationIssue>, AnalysisError> {
        let mut issues = Vec::new();

        for check in &self.consistency_checks {
            let check_issues = match check {
                ConsistencyCheck::EntityRelationshipConsistency => {
                    self.check_entity_relationship_consistency(knowledge).await?
                }
                ConsistencyCheck::MetricRangeConsistency => {
                    self.check_metric_range_consistency(knowledge).await?
                }
                ConsistencyCheck::SemanticStructuralAlignment => {
                    self.check_semantic_structural_alignment(knowledge).await?
                }
            };
            issues.extend(check_issues);
        }

        Ok(issues)
    }

    /// Check entity-relationship consistency
    async fn check_entity_relationship_consistency(&self, knowledge: &BatchKnowledgeResult) -> Result<Vec<ValidationIssue>, AnalysisError> {
        let mut issues = Vec::new();
        let entity_ids: HashSet<_> = knowledge.entity_knowledge.iter()
            .map(|e| e.entity_id.as_str())
            .collect();

        // Check that all relationships reference existing entities
        for rel in &knowledge.relationships {
            if !entity_ids.contains(rel.from_entity.as_str()) {
                issues.push(ValidationIssue {
                    issue_type: IssueType::MissingEntity,
                    entity_id: rel.from_entity.clone(),
                    description: "Relationship references non-existent source entity".to_string(),
                    severity: 0.8,
                    suggested_fix: "Ensure entity extraction completeness".to_string(),
                });
            }

            if !entity_ids.contains(rel.to_entity.as_str()) {
                issues.push(ValidationIssue {
                    issue_type: IssueType::MissingEntity,
                    entity_id: rel.to_entity.clone(),
                    description: "Relationship references non-existent target entity".to_string(),
                    severity: 0.8,
                    suggested_fix: "Ensure entity extraction completeness".to_string(),
                });
            }
        }

        Ok(issues)
    }

    /// Check metric range consistency
    async fn check_metric_range_consistency(&self, knowledge: &BatchKnowledgeResult) -> Result<Vec<ValidationIssue>, AnalysisError> {
        let mut issues = Vec::new();

        // Collect all metric values for range checking
        let mut metric_ranges: HashMap<String, (f64, f64)> = HashMap::new();

        for entity in &knowledge.entity_knowledge {
            for (metric_name, &value) in &entity.structural.metrics {
                let entry = metric_ranges.entry(metric_name.clone()).or_insert((value, value));
                entry.0 = entry.0.min(value);
                entry.1 = entry.1.max(value);
            }
        }

        // Check for suspicious ranges
        for (metric_name, (min_val, max_val)) in metric_ranges {
            if max_val - min_val > 1000.0 && metric_name.contains("count") {
                issues.push(ValidationIssue {
                    issue_type: IssueType::SuspiciousRange,
                    entity_id: "global".to_string(),
                    description: format!("Suspicious range for {}: {} to {}", metric_name, min_val, max_val),
                    severity: 0.5,
                    suggested_fix: "Review metric calculation consistency".to_string(),
                });
            }
        }

        Ok(issues)
    }

    /// Check semantic-structural alignment
    async fn check_semantic_structural_alignment(&self, knowledge: &BatchKnowledgeResult) -> Result<Vec<ValidationIssue>, AnalysisError> {
        let mut issues = Vec::new();

        for entity in &knowledge.entity_knowledge {
            // Check if high complexity entities have appropriate semantic complexity indicators
            if let Some(&complexity) = entity.structural.metrics.get("cyclomatic_complexity") {
                if complexity > 10.0 && !entity.semantic.concepts.iter().any(|c| c.to_lowercase().contains("complex")) {
                    issues.push(ValidationIssue {
                        issue_type: IssueType::SemanticStructuralMismatch,
                        entity_id: entity.entity_id.clone(),
                        description: "High structural complexity not reflected in semantic analysis".to_string(),
                        severity: 0.4,
                        suggested_fix: "Improve semantic analysis of complex entities".to_string(),
                    });
                }
            }
        }

        Ok(issues)
    }

    /// Validate an inferred fact
    async fn validate_inferred_fact(
        &self,
        fact: &InferredFact,
        base_knowledge: &BatchKnowledgeResult,
    ) -> Result<FactValidation, AnalysisError> {
        let supporting_evidence = self.find_supporting_evidence(fact, base_knowledge);
        let confidence_adjustment = self.calculate_confidence_adjustment(fact, &supporting_evidence);

        Ok(FactValidation {
            fact_id: fact.id.clone(),
            is_supported: !supporting_evidence.is_empty(),
            supporting_evidence,
            confidence_adjustment,
            reliability_score: fact.confidence * confidence_adjustment,
        })
    }

    /// Validate a security insight
    async fn validate_security_insight(
        &self,
        insight: &SecurityInsight,
        base_knowledge: &BatchKnowledgeResult,
    ) -> Result<InsightValidation, AnalysisError> {
        let affected_entities_exist = insight.affected_entities.iter()
            .all(|entity_id| base_knowledge.entity_knowledge.iter()
                .any(|e| e.entity_id == *entity_id));

        let severity_justified = self.is_severity_justified(insight, base_knowledge);

        Ok(InsightValidation {
            insight_description: insight.description.clone(),
            entities_exist: affected_entities_exist,
            severity_justified,
            actionability_score: self.calculate_actionability_score(insight),
        })
    }

    /// Detect logical contradictions in inferred knowledge
    async fn detect_logical_contradictions(&self, inference: &InferenceResult) -> Result<Vec<LogicalContradiction>, AnalysisError> {
        let mut contradictions = Vec::new();

        // Check for contradictory facts about the same subject
        let mut fact_groups: HashMap<String, Vec<&InferredFact>> = HashMap::new();
        for fact in &inference.inferred_facts {
            fact_groups.entry(fact.subject.clone()).or_default().push(fact);
        }

        for (subject, facts) in fact_groups {
            if facts.len() > 1 {
                // Look for contradictory fact types
                let fact_types: HashSet<_> = facts.iter()
                    .map(|f| std::mem::discriminant(&f.fact_type))
                    .collect();

                if fact_types.len() != facts.len() {
                    // Multiple facts of same type about same subject - potential contradiction
                    contradictions.push(LogicalContradiction {
                        subject,
                        contradicting_facts: facts.iter().map(|f| f.id.clone()).collect(),
                        contradiction_type: ContradictionType::ConflictingFacts,
                        severity: 0.7,
                    });
                }
            }
        }

        Ok(contradictions)
    }

    /// Helper methods for validation logic
    fn has_circular_inheritance(&self, relationships: &[CodeRelationship]) -> bool {
        let inheritance_edges: Vec<_> = relationships.iter()
            .filter(|r| matches!(r.relationship_type, RelationshipType::Inherits))
            .collect();

        // Simple cycle detection using DFS
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for edge in &inheritance_edges {
            if !visited.contains(&edge.from_entity) {
                if self.has_cycle_dfs(&edge.from_entity, &inheritance_edges, &mut visited, &mut rec_stack) {
                    return true;
                }
            }
        }

        false
    }

    fn has_cycle_dfs(
        &self,
        node: &str,
        edges: &[&CodeRelationship],
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> bool {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());

        for edge in edges {
            if edge.from_entity == node {
                if !visited.contains(&edge.to_entity) {
                    if self.has_cycle_dfs(&edge.to_entity, edges, visited, rec_stack) {
                        return true;
                    }
                } else if rec_stack.contains(&edge.to_entity) {
                    return true;
                }
            }
        }

        rec_stack.remove(node);
        false
    }

    fn compare_entity_knowledge(&self, primary: &EntityKnowledge, secondary: &EntityKnowledge) -> KnowledgeComparison {
        let mut agreement_score = 0.0;
        let mut differences = Vec::new();

        // Compare structural metrics
        let common_metrics: HashSet<_> = primary.structural.metrics.keys()
            .filter(|k| secondary.structural.metrics.contains_key(*k))
            .collect();

        if !common_metrics.is_empty() {
            let metric_agreement = common_metrics.iter()
                .map(|metric| {
                    let primary_val = primary.structural.metrics[*metric];
                    let secondary_val = secondary.structural.metrics[*metric];
                    1.0 - (primary_val - secondary_val).abs() / (primary_val.max(secondary_val) + 1.0)
                })
                .sum::<f64>() / common_metrics.len() as f64;

            agreement_score += metric_agreement * 0.4;
        }

        // Compare semantic concepts
        let primary_concepts: HashSet<_> = primary.semantic.concepts.iter().collect();
        let secondary_concepts: HashSet<_> = secondary.semantic.concepts.iter().collect();
        let concept_overlap = primary_concepts.intersection(&secondary_concepts).count() as f64;
        let concept_union = primary_concepts.union(&secondary_concepts).count() as f64;

        if concept_union > 0.0 {
            agreement_score += (concept_overlap / concept_union) * 0.3;
        }

        // Compare security indicators
        let primary_risks: HashSet<_> = primary.security.risk_indicators.iter().collect();
        let secondary_risks: HashSet<_> = secondary.security.risk_indicators.iter().collect();
        let risk_overlap = primary_risks.intersection(&secondary_risks).count() as f64;
        let risk_union = primary_risks.union(&secondary_risks).count() as f64;

        if risk_union > 0.0 {
            agreement_score += (risk_overlap / risk_union) * 0.3;
        }

        KnowledgeComparison {
            agreement_score,
            differences,
        }
    }

    fn find_supporting_evidence(&self, _fact: &InferredFact, _base_knowledge: &BatchKnowledgeResult) -> Vec<String> {
        // Placeholder implementation
        Vec::new()
    }

    fn calculate_confidence_adjustment(&self, _fact: &InferredFact, evidence: &[String]) -> f64 {
        if evidence.is_empty() {
            0.5
        } else {
            1.0
        }
    }

    fn is_severity_justified(&self, _insight: &SecurityInsight, _base_knowledge: &BatchKnowledgeResult) -> bool {
        // Placeholder implementation
        true
    }

    fn calculate_actionability_score(&self, _insight: &SecurityInsight) -> f64 {
        // Placeholder implementation
        0.7
    }

    fn calculate_validation_score(&self, issues: &[ValidationIssue]) -> f64 {
        if issues.is_empty() {
            1.0
        } else {
            let total_severity: f64 = issues.iter().map(|i| i.severity).sum();
            (1.0 - (total_severity / issues.len() as f64)).max(0.0)
        }
    }

    fn calculate_inference_confidence(&self, validations: &[FactValidation]) -> f64 {
        if validations.is_empty() {
            0.0
        } else {
            validations.iter().map(|v| v.reliability_score).sum::<f64>() / validations.len() as f64
        }
    }

    fn calculate_overall_agreement(&self, agreements: &[KnowledgeAgreement], disagreements: &[KnowledgeDisagreement]) -> f64 {
        let total_items = agreements.len() + disagreements.len();
        if total_items == 0 {
            0.0
        } else {
            agreements.len() as f64 / total_items as f64
        }
    }

    fn default_validation_rules() -> Vec<ValidationRule> {
        vec![
            ValidationRule::SemanticConsistency,
            ValidationRule::StructuralValidity,
            ValidationRule::SecurityRelevance,
            ValidationRule::ConfidenceThreshold,
            ValidationRule::DataCompleteness,
        ]
    }

    fn default_consistency_checks() -> Vec<ConsistencyCheck> {
        vec![
            ConsistencyCheck::EntityRelationshipConsistency,
            ConsistencyCheck::MetricRangeConsistency,
            ConsistencyCheck::SemanticStructuralAlignment,
        ]
    }
}

/// Validation rules for knowledge quality
#[derive(Debug, Clone)]
pub enum ValidationRule {
    SemanticConsistency,
    StructuralValidity,
    SecurityRelevance,
    ConfidenceThreshold,
    DataCompleteness,
}

/// Consistency checks for knowledge coherence
#[derive(Debug, Clone)]
pub enum ConsistencyCheck {
    EntityRelationshipConsistency,
    MetricRangeConsistency,
    SemanticStructuralAlignment,
}

/// Result of knowledge validation
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub issues: Vec<ValidationIssue>,
    pub confidence_adjustments: HashMap<String, f64>,
    pub validation_score: f64,
}

/// Individual validation issue
#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub issue_type: IssueType,
    pub entity_id: String,
    pub description: String,
    pub severity: f64,
    pub suggested_fix: String,
}

/// Types of validation issues
#[derive(Debug, Clone)]
pub enum IssueType {
    SemanticInconsistency,
    StructuralInvalidity,
    SecurityInconsistency,
    LowConfidence,
    IncompleteData,
    SelfRelationship,
    InvalidStrength,
    CircularInheritance,
    MissingEntity,
    SuspiciousRange,
    SemanticStructuralMismatch,
}

/// Validation result for inferred knowledge
#[derive(Debug, Clone)]
pub struct InferenceValidation {
    pub fact_validations: Vec<FactValidation>,
    pub insight_validations: Vec<InsightValidation>,
    pub contradictions: Vec<LogicalContradiction>,
    pub overall_confidence: f64,
}

/// Validation of a single inferred fact
#[derive(Debug, Clone)]
pub struct FactValidation {
    pub fact_id: String,
    pub is_supported: bool,
    pub supporting_evidence: Vec<String>,
    pub confidence_adjustment: f64,
    pub reliability_score: f64,
}

/// Validation of a security insight
#[derive(Debug, Clone)]
pub struct InsightValidation {
    pub insight_description: String,
    pub entities_exist: bool,
    pub severity_justified: bool,
    pub actionability_score: f64,
}

/// Logical contradiction in inferred knowledge
#[derive(Debug, Clone)]
pub struct LogicalContradiction {
    pub subject: String,
    pub contradicting_facts: Vec<String>,
    pub contradiction_type: ContradictionType,
    pub severity: f64,
}

/// Types of logical contradictions
#[derive(Debug, Clone)]
pub enum ContradictionType {
    ConflictingFacts,
    ImpossibleCombination,
    CircularReasoning,
}

/// Cross-validation result
#[derive(Debug, Clone)]
pub struct CrossValidationResult {
    pub agreements: Vec<KnowledgeAgreement>,
    pub disagreements: Vec<KnowledgeDisagreement>,
    pub unique_insights: Vec<UniqueInsight>,
    pub overall_agreement_score: f64,
}

/// Agreement between knowledge sources
#[derive(Debug, Clone)]
pub struct KnowledgeAgreement {
    pub entity_id: String,
    pub agreement_type: AgreementType,
    pub confidence: f64,
    pub sources: Vec<String>,
}

/// Types of knowledge agreements
#[derive(Debug, Clone)]
pub enum AgreementType {
    EntityKnowledge,
    RelationshipConsistency,
    SecurityAssessment,
}

/// Disagreement between knowledge sources
#[derive(Debug, Clone)]
pub struct KnowledgeDisagreement {
    pub entity_id: String,
    pub disagreement_type: DisagreementType,
    pub severity: f64,
    pub details: Vec<String>,
}

/// Types of knowledge disagreements
#[derive(Debug, Clone)]
pub enum DisagreementType {
    ConflictingFacts,
    DifferentAssessments,
    MissingInformation,
}

/// Unique insight from a single source
#[derive(Debug, Clone)]
pub struct UniqueInsight {
    pub entity_id: String,
    pub source: String,
    pub insight_type: InsightType,
    pub confidence: f64,
}

/// Types of unique insights
#[derive(Debug, Clone)]
pub enum InsightType {
    EntitySpecific,
    RelationshipSpecific,
    SecuritySpecific,
}

/// Knowledge comparison result
#[derive(Debug, Clone)]
pub struct KnowledgeComparison {
    pub agreement_score: f64,
    pub differences: Vec<String>,
}