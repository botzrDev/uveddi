use crate::analysis::graph::dependency::{ComponentNode, LocalDependencyGraph};
use std::collections::HashMap;
use tracing::debug;

use super::super::types::{CouplingMetrics, Dependency};

/// Core coupling matrix data structure
#[derive(Debug, Clone)]
pub struct CouplingMatrix {
    pub components: Vec<ComponentNode>,
    pub matrix: Vec<Vec<f64>>,
    pub labels: Vec<String>,
    pub metrics: HashMap<ComponentNode, CouplingMetrics>,
}

/// Builds coupling matrices from dependency data
#[derive(Debug, Clone)]
pub struct MatrixBuilder;

impl MatrixBuilder {
    pub fn new() -> Self {
        Self
    }

    /// Generate a coupling matrix from dependency graph
    pub fn build_matrix(
        &self,
        graph: &LocalDependencyGraph,
        metrics: &HashMap<ComponentNode, CouplingMetrics>,
        dependencies: &[Dependency],
    ) -> CouplingMatrix {
        debug!(
            "Generating coupling matrix for {} components",
            metrics.len()
        );

        let components: Vec<ComponentNode> = metrics.keys().cloned().collect();
        let labels: Vec<String> = components
            .iter()
            .map(|c| self.component_to_label(c))
            .collect();

        let matrix = self.build_coupling_matrix(&components, dependencies);

        CouplingMatrix {
            components,
            matrix,
            labels,
            metrics: metrics.clone(),
        }
    }

    /// Build the actual matrix values from dependencies
    fn build_coupling_matrix(
        &self,
        components: &[ComponentNode],
        dependencies: &[Dependency],
    ) -> Vec<Vec<f64>> {
        let size = components.len();
        let mut matrix = vec![vec![0.0; size]; size];

        // Create index mapping
        let component_to_index: HashMap<ComponentNode, usize> = components
            .iter()
            .enumerate()
            .map(|(i, c)| (c.clone(), i))
            .collect();

        // Fill matrix with dependency weights
        for dep in dependencies {
            if let (Some(&from_idx), Some(&to_idx)) = (
                component_to_index.get(&dep.from_component),
                component_to_index.get(&dep.to_component),
            ) {
                let weight = match dep.strength {
                    super::super::types::DependencyStrength::Weak => 1.0,
                    super::super::types::DependencyStrength::Medium => 2.0,
                    super::super::types::DependencyStrength::Strong => 3.0,
                };

                matrix[from_idx][to_idx] += weight;
            }
        }

        matrix
    }

    /// Convert component to display label
    pub fn component_to_label(&self, component: &ComponentNode) -> String {
        match component {
            ComponentNode::Class { name, file_path } => {
                let file_name = file_path.split('/').last().unwrap_or(file_path);
                format!("{}::{}", file_name, name)
            }
            ComponentNode::Function { name, file_path } => {
                let file_name = file_path.split('/').last().unwrap_or(file_path);
                format!("{}::{}", file_name, name)
            }
            ComponentNode::Module { path } => path.split('/').last().unwrap_or(path).to_string(),
        }
    }

    /// Extract module name from component
    pub fn extract_module_name(&self, component: &ComponentNode) -> String {
        match component {
            ComponentNode::Class { file_path, .. } => self.path_to_module_name(file_path),
            ComponentNode::Function { file_path, .. } => self.path_to_module_name(file_path),
            ComponentNode::Module { path } => self.path_to_module_name(path),
        }
    }

    fn path_to_module_name(&self, path: &str) -> String {
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() > 1 {
            parts[..parts.len() - 1].join("::")
        } else {
            "root".to_string()
        }
    }
}

impl Default for MatrixBuilder {
    fn default() -> Self {
        Self::new()
    }
}
