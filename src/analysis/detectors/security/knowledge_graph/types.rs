//! Core data structures for the knowledge graph system
//!
//! This module contains all the fundamental types used throughout the
//! knowledge graph implementation for security analysis.

use crate::analysis::detectors::security::types::{SecurityIssue, SecurityIssueType};
use crate::ast::SourceLanguage;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Core code entity representation in the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeEntity {
    pub id: String,
    pub name: String,
    pub entity_type: EntityType,
    pub location: CodeLocation,
    pub metadata: HashMap<String, serde_json::Value>,
    pub language: SourceLanguage,
}

/// Types of code entities that can be represented
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

/// Location information for code entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeLocation {
    pub file_path: PathBuf,
    pub start_line: u32,
    pub end_line: u32,
    pub start_column: u32,
    pub end_column: u32,
}

/// Relationships between code entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeRelationship {
    pub from_entity: String,
    pub to_entity: String,
    pub relationship_type: RelationshipType,
    pub strength: f64,
}

/// Types of relationships between entities
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

/// AI-generated semantic enrichment for entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticEnrichment {
    pub entity_id: String,
    pub summary: String,
    pub security_insights: Vec<String>,
    pub architectural_patterns: Vec<String>,
    pub risk_indicators: Vec<String>,
    pub confidence_score: f64,
}

/// Concept mapping for semantic understanding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptMapping {
    pub concept: String,
    pub entities: Vec<String>,
    pub relevance_score: f64,
}

/// Anti-pattern mapping in the codebase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiPatternMapping {
    pub pattern_type: String,
    pub affected_entities: Vec<String>,
    pub severity: f64,
}

/// Historical security patterns for learning
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

/// Temporal trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalTrend {
    pub pattern_type: SecurityIssueType,
    pub trend_direction: TrendDirection,
    pub confidence: f64,
    pub time_range: (chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>),
}

/// Direction of security trend patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
    Volatile,
}

/// Query types for the knowledge graph
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

/// Comprehensive knowledge graph query results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityKnowledgeResult {
    pub structural_facts: StructuralQueryResult,
    pub semantic_insights: SemanticQueryResult,
    pub contextual_information: RAGQueryResult,
    pub historical_patterns: Vec<HistoricalPattern>,
    pub confidence_score: f64,
}

/// Results from structural graph queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuralQueryResult {
    pub entities: Vec<CodeEntity>,
    pub relationships: Vec<CodeRelationship>,
    pub anti_patterns: Vec<AntiPatternInfo>,
}

/// Results from semantic graph queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticQueryResult {
    pub insights: Vec<String>,
    pub concepts: Vec<String>,
    pub confidence_score: f64,
}

/// Results from RAG system queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAGQueryResult {
    pub relevant_contexts: Vec<String>,
    pub similarity_scores: HashMap<String, f64>,
    pub retrieved_facts: Vec<String>,
}

/// Historical pattern information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalPattern {
    pub pattern_description: String,
    pub frequency: usize,
    pub trend: TrendDirection,
    pub relevance_score: f64,
}

/// Anti-pattern information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiPatternInfo {
    pub pattern_type: String,
    pub description: String,
    pub severity: f64,
    pub affected_entities: Vec<String>,
}

/// Correlation between security issues and architecture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitecturalCorrelation {
    pub security_issue_id: String,
    pub architectural_patterns: Vec<AntiPatternInfo>,
    pub correlation_strength: f64,
    pub amplification_factors: HashMap<String, f64>,
    pub recommendations: Vec<String>,
}

/// File analysis input for knowledge graph construction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAnalysis {
    pub file_path: PathBuf,
    pub entities: Vec<CodeEntity>,
    pub security_issues: Vec<SecurityIssue>,
    pub anti_patterns: Vec<AntiPatternInfo>,
}

/// Unified graph representation for multi-language support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuralSemanticGraph {
    pub nodes: HashMap<String, GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub metadata: GraphMetadata,
}

/// Individual graph node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub node_type: GraphNodeType,
    pub properties: HashMap<String, serde_json::Value>,
    pub location: Option<CodeLocation>,
}

/// Types of graph nodes
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

/// Graph edge representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub edge_type: GraphEdgeType,
    pub weight: f64,
}

/// Types of graph edges
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphEdgeType {
    CallsTo,
    DataFlow,
    ControlFlow,
    Inheritance,
    Composition,
    Dependency,
}

/// Metadata about the graph structure
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