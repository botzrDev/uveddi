use crate::analysis::graph::dependency::{ComponentNode, LocalDependencyGraph};
use std::collections::HashMap;
use tracing::debug;

/// Calculator for Afferent Coupling (Ca) - incoming dependencies
#[derive(Debug, Clone)]
pub struct AfferentCouplingCalculator;

impl AfferentCouplingCalculator {
    pub fn new() -> Self {
        Self
    }

    /// Calculate afferent coupling for a single component
    /// Ca = number of classes that depend on this class
    pub fn calculate_for_component(
        &self,
        graph: &LocalDependencyGraph,
        component: &ComponentNode,
    ) -> usize {
        let petgraph = graph.get_petgraph();

        // Find the node index by iterating through all nodes
        for node_index in petgraph.node_indices() {
            if let Some(node_component) = graph.get_node_from_index(node_index) {
                if node_component == component {
                    let incoming_count = petgraph
                        .edges_directed(node_index, petgraph::Direction::Incoming)
                        .count();

                    debug!(
                        "Component {:?} has {} afferent couplings",
                        component, incoming_count
                    );

                    return incoming_count;
                }
            }
        }

        debug!("Component {:?} not found in graph", component);
        0
    }

    /// Calculate afferent coupling for all components in the graph
    pub fn calculate_for_all_components(
        &self,
        graph: &LocalDependencyGraph,
    ) -> HashMap<ComponentNode, usize> {
        let mut afferent_couplings = HashMap::new();
        let petgraph = graph.get_petgraph();

        debug!("Calculating afferent coupling for {} nodes", petgraph.node_count());

        for node_index in petgraph.node_indices() {
            if let Some(component) = graph.get_node_from_index(node_index) {
                let incoming_count = petgraph
                    .edges_directed(node_index, petgraph::Direction::Incoming)
                    .count();

                afferent_couplings.insert(component.clone(), incoming_count);
            }
        }

        debug!("Calculated afferent coupling for {} components", afferent_couplings.len());
        afferent_couplings
    }

    /// Identify components with high afferent coupling (many dependents)
    pub fn identify_high_afferent_coupling(
        &self,
        graph: &LocalDependencyGraph,
        threshold: usize,
    ) -> Vec<(ComponentNode, usize)> {
        let afferent_couplings = self.calculate_for_all_components(graph);

        let mut high_coupling: Vec<_> = afferent_couplings
            .into_iter()
            .filter(|(_, count)| *count >= threshold)
            .collect();

        // Sort by coupling count descending
        high_coupling.sort_by(|a, b| b.1.cmp(&a.1));

        debug!(
            "Found {} components with afferent coupling >= {}",
            high_coupling.len(),
            threshold
        );

        high_coupling
    }

    /// Calculate afferent coupling by component type
    pub fn calculate_by_component_type(
        &self,
        graph: &LocalDependencyGraph,
    ) -> HashMap<String, Vec<(ComponentNode, usize)>> {
        let afferent_couplings = self.calculate_for_all_components(graph);
        let mut by_type: HashMap<String, Vec<(ComponentNode, usize)>> = HashMap::new();

        for (component, count) in afferent_couplings {
            let component_type = match &component {
                ComponentNode::Class { .. } => "Class".to_string(),
                ComponentNode::Function { .. } => "Function".to_string(),
                ComponentNode::Module { .. } => "Module".to_string(),
            };

            by_type
                .entry(component_type)
                .or_insert_with(Vec::new)
                .push((component, count));
        }

        // Sort each type by coupling count
        for (_, components) in by_type.iter_mut() {
            components.sort_by(|a, b| b.1.cmp(&a.1));
        }

        by_type
    }

    /// Calculate statistics for afferent coupling distribution
    pub fn calculate_statistics(
        &self,
        graph: &LocalDependencyGraph,
    ) -> AfferentCouplingStatistics {
        let afferent_couplings = self.calculate_for_all_components(graph);
        let values: Vec<usize> = afferent_couplings.values().cloned().collect();

        if values.is_empty() {
            return AfferentCouplingStatistics::default();
        }

        let total: usize = values.iter().sum();
        let count = values.len();
        let mean = total as f64 / count as f64;

        let mut sorted_values = values.clone();
        sorted_values.sort_unstable();

        let median = if count % 2 == 0 {
            (sorted_values[count / 2 - 1] + sorted_values[count / 2]) as f64 / 2.0
        } else {
            sorted_values[count / 2] as f64
        };

        let variance = values
            .iter()
            .map(|&x| {
                let diff = x as f64 - mean;
                diff * diff
            })
            .sum::<f64>() / count as f64;

        let std_dev = variance.sqrt();

        AfferentCouplingStatistics {
            total_components: count,
            mean,
            median,
            std_dev,
            min: *sorted_values.first().unwrap_or(&0),
            max: *sorted_values.last().unwrap_or(&0),
            total_afferent_couplings: total,
        }
    }

    /// Identify components that are central to the system (high afferent coupling)
    pub fn identify_central_components(
        &self,
        graph: &LocalDependencyGraph,
        percentile: f64, // e.g., 0.9 for top 10%
    ) -> Vec<(ComponentNode, usize)> {
        let afferent_couplings = self.calculate_for_all_components(graph);
        let mut sorted_components: Vec<_> = afferent_couplings.into_iter().collect();

        sorted_components.sort_by(|a, b| b.1.cmp(&a.1));

        let cutoff_index = ((sorted_components.len() as f64) * (1.0 - percentile)).max(1.0) as usize;

        sorted_components.into_iter().take(cutoff_index).collect()
    }
}

/// Statistics for afferent coupling distribution
#[derive(Debug, Clone, Default)]
pub struct AfferentCouplingStatistics {
    pub total_components: usize,
    pub mean: f64,
    pub median: f64,
    pub std_dev: f64,
    pub min: usize,
    pub max: usize,
    pub total_afferent_couplings: usize,
}

impl Default for AfferentCouplingCalculator {
    fn default() -> Self {
        Self::new()
    }
}