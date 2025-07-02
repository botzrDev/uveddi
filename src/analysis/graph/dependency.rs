//! Represents the architectural dependency graph of a software project.

use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use std::collections::HashMap;
use std::path::PathBuf;
use uveddi_plugin_api::models::{Dependency, DependencyGraph, DependencyType};

/// Represents a node in the dependency graph.
/// This can be a module, a class, a function, or any other architectural component.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ComponentNode {
    Module { path: String },
    Class { name: String, file_path: String },
    Function { name: String, file_path: String },
}

/// Represents the type of dependency between two components.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LocalDependencyType {
    /// A direct function or method call.
    Call,
    /// An import or `use` statement.
    Import,
    /// A class inheritance relationship (`extends`).
    Inheritance,
    /// A class implementing an interface (`implements`).
    Implementation,
}

/// Represents a dependency relationship (an edge) in the graph.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DependencyEdge {
    pub dependency_type: LocalDependencyType,
    /// Could be used to store metadata like line numbers.
    pub weight: u32,
}

/// The main Architectural Dependency Graph (ADG).
///
/// This struct encapsulates a `petgraph::DiGraph` to model the relationships
/// between software components. It provides methods for adding components
/// and dependencies, and will be the basis for running architectural analyses
/// like cycle detection.
pub struct LocalDependencyGraph {
    graph: DiGraph<ComponentNode, DependencyEdge>,
    node_map: HashMap<ComponentNode, NodeIndex>,
}

impl LocalDependencyGraph {
    /// Creates a new, empty `LocalDependencyGraph`.
    pub fn new() -> Self {
        LocalDependencyGraph {
            graph: DiGraph::new(),
            node_map: HashMap::new(),
        }
    }

    /// Adds a component to the graph if it doesn't already exist.
    /// Returns the `NodeIndex` of the component.
    pub fn add_component(&mut self, node: ComponentNode) -> NodeIndex {
        if let Some(index) = self.node_map.get(&node) {
            *index
        } else {
            let index = self.graph.add_node(node.clone());
            self.node_map.insert(node, index);
            index
        }
    }

    /// Adds a dependency between two components.
    pub fn add_dependency(
        &mut self,
        from: &ComponentNode,
        to: &ComponentNode,
        dep_type: LocalDependencyType,
    ) {
        let from_index = self.add_component(from.clone());
        let to_index = self.add_component(to.clone());

        let edge = DependencyEdge {
            dependency_type: dep_type,
            weight: 1, // Default weight
        };

        self.graph.add_edge(from_index, to_index, edge);
    }

    /// Provides read-only access to the underlying petgraph DiGraph.
    pub fn get_petgraph(&self) -> &DiGraph<ComponentNode, DependencyEdge> {
        &self.graph
    }

    /// Retrieves a component node by its index.
    pub fn get_node_from_index(&self, index: NodeIndex) -> Option<&ComponentNode> {
        self.graph.node_weight(index)
    }
}

impl Default for LocalDependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert LocalDependencyGraph to plugin API DependencyGraph
impl From<&LocalDependencyGraph> for DependencyGraph {
    fn from(local_graph: &LocalDependencyGraph) -> Self {
        let mut plugin_graph = DependencyGraph::new();
        let mut dependencies = Vec::new();

        // Convert edges to dependencies
        for edge_ref in local_graph.graph.edge_references() {
            let from_node = local_graph.graph.node_weight(edge_ref.source()).unwrap();
            let to_node = local_graph.graph.node_weight(edge_ref.target()).unwrap();

            let from_file = match from_node {
                ComponentNode::Module { path } => PathBuf::from(path),
                ComponentNode::Class { file_path, .. } => PathBuf::from(file_path),
                ComponentNode::Function { file_path, .. } => PathBuf::from(file_path),
            };

            let to_module = match to_node {
                ComponentNode::Module { path } => path.clone(),
                ComponentNode::Class { name, .. } => name.clone(),
                ComponentNode::Function { name, .. } => name.clone(),
            };

            let dependency_type = match edge_ref.weight().dependency_type {
                LocalDependencyType::Call => DependencyType::Use,
                LocalDependencyType::Import => DependencyType::Import,
                LocalDependencyType::Inheritance => DependencyType::Use,
                LocalDependencyType::Implementation => DependencyType::Use,
            };

            dependencies.push(Dependency {
                from_file,
                to_module,
                dependency_type,
                line_number: None,
            });
        }

        plugin_graph.build_from_dependencies(dependencies);
        plugin_graph
    }
}
