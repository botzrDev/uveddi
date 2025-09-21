use crate::analysis::graph::dependency::{ComponentNode, LocalDependencyType};

/// Represents a dependency relationship between components
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Dependency {
    pub from_component: ComponentNode,
    pub to_component: ComponentNode,
    pub dependency_type: LocalDependencyType,
    pub line_number: Option<u32>,
    pub strength: DependencyStrength,
}

/// Strength of dependency coupling
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DependencyStrength {
    Weak,   // Loose coupling (interfaces, abstractions)
    Medium, // Moderate coupling (data structures, method calls)
    Strong, // Tight coupling (direct field access, inheritance)
}

/// Coupling metrics for a component
#[derive(Debug, Clone, Default)]
pub struct CouplingMetrics {
    pub fan_out: usize, // Number of components this depends on
    pub fan_in: usize,  // Number of components depending on this
    pub cbo: usize,     // Coupling Between Objects
    pub rfc: usize,     // Response for Class
    pub lcom: f64,      // Lack of Cohesion in Methods
}

/// Language-specific coupling thresholds
#[derive(Debug, Clone)]
pub struct CouplingThresholds {
    pub fan_out_warning: usize,
    pub fan_out_critical: usize,
    pub cbo_warning: usize,
    pub cbo_critical: usize,
    pub rfc_warning: usize,
    pub rfc_critical: usize,
    pub production_multiplier: f64,
    pub test_multiplier: f64,
    pub framework_multiplier: f64,
}

impl Default for CouplingThresholds {
    fn default() -> Self {
        Self {
            fan_out_warning: 7,
            fan_out_critical: 12,
            cbo_warning: 6,
            cbo_critical: 10,
            rfc_warning: 15,
            rfc_critical: 25,
            production_multiplier: 1.0,
            test_multiplier: 1.5,
            framework_multiplier: 2.0,
        }
    }
}
