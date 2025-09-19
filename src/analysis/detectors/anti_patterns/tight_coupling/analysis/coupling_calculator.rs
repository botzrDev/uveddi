use crate::analysis::graph::dependency::{ComponentNode, LocalDependencyGraph};
use std::collections::HashMap;
use tracing::debug;

use super::super::types::CouplingMetrics;

/// Calculates various coupling metrics for components
#[derive(Debug, Clone)]
pub struct CouplingCalculator;

impl CouplingCalculator {
    pub fn new() -> Self {
        Self
    }

    /// Calculate coupling metrics for all components in the dependency graph
    pub fn calculate_metrics(
        &self,
        graph: &LocalDependencyGraph,
    ) -> HashMap<ComponentNode, CouplingMetrics> {
        debug!("Calculating coupling metrics for {} nodes", graph.get_petgraph().node_count());

        let mut metrics = HashMap::new();
        let petgraph = graph.get_petgraph();

        for node_index in petgraph.node_indices() {
            if let Some(component) = graph.get_node_from_index(node_index) {
                let component_metrics = self.calculate_component_metrics(graph, component, node_index);
                metrics.insert(component.clone(), component_metrics);
            }
        }

        debug!("Calculated metrics for {} components", metrics.len());
        metrics
    }

    /// Calculate metrics for a single component
    fn calculate_component_metrics(
        &self,
        graph: &LocalDependencyGraph,
        component: &ComponentNode,
        node_index: petgraph::graph::NodeIndex,
    ) -> CouplingMetrics {
        let petgraph = graph.get_petgraph();

        let mut metrics = CouplingMetrics::default();

        // Calculate fan-out (outgoing dependencies - efferent coupling)
        metrics.fan_out = petgraph.edges(node_index).count();

        // Calculate fan-in (incoming dependencies - afferent coupling)
        metrics.fan_in = petgraph
            .edges_directed(node_index, petgraph::Direction::Incoming)
            .count();

        // CBO = fan-in + fan-out (Coupling Between Objects)
        metrics.cbo = metrics.fan_in + metrics.fan_out;

        // RFC approximation (Response for Class - would need method-level analysis for accuracy)
        metrics.rfc = metrics.fan_out + self.estimate_local_methods(component);

        // LCOM approximation (Lack of Cohesion in Methods - simplified calculation)
        metrics.lcom = self.calculate_lcom_approximation(component, &metrics);

        metrics
    }

    /// Calculate Afferent Coupling (Ca) - number of classes that depend on this class
    pub fn calculate_afferent_coupling(
        &self,
        graph: &LocalDependencyGraph,
        component: &ComponentNode,
    ) -> usize {
        let petgraph = graph.get_petgraph();

        // Find the node index by iterating through all nodes
        for node_index in petgraph.node_indices() {
            if let Some(node_component) = graph.get_node_from_index(node_index) {
                if node_component == component {
                    return petgraph
                        .edges_directed(node_index, petgraph::Direction::Incoming)
                        .count();
                }
            }
        }
        0
    }

    /// Calculate Efferent Coupling (Ce) - number of classes this class depends on
    pub fn calculate_efferent_coupling(
        &self,
        graph: &LocalDependencyGraph,
        component: &ComponentNode,
    ) -> usize {
        let petgraph = graph.get_petgraph();

        // Find the node index by iterating through all nodes
        for node_index in petgraph.node_indices() {
            if let Some(node_component) = graph.get_node_from_index(node_index) {
                if node_component == component {
                    return petgraph.edges(node_index).count();
                }
            }
        }
        0
    }

    /// Calculate Instability Index (I = Ce / (Ca + Ce))
    /// Values range from 0 (maximally stable) to 1 (maximally unstable)
    pub fn calculate_instability(
        &self,
        graph: &LocalDependencyGraph,
        component: &ComponentNode,
    ) -> f64 {
        let ca = self.calculate_afferent_coupling(graph, component) as f64;
        let ce = self.calculate_efferent_coupling(graph, component) as f64;

        if ca + ce == 0.0 {
            0.0 // Isolated component
        } else {
            ce / (ca + ce)
        }
    }

