pub mod dependency_graph;
pub mod graph_builder;
pub mod graph_analyzer;
pub mod coupling_matrix;
pub mod matrix_builder;
pub mod heatmap_generator;
pub mod coupling_analyzer;

pub use dependency_graph::DependencyGraphVisualizer;
pub use graph_builder::{GraphBuilder, GraphNode, GraphEdge};
pub use graph_analyzer::{GraphAnalyzer, GraphStatistics, ConnectivityAnalysis};
pub use coupling_matrix::CouplingMatrixGenerator;
pub use matrix_builder::{MatrixBuilder, CouplingMatrix};
pub use heatmap_generator::{HeatmapGenerator, CouplingHeatmapData};
pub use coupling_analyzer::{CouplingAnalyzer, ModuleCouplingData};