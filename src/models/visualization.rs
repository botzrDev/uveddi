//! Visualization Models for Arquitectual Components and Diagrams
//!
//! This module provides the core data models for the enhanced visualization system,
//! implementing the Software Architecture Model (SAM) components and diagram specifications
//! as outlined in the architectural visualization enhancement requirements.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

/// Represents an architectural component extracted from the codebase
///
/// This is a core entity in the Software Architecture Model (SAM) that represents
/// discrete architectural units such as modules, services, classes, or functions.
/// Components are extracted from AST analysis and form the nodes in architectural diagrams.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArchitecturalComponent {
    /// Unique identifier for this component
    pub component_id: Uuid,
    /// Human-readable name of the component (e.g., UserService, AuthMiddleware)
    pub name: String,
    /// File path where this component is defined
    pub file_path: PathBuf,
    /// Type classification of this component
    pub component_type: ComponentType,
    /// List of dependencies this component has on other components
    pub dependencies: Vec<Dependency>,
    /// Metrics associated with this component
    pub metrics: ComponentMetrics,
    /// Hierarchical grouping (e.g., microservice name, package name)
    pub group: Option<String>,
}

/// Classification of architectural components
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ComponentType {
    /// A module or namespace
    Module,
    /// A service in a microservices architecture
    Service,
    /// A class or struct definition
    Class,
    /// A function or method
    Function,
    /// A database or persistent storage
    Database,
    /// An API endpoint
    ApiEndpoint,
    /// A configuration component
    Configuration,
    /// An external system or service
    ExternalSystem,
    /// A user or actor in the system
    User,
    /// A message broker or queue
    MessageBroker,
    /// A cache layer
    Cache,
}

/// Dependency relationship between components
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Dependency {
    /// Target component this dependency points to
    pub target_component_id: Uuid,
    /// Type of dependency relationship
    pub dependency_type: DependencyType,
    /// Additional metadata about the dependency
    pub properties: HashMap<String, String>,
}

/// Types of dependency relationships
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DependencyType {
    /// Direct method or function call
    Calls,
    /// Inheritance relationship
    InheritsFrom,
    /// Interface implementation
    Implements,
    /// Data access (reads from)
    ReadsDataFrom,
    /// Data modification (writes to)
    WritesToData,
    /// Event publishing
    PublishesToTopic,
    /// Event consumption
    ConsumesFromTopic,
    /// HTTP API call
    HttpCall,
    /// Import or use statement
    Imports,
    /// Composition relationship
    Contains,
}

/// Metrics for architectural components
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComponentMetrics {
    /// Lines of code in this component
    pub lines_of_code: Option<u32>,
    /// Cyclomatic complexity
    pub complexity: Option<f64>,
    /// Number of incoming dependencies (fan-in)
    pub afferent_coupling: u32,
    /// Number of outgoing dependencies (fan-out)
    pub efferent_coupling: u32,
    /// Coupling between objects (CBO)
    pub coupling_between_objects: Option<u32>,
    /// Number of public methods
    pub public_methods: Option<u32>,
}

/// Specification for generating diagrams from architectural components
///
/// Defines how to transform the Software Architecture Model (SAM) into
/// visual diagrams using template-based generation with severity-based styling.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagramSpec {
    /// Unique identifier for this diagram specification
    pub spec_id: Uuid,
    /// Anti-pattern type this diagram spec applies to
    pub anti_pattern_type_id: i64,
    /// Type of diagram to generate
    pub diagram_type: DiagramType,
    /// Template for generating Mermaid.js code
    pub mermaid_template: String,
    /// Style configurations based on severity levels
    pub severity_styles: HashMap<String, StyleConfig>,
    /// Layout preference for the diagram
    pub layout: DiagramLayout,
}

/// Types of diagrams that can be generated
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DiagramType {
    /// Component diagram showing architectural components and relationships
    Component,
    /// Sequence diagram showing interaction flows
    Sequence,
    /// Class diagram showing object-oriented relationships
    Class,
    /// Dependency graph showing module dependencies
    Dependency,
    /// Data flow diagram showing information movement
    DataFlow,
    /// Deployment diagram showing infrastructure mapping
    Deployment,
    /// Security view highlighting trust boundaries
    Security,
}

/// Layout options for diagram rendering
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DiagramLayout {
    /// Top-to-bottom layout
    TopDown,
    /// Left-to-right layout
    LeftRight,
    /// Bottom-to-top layout
    BottomUp,
    /// Right-to-left layout
    RightLeft,
}

