pub mod coupling_analyzer;
pub mod coupling_matrix;
pub mod dependency_graph;
pub mod graph_analyzer;
pub mod graph_builder;
pub mod heatmap_generator;
pub mod matrix_builder;

pub use coupling_analyzer::{CouplingAnalyzer, ModuleCouplingData};
pub use coupling_matrix::CouplingMatrixGenerator;
pub use dependency_graph::DependencyGraphVisualizer;
pub use graph_analyzer::{ConnectivityAnalysis, GraphAnalyzer, GraphStatistics};
pub use graph_builder::{GraphBuilder, GraphEdge, GraphNode};
pub use heatmap_generator::{CouplingHeatmapData, HeatmapGenerator};
pub use matrix_builder::{CouplingMatrix, MatrixBuilder};
