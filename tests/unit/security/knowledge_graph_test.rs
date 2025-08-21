//! Unit tests for security knowledge graph

use uveddi::analysis::detectors::security::knowledge_graph::*;
use uveddi::analysis::detectors::security::types::*;
use uveddi::ast::{ParsedFile, SourceLanguage};
use std::path::PathBuf;
use std::collections::HashMap;

#[test]
fn test_security_knowledge_graph_creation() {
    let graph = SecurityKnowledgeGraph::new();
    
    assert_eq!(graph.structural_graph.nodes.len(), 0);
    assert_eq!(graph.structural_graph.edges.len(), 0);
    assert_eq!(graph.semantic_graph.embeddings.len(), 0);
    assert_eq!(graph.security_correlations.len(), 0);
}

#[test]
fn test_structural_graph_add_node() {
    let mut graph = StructuralGraph::new();
    
    let node = StructuralNode {
        id: "node_1".to_string(),
        node_type: NodeType::Function,
        name: "calculate_hash".to_string(),
        file_path: PathBuf::from("src/crypto.rs"),
        line_number: 42,
        properties: HashMap::new(),
        security_annotations: Vec::new(),
    };
    
    graph.add_node(node.clone());
    
    assert_eq!(graph.nodes.len(), 1);
    assert!(graph.nodes.contains_key(&node.id));
}

#[test]
fn test_structural_graph_add_edge() {
    let mut graph = StructuralGraph::new();
    
    // Add nodes first
    let node1 = StructuralNode {
        id: "node_1".to_string(),
        node_type: NodeType::Function,
        name: "function1".to_string(),
        file_path: PathBuf::from("src/test.rs"),
        line_number: 10,
        properties: HashMap::new(),
        security_annotations: Vec::new(),
    };
    
    let node2 = StructuralNode {
        id: "node_2".to_string(),
        node_type: NodeType::Function,
        name: "function2".to_string(),
        file_path: PathBuf::from("src/test.rs"),
        line_number: 20,
        properties: HashMap::new(),
        security_annotations: Vec::new(),
    };
    
    graph.add_node(node1.clone());
    graph.add_node(node2.clone());
    
    let edge = StructuralEdge {
        id: "edge_1".to_string(),
        source_id: node1.id.clone(),
        target_id: node2.id.clone(),
        edge_type: EdgeType::Calls,
        weight: 1.0,
        security_implications: Vec::new(),
    };
    
    graph.add_edge(edge.clone());
    
    assert_eq!(graph.edges.len(), 1);
    assert!(graph.edges.contains_key(&edge.id));
}

#[test]
fn test_semantic_graph_creation() {
    let graph = SemanticGraph::new();
    
    assert_eq!(graph.embeddings.len(), 0);
    assert_eq!(graph.semantic_clusters.len(), 0);
    assert_eq!(graph.similarity_matrix.len(), 0);
}

#[test]
fn test_code_embedding_creation() {
    let embedding = CodeEmbedding {
        node_id: "func_123".to_string(),
        vector: vec![0.1, 0.2, 0.3, 0.4, 0.5],
        embedding_type: EmbeddingType::FunctionSignature,
        model_version: "v1.0".to_string(),
        created_at: chrono::Utc::now(),
    };
    
    assert_eq!(embedding.node_id, "func_123");
    assert_eq!(embedding.vector.len(), 5);
    assert_eq!(embedding.embedding_type, EmbeddingType::FunctionSignature);
    assert_eq!(embedding.model_version, "v1.0");
}

#[test]
fn test_semantic_cluster_creation() {
    let cluster = SemanticCluster {
        id: "cluster_1".to_string(),
        cluster_type: ClusterType::SecurityVulnerability,
        node_ids: vec!["node1".to_string(), "node2".to_string()],
        centroid: vec![0.5, 0.5, 0.5],
        confidence_score: 0.8,
        security_pattern: Some("SQL Injection".to_string()),
    };
    
    assert_eq!(cluster.id, "cluster_1");
    assert_eq!(cluster.cluster_type, ClusterType::SecurityVulnerability);
    assert_eq!(cluster.node_ids.len(), 2);
    assert_eq!(cluster.centroid.len(), 3);
    assert_eq!(cluster.confidence_score, 0.8);
}

#[test]
fn test_code_centric_rag_creation() {
    let rag = CodeCentricRAG::new();
    
    assert_eq!(rag.knowledge_base.len(), 0);
    assert_eq!(rag.query_cache.len(), 0);
}

