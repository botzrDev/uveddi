//! # AST Visitor Pattern
//!
//! Provides consistent AST traversal patterns for all detectors.
//! Implements the visitor pattern for tree-sitter nodes.

// Tree-sitter imports with feature gate
#[cfg(not(feature = "tree-sitter"))]
use crate::ast::tree_sitter::{Node, Tree};
#[cfg(feature = "tree-sitter")]
use tree_sitter::{Node, Tree};

/// Result of visiting a node
#[derive(Debug, PartialEq)]
pub enum VisitResult {
    /// Continue traversing child nodes
    Continue,
    /// Skip child nodes but continue traversing siblings
    Skip,
    /// Stop traversing entirely
    Stop,
}

/// Trait for AST visitors
pub trait AstVisitor {
    /// Visit a function node
    fn visit_function(&mut self, node: &Node) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit a class/struct node
    fn visit_class(&mut self, node: &Node) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit a module node
    fn visit_module(&mut self, node: &Node) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit a variable declaration
    fn visit_variable(&mut self, node: &Node) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit an import statement
    fn visit_import(&mut self, node: &Node) -> VisitResult {
        VisitResult::Continue
    }

    /// Visit any other node type
    fn visit_node(&mut self, node: &Node) -> VisitResult {
        VisitResult::Continue
    }
}

/// Tree walker that applies visitor pattern
pub struct TreeWalker;

impl TreeWalker {
    /// Walk a tree using the visitor pattern
    pub fn walk<V: AstVisitor>(
        visitor: &mut V,
        tree: &Tree,
        source: &str,
    ) -> Result<(), WalkError> {
        let root = tree.root_node();
        Self::walk_node(visitor, &root, source)
    }

    /// Walk a specific node and its children
    fn walk_node<V: AstVisitor>(
        visitor: &mut V,
        node: &Node,
        source: &str,
    ) -> Result<(), WalkError> {
        let result = match node.kind() {
            "function_declaration" | "function_definition" => visitor.visit_function(node),
            "class_declaration" | "struct_item" | "impl_item" => visitor.visit_class(node),
            "module" | "module_item" => visitor.visit_module(node),
            "variable_declaration" | "let_declaration" => visitor.visit_variable(node),
            "import_declaration" | "use_declaration" => visitor.visit_import(node),
            _ => visitor.visit_node(node),
        };

        match result {
            VisitResult::Continue => {
                // Visit all children
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if let Err(e) = Self::walk_node(visitor, &child, source) {
                        return Err(e);
                    }
                }
                Ok(())
            }
            VisitResult::Skip => Ok(()),
            VisitResult::Stop => Err(WalkError::Stopped),
        }
    }
}

/// Errors that can occur during tree walking
#[derive(Debug, thiserror::Error)]
pub enum WalkError {
    #[error("Tree walking was stopped by visitor")]
    Stopped,

    #[error("Invalid node structure")]
    InvalidNode,
}

/// Base visitor implementation for common patterns
pub struct BaseVisitor {
    /// Collected issues
    pub issues: Vec<String>,
}

impl BaseVisitor {
    /// Create a new base visitor
    pub fn new() -> Self {
        Self { issues: Vec::new() }
    }

    /// Add an issue to the collection
    pub fn add_issue(&mut self, message: String) {
        self.issues.push(message);
    }
}

impl AstVisitor for BaseVisitor {
    // Default implementations are provided by the trait
}
