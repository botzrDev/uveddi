use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct DependencyGraph {
    pub nodes: Vec<String>,
    pub edges: Vec<(String, String)>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(&mut self, node: String) {
        if !self.nodes.contains(&node) {
            self.nodes.push(node);
        }
    }

    pub fn add_edge(&mut self, from: String, to: String) {
        self.add_node(from.clone());
        self.add_node(to.clone());
        self.edges.push((from, to));
    }
}
