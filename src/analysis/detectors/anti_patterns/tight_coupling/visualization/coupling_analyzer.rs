use crate::analysis::graph::dependency::{ComponentNode, LocalDependencyGraph};
use std::collections::HashMap;
use tracing::debug;

use super::super::types::Dependency;
use super::matrix_builder::{CouplingMatrix, MatrixBuilder};

/// Module-level coupling analysis data
#[derive(Debug, Clone)]
pub struct ModuleCouplingData {
    pub module_name: String,
    pub internal_coupling: f64,
    pub external_coupling: f64,
    pub coupling_ratio: f64, // external / internal
    pub components: Vec<ComponentNode>,
}

/// Analyzes coupling patterns and relationships
#[derive(Debug, Clone)]
pub struct CouplingAnalyzer {
    matrix_builder: MatrixBuilder,
}

impl CouplingAnalyzer {
    pub fn new() -> Self {
        Self {
            matrix_builder: MatrixBuilder::new(),
        }
    }

    /// Generate module-level coupling analysis
    pub fn analyze_module_coupling(
        &self,
        dependencies: &[Dependency],
        graph: &LocalDependencyGraph,
    ) -> Vec<ModuleCouplingData> {
        debug!("Analyzing module-level coupling");

        let mut module_deps: HashMap<String, Vec<&Dependency>> = HashMap::new();

        // Group dependencies by module
        for dep in dependencies {
            let module_name = self.matrix_builder.extract_module_name(&dep.from_component);
            module_deps
                .entry(module_name)
                .or_insert_with(Vec::new)
                .push(dep);
        }

        let mut module_data = Vec::new();

        for (module_name, deps) in module_deps {
            let (internal_coupling, external_coupling) =
                self.calculate_module_coupling_metrics(&deps);
            let coupling_ratio = if internal_coupling > 0.0 {
                external_coupling / internal_coupling
            } else {
                external_coupling
            };

            let components: Vec<ComponentNode> =
                deps.iter().map(|d| d.from_component.clone()).collect();

            module_data.push(ModuleCouplingData {
                module_name,
                internal_coupling,
                external_coupling,
                coupling_ratio,
                components,
            });
        }

        // Sort by coupling ratio (descending)
        module_data.sort_by(|a, b| {
            b.coupling_ratio
                .partial_cmp(&a.coupling_ratio)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        module_data
    }

    /// Identify strongly coupled component pairs
    pub fn identify_strong_coupling_pairs(
        &self,
        matrix: &CouplingMatrix,
        threshold: f64,
    ) -> Vec<(String, String, f64)> {
        let mut strong_pairs = Vec::new();

        for i in 0..matrix.matrix.len() {
            for j in (i + 1)..matrix.matrix[i].len() {
                let coupling_strength = matrix.matrix[i][j] + matrix.matrix[j][i];
                if coupling_strength >= threshold {
                    strong_pairs.push((
                        matrix.labels[i].clone(),
                        matrix.labels[j].clone(),
                        coupling_strength,
                    ));
                }
            }
        }

        // Sort by coupling strength (descending)
        strong_pairs.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

        strong_pairs
    }

    /// Calculate coupling density for the entire system
    pub fn calculate_coupling_density(&self, matrix: &CouplingMatrix) -> f64 {
        let total_possible_connections = matrix.components.len() * (matrix.components.len() - 1);
        if total_possible_connections == 0 {
            return 0.0;
        }

        let actual_connections: f64 = matrix
            .matrix
            .iter()
            .flatten()
            .filter(|&&value| value > 0.0)
            .count() as f64;

        actual_connections / total_possible_connections as f64
    }

    fn calculate_module_coupling_metrics(&self, dependencies: &[&Dependency]) -> (f64, f64) {
        let mut internal_coupling = 0.0;
        let mut external_coupling = 0.0;

        for dep in dependencies {
            let from_module = self.matrix_builder.extract_module_name(&dep.from_component);
            let to_module = self.matrix_builder.extract_module_name(&dep.to_component);

            let weight = match dep.strength {
                super::super::types::DependencyStrength::Weak => 1.0,
                super::super::types::DependencyStrength::Medium => 2.0,
                super::super::types::DependencyStrength::Strong => 3.0,
            };

            if from_module == to_module {
                internal_coupling += weight;
            } else {
                external_coupling += weight;
            }
        }

        (internal_coupling, external_coupling)
    }
}

impl Default for CouplingAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
