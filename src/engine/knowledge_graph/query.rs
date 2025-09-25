//! # Knowledge Graph Query Engine
//!
//! Provides efficient querying capabilities for the knowledge graph.

use super::builder::{GraphNode, KnowledgeGraph};
use super::relations::{GraphRelation, RelationType};
use std::collections::{HashSet, VecDeque};

/// Query builder for knowledge graph searches
pub struct QueryBuilder {
    query: GraphQuery,
}

/// Graph query specification
#[derive(Default)]
pub struct GraphQuery {
    /// Start nodes for the query
    start_nodes: Vec<String>,

    /// Relation types to follow
    relation_types: Option<Vec<RelationType>>,

    /// Maximum depth to search
    max_depth: Option<usize>,

    /// Node filter predicate
    node_filter: Option<NodeFilter>,
}

/// Node filter function type
type NodeFilter = Box<dyn Fn(&GraphNode) -> bool + Send + Sync>;

impl std::fmt::Debug for GraphQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GraphQuery")
            .field("start_nodes", &self.start_nodes)
            .field("relation_types", &self.relation_types)
            .field("max_depth", &self.max_depth)
            .field("node_filter", &self.node_filter.as_ref().map(|_| "<closure>"))
            .finish()
    }
}

/// Query result
#[derive(Debug)]
pub struct QueryResult {
    pub nodes: Vec<GraphNode>,
    pub paths: Vec<GraphPath>,
}

/// Path through the graph
#[derive(Debug, Clone)]
pub struct GraphPath {
    pub nodes: Vec<String>,
    pub relations: Vec<GraphRelation>,
}

impl QueryBuilder {
    /// Create a new query builder
    pub fn new() -> Self {
        Self {
            query: GraphQuery::default(),
        }
    }

    /// Set starting nodes for the query
    pub fn from_nodes(mut self, node_ids: Vec<String>) -> Self {
        self.query.start_nodes = node_ids;
        self
    }

    /// Find nodes by symbol name
    pub fn from_symbol(mut self, symbol_name: String) -> Self {
        self.query.node_filter = Some(Box::new(move |node| node.symbol.name == symbol_name));
        self
    }

    /// Follow specific relation types
    pub fn follow_relations(mut self, relation_types: Vec<RelationType>) -> Self {
        self.query.relation_types = Some(relation_types);
        self
    }

    /// Set maximum search depth
    pub fn max_depth(mut self, depth: usize) -> Self {
        self.query.max_depth = Some(depth);
        self
    }

    /// Execute the query on a knowledge graph
    pub fn execute(self, graph: &KnowledgeGraph) -> QueryResult {
        let mut visited = HashSet::new();
        let mut result_nodes = Vec::new();
        let mut result_paths = Vec::new();

        // Start from specified nodes or find matching nodes
        let start_nodes = if self.query.start_nodes.is_empty() {
            // Find nodes matching filter
            if let Some(ref filter) = self.query.node_filter {
                graph
                    .nodes()
                    .values()
                    .filter(|node| filter(node))
                    .map(|node| node.id.clone())
                    .collect()
            } else {
                Vec::new()
            }
        } else {
            self.query.start_nodes
        };

        // Breadth-first search from start nodes
        for start_node in start_nodes {
            if let Some(node) = graph.nodes().get(&start_node) {
                let mut queue = VecDeque::new();
                queue.push_back((start_node.clone(), 0, GraphPath::new()));

                while let Some((current_id, depth, path)) = queue.pop_front() {
                    if visited.contains(&current_id) {
                        continue;
                    }
                    visited.insert(current_id.clone());

                    // Check depth limit
                    if let Some(max_depth) = self.query.max_depth {
                        if depth > max_depth {
                            continue;
                        }
                    }

                    // Add current node to results
                    if let Some(current_node) = graph.nodes().get(&current_id) {
                        result_nodes.push(current_node.clone());
                        if !path.is_empty() {
                            result_paths.push(path.clone());
                        }
                    }

                    // Follow edges
                    if let Some(edges) = graph.edges_from(&current_id) {
                        for edge in edges {
                            // Filter by relation type if specified
                            if let Some(ref allowed_types) = self.query.relation_types {
                                if !allowed_types.contains(&edge.relation_type) {
                                    continue;
                                }
                            }

                            let mut new_path = path.clone();
                            new_path.add_step(current_id.clone(), edge.clone());
                            queue.push_back((edge.to.clone(), depth + 1, new_path));
                        }
                    }
                }
            }
        }

        QueryResult {
            nodes: result_nodes,
            paths: result_paths,
        }
    }
}

impl GraphPath {
    /// Create a new empty path
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            relations: Vec::new(),
        }
    }

    /// Add a step to the path
    pub fn add_step(&mut self, node_id: String, relation: GraphRelation) {
        self.nodes.push(node_id);
        self.relations.push(relation);
    }

    /// Check if the path is empty
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Get path length
    pub fn length(&self) -> usize {
        self.relations.len()
    }
}
