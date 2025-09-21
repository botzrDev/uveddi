//! AST node types and token definitions

/// Token types for AST construction
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Keyword(String),
    Identifier(String),
    Literal(String),
    Operator(String),
}

/// AST node representation
#[derive(Debug, Clone)]
pub struct AstNode {
    pub node_type: NodeType,
    pub value: String,
    pub children: Vec<AstNode>,
}

impl AstNode {
    /// Creates a new AST node
    pub fn new(node_type: NodeType, value: String) -> Self {
        Self {
            node_type,
            value,
            children: Vec::new(),
        }
    }

    /// Adds a child node
    pub fn add_child(&mut self, child: AstNode) {
        self.children.push(child);
    }

    /// Returns the depth of this node in the tree
    pub fn depth(&self) -> usize {
        if self.children.is_empty() {
            1
        } else {
            1 + self.children.iter().map(|c| c.depth()).max().unwrap_or(0)
        }
    }

    /// Returns the total number of nodes in this subtree
    pub fn node_count(&self) -> usize {
        1 + self.children.iter().map(|c| c.node_count()).sum::<usize>()
    }

    /// Visits all nodes in the tree
    pub fn visit<F>(&self, visitor: &mut F)
    where
        F: FnMut(&AstNode),
    {
        visitor(self);
        for child in &self.children {
            child.visit(visitor);
        }
    }
}

#[derive(Debug, Clone)]
pub enum NodeType {
    Keyword(String),
    Identifier(String),
    Literal(String),
    Operator(String),
    Block,
    Expression,
    Statement,
    Function,
    Class,
}