/// Style configuration for diagram elements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleConfig {
    /// Fill color for the element
    pub fill_color: String,
    /// Border color
    pub stroke_color: String,
    /// Border width
    pub stroke_width: u32,
    /// Text color
    pub text_color: Option<String>,
    /// Additional CSS classes
    pub css_classes: Vec<String>,
}

/// Complete visualization pipeline stages
#[derive(Debug, Clone)]
pub enum DiagramPipeline {
    /// Extract components from AST
    Extract,
    /// Transform components using diagram specifications
    Transform(DiagramSpec),
    /// Generate Mermaid.js code
    Generate,
    /// Render to image (optional)
    Render,
}

/// Diagram metadata for inclusion in reports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagramMetadata {
    /// Type of diagram
    pub diagram_type: DiagramType,
    /// Generated Mermaid.js source code
    pub mermaid_src: String,
    /// Optional path to rendered image
    pub image_path: Option<PathBuf>,
    /// Timestamp when diagram was generated
    pub generated_at: chrono::DateTime<chrono::Utc>,
    /// Components included in this diagram
    pub components: Vec<Uuid>,
    /// Validation metrics for diagram accuracy
    pub validation_metrics: Option<ValidationMetrics>,
}

/// Validation metrics for diagram accuracy assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationMetrics {
    /// Structural fidelity metrics
    pub node_precision: f64,
    pub node_recall: f64,
    pub edge_precision: f64,
    pub edge_recall: f64,
    /// Graph edit distance
    pub graph_edit_distance: Option<f64>,
    /// Semantic coherence score
    pub semantic_coherence_score: Option<f64>,
    /// Overall quality score
    pub overall_quality: f64,
}

impl Default for ComponentMetrics {
    fn default() -> Self {
        Self {
            lines_of_code: None,
            complexity: None,
            afferent_coupling: 0,
            efferent_coupling: 0,
            coupling_between_objects: None,
            public_methods: None,
        }
    }
}

impl Default for StyleConfig {
    fn default() -> Self {
        Self {
            fill_color: "#f9f9f9".to_string(),
            stroke_color: "#333".to_string(),
            stroke_width: 1,
            text_color: None,
            css_classes: Vec::new(),
        }
    }
}

impl DiagramLayout {
    /// Convert to Mermaid.js syntax
    pub fn to_mermaid_syntax(&self) -> &'static str {
        match self {
            DiagramLayout::TopDown => "TD",
            DiagramLayout::LeftRight => "LR",
            DiagramLayout::BottomUp => "BT",
            DiagramLayout::RightLeft => "RL",
        }
    }
}

impl ComponentType {
    /// Check if this component type represents a data store
    pub fn is_data_store(&self) -> bool {
        matches!(self, ComponentType::Database | ComponentType::Cache)
    }

    /// Check if this component type represents an external boundary
    pub fn is_external(&self) -> bool {
        matches!(self, ComponentType::ExternalSystem | ComponentType::User)
    }

    /// Get icon representation for diagram rendering
    pub fn icon(&self) -> &'static str {
        match self {
            ComponentType::Service => "🔧",
            ComponentType::Database => "🗄️",
            ComponentType::User => "👤",
            ComponentType::ApiEndpoint => "🌐",
            ComponentType::MessageBroker => "📨",
            ComponentType::Cache => "⚡",
            ComponentType::ExternalSystem => "🔗",
            _ => "📦",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_architectural_component_creation() {
        let component = ArchitecturalComponent {
            component_id: Uuid::new_v4(),
            name: "UserService".to_string(),
            file_path: PathBuf::from("src/services/user.rs"),
            component_type: ComponentType::Service,
            dependencies: vec![],
            metrics: ComponentMetrics::default(),
            group: Some("auth".to_string()),
        };

        assert_eq!(component.name, "UserService");
        assert_eq!(component.component_type, ComponentType::Service);
    }

    #[test]
    fn test_diagram_layout_mermaid_syntax() {
        assert_eq!(DiagramLayout::TopDown.to_mermaid_syntax(), "TD");
        assert_eq!(DiagramLayout::LeftRight.to_mermaid_syntax(), "LR");
        assert_eq!(DiagramLayout::BottomUp.to_mermaid_syntax(), "BT");
        assert_eq!(DiagramLayout::RightLeft.to_mermaid_syntax(), "RL");
    }

    #[test]
    fn test_component_type_classification() {
        assert!(ComponentType::Database.is_data_store());
        assert!(ComponentType::Cache.is_data_store());
        assert!(!ComponentType::Service.is_data_store());

        assert!(ComponentType::ExternalSystem.is_external());
        assert!(ComponentType::User.is_external());
        assert!(!ComponentType::Service.is_external());
    }
}
