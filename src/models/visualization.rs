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

    // Rust-specific types
    /// A Rust module with visibility information
    RustModule { is_public: bool },
    /// A Rust struct with field information
    RustStruct { fields: Vec<FieldInfo> },
    /// A Rust enum with variant information
    RustEnum { variants: Vec<VariantInfo> },
    /// A Rust trait with method signatures
    RustTrait { methods: Vec<MethodSignature> },
    /// A Rust impl block with implementation details
    RustImpl { 
        target_type: String, 
        trait_impl: Option<String> 
    },
    /// A Rust function with signature information
    RustFunction {
        is_async: bool,
        is_const: bool,
        visibility: Visibility,
    },

    // Python-specific types
    /// A Python class with inheritance and method information
    PythonClass { 
        bases: Vec<String>,
        methods: Vec<MethodInfo>,
        is_abstract: bool,
    },
    /// A Python method with classification
    PythonMethod {
        is_static: bool,
        is_class_method: bool,
        is_property: bool,
    },
    /// A Python function
    PythonFunction {
        is_async: bool,
        decorators: Vec<String>,
    },

    // JavaScript-specific types
    /// An ES module with export information
    JavaScriptEsModule { exports: Vec<ExportInfo> },
    /// A JavaScript class with inheritance
    JavaScriptClass { extends: Option<String> },
    /// A JavaScript function with characteristics
    JavaScriptFunction {
        is_async: bool,
        is_generator: bool,
        is_arrow: bool,
    },

    // TypeScript-specific types (extends JavaScript types)
    /// A TypeScript interface
    TypeScriptInterface { methods: Vec<MethodSignature> },
    /// A TypeScript type alias
    TypeScriptType { type_definition: String },
    /// A TypeScript namespace
    TypeScriptNamespace { exports: Vec<String> },

    // Legacy generic types for backward compatibility
    /// A generic class or struct definition
    Class,
    /// A generic function or method
    Function,
}

/// Field information for structs and classes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct FieldInfo {
    pub name: String,
    pub field_type: String,
    pub visibility: Visibility,
    pub is_optional: bool,
}

/// Variant information for enums
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct VariantInfo {
    pub name: String,
    pub fields: Option<Vec<FieldInfo>>,
    pub discriminant: Option<String>,
}

/// Method signature information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct MethodSignature {
    pub name: String,
    pub parameters: Vec<ParameterInfo>,
    pub return_type: Option<String>,
    pub visibility: Visibility,
}

/// Method information with implementation details
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct MethodInfo {
    pub name: String,
    pub signature: MethodSignature,
    pub is_virtual: bool,
    pub is_override: bool,
}

/// Parameter information for methods and functions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ParameterInfo {
    pub name: String,
    pub param_type: String,
    pub is_optional: bool,
    pub default_value: Option<String>,
}

/// Export information for modules
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ExportInfo {
    pub name: String,
    pub export_type: ExportType,
    pub is_default: bool,
}

/// Types of exports
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ExportType {
    Function,
    Class,
    Variable,
    Type,
    Namespace,
}