    /// Calculate Abstractness (A) - approximation based on component type
    /// Values range from 0 (concrete) to 1 (abstract)
    pub fn calculate_abstractness(&self, component: &ComponentNode) -> f64 {
        match component {
            ComponentNode::Class { name, .. } => {
                // Heuristics based on naming conventions
                if name.contains("Abstract") || name.contains("Base") || name.contains("Interface") {
                    0.8
                } else if name.contains("Trait") || name.contains("Protocol") {
                    0.9
                } else {
                    0.2 // Most classes are concrete
                }
            }
            ComponentNode::Function { .. } => 0.1, // Functions are typically concrete
            ComponentNode::Module { .. } => 0.5,   // Modules are moderately abstract
        }
    }

    /// Calculate Distance from Main Sequence (D = |A + I - 1|)
    /// Values range from 0 (on main sequence) to 1 (furthest from main sequence)
    pub fn calculate_distance_from_main_sequence(
        &self,
        graph: &LocalDependencyGraph,
        component: &ComponentNode,
    ) -> f64 {
        let abstractness = self.calculate_abstractness(component);
        let instability = self.calculate_instability(graph, component);

        (abstractness + instability - 1.0).abs()
    }

    /// Estimate local methods for RFC calculation
    fn estimate_local_methods(&self, component: &ComponentNode) -> usize {
        match component {
            ComponentNode::Class { name, .. } => {
                // Estimate based on typical class sizes
                if name.contains("Manager") || name.contains("Controller") || name.contains("Service") {
                    8 // Larger service classes
                } else {
                    5 // Average methods per class
                }
            }
            ComponentNode::Module { .. } => 3,   // Average functions per module
            ComponentNode::Function { .. } => 1, // Single function
        }
    }

    /// Calculate simplified LCOM (Lack of Cohesion in Methods)
    /// This is a very basic approximation - real LCOM requires method-level analysis
    fn calculate_lcom_approximation(
        &self,
        component: &ComponentNode,
        metrics: &CouplingMetrics,
    ) -> f64 {
        match component {
            ComponentNode::Class { .. } => {
                // High coupling often correlates with low cohesion
                // This is a rough heuristic
                if metrics.cbo > 10 {
                    0.8 // High LCOM (low cohesion) when many dependencies
                } else if metrics.cbo > 5 {
                    0.5 // Medium LCOM
                } else {
                    0.2 // Low LCOM (high cohesion)
                }
            }
            ComponentNode::Function { .. } => 0.1, // Functions are inherently cohesive
            ComponentNode::Module { .. } => {
                // Module cohesion depends on internal organization
                if metrics.cbo > 15 {
                    0.7
                } else {
                    0.3
                }
            }
        }
    }

    /// Calculate weighted coupling considering dependency strength
    pub fn calculate_weighted_coupling(
        &self,
        dependencies: &[super::super::types::Dependency],
    ) -> f64 {
        let mut total_weight = 0.0;

        for dep in dependencies {
            let weight = match dep.strength {
                super::super::types::DependencyStrength::Weak => 1.0,
                super::super::types::DependencyStrength::Medium => 2.0,
                super::super::types::DependencyStrength::Strong => 3.0,
            };
            total_weight += weight;
        }

        total_weight
    }

    /// Identify coupling hotspots - components with highest coupling metrics
    pub fn identify_coupling_hotspots(
        &self,
        metrics: &HashMap<ComponentNode, CouplingMetrics>,
        threshold_percentile: f64, // e.g., 0.9 for top 10%
    ) -> Vec<(ComponentNode, CouplingMetrics)> {
        let mut sorted_metrics: Vec<_> = metrics.iter().collect();

        // Sort by CBO (primary) and RFC (secondary)
        sorted_metrics.sort_by(|a, b| {
            b.1.cbo.cmp(&a.1.cbo)
                .then(b.1.rfc.cmp(&a.1.rfc))
        });

        let hotspot_count = ((sorted_metrics.len() as f64) * (1.0 - threshold_percentile)).max(1.0) as usize;

        sorted_metrics
            .into_iter()
            .take(hotspot_count)
            .map(|(comp, metrics)| (comp.clone(), metrics.clone()))
            .collect()
    }
}

impl Default for CouplingCalculator {
    fn default() -> Self {
        Self::new()
    }
}