#[test]
fn test_knowledge_entry_creation() {
    let entry = KnowledgeEntry {
        id: "entry_1".to_string(),
        content_type: ContentType::SecurityPattern,
        content: "SQL injection vulnerability pattern".to_string(),
        embedding: vec![0.1, 0.2, 0.3],
        metadata: serde_json::json!({
            "severity": "critical",
            "cwe_id": "CWE-89"
        }),
        tags: vec!["sql".to_string(), "injection".to_string()],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    assert_eq!(entry.id, "entry_1");
    assert_eq!(entry.content_type, ContentType::SecurityPattern);
    assert!(entry.content.contains("SQL injection"));
    assert_eq!(entry.embedding.len(), 3);
    assert_eq!(entry.tags.len(), 2);
}

#[test]
fn test_long_term_memory_creation() {
    let memory = LongTermMemory::new();
    
    assert_eq!(memory.historical_patterns.len(), 0);
    assert_eq!(memory.false_positive_patterns.len(), 0);
    assert_eq!(memory.remediation_strategies.len(), 0);
}

#[test]
fn test_historical_pattern_creation() {
    let pattern = HistoricalPattern {
        pattern_id: "pattern_123".to_string(),
        pattern_type: SecurityIssueType::Injection,
        frequency: 15,
        first_seen: chrono::Utc::now(),
        last_seen: chrono::Utc::now(),
        confidence_trend: vec![0.6, 0.7, 0.8, 0.9],
        remediation_success_rate: 0.85,
        false_positive_rate: 0.15,
    };
    
    assert_eq!(pattern.pattern_id, "pattern_123");
    assert_eq!(pattern.pattern_type, SecurityIssueType::Injection);
    assert_eq!(pattern.frequency, 15);
    assert_eq!(pattern.confidence_trend.len(), 4);
    assert_eq!(pattern.remediation_success_rate, 0.85);
}

#[test]
fn test_false_positive_pattern_creation() {
    let fp_pattern = FalsePositivePattern {
        pattern_signature: "test_*".to_string(),
        context: "unit tests".to_string(),
        frequency: 25,
        suppression_rule: "exclude files matching test_*".to_string(),
        confidence: 0.95,
    };
    
    assert_eq!(fp_pattern.pattern_signature, "test_*");
    assert_eq!(fp_pattern.context, "unit tests");
    assert_eq!(fp_pattern.frequency, 25);
    assert_eq!(fp_pattern.confidence, 0.95);
}

#[test]
fn test_remediation_strategy_creation() {
    let strategy = RemediationStrategy {
        vulnerability_type: SecurityIssueType::CrossSiteScripting,
        strategy_name: "Input Sanitization".to_string(),
        description: "Sanitize user inputs before rendering".to_string(),
        code_examples: vec!["DOMPurify.sanitize(userInput)".to_string()],
        success_rate: 0.9,
        implementation_difficulty: DifficultyLevel::Medium,
        estimated_effort_hours: 4,
    };
    
    assert_eq!(strategy.vulnerability_type, SecurityIssueType::CrossSiteScripting);
    assert_eq!(strategy.strategy_name, "Input Sanitization");
    assert_eq!(strategy.success_rate, 0.9);
    assert_eq!(strategy.implementation_difficulty, DifficultyLevel::Medium);
    assert_eq!(strategy.estimated_effort_hours, 4);
}

#[test]
fn test_node_type_variants() {
    let types = vec![
        NodeType::Function,
        NodeType::Class,
        NodeType::Module,
        NodeType::Variable,
        NodeType::Parameter,
        NodeType::Configuration,
    ];
    
    for node_type in types {
        match node_type {
            NodeType::Function => assert_eq!(node_type, NodeType::Function),
            NodeType::Class => assert_eq!(node_type, NodeType::Class),
            NodeType::Module => assert_eq!(node_type, NodeType::Module),
            NodeType::Variable => assert_eq!(node_type, NodeType::Variable),
            NodeType::Parameter => assert_eq!(node_type, NodeType::Parameter),
            NodeType::Configuration => assert_eq!(node_type, NodeType::Configuration),
        }
    }
}

#[test]
fn test_edge_type_variants() {
    let types = vec![
        EdgeType::Calls,
        EdgeType::References,
        EdgeType::Inherits,
        EdgeType::Contains,
        EdgeType::DependsOn,
        EdgeType::SecurityFlow,
    ];
    
    for edge_type in types {
        match edge_type {
            EdgeType::Calls => assert_eq!(edge_type, EdgeType::Calls),
            EdgeType::References => assert_eq!(edge_type, EdgeType::References),
            EdgeType::Inherits => assert_eq!(edge_type, EdgeType::Inherits),
            EdgeType::Contains => assert_eq!(edge_type, EdgeType::Contains),
            EdgeType::DependsOn => assert_eq!(edge_type, EdgeType::DependsOn),
            EdgeType::SecurityFlow => assert_eq!(edge_type, EdgeType::SecurityFlow),
        }
    }
}

#[test]
fn test_embedding_type_variants() {
    let types = vec![
        EmbeddingType::FunctionSignature,
        EmbeddingType::CodeStructure,
        EmbeddingType::SemanticMeaning,
        EmbeddingType::SecurityContext,
    ];
    
    for embedding_type in types {
        match embedding_type {
            EmbeddingType::FunctionSignature => assert_eq!(embedding_type, EmbeddingType::FunctionSignature),
            EmbeddingType::CodeStructure => assert_eq!(embedding_type, EmbeddingType::CodeStructure),
            EmbeddingType::SemanticMeaning => assert_eq!(embedding_type, EmbeddingType::SemanticMeaning),
            EmbeddingType::SecurityContext => assert_eq!(embedding_type, EmbeddingType::SecurityContext),
        }
    }
}

#[test]
fn test_cluster_type_variants() {
    let types = vec![
        ClusterType::SecurityVulnerability,
        ClusterType::AntiPattern,
        ClusterType::FunctionGroup,
        ClusterType::DataFlow,
    ];
    
    for cluster_type in types {
        match cluster_type {
            ClusterType::SecurityVulnerability => assert_eq!(cluster_type, ClusterType::SecurityVulnerability),
            ClusterType::AntiPattern => assert_eq!(cluster_type, ClusterType::AntiPattern),
            ClusterType::FunctionGroup => assert_eq!(cluster_type, ClusterType::FunctionGroup),
            ClusterType::DataFlow => assert_eq!(cluster_type, ClusterType::DataFlow),
        }
    }
}

#[test]
fn test_content_type_variants() {
    let types = vec![
        ContentType::SecurityPattern,
        ContentType::VulnerabilityDescription,
        ContentType::RemediationGuidance,
        ContentType::CodeExample,
        ContentType::BestPractice,
    ];
    
    for content_type in types {
        match content_type {
            ContentType::SecurityPattern => assert_eq!(content_type, ContentType::SecurityPattern),
            ContentType::VulnerabilityDescription => assert_eq!(content_type, ContentType::VulnerabilityDescription),
            ContentType::RemediationGuidance => assert_eq!(content_type, ContentType::RemediationGuidance),
            ContentType::CodeExample => assert_eq!(content_type, ContentType::CodeExample),
            ContentType::BestPractice => assert_eq!(content_type, ContentType::BestPractice),
        }
    }
}

#[test]
fn test_difficulty_level_ordering() {
    assert!(DifficultyLevel::Hard > DifficultyLevel::Medium);
    assert!(DifficultyLevel::Medium > DifficultyLevel::Easy);
}

#[tokio::test]
async fn test_knowledge_graph_query() {
    let graph = SecurityKnowledgeGraph::new();
    
    let query_result = graph.query_similar_patterns("sql injection").await;
    // Since the graph is empty, should return empty results
    assert!(query_result.is_ok());
    assert_eq!(query_result.unwrap().len(), 0);
}

#[tokio::test]
async fn test_rag_contextual_query() {
    let rag = CodeCentricRAG::new();
    
    let query = "How to prevent SQL injection?";
    let result = rag.contextual_query(query, 5).await;
    
    // Since the knowledge base is empty, should return empty results
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}

#[test]
fn test_security_correlation_creation() {
    let correlation = SecurityCorrelation {
        security_issue_id: "issue_123".to_string(),
        anti_pattern: "God Object".to_string(),
        correlation_strength: 0.8,
        amplification_factor: 1.5,
        explanation: "God Objects increase attack surface".to_string(),
    };
    
    assert_eq!(correlation.security_issue_id, "issue_123");
    assert_eq!(correlation.anti_pattern, "God Object");
    assert_eq!(correlation.correlation_strength, 0.8);
    assert_eq!(correlation.amplification_factor, 1.5);
}