use crate::analysis::graph::dependency::{ComponentNode, LocalDependencyGraph};
use std::collections::HashMap;

use super::super::types::{CouplingMetrics, Dependency};
use super::{
    matrix_builder::{MatrixBuilder, CouplingMatrix},
    heatmap_generator::{HeatmapGenerator, CouplingHeatmapData},
    coupling_analyzer::{CouplingAnalyzer, ModuleCouplingData},
};

/// Coordinates coupling matrix generation and analysis
#[derive(Debug, Clone)]
pub struct CouplingMatrixGenerator {
    matrix_builder: MatrixBuilder,
    heatmap_generator: HeatmapGenerator,
    coupling_analyzer: CouplingAnalyzer,
}

impl CouplingMatrixGenerator {
    pub fn new() -> Self {
        Self {
            matrix_builder: MatrixBuilder::new(),
            heatmap_generator: HeatmapGenerator::new(),
            coupling_analyzer: CouplingAnalyzer::new(),
        }
    }

    /// Generate a coupling matrix from dependency graph
    pub fn generate_matrix(
        &self,
        graph: &LocalDependencyGraph,
        metrics: &HashMap<ComponentNode, CouplingMetrics>,
        dependencies: &[Dependency],
    ) -> CouplingMatrix {
        self.matrix_builder.build_matrix(graph, metrics, dependencies)
    }

    /// Generate heatmap data for web visualization
    pub fn generate_heatmap_data(&self, matrix: &CouplingMatrix) -> CouplingHeatmapData {
        self.heatmap_generator.generate_heatmap_data(matrix)
    }

    /// Generate module-level coupling analysis
    pub fn analyze_module_coupling(
        &self,
        dependencies: &[Dependency],
        graph: &LocalDependencyGraph,
    ) -> Vec<ModuleCouplingData> {
        self.coupling_analyzer.analyze_module_coupling(dependencies, graph)
    }

    /// Generate CSV format for the coupling matrix
    pub fn generate_csv(&self, matrix: &CouplingMatrix) -> String {
        self.heatmap_generator.generate_csv(matrix)
    }

    /// Generate JSON format for web visualization
    pub fn generate_json(&self, matrix: &CouplingMatrix) -> String {
        self.heatmap_generator.generate_json(matrix)
    }

    /// Identify strongly coupled component pairs
    pub fn identify_strong_coupling_pairs(
        &self,
        matrix: &CouplingMatrix,
        threshold: f64,
    ) -> Vec<(String, String, f64)> {
        self.coupling_analyzer.identify_strong_coupling_pairs(matrix, threshold)
    }

    /// Calculate coupling density for the entire system
    pub fn calculate_coupling_density(&self, matrix: &CouplingMatrix) -> f64 {
        self.coupling_analyzer.calculate_coupling_density(matrix)
    }
}

impl Default for CouplingMatrixGenerator {
    fn default() -> Self {
        Self::new()
    }
}