use crate::analysis::graph::dependency::{ComponentNode, LocalDependencyGraph};
use std::collections::HashMap;
use tracing::debug;

/// Calculator for Efferent Coupling (Ce) - outgoing dependencies
#[derive(Debug, Clone)]
pub struct EfferentCouplingCalculator;

impl EfferentCouplingCalculator {
    pub fn new() -> Self {
        Self
    }

    /// Calculate efferent coupling for a single component
    /// Ce = number of classes this class depends on
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
                    let outgoing_count = petgraph.edges(node_index).count();

                    debug!(
                        "Component {:?} has {} efferent couplings",
                        component, outgoing_count
                    );

                    return outgoing_count;
                }
            }
        }

        debug!("Component {:?} not found in graph", component);
        0
    }

    /// Calculate efferent coupling for all components in the graph
    pub fn calculate_for_all_components(
        &self,
        graph: &LocalDependencyGraph,
    ) -> HashMap<ComponentNode, usize> {
        let mut efferent_couplings = HashMap::new();
        let petgraph = graph.get_petgraph();

        debug!("Calculating efferent coupling for {} nodes", petgraph.node_count());

        for node_index in petgraph.node_indices() {
            if let Some(component) = graph.get_node_from_index(node_index) {
                let outgoing_count = petgraph.edges(node_index).count();
                efferent_couplings.insert(component.clone(), outgoing_count);
            }
        }

        debug!("Calculated efferent coupling for {} components", efferent_couplings.len());
        efferent_couplings
    }

    /// Identify components with high efferent coupling (many dependencies)
    pub fn identify_high_efferent_coupling(
        &self,
        graph: &LocalDependencyGraph,
        threshold: usize,
    ) -> Vec<(ComponentNode, usize)> {
        let efferent_couplings = self.calculate_for_all_components(graph);

        let mut high_coupling: Vec<_> = efferent_couplings
            .into_iter()
            .filter(|(_, count)| *count >= threshold)
            .collect();

        // Sort by coupling count descending
        high_coupling.sort_by(|a, b| b.1.cmp(&a.1));

        debug!(
            "Found {} components with efferent coupling >= {}",
            high_coupling.len(),
            threshold
        );

        high_coupling
    }

    /// Calculate efferent coupling by component type
    pub fn calculate_by_component_type(
        &self,
        graph: &LocalDependencyGraph,
    ) -> HashMap<String, Vec<(ComponentNode, usize)>> {
        let efferent_couplings = self.calculate_for_all_components(graph);
        let mut by_type: HashMap<String, Vec<(ComponentNode, usize)>> = HashMap::new();

        for (component, count) in efferent_couplings {
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

    /// Calculate statistics for efferent coupling distribution
    pub fn calculate_statistics(
        &self,
        graph: &LocalDependencyGraph,
    ) -> EfferentCouplingStatistics {
        let efferent_couplings = self.calculate_for_all_components(graph);
        let values: Vec<usize> = efferent_couplings.values().cloned().collect();

        if values.is_empty() {
            return EfferentCouplingStatistics::default();
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

        EfferentCouplingStatistics {
            total_components: count,
            mean,
            median,
            std_dev,
            min: *sorted_values.first().unwrap_or(&0),
            max: *sorted_values.last().unwrap_or(&0),
            total_efferent_couplings: total,
        }
    }

    /// Identify components with excessive dependencies (high efferent coupling)
    pub fn identify_dependency_heavy_components(
        &self,
        graph: &LocalDependencyGraph,
        percentile: f64, // e.g., 0.9 for top 10%
    ) -> Vec<(ComponentNode, usize)> {
        let efferent_couplings = self.calculate_for_all_components(graph);
        let mut sorted_components: Vec<_> = efferent_couplings.into_iter().collect();

        sorted_components.sort_by(|a, b| b.1.cmp(&a.1));

        let cutoff_index = ((sorted_components.len() as f64) * (1.0 - percentile)).max(1.0) as usize;

        sorted_components.into_iter().take(cutoff_index).collect()
    }

    /// Analyze dependency patterns to identify potential refactoring opportunities
    pub fn analyze_dependency_patterns(
        &self,
        graph: &LocalDependencyGraph,
    ) -> DependencyPatternAnalysis {
        let efferent_couplings = self.calculate_for_all_components(graph);
        let statistics = self.calculate_statistics(graph);

        let high_threshold = (statistics.mean + statistics.std_dev) as usize;
        let very_high_threshold = (statistics.mean + 2.0 * statistics.std_dev) as usize;

        let high_efferent = efferent_couplings
            .iter()
            .filter(|(_, &count)| count >= high_threshold && count < very_high_threshold)
            .count();

        let very_high_efferent = efferent_couplings
            .iter()
            .filter(|(_, &count)| count >= very_high_threshold)
            .count();

        let isolated_components = efferent_couplings
            .iter()
            .filter(|(_, &count)| count == 0)
            .count();

        DependencyPatternAnalysis {
            total_components: efferent_couplings.len(),
            isolated_components,
            high_efferent_coupling: high_efferent,
            very_high_efferent_coupling: very_high_efferent,
            statistics,
        }
    }
}

/// Statistics for efferent coupling distribution
#[derive(Debug, Clone, Default)]
pub struct EfferentCouplingStatistics {
    pub total_components: usize,
    pub mean: f64,
    pub median: f64,
    pub std_dev: f64,
    pub min: usize,
    pub max: usize,
    pub total_efferent_couplings: usize,
}

/// Analysis of dependency patterns in the system
#[derive(Debug, Clone)]
pub struct DependencyPatternAnalysis {
    pub total_components: usize,
    pub isolated_components: usize,
    pub high_efferent_coupling: usize,
    pub very_high_efferent_coupling: usize,
    pub statistics: EfferentCouplingStatistics,
}

impl Default for EfferentCouplingCalculator {
    fn default() -> Self {
        Self::new()
    }
}