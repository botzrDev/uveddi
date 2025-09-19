//! Knowledge extraction and management components
//!
//! This module provides the knowledge management layer for the security knowledge graph,
//! including extraction, inference, and validation of knowledge from code entities.

pub mod extraction;
pub mod inference;
pub mod validation;

pub use extraction::{KnowledgeExtractor, EntityKnowledge, BatchKnowledgeResult};
pub use inference::{InferenceEngine, InferenceResult, InferredFact, SecurityInsight};
pub use validation::{KnowledgeValidator, ValidationResult, InferenceValidation};

use crate::analysis::detectors::security::knowledge_graph::types::{
    CodeEntity, CodeRelationship, SecurityQuery,
};
use crate::analysis::AnalysisError;
use crate::ast::SourceLanguage;
use tracing::info;

/// Unified knowledge management interface
pub struct KnowledgeManager {
    extractor: KnowledgeExtractor,
    inference_engine: InferenceEngine,
    validator: KnowledgeValidator,
}

impl KnowledgeManager {
    /// Create a new knowledge manager
    pub fn new(language: SourceLanguage) -> Self {
        Self {
            extractor: KnowledgeExtractor::new(language),
            inference_engine: InferenceEngine::new(),
            validator: KnowledgeValidator::new(),
        }
    }

    /// Process entities to extract and validate knowledge
    pub async fn process_entities(
        &mut self,
        entities: &[CodeEntity],
    ) -> Result<ProcessedKnowledge, AnalysisError> {
        info!("Processing {} entities for knowledge extraction", entities.len());

        // Extract base knowledge
        let extracted_knowledge = self.extractor.extract_batch_knowledge(entities).await?;

        // Validate extracted knowledge
        let extraction_validation = self.validator
            .validate_extracted_knowledge(&extracted_knowledge)
            .await?;

        // Perform inference on validated knowledge
        let inferred_knowledge = self.inference_engine
            .infer_knowledge(entities, &extracted_knowledge.relationships)
            .await?;

        // Validate inferred knowledge
        let inference_validation = self.validator
            .validate_inferred_knowledge(&inferred_knowledge, &extracted_knowledge)
            .await?;

        Ok(ProcessedKnowledge {
            extracted: extracted_knowledge,
            inferred: inferred_knowledge,
            extraction_validation,
            inference_validation,
        })
    }

    /// Query processed knowledge
    pub async fn query_knowledge(
        &self,
        query: &SecurityQuery,
        processed_knowledge: &ProcessedKnowledge,
    ) -> Result<KnowledgeQueryResult, AnalysisError> {
        info!("Querying knowledge: {:?}", query);

        // Query inference engine
        let inference_result = self.inference_engine
            .query_inferred_knowledge(query)
            .await?;

        // Find relevant extracted knowledge
        let relevant_entities = self.find_relevant_entities(query, &processed_knowledge.extracted);

        Ok(KnowledgeQueryResult {
            relevant_entities,
            inferred_facts: inference_result.relevant_facts,
            inferred_relationships: inference_result.inferred_relationships,
            correlations: inference_result.correlations,
            confidence_score: inference_result.confidence_score,
        })
    }

    /// Get knowledge statistics
    pub fn get_knowledge_statistics(&self, knowledge: &ProcessedKnowledge) -> KnowledgeStatistics {
        KnowledgeStatistics {
            total_entities: knowledge.extracted.entity_knowledge.len(),
            total_relationships: knowledge.extracted.relationships.len(),
            inferred_facts: knowledge.inferred.inferred_facts.len(),
            security_insights: knowledge.inferred.security_insights.len(),
            validation_score: knowledge.extraction_validation.validation_score,
            inference_confidence: knowledge.inference_validation.overall_confidence,
        }
    }

    /// Find entities relevant to a query
    fn find_relevant_entities(
        &self,
        query: &SecurityQuery,
        extracted_knowledge: &BatchKnowledgeResult,
    ) -> Vec<EntityKnowledge> {
        match query {
            SecurityQuery::ArchitecturalCorrelation { location, .. } => {
                // Find entities in the same file
                extracted_knowledge.entity_knowledge.iter()
                    .filter(|knowledge| {
                        // This would need to be implemented based on entity location data
                        knowledge.entity_id.contains(&location.file_path.display().to_string())
                    })
                    .cloned()
                    .collect()
            }
            SecurityQuery::DependencyAnalysis { entity_id } => {
                // Find the specific entity and its dependencies
                extracted_knowledge.entity_knowledge.iter()
                    .filter(|knowledge| {
                        knowledge.entity_id == *entity_id ||
                        self.is_dependency_related(entity_id, &knowledge.entity_id, &extracted_knowledge.relationships)
                    })
                    .cloned()
                    .collect()
            }
            SecurityQuery::PatternSearch { pattern_type } => {
                // Find entities matching the pattern
                extracted_knowledge.entity_knowledge.iter()
                    .filter(|knowledge| {
                        knowledge.structural.patterns.iter()
                            .any(|pattern| pattern.contains(pattern_type)) ||
                        knowledge.semantic.concepts.iter()
                            .any(|concept| concept.contains(pattern_type))
                    })
                    .cloned()
                    .collect()
            }
        }
    }

    /// Check if entities are related through dependencies
    fn is_dependency_related(
        &self,
        entity1: &str,
        entity2: &str,
        relationships: &[CodeRelationship],
    ) -> bool {
        relationships.iter().any(|rel| {
            (rel.from_entity == entity1 && rel.to_entity == entity2) ||
            (rel.from_entity == entity2 && rel.to_entity == entity1)
        })
    }
}

/// Complete processed knowledge result
#[derive(Debug, Clone)]
pub struct ProcessedKnowledge {
    pub extracted: BatchKnowledgeResult,
    pub inferred: InferenceResult,
    pub extraction_validation: ValidationResult,
    pub inference_validation: InferenceValidation,
}

/// Result of querying the knowledge base
#[derive(Debug, Clone)]
pub struct KnowledgeQueryResult {
    pub relevant_entities: Vec<EntityKnowledge>,
    pub inferred_facts: Vec<InferredFact>,
    pub inferred_relationships: Vec<CodeRelationship>,
    pub correlations: Vec<crate::analysis::detectors::security::knowledge_graph::types::ArchitecturalCorrelation>,
    pub confidence_score: f64,
}

/// Statistics about the knowledge base
#[derive(Debug, Clone)]
pub struct KnowledgeStatistics {
    pub total_entities: usize,
    pub total_relationships: usize,
    pub inferred_facts: usize,
    pub security_insights: usize,
    pub validation_score: f64,
    pub inference_confidence: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::detectors::security::knowledge_graph::types::{
        EntityType, CodeLocation,
    };
    use std::path::PathBuf;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_knowledge_manager_creation() {
        let manager = KnowledgeManager::new(SourceLanguage::Rust);
        // Should create without error
    }

    #[tokio::test]
    async fn test_entity_processing() {
        let mut manager = KnowledgeManager::new(SourceLanguage::Rust);

        let entity = CodeEntity {
            id: "test_entity".to_string(),
            name: "test_function".to_string(),
            entity_type: EntityType::Function,
            location: CodeLocation {
                file_path: PathBuf::from("test.rs"),
                start_line: 1,
                end_line: 10,
                start_column: 0,
                end_column: 0,
            },
            metadata: HashMap::new(),
            language: SourceLanguage::Rust,
        };

        let result = manager.process_entities(&[entity]).await;
        assert!(result.is_ok());

        let processed = result.unwrap();
        assert_eq!(processed.extracted.entity_knowledge.len(), 1);
    }
}