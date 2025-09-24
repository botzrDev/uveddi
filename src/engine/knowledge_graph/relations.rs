//! # Knowledge Graph Relations
//!
//! Defines the types of relationships that can exist between code elements.

/// A relationship between two nodes in the knowledge graph
#[derive(Debug, Clone)]
pub struct GraphRelation {
    /// Source node ID
    pub from: String,

    /// Target node ID
    pub to: String,

    /// Type of relationship
    pub relation_type: RelationType,

    /// File where this relation was defined
    pub file_path: String,
}

/// Types of relationships between code elements
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RelationType {
    /// One module imports another
    Imports,

    /// Class extends another class
    Extends,

    /// Class implements an interface/trait
    Implements,

    /// Code uses/references another symbol
    Uses,

    /// Function calls another function
    Calls,

    /// Variable is assigned a value
    Assigns,

    /// Generic type parameter relationship
    TypeParameter,

    /// Inheritance relationship
    Inherits,

    /// Composition relationship
    Composes,

    /// Dependency relationship
    Depends,
}

impl RelationType {
    /// Get all relation types
    pub fn all() -> Vec<RelationType> {
        vec![
            RelationType::Imports,
            RelationType::Extends,
            RelationType::Implements,
            RelationType::Uses,
            RelationType::Calls,
            RelationType::Assigns,
            RelationType::TypeParameter,
            RelationType::Inherits,
            RelationType::Composes,
            RelationType::Depends,
        ]
    }

    /// Check if this is a direct dependency relationship
    pub fn is_direct_dependency(&self) -> bool {
        matches!(
            self,
            RelationType::Imports | RelationType::Uses | RelationType::Calls
        )
    }

    /// Check if this is an inheritance relationship
    pub fn is_inheritance(&self) -> bool {
        matches!(
            self,
            RelationType::Extends | RelationType::Implements | RelationType::Inherits
        )
    }

    /// Get the inverse relation type
    pub fn inverse(&self) -> Option<RelationType> {
        match self {
            RelationType::Extends => Some(RelationType::Inherits),
            RelationType::Inherits => Some(RelationType::Extends),
            RelationType::Composes => Some(RelationType::Depends),
            RelationType::Depends => Some(RelationType::Composes),
            _ => None,
        }
    }

    /// Get a human-readable description of the relation
    pub fn description(&self) -> &'static str {
        match self {
            RelationType::Imports => "imports",
            RelationType::Extends => "extends",
            RelationType::Implements => "implements",
            RelationType::Uses => "uses",
            RelationType::Calls => "calls",
            RelationType::Assigns => "assigns",
            RelationType::TypeParameter => "type parameter of",
            RelationType::Inherits => "inherited by",
            RelationType::Composes => "composed of",
            RelationType::Depends => "depends on",
        }
    }
}

impl std::fmt::Display for RelationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description())
    }
}