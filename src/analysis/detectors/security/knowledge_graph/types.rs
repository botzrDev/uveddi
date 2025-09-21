//! Core data structures for the knowledge graph system

use crate::analysis::detectors::security::types::{SecurityIssue, SecurityIssueType};
use crate::ast::SourceLanguage;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeEntity {
    pub id: String,
    pub name: String,
    pub entity_type: EntityType,
    pub location: CodeLocation,
    pub metadata: HashMap<String, serde_json::Value>,
    pub language: SourceLanguage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityType {
    Function,
    Class,
    Module,
    Variable,
    Constant,
    Interface,
    Struct,
    Enum,
    Trait,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeLocation {
    pub file_path: PathBuf,
    pub start_line: u32,
    pub end_line: u32,
    pub start_column: u32,
    pub end_column: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeRelationship {
    pub from_entity: String,
    pub to_entity: String,
    pub relationship_type: RelationshipType,
    pub strength: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationshipType {
    Calls,
    Inherits,
    Implements,
    Uses,
    Contains,
    DependsOn,
    Imports,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticEnrichment {
    pub entity_id: String,
    pub summary: String,
    pub security_insights: Vec<String>,
    pub architectural_patterns: Vec<String>,
    pub risk_indicators: Vec<String>,
    pub confidence_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptMapping {
    pub concept: String,
    pub entities: Vec<String>,
    pub relevance_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiPatternMapping {
    pub pattern_type: String,
    pub affected_entities: Vec<String>,
    pub severity: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPattern {
    pub pattern_type: SecurityIssueType,
    pub location_pattern: PathBuf,
    pub frequency: usize,
    pub severity_distribution: Vec<(
        crate::analysis::detectors::security::types::SecuritySeverity,
        usize,
    )>,
    pub temporal_data: Vec<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalTrend {
    pub pattern_type: SecurityIssueType,
    pub trend_direction: TrendDirection,
    pub confidence: f64,
    pub time_range: (chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
    Volatile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityQuery {
    ArchitecturalCorrelation {
        location: crate::analysis::detectors::security::types::SecurityLocation,
        issue_type: SecurityIssueType,
        context_radius: usize,
    },
    DependencyAnalysis {
        entity_id: String,
    },
    PatternSearch {
        pattern_type: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityKnowledgeResult {
    pub structural_facts: StructuralQueryResult,
    pub semantic_insights: SemanticQueryResult,
    pub contextual_information: RAGQueryResult,
    pub historical_patterns: Vec<HistoricalPattern>,
    pub confidence_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StructuralQueryResult {
    pub entities: Vec<CodeEntity>,
    pub relationships: Vec<CodeRelationship>,
    pub anti_patterns: Vec<AntiPatternInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticQueryResult {
    pub insights: Vec<String>,
    pub concepts: Vec<String>,
    pub confidence_score: f64,
}

impl Default for SemanticQueryResult {
    fn default() -> Self {
        Self {
            insights: Vec::new(),
            concepts: Vec::new(),
            confidence_score: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAGQueryResult {
    pub relevant_contexts: Vec<String>,
    pub similarity_scores: HashMap<String, f64>,
    pub retrieved_facts: Vec<String>,
}

impl Default for RAGQueryResult {
    fn default() -> Self {
        Self {
            relevant_contexts: Vec::new(),
            similarity_scores: HashMap::new(),
            retrieved_facts: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalPattern {
    pub pattern_description: String,
    pub frequency: usize,
    pub trend: TrendDirection,
    pub relevance_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiPatternInfo {
    pub pattern_type: String,
    pub description: String,
    pub severity: f64,
    pub affected_entities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitecturalCorrelation {
    pub security_issue_id: String,
    pub architectural_patterns: Vec<AntiPatternInfo>,
    pub correlation_strength: f64,
    pub amplification_factors: HashMap<String, f64>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAnalysis {
    pub file_path: PathBuf,
    pub entities: Vec<CodeEntity>,
    pub security_issues: Vec<SecurityIssue>,
    pub anti_patterns: Vec<AntiPatternInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuralSemanticGraph {
    pub nodes: HashMap<String, GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub metadata: GraphMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub node_type: GraphNodeType,
    pub properties: HashMap<String, serde_json::Value>,
    pub location: Option<CodeLocation>,
    pub code_entity: Option<CodeEntity>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphNodeType {
    Function,
    Class,
    Module,
    Variable,
    Call,
    Expression,
    Statement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub edge_type: GraphEdgeType,
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphEdgeType {
    CallsTo,
    DataFlow,
    ControlFlow,
    Inheritance,
    Composition,
    Dependency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetadata {
    pub language: SourceLanguage,
    pub creation_time: chrono::DateTime<chrono::Utc>,
    pub node_count: usize,
    pub edge_count: usize,
}

impl StructuralSemanticGraph {
    pub fn new(language: SourceLanguage) -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            metadata: GraphMetadata {
                language,
                creation_time: chrono::Utc::now(),
                node_count: 0,
                edge_count: 0,
            },
        }
    }

    pub fn add_node(&mut self, node: GraphNode) {
        self.nodes.insert(node.id.clone(), node);
        self.metadata.node_count = self.nodes.len();
    }

    pub fn add_edge(&mut self, edge: GraphEdge) {
        self.edges.push(edge);
        self.metadata.edge_count = self.edges.len();
    }
}
