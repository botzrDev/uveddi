use crate::analysis::graph::dependency::{ComponentNode, LocalDependencyGraph};
use std::collections::HashMap;
use tracing::info;

use super::{
    analysis::CouplingCalculator,
    metrics::InstabilityCalculator,
    types::{CouplingMetrics, Dependency},
    visualization::{CouplingMatrixGenerator, DependencyGraphVisualizer},
};

/// Comprehensive coupling analysis report
#[derive(Debug)]
pub struct CouplingAnalysisReport {
    pub metrics: HashMap<ComponentNode, CouplingMetrics>,
    pub stability_analysis: super::metrics::instability_calculator::StabilityAnalysis,
    pub hotspots: Vec<(ComponentNode, CouplingMetrics)>,
    pub visualization_data: Option<super::visualization::dependency_graph::DependencyGraphData>,
    pub coupling_matrix: Option<super::visualization::CouplingMatrix>,
}

/// Report generator for coupling analysis
#[derive(Debug)]
pub struct ReportGenerator {
    coupling_calculator: CouplingCalculator,
    instability_calculator: InstabilityCalculator,
    graph_visualizer: DependencyGraphVisualizer,
    matrix_generator: CouplingMatrixGenerator,
}

impl ReportGenerator {
    pub fn new() -> Self {
        Self {
            coupling_calculator: CouplingCalculator::new(),
            instability_calculator: InstabilityCalculator::new(),
            graph_visualizer: DependencyGraphVisualizer::new(),
            matrix_generator: CouplingMatrixGenerator::new(),
        }
    }

    /// Generate comprehensive coupling analysis report
    pub fn generate_report(
        &self,
        graph: &LocalDependencyGraph,
        dependencies: &[Dependency],
        config: &super::config::TightCouplingConfig,
    ) -> CouplingAnalysisReport {
        info!("Generating comprehensive coupling analysis report");

        let metrics = self.coupling_calculator.calculate_metrics(graph);
        let stability_analysis = self.instability_calculator.analyze_system_stability(graph);
        let hotspots = self
            .coupling_calculator
            .identify_coupling_hotspots(&metrics, 0.9);

        let visualization_data = if config.visualization_enabled {
            Some(
                self.graph_visualizer
                    .generate_graph_data(graph, &metrics, dependencies),
            )
        } else {
            None
        };

        let coupling_matrix = if config.generate_coupling_matrix {
            Some(
                self.matrix_generator
                    .generate_matrix(graph, &metrics, dependencies),
            )
        } else {
            None
        };

        CouplingAnalysisReport {
            metrics,
            stability_analysis,
            hotspots,
            visualization_data,
            coupling_matrix,
        }
    }
}

impl Default for ReportGenerator {
    fn default() -> Self {
        Self::new()
    }
}
