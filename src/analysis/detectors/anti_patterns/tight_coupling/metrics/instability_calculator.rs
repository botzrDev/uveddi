use crate::analysis::graph::dependency::{ComponentNode, LocalDependencyGraph};
use std::collections::HashMap;
use tracing::debug;

use super::{AfferentCouplingCalculator, EfferentCouplingCalculator};

/// Calculator for Instability Index and related stability metrics
#[derive(Debug, Clone)]
pub struct InstabilityCalculator {
    afferent_calculator: AfferentCouplingCalculator,
    efferent_calculator: EfferentCouplingCalculator,
}

#[derive(Debug, Clone)]
pub struct StabilityMetrics {
    pub afferent_coupling: usize,
    pub efferent_coupling: usize,
    pub instability: f64,
    pub abstractness: f64,
    pub distance_from_main_sequence: f64,
}

#[derive(Debug, Clone)]
pub struct StabilityAnalysis {
    pub stable_components: Vec<(ComponentNode, StabilityMetrics)>,
    pub unstable_components: Vec<(ComponentNode, StabilityMetrics)>,
    pub balanced_components: Vec<(ComponentNode, StabilityMetrics)>,
    pub zone_of_pain: Vec<(ComponentNode, StabilityMetrics)>,     // Stable + Concrete
    pub zone_of_uselessness: Vec<(ComponentNode, StabilityMetrics)>, // Unstable + Abstract
}

impl InstabilityCalculator {
    pub fn new() -> Self {
        Self {
            afferent_calculator: AfferentCouplingCalculator::new(),
            efferent_calculator: EfferentCouplingCalculator::new(),
        }
    }

    /// Calculate Instability Index for a single component
    /// I = Ce / (Ca + Ce)
    /// Values range from 0 (maximally stable) to 1 (maximally unstable)
    pub fn calculate_instability(
        &self,
        graph: &LocalDependencyGraph,
        component: &ComponentNode,
    ) -> f64 {
        let ca = self.afferent_calculator.calculate_for_component(graph, component) as f64;
        let ce = self.efferent_calculator.calculate_for_component(graph, component) as f64;

        if ca + ce == 0.0 {
            0.0 // Isolated component is considered stable
        } else {
            let instability = ce / (ca + ce);
            debug!(
                "Component {:?}: Ca={}, Ce={}, Instability={:.3}",
                component, ca, ce, instability
            );
            instability
        }
    }

    /// Calculate instability for all components
    pub fn calculate_instability_for_all(
        &self,
        graph: &LocalDependencyGraph,
    ) -> HashMap<ComponentNode, f64> {
        let afferent_couplings = self.afferent_calculator.calculate_for_all_components(graph);
        let efferent_couplings = self.efferent_calculator.calculate_for_all_components(graph);

        let mut instabilities = HashMap::new();

        for (component, &ca) in &afferent_couplings {
            let ce = efferent_couplings.get(component).unwrap_or(&0);
            let ca_f64 = ca as f64;
            let ce_f64 = *ce as f64;

            let instability = if ca_f64 + ce_f64 == 0.0 {
                0.0
            } else {
                ce_f64 / (ca_f64 + ce_f64)
            };

            instabilities.insert(component.clone(), instability);
        }

        debug!("Calculated instability for {} components", instabilities.len());
        instabilities
    }

    /// Calculate Abstractness based on component characteristics
    /// A = Abstract / (Abstract + Concrete)
    /// Values range from 0 (concrete) to 1 (abstract)
    pub fn calculate_abstractness(&self, component: &ComponentNode) -> f64 {
        match component {
            ComponentNode::Class { name, .. } => {
                // Heuristics based on naming conventions and patterns
                if name.contains("Abstract") || name.contains("Base") {
                    0.9
                } else if name.contains("Interface") || name.contains("Protocol") {
                    1.0
                } else if name.contains("Trait") {
                    0.8
                } else if name.contains("Impl") || name.contains("Concrete") {
                    0.1
                } else if name.ends_with("Service") || name.ends_with("Manager") {
                    0.3 // Service classes are often concrete but provide abstraction
                } else {
                    0.2 // Most classes are concrete
                }
            }
            ComponentNode::Function { name, .. } => {
                if name.contains("trait_") || name.contains("interface_") {
                    0.7
                } else {
                    0.1 // Functions are typically concrete
                }
            }
            ComponentNode::Module { path } => {
                if path.contains("trait") || path.contains("interface") || path.contains("abstract") {
                    0.8
                } else if path.contains("impl") || path.contains("concrete") {
                    0.2
                } else {
                    0.5 // Modules are moderately abstract
                }
            }
        }
    }

