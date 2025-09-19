use crate::analysis::graph::dependency::{ComponentNode, LocalDependencyGraph};
use std::collections::HashMap;
use tracing::debug;

use super::super::types::{CouplingMetrics, Dependency};

/// Generates coupling matrices for visualization and analysis
#[derive(Debug, Clone)]
pub struct CouplingMatrixGenerator;

#[derive(Debug, Clone)]
pub struct CouplingMatrix {
    pub components: Vec<ComponentNode>,
    pub matrix: Vec<Vec<f64>>,
    pub labels: Vec<String>,
    pub metrics: HashMap<ComponentNode, CouplingMetrics>,
}

#[derive(Debug, Clone)]
pub struct CouplingHeatmapData {
    pub rows: Vec<String>,
    pub columns: Vec<String>,
    pub values: Vec<Vec<f64>>,
    pub max_value: f64,
    pub min_value: f64,
}

#[derive(Debug, Clone)]
pub struct ModuleCouplingData {
    pub module_name: String,
    pub internal_coupling: f64,
    pub external_coupling: f64,
    pub coupling_ratio: f64, // external / internal
    pub components: Vec<ComponentNode>,
}

impl CouplingMatrixGenerator {
    pub fn new() -> Self {
        Self
    }

    /// Generate a coupling matrix from dependency graph
    pub fn generate_matrix(
        &self,
        graph: &LocalDependencyGraph,
        metrics: &HashMap<ComponentNode, CouplingMetrics>,
        dependencies: &[Dependency],
    ) -> CouplingMatrix {
        debug!("Generating coupling matrix for {} components", metrics.len());

        let components: Vec<ComponentNode> = metrics.keys().cloned().collect();
        let labels: Vec<String> = components.iter().map(|c| self.component_to_label(c)).collect();

        let matrix = self.build_coupling_matrix(&components, dependencies);

        CouplingMatrix {
            components,
            matrix,
            labels,
            metrics: metrics.clone(),
        }
    }

    /// Generate heatmap data for web visualization
    pub fn generate_heatmap_data(&self, matrix: &CouplingMatrix) -> CouplingHeatmapData {
        let rows = matrix.labels.clone();
        let columns = matrix.labels.clone();
        let values = matrix.matrix.clone();

        let all_values: Vec<f64> = values.iter().flatten().cloned().collect();
        let max_value = all_values.iter().fold(0.0f64, |a, &b| a.max(b));
        let min_value = all_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));

        CouplingHeatmapData {
            rows,
            columns,
            values,
            max_value,
            min_value: min_value.min(0.0),
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
            let module_name = self.extract_module_name(&dep.from_component);
            module_deps.entry(module_name).or_insert_with(Vec::new).push(dep);
        }

        let mut module_data = Vec::new();

        for (module_name, deps) in module_deps {
            let (internal_coupling, external_coupling) = self.calculate_module_coupling_metrics(&deps);
            let coupling_ratio = if internal_coupling > 0.0 {
                external_coupling / internal_coupling
            } else {
                external_coupling
            };

            let components: Vec<ComponentNode> = deps
                .iter()
                .map(|d| d.from_component.clone())
                .collect();

            module_data.push(ModuleCouplingData {
                module_name,
                internal_coupling,
                external_coupling,
                coupling_ratio,
                components,
            });
        }

        // Sort by coupling ratio (descending)
        module_data.sort_by(|a, b| b.coupling_ratio.partial_cmp(&a.coupling_ratio).unwrap_or(std::cmp::Ordering::Equal));

        module_data
    }

    /// Generate CSV format for the coupling matrix
    pub fn generate_csv(&self, matrix: &CouplingMatrix) -> String {
        let mut csv = String::new();

        // Header row
        csv.push_str(",");
        for label in &matrix.labels {
            csv.push_str(&format!("\"{}\",", label));
        }
        csv.pop(); // Remove trailing comma
        csv.push('\n');

        // Data rows
        for (i, row) in matrix.matrix.iter().enumerate() {
            csv.push_str(&format!("\"{}\",", matrix.labels[i]));
            for value in row {
                csv.push_str(&format!("{:.3},", value));
            }
            csv.pop(); // Remove trailing comma
            csv.push('\n');
        }

        csv
    }

    /// Generate JSON format for web visualization
    pub fn generate_json(&self, matrix: &CouplingMatrix) -> String {
        let heatmap_data = self.generate_heatmap_data(matrix);

        format!(
            r#"{{
  "rows": {:?},
  "columns": {:?},
  "values": {:?},
  "maxValue": {},
  "minValue": {}
}}"#,
            heatmap_data.rows,
            heatmap_data.columns,
            heatmap_data.values,
            heatmap_data.max_value,
            heatmap_data.min_value
        )
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

    fn component_to_label(&self, component: &ComponentNode) -> String {
        match component {
            ComponentNode::Class { name, file_path } => {
                let file_name = file_path.split('/').last().unwrap_or(file_path);
                format!("{}::{}", file_name, name)
            }
            ComponentNode::Function { name, file_path } => {
                let file_name = file_path.split('/').last().unwrap_or(file_path);
                format!("{}::{}", file_name, name)
            }
            ComponentNode::Module { path } => {
                path.split('/').last().unwrap_or(path).to_string()
            }
        }
    }

    fn extract_module_name(&self, component: &ComponentNode) -> String {
        match component {
            ComponentNode::Class { file_path, .. } => {
                self.path_to_module_name(file_path)
            }
            ComponentNode::Function { file_path, .. } => {
                self.path_to_module_name(file_path)
            }
            ComponentNode::Module { path } => {
                self.path_to_module_name(path)
            }
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

    fn calculate_module_coupling_metrics(&self, dependencies: &[&Dependency]) -> (f64, f64) {
        let mut internal_coupling = 0.0;
        let mut external_coupling = 0.0;

        for dep in dependencies {
            let from_module = self.extract_module_name(&dep.from_component);
            let to_module = self.extract_module_name(&dep.to_component);

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

impl Default for CouplingMatrixGenerator {
    fn default() -> Self {
        Self::new()
    }
}