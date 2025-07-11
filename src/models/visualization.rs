//! Visualization Models for Architectural Components and Diagrams
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

/// Classification of architectural components with language-specific variants
///
/// Enhanced component types that provide detailed language-specific information
/// to improve diagram accuracy and architectural analysis.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ComponentType {
    // Generic types
    /// A module or namespace
    Module,
    /// A service in a microservices architecture
    Service,
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
    /// A class or struct
    Class,
    /// A function or method
    Function,
    // Language-specific types
    /// Rust struct with field info
    RustStruct {
        fields: Vec<FieldInfo>,
    },
    /// Rust module with visibility
    RustModule {
        is_public: bool,
    },
    /// Rust function with signature
    RustFunction {
        signature: MethodSignature,
        is_async: bool,
        is_const: bool,
        visibility: Visibility,
    },
    /// Python class with bases and methods
    PythonClass {
        bases: Vec<String>,
        methods: Vec<MethodSignature>,
        is_abstract: bool,
    },
    /// JavaScript ES module with exports
    JavaScriptEsModule {
        exports: Vec<String>,
    },
}

/// Represents a dependency relationship between components
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Dependency {
    /// Source component of the dependency
    pub from: DependencyNode,
    /// Target component of the dependency
    pub to: DependencyNode,
    /// Type of dependency relationship
    pub dependency_type: DependencyType,
    /// Kind of dependency (alias for dependency_type for backwards compatibility)
    pub kind: Option<DependencyType>,
    /// Strength or weight of the dependency
    pub weight: Option<f64>,
    /// Target component ID for backwards compatibility
    pub target_component_id: Option<String>,
    /// Additional properties for backwards compatibility
    pub properties: Option<std::collections::HashMap<String, String>>,
}

/// Node in a dependency relationship
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct DependencyNode {
    /// Unique identifier for the node
    pub id: String,
    /// Human-readable name
    pub name: String,
}

/// Types of dependency relationships
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DependencyType {
    /// Direct function or method call
    Call,
    /// Direct function or method call (alias for backwards compatibility)
    Calls,
    /// Import or use statement
    Import,
    /// Import or use statement (alias for backwards compatibility)
    Imports,
    /// Class inheritance relationship
    Inheritance,
    /// Interface implementation
    Implementation,
    /// Data flow dependency
    DataFlow,
    /// Configuration dependency
    Configuration,
}

/// Metrics associated with an architectural component
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComponentMetrics {
    /// Lines of code in this component
    pub lines_of_code: Option<u32>,
    /// Cyclomatic complexity
    pub complexity: Option<f64>,
    /// Number of incoming dependencies
    pub afferent_coupling: u32,
    /// Number of outgoing dependencies
    pub efferent_coupling: u32,
    /// Coupling between objects metric
    pub coupling_between_objects: Option<f64>,
    /// Number of public methods/functions
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
    /// Graph diagram for general graph visualization
    Graph,
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

impl ComponentType {
    /// Convert ComponentType to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            ComponentType::Module => "module",
            ComponentType::Service => "service",
            ComponentType::Database => "database",
            ComponentType::ApiEndpoint => "api_endpoint",
            ComponentType::Configuration => "configuration",
            ComponentType::ExternalSystem => "external_system",
            ComponentType::User => "user",
            ComponentType::MessageBroker => "message_broker",
            ComponentType::Cache => "cache",
            ComponentType::Class => "class",
            ComponentType::Function => "function",
            ComponentType::RustStruct { .. } => "rust_struct",
            ComponentType::RustModule { .. } => "rust_module",
            ComponentType::RustFunction { .. } => "rust_function",
            ComponentType::PythonClass { .. } => "python_class",
            ComponentType::JavaScriptEsModule { .. } => "js_es_module",
        }
    }

    /// Get the programming language associated with this component type
    pub fn language(&self) -> Option<String> {
        match self {
            ComponentType::RustStruct { .. } |
            ComponentType::RustModule { .. } |
            ComponentType::RustFunction { .. } => Some("rust".to_string()),
            ComponentType::PythonClass { .. } => Some("python".to_string()),
            ComponentType::JavaScriptEsModule { .. } => Some("javascript".to_string()),
            _ => None,
        }
    }

    /// Check if this component type is language-specific
    pub fn is_language_specific(&self) -> bool {
        matches!(self,
            ComponentType::RustStruct { .. } |
            ComponentType::RustModule { .. } |
            ComponentType::RustFunction { .. } |
            ComponentType::PythonClass { .. } |
            ComponentType::JavaScriptEsModule { .. }
        )
    }

    /// Calculate a complexity score for this component type
    pub fn complexity_score(&self) -> u32 {
        match self {
            ComponentType::Module => 10,
            ComponentType::Service => 50,
            ComponentType::Database => 30,
            ComponentType::ApiEndpoint => 20,
            ComponentType::Configuration => 5,
            ComponentType::ExternalSystem => 40,
            ComponentType::User => 1,
            ComponentType::MessageBroker => 35,
            ComponentType::Cache => 15,
            ComponentType::Class => 25,
            ComponentType::Function => 10,
            ComponentType::RustStruct { fields } => 10 + fields.len() as u32 * 2,
            ComponentType::RustModule { .. } => 15,
            ComponentType::RustFunction { .. } => 12,
            ComponentType::PythonClass { methods, .. } => 25 + methods.len() as u32 * 3,
            ComponentType::JavaScriptEsModule { exports } => 20 + exports.len() as u32 * 2,
        }
    }
}

impl Default for StyleConfig {
    fn default() -> Self {
        Self {
            fill_color: "#f9f9f9".to_string(),
            stroke_color: "#333333".to_string(),
            stroke_width: 1,
            text_color: Some("#000000".to_string()),
            css_classes: vec![],
        }
    }
}

/// Visibility of a field or method
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Visibility {
    Public,
    Private,
    Protected,
}

/// Information about a struct/class field
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct FieldInfo {
    pub name: String,
    pub field_type: String,
    pub visibility: Visibility,
    pub is_optional: bool,
}

/// Information about a function/method parameter
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ParameterInfo {
    pub name: String,
    pub param_type: String,
    pub is_optional: bool,
}

/// Information about a method or function signature
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct MethodSignature {
    pub name: String,
    pub parameters: Vec<ParameterInfo>,
    pub return_type: Option<String>,
    pub visibility: Visibility,
    pub is_async: bool,
}

/// Result of diagram generation containing the rendered diagram and metadata
#[derive(Debug, Clone)]
pub struct DiagramResult {
    /// Type of diagram generated
    pub diagram_type: DiagramType,
    /// Generated Mermaid.js source code
    pub mermaid_src: String,
    /// Components included in the diagram
    pub components: Vec<Uuid>,
    /// Optional path to generated image file
    pub image_path: Option<PathBuf>,
    /// Timestamp when diagram was generated
    pub generated_at: chrono::DateTime<chrono::Utc>,
    /// Optional validation metrics
    pub validation_metrics: Option<ValidationMetrics>,
}

impl DiagramResult {
    pub fn new(diagram_type: DiagramType, mermaid_src: String, components: Vec<Uuid>) -> Self {
        Self {
            diagram_type,
            mermaid_src,
            components,
            image_path: None,
            generated_at: chrono::Utc::now(),
            validation_metrics: None,
        }
    }
}

/// Placeholder documentation for public items