/// Visibility levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Visibility {
    Public,
    Private,
    Protected,
    Internal,
    Package,
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

    /// Check if this component type is language-specific
    pub fn is_language_specific(&self) -> bool {
        matches!(self, 
            ComponentType::RustModule { .. } |
            ComponentType::RustStruct { .. } |
            ComponentType::RustEnum { .. } |
            ComponentType::RustTrait { .. } |
            ComponentType::RustImpl { .. } |
            ComponentType::RustFunction { .. } |
            ComponentType::PythonClass { .. } |
            ComponentType::PythonMethod { .. } |
            ComponentType::PythonFunction { .. } |
            ComponentType::JavaScriptEsModule { .. } |
            ComponentType::JavaScriptClass { .. } |
            ComponentType::JavaScriptFunction { .. } |
            ComponentType::TypeScriptInterface { .. } |
            ComponentType::TypeScriptType { .. } |
            ComponentType::TypeScriptNamespace { .. }
        )
    }

    /// Get the programming language for language-specific types
    pub fn language(&self) -> Option<&'static str> {
        match self {
            ComponentType::RustModule { .. } |
            ComponentType::RustStruct { .. } |
            ComponentType::RustEnum { .. } |
            ComponentType::RustTrait { .. } |
            ComponentType::RustImpl { .. } |
            ComponentType::RustFunction { .. } => Some("rust"),
            
            ComponentType::PythonClass { .. } |
            ComponentType::PythonMethod { .. } |
            ComponentType::PythonFunction { .. } => Some("python"),
            
            ComponentType::JavaScriptEsModule { .. } |
            ComponentType::JavaScriptClass { .. } |
            ComponentType::JavaScriptFunction { .. } => Some("javascript"),
            
            ComponentType::TypeScriptInterface { .. } |
            ComponentType::TypeScriptType { .. } |
            ComponentType::TypeScriptNamespace { .. } => Some("typescript"),
            
            _ => None,
        }
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
            
            // Rust-specific icons
            ComponentType::RustModule { .. } => "📦",
            ComponentType::RustStruct { .. } => "🏗️",
            ComponentType::RustEnum { .. } => "🔀",
            ComponentType::RustTrait { .. } => "🎭",
            ComponentType::RustImpl { .. } => "⚙️",
            ComponentType::RustFunction { .. } => "⚡",
            
            // Python-specific icons
            ComponentType::PythonClass { .. } => "🐍",
            ComponentType::PythonMethod { .. } => "🔧",
            ComponentType::PythonFunction { .. } => "⚡",
            
            // JavaScript/TypeScript-specific icons
            ComponentType::JavaScriptEsModule { .. } => "📦",
            ComponentType::JavaScriptClass { .. } => "🟨",
            ComponentType::JavaScriptFunction { .. } => "⚡",
            ComponentType::TypeScriptInterface { .. } => "🔷",
            ComponentType::TypeScriptType { .. } => "🏷️",
            ComponentType::TypeScriptNamespace { .. } => "📦",
            
            // Generic fallbacks
            ComponentType::Class => "🏛️",
            ComponentType::Function => "⚡",
            _ => "📦",
        }
    }

    /// Get complexity score for this component type (used for sizing in diagrams)
    pub fn complexity_score(&self) -> u32 {
        match self {
            // High complexity types
            ComponentType::RustImpl { .. } => 8,
            ComponentType::PythonClass { methods, .. } => 5 + methods.len() as u32,
            ComponentType::RustTrait { methods } => 5 + methods.len() as u32,
            ComponentType::TypeScriptInterface { methods } => 5 + methods.len() as u32,
            
            // Medium complexity types
            ComponentType::RustStruct { fields } => 3 + fields.len() as u32,
            ComponentType::RustEnum { variants } => 3 + variants.len() as u32,
            ComponentType::JavaScriptClass { .. } => 5,
            
            // Lower complexity types
            ComponentType::RustFunction { .. } => 2,
            ComponentType::PythonFunction { .. } => 2,
            ComponentType::JavaScriptFunction { .. } => 2,
            ComponentType::Function => 2,
            
            // Infrastructure components
            ComponentType::Service => 6,
            ComponentType::Database => 4,
            ComponentType::MessageBroker => 4,
            
            // Simple types
            _ => 1,
        }
    }

    /// Get diagram styling class based on component type
    pub fn style_class(&self) -> &'static str {
        match self {
            ComponentType::Service => "service-node",
            ComponentType::Database => "data-store",
            ComponentType::Cache => "data-store",
            ComponentType::User => "external-user",
            ComponentType::ExternalSystem => "external-system",
            ComponentType::ApiEndpoint => "api-endpoint",
            ComponentType::MessageBroker => "message-broker",
            
            // Language-specific styles
            ComponentType::RustModule { .. } |
            ComponentType::RustStruct { .. } |
            ComponentType::RustEnum { .. } |
            ComponentType::RustTrait { .. } |
            ComponentType::RustImpl { .. } |
            ComponentType::RustFunction { .. } => "rust-component",
            
            ComponentType::PythonClass { .. } |
            ComponentType::PythonMethod { .. } |
            ComponentType::PythonFunction { .. } => "python-component",
            
            ComponentType::JavaScriptEsModule { .. } |
            ComponentType::JavaScriptClass { .. } |
            ComponentType::JavaScriptFunction { .. } => "js-component",
            
            ComponentType::TypeScriptInterface { .. } |
            ComponentType::TypeScriptType { .. } |
            ComponentType::TypeScriptNamespace { .. } => "ts-component",
            
            _ => "generic-component",
        }
    }

    /// Get a string representation of the component type
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
            
            ComponentType::RustModule { .. } => "rust_module",
            ComponentType::RustStruct { .. } => "rust_struct",
            ComponentType::RustEnum { .. } => "rust_enum",
            ComponentType::RustTrait { .. } => "rust_trait",
            ComponentType::RustImpl { .. } => "rust_impl",
            ComponentType::RustFunction { .. } => "rust_function",
            
            ComponentType::PythonClass { .. } => "python_class",
            ComponentType::PythonMethod { .. } => "python_method",
            ComponentType::PythonFunction { .. } => "python_function",
            
            ComponentType::JavaScriptEsModule { .. } => "javascript_es_module",
            ComponentType::JavaScriptClass { .. } => "javascript_class",
            ComponentType::JavaScriptFunction { .. } => "javascript_function",
            
            ComponentType::TypeScriptInterface { .. } => "typescript_interface",
            ComponentType::TypeScriptType { .. } => "typescript_type",
            ComponentType::TypeScriptNamespace { .. } => "typescript_namespace",
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