    /// Calculate Distance from Main Sequence
    /// D = |A + I - 1|
    /// Values range from 0 (on main sequence) to 1 (furthest from main sequence)
    pub fn calculate_distance_from_main_sequence(
        &self,
        graph: &LocalDependencyGraph,
        component: &ComponentNode,
    ) -> f64 {
        let abstractness = self.calculate_abstractness(component);
        let instability = self.calculate_instability(graph, component);

        let distance = (abstractness + instability - 1.0).abs();

        debug!(
            "Component {:?}: A={:.3}, I={:.3}, D={:.3}",
            component, abstractness, instability, distance
        );

        distance
    }

    /// Calculate comprehensive stability metrics for a component
    pub fn calculate_stability_metrics(
        &self,
        graph: &LocalDependencyGraph,
        component: &ComponentNode,
    ) -> StabilityMetrics {
        let afferent_coupling = self.afferent_calculator.calculate_for_component(graph, component);
        let efferent_coupling = self.efferent_calculator.calculate_for_component(graph, component);
        let instability = self.calculate_instability(graph, component);
        let abstractness = self.calculate_abstractness(component);
        let distance_from_main_sequence = self.calculate_distance_from_main_sequence(graph, component);

        StabilityMetrics {
            afferent_coupling,
            efferent_coupling,
            instability,
            abstractness,
            distance_from_main_sequence,
        }
    }

    /// Perform comprehensive stability analysis of the system
    pub fn analyze_system_stability(
        &self,
        graph: &LocalDependencyGraph,
    ) -> StabilityAnalysis {
        let petgraph = graph.get_petgraph();
        let mut stable_components = Vec::new();
        let mut unstable_components = Vec::new();
        let mut balanced_components = Vec::new();
        let mut zone_of_pain = Vec::new();
        let mut zone_of_uselessness = Vec::new();

        debug!("Analyzing stability for {} components", petgraph.node_count());

        for node_index in petgraph.node_indices() {
            if let Some(component) = graph.get_node_from_index(node_index) {
                let metrics = self.calculate_stability_metrics(graph, component);

                // Classify based on instability
                if metrics.instability <= 0.3 {
                    stable_components.push((component.clone(), metrics.clone()));
                } else if metrics.instability >= 0.7 {
                    unstable_components.push((component.clone(), metrics.clone()));
                } else {
                    balanced_components.push((component.clone(), metrics.clone()));
                }

                // Identify problematic zones
                if metrics.instability <= 0.3 && metrics.abstractness <= 0.3 {
                    // Zone of Pain: Stable and Concrete (hard to change)
                    zone_of_pain.push((component.clone(), metrics.clone()));
                } else if metrics.instability >= 0.7 && metrics.abstractness >= 0.7 {
                    // Zone of Uselessness: Unstable and Abstract (unused abstractions)
                    zone_of_uselessness.push((component.clone(), metrics.clone()));
                }
            }
        }

        // Sort by distance from main sequence (higher distance = more problematic)
        let sort_by_distance = |a: &(ComponentNode, StabilityMetrics), b: &(ComponentNode, StabilityMetrics)| {
            b.1.distance_from_main_sequence.partial_cmp(&a.1.distance_from_main_sequence).unwrap_or(std::cmp::Ordering::Equal)
        };

        stable_components.sort_by(sort_by_distance);
        unstable_components.sort_by(sort_by_distance);
        balanced_components.sort_by(sort_by_distance);
        zone_of_pain.sort_by(sort_by_distance);
        zone_of_uselessness.sort_by(sort_by_distance);

        debug!(
            "Stability analysis complete: {} stable, {} unstable, {} balanced, {} in pain zone, {} in uselessness zone",
            stable_components.len(),
            unstable_components.len(),
            balanced_components.len(),
            zone_of_pain.len(),
            zone_of_uselessness.len()
        );

        StabilityAnalysis {
            stable_components,
            unstable_components,
            balanced_components,
            zone_of_pain,
            zone_of_uselessness,
        }
    }

    /// Identify components that violate stability principles
    pub fn identify_stability_violations(
        &self,
        graph: &LocalDependencyGraph,
        max_distance_threshold: f64, // e.g., 0.5
    ) -> Vec<(ComponentNode, StabilityMetrics)> {
        let petgraph = graph.get_petgraph();
        let mut violations = Vec::new();

        for node_index in petgraph.node_indices() {
            if let Some(component) = graph.get_node_from_index(node_index) {
                let metrics = self.calculate_stability_metrics(graph, component);

                if metrics.distance_from_main_sequence > max_distance_threshold {
                    violations.push((component.clone(), metrics));
                }
            }
        }

        // Sort by distance from main sequence (descending)
        violations.sort_by(|a, b| {
            b.1.distance_from_main_sequence.partial_cmp(&a.1.distance_from_main_sequence).unwrap_or(std::cmp::Ordering::Equal)
        });

        debug!(
            "Found {} components violating stability principles (distance > {:.3})",
            violations.len(),
            max_distance_threshold
        );

        violations
    }
}

impl Default for InstabilityCalculator {
    fn default() -> Self {
        Self::new()
    }
}