//! Mermaid.js diagram generation for code dependencies and anti-patterns

use crate::models::visualization::Dependency;
use crate::database::models::ArchitecturalIssue;
use std::collections::HashSet;

/// Placeholder documentation for public items
///
/// Generates a Mermaid.js diagram representing code dependencies and highlights issues.
///
/// This function creates a graph in Mermaid.js syntax, showing modules as nodes and dependencies as edges.
/// Nodes associated with architectural issues are visually highlighted.
///
/// # Arguments
///
/// * `deps` - Slice of `Dependency` objects representing code dependencies.
/// * `issues` - Slice of `ArchitecturalIssue` objects to highlight in the diagram.
///
/// # Returns
///
/// * `String` - Mermaid.js formatted diagram as a string.
///
/// # Example
/// ```rust
/// use uveddi::report::diagrams::generate_mermaid_diagram;
/// let diagram = generate_mermaid_diagram(&deps, &issues);
/// println!("{}", diagram);
/// ```
pub fn generate_mermaid_diagram(deps: &[Dependency], issues: &[ArchitecturalIssue]) -> String {
    let mut output = String::from("```mermaid\ngraph TD;\n");
    let mut processed_nodes = HashSet::new();

    for dep in deps {
        if processed_nodes.insert(&dep.from) {
            output.push_str(&format!("    {}[{}];\n", dep.from.id, dep.from.name));
        }
        if processed_nodes.insert(&dep.to) {
            output.push_str(&format!("    {}[{}];\n", dep.to.id, dep.to.name));
        }
        output.push_str(&format!("    {} --> {};\n", dep.from.id, dep.to.id));
    }

    // Highlight nodes with issues
    for issue in issues {
        // Use file_path as the node identifier for highlighting
        let node_id = &issue.file_path;
        output.push_str(&format!("    \"{}\"[\"{}\" class=\"issue\"];\n", node_id, node_id));
    }
    output.push_str("classDef issue fill:#f96,stroke:#333,stroke-width:2px;\n");
    output.push_str("```\n");
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::visualization::Dependency;
    use crate::database::models::ArchitecturalIssue;

    #[test]
    fn test_generate_mermaid_diagram() {
        let deps = vec![
            Dependency {
                from: crate::models::visualization::DependencyNode {
                    id: "mod1".to_string(), 
                    name: "Module1".to_string(),
                },
                to: crate::models::visualization::DependencyNode {
                    id: "mod2".to_string(), 
                    name: "Module2".to_string(),
                },
                dependency_type: crate::models::visualization::DependencyType::Imports,
                kind: Some(crate::models::visualization::DependencyType::Imports),
                weight: None,
                target_component_id: Some("mod2".to_string()),
                properties: Some(std::collections::HashMap::new()),
            },
        ];
        let issues = vec![ArchitecturalIssue { file_path: "mod1".to_string(), ..Default::default() }];
        let diagram = generate_mermaid_diagram(&deps, &issues);
        assert!(diagram.contains("mod1 --> mod2"));
        assert!(diagram.contains("classDef issue"));
    }
